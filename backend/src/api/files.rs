//! File system API handlers
//!
//! Provides HTTP endpoints for browsing the file system and managing file context.
//! Uses the file service layer for business logic.

use crate::api::utils::RouterState;
use crate::error::AppError;
use crate::services::files::FileService;
use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export FileInfo for API responses (used by frontend)
pub use crate::services::files::FileInfo;

/// Response for listing files
#[derive(Debug, Serialize)]
pub struct ListFilesResponse {
    /// List of files and directories in the path
    pub files: Vec<FileInfo>,
    /// Absolute path that was listed
    pub path: String,
}

/// Request to set working directory
#[derive(Deserialize)]
pub struct SetWorkingDirectoryRequest {
    /// Path to set as working directory (None to clear)
    pub path: Option<String>,
}

/// Response for working directory
#[derive(Debug, Serialize)]
pub struct WorkingDirectoryResponse {
    /// Current working directory path (None if not set)
    pub path: Option<String>,
}

/// GET /api/files - List files in a directory
pub async fn list_files(
    State((_state, _, _)): State<RouterState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListFilesResponse>, AppError> {
    // Get path from query params, default to home directory
    // In Docker, home is mounted at /host/home
    let default_path = if std::path::Path::new("/host/home").exists() {
        "/host/home".to_string()
    } else {
        // Fallback to current user's home directory if not in Docker
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE")) // Windows fallback
            .unwrap_or_else(|_| ".".to_string())
    };
    let path_str = params.get("path").unwrap_or(&default_path);

    // Use service layer to list directory
    let (files, absolute_path) = FileService::list_directory(path_str).await?;

    Ok(Json(ListFilesResponse {
        files,
        path: absolute_path.to_string_lossy().to_string(),
    }))
}

/// GET /api/files/working-directory - Get current working directory context
pub async fn get_working_directory(
    State((state, _, _)): State<RouterState>,
) -> Result<Json<WorkingDirectoryResponse>, AppError> {
    let state = state.read().await;
    let path = state.working_directory().cloned();
    Ok(Json(WorkingDirectoryResponse { path }))
}

/// POST /api/files/working-directory - Set working directory context
pub async fn set_working_directory(
    State((state, _, _)): State<RouterState>,
    Json(request): Json<SetWorkingDirectoryRequest>,
) -> Result<Json<WorkingDirectoryResponse>, AppError> {
    // Validate and canonicalize path if provided using service layer
    let canonical_path = if let Some(ref path_str) = request.path {
        let canonical = FileService::validate_directory_path(path_str)?;
        Some(canonical.to_string_lossy().to_string())
    } else {
        None
    };

    let mut state = state.write().await;
    state.set_working_directory(canonical_path.clone());

    Ok(Json(WorkingDirectoryResponse {
        path: canonical_path,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::utils::RouterState;
    use crate::chat::ChatDb;
    use crate::state::AppState;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tempfile::TempDir;
    use tokio::sync::RwLock;

    async fn create_test_router_state() -> RouterState {
        let app_state = Arc::new(RwLock::new(AppState::new()));
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let chat_db = ChatDb::new(db_path.to_str().unwrap())
            .await
            .expect("Failed to create test database");
        let bridge_manager = Arc::new(crate::chat::BridgeManager::new());
        (app_state, Arc::new(chat_db), bridge_manager)
    }

    #[tokio::test]
    async fn test_list_files_current_directory() {
        let router_state = create_test_router_state().await;
        let params = HashMap::new();
        // Don't set path, should default to "."

        let result = list_files(State(router_state.clone()), Query(params)).await;
        assert!(result.is_ok(), "Should list current directory");
        let response = result.unwrap();
        assert!(!response.files.is_empty() || response.path.contains('.'));
    }

    #[tokio::test]
    async fn test_list_files_specific_directory() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_str().unwrap().to_string();

        // Create some test files
        std::fs::write(temp_dir.path().join("test1.txt"), "content1")
            .expect("Failed to create test file");
        std::fs::write(temp_dir.path().join("test2.txt"), "content2")
            .expect("Failed to create test file");
        std::fs::create_dir(temp_dir.path().join("subdir")).expect("Failed to create subdir");

        let router_state = create_test_router_state().await;
        let mut params = HashMap::new();
        params.insert("path".to_string(), temp_path.clone());

        let result = list_files(State(router_state.clone()), Query(params)).await;
        assert!(result.is_ok(), "Should list directory");
        let response = result.unwrap();
        assert_eq!(response.files.len(), 3);
        // Path might be canonicalized, so check if it contains the temp path
        let canonical_temp = std::path::Path::new(&temp_path).canonicalize().unwrap();
        assert!(
            response.path.contains(canonical_temp.to_str().unwrap())
                || response.path.contains(&temp_path)
        );
    }

    #[tokio::test]
    async fn test_list_files_nonexistent() {
        let router_state = create_test_router_state().await;
        let mut params = HashMap::new();
        params.insert("path".to_string(), "/nonexistent/path/12345".to_string());

        let result = list_files(State(router_state.clone()), Query(params)).await;
        assert!(result.is_err(), "Should fail for nonexistent path");
        match result.unwrap_err() {
            AppError::FileNotFound(_) => {
                // Expected error
            }
            other => {
                panic!("Expected FileNotFound error, got: {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn test_get_working_directory_default() {
        let router_state = create_test_router_state().await;
        let result = get_working_directory(State(router_state.clone())).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.path.is_none(), "Default should be None");
    }

    #[tokio::test]
    async fn test_set_and_get_working_directory() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_str().unwrap().to_string();

        let router_state = create_test_router_state().await;
        let request = SetWorkingDirectoryRequest {
            path: Some(temp_path.clone()),
        };

        // Set working directory
        let result = set_working_directory(State(router_state.clone()), Json(request)).await;
        assert!(result.is_ok(), "Should set working directory");
        let response = result.unwrap();
        assert!(response.path.is_some());
        assert!(response.path.as_ref().unwrap().contains(&temp_path));

        // Get working directory
        let result = get_working_directory(State(router_state.clone())).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.path.is_some());
        assert!(response.path.as_ref().unwrap().contains(&temp_path));
    }

    #[tokio::test]
    async fn test_set_working_directory_nonexistent() {
        let router_state = create_test_router_state().await;
        let request = SetWorkingDirectoryRequest {
            path: Some("/nonexistent/path/12345".to_string()),
        };

        let result = set_working_directory(State(router_state.clone()), Json(request)).await;
        assert!(result.is_err(), "Should fail for nonexistent path");
        match result.unwrap_err() {
            AppError::FileNotFound(_) => {
                // Expected error
            }
            other => {
                panic!("Expected FileNotFound error, got: {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn test_set_working_directory_file_not_dir() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").expect("Failed to create file");

        let router_state = create_test_router_state().await;
        let request = SetWorkingDirectoryRequest {
            path: Some(file_path.to_str().unwrap().to_string()),
        };

        let result = set_working_directory(State(router_state.clone()), Json(request)).await;
        assert!(result.is_err(), "Should fail for file path");
        match result.unwrap_err() {
            AppError::NotADirectory(_) => {
                // Expected error
            }
            other => {
                panic!("Expected NotADirectory error, got: {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn test_clear_working_directory() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_str().unwrap().to_string();

        let router_state = create_test_router_state().await;

        // Set working directory first
        let request = SetWorkingDirectoryRequest {
            path: Some(temp_path),
        };
        let _ = set_working_directory(State(router_state.clone()), Json(request)).await;

        // Clear working directory
        let request = SetWorkingDirectoryRequest { path: None };
        let result = set_working_directory(State(router_state.clone()), Json(request)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.path.is_none(), "Should clear working directory");

        // Verify it's cleared
        let result = get_working_directory(State(router_state.clone())).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.path.is_none());
    }
}
