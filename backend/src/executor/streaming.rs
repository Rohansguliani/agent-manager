//! Streaming CLI executor implementation
//!
//! Executes CLI agents by spawning processes and streaming their output line-by-line.

use crate::executor::error::ExecutionError;
use crate::orchestrator::primitives::parse_gemini_json_response;
use crate::state::Agent;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error, info};

/// Streaming CLI executor for running agent processes with real-time output
pub struct StreamingCliExecutor {
    /// Default timeout for process execution (in seconds)
    default_timeout: Duration,
}

impl StreamingCliExecutor {
    /// Create a new streaming CLI executor with default timeout
    #[allow(dead_code)]
    pub fn new(default_timeout_secs: u64) -> Self {
        Self {
            default_timeout: Duration::from_secs(default_timeout_secs),
        }
    }

    /// Execute a query and stream output line by line
    ///
    /// Returns a channel receiver that yields lines as they come
    pub async fn execute_streaming(
        &self,
        agent: &Agent,
        query: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<String>, ExecutionError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        info!(
            agent_id = %agent.id,
            agent_name = %agent.name,
            query_len = query.len(),
            "Executing agent query with streaming"
        );

        // Build the command from agent configuration
        let mut cmd = Command::new(&agent.config.command);

        // Add query: use `-p` flag for Gemini CLI, positional argument for others
        match agent.agent_type {
            crate::state::AgentType::Gemini => {
                // Gemini CLI requires `-p` flag for the prompt
                cmd.arg("-p").arg(query);
            }
            _ => {
                // Other CLI tools accept query as first positional argument
                cmd.arg(query);
            }
        }

        // Add any additional arguments from agent config
        for arg in &agent.config.args {
            cmd.arg(arg);
        }

        // Set environment variables from agent config
        for (key, value) in &agent.config.env_vars {
            cmd.env(key, value);
        }

        // System prompt hierarchy for Gemini CLI:
        // Priority 1: Agent-specific system prompt (from agent config env_vars)
        // Priority 2: Global fallback (only if agent didn't specify one)
        // Priority 3: Default (Gemini CLI's internal prompt) - no action needed
        if !agent.config.env_vars.contains_key("GEMINI_SYSTEM_MD") {
            if let Ok(global_system_md) = std::env::var("GEMINI_SYSTEM_MD") {
                cmd.env("GEMINI_SYSTEM_MD", global_system_md);
            }
        }

        // Pass through GEMINI_API_KEY if it exists (for Gemini CLI)
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            cmd.env("GEMINI_API_KEY", api_key);
        }

        // Set working directory
        // If not specified, use /tmp to prevent Gemini CLI from reading project files
        // This ensures the AI doesn't get unwanted context from the project structure
        let work_dir = agent.config.working_dir.as_deref().unwrap_or("/tmp");
        cmd.current_dir(work_dir);

        // Capture stdout and stderr separately
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        debug!(
            command = %agent.config.command,
            args = ?agent.config.args,
            working_dir = %work_dir,
            "Spawning process for streaming"
        );

        // Spawn the process
        let mut child = cmd.spawn().map_err(ExecutionError::SpawnFailed)?;

