//! Bridge Session
//!
//! Manages a persistent Node.js bridge process for a single conversation.
//! Handles JSON protocol communication over stdin/stdout.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Request sent to the bridge process
#[derive(Debug, Serialize)]
pub struct BridgeRequest {
    /// Type of request
    #[serde(rename = "type")]
    pub request_type: String,
    /// Message content (for "message" type)
    pub content: Option<String>,
    /// Model to use (optional)
    pub model: Option<String>,
}

/// Response received from the bridge process
#[derive(Debug, Deserialize)]
pub struct BridgeResponse {
    /// Status of the response
    pub status: String,
    /// Response data (for success)
    pub data: Option<String>,
    /// Error message (for error)
    pub message: Option<String>,
}

/// Handle to a persistent bridge subprocess
///
/// Each BridgeSession manages one Node.js bridge process that maintains
/// conversation state for a single conversation.
pub struct BridgeSession {
    /// Child process handle (without stdin/stdout/stderr)
    child: Mutex<Option<Child>>,
    /// Stdin handle for sending requests
    stdin: Mutex<Option<ChildStdin>>,
    /// Stdout handle for receiving responses
    stdout: Mutex<Option<BufReader<ChildStdout>>>,
    /// Stderr handle for reading error messages
    stderr: Mutex<Option<tokio::task::JoinHandle<String>>>,
    /// Path to the bridge script (kept for reference/debugging)
    #[allow(dead_code)]
    bridge_script_path: PathBuf,
    /// Conversation ID this session belongs to
    conversation_id: String,
}

impl BridgeSession {
    /// Create a new bridge session for a conversation
    ///
    /// # Arguments
    /// * `conversation_id` - ID of the conversation this session belongs to
    /// * `bridge_script_path` - Path to the Node.js bridge script
    ///
    /// # Returns
    /// * `Result<Self, String>` - New BridgeSession or error
    pub async fn new(conversation_id: String, bridge_script_path: PathBuf) -> Result<Self, String> {
        debug!(
            conversation_id = %conversation_id,
            "Creating new bridge session"
        );

        // Spawn the Node.js bridge process
        let mut child = Command::new("node")
            .arg(&bridge_script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn bridge process: {}", e))?;

        // Extract stdin/stdout/stderr handles
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to get stdin handle".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to get stdout handle".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to get stderr handle".to_string())?;

        info!(
            conversation_id = %conversation_id,
            pid = child.id(),
            "Bridge process spawned successfully"
        );

        // Wrap stdout in BufReader for line-by-line reading
        let stdout_reader = BufReader::new(stdout);

        // Spawn task to collect stderr for debugging
        let conversation_id_for_stderr = conversation_id.clone();
        let stderr_handle = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut stderr_buf = Vec::new();
            let mut stderr_reader = stderr;
            let _ = stderr_reader.read_to_end(&mut stderr_buf).await;
            let stderr_text = String::from_utf8_lossy(&stderr_buf).to_string();
            if !stderr_text.trim().is_empty() {
                error!(
                    conversation_id = %conversation_id_for_stderr,
                    stderr = %stderr_text,
                    "Bridge process stderr output"
                );
            }
            stderr_text
        });

