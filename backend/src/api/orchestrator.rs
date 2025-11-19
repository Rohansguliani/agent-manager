//! Orchestrator API handlers
//!
//! Contains HTTP request handlers for orchestration workflows.
//! This implements the "V1 Orchestrator" pattern - hard-coded orchestration
//! that chains worker agents and tools to complete high-level goals.
//!
//! The orchestration uses SSE (Server-Sent Events) to stream status updates
//! to the frontend, allowing real-time feedback on multi-step operations.

use crate::api::utils::RouterState;
use crate::error::AppError;
use crate::orchestrator::config::{
    validate_and_apply_config_update, ConfigUpdateRequest, OrchestratorConfig,
};
use crate::orchestrator::constants::SSE_DONE_SIGNAL;
use crate::orchestrator::graph_executor::execute_plan;
use crate::orchestrator::plan_optimizer::{
    analyze_bottlenecks, estimate_execution_time, estimate_token_usage, BottleneckAnalysis,
};
use crate::orchestrator::primitives::{
    internal_create_file, internal_run_gemini, internal_run_planner,
};
#[allow(unused_imports)] // Used in map_err on lines 179 and 289
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::Response,
    Json,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

/// Helper function to serialize an OrchestrationEvent to JSON string
///
/// This centralizes event serialization with proper error handling.
/// If serialization fails, it logs the error and returns a fallback JSON.
///
/// # Arguments
/// * `event` - The orchestration event to serialize
///
/// # Returns
/// * `String` - JSON string representation of the event (or fallback on error)
fn serialize_event_or_fallback(event: &OrchestrationEvent) -> String {
    serde_json::to_string(event).unwrap_or_else(|e| {
        tracing::error!("Failed to serialize OrchestrationEvent: {} - Event: {:?}", e, event);
        // Return a minimal error event as fallback
        format!(
            r#"{{"type": "serialization_error", "message": "Event serialization failed: {}", "status": "error"}}"#,
            e
        )
    })
}

/// Helper function to format a stream into SSE (Server-Sent Events) format
///
/// Takes a stream of `Result<String, axum::Error>` and converts it to SSE format
/// where each item is formatted as "data: <content>\n\n"
fn format_sse_stream(
    stream: impl futures_util::Stream<Item = Result<String, axum::Error>> + Send + 'static,
) -> impl futures_util::Stream<Item = Result<String, std::io::Error>> {
    stream.map(|event_result| {
        let sse_text = match event_result {
            Ok(data) => format!("data: {}\n\n", data),
            Err(e) => format!("data: [ERROR] {}\n\n", e),
        };
        Ok::<_, std::io::Error>(sse_text)
    })
}

/// Orchestration request
#[derive(Deserialize, Debug)]
pub struct OrchestrationRequest {
    /// The goal or prompt for the orchestration
    pub goal: String,
}

/// Orchestration status update
/// Sent via SSE to provide real-time feedback on orchestration progress
#[derive(Debug, Serialize, Clone)]
#[allow(dead_code)] // Used by frontend TypeScript, not constructed in Rust
pub struct OrchestrationStatus {
    /// Step number (1, 2, 3, etc.)
    pub step: u32,
    /// Human-readable message describing current step
    pub message: String,
    /// Status: "running", "completed", or "error"
    pub status: String,
}

/// Phase 6.3: Structured orchestration events for live graph updates
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrchestrationEvent {
    /// Plan generated with analysis
    PlanGenerated {
        /// Number of steps in the plan
        step_count: usize,
        /// Estimated token usage for the plan
        estimated_tokens: usize,
        /// Estimated execution time in seconds
        estimated_time_secs: usize,
    },
    /// Step started executing
    StepStart {
        /// Unique identifier for the step
        step_id: String,
        /// Sequential step number (1-indexed)
        step_number: u32,
        /// Task type being executed (e.g., "run_gemini", "create_file")
        task: String,
    },
    /// Step completed successfully
    StepComplete {
        /// Unique identifier for the step
        step_id: String,
        /// Sequential step number (1-indexed)
        step_number: u32,
        /// Output from the step execution
        output: String,
    },
    /// Step failed
    StepError {
        /// Unique identifier for the step
        step_id: String,
        /// Sequential step number (1-indexed)
        step_number: u32,
        /// Error message describing the failure
        error: String,
    },
    /// All steps completed
    ExecutionComplete {
        /// Total number of steps in the plan
        total_steps: usize,
        /// Number of steps that completed successfully
        successful_steps: usize,
    },
    /// Execution failed
    ExecutionError {
        /// Error message describing the failure
        error: String,
    },
}

