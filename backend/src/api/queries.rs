//! Query execution API handlers
//!
//! Contains HTTP request handlers for executing queries with agents
//! and streaming responses using Server-Sent Events (SSE).

use crate::api::streaming::create_sse_stream;
use crate::api::utils::{
    apply_working_directory_context, create_executor, find_or_create_gemini_agent,
    update_agent_status, validate_query,
};
use crate::error::AppError;
use crate::executor::StreamingCliExecutor;
use crate::state::{AgentId, AgentStatus, AppState};
use axum::{
    extract::{Path, State},
    response::{Json, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Query request
#[derive(Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

/// Query response
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub response: String,
    pub agent_id: AgentId,
    pub execution_time_ms: u64,
}

/// POST /api/agents/:id/query - Execute a query with the agent
pub async fn query_agent(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<AgentId>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, AppError> {
    // Get agent and apply working directory context
    let agent = {
        let state = state.read().await;
        let mut agent = state
            .agents
            .get(&id)
            .ok_or_else(|| AppError::AgentNotFound(id.clone()))?
            .clone();
        // Apply working directory context
        apply_working_directory_context(&mut agent, &state);
        agent
    };

    // Validate query
    validate_query(&request.query)?;

    // Update agent status to Running
    update_agent_status(&state, &id, AgentStatus::Running).await;

    // Create executor and execute query
    let executor = create_executor(None);
    let start = Instant::now();

    let result = executor.execute(&agent, &request.query).await;

    let duration = start.elapsed();
    let execution_time_ms = duration.as_millis() as u64;

    // Update agent status based on result
    let final_status = if result.is_ok() {
        AgentStatus::Idle
    } else {
        AgentStatus::Error
    };
    update_agent_status(&state, &id, final_status).await;

    // Convert execution error to AppError if needed
    let response = result?;

    Ok(Json(QueryResponse {
        response,
        agent_id: id,
        execution_time_ms,
    }))
}

/// POST /api/query/stream - Stream query response using Server-Sent Events
/// (simplified - auto-uses first Gemini agent)
pub async fn query_stream(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<QueryRequest>,
) -> Result<Response, AppError> {
    // Find or create Gemini agent and apply working directory context
    let agent = find_or_create_gemini_agent(&state).await;

    // Log working directory for debugging
    tracing::debug!(
        agent_id = %agent.id,
        working_dir = ?agent.config.working_dir,
        "Agent configured for query execution"
    );

    // Validate query
    validate_query(&request.query)?;

    // Update agent status to Running
    update_agent_status(&state, &agent.id, AgentStatus::Running).await;

    // Create streaming executor
    let executor = StreamingCliExecutor::new(30);

    // Create SSE stream
    create_sse_stream(executor, agent, request.query, state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::utils::MAX_QUERY_LENGTH;
    use crate::state::{Agent, AgentType, AppState};

    fn create_test_state() -> Arc<RwLock<AppState>> {
        Arc::new(RwLock::new(AppState::new()))
    }

    #[tokio::test]
    async fn test_query_agent_empty_query() {
        let state = create_test_state();
        // Create an agent
        let mut state_write = state.write().await;
        let agent = Agent::new(
            "test-1".to_string(),
            "Test Agent".to_string(),
            AgentType::Generic,
        );
        state_write.add_agent(agent);
        drop(state_write);

        let request = QueryRequest {
            query: "".to_string(),
        };

        let result = query_agent(State(state), Path("test-1".to_string()), Json(request)).await;
        assert!(result.is_err(), "Should fail with empty query");
    }

    #[tokio::test]
    async fn test_query_agent_too_long() {
        let state = create_test_state();
        // Create an agent
        let mut state_write = state.write().await;
        let agent = Agent::new(
            "test-1".to_string(),
            "Test Agent".to_string(),
            AgentType::Generic,
        );
        state_write.add_agent(agent);
        drop(state_write);

        let request = QueryRequest {
            query: "a".repeat(MAX_QUERY_LENGTH + 1),
        };

        let result = query_agent(State(state), Path("test-1".to_string()), Json(request)).await;
        assert!(result.is_err(), "Should fail with too long query");
    }
}
