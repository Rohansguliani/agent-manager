//! Orchestrator API handlers
//!
//! Contains HTTP request handlers for orchestration workflows.
//! This implements the "V1 Orchestrator" pattern - hard-coded orchestration
//! that chains worker agents and tools to complete high-level goals.
//!
//! The orchestration uses SSE (Server-Sent Events) to stream status updates
//! to the frontend, allowing real-time feedback on multi-step operations.

use crate::error::AppError;
use crate::orchestrator::graph_executor::execute_plan;
use crate::orchestrator::primitives::{
    internal_create_file, internal_run_gemini, internal_run_planner,
};
use crate::state::AppState;
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
use std::sync::Arc;
use tokio::sync::RwLock;

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
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<OrchestrationRequest>,
) -> Result<Response, AppError> {
    const MAX_GOAL_LENGTH: usize = 10000; // 10KB

    // Validate input size
    if request.goal.len() > MAX_GOAL_LENGTH {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Goal too long ({} > {} characters). Maximum allowed length is {} characters.",
            request.goal.len(),
            MAX_GOAL_LENGTH,
            MAX_GOAL_LENGTH
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
                        yield Ok::<String, axum::Error>("[DONE]".to_string());
                    }
                    Err(e) => {
                        // Error saving file
                        yield Ok::<String, axum::Error>(format!(
                            r#"{{"step": 2, "message": "Error saving file: {}", "status": "error"}}"#,
                            e
                        ));
                        // Signal stream completion
                        yield Ok::<String, axum::Error>("[DONE]".to_string());
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
                yield Ok::<String, axum::Error>("[DONE]".to_string());
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
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<OrchestrationRequest>,
) -> Result<Response, AppError> {
    use async_stream::stream;

    const MAX_GOAL_LENGTH: usize = 10000; // 10KB

    // Validate input size
    if request.goal.len() > MAX_GOAL_LENGTH {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Goal too long ({} > {} characters). Maximum allowed length is {} characters.",
            request.goal.len(),
            MAX_GOAL_LENGTH,
            MAX_GOAL_LENGTH
        )));
    }

    let state_clone = state.clone();
    let goal = request.goal;

    let stream = stream! {
        // Step 1: Planning
        yield Ok::<String, axum::Error>(
            r#"{"step": 0, "step_id": "planning", "message": "Planning: Generating execution plan...", "status": "running"}"#
                .to_string(),
        );

        // Generate plan using planner agent
        let plan = match internal_run_planner(&goal).await {
            Ok(plan) => {
                yield Ok::<String, axum::Error>(format!(
                    r#"{{"step": 0, "step_id": "planning", "message": "Plan generated: {} steps", "status": "running"}}"#,
                    plan.steps.len()
                ));
                plan
            }
            Err(e) => {
                yield Ok::<String, axum::Error>(format!(
                    r#"{{"step": 0, "step_id": "planning", "message": "Planning failed: {}", "status": "error"}}"#,
                    e
                ));
                yield Ok::<String, axum::Error>("[DONE]".to_string());
                return;
            }
        };

        // Step 2: Execution - stream events as steps execute
        // Note: execute_plan returns results after all steps complete,
        // but we can still stream completion events for each step
        match execute_plan(plan.clone(), &state_clone).await {
            Ok(results) => {
                // Stream results from each step with step_id
                for result in &results {
                    if result.success {
                        yield Ok::<String, axum::Error>(format!(
                            r#"{{"step": {}, "step_id": "{}", "message": "Step {} ({}) completed", "status": "running"}}"#,
                            result.step_number,
                            result.step_id,
                            result.step_number,
                            result.step_id
                        ));
                    } else {
                        yield Ok::<String, axum::Error>(format!(
                            r#"{{"step": {}, "step_id": "{}", "message": "Step {} ({}) failed: {}", "status": "error"}}"#,
                            result.step_number,
                            result.step_id,
                            result.step_number,
                            result.step_id,
                            result.error.as_deref().unwrap_or("Unknown error")
                        ));
                        yield Ok::<String, axum::Error>("[DONE]".to_string());
                        return;
                    }
                }

                // All steps completed successfully
                yield Ok::<String, axum::Error>(format!(
                    r#"{{"step": {}, "step_id": "completion", "message": "All {} steps completed successfully!", "status": "completed"}}"#,
                    results.len(),
                    results.len()
                ));
                yield Ok::<String, axum::Error>("[DONE]".to_string());
            }
            Err(e) => {
                yield Ok::<String, axum::Error>(format!(
                    r#"{{"step": 0, "step_id": "execution_error", "message": "Execution failed: {}", "status": "error"}}"#,
                    e
                ));
                yield Ok::<String, axum::Error>("[DONE]".to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_state() -> Arc<RwLock<AppState>> {
        Arc::new(RwLock::new(AppState::new()))
    }

    #[tokio::test]
    async fn test_orchestrate_poem_request_structure() {
        // Test that the endpoint accepts requests and returns SSE response
        // This is a structural test - full integration would require Gemini CLI
        let state = create_test_state();
        let request = OrchestrationRequest {
            goal: "Write a test poem".to_string(),
        };

        // This will fail if Gemini CLI is not available, but we can at least
        // test that the endpoint structure is correct
        let result = orchestrate_poem(State(state), Json(request)).await;

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
        let state = create_test_state();
        let request = OrchestrationRequest {
            goal: String::new(),
        };

        let result = orchestrate_poem(State(state), Json(request)).await;

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
}
