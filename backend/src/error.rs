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

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Invalid agent configuration: {0}")]
    InvalidAgentConfig(String),

    #[error("Persistence error: {0}")]
    Persistence(#[from] crate::state::PersistenceError),

    #[error("Execution error: {0}")]
    ExecutionError(#[from] crate::executor::ExecutionError),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Path is not a directory: {0}")]
    NotADirectory(String),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
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
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
