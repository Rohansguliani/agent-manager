//! Error types and error handling for the application
//!
//! This module defines custom error types that can be converted to HTTP responses.
//! All errors implement `IntoResponse` to provide consistent error formatting.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Application-level error types
///
/// All errors that can occur in the application are represented by this enum.
/// Each variant implements automatic conversion to HTTP responses via `IntoResponse`.
#[derive(Error, Debug)]
pub enum AppError {
    /// Agent with the given ID was not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Agent configuration is invalid
    #[error("Invalid agent configuration: {0}")]
    InvalidAgentConfig(String),

    /// Error occurred during state persistence
    #[error("Persistence error: {0}")]
    Persistence(#[from] crate::state::PersistenceError),

    /// Error occurred during agent execution
    #[error("Execution error: {0}")]
    ExecutionError(#[from] crate::executor::ExecutionError),

    /// File or directory was not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Path is invalid (e.g., contains invalid characters)
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Permission denied for the requested operation
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Path exists but is not a directory
    #[error("Path is not a directory: {0}")]
    NotADirectory(String),

    /// Internal server error (catch-all for unexpected errors)
    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    /// Plan validation failed
    #[error("Invalid plan: {0}")]
    InvalidPlan(String),

    /// Plan execution failed (e.g., timeout, graph error)
    #[error("Plan execution failed: {0}")]
    PlanExecutionFailed(String),

    /// Individual task execution failed
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    /// Graph-flow session error
    #[error("Session error: {0}")]
    SessionError(String),

    /// Graph-flow graph construction or execution error
    #[error("Graph error: {0}")]
    GraphError(String),

    /// Planner failed to generate a valid plan
    #[error("Planning failed: {0}")]
    #[allow(dead_code)] // Will be used when planner errors are properly categorized
    PlanningFailed(String),

    /// Operation timed out
    #[error("Timeout: {0}")]
    Timeout(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::AgentNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::InvalidAgentConfig(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Persistence(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::ExecutionError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::FileNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::InvalidPath(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::PermissionDenied(_) => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::NotADirectory(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidPlan(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::PlanExecutionFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::TaskExecutionFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::SessionError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::GraphError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::PlanningFailed(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Timeout(_) => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
