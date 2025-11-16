//! Execution-specific error types
//!
//! Errors that can occur during agent execution (process spawning, timeouts, etc.)

use thiserror::Error;

/// Errors that can occur during agent execution
///
/// These errors are specific to process spawning, execution, and output handling.
#[derive(Error, Debug)]
pub enum ExecutionError {
    /// Process execution failed with non-zero exit code
    #[error("Process execution failed: {0}")]
    ProcessFailed(String),

    /// Command execution exceeded the timeout limit
    #[error("Command execution timed out after {0} seconds")]
    Timeout(u64),

    /// Failed to spawn the process (e.g., command not found, permission denied)
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[from] std::io::Error),

    /// Process output could not be decoded as UTF-8
    #[error("Invalid output encoding: {0}")]
    InvalidEncoding(String),

    /// Command executable was not found in PATH
    #[error("Command not found: {0}")]
    #[allow(dead_code)] // Reserved for future use
    CommandNotFound(String),
}
