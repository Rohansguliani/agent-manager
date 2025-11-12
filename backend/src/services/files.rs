//! File system service
//!
//! Provides file system operations with proper error handling and validation.

use crate::error::AppError;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::fs;

/// File or directory information
#[derive(Debug, Serialize, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified: Option<u64>, // Unix timestamp
}

/// File system service
pub struct FileService;

impl FileService {
    /// Validate and canonicalize a path
    ///
    /// # Arguments
    /// * `path_str` - Path string to validate
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Canonicalized absolute path
    /// * `Err(AppError)` - If path is invalid, doesn't exist, or cannot be accessed
    pub fn validate_and_canonicalize_path(path_str: &str) -> Result<PathBuf, AppError> {
        let path = Path::new(path_str);

        // Check if path exists
        if !path.exists() {
            return Err(AppError::FileNotFound(format!(
                "Path does not exist: {}",
                path_str
            )));
        }

        // Canonicalize path (resolve .. and .)
        let canonical = path
            .canonicalize()
            .map_err(|e| AppError::InvalidPath(format!("Invalid path: {} - {}", path_str, e)))?;

        Ok(canonical)
    }

    /// Validate that a path is a directory
    ///
    /// # Arguments
    /// * `path_str` - Path string to validate
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Canonicalized absolute path
    /// * `Err(AppError)` - If path is not a directory or doesn't exist
    pub fn validate_directory_path(path_str: &str) -> Result<PathBuf, AppError> {
        let canonical = Self::validate_and_canonicalize_path(path_str)?;

        if !canonical.is_dir() {
            return Err(AppError::NotADirectory(format!(
                "Path is not a directory: {}",
                path_str
            )));
        }

        Ok(canonical)
    }

    /// List files and directories in a path
    ///
    /// # Arguments
    /// * `path` - Path to list (will be validated and canonicalized)
    ///
    /// # Returns
    /// * `Ok(Vec<FileInfo>)` - List of files and directories
    /// * `Err(AppError)` - If path is invalid or cannot be read
    pub async fn list_directory(path_str: &str) -> Result<(Vec<FileInfo>, PathBuf), AppError> {
        // Validate and canonicalize path
        let absolute_path = Self::validate_directory_path(path_str)?;

        // Read directory entries
        let mut entries = fs::read_dir(&absolute_path).await.map_err(|e| {
            AppError::PermissionDenied(format!("Failed to read directory: {} - {}", path_str, e))
        })?;

        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            AppError::PermissionDenied(format!(
                "Failed to read directory entry: {} - {}",
                path_str, e
            ))
        })? {
            let entry_path = entry.path();
            let metadata = entry.metadata().await.map_err(|e| {
                AppError::PermissionDenied(format!(
                    "Failed to read metadata: {} - {}",
                    entry_path.display(),
                    e
                ))
            })?;

            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let path_str = entry_path.to_string_lossy().to_string();
            let is_directory = metadata.is_dir();
            let size = if is_directory {
                None
            } else {
                Some(metadata.len())
            };
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            files.push(FileInfo {
                name,
                path: path_str,
                is_directory,
                size,
                modified,
            });
        }

        // Sort: directories first, then by name
        files.sort_by(|a, b| match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok((files, absolute_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_list_directory_simple() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create some test files and directories
        std::fs::write(temp_path.join("file1.txt"), "content1").expect("Failed to create file1");
        std::fs::write(temp_path.join("file2.rs"), "content2").expect("Failed to create file2");
        std::fs::create_dir(temp_path.join("subdir")).expect("Failed to create subdir");

        let (files, canonical_path) = FileService::list_directory(temp_path.to_str().unwrap())
            .await
            .expect("Failed to list directory");

        assert_eq!(files.len(), 3);
        assert!(canonical_path.exists());
        assert!(canonical_path.is_dir());

        // Check that directories come first
        assert!(files[0].is_directory);
        assert_eq!(files[0].name, "subdir");
    }

    #[tokio::test]
    async fn test_list_directory_nonexistent() {
        let result = FileService::list_directory("/nonexistent/path/12345").await;
        assert!(result.is_err());
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
    async fn test_validate_directory_path_file() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").expect("Failed to create file");

        let result = FileService::validate_directory_path(file_path.to_str().unwrap());
        assert!(result.is_err());
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
    async fn test_validate_directory_path_valid() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let result = FileService::validate_directory_path(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert!(canonical.is_dir());
        assert!(canonical.is_absolute());
    }

    #[test]
    fn test_validate_and_canonicalize_path_with_dot() {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let result = FileService::validate_and_canonicalize_path(".");
        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert_eq!(canonical, current_dir.canonicalize().unwrap());
    }

    #[test]
    fn test_validate_and_canonicalize_path_nonexistent() {
        let result = FileService::validate_and_canonicalize_path("/nonexistent/path/12345");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::FileNotFound(_) => {
                // Expected error
            }
            other => {
                panic!("Expected FileNotFound error, got: {:?}", other);
            }
        }
    }
}
