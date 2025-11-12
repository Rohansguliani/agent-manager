//! Agent execution module
//!
//! This module provides functionality for executing CLI-based agents.
//! It handles process spawning, output capture, timeout management, and error handling.

pub mod cli;
pub mod error;
pub mod streaming;

pub use cli::CliExecutor;
pub use error::ExecutionError;
pub use streaming::StreamingCliExecutor;