/// POST /api/orchestrate/poem - Hard-coded orchestrator example
///
/// Creates a poem using Gemini and saves it to a file.
/// This is a V1 implementation - hard-coded orchestration to validate
/// the pattern before building a generic orchestrator.
///
/// # Flow
/// 1. Run Gemini to generate a poem
/// 2. Save the poem to `poem.txt` in the working directory (if set)
/// 3. Stream status updates via SSE
///
/// # Arguments
/// * `State(state)` - Application state
/// * `Json(request)` - Orchestration request with goal/prompt
///
/// # Returns
/// * `Ok(Response)` - SSE stream with status updates
/// * `Err(AppError)` - If orchestration fails
pub async fn orchestrate_poem(
    State((state, _, _)): State<RouterState>,
    Json(request): Json<OrchestrationRequest>,
) -> Result<Response, AppError> {
    let config = OrchestratorConfig::default();

    // Validate input size
    if request.goal.len() > config.max_goal_length {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Goal too long ({} > {} characters). Maximum allowed length is {} characters.",
            request.goal.len(),
            config.max_goal_length,
            config.max_goal_length
        )));
    }

    // Get working directory from state
    let working_dir = {
        let state_read = state.read().await;
        let wd = state_read.working_directory().cloned();
        tracing::debug!(
            working_dir = ?wd,
            "Orchestrator: Retrieved working directory from state"
        );
        wd
    };

    // Create SSE stream using async_stream (same pattern as query_stream)
    use async_stream::stream;

    let state_clone = state.clone();
    let goal = request.goal;
    let working_dir_clone = working_dir.clone();

    let stream = stream! {
        // Step 1: Status update - asking Gemini
        yield Ok::<String, axum::Error>(
            r#"{"step": 1, "message": "Task 1: Asking Gemini for a poem...", "status": "running"}"#
                .to_string(),
        );

        // Step 2: Run Gemini to generate poem
        let poem_prompt = if goal.is_empty() {
            "Write a 4-line poem about the Rust programming language."
        } else {
            &goal
        };

        match internal_run_gemini(&state_clone, poem_prompt).await {
            Ok(poem) => {
                // Step 3: Status update - saving file
                yield Ok::<String, axum::Error>(format!(
                    r#"{{"step": 2, "message": "Task 2: Saving poem to 'poem.txt'... (Generated {} characters)", "status": "running"}}"#,
                    poem.len()
                ));

                // Step 4: Save poem to file
                tracing::debug!(
                    working_dir = ?working_dir_clone,
                    poem_len = poem.len(),
                    "Orchestrator: About to create file 'poem.txt' with working directory"
                );
                match internal_create_file(
                    "poem.txt",
                    &poem,
                    working_dir_clone.as_deref(),
                ).await {
                    Ok(file_path) => {
                        // Step 5: Success status
                        yield Ok::<String, axum::Error>(format!(
                            r#"{{"step": 3, "message": "Done! Poem saved to: {}", "status": "completed"}}"#,
                            file_path
                        ));
                        // Signal stream completion
                        use crate::orchestrator::constants::SSE_DONE_SIGNAL;
                        yield Ok::<String, axum::Error>(SSE_DONE_SIGNAL.to_string());
                    }
                    Err(e) => {
                        // Error saving file
                        yield Ok::<String, axum::Error>(format!(
                            r#"{{"step": 2, "message": "Error saving file: {}", "status": "error"}}"#,
                            e
                        ));
                        // Signal stream completion
                        use crate::orchestrator::constants::SSE_DONE_SIGNAL;
                        yield Ok::<String, axum::Error>(SSE_DONE_SIGNAL.to_string());
                    }
                }
            }
            Err(e) => {
                // Error running Gemini
                yield Ok::<String, axum::Error>(format!(
                    r#"{{"step": 1, "message": "Error: {}", "status": "error"}}"#,
                    e
                ));
                // Signal stream completion
                yield Ok::<String, axum::Error>(SSE_DONE_SIGNAL.to_string());
            }
        }
    };

    // Convert stream to SSE format
    let sse_stream = format_sse_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(sse_stream))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to build response: {}", e)))
}

