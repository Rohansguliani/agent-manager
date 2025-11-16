//! Streaming utilities for Server-Sent Events (SSE)
//!
//! Contains utilities for creating SSE streams from agent execution results.

use crate::api::utils::update_agent_status;
use crate::error::AppError;
use crate::executor::StreamingCliExecutor;
use crate::orchestrator::constants::{SSE_DONE_SIGNAL, SSE_ERROR_PREFIX};
use crate::state::{Agent, AgentStatus, AppState};
#[allow(unused_imports)] // Used in anyhow! macro on line 51
use anyhow::anyhow;
use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
};
use futures_util::{stream::Stream, StreamExt};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Create an SSE stream from a streaming executor
///
/// # Arguments
/// * `executor` - Streaming executor
/// * `agent` - Agent to execute
/// * `query` - Query string
/// * `app_state` - Application state
///
/// # Returns
/// * `Result<Response, AppError>` - SSE HTTP response or error
pub fn create_sse_stream(
    executor: StreamingCliExecutor,
    agent: Agent,
    query: String,
    app_state: Arc<RwLock<AppState>>,
) -> Result<Response, AppError> {
    let stream = create_stream(executor, agent, query, app_state);

    let sse_stream = stream.map(|event_result| {
        let sse_text = match event_result {
            Ok(data) => format!("data: {}\n\n", data),
            Err(e) => format!("data: {} {}\n\n", SSE_ERROR_PREFIX, e),
        };
        Ok::<_, std::io::Error>(sse_text)
    });

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(sse_stream))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to build SSE response: {}", e)))
}

/// Create a stream from executor results
///
/// # Arguments
/// * `executor` - Streaming executor
/// * `agent` - Agent to execute
/// * `query` - Query string
/// * `app_state` - Application state
///
/// # Returns
/// * `impl Stream<Item = Result<String, axum::Error>>` - Stream of results
fn create_stream(
    executor: StreamingCliExecutor,
    agent: Agent,
    query: String,
    app_state: Arc<RwLock<AppState>>,
) -> impl Stream<Item = Result<String, axum::Error>> {
    use async_stream::stream;

    stream! {
        let agent_id = agent.id.clone();

        // Start execution and get receiver
        match executor.execute_streaming(&agent, &query).await {
            Ok(mut rx) => {
                // Stream lines as they come
                while let Some(line) = rx.recv().await {
                    yield Ok(line);
                }

                // Process completed successfully
                update_agent_status(&app_state, &agent_id, AgentStatus::Idle).await;
                yield Ok(SSE_DONE_SIGNAL.to_string());
            }
            Err(e) => {
                update_agent_status(&app_state, &agent_id, AgentStatus::Error).await;
                yield Ok(format!("{} {}", SSE_ERROR_PREFIX, e));
            }
        }
    }
}
