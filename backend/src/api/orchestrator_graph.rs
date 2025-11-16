//! Graph visualization API endpoint
//!
//! Provides endpoints to inspect and visualize the execution graph structure
//! for debugging and monitoring purposes.

use crate::error::AppError;
use crate::orchestrator::plan_to_graph::build_graph_from_plan;
use crate::state::AppState;
use axum::{extract::State, response::Json};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Graph structure representation for visualization
#[derive(Debug, Serialize)]
pub struct GraphStructure {
    /// Graph ID
    pub graph_id: String,
    /// Number of tasks in the graph
    pub task_count: usize,
    /// Task IDs
    pub task_ids: Vec<String>,
    /// Edges (dependencies) in the graph
    pub edges: Vec<GraphEdge>,
}

/// Represents an edge (dependency) in the graph
#[derive(Debug, Serialize)]
pub struct GraphEdge {
    /// Source task ID
    pub from: String,
    /// Target task ID
    pub to: String,
}

/// GET /api/orchestrate/graph - Get graph structure for a plan
///
/// This endpoint allows inspection of the execution graph structure
/// that would be built from a given plan.
///
/// # Query Parameters
/// * `goal` - The goal string to build a plan and graph from (passed as query param)
///
/// # Returns
/// * `Ok(Json<GraphStructure>)` - The graph structure
/// * `Err(AppError)` - If plan generation or graph building fails
pub async fn get_graph_structure(
    State(state): State<Arc<RwLock<AppState>>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<GraphStructure>, AppError> {
    use crate::orchestrator::primitives::internal_run_planner;

    // Get goal from query parameters
    let goal = params
        .get("goal")
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing 'goal' query parameter")))?;

    // Generate plan using planner agent (via CLI)
    let plan = internal_run_planner(&state, goal).await?;

    // Build graph
    let graph = build_graph_from_plan(plan.clone(), state)?;

    // Extract graph structure using plan utilities
    use crate::orchestrator::plan_utils::{extract_edges, extract_task_ids};
    let task_ids = extract_task_ids(&plan);
    let edges: Vec<GraphEdge> = extract_edges(&plan)
        .into_iter()
        .map(|(from, to)| GraphEdge { from, to })
        .collect();

    Ok(Json(GraphStructure {
        graph_id: graph.id.clone(),
        task_count: task_ids.len(),
        task_ids,
        edges,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_state() -> Arc<RwLock<AppState>> {
        Arc::new(RwLock::new(AppState::new()))
    }

    #[tokio::test]
    async fn test_get_graph_structure_missing_goal() {
        // Test that missing 'goal' parameter returns an error
        let state = create_test_state();
        let params = HashMap::new();

        let result = get_graph_structure(State(state), axum::extract::Query(params)).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("goal") || error.to_string().contains("Missing"));
    }

    #[tokio::test]
    async fn test_get_graph_structure_structure() {
        // Test graph structure extraction from a manually created plan
        // This tests the structure extraction logic without requiring API calls
        use crate::orchestrator::plan_types::{Plan, Step, StepParams};

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

        // Verify plan structure
        assert!(plan.validate().is_ok());

        // Extract structure manually (mimicking the endpoint logic)
        let task_ids: Vec<String> = plan.steps.iter().map(|s| s.id.clone()).collect();
        let mut edges = Vec::new();
        for step in &plan.steps {
            for dep in &step.dependencies {
                edges.push(GraphEdge {
                    from: dep.clone(),
                    to: step.id.clone(),
                });
            }
        }

        // Verify extracted structure
        assert_eq!(task_ids.len(), 2);
        assert!(task_ids.contains(&"step_1".to_string()));
        assert!(task_ids.contains(&"step_2".to_string()));
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, "step_1");
        assert_eq!(edges[0].to, "step_2");
    }

    #[tokio::test]
    async fn test_graph_structure_parallel_steps() {
        // Test graph structure with parallel steps (no dependencies)
        use crate::orchestrator::plan_types::{Plan, Step, StepParams};

        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Task 1".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Task 2".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
            ],
        };

        // Extract structure
        let task_ids: Vec<String> = plan.steps.iter().map(|s| s.id.clone()).collect();
        let mut edges = Vec::new();
        for step in &plan.steps {
            for dep in &step.dependencies {
                edges.push(GraphEdge {
                    from: dep.clone(),
                    to: step.id.clone(),
                });
            }
        }

        // Parallel steps should have no edges
        assert_eq!(task_ids.len(), 2);
        assert_eq!(edges.len(), 0);
    }

    #[tokio::test]
    async fn test_graph_structure_diamond_pattern() {
        // Test graph structure with diamond dependency pattern
        use crate::orchestrator::plan_types::{Plan, Step, StepParams};

        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Source".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Branch 1".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_1".to_string()],
                },
                Step {
                    id: "step_3".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Branch 2".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_1".to_string()],
                },
                Step {
                    id: "step_4".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams {
                        filename: Some("output.txt".to_string()),
                        content_from: Some("step_2.output".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec!["step_2".to_string(), "step_3".to_string()],
                },
            ],
        };

        // Extract structure
        let task_ids: Vec<String> = plan.steps.iter().map(|s| s.id.clone()).collect();
        let mut edges = Vec::new();
        for step in &plan.steps {
            for dep in &step.dependencies {
                edges.push(GraphEdge {
                    from: dep.clone(),
                    to: step.id.clone(),
                });
            }
        }

        // Diamond pattern: step_1 -> step_2, step_3; step_2, step_3 -> step_4
        assert_eq!(task_ids.len(), 4);
        assert_eq!(edges.len(), 4); // step_1->step_2, step_1->step_3, step_2->step_4, step_3->step_4
        assert!(edges.iter().any(|e| e.from == "step_1" && e.to == "step_2"));
        assert!(edges.iter().any(|e| e.from == "step_1" && e.to == "step_3"));
        assert!(edges.iter().any(|e| e.from == "step_2" && e.to == "step_4"));
        assert!(edges.iter().any(|e| e.from == "step_3" && e.to == "step_4"));
    }
}
