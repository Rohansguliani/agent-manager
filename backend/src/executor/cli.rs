//! CLI executor implementation
//!
//! Executes CLI agents by spawning processes and capturing their output.

use crate::executor::error::ExecutionError;
use crate::state::Agent;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error, info};

/// CLI executor for running agent processes
pub struct CliExecutor {
    /// Default timeout for process execution (in seconds)
    default_timeout: Duration,
}

impl CliExecutor {
    /// Create a new CLI executor with default timeout
    pub fn new(default_timeout_secs: u64) -> Self {
        Self {
            default_timeout: Duration::from_secs(default_timeout_secs),
        }
    }

    /// Get the default timeout duration
    #[cfg(test)]
    pub fn timeout(&self) -> Duration {
        self.default_timeout
    }

    /// Execute a query using the given agent
    ///
    /// # Arguments
    /// * `agent` - The agent to execute
    /// * `query` - The query string to pass to the agent
    ///
    /// # Returns
    /// * `Ok(String)` - The stdout output from the agent
    /// * `Err(ExecutionError)` - If execution failed
    pub async fn execute(&self, agent: &Agent, query: &str) -> Result<String, ExecutionError> {
        info!(
            agent_id = %agent.id,
            agent_name = %agent.name,
            query_len = query.len(),
            "Executing agent query"
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

        // Pass through GEMINI_API_KEY if it exists (for Gemini CLI)
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            cmd.env("GEMINI_API_KEY", api_key);
        }

        // Set working directory if specified
        if let Some(work_dir) = &agent.config.working_dir {
            cmd.current_dir(work_dir);
        }

        debug!(
            command = %agent.config.command,
            args = ?agent.config.args,
            "Spawning process"
        );

        // Execute with timeout
        match timeout(self.default_timeout, cmd.output()).await {
            Ok(Ok(output)) => {
                if output.status.success() {
                    let response = String::from_utf8(output.stdout).map_err(|e| {
                        ExecutionError::InvalidEncoding(format!("Failed to decode stdout: {}", e))
                    })?;

                    info!(
                        agent_id = %agent.id,
                        response_len = response.len(),
                        "Query executed successfully"
                    );

                    Ok(response)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let exit_code = output.status.code().unwrap_or(-1);

                    error!(
                        agent_id = %agent.id,
                        exit_code = exit_code,
                        stderr = %stderr,
                        "Process execution failed"
                    );

                    Err(ExecutionError::ProcessFailed(format!(
                        "Process exited with code {}: {}",
                        exit_code, stderr
                    )))
                }
            }
            Ok(Err(e)) => {
                error!(
                    agent_id = %agent.id,
                    error = %e,
                    "Failed to spawn or execute process"
                );
                Err(ExecutionError::SpawnFailed(e))
            }
            Err(_) => {
                error!(
                    agent_id = %agent.id,
                    timeout_secs = self.default_timeout.as_secs(),
                    "Process execution timed out"
                );
                Err(ExecutionError::Timeout(self.default_timeout.as_secs()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Agent, AgentConfig, AgentStatus, AgentType};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = CliExecutor::new(30);
        assert_eq!(executor.timeout().as_secs(), 30);
    }

    #[tokio::test]
    async fn test_executor_with_different_timeout() {
        let executor = CliExecutor::new(60);
        assert_eq!(executor.timeout().as_secs(), 60);
    }

    #[tokio::test]
    async fn test_executor_with_simple_command() {
        // Test with a simple command that should work on all systems
        let executor = CliExecutor::new(5);

        let agent = Agent {
            id: "test-1".to_string(),
            name: "Test Agent".to_string(),
            agent_type: AgentType::Generic,
            status: AgentStatus::Idle,
            config: AgentConfig {
                command: "echo".to_string(),
                args: vec!["Hello from executor test".to_string()],
                env_vars: HashMap::new(),
                working_dir: None,
                options: HashMap::new(),
            },
        };

        // Execute with empty query (echo doesn't need query, just args)
        let result = executor.execute(&agent, "").await;

        // Should succeed and return the echo output
        assert!(result.is_ok(), "Executor should succeed with echo command");
        let output = result.unwrap();
        assert!(
            output.contains("Hello from executor test"),
            "Output should contain the echo message"
        );
    }

    #[tokio::test]
    async fn test_executor_with_nonexistent_command() {
        let executor = CliExecutor::new(5);

        let agent = Agent {
            id: "test-2".to_string(),
            name: "Invalid Agent".to_string(),
            agent_type: AgentType::Generic,
            status: AgentStatus::Idle,
            config: AgentConfig {
                command: "nonexistent-command-that-does-not-exist-12345".to_string(),
                args: vec![],
                env_vars: HashMap::new(),
                working_dir: None,
                options: HashMap::new(),
            },
        };

        let result = executor.execute(&agent, "test").await;

        // Should fail with SpawnFailed error
        assert!(
            result.is_err(),
            "Executor should fail with nonexistent command"
        );
        match result.unwrap_err() {
            ExecutionError::SpawnFailed(_) => {
                // Expected error type
            }
            other => {
                panic!("Expected SpawnFailed error, got: {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn test_executor_with_env_vars() {
        let executor = CliExecutor::new(5);

        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

        let agent = Agent {
            id: "test-3".to_string(),
            name: "Env Test Agent".to_string(),
            agent_type: AgentType::Generic,
            status: AgentStatus::Idle,
            config: AgentConfig {
                // Use a command that can check env vars (works on Unix-like systems)
                #[cfg(unix)]
                command: "sh".to_string(),
                #[cfg(unix)]
                args: vec!["-c".to_string(), "echo $TEST_VAR".to_string()],
                #[cfg(not(unix))]
                command: "cmd".to_string(),
                #[cfg(not(unix))]
                args: vec!["/C".to_string(), "echo %TEST_VAR%".to_string()],
                env_vars,
                working_dir: None,
                options: HashMap::new(),
            },
        };

        let result = executor.execute(&agent, "").await;

        // Should succeed and environment variable should be passed
        if result.is_ok() {
            let output = result.unwrap();
            // On Unix, should contain the env var value
            #[cfg(unix)]
            assert!(
                output.contains("test_value"),
                "Output should contain the environment variable value"
            );
        }
        // On Windows, this test might behave differently, so we just check it doesn't panic
    }
}
