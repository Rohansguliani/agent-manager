//! Streaming utilities for Server-Sent Events (SSE)
//!
//! Contains utilities for creating SSE streams from agent execution results.

use crate::api::utils::update_agent_status;
use crate::chat::{ChatDb, Message, MessageRole};
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
use uuid::Uuid;

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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// Create an SSE stream from a streaming executor with chat support
///
/// This function streams the response and saves it to the database if a conversation_id is provided.
///
/// # Arguments
/// * `executor` - Streaming executor
/// * `agent` - Agent to execute
/// * `query` - Query string
/// * `app_state` - Application state
/// * `chat_db` - Chat database for saving messages
/// * `conversation_id` - Optional conversation ID to save assistant message
///
/// # Returns
/// * `Result<Response, AppError>` - SSE HTTP response or error
#[allow(dead_code)]
pub fn create_sse_stream_with_chat(
    executor: StreamingCliExecutor,
    agent: Agent,
    query: String,
    app_state: Arc<RwLock<AppState>>,
    chat_db: Arc<ChatDb>,
    conversation_id: Option<String>,
) -> Result<Response, AppError> {
    let stream =
        create_stream_with_chat(executor, agent, query, app_state, chat_db, conversation_id);

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

/// Create a stream from executor results with chat support
///
/// Collects all streamed content and saves it as an assistant message when done.
///
/// # Arguments
/// * `executor` - Streaming executor
/// * `agent` - Agent to execute
/// * `query` - Query string
/// * `app_state` - Application state
/// * `chat_db` - Chat database for saving messages
/// * `conversation_id` - Optional conversation ID to save assistant message
///
/// # Returns
/// * `impl Stream<Item = Result<String, axum::Error>>` - Stream of results
#[allow(dead_code)]
fn create_stream_with_chat(
    executor: StreamingCliExecutor,
    agent: Agent,
    query: String,
    app_state: Arc<RwLock<AppState>>,
    chat_db: Arc<ChatDb>,
    conversation_id: Option<String>,
) -> impl Stream<Item = Result<String, axum::Error>> {
    use async_stream::stream;

    stream! {
        let agent_id = agent.id.clone();
        let mut full_response = String::new();

        // Start execution and get receiver
        match executor.execute_streaming(&agent, &query).await {
            Ok(mut rx) => {
                // Stream chunks as they come and collect them
                while let Some(chunk) = rx.recv().await {
                    full_response.push_str(&chunk);
                    // Don't add newline - chunks are already properly formatted
                    yield Ok(chunk);
                }

                // Process completed successfully
                update_agent_status(&app_state, &agent_id, AgentStatus::Idle).await;

                // Save assistant message if conversation_id is provided
                if let Some(conv_id) = conversation_id {
                    // Trim trailing newline from collected response
                    let response_content = full_response.trim_end().to_string();

                    if !response_content.is_empty() {
                        let assistant_message = Message::new(
                            Uuid::new_v4().to_string(),
                            conv_id,
                            MessageRole::Assistant,
                            response_content,
                        );

                        // Save message (ignore errors in streaming context)
                        if let Err(e) = chat_db.add_message(&assistant_message).await {
                            tracing::error!("Failed to save assistant message: {}", e);
                        }
                    }
                }

                yield Ok(SSE_DONE_SIGNAL.to_string());
            }
            Err(e) => {
                update_agent_status(&app_state, &agent_id, AgentStatus::Error).await;
                yield Ok(format!("{} {}", SSE_ERROR_PREFIX, e));
            }
        }
    }
}