/// POST /api/orchestrate - Dynamic orchestrator endpoint
///
/// Takes a high-level goal and uses the planner agent to generate a plan,
/// then executes the plan using GraphFlow-rs (via graph_executor).
///
/// This is the Phase 2B implementation - dynamic orchestration that replaces
/// the hard-coded V1 "poem" orchestrator.
///
/// # Flow
/// 1. Call planner agent to generate a JSON plan
/// 2. Execute the plan step by step
/// 3. Stream status updates via SSE
///
/// # Arguments
/// * `State(state)` - Application state
/// * `Json(request)` - Orchestration request with goal
///
/// # Returns
/// * `Ok(Response)` - SSE stream with status updates
/// * `Err(AppError)` - If orchestration fails
pub async fn orchestrate(
    State((state, _, _)): State<RouterState>,
    Json(request): Json<OrchestrationRequest>,
) -> Result<Response, AppError> {
    use async_stream::stream;

    let config = OrchestratorConfig::default();

    // Validate input size
    if request.goal.len() > config.max_goal_length {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Goal too long ({} > {} characters). Maximum allowed length is {} characters.",
            request.goal.len(),
            config.max_goal_length,
            config.max_goal_length
        )));
    }

    let state_clone = state.clone();
    let goal = request.goal;

    // Create execution ID for tracing
    let execution_id = uuid::Uuid::new_v4().to_string();
    use crate::orchestrator::utils::hash_goal;
    let goal_hash = hash_goal(&goal);

    let span = tracing::info_span!(
        "orchestrate",
        execution_id = %execution_id,
        goal_len = goal.len(),
        goal_hash = %goal_hash,
    );
    let _enter = span.enter();

    let stream = stream! {
        // Step 1: Planning
        yield Ok::<String, axum::Error>(
            r#"{"step": 0, "step_id": "planning", "message": "Planning: Generating execution plan...", "status": "running"}"#
                .to_string(),
        );

        // Generate plan using planner agent (via CLI)
        let plan = match internal_run_planner(&state_clone, &goal).await {
            Ok(plan) => {
                // Phase 6.3: Emit structured event for plan generation
                let plan_event = OrchestrationEvent::PlanGenerated {
                    step_count: plan.steps.len(),
                    estimated_tokens: crate::orchestrator::plan_optimizer::estimate_token_usage(&plan),
                    estimated_time_secs: crate::orchestrator::plan_optimizer::estimate_execution_time(&plan),
                };
                yield Ok::<String, axum::Error>(serialize_event_or_fallback(&plan_event));
                plan
            }
            Err(e) => {
                let error_event = OrchestrationEvent::ExecutionError {
                    error: format!("Planning failed: {}", e),
                };
                yield Ok::<String, axum::Error>(serialize_event_or_fallback(&error_event));
                yield Ok::<String, axum::Error>(SSE_DONE_SIGNAL.to_string());
                return;
            }
        };

        // Phase 6.3: Emit StepStart events for all steps (before execution)
        // This gives the frontend a "map" of all steps that will run
        for (idx, step) in plan.steps.iter().enumerate() {
            let step_event = OrchestrationEvent::StepStart {
                step_id: step.id.clone(),
                step_number: (idx + 1) as u32,
                task: step.task.clone(),
            };
            yield Ok::<String, axum::Error>(serialize_event_or_fallback(&step_event));
        }

        // Step 2: Execution - stream events as steps execute
        // Note: execute_plan returns results after all steps complete,
        // but we can still stream completion events for each step
        match execute_plan(&plan, &state_clone).await {
            Ok(results) => {
                // Stream results from each step with structured events
                for result in &results {
                    if result.success {
                        let complete_event = OrchestrationEvent::StepComplete {
                            step_id: result.step_id.clone(),
                            step_number: result.step_number,
                            output: result.output.clone().unwrap_or_default(),
                        };
                        yield Ok::<String, axum::Error>(serialize_event_or_fallback(&complete_event));
                    } else {
                        let error_event = OrchestrationEvent::StepError {
                            step_id: result.step_id.clone(),
                            step_number: result.step_number,
                            error: result.error.clone().unwrap_or_else(|| "Unknown error".to_string()),
                        };
                        yield Ok::<String, axum::Error>(serialize_event_or_fallback(&error_event));
                        use crate::orchestrator::constants::SSE_DONE_SIGNAL;
                        yield Ok::<String, axum::Error>(SSE_DONE_SIGNAL.to_string());
                        return;
                    }
                }

                // All steps completed successfully
                let complete_event = OrchestrationEvent::ExecutionComplete {
                    total_steps: results.len(),
                    successful_steps: results.iter().filter(|r| r.success).count(),
                };
                yield Ok::<String, axum::Error>(serialize_event_or_fallback(&complete_event));
                yield Ok::<String, axum::Error>(SSE_DONE_SIGNAL.to_string());
            }
            Err(e) => {
                let error_event = OrchestrationEvent::ExecutionError {
                    error: format!("Execution failed: {}", e),
                };
                yield Ok::<String, axum::Error>(serialize_event_or_fallback(&error_event));
                yield Ok::<String, axum::Error>(SSE_DONE_SIGNAL.to_string());
            }
        }
    };

    // Convert stream to SSE format
    let sse_stream = format_sse_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(sse_stream))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to build response: {}", e)))
}

