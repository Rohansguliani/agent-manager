//! File system service
//!
//! Provides file system operations with proper error handling and validation.

use crate::error::AppError;
use anyhow::anyhow;
use serde::Serialize;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::warn;

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

            // Try to read metadata, but skip entries that can't be read
            // This handles symlinks, deleted files, and permission issues gracefully
            let metadata = match entry.metadata().await {
                Ok(meta) => meta,
                Err(e) => {
                    // Check the error kind
                    match e.kind() {
                        // File doesn't exist (e.g., broken symlink, deleted file)
                        ErrorKind::NotFound => {
                            warn!(
                                path = %entry_path.display(),
                                "Skipping entry that no longer exists (possibly a broken symlink)"
                            );
                            continue;
                        }
                        // Permission denied - log but continue (don't fail entire operation)
                        ErrorKind::PermissionDenied => {
                            warn!(
                                path = %entry_path.display(),
                                "Skipping entry due to permission denied"
                            );
                            continue;
                        }
                        // Other errors - log and skip
                        _ => {
                            warn!(
                                path = %entry_path.display(),
                                error = %e,
                                "Skipping entry due to error reading metadata"
                            );
                            continue;
                        }
                    }
                }
            };

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

    /// Write content to a file
    ///
    /// # Arguments
    /// * `file_path` - Path to the file (can be relative or absolute)
    /// * `content` - Content to write to the file
    /// * `working_dir` - Optional working directory context (for relative paths)
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Canonicalized absolute path of the created file
    /// * `Err(AppError)` - If file cannot be created or written
    pub async fn write_file(
        file_path: &str,
        content: &str,
        working_dir: Option<&str>,
    ) -> Result<PathBuf, AppError> {
        let path = Path::new(file_path);

        // If path is relative and working_dir is provided, resolve relative to working_dir
        let absolute_path = if path.is_relative() {
            if let Some(work_dir) = working_dir {
                tracing::debug!(
                    working_dir_input = %work_dir,
                    relative_path = %file_path,
                    "FileService::write_file: Resolving relative path with working directory"
                );
                let work_dir_path = Self::validate_directory_path(work_dir)?;
                let resolved = work_dir_path.join(path);
                tracing::debug!(
                    resolved_path = %resolved.display(),
                    "FileService::write_file: Resolved absolute path"
                );
                resolved
            } else {
                // Use current directory
                std::env::current_dir()
                    .map_err(|e| {
                        AppError::Internal(anyhow!("Failed to get current directory: {}", e))
                    })?
                    .join(path)
            }
        } else {
            path.to_path_buf()
        };

        // Create parent directories if they don't exist
        if let Some(parent) = absolute_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AppError::Internal(anyhow!(
                    "Failed to create parent directories for {}: {}",
                    file_path,
                    e
                ))
            })?;
        }

        // Write the file
        fs::write(&absolute_path, content).await.map_err(|e| {
            AppError::Internal(anyhow!("Failed to write file {}: {}", file_path, e))
        })?;

        // Canonicalize the path
        let canonical = absolute_path
            .canonicalize()
            .map_err(|e| AppError::InvalidPath(format!("Failed to canonicalize path: {}", e)))?;

        Ok(canonical)
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

    #[tokio::test]
    async fn test_write_file_simple() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, world!";

        let result = FileService::write_file(file_path.to_str().unwrap(), content, None).await;

        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert!(canonical.exists());
        assert!(canonical.is_file());

        // Verify content
        let written_content = std::fs::read_to_string(&canonical).expect("Failed to read file");
        assert_eq!(written_content, content);
    }

    #[tokio::test]
    async fn test_write_file_with_working_dir() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap();
        let file_path = "subdir/test.txt";
        let content = "Test content";

        let result = FileService::write_file(file_path, content, Some(work_dir)).await;

        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert!(canonical.exists());
        assert!(canonical.is_file());
        assert!(canonical.parent().unwrap().exists());
        assert_eq!(canonical.parent().unwrap().file_name().unwrap(), "subdir");

        // Verify content
        let written_content = std::fs::read_to_string(&canonical).expect("Failed to read file");
        assert_eq!(written_content, content);
    }
}
