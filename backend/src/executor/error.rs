//! Execution-specific error types
//!
//! Errors that can occur during agent execution (process spawning, timeouts, etc.)

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Process execution failed: {0}")]
    ProcessFailed(String),

    #[error("Command execution timed out after {0} seconds")]
    Timeout(u64),

    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("Invalid output encoding: {0}")]
    InvalidEncoding(String),

    #[error("Command not found: {0}")]
    #[allow(dead_code)] // Reserved for future use
    CommandNotFound(String),
}
