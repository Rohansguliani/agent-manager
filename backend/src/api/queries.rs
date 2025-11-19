//! Query execution API handlers
//!
//! Contains HTTP request handlers for executing queries with agents
//! and streaming responses using Server-Sent Events (SSE).

use crate::api::utils::{
    apply_working_directory_context, create_executor, update_agent_status, validate_query,
    RouterState,
};
use crate::chat::{Message, MessageRole};
use crate::error::AppError;
use crate::state::{AgentId, AgentStatus};
use axum::{
    extract::{Path, State},
    response::{Json, Response},
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

/// Query request
#[derive(Deserialize)]
pub struct QueryRequest {
    /// The query string to execute
    pub query: String,
    /// Optional conversation ID to associate this query with a chat conversation
    pub conversation_id: Option<String>,
}

/// Query response
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    /// The response from the agent
    pub response: String,
    /// ID of the agent that executed the query
    pub agent_id: AgentId,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// POST /api/agents/:id/query - Execute a query with the agent
pub async fn query_agent(
    State((state, _, _)): State<RouterState>,
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
/// Uses persistent subprocess per conversation (no manual context building)
pub async fn query_stream(
    State((_state, chat_db, _process_manager)): State<RouterState>,
    Json(request): Json<QueryRequest>,
) -> Result<Response, AppError> {
    // Validate query
    validate_query(&request.query)?;

    // Get conversation_id - required for persistent subprocess approach
    let conversation_id = request.conversation_id.as_ref().ok_or_else(|| {
        AppError::InvalidAgentConfig(
            "conversation_id is required for persistent subprocess approach".to_string(),
        )
    })?;

    // Verify conversation exists
    let conversation = chat_db
        .get_conversation(conversation_id)
        .await?
        .ok_or_else(|| {
            AppError::FileNotFound(format!("Conversation not found: {}", conversation_id))
        })?;

    // Create user message
    let user_message = Message::new(
        Uuid::new_v4().to_string(),
        conversation_id.clone(),
        MessageRole::User,
        request.query.clone(),
    );

    // Save user message first
    chat_db.add_message(&user_message).await?;

    // Update title if needed (after message is saved)
    if conversation.title == "New Chat" {
        let generated_title = crate::api::chat::generate_title_from_message(&request.query);
        chat_db
            .update_conversation(conversation_id, &generated_title)
            .await?;
    }

    // Get conversation history (excluding the message we just added)
    // We'll include all previous messages for context
    let mut conversation_history = chat_db.get_messages(conversation_id).await?;
    // Remove the user message we just added (we'll add it back with the query)
    conversation_history.pop();

    // TODO: Update to use bridge manager once implemented (Phase 3)
    // For now, this endpoint is disabled - use simple_chat endpoint instead
    Err(AppError::Internal(anyhow::anyhow!(
        "Bridge approach not yet implemented for queries endpoint. Use simple_chat endpoint instead."
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::utils::{RouterState, MAX_QUERY_LENGTH};
    use crate::chat::ChatDb;
    use crate::state::{Agent, AgentType, AppState};
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
    async fn test_query_agent_empty_query() {
        let router_state = create_test_router_state().await;
        let (state, _, _) = &router_state;
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
            conversation_id: None,
        };

        let result = query_agent(
            State(router_state.clone()),
            Path("test-1".to_string()),
            Json(request),
        )
        .await;
        assert!(result.is_err(), "Should fail with empty query");
    }

    #[tokio::test]
    async fn test_query_agent_too_long() {
        let router_state = create_test_router_state().await;
        let (state, _, _) = &router_state;
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
            conversation_id: None,
        };

        let result = query_agent(
            State(router_state.clone()),
            Path("test-1".to_string()),
            Json(request),
        )
        .await;
        assert!(result.is_err(), "Should fail with too long query");
    }
}
