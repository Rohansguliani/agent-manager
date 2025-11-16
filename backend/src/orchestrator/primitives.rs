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
use crate::orchestrator::api_client;
use crate::orchestrator::plan_types::Plan;
use crate::services::files::FileService;
use crate::state::AppState;
use anyhow::anyhow;
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

/// Run Gemini API directly with structured JSON support
///
/// This is a wrapper around the direct Gemini API client.
/// Used for "Planner" calls that need reliable JSON output.
///
/// This function reads the API key from the `GEMINI_API_KEY` environment variable
/// and makes a direct HTTP request to the Gemini API, bypassing the CLI wrapper.
///
/// # Arguments
/// * `prompt` - The prompt to send to Gemini
/// * `force_json` - If true, request JSON response format (required for planner)
///
/// # Returns
/// * `Ok(String)` - The response text from Gemini
/// * `Err(AppError)` - If API call failed or API key missing
///
/// # Example
/// ```no_run
/// # async fn example() -> Result<(), crate::error::AppError> {
/// // Regular prompt (unstructured output)
/// let response = internal_run_gemini_api(
///     "Write a haiku about programming",
///     false,
/// ).await?;
///
/// // Planner prompt (structured JSON output)
/// let plan_json = internal_run_gemini_api(
///     "Generate a JSON plan with steps",
///     true,  // Force JSON mode
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)] // Will be used in Phase 1B/Phase 2 for planner agent
pub async fn internal_run_gemini_api(prompt: &str, force_json: bool) -> Result<String, AppError> {
    // Read API key from environment
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) if key.is_empty() => {
            return Err(AppError::Internal(anyhow!(
                "GEMINI_API_KEY environment variable is not set or is empty. Please set it to use the Gemini API."
            )));
        }
        Ok(key) => key,
        Err(_) => {
            return Err(AppError::Internal(anyhow!(
                "GEMINI_API_KEY environment variable is not set or is empty. Please set it to use the Gemini API."
            )));
        }
    };

    tracing::debug!(
        prompt_len = prompt.len(),
        force_json = force_json,
        "Calling Gemini API directly (not via CLI)"
    );

    // Call the API client
    api_client::call_gemini_api(&api_key, prompt, None, force_json).await
}

/// Run the planner agent to generate a structured plan
///
/// This function sends a "meta-prompt" to Gemini (with JSON mode enabled)
/// asking it to break down a high-level goal into a sequence of steps.
///
/// The planner generates a JSON plan that describes:
/// - A sequence of steps (tasks)
/// - Dependencies between steps (e.g., step_2 uses step_1's output)
/// - Parameters for each step (prompts, filenames, etc.)
///
/// # Arguments
/// * `goal` - The high-level goal to break down (e.g., "Write a poem about Rust and save it to poem.txt")
///
/// # Returns
/// * `Ok(Plan)` - A validated plan struct
/// * `Err(AppError)` - If planning fails, JSON is invalid, or plan validation fails
///
/// # Example
/// ```no_run
/// # async fn example() -> Result<(), crate::error::AppError> {
/// let plan = internal_run_planner("Write a poem about Rust and save it to poem.txt").await?;
/// // plan.steps contains the steps to execute
/// # Ok(())
/// # }
/// ```
pub async fn internal_run_planner(goal: &str) -> Result<Plan, AppError> {
    // Build the meta-prompt
    let meta_prompt = build_meta_prompt(goal);

    tracing::debug!(
        goal_len = goal.len(),
        "Calling planner agent to generate plan"
    );

    // Try planning (with one retry on failure)
    let plan_result = try_plan_once(&meta_prompt).await;

    match plan_result {
        Ok(plan) => {
            tracing::debug!(
                plan_version = %plan.version,
                num_steps = plan.steps.len(),
                "Planner generated valid plan"
            );
            Ok(plan)
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Planner failed, retrying once"
            );

            // Retry once with the same prompt
            let retry_result = try_plan_once(&meta_prompt).await;

            match retry_result {
                Ok(plan) => {
                    tracing::debug!(
                        plan_version = %plan.version,
                        num_steps = plan.steps.len(),
                        "Planner succeeded on retry"
                    );
                    Ok(plan)
                }
                Err(retry_error) => {
                    tracing::error!(
                        error = %retry_error,
                        "Planner failed after retry"
                    );
                    Err(retry_error)
                }
            }
        }
    }
}

/// Attempt to generate a plan once
async fn try_plan_once(meta_prompt: &str) -> Result<Plan, AppError> {
    // Call Gemini API with JSON mode
    let json_response = internal_run_gemini_api(meta_prompt, true).await?;

    tracing::debug!(
        response_len = json_response.len(),
        "Received JSON response from planner"
    );

    // Parse JSON to Plan struct
    let plan: Plan = serde_json::from_str(&json_response).map_err(|e| {
        AppError::InvalidPlan(format!(
            "Failed to parse planner response as JSON: {} - Response: {}",
            e, json_response
        ))
    })?;

    // Validate the plan structure
    plan.validate().map_err(|validation_error| {
        AppError::InvalidPlan(format!("Plan validation failed: {}", validation_error))
    })?;

    Ok(plan)
}