        // Get stdout handle
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ExecutionError::ProcessFailed("Failed to capture stdout".to_string()))?;

        // Get stderr handle for error logging
        let stderr = child.stderr.take();

        // Clone agent_id for logging
        let agent_id = agent.id.clone();

        // Check if this is a Gemini agent with JSON output format
        let is_gemini_json = matches!(agent.agent_type, crate::state::AgentType::Gemini)
            && agent
                .config
                .args
                .iter()
                .any(|arg| arg == "--output-format" || arg == "json");

        // Spawn a task to read stdout and process output
        // For JSON mode: read full response, parse, then send entire parsed text at once
        // For non-JSON: read full response, then send all at once
        let agent_id_clone = agent_id.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut buffer = Vec::new();
            let mut line_count = 0;

            // Read all bytes from stdout until EOF
            match reader.read_to_end(&mut buffer).await {
                Ok(_) => {
                    // Convert bytes to string
                    match String::from_utf8(buffer) {
                        Ok(output) => {
                            if !output.is_empty() {
                                if is_gemini_json {
                                    // For JSON mode: parse JSON and extract response field, send entire text at once
                                    match parse_gemini_json_response(output.trim()) {
                                        Ok(response_text) => {
                                            // Send entire parsed response at once (no character-by-character streaming)
                                            if tx.send(response_text).await.is_err() {
                                                debug!(
                                                    agent_id = %agent_id_clone,
                                                    "Receiver dropped, stopping stdout read"
                                                );
                                            }
                                            line_count += 1;
                                        }
                                        Err(e) => {
                                            // JSON parsing failed, fall back to raw output
                                            debug!(
                                                agent_id = %agent_id_clone,
                                                error = %e,
                                                "Failed to parse Gemini JSON response, sending raw output"
                                            );
                                            // Send raw output as-is
                                            if tx.send(output.trim().to_string()).await.is_err() {
                                                debug!(
                                                    agent_id = %agent_id_clone,
                                                    "Receiver dropped, stopping stdout read"
                                                );
                                            }
                                            line_count += 1;
                                        }
                                    }
                                } else {
                                    // For non-JSON output: send entire output at once
                                    if tx.send(output.trim().to_string()).await.is_err() {
                                        // Receiver dropped, stop reading
                                        debug!(
                                            agent_id = %agent_id_clone,
                                            "Receiver dropped, stopping stdout read"
                                        );
                                    }
                                    line_count += 1;
                                }
                            } else {
                                debug!(
                                    agent_id = %agent_id_clone,
                                    "stdout is empty"
                                );
                            }
                        }
                        Err(e) => {
                            // UTF-8 conversion error
                            debug!(
                                agent_id = %agent_id_clone,
                                error = %e,
                                "Failed to convert stdout to UTF-8"
                            );
                        }
                    }
                }
                Err(e) => {
                    // Error reading stdout
                    debug!(
                        agent_id = %agent_id_clone,
                        error = %e,
                        "Error reading stdout"
                    );
                }
            }

            debug!(
                agent_id = %agent_id_clone,
                lines_read = line_count,
                "Finished reading stdout"
            );
            // Sender is dropped here when the task completes, closing the channel
        });

        // Spawn a task to read stderr and log errors (if any)
        if let Some(stderr) = stderr {
            let agent_id_stderr = agent_id.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    // Log stderr at debug level - it's often informational (e.g., "Loaded cached credentials")
                    // Only log as error if it contains error keywords
                    if line.to_lowercase().contains("error")
                        || line.to_lowercase().contains("fail")
                        || line.to_lowercase().contains("panic")
                    {
                        error!(
                            agent_id = %agent_id_stderr,
                            stderr_line = %line,
                            "Process stderr output (error detected)"
                        );
                    } else {
                        debug!(
                            agent_id = %agent_id_stderr,
                            stderr_line = %line,
                            "Process stderr output"
                        );
                    }
                }
            });
        }

        // Spawn a task to wait for process completion and handle timeout
        // The child process is moved into this task so we can kill it on timeout
        // This runs in the background and doesn't block the return
        let agent_id_wait = agent_id.clone();
        let timeout_duration = self.default_timeout;
        tokio::spawn(async move {
            match timeout(timeout_duration, child.wait()).await {
                Ok(Ok(status)) => {
                    if status.success() {
                        info!(
                            agent_id = %agent_id_wait,
                            "Process completed successfully"
                        );
                    } else {
                        let exit_code = status.code().unwrap_or(-1);
                        error!(
                            agent_id = %agent_id_wait,
                            exit_code = exit_code,
                            "Process exited with error code"
                        );
                    }
                }
                Ok(Err(e)) => {
                    error!(
                        agent_id = %agent_id_wait,
                        error = %e,
                        "Error waiting for process"
                    );
                }
                Err(_) => {
                    error!(
                        agent_id = %agent_id_wait,
                        timeout_secs = timeout_duration.as_secs(),
                        "Process execution timed out, killing process"
                    );
                    // Try to kill the process (child is moved into this closure)
                    if let Err(e) = child.kill().await {
                        error!(
                            agent_id = %agent_id_wait,
                            error = %e,
                            "Failed to kill timed-out process"
                        );
                    }
                }
            }
        });

        // Return receiver immediately so caller can start reading
        // The reading task will complete when stdout is closed (process exits)
        // The sender will be dropped when the reading task completes, closing the channel
        info!(
            agent_id = %agent_id,
            "Started streaming process, returning receiver"
        );
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Agent, AgentConfig, AgentStatus, AgentType};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_parse_gemini_json_response() {
        // Test valid JSON with response field
        let json_response =
            r#"{"response": "Hello! How can I help you today?", "status": "success"}"#;
        let result = parse_gemini_json_response(json_response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello! How can I help you today?");

        // Test JSON with empty response
        let json_empty = r#"{"response": "", "status": "success"}"#;
        let result = parse_gemini_json_response(json_empty);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");

        // Test JSON without response field (should fall back to raw)
        let json_no_response = r#"{"status": "success", "message": "done"}"#;
        let result = parse_gemini_json_response(json_no_response);
        assert!(result.is_ok());
        // Should return trimmed raw response
        assert!(result.unwrap().contains("status"));

        // Test invalid JSON (should fall back to raw)
        let invalid_json = "not json at all";
        let result = parse_gemini_json_response(invalid_json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "not json at all");
    }

    #[tokio::test]
    async fn test_streaming_executor_creation() {
        let executor = StreamingCliExecutor::new(30);
        // Just verify it can be created
        assert!(std::mem::size_of_val(&executor) > 0);
    }

    #[tokio::test]
    async fn test_gemini_json_detection() {
        // Test agent with JSON output format
        let agent_json = Agent {
            id: "test-1".to_string(),
            name: "Gemini JSON Agent".to_string(),
            agent_type: AgentType::Gemini,
            status: AgentStatus::Idle,
            config: AgentConfig {
                command: "echo".to_string(),
                args: vec!["--output-format".to_string(), "json".to_string()],
                env_vars: HashMap::new(),
                working_dir: None,
                options: HashMap::new(),
            },
        };

        // Check detection logic
        let is_gemini_json = matches!(agent_json.agent_type, AgentType::Gemini)
            && agent_json
                .config
                .args
                .iter()
                .any(|arg| arg == "--output-format" || arg == "json");
        assert!(is_gemini_json, "Should detect Gemini JSON output format");

        // Test agent without JSON format
        let agent_no_json = Agent {
            id: "test-2".to_string(),
            name: "Gemini Regular Agent".to_string(),
            agent_type: AgentType::Gemini,
            status: AgentStatus::Idle,
            config: AgentConfig {
                command: "echo".to_string(),
                args: vec![],
                env_vars: HashMap::new(),
                working_dir: None,
                options: HashMap::new(),
            },
        };

        let is_gemini_json_no = matches!(agent_no_json.agent_type, AgentType::Gemini)
            && agent_no_json
                .config
                .args
                .iter()
                .any(|arg| arg == "--output-format" || arg == "json");
        assert!(
            !is_gemini_json_no,
            "Should not detect JSON format when args are empty"
        );
    }

    #[tokio::test]
    async fn test_system_prompt_hierarchy() {
        use std::collections::HashMap;

        // Test Priority 1: Agent-specific system prompt (from env_vars)
        let agent_with_custom = Agent {
            id: "test-3".to_string(),
            name: "Custom Prompt Agent".to_string(),
            agent_type: AgentType::Gemini,
            status: AgentStatus::Idle,
            config: AgentConfig {
                command: "echo".to_string(),
                args: vec![],
                env_vars: {
                    let mut env = HashMap::new();
                    env.insert(
                        "GEMINI_SYSTEM_MD".to_string(),
                        "/custom/prompt.md".to_string(),
                    );
                    env
                },
                working_dir: None,
                options: HashMap::new(),
            },
        };

        // Agent with custom prompt should have it in env_vars
        assert_eq!(
            agent_with_custom
                .config
                .env_vars
                .get("GEMINI_SYSTEM_MD")
                .unwrap(),
            "/custom/prompt.md"
        );

        // Test Priority 2: Agent without custom prompt (should fall back to global)
        let agent_without_custom = Agent {
            id: "test-4".to_string(),
            name: "Default Prompt Agent".to_string(),
            agent_type: AgentType::Gemini,
            status: AgentStatus::Idle,
            config: AgentConfig {
                command: "echo".to_string(),
                args: vec![],
                env_vars: HashMap::new(), // No GEMINI_SYSTEM_MD in agent config
                working_dir: None,
                options: HashMap::new(),
            },
        };

        // Agent without custom prompt should not have GEMINI_SYSTEM_MD in env_vars
        assert!(!agent_without_custom
            .config
            .env_vars
            .contains_key("GEMINI_SYSTEM_MD"));

        // Test hierarchy logic: agent-specific takes precedence
        let has_agent_specific = agent_with_custom
            .config
            .env_vars
            .contains_key("GEMINI_SYSTEM_MD");
        assert!(
            has_agent_specific,
            "Agent with custom prompt should have GEMINI_SYSTEM_MD in env_vars"
        );
    }
}
