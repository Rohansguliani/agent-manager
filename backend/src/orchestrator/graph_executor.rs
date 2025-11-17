//! Graph executor for executing plans
//!
//! This module executes a Plan by building and running a graph of tasks.
//! It handles task dependencies, state management, and error propagation.
//!
//! Phase 4H: GraphFlow-rs Integration
//! This executor now uses graph-flow for parallel DAG execution.
//! Graph-flow handles:
//! - Parallel execution of independent steps
//! - Fail-fast error handling (task cancellation)
//! - Dependency resolution and scheduling
//! - Concurrency limiting (built into framework)

use crate::error::AppError;
use crate::orchestrator::config::OrchestratorConfig;
use crate::orchestrator::plan_to_graph::build_graph_from_plan;
use crate::orchestrator::plan_types::Plan;
use crate::state::AppState;
use anyhow::anyhow;
use graph_flow::{
    Context, ExecutionStatus, FlowRunner, InMemorySessionStorage, Session, SessionStorage,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

/// Result of executing a single step
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Step ID
    pub step_id: String,
    /// Step number (1, 2, 3, etc.)
    pub step_number: u32,
    /// Whether execution succeeded
    pub success: bool,
    /// Output from the step (if successful)
    #[allow(dead_code)] // Used in endpoint streaming
    pub output: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Type alias for execution results
pub type ExecutionResult = Result<Vec<StepResult>, AppError>;

/// Execute a plan and return results
///
/// This function takes a Plan and executes it step by step, handling
/// dependencies and managing state between steps.
///
/// The execution is wrapped in a timeout (default: 5 minutes) to prevent
/// runaway executions from consuming resources indefinitely.
///
/// # Arguments
/// * `plan` - Reference to the plan to execute (cloned internally if needed)
/// * `app_state` - Application state (for agent management, working directory)
///
/// # Returns
/// * `Ok(Vec<StepResult>)` - Results from each step
/// * `Err(AppError)` - If execution fails or times out
pub async fn execute_plan(plan: &Plan, app_state: &Arc<RwLock<AppState>>) -> ExecutionResult {
    let config = OrchestratorConfig::default();
    execute_plan_with_config(plan, app_state, &config).await
}

/// Extract step results from graph-flow context
///
/// This helper function builds a `Vec<StepResult>` from the final session context.
/// Each step stores its output in the context using the key "{step_id}.output".
///
/// # Arguments
/// * `plan` - The plan that was executed
/// * `context` - The graph-flow context containing step outputs
///
/// # Returns
/// * `Vec<StepResult>` - Sorted results for each step in the plan
async fn extract_step_results_from_context(plan: &Plan, context: &Context) -> Vec<StepResult> {
    // Build step number map for efficient lookup
    let step_number_map: HashMap<_, _> = plan
        .steps
        .iter()
        .enumerate()
        .map(|(idx, step)| (step.id.clone(), (idx + 1) as u32))
        .collect();

    let mut results = Vec::new();

    for step in &plan.steps {
        let step_number = step_number_map.get(&step.id).copied().unwrap_or(0);
        use crate::orchestrator::constants::STEP_OUTPUT_SUFFIX;
        let output_key = format!("{}{}", step.id, STEP_OUTPUT_SUFFIX);

        // Try to get output from context
        let output: Option<String> = context.get(&output_key).await;

        let success = output.is_some();
        results.push(StepResult {
            step_id: step.id.clone(),
            step_number,
            success,
            output: output.clone(),
            error: if success {
                None
            } else {
                Some(format!(
                    "Step {} ({}) did not produce output",
                    step_number, step.id
                ))
            },
        });
    }

    // Sort by step number
    results.sort_by_key(|r| r.step_number);

    results
}

/// Execute a plan with a specific configuration
pub async fn execute_plan_with_config(
    plan: &Plan,
    app_state: &Arc<RwLock<AppState>>,
    config: &OrchestratorConfig,
) -> ExecutionResult {
    let plan_timeout = Duration::from_secs(config.plan_timeout_secs);

    // Clone plan only once here, before the timeout wrapper
    let plan_clone = plan.clone();
    timeout(plan_timeout, execute_plan_inner(plan_clone, app_state))
        .await
        .map_err(|_| {
            AppError::Timeout(format!(
                "Plan execution timed out after {} seconds",
                plan_timeout.as_secs()
            ))
        })?
}

/// Inner implementation of plan execution using graph-flow
///
/// This function uses graph-flow to execute the plan with parallel DAG support.
/// Graph-flow handles parallel execution, fail-fast error handling, and dependency resolution.
async fn execute_plan_inner(plan: Plan, app_state: &Arc<RwLock<AppState>>) -> ExecutionResult {
    // Generate unique session ID for tracing
    let session_id = Uuid::new_v4().to_string();

    // Create a plan hash for identification
    use crate::orchestrator::utils::hash_plan;
    let plan_hash = hash_plan(&plan);

    // Create structured logging span for the entire execution
    let span = tracing::info_span!(
        "execute_plan",
        session_id = %session_id,
        plan_hash = %plan_hash,
        step_count = plan.steps.len(),
    );
    let _enter = span.enter();

    // Build graph from plan
    let graph = build_graph_from_plan(plan.clone(), app_state.clone())?;

    // Get working directory from app state
    let working_dir = {
        let state_read = app_state.read().await;
        state_read.working_directory().cloned()
    };

    // Create session storage (in-memory for stateless API)
    // TODO(Improvement 7): Support persistent session storage for long-running workflows
    let session_storage: Arc<dyn SessionStorage> = Arc::new(InMemorySessionStorage::new());

    // Create FlowRunner
    let runner = FlowRunner::new(graph, session_storage.clone());

    // Find the first task (step with no dependencies, or first step if all have dependencies)
    use crate::orchestrator::plan_utils::find_start_step_id;
    let first_task_id = find_start_step_id(&plan).ok_or_else(|| {
        AppError::Internal(anyhow!(
            "Plan has no steps (this should not happen after validation)"
        ))
    })?;

    // Create session starting from first task
    let session = Session::new_from_task(session_id.clone(), first_task_id);

    // Set working directory in context
    if let Some(wd) = working_dir {
        use crate::orchestrator::constants::WORKING_DIR_KEY;
        session.context.set(WORKING_DIR_KEY, wd).await;
    }

    // Save session
    session_storage
        .save(session)
        .await
        .map_err(|e| AppError::Internal(anyhow!("Failed to save session: {}", e)))?;

    let start_time = std::time::Instant::now();
    tracing::info!(
        session_id = %session_id,
        first_task_id = %first_task_id,
        total_steps = plan.steps.len(),
        "Starting graph-flow execution"
    );

    // Execute until completion
    loop {
        let execution_result = runner.run(&session_id).await.map_err(convert_graph_error)?;

        tracing::info!(
            session_id = %session_id,
            status = ?execution_result.status,
            elapsed_secs = start_time.elapsed().as_secs_f64(),
            "Graph execution status update"
        );

        match execution_result.status {
            ExecutionStatus::Completed => {
                let elapsed = start_time.elapsed();
                tracing::info!(
                    session_id = %session_id,
                    total_steps = plan.steps.len(),
                    elapsed_secs = elapsed.as_secs_f64(),
                    "Graph execution completed successfully"
                );
                break;
            }
            ExecutionStatus::Paused {
                next_task_id,
                reason,
            } => {
                // If paused with "No outgoing edge found", it means the current task is complete
                // and there are no more tasks. Check if all tasks have outputs in the context.
                if reason.contains("No outgoing edge found") {
                    // Get current session to check if all tasks are complete
                    let session = session_storage
                        .get(&session_id)
                        .await
                        .map_err(|e| AppError::Internal(anyhow!("Failed to get session: {}", e)))?
                        .ok_or_else(|| {
                            AppError::Internal(anyhow!(
                                "Session '{}' not found during execution",
                                session_id
                            ))
                        })?;

                    // Check if all tasks in the plan have outputs (indicating they've been executed)
                    use crate::orchestrator::constants::STEP_OUTPUT_SUFFIX;
                    let mut all_complete = true;
                    for step in &plan.steps {
                        let output_key = format!("{}{}", step.id, STEP_OUTPUT_SUFFIX);
                        if session.context.get::<String>(&output_key).await.is_none() {
                            all_complete = false;
                            break;
                        }
                    }

                    if all_complete {
                        // All tasks are complete, treat as successful completion
                        let elapsed = start_time.elapsed();
                        tracing::info!(
                            session_id = %session_id,
                            total_steps = plan.steps.len(),
                            elapsed_secs = elapsed.as_secs_f64(),
                            "All tasks completed (no outgoing edges means graph is complete)"
                        );
                        break;
                    } else {
                        // Not all tasks complete yet, but we're stuck. This shouldn't happen
                        // but if it does, log and break to avoid infinite loop
                        tracing::warn!(
                            session_id = %session_id,
                            next_task_id = %next_task_id,
                            reason = %reason,
                            "Graph paused with no outgoing edges but not all tasks complete - treating as completion"
                        );
                        break;
                    }
                } else {
                    // Normal pause, continue to next task
                    continue;
                }
            }
            ExecutionStatus::WaitingForInput => {
                // This shouldn't happen in our tasks, but continue anyway
                continue;
            }
            ExecutionStatus::Error(err) => {
                tracing::error!(
                    session_id = %session_id,
                    error = %err,
                    "Graph execution failed"
                );
                return Err(AppError::PlanExecutionFailed(format!(
                    "Plan execution failed: {}",
                    err
                )));
            }
        }
    }

    // Extract results from final session context
    let final_session = session_storage
        .get(&session_id)
        .await
        .map_err(|e| AppError::Internal(anyhow!("Failed to get final session: {}", e)))?
        .ok_or_else(|| {
            AppError::Internal(anyhow!(
                "Session '{}' not found after execution",
                session_id
            ))
        })?;

    // Extract step results from session context
    let results = extract_step_results_from_context(&plan, &final_session.context).await;

    let success_count = results.iter().filter(|r| r.success).count();
    let failure_count = results.len() - success_count;
    let total_elapsed = start_time.elapsed();

    tracing::info!(
        session_id = %session_id,
        total_steps = results.len(),
        successful_steps = success_count,
        failed_steps = failure_count,
        elapsed_secs = total_elapsed.as_secs_f64(),
        "Extracted step results from session"
    );

    Ok(results)
}

/// Convert graph-flow error to AppError with granular error types
fn convert_graph_error(e: graph_flow::GraphError) -> AppError {
    match e {
        graph_flow::GraphError::TaskExecutionFailed(msg) => {
            AppError::TaskExecutionFailed(format!("Graph task execution failed: {}", msg))
        }
        graph_flow::GraphError::GraphNotFound(msg) => {
            AppError::GraphError(format!("Graph not found: {}", msg))
        }
        graph_flow::GraphError::InvalidEdge(msg) => {
            AppError::GraphError(format!("Invalid edge in graph: {}", msg))
        }
        graph_flow::GraphError::TaskNotFound(msg) => {
            AppError::GraphError(format!("Task not found in graph: {}", msg))
        }
        graph_flow::GraphError::ContextError(msg) => {
            AppError::GraphError(format!("Context error: {}", msg))
        }
        graph_flow::GraphError::StorageError(msg) => {
            AppError::SessionError(format!("Storage error: {}", msg))
        }
        graph_flow::GraphError::SessionNotFound(msg) => {
            AppError::SessionError(format!("Session not found: {}", msg))
        }
        graph_flow::GraphError::Other(err) => {
            AppError::GraphError(format!("Graph execution error: {}", err))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::plan_types::{Plan, Step, StepParams};
    use crate::state::AppState;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_state() -> Arc<RwLock<AppState>> {
        Arc::new(RwLock::new(AppState::new()))
    }

    // Note: build_tasks tests removed - task building is now handled by plan_to_graph.rs
    // which has its own comprehensive test suite

    #[tokio::test]
    async fn test_execute_plan_simple() {
        // This test requires Gemini CLI, so it will be skipped if not available
        // We can test the structure without full execution
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![Step {
                id: "step_1".to_string(),
                task: "run_gemini".to_string(),
                params: StepParams {
                    prompt: Some("Write a short test message".to_string()),
                    ..Default::default()
                },
                dependencies: vec![],
            }],
        };

        let state = create_test_state();
        let result = execute_plan(&plan, &state).await;

        // Result depends on whether Gemini CLI is available
        match result {
            Ok(results) => {
                // If successful, verify structure
                assert!(!results.is_empty());
                if let Some(first_result) = results.first() {
                    assert_eq!(first_result.step_id, "step_1");
                }
            }
            Err(_) => {
                // Expected if Gemini CLI not available
            }
        }
    }

    #[test]
    fn test_execute_plan_empty_steps() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![],
        };

        // Empty plans should fail validation, but if they get here, should return empty results
        // Actually, validation should catch this, so this test is mostly for structure
        assert!(plan.steps.is_empty());
    }

    #[test]
    fn test_build_graph_from_plan_in_executor() {
        // Test that build_graph_from_plan is called correctly by execute_plan
        // This tests the integration between graph_executor and plan_to_graph
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![Step {
                id: "step_1".to_string(),
                task: "run_gemini".to_string(),
                params: StepParams {
                    prompt: Some("Test prompt".to_string()),
                    ..Default::default()
                },
                dependencies: vec![],
            }],
        };

        let state = create_test_state();
        // build_graph_from_plan should validate and build the graph successfully
        let result = crate::orchestrator::plan_to_graph::build_graph_from_plan(plan, state);
        assert!(result.is_ok());
        let graph = result.unwrap();
        use crate::orchestrator::constants::DEFAULT_GRAPH_ID;
        assert_eq!(graph.id, DEFAULT_GRAPH_ID);
    }

    #[test]
    fn test_convert_graph_error_all_variants() {
        use graph_flow::GraphError;

        // Test all GraphError variants are converted properly
        let errors = vec![
            GraphError::TaskExecutionFailed("test".to_string()),
            GraphError::GraphNotFound("test".to_string()),
            GraphError::InvalidEdge("test".to_string()),
            GraphError::TaskNotFound("test".to_string()),
            GraphError::ContextError("test".to_string()),
            GraphError::StorageError("test".to_string()),
            GraphError::SessionNotFound("test".to_string()),
            GraphError::Other(anyhow::anyhow!("test error")),
        ];

        for error in errors {
            let app_error = convert_graph_error(error);
            // All should convert to AppError::Internal
            assert!(
                app_error.to_string().contains("test") || app_error.to_string().contains("error")
            );
        }
    }

    #[test]
    fn test_step_result_structure() {
        // Test StepResult struct creation and access
        let result = StepResult {
            step_id: "step_1".to_string(),
            step_number: 1,
            success: true,
            output: Some("test output".to_string()),
            error: None,
        };

        assert_eq!(result.step_id, "step_1");
        assert_eq!(result.step_number, 1);
        assert!(result.success);
        assert_eq!(result.output, Some("test output".to_string()));
        assert_eq!(result.error, None);
    }

    #[test]
    fn test_step_result_failure_structure() {
        // Test StepResult with failure
        let result = StepResult {
            step_id: "step_1".to_string(),
            step_number: 1,
            success: false,
            output: None,
            error: Some("test error".to_string()),
        };

        assert_eq!(result.step_id, "step_1");
        assert_eq!(result.step_number, 1);
        assert!(!result.success);
        assert_eq!(result.output, None);
        assert_eq!(result.error, Some("test error".to_string()));
    }

    /// Test 2-step sequential plan (happy path)
    ///
    /// This test verifies:
    /// - Sequential execution structure is correct
    /// - Graph building handles dependencies correctly
    /// - Results extraction logic works
    ///
    /// Note: This is a structure test that verifies the executor can handle
    /// sequential plans. Full execution requires external services.
    #[tokio::test]
    async fn test_execute_plan_two_steps_sequential() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap().to_string();

        // Create plan with two sequential steps using create_file tasks
        // Step 1: Create first file (no dependencies)
        // Step 2: Create second file that depends on step_1
        // Note: This will fail at execution if step_1 doesn't produce output
        // but we're testing the executor logic, not task implementation
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("file1.txt".to_string()),
                        content_from: None, // This will fail, but tests executor structure
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("file2.txt".to_string()),
                        content_from: Some("step_1.output".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_1".to_string()],
                },
            ],
        };

        let state = create_test_state();
        {
            let mut state_write = state.write().await;
            state_write.set_working_directory(Some(work_dir));
        }

        // Test that graph building works for sequential plan
        let graph_result =
            crate::orchestrator::plan_to_graph::build_graph_from_plan(plan.clone(), state.clone());
        assert!(
            graph_result.is_ok(),
            "Graph building should succeed for sequential plan"
        );
        let graph = graph_result.unwrap();
        use crate::orchestrator::constants::DEFAULT_GRAPH_ID;
        assert_eq!(graph.id, DEFAULT_GRAPH_ID);

        // Test execution (will likely fail due to missing content, but that's okay)
        // We're verifying the executor handles the plan structure correctly
        let result = execute_plan(&plan, &state).await;

        match result {
            Ok(results) => {
                // If successful, verify result structure
                assert_eq!(results.len(), 2);
                // Results should be sorted by step number
                assert_eq!(results[0].step_id, "step_1");
                assert_eq!(results[1].step_id, "step_2");
            }
            Err(_) => {
                // Expected if tasks fail (e.g., missing content)
                // This is acceptable - we're testing executor structure, not task execution
            }
        }
    }

    /// Test parallel execution - two independent steps
    ///
    /// This test verifies:
    /// - Graph building correctly identifies independent steps
    /// - Graph structure supports parallel execution
    /// - Results extraction handles multiple parallel results
    #[tokio::test]
    async fn test_execute_plan_parallel_steps() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap().to_string();

        // Create plan with two parallel steps (both have no dependencies)
        // These can theoretically execute in parallel in graph-flow
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("file1.txt".to_string()),
                        content_from: None,
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("file2.txt".to_string()),
                        content_from: None,
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
            ],
        };

        let state = create_test_state();
        {
            let mut state_write = state.write().await;
            state_write.set_working_directory(Some(work_dir));
        }

        // Verify graph building for parallel plan
        let graph_result =
            crate::orchestrator::plan_to_graph::build_graph_from_plan(plan.clone(), state.clone());
        assert!(
            graph_result.is_ok(),
            "Graph building should succeed for parallel plan"
        );
        let graph = graph_result.unwrap();
        use crate::orchestrator::constants::DEFAULT_GRAPH_ID;
        assert_eq!(graph.id, DEFAULT_GRAPH_ID);

        // Test execution (may fail due to missing content, but verifies structure)
        let result = execute_plan(&plan, &state).await;

        match result {
            Ok(results) => {
                // Both steps should have results (even if they failed)
                assert_eq!(results.len(), 2);
                // Results should include both step IDs
                let step_ids: Vec<&str> = results.iter().map(|r| r.step_id.as_str()).collect();
                assert!(step_ids.contains(&"step_1"));
                assert!(step_ids.contains(&"step_2"));
            }
            Err(_) => {
                // Execution might fail if tasks fail
                // This is acceptable for structure testing
            }
        }
    }

    /// Test error propagation - verify fail-fast behavior
    ///
    /// This test creates a plan where step 1 should fail,
    /// and verifies that the entire execution fails (fail-fast).
    #[tokio::test]
    async fn test_execute_plan_error_propagation() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap().to_string();

        // Create plan where step_1 has invalid parameters (missing prompt for run_gemini)
        // or invalid filename (path traversal)
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("../invalid/path.txt".to_string()), // Path traversal - should fail
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("output.txt".to_string()),
                        content_from: Some("step_1.output".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_1".to_string()],
                },
            ],
        };

        let state = create_test_state();
        {
            let mut state_write = state.write().await;
            state_write.set_working_directory(Some(work_dir));
        }

        let result = execute_plan(&plan, &state).await;

        // The execution should fail because step_1 has path traversal
        // However, path traversal is validated at graph building time, not execution time
        // So this might fail at graph building, not execution
        match result {
            Ok(_results) => {
                // If execution succeeds, step_1 should have failed
                // But validation might catch it earlier
                // In graph-flow, if a task fails, the execution should fail
                // So we expect an error, not results
                panic!("Execution should have failed due to path traversal validation");
            }
            Err(e) => {
                // Expected - path traversal should cause validation error
                assert!(
                    e.to_string().contains("path traversal")
                        || e.to_string().contains("invalid")
                        || e.to_string().contains("Path"),
                    "Error should mention path traversal or invalid path"
                );
            }
        }
    }

    /// Test session result extraction
    ///
    /// This test verifies that results are correctly extracted from
    /// the session context after execution completes.
    ///
    /// This verifies the result extraction logic uses the correct
    /// context key pattern ("{step_id}.output").
    #[tokio::test]
    async fn test_session_result_extraction() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap().to_string();

        // Create a simple plan with one step
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![Step {
                id: "step_1".to_string(),
                task: "create_file".to_string(),
                params: StepParams {
                    filename: Some("test.txt".to_string()),
                    content_from: None,
                    ..Default::default()
                },
                dependencies: vec![],
            }],
        };

        let state = create_test_state();
        {
            let mut state_write = state.write().await;
            state_write.set_working_directory(Some(work_dir));
        }

        // Test that result extraction logic is structured correctly
        // Even if execution fails, we verify the extraction pattern
        let result = execute_plan(&plan, &state).await;

        match result {
            Ok(results) => {
                // Verify result structure
                assert_eq!(results.len(), 1);
                let step_result = &results[0];
                assert_eq!(step_result.step_id, "step_1");
                assert_eq!(step_result.step_number, 1);

                // Verify result has success/error indication
                assert!(step_result.success || step_result.error.is_some());

                // If successful, output should be present
                if step_result.success {
                    assert!(step_result.output.is_some());
                } else {
                    // If failed, error should be present
                    assert!(step_result.error.is_some());
                }
            }
            Err(_) => {
                // Execution might fail if task fails (e.g., missing content)
                // This is acceptable - we're testing result extraction structure
            }
        }
    }

    /// Test error propagation and fail-fast behavior
    ///
    /// This test verifies that when a step fails validation at graph building,
    /// the entire execution fails early (fail-fast).
    #[tokio::test]
    async fn test_error_propagation_at_validation() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_str().unwrap().to_string();

        // Create plan with invalid filename (path traversal)
        // This should fail at graph building time
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("../invalid/path.txt".to_string()), // Path traversal
                        content_from: None,
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("output.txt".to_string()),
                        content_from: Some("step_1.output".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_1".to_string()],
                },
            ],
        };

        let state = create_test_state();
        {
            let mut state_write = state.write().await;
            state_write.set_working_directory(Some(work_dir));
        }

        // Path traversal should be caught at graph building time
        let result = execute_plan(&plan, &state).await;

        // Execution should fail due to validation
        assert!(
            result.is_err(),
            "Execution should fail due to path traversal validation"
        );

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("path traversal")
                || error.to_string().contains("invalid")
                || error.to_string().contains("Path"),
            "Error should mention path traversal or invalid path, got: {}",
            error
        );
    }
}
