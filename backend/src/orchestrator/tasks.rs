//! Task implementations for GraphFlow-rs
//!
//! This module contains task structs that wrap our primitives and implement
//! the GraphFlow-rs Task trait. This is the "adapter layer" that connects
//! our primitives to the orchestration framework.
//!
//! Tasks:
//! - RunGeminiTask: Wraps internal_run_gemini
//! - CreateFileTask: Wraps internal_create_file
//!
//! Phase 4F: Tasks now implement graph_flow::Task instead of PlanTask.
//! They use graph_flow::Context for state management and store outputs
//! using keys like "step_X.output" in the context.

use crate::error::AppError;
use crate::orchestrator::primitives::{internal_create_file, internal_run_gemini};
use crate::state::AppState;
use async_trait::async_trait;
use graph_flow::{Context, NextAction, Result as GraphFlowResult, Task, TaskResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Execution context that stores outputs from previous steps
///
/// DEPRECATED: This is being phased out in favor of graph_flow::Context.
/// Kept temporarily for backward compatibility with legacy executor (which is now removed).
/// Will be fully removed in Phase 4J cleanup.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Deprecated - kept for backward compatibility
pub struct ExecutionContext {
    /// Map of step_id -> output string
    outputs: HashMap<String, String>,
    /// Working directory for file operations
    working_dir: Option<String>,
}

#[allow(dead_code)] // Deprecated - kept for backward compatibility
impl ExecutionContext {
    /// Create a new execution context
    pub fn new(working_dir: Option<String>) -> Self {
        Self {
            outputs: HashMap::new(),
            working_dir,
        }
    }

    /// Store the output of a step
    pub fn set_output(&mut self, step_id: &str, output: String) {
        self.outputs.insert(step_id.to_string(), output);
    }

    /// Get the output of a step
    pub fn get_output(&self, step_id: &str) -> Option<&String> {
        self.outputs.get(step_id)
    }

    /// Get working directory
    pub fn working_dir(&self) -> Option<&str> {
        self.working_dir.as_deref()
    }
}

/// Trait for tasks that can be executed as part of a plan
///
/// DEPRECATED: This trait is being phased out in favor of graph_flow::Task.
/// It's kept temporarily for backward compatibility (tests).
/// Will be removed in Phase 4J cleanup.
#[async_trait::async_trait]
#[allow(dead_code)] // Deprecated - kept for backward compatibility
pub trait PlanTask: Send + Sync {
    /// Unique identifier for this task
    fn id(&self) -> &str;

    /// Execute the task with the given context
    async fn execute(
        &self,
        context: &mut ExecutionContext,
        app_state: &Arc<RwLock<AppState>>,
    ) -> Result<String, AppError>;
}

/// Task that runs Gemini with a prompt
///
/// Phase 4F: Now implements graph_flow::Task.
/// Stores output in context under key "step_X.output".
/// AppState is passed via constructor and stored in the task.
pub struct RunGeminiTask {
    /// Step ID (e.g., "step_1")
    step_id: String,
    /// Prompt to send to Gemini
    prompt: String,
    /// Application state (for agent management, working directory)
    app_state: Arc<RwLock<AppState>>,
}

impl RunGeminiTask {
    /// Create a new RunGeminiTask
    pub fn new(step_id: String, prompt: String) -> Self {
        // Note: app_state will be set via with_app_state() method
        // For backward compatibility with existing code, we create with a new AppState
        // In Phase 4G/H, we'll require app_state to be passed during construction
        Self {
            step_id,
            prompt,
            app_state: Arc::new(RwLock::new(AppState::new())),
        }
    }

    /// Set the application state for this task
    #[allow(dead_code)] // Will be used in Phase 4G/H when building graph from plan
    pub fn with_app_state(mut self, app_state: Arc<RwLock<AppState>>) -> Self {
        self.app_state = app_state;
        self
    }
}

#[async_trait]
impl Task for RunGeminiTask {
    fn id(&self) -> &str {
        &self.step_id
    }

    async fn run(&self, context: Context) -> GraphFlowResult<TaskResult> {
        tracing::debug!(
            step_id = %self.step_id,
            prompt_len = self.prompt.len(),
            "Executing RunGeminiTask (graph-flow)"
        );

        // Execute Gemini
        let output = internal_run_gemini(&self.app_state, &self.prompt)
            .await
            .map_err(|e| {
                graph_flow::GraphError::TaskExecutionFailed(format!(
                    "Gemini execution failed in step '{}': {}",
                    self.step_id, e
                ))
            })?;

        // Store output in context for next steps (key: "step_X.output")
        let output_key = format!("{}.output", self.step_id);
        context.set(&output_key, output.clone()).await;

        tracing::debug!(
            step_id = %self.step_id,
            output_len = output.len(),
            "RunGeminiTask completed (graph-flow)"
        );

        Ok(TaskResult::new(Some(output.clone()), NextAction::Continue))
    }
}

// Keep PlanTask implementation for backward compatibility with current executor
#[async_trait::async_trait]
impl PlanTask for RunGeminiTask {
    fn id(&self) -> &str {
        &self.step_id
    }

    async fn execute(
        &self,
        context: &mut ExecutionContext,
        app_state: &Arc<RwLock<AppState>>,
    ) -> Result<String, AppError> {
        tracing::debug!(
            step_id = %self.step_id,
            prompt_len = self.prompt.len(),
            "Executing RunGeminiTask (legacy PlanTask)"
        );

        // Execute Gemini
        let output = internal_run_gemini(app_state, &self.prompt).await?;

        // Store output in context for next steps
        context.set_output(&self.step_id, output.clone());

        tracing::debug!(
            step_id = %self.step_id,
            output_len = output.len(),
            "RunGeminiTask completed (legacy PlanTask)"
        );

        Ok(output)
    }
}

/// Task that creates a file with content
///
/// Phase 4F: Now implements graph_flow::Task.
/// Reads content from context using keys like "step_X.output".
/// AppState is passed via constructor and stored in the task.
pub struct CreateFileTask {
    /// Step ID (e.g., "step_2")
    step_id: String,
    /// Filename to create
    filename: String,
    /// Reference to content from another step (e.g., "step_1.output")
    content_from: Option<String>,
    /// Direct content (if not using content_from)
    direct_content: Option<String>,
    /// Application state (for working directory)
    app_state: Arc<RwLock<AppState>>,
}

impl CreateFileTask {
    /// Create a new CreateFileTask
    pub fn new(step_id: String, filename: String, content_from: Option<String>) -> Self {
        // Note: app_state will be set via with_app_state() method
        // For backward compatibility with existing code, we create with a new AppState
        // In Phase 4G/H, we'll require app_state to be passed during construction
        Self {
            step_id,
            filename,
            content_from,
            direct_content: None,
            app_state: Arc::new(RwLock::new(AppState::new())),
        }
    }

    /// Create a new CreateFileTask with direct content
    #[allow(dead_code)] // May be used in future
    pub fn with_content(step_id: String, filename: String, content: String) -> Self {
        Self {
            step_id,
            filename,
            content_from: None,
            direct_content: Some(content),
            app_state: Arc::new(RwLock::new(AppState::new())),
        }
    }

    /// Set the application state for this task
    #[allow(dead_code)] // Will be used in Phase 4G/H when building graph from plan
    pub fn with_app_state(mut self, app_state: Arc<RwLock<AppState>>) -> Self {
        self.app_state = app_state;
        self
    }
}

#[async_trait]
impl Task for CreateFileTask {
    fn id(&self) -> &str {
        &self.step_id
    }

    async fn run(&self, context: Context) -> GraphFlowResult<TaskResult> {
        tracing::debug!(
            step_id = %self.step_id,
            filename = %self.filename,
            "Executing CreateFileTask (graph-flow)"
        );

        // Validate filename for path traversal protection
        if self.filename.contains("..") || self.filename.starts_with('/') {
            return Err(graph_flow::GraphError::TaskExecutionFailed(format!(
                "Filename '{}' in step '{}' contains invalid characters (path traversal detected or absolute path)",
                self.filename, self.step_id
            )));
        }

        // Also check for null bytes and other dangerous characters
        if self.filename.contains('\0') || self.filename.chars().any(|c| c.is_control()) {
            return Err(graph_flow::GraphError::TaskExecutionFailed(format!(
                "Filename '{}' in step '{}' contains invalid characters (control characters detected)",
                self.filename, self.step_id
            )));
        }

        // Get working directory from context or app_state
        let working_dir = {
            // Try to get from context first (set by graph builder)
            if let Some(wd) = context.get::<String>("working_dir").await {
                Some(wd)
            } else {
                // Fall back to app_state
                let state_read = self.app_state.read().await;
                state_read.working_directory().cloned()
            }
        };

        // Get content from context or use direct content
        let content = if let Some(ref content_from) = self.content_from {
            // Parse "step_1.output" -> get from context using key "step_1.output" or "step_1.output"
            // The context key should match what RunGeminiTask stores
            context.get::<String>(content_from).await.ok_or_else(|| {
                graph_flow::GraphError::TaskExecutionFailed(format!(
                    "Step '{}' references output from '{}' but that step has not been executed yet",
                    self.step_id, content_from
                ))
            })?
        } else if let Some(ref direct) = self.direct_content {
            direct.clone()
        } else {
            return Err(graph_flow::GraphError::TaskExecutionFailed(format!(
                "CreateFileTask '{}' has no content source (neither content_from nor direct_content)",
                self.step_id
            )));
        };

        // Create the file
        let file_path = internal_create_file(&self.filename, &content, working_dir.as_deref())
            .await
            .map_err(|e| {
                graph_flow::GraphError::TaskExecutionFailed(format!(
                    "File creation failed in step '{}': {}",
                    self.step_id, e
                ))
            })?;

        // Store output in context (the file path) using key "step_X.output"
        let output_key = format!("{}.output", self.step_id);
        context.set(&output_key, file_path.clone()).await;

        tracing::debug!(
            step_id = %self.step_id,
            file_path = %file_path,
            "CreateFileTask completed (graph-flow)"
        );

        Ok(TaskResult::new(
            Some(file_path.clone()),
            NextAction::Continue,
        ))
    }
}

// Keep PlanTask implementation for backward compatibility with current executor
#[async_trait::async_trait]
impl PlanTask for CreateFileTask {
    fn id(&self) -> &str {
        &self.step_id
    }

    async fn execute(
        &self,
        context: &mut ExecutionContext,
        _app_state: &Arc<RwLock<AppState>>,
    ) -> Result<String, AppError> {
        tracing::debug!(
            step_id = %self.step_id,
            filename = %self.filename,
            "Executing CreateFileTask (legacy PlanTask)"
        );

        // Validate filename for path traversal protection
        if self.filename.contains("..") || self.filename.starts_with('/') {
            return Err(AppError::InvalidPath(format!(
                "Filename '{}' in step '{}' contains invalid characters (path traversal detected or absolute path)",
                self.filename, self.step_id
            )));
        }

        // Also check for null bytes and other dangerous characters
        if self.filename.contains('\0') || self.filename.chars().any(|c| c.is_control()) {
            return Err(AppError::InvalidPath(format!(
                "Filename '{}' in step '{}' contains invalid characters (control characters detected)",
                self.filename, self.step_id
            )));
        }

        // Get content from context or use direct content
        let content = if let Some(ref content_from) = self.content_from {
            // Parse "step_1.output" -> "step_1"
            let referenced_step_id = content_from.split('.').next().unwrap_or(content_from);
            context
                .get_output(referenced_step_id)
                .ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!(
                        "Step '{}' references output from '{}' but that step has not been executed yet",
                        self.step_id,
                        referenced_step_id
                    ))
                })?
                .clone()
        } else if let Some(ref direct) = self.direct_content {
            direct.clone()
        } else {
            return Err(AppError::Internal(anyhow::anyhow!(
                "CreateFileTask '{}' has no content source (neither content_from nor direct_content)",
                self.step_id
            )));
        };

        // Create the file
        let file_path =
            internal_create_file(&self.filename, &content, context.working_dir()).await?;

        // Store output in context (the file path)
        context.set_output(&self.step_id, file_path.clone());

        tracing::debug!(
            step_id = %self.step_id,
            file_path = %file_path,
            "CreateFileTask completed (legacy PlanTask)"
        );

        Ok(file_path)
    }
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

    #[test]
    fn test_execution_context() {
        let mut ctx = ExecutionContext::new(Some("/tmp".to_string()));

        ctx.set_output("step_1", "Hello, world!".to_string());
        assert_eq!(ctx.get_output("step_1"), Some(&"Hello, world!".to_string()));
        assert_eq!(ctx.get_output("step_2"), None);
        assert_eq!(ctx.working_dir(), Some("/tmp"));
    }

    #[tokio::test]
    async fn test_create_file_task_with_content_from() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap().to_string();

        let mut ctx = ExecutionContext::new(Some(work_dir.clone()));
        ctx.set_output("step_1", "Test content".to_string());

        let task = CreateFileTask::new(
            "step_2".to_string(),
            "test.txt".to_string(),
            Some("step_1.output".to_string()),
        );

        let state = create_test_state();
        let result = task.execute(&mut ctx, &state).await;

        assert!(result.is_ok());
        let file_path = result.unwrap();
        assert!(std::path::Path::new(&file_path).exists());

        // Verify content
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_create_file_task_missing_content() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap().to_string();

        let mut ctx = ExecutionContext::new(Some(work_dir));

        let task = CreateFileTask::new(
            "step_2".to_string(),
            "test.txt".to_string(),
            Some("step_999.output".to_string()), // Non-existent step
        );

        let state = create_test_state();
        let result = task.execute(&mut ctx, &state).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not been executed"));
    }

    #[tokio::test]
    async fn test_run_gemini_task_structure() {
        let task = RunGeminiTask::new("step_1".to_string(), "test prompt".to_string());
        // Both traits have id() method - use PlanTask for backward compatibility check
        assert_eq!(PlanTask::id(&task), "step_1");

        // Full execution test would require Gemini CLI, which is tested elsewhere
    }

    #[test]
    fn test_create_file_task_path_traversal_protection() {
        let task = CreateFileTask::new("step_1".to_string(), "../etc/passwd".to_string(), None);

        // Task should be created, but execution should fail
        // Both traits have id() method - use PlanTask for backward compatibility check
        assert_eq!(PlanTask::id(&task), "step_1");
        // Note: Actual validation happens in execute(), which is tested in test_create_file_task_* tests
    }

    #[tokio::test]
    async fn test_create_file_task_rejects_path_traversal() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap().to_string();

        let mut ctx = ExecutionContext::new(Some(work_dir));
        ctx.set_output("step_1", "Test content".to_string());

        let task = CreateFileTask::new(
            "step_2".to_string(),
            "../etc/passwd".to_string(), // Path traversal attempt
            Some("step_1.output".to_string()),
        );

        let state = create_test_state();
        let result = task.execute(&mut ctx, &state).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("path traversal") || error_msg.contains("absolute path"),
            "Error message should mention path traversal, got: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_create_file_task_rejects_control_characters() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap().to_string();

        let mut ctx = ExecutionContext::new(Some(work_dir));
        ctx.set_output("step_1", "Test content".to_string());

        let task = CreateFileTask::new(
            "step_2".to_string(),
            "test\0file.txt".to_string(), // Null byte
            Some("step_1.output".to_string()),
        );

        let state = create_test_state();
        let result = task.execute(&mut ctx, &state).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("control characters"));
    }
}
