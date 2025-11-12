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

        // Read stdout line by line
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        // Spawn a task to read lines and send them through the channel
        let agent_id = agent.id.clone();
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_clone.send(line).await.is_err() {
                    // Receiver dropped, stop reading
                    break;
                }
            }
        });

        // Wait for process to complete with overall timeout
        let status_result = timeout(self.default_timeout, child.wait()).await;

        // Close the sender to signal end of stream
        drop(tx);

        match status_result {
            Ok(Ok(status)) => {
                if status.success() {
                    info!(
                        agent_id = %agent_id,
                        "Query executed successfully with streaming"
                    );
                    Ok(rx)
                } else {
                    let exit_code = status.code().unwrap_or(-1);
                    error!(
                        agent_id = %agent_id,
                        exit_code = exit_code,
                        "Process execution failed"
                    );
                    Err(ExecutionError::ProcessFailed(format!(
                        "Process exited with code {}",
                        exit_code
                    )))
                }
            }
            Ok(Err(e)) => {
                error!(
                    agent_id = %agent_id,
                    error = %e,
                    "Error waiting for process"
                );
                Err(ExecutionError::SpawnFailed(e))
            }
            Err(_) => {
                error!(
                    agent_id = %agent_id,
                    timeout_secs = self.default_timeout.as_secs(),
                    "Process execution timed out"
                );
                // Kill the process
                let _ = child.kill().await;
                Err(ExecutionError::Timeout(self.default_timeout.as_secs()))
            }
        }
    }
}