        Ok(Self {
            child: Mutex::new(Some(child)),
            stdin: Mutex::new(Some(stdin)),
            stdout: Mutex::new(Some(stdout_reader)),
            stderr: Mutex::new(Some(stderr_handle)),
            bridge_script_path,
            conversation_id,
        })
    }

    /// Get the path to the bridge script
    ///
    /// This is a helper to find the bridge script relative to the backend binary.
    pub fn get_bridge_script_path() -> PathBuf {
        // Try to find bridge script relative to current executable or cargo project
        // In production, this would be relative to the binary location
        // For development, look in the backend/bridge directory
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("bridge");
        path.push("gemini-bridge.js");
        path
    }

    /// Send a message to the bridge process
    ///
    /// # Arguments
    /// * `content` - Message content to send
    /// * `model` - Optional model to use
    ///
    /// # Returns
    /// * `Result<String, String>` - Response text or error
    ///
    /// # Timeout
    /// This operation has a timeout of 120 seconds. If the bridge process
    /// doesn't respond within this time, an error is returned.
    pub async fn send_message(&self, content: &str, model: Option<&str>) -> Result<String, String> {
        debug!(
            conversation_id = %self.conversation_id,
            content_len = content.len(),
            "Sending message to bridge"
        );

        // Build request
        let request = BridgeRequest {
            request_type: "message".to_string(),
            content: Some(content.to_string()),
            model: model.map(|s| s.to_string()),
        };

        // Serialize request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        // Send request to stdin
        {
            let mut stdin_guard = self.stdin.lock().await;
            let stdin = stdin_guard
                .as_mut()
                .ok_or_else(|| "Stdin handle not available".to_string())?;

            stdin
                .write_all(request_json.as_bytes())
                .await
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            stdin
                .write_all(b"\n")
                .await
                .map_err(|e| format!("Failed to write newline: {}", e))?;
            stdin
                .flush()
                .await
                .map_err(|e| format!("Failed to flush stdin: {}", e))?;
        }

        // Read response from stdout with timeout
        let timeout_duration = tokio::time::Duration::from_secs(120);
        let response_line = tokio::time::timeout(timeout_duration, async {
            // Check if process is still alive before reading
            {
                let mut child_guard = self.child.lock().await;
                if let Some(child) = child_guard.as_mut() {
                    if let Ok(Some(status)) = child.try_wait() {
                        // Process exited, get stderr for error details
                        let stderr_handle = self.stderr.lock().await.take();
                        if let Some(handle) = stderr_handle {
                            if let Ok(stderr_output) = handle.await {
                                if !stderr_output.trim().is_empty() {
                                    error!(
                                        conversation_id = %self.conversation_id,
                                        stderr = %stderr_output,
                                        exit_status = ?status,
                                        "Bridge process exited unexpectedly before response"
                                    );
                                    return Err(format!(
                                        "Bridge process exited with status {:?}. Stderr: {}",
                                        status, stderr_output
                                    ));
                                }
                            }
                        }
                        return Err(format!(
                            "Bridge process exited unexpectedly with status {:?}",
                            status
                        ));
                    }
                }
            }

            let mut stdout_guard = self.stdout.lock().await;
            let stdout_reader = stdout_guard
                .as_mut()
                .ok_or_else(|| "Stdout handle not available".to_string())?;

            // Read one line from stdout
            let mut response_buffer = String::new();
            let bytes_read = stdout_reader
                .read_line(&mut response_buffer)
                .await
                .map_err(|e| format!("Failed to read response: {}", e))?;

            if bytes_read == 0 {
                // EOF - process might have exited
                let mut child_guard = self.child.lock().await;
                if let Some(child) = child_guard.as_mut() {
                    if let Ok(Some(status)) = child.try_wait() {
                        // Process exited, get stderr
                        let stderr_handle = self.stderr.lock().await.take();
                        if let Some(handle) = stderr_handle {
                            if let Ok(stderr_output) = handle.await {
                                error!(
                                    conversation_id = %self.conversation_id,
                                    stderr = %stderr_output,
                                    exit_status = ?status,
                                    "Bridge process exited (EOF)"
                                );
                                return Err(format!(
                                    "Bridge process exited with status {:?}. Stderr: {}",
                                    status, stderr_output
                                ));
                            }
                        }
                        return Err(format!(
                            "Bridge process exited unexpectedly with status {:?} (EOF)",
                            status
                        ));
                    }
                }
                return Err("EOF while reading response (process may have exited)".to_string());
            }

            Ok::<String, String>(response_buffer.trim().to_string())
        })
        .await
        .map_err(|_| "Request timed out after 120 seconds".to_string())
        .and_then(|r| r)?;

        // Parse response
        let response: BridgeResponse = serde_json::from_str(&response_line)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        match response.status.as_str() {
            "success" => {
                debug!(
                    conversation_id = %self.conversation_id,
                    "Message sent successfully"
                );
                Ok(response.data.unwrap_or_default())
            }
            "error" => {
                let error_msg = response
                    .message
                    .unwrap_or_else(|| "Unknown error".to_string());
                error!(
                    conversation_id = %self.conversation_id,
                    error = %error_msg,
                    "Bridge returned error"
                );
                Err(error_msg)
            }
            _ => Err(format!("Unexpected response status: {}", response.status)),
        }
    }

    /// Kill the bridge process
    ///
    /// # Returns
    /// * `Result<(), String>` - Success or error
    pub async fn kill(&self) -> Result<(), String> {
        debug!(
            conversation_id = %self.conversation_id,
            "Killing bridge process"
        );

        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            child
                .kill()
                .await
                .map_err(|e| format!("Failed to kill bridge process: {}", e))?;

            // Wait for process to exit
            let _ = child.wait().await;

            info!(
                conversation_id = %self.conversation_id,
                "Bridge process killed successfully"
            );
        }

        Ok(())
    }

    /// Check if the bridge process is still running
    ///
    /// # Returns
    /// * `bool` - True if process is still running
    pub async fn is_running(&self) -> bool {
        let mut child_guard = self.child.lock().await;
        if let Some(child) = child_guard.as_mut() {
            // Try to get exit status without waiting
            match child.try_wait() {
                Ok(Some(_)) => false, // Process has exited
                Ok(None) => true,     // Process is still running
                Err(_) => false,      // Error checking status
            }
        } else {
            false
        }
    }

    /// Get the conversation ID
    #[allow(dead_code)] // Will be used in Phase 4 for metrics/monitoring
    pub fn conversation_id(&self) -> &str {
        &self.conversation_id
    }
}

impl Drop for BridgeSession {
    fn drop(&mut self) {
        // Attempt to kill the process on drop
        // Note: We can't await in Drop, so we use start_kill() which is synchronous
        // The process will be killed asynchronously by tokio
        if let Ok(mut child_guard) = self.child.try_lock() {
            if let Some(mut child) = child_guard.take() {
                let _ = child.start_kill();
            }
        }
    }
}