/// Plan analysis response (Phase 6.1: Pre-flight Check)
#[derive(Debug, Serialize)]
pub struct PlanAnalysisResponse {
    /// The generated plan
    pub plan: crate::orchestrator::plan_types::Plan,
    /// Estimated token usage
    pub estimated_tokens: usize,
    /// Estimated execution time in seconds
    pub estimated_time_secs: usize,
    /// Bottleneck analysis
    pub bottlenecks: BottleneckAnalysis,
}

/// POST /api/plan - Pre-flight check: Plan + Optimizer (Phase 6.1)
///
/// This endpoint generates a plan and runs optimization analysis
/// WITHOUT executing it. This allows users to see cost/time estimates
/// and bottlenecks before committing to execution.
///
/// # Flow
/// 1. Call planner agent to generate a JSON plan
/// 2. Run optimizer functions (token usage, execution time, bottlenecks)
/// 3. Return plan + analysis (NO execution)
///
/// # Arguments
/// * `State(state)` - Application state
/// * `Json(request)` - Orchestration request with goal
///
/// # Returns
/// * `Ok(Json<PlanAnalysisResponse>)` - Plan + analysis
/// * `Err(AppError)` - If planning fails
pub async fn plan_with_analysis(
    State((state, _, _)): State<RouterState>,
    Json(request): Json<OrchestrationRequest>,
) -> Result<Json<PlanAnalysisResponse>, AppError> {
    let config = OrchestratorConfig::default();

    // Validate input size
    if request.goal.len() > config.max_goal_length {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Goal too long ({} > {} characters). Maximum allowed length is {} characters.",
            request.goal.len(),
            config.max_goal_length,
            config.max_goal_length
        )));
    }

    // Generate plan using planner agent (via CLI)
    let plan = internal_run_planner(&state, &request.goal).await?;

    // Run optimizer functions
    let estimated_tokens = estimate_token_usage(&plan);
    let estimated_time_secs = estimate_execution_time(&plan);
    let bottlenecks = analyze_bottlenecks(&plan);

    Ok(Json(PlanAnalysisResponse {
        plan,
        estimated_tokens,
        estimated_time_secs,
        bottlenecks,
    }))
}

/// Phase 6.4: Settings Panel - Get current config
/// GET /api/config
pub async fn get_config() -> Json<OrchestratorConfig> {
    Json(OrchestratorConfig::default())
}

