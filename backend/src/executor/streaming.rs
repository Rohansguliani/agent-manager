//! Streaming CLI executor implementation
//!
//! Executes CLI agents by spawning processes and streaming their output line-by-line.

use crate::executor::error::ExecutionError;
use crate::state::Agent;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
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

        // Add query as argument
        cmd.arg(query);

        // Add any additional arguments from agent config
        for arg in &agent.config.args {
            cmd.arg(arg);
        }

        // Set environment variables from agent config
        for (key, value) in &agent.config.env_vars {
            cmd.env(key, value);
        }

        // Pass through GEMINI_API_KEY if it exists (for Gemini CLI)
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            cmd.env("GEMINI_API_KEY", api_key);
        }

        // Set working directory if specified
        if let Some(work_dir) = &agent.config.working_dir {
            cmd.current_dir(work_dir);
        }

        // Capture stdout and stderr separately
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        debug!(
            command = %agent.config.command,
            args = ?agent.config.args,
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

        // Spawn a task to read stdout line by line and send through the channel
        // The task owns the sender and will drop it when reading is complete
        // This ensures the channel stays open until all data is read
        let agent_id_clone = agent_id.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut line_count = 0;

            while let Ok(Some(line)) = lines.next_line().await {
                line_count += 1;
                if tx.send(line).await.is_err() {
                    // Receiver dropped, stop reading
                    debug!(
                        agent_id = %agent_id_clone,
                        "Receiver dropped, stopping stdout read"
                    );
                    break;
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
                    error!(
                        agent_id = %agent_id_stderr,
                        stderr_line = %line,
                        "Process stderr output"
                    );
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
