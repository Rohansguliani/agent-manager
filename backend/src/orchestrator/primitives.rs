//! Orchestrator primitives
//!
//! Reusable functions that wrap existing services (CliExecutor, FileService)
//! to provide clean, testable building blocks for orchestration.
//!
//! These primitives are intentionally designed to be:
//! - Reusable: Can be used by multiple orchestration workflows
//! - Testable: Each primitive can be tested independently
//! - Composable: Easy to chain together in orchestration logic

use crate::api::utils::find_or_create_gemini_agent;
use crate::error::AppError;
use crate::executor::CliExecutor;
use crate::services::files::FileService;
use crate::state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Run Gemini agent with a prompt and return the full result (non-streaming)
///
/// This is a wrapper around `CliExecutor` that:
/// - Finds or creates a Gemini agent with proper context
/// - Executes the query non-streaming (waits for full result)
/// - Returns the complete output as a String
///
/// # Arguments
/// * `state` - Application state (for agent management)
/// * `prompt` - The prompt to send to Gemini
///
/// # Returns
/// * `Ok(String)` - The full response from Gemini
/// * `Err(AppError)` - If execution failed
///
/// # Example
/// ```no_run
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// # async fn example() -> Result<(), crate::error::AppError> {
/// # let state = Arc::new(RwLock::new(crate::state::AppState::new()));
/// let poem = internal_run_gemini(&state, "create a 4-line poem about Rust").await?;
/// # Ok(())
/// # }
/// ```
pub async fn internal_run_gemini(
    state: &Arc<RwLock<AppState>>,
    prompt: &str,
) -> Result<String, AppError> {
    // Find or create Gemini agent (automatically applies working directory context)
    let agent = find_or_create_gemini_agent(state).await;

    // Create executor with 30 second timeout
    let executor = CliExecutor::new(30);

    // Execute and wait for full result (non-streaming)
    executor
        .execute(&agent, prompt)
        .await
        .map_err(AppError::ExecutionError)
}

/// Create or write a file with the given content
///
/// This is a wrapper around `FileService::write_file` that provides
/// a clean interface for orchestration workflows.
///
/// # Arguments
/// * `file_path` - Path to the file (can be relative or absolute)
/// * `content` - Content to write to the file
/// * `working_dir` - Optional working directory context (for relative paths)
///
/// # Returns
/// * `Ok(String)` - The canonicalized absolute path of the created file
/// * `Err(AppError)` - If file cannot be created or written
///
/// # Example
/// ```no_run
/// # async fn example() -> Result<(), crate::error::AppError> {
/// let file_path = internal_create_file(
///     "poem.txt",
///     "Here is my poem...",
///     Some("/host/home/dev"),
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn internal_create_file(
    file_path: &str,
    content: &str,
    working_dir: Option<&str>,
) -> Result<String, AppError> {
    let canonical_path = FileService::write_file(file_path, content, working_dir).await?;
    Ok(canonical_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::sync::RwLock;

    fn create_test_state() -> Arc<RwLock<AppState>> {
        Arc::new(RwLock::new(AppState::new()))
    }

    #[tokio::test]
    async fn test_internal_create_file_simple() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, world!";

        let result = internal_create_file(file_path.to_str().unwrap(), content, None).await;

        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert!(std::path::Path::new(&canonical).exists());

        // Verify content
        let written_content = std::fs::read_to_string(&canonical).expect("Failed to read file");
        assert_eq!(written_content, content);
    }

    #[tokio::test]
    async fn test_internal_create_file_with_working_dir() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap();
        let file_path = "subdir/test.txt";
        let content = "Test content";

        let result = internal_create_file(file_path, content, Some(work_dir)).await;

        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert!(std::path::Path::new(&canonical).exists());
        assert!(canonical.contains("subdir"));
        assert!(canonical.contains("test.txt"));

        // Verify content
        let written_content = std::fs::read_to_string(&canonical).expect("Failed to read file");
        assert_eq!(written_content, content);
    }

    #[tokio::test]
    async fn test_internal_create_file_creates_parent_dirs() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("nested/deep/path/test.txt");
        let content = "Nested content";

        let result = internal_create_file(file_path.to_str().unwrap(), content, None).await;

        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert!(std::path::Path::new(&canonical).exists());

        // Verify parent directories were created
        let parent = std::path::Path::new(&canonical).parent().unwrap();
        assert!(parent.exists());
        assert!(parent.ends_with("deep/path"));
    }

    #[tokio::test]
    async fn test_internal_run_gemini_with_state() {
        // This test verifies that internal_run_gemini can create a Gemini agent
        // from state, but doesn't actually run Gemini (would require real CLI)
        // We test that it doesn't panic and handles state correctly
        let state = create_test_state();

        // Should be able to call it (will fail if Gemini CLI not available, but that's OK for unit test)
        // We're testing that the function structure works, not that Gemini actually runs
        let result = internal_run_gemini(&state, "test prompt").await;

        // Result will be Err if Gemini CLI is not available, which is expected in test environment
        // We just verify the function doesn't panic and returns an AppError variant
        match result {
            Ok(_) => {
                // If Gemini is available and works, that's fine
            }
            Err(e) => {
                // Expected if Gemini CLI not available in test environment
                // Verify it's an ExecutionError (from executor) or Internal (from state)
                match e {
                    AppError::ExecutionError(_) => {
                        // Expected - Gemini CLI might not be available
                    }
                    AppError::Internal(_) => {
                        // Also acceptable - state error
                    }
                    _ => {
                        panic!("Unexpected error type: {:?}", e);
                    }
                }
            }
        }
    }
}
