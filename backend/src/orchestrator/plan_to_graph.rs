//! Convert Plan to graph-flow Graph
//!
//! This module builds a graph-flow graph from a Plan structure.
//! It handles task creation, dependency resolution, and parallel execution
//! using FanOutTask for independent steps.

use crate::error::AppError;
use crate::orchestrator::plan_types::Plan;
use crate::orchestrator::tasks::{CreateFileTask, RunGeminiTask};
use crate::state::AppState;
use anyhow::anyhow;
use graph_flow::{Graph, GraphBuilder, Task};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Build a graph-flow graph from a plan
///
/// This function converts a Plan into a graph-flow Graph that can be executed.
/// It handles:
/// - Creating task instances from plan steps
/// - Building dependency edges between tasks
/// - Parallel execution of independent steps (via graph-flow's edge-based execution)
///
/// # Arguments
/// * `plan` - The plan to convert
/// * `app_state` - Application state (for agent management, working directory)
///
/// # Returns
/// * `Ok(Arc<Graph>)` - The constructed graph
/// * `Err(AppError)` - If graph building fails
#[allow(dead_code)] // Will be used in Phase 4H when replacing executor
pub fn build_graph_from_plan(
    plan: Plan,
    app_state: Arc<RwLock<AppState>>,
) -> Result<Arc<Graph>, AppError> {
    // Validate plan first
    plan.validate()
        .map_err(|e| AppError::InvalidPlan(format!("Plan validation failed: {}", e)))?;

    if plan.steps.is_empty() {
        return Err(AppError::InvalidPlan("Plan has no steps".to_string()));
    }

    // Note: Working directory will be set in context when session is created
    // We don't need to read it here since tasks will get it from app_state or context

    // Build task instances from plan steps
    let mut task_map: HashMap<String, Arc<dyn Task>> = HashMap::new();

    for step in &plan.steps {
        let task: Arc<dyn Task> = match step.task.as_str() {
            "run_gemini" => {
                let prompt = step.params.prompt.as_ref().ok_or_else(|| {
                    AppError::InvalidPlan(format!(
                        "Step '{}' (run_gemini) missing required parameter: prompt",
                        step.id
                    ))
                })?;

                let run_task = RunGeminiTask::new(step.id.clone(), prompt.clone())
                    .with_app_state(app_state.clone());
                Arc::new(run_task)
            }
            "create_file" => {
                let filename = step.params.filename.as_ref().ok_or_else(|| {
                    AppError::InvalidPlan(format!(
                        "Step '{}' (create_file) missing required parameter: filename",
                        step.id
                    ))
                })?;

                // Validate filename for path traversal protection
                if filename.contains("..") || filename.starts_with('/') {
                    return Err(AppError::InvalidPlan(format!(
                        "Step '{}' (create_file) has invalid filename '{}': path traversal detected or absolute path",
                        step.id, filename
                    )));
                }

                if filename.contains('\0') || filename.chars().any(|c| c.is_control()) {
                    return Err(AppError::InvalidPlan(format!(
                        "Step '{}' (create_file) has invalid filename '{}': control characters detected",
                        step.id, filename
                    )));
                }

                let create_task = CreateFileTask::new(
                    step.id.clone(),
                    filename.clone(),
                    step.params.content_from.clone(),
                )
                .with_app_state(app_state.clone());
                Arc::new(create_task)
            }
            _ => {
                return Err(AppError::InvalidPlan(format!(
                    "Unknown task type: '{}' in step '{}'",
                    step.task, step.id
                )));
            }
        };

        task_map.insert(step.id.clone(), task);
    }

    // Build graph
    use crate::orchestrator::constants::DEFAULT_GRAPH_ID;
    let mut builder = GraphBuilder::new(DEFAULT_GRAPH_ID);

    // Add all tasks to the graph
    for task in task_map.values() {
        builder = builder.add_task(task.clone());
    }

    // Add edges based on dependencies
    for step in &plan.steps {
        for dep in &step.dependencies {
            builder = builder.add_edge(dep, &step.id);
        }
    }

    // Set start task (first step with no dependencies, or first step if all have dependencies)
    use crate::orchestrator::plan_utils::find_start_step_id;
    let start_task_id = find_start_step_id(&plan).ok_or_else(|| {
        AppError::Internal(anyhow!(
            "Plan has no steps (this should not happen after validation)"
        ))
    })?;

    builder = builder.set_start_task(start_task_id);

    let graph = Arc::new(builder.build());

    // Set working directory in context when session is created (handled in executor)

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::plan_types::{Plan, Step, StepParams};

    fn create_test_state() -> Arc<RwLock<AppState>> {
        Arc::new(RwLock::new(AppState::new()))
    }

    #[test]
    fn test_build_graph_from_plan_sequential() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Write a test".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("test.txt".to_string()),
                        content_from: Some("step_1.output".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_1".to_string()],
                },
            ],
        };

        let state = create_test_state();
        let result = build_graph_from_plan(plan, state);

        assert!(result.is_ok());
        let graph = result.unwrap();
        use crate::orchestrator::constants::DEFAULT_GRAPH_ID;
        assert_eq!(graph.id, DEFAULT_GRAPH_ID);
    }

    #[test]
    fn test_build_graph_from_plan_parallel() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Write test 1".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Write test 2".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_3".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("combined.txt".to_string()),
                        content_from: Some("step_1.output".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_1".to_string(), "step_2".to_string()],
                },
            ],
        };

        let state = create_test_state();
        let result = build_graph_from_plan(plan, state);

        assert!(result.is_ok());
        let graph = result.unwrap();
        use crate::orchestrator::constants::DEFAULT_GRAPH_ID;
        assert_eq!(graph.id, DEFAULT_GRAPH_ID);
    }

    #[test]
    fn test_build_graph_from_plan_invalid_task() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![Step {
                id: "step_1".to_string(),
                task: "unknown_task".to_string(),
                params: StepParams::default(),
                dependencies: vec![],
            }],
        };

        let state = create_test_state();
        let result = build_graph_from_plan(plan, state);

        match result {
            Err(e) => {
                let error_msg = e.to_string();
                // Plan validation catches invalid task names, so we check for "invalid task name" instead
                assert!(
                    error_msg.contains("invalid task name")
                        || error_msg.contains("Unknown task type"),
                    "Error message should mention invalid/unknown task, got: {}",
                    error_msg
                );
            }
            Ok(_) => panic!("Expected error for unknown task type"),
        }
    }

    #[test]
    fn test_build_graph_from_plan_missing_prompt() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![Step {
                id: "step_1".to_string(),
                task: "run_gemini".to_string(),
                params: StepParams {
                    // Missing prompt
                    ..Default::default()
                },
                dependencies: vec![],
            }],
        };

        let state = create_test_state();
        let result = build_graph_from_plan(plan, state);

        match result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("prompt"),
                    "Error message should mention 'prompt', got: {}",
                    error_msg
                );
            }
            Ok(_) => panic!("Expected error for missing prompt"),
        }
    }

    #[test]
    fn test_build_graph_from_plan_missing_filename() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![Step {
                id: "step_1".to_string(),
                task: "create_file".to_string(),
                params: StepParams {
                    // Missing filename
                    ..Default::default()
                },
                dependencies: vec![],
            }],
        };

        let state = create_test_state();
        let result = build_graph_from_plan(plan, state);

        match result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("filename"),
                    "Error message should mention 'filename', got: {}",
                    error_msg
                );
            }
            Ok(_) => panic!("Expected error for missing filename"),
        }
    }

    #[test]
    fn test_build_graph_from_plan_empty_steps() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![],
        };

        let state = create_test_state();
        let result = build_graph_from_plan(plan, state);

        match result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("no steps"),
                    "Error message should mention 'no steps', got: {}",
                    error_msg
                );
            }
            Ok(_) => panic!("Expected error for empty steps"),
        }
    }

    #[test]
    fn test_build_graph_from_plan_path_traversal() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![Step {
                id: "step_1".to_string(),
                task: "create_file".to_string(),
                params: StepParams {
                    filename: Some("../etc/passwd".to_string()),
                    ..Default::default()
                },
                dependencies: vec![],
            }],
        };

        let state = create_test_state();
        let result = build_graph_from_plan(plan, state);

        match result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("path traversal") || error_msg.contains("absolute path"),
                    "Error message should mention path traversal, got: {}",
                    error_msg
                );
            }
            Ok(_) => panic!("Expected error for path traversal"),
        }
    }

    #[test]
    fn test_build_graph_sets_start_task() {
        // Test that the graph builder correctly identifies and sets the start task
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Test 1".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Test 2".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_1".to_string()],
                },
            ],
        };

        let state = create_test_state();
        let result = build_graph_from_plan(plan, state);

        assert!(result.is_ok());
        let graph = result.unwrap();
        use crate::orchestrator::constants::DEFAULT_GRAPH_ID;
        assert_eq!(graph.id, DEFAULT_GRAPH_ID);
        // The start task should be step_1 (first independent step)
    }

    #[test]
    fn test_build_graph_with_complex_dependencies() {
        // Test graph building with a complex dependency structure
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Test 1".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Test 2".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_3".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Test 3".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_1".to_string()],
                },
                Step {
                    id: "step_4".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("output.txt".to_string()),
                        content_from: Some("step_3.output".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_2".to_string(), "step_3".to_string()],
                },
            ],
        };

        let state = create_test_state();
        let result = build_graph_from_plan(plan, state);

        assert!(result.is_ok());
        let graph = result.unwrap();
        use crate::orchestrator::constants::DEFAULT_GRAPH_ID;
        assert_eq!(graph.id, DEFAULT_GRAPH_ID);
        // Graph should have 4 tasks with proper dependency edges
    }
}