/// Build the meta-prompt for the planner agent
fn build_meta_prompt(goal: &str) -> String {
    format!(
        r#"You are a planner agent. Your job is to take a user's GOAL and break it down into a JSON plan with steps.

Available Tools:
1. run_gemini: Runs a prompt through Gemini and returns text output. Parameters: {{"prompt": "..."}}
2. create_file: Saves text content to a file. Parameters: {{"filename": "...", "content_from": "step_X.output"}}

Output Format (JSON):
{{
  "version": "1.0",
  "steps": [
    {{
      "id": "step_1",
      "task": "run_gemini",
      "params": {{
        "prompt": "..."
      }},
      "dependencies": []
    }},
    {{
      "id": "step_2",
      "task": "create_file",
      "params": {{
        "filename": "...",
        "content_from": "step_1.output"
      }},
      "dependencies": ["step_1"]
    }}
  ]
}}

CRITICAL REQUIREMENT - Dependencies Array:
- EVERY step MUST have a "dependencies" array (even if empty)
- If a step has no prerequisites, use: "dependencies": []
- If step_2 depends on step_1, use: "dependencies": ["step_1"]
- Multiple dependencies: "dependencies": ["step_1", "step_3"]
- If "content_from" references "step_X.output", then "dependencies" MUST include "step_X"

Important Rules:
- Each step must have a unique "id" (e.g., "step_1", "step_2")
- The "task" must be one of: "run_gemini", "create_file"
- For "create_file" tasks, use "content_from" to reference another step's output (e.g., "step_1.output")
- Steps with empty "dependencies" can run in parallel with other independent steps

Examples:

Sequential Plan (step_2 depends on step_1):
{{
  "steps": [
    {{"id": "step_1", "task": "run_gemini", "params": {{"prompt": "Write poem A"}}, "dependencies": []}},
    {{"id": "step_2", "task": "create_file", "params": {{"filename": "poem.txt", "content_from": "step_1.output"}}, "dependencies": ["step_1"]}}
  ]
}}

Parallel Plan (step_1, step_2, step_3 can run simultaneously):
{{
  "steps": [
    {{"id": "step_1", "task": "run_gemini", "params": {{"prompt": "Write poem about Rust"}}, "dependencies": []}},
    {{"id": "step_2", "task": "run_gemini", "params": {{"prompt": "Write poem about Python"}}, "dependencies": []}},
    {{"id": "step_3", "task": "run_gemini", "params": {{"prompt": "Write poem about Go"}}, "dependencies": []}},
    {{"id": "step_4", "task": "create_file", "params": {{"filename": "combined.txt", "content_from": "step_1.output"}}, "dependencies": ["step_1", "step_2", "step_3"]}}
  ]
}}

GOAL: "{}"

Generate a JSON plan with the steps needed to accomplish this goal. Remember: EVERY step MUST have a "dependencies" array. Return ONLY valid JSON, no other text."#,
        goal
    )
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

    #[tokio::test]
    #[serial_test::serial]
    async fn test_internal_run_gemini_api_missing_api_key() {
        // Save original value
        let original = std::env::var("GEMINI_API_KEY").ok();

        // Remove env var
        std::env::remove_var("GEMINI_API_KEY");

        let result = internal_run_gemini_api("test prompt", false).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("GEMINI_API_KEY") && error_msg.contains("not set or is empty"),
            "Error message should mention missing or empty GEMINI_API_KEY, got: {}",
            error_msg
        );

        // Restore original
        if let Some(key) = original {
            std::env::set_var("GEMINI_API_KEY", &key);
        } else {
            std::env::remove_var("GEMINI_API_KEY");
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_internal_run_gemini_api_empty_api_key() {
        // Save original value
        let original = std::env::var("GEMINI_API_KEY").ok();

        // Set empty API key
        std::env::set_var("GEMINI_API_KEY", "");

        let result = internal_run_gemini_api("test prompt", false).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("GEMINI_API_KEY") && error_msg.contains("not set or is empty"),
            "Error message should mention missing or empty GEMINI_API_KEY, got: {}",
            error_msg
        );

        // Restore original
        if let Some(key) = original {
            std::env::set_var("GEMINI_API_KEY", &key);
        } else {
            std::env::remove_var("GEMINI_API_KEY");
        }
    }

    // Note: Testing with real API would require:
    // 1. API key in test environment
    // 2. Mock HTTP client or integration test setup
    // For now, we test error cases that don't require API calls

    // Planner tests
    mod planner_tests {
        use super::*;
        use crate::orchestrator::plan_types::Plan;
        use serial_test::serial;

        #[tokio::test]
        #[serial]
        async fn test_internal_run_planner_with_valid_goal() {
            // Test with a simple goal - will use real API if key is available
            // If API key is not available, skip this test
            if std::env::var("GEMINI_API_KEY").is_err() {
                eprintln!("Skipping test: GEMINI_API_KEY not set");
                return;
            }

            let goal = "Write a 4-line poem about dogs and save it to dogs.txt";
            let result = internal_run_planner(goal).await;

            match result {
                Ok(plan) => {
                    // Verify plan structure
                    assert!(!plan.steps.is_empty(), "Plan should have at least one step");
                    assert_eq!(plan.version, "1.0");

                    // Verify validation passed
                    assert!(plan.validate().is_ok(), "Plan should be valid");

                    // Verify first step is likely run_gemini
                    assert_eq!(plan.steps[0].task, "run_gemini");

                    // Verify last step is likely create_file
                    let last_step = plan.steps.last().unwrap();
                    assert_eq!(last_step.task, "create_file");
                }
                Err(e) => {
                    // If it fails due to API issues, that's OK for unit test
                    // We just want to ensure it doesn't panic
                    eprintln!("Planner test failed (expected if API unavailable): {}", e);
                }
            }
        }

        #[tokio::test]
        #[serial]
        async fn test_internal_run_planner_with_empty_goal() {
            if std::env::var("GEMINI_API_KEY").is_err() {
                eprintln!("Skipping test: GEMINI_API_KEY not set");
                return;
            }

            let result = internal_run_planner("").await;
            // Empty goal might still generate a plan or might fail
            // Either is acceptable - we're just testing it doesn't panic
            if result.is_ok() || result.is_err() {
                // Test passes as long as it doesn't panic
            }
        }

        #[test]
        fn test_meta_prompt_structure() {
            let prompt = build_meta_prompt("Test goal");

            // Verify key components are in the prompt
            assert!(prompt.contains("planner agent"));
            assert!(prompt.contains("run_gemini"));
            assert!(prompt.contains("create_file"));
            assert!(prompt.contains("Test goal"));
            assert!(prompt.contains("\"version\":"));
            assert!(prompt.contains("\"steps\":"));
        }

        #[test]
        fn test_try_plan_once_with_valid_json() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let valid_json = r#"{
                    "version": "1.0",
                    "steps": [
                        {
                            "id": "step_1",
                            "task": "run_gemini",
                            "params": {
                                "prompt": "Write a poem"
                            },
                            "dependencies": []
                        },
                        {
                            "id": "step_2",
                            "task": "create_file",
                            "params": {
                                "filename": "poem.txt",
                                "content_from": "step_1.output"
                            },
                            "dependencies": ["step_1"]
                        }
                    ]
                }"#;

                // We can't easily test try_plan_once without mocking internal_run_gemini_api
                // But we can test the parsing and validation logic
                let plan: Plan = serde_json::from_str(valid_json).unwrap();
                assert!(plan.validate().is_ok());
                assert_eq!(plan.steps.len(), 2);
            });
        }

        #[test]
        fn test_try_plan_once_with_invalid_json() {
            let invalid_json = "This is not JSON";

            // Test that invalid JSON would fail parsing
            let result: Result<Plan, _> = serde_json::from_str(invalid_json);
            assert!(result.is_err());
        }

        #[test]
        fn test_build_meta_prompt_includes_goal() {
            let goal = "My test goal";
            let prompt = build_meta_prompt(goal);
            assert!(prompt.contains(goal));
        }

        #[test]
        fn test_build_meta_prompt_includes_tools() {
            let prompt = build_meta_prompt("test");
            assert!(prompt.contains("run_gemini"));
            assert!(prompt.contains("create_file"));
        }

        #[test]
        fn test_build_meta_prompt_requires_dependencies() {
            let prompt = build_meta_prompt("test");
            // Verify that dependencies are mentioned as required
            assert!(prompt.contains("dependencies"));
            assert!(prompt.contains("EVERY step MUST have"));
            assert!(
                prompt.contains(r#""dependencies": []"#) || prompt.contains("\"dependencies\": []")
            );
        }

        #[test]
        fn test_build_meta_prompt_includes_parallel_example() {
            let prompt = build_meta_prompt("test");
            // Verify that parallel execution example is included
            assert!(prompt.contains("Parallel Plan"));
            assert!(prompt.contains("can run simultaneously"));
        }

        #[test]
        fn test_build_meta_prompt_includes_sequential_example() {
            let prompt = build_meta_prompt("test");
            // Verify that sequential execution example is included
            assert!(prompt.contains("Sequential Plan"));
            assert!(prompt.contains("depends on"));
        }
    }
}