/// Phase 6.4: Settings Panel - Update config
/// POST /api/config
///
/// Note: This updates the default config. For a production system,
/// config should be persisted (e.g., in a database or config file).
pub async fn update_config(
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<Json<OrchestratorConfig>, AppError> {
    let config = OrchestratorConfig::default();

    // Validate and apply updates using the helper function
    let updated_config = validate_and_apply_config_update(config, request)?;

    // TODO: Persist config to database or config file
    // For now, this just validates and returns the updated config
    // The actual OrchestratorConfig::default() is still used in other endpoints

    Ok(Json(updated_config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::utils::RouterState;
    use crate::chat::ChatDb;
    use crate::state::AppState;
    use std::sync::Arc;
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
    async fn test_orchestrate_poem_request_structure() {
        // Test that the endpoint accepts requests and returns SSE response
        // This is a structural test - full integration would require Gemini CLI
        let router_state = create_test_router_state().await;
        let request = OrchestrationRequest {
            goal: "Write a test poem".to_string(),
        };

        // This will fail if Gemini CLI is not available, but we can at least
        // test that the endpoint structure is correct
        let result = orchestrate_poem(State(router_state), Json(request)).await;

        // Should return Ok(Response) even if Gemini fails internally
        // The response should be an SSE stream
        match result {
            Ok(response) => {
                // Verify response is SSE
                assert_eq!(response.status(), StatusCode::OK);
                // Verify content type header
                let content_type = response
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .and_then(|h| h.to_str().ok());
                assert_eq!(content_type, Some("text/event-stream"));
            }
            Err(e) => {
                // If there's an error, it should be a validation error, not a structure error
                // Internal errors would be in the SSE stream, not the response
                panic!("Endpoint structure error: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_orchestrate_poem_with_empty_goal() {
        // Test that empty goal uses default prompt
        let router_state = create_test_router_state().await;
        let request = OrchestrationRequest {
            goal: String::new(),
        };

        let result = orchestrate_poem(State(router_state), Json(request)).await;

        // Should return SSE response (even if Gemini fails)
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_orchestration_status_structure() {
        // Test that OrchestrationStatus can be serialized (used in SSE)
        let status = OrchestrationStatus {
            step: 1,
            message: "Test message".to_string(),
            status: "running".to_string(),
        };

        // Should serialize to JSON
        let json = serde_json::to_string(&status);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("\"step\":1"));
        assert!(json_str.contains("\"message\":\"Test message\""));
        assert!(json_str.contains("\"status\":\"running\""));
    }

    // ============================================================================
    // Config Endpoint Tests (Phase 6.4)
    // ============================================================================

    #[tokio::test]
    async fn test_get_config() {
        // Test that get_config returns the default config
        let response = get_config().await;
        let config = response.0;

        // Verify default values
        assert_eq!(config.gemini_model, "gemini-2.5-flash");
        assert_eq!(config.max_goal_length, 10000);
        assert_eq!(config.plan_timeout_secs, 300);
        assert_eq!(config.max_parallel_tasks, 10);
    }

    #[tokio::test]
    async fn test_update_config_valid() {
        // Test updating config with valid values
        use crate::orchestrator::config::ConfigUpdateRequest;
        let request = ConfigUpdateRequest {
            max_parallel_tasks: Some(5),
            gemini_model: Some("gemini-2.0-flash".to_string()),
            max_goal_length: Some(5000),
            plan_timeout_secs: Some(600),
        };

        let result = update_config(Json(request)).await;
        assert!(result.is_ok());
        let config = result.unwrap().0;

        // Verify updated values
        assert_eq!(config.max_parallel_tasks, 5);
        assert_eq!(config.gemini_model, "gemini-2.0-flash");
        assert_eq!(config.max_goal_length, 5000);
        assert_eq!(config.plan_timeout_secs, 600);
    }

    #[tokio::test]
    async fn test_update_config_partial() {
        // Test updating config with partial values (some fields None)
        use crate::orchestrator::config::ConfigUpdateRequest;
        let request = ConfigUpdateRequest {
            max_parallel_tasks: Some(20),
            gemini_model: None,
            max_goal_length: None,
            plan_timeout_secs: None,
        };

        let result = update_config(Json(request)).await;
        assert!(result.is_ok());
        let config = result.unwrap().0;

        // Verify only max_parallel_tasks was updated
        assert_eq!(config.max_parallel_tasks, 20);
        // Other fields should retain default values
        assert_eq!(config.gemini_model, "gemini-2.5-flash");
        assert_eq!(config.max_goal_length, 10000);
        assert_eq!(config.plan_timeout_secs, 300);
    }

    #[tokio::test]
    async fn test_update_config_invalid_max_parallel_zero() {
        // Test that max_parallel_tasks = 0 is rejected
        use crate::orchestrator::config::ConfigUpdateRequest;
        let request = ConfigUpdateRequest {
            max_parallel_tasks: Some(0),
            gemini_model: None,
            max_goal_length: None,
            plan_timeout_secs: None,
        };

        let result = update_config(Json(request)).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("max_parallel_tasks must be > 0"));
    }

    #[tokio::test]
    async fn test_update_config_invalid_empty_model() {
        // Test that empty gemini_model is rejected
        use crate::orchestrator::config::ConfigUpdateRequest;
        let request = ConfigUpdateRequest {
            max_parallel_tasks: None,
            gemini_model: Some(String::new()),
            max_goal_length: None,
            plan_timeout_secs: None,
        };

        let result = update_config(Json(request)).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("gemini_model cannot be empty"));
    }

    #[tokio::test]
    async fn test_update_config_invalid_max_goal_zero() {
        // Test that max_goal_length = 0 is rejected
        use crate::orchestrator::config::ConfigUpdateRequest;
        let request = ConfigUpdateRequest {
            max_parallel_tasks: None,
            gemini_model: None,
            max_goal_length: Some(0),
            plan_timeout_secs: None,
        };

        let result = update_config(Json(request)).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("max_goal_length must be > 0"));
    }

    #[tokio::test]
    async fn test_update_config_invalid_timeout_zero() {
        // Test that plan_timeout_secs = 0 is rejected
        use crate::orchestrator::config::ConfigUpdateRequest;
        let request = ConfigUpdateRequest {
            max_parallel_tasks: None,
            gemini_model: None,
            max_goal_length: None,
            plan_timeout_secs: Some(0),
        };

        let result = update_config(Json(request)).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("plan_timeout_secs must be > 0"));
    }
}
