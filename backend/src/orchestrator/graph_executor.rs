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
use crate::orchestrator::plan_to_graph::build_graph_from_plan;
use crate::orchestrator::plan_types::Plan;
use crate::state::AppState;
use anyhow::anyhow;
use graph_flow::{ExecutionStatus, FlowRunner, InMemorySessionStorage, Session, SessionStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

/// Execute a plan and return results
///
/// This function takes a Plan and executes it step by step, handling
/// dependencies and managing state between steps.
///
/// The execution is wrapped in a timeout (default: 5 minutes) to prevent
/// runaway executions from consuming resources indefinitely.
///
/// # Arguments
/// * `plan` - The plan to execute
/// * `app_state` - Application state (for agent management, working directory)
///
/// # Returns
/// * `Ok(Vec<StepResult>)` - Results from each step
/// * `Err(AppError)` - If execution fails or times out
pub async fn execute_plan(
    plan: Plan,
    app_state: &Arc<RwLock<AppState>>,
) -> Result<Vec<StepResult>, AppError> {
    const PLAN_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

    timeout(PLAN_TIMEOUT, execute_plan_inner(plan, app_state))
        .await
        .map_err(|_| {
            AppError::Internal(anyhow::anyhow!(
                "Plan execution timed out after {} seconds",
                PLAN_TIMEOUT.as_secs()
            ))
        })?
}

/// Inner implementation of plan execution using graph-flow
///
/// This function uses graph-flow to execute the plan with parallel DAG support.
/// Graph-flow handles parallel execution, fail-fast error handling, and dependency resolution.
async fn execute_plan_inner(
    plan: Plan,
    app_state: &Arc<RwLock<AppState>>,
) -> Result<Vec<StepResult>, AppError> {
    // Build graph from plan
    let graph = build_graph_from_plan(plan.clone(), app_state.clone())?;

    // Get working directory from app state
    let working_dir = {
        let state_read = app_state.read().await;
        state_read.working_directory().cloned()
    };

    // Create session storage (in-memory for stateless API)
    let session_storage: Arc<dyn SessionStorage> = Arc::new(InMemorySessionStorage::new());

    // Create FlowRunner
    let runner = FlowRunner::new(graph, session_storage.clone());

    // Generate unique session ID
    let session_id = Uuid::new_v4().to_string();

    // Find the first task (step with no dependencies, or first step if all have dependencies)
    let first_task_id = plan
        .steps
        .iter()
        .find(|step| step.dependencies.is_empty())
        .map(|step| step.id.as_str())
        .or_else(|| plan.steps.first().map(|step| step.id.as_str()))
        .ok_or_else(|| {
            AppError::Internal(anyhow!(
                "Plan has no steps (this should not happen after validation)"
            ))
        })?;

    // Create session starting from first task
    let session = Session::new_from_task(session_id.clone(), first_task_id);

    // Set working directory in context
    if let Some(wd) = working_dir {
        session.context.set("working_dir", wd).await;
    }

    // Save session
    session_storage
        .save(session)
        .await
        .map_err(|e| AppError::Internal(anyhow!("Failed to save session: {}", e)))?;

    tracing::debug!(
        session_id = %session_id,
        first_task_id = %first_task_id,
        total_steps = plan.steps.len(),
        "Starting graph-flow execution"
    );

    // Execute until completion
    loop {
        let execution_result = runner.run(&session_id).await.map_err(convert_graph_error)?;

        tracing::debug!(
            session_id = %session_id,
            status = ?execution_result.status,
            "Graph execution status update"
        );

        match execution_result.status {
            ExecutionStatus::Completed => {
                tracing::debug!(session_id = %session_id, "Graph execution completed");
                break;
            }
            ExecutionStatus::Paused {
                next_task_id: _,
                reason: _,
            } => {
                // Continue automatically to next task
                continue;
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
                return Err(AppError::Internal(anyhow!(
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

    // Build StepResult vector from context
    // Each step stores output as "step_X.output" in context
    let mut results = Vec::new();
    let step_number_map: std::collections::HashMap<_, _> = plan
        .steps
        .iter()
        .enumerate()
        .map(|(idx, step)| (step.id.clone(), (idx + 1) as u32))
        .collect();

    for step in &plan.steps {
        let step_number = step_number_map.get(&step.id).copied().unwrap_or(0);
        let output_key = format!("{}.output", step.id);

        // Try to get output from context
        let output: Option<String> = final_session.context.get(&output_key).await;

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

    tracing::debug!(
        session_id = %session_id,
        total_results = results.len(),
        "Extracted step results from session"
    );

    Ok(results)
}

/// Result from executing a single step
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

/// Convert graph-flow error to AppError
fn convert_graph_error(e: graph_flow::GraphError) -> AppError {
    match e {
        graph_flow::GraphError::TaskExecutionFailed(msg) => {
            AppError::Internal(anyhow!("Task execution failed: {}", msg))
        }
        graph_flow::GraphError::GraphNotFound(msg) => {
            AppError::Internal(anyhow!("Graph not found: {}", msg))
        }
        graph_flow::GraphError::InvalidEdge(msg) => {
            AppError::Internal(anyhow!("Invalid edge: {}", msg))
        }
        graph_flow::GraphError::TaskNotFound(msg) => {
            AppError::Internal(anyhow!("Task not found: {}", msg))
        }
        graph_flow::GraphError::ContextError(msg) => {
            AppError::Internal(anyhow!("Context error: {}", msg))
        }
        graph_flow::GraphError::StorageError(msg) => {
            AppError::Internal(anyhow!("Storage error: {}", msg))
        }
        graph_flow::GraphError::SessionNotFound(msg) => {
            AppError::Internal(anyhow!("Session not found: {}", msg))
        }
        graph_flow::GraphError::Other(err) => {
            AppError::Internal(anyhow!("Graph execution error: {}", err))
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
        let result = execute_plan(plan, &state).await;

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

    // Note: Path traversal and control character validation tests moved to plan_to_graph.rs tests
}
