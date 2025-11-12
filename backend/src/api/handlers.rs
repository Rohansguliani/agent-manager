//! API handlers for agent management
//!
//! This module contains all HTTP request handlers for the agent management API.
//! Each handler function corresponds to a specific API endpoint and handles
//! request/response serialization, state management, and error handling.

use crate::config::Config;
use crate::error::AppError;
use crate::executor::{CliExecutor, StreamingCliExecutor};
use crate::state::{Agent, AgentId, AgentStatus, AgentType, AppState};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Json, Response},
};
use futures_util::{stream::Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

// Constants
const MAX_QUERY_LENGTH: usize = 10_000; // 10KB max query length

// Response types
#[derive(Serialize)]
pub struct AgentResponse {
    pub id: AgentId,
    pub name: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
}

impl From<&Agent> for AgentResponse {
    fn from(agent: &Agent) -> Self {
        Self {
            id: agent.id.clone(),
            name: agent.name.clone(),
            agent_type: agent.agent_type.clone(),
            status: agent.status,
        }
    }
}

#[derive(Serialize)]
pub struct AgentsListResponse {
    pub agents: Vec<AgentResponse>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
    pub status: String,
}

// Request types
#[derive(Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub agent_type: AgentType,
}

#[derive(Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub agent_type: Option<AgentType>,
    pub status: Option<AgentStatus>,
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub response: String,
    pub agent_id: AgentId,
    pub execution_time_ms: u64,
}

// GET /api/agents - List all agents
pub async fn list_agents(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<AgentsListResponse>, AppError> {
    let state = state.read().await;
    let agents: Vec<AgentResponse> = state
        .agents_list()
        .iter()
        .map(|agent| AgentResponse::from(*agent))
        .collect();

    Ok(Json(AgentsListResponse {
        count: agents.len(),
        agents,
    }))
}

// GET /api/agents/:id - Get a specific agent
pub async fn get_agent(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<AgentId>,
) -> Result<Json<AgentResponse>, AppError> {
    let state = state.read().await;
    let agent = state
        .agents
        .get(&id)
        .ok_or_else(|| AppError::AgentNotFound(id.clone()))?;

    Ok(Json(AgentResponse::from(agent)))
}

// POST /api/agents - Create a new agent
pub async fn create_agent(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<AgentResponse>), AppError> {
    let id = Agent::generate_id();
    let agent = Agent::new(id.clone(), request.name, request.agent_type);

    // Validate agent
    agent.validate().map_err(AppError::InvalidAgentConfig)?;

    let mut state = state.write().await;
    if !state.add_agent(agent) {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Failed to add agent (ID already exists)"
        )));
    }

    let agent = state
        .agents
        .get(&id)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Agent not found after creation")))?;

    Ok((StatusCode::CREATED, Json(AgentResponse::from(agent))))
}

// PUT /api/agents/:id - Update an agent
pub async fn update_agent(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<AgentId>,
    Json(request): Json<UpdateAgentRequest>,
) -> Result<Json<AgentResponse>, AppError> {
    let mut state = state.write().await;
    let agent = state
        .agents
        .get_mut(&id)
        .ok_or_else(|| AppError::AgentNotFound(id.clone()))?;

    if let Some(name) = request.name {
        agent.name = name;
    }

    if let Some(agent_type) = request.agent_type {
        agent.agent_type = agent_type.clone();
        agent.config = crate::state::AgentConfig::for_type(&agent_type);
    }

    if let Some(status) = request.status {
        agent.status = status;
    }

    // Validate updated agent
    agent.validate().map_err(AppError::InvalidAgentConfig)?;

    let agent = state
        .agents
        .get(&id)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Agent not found after update")))?;

    Ok(Json(AgentResponse::from(agent)))
}

// DELETE /api/agents/:id - Delete an agent
pub async fn delete_agent(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<AgentId>,
) -> Result<Json<MessageResponse>, AppError> {
    let mut state = state.write().await;
    state
        .remove_agent(&id)
        .ok_or_else(|| AppError::AgentNotFound(id))?;

    Ok(Json(MessageResponse {
        message: "Agent deleted successfully".to_string(),
        status: "ok".to_string(),
    }))
}

// POST /api/agents/:id/start - Start an agent (placeholder for future)
pub async fn start_agent(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<AgentId>,
) -> Result<Json<AgentResponse>, AppError> {
    let mut state = state.write().await;
    if !state.update_agent_status(&id, AgentStatus::Running) {
        return Err(AppError::AgentNotFound(id));
    }

    let agent = state
        .agents
        .get(&id)
        .ok_or_else(|| AppError::AgentNotFound(id.clone()))?;

    Ok(Json(AgentResponse::from(agent)))
}

// POST /api/agents/:id/stop - Stop an agent (placeholder for future)
pub async fn stop_agent(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<AgentId>,
) -> Result<Json<AgentResponse>, AppError> {
    let mut state = state.write().await;
    if !state.update_agent_status(&id, AgentStatus::Stopped) {
        return Err(AppError::AgentNotFound(id));
    }

    let agent = state
        .agents
        .get(&id)
        .ok_or_else(|| AppError::AgentNotFound(id.clone()))?;

    Ok(Json(AgentResponse::from(agent)))
}

// Helper function to validate query
fn validate_query(query: &str) -> Result<(), AppError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidAgentConfig(
            "Query cannot be empty".to_string(),
        ));
    }
    if trimmed.len() > MAX_QUERY_LENGTH {
        return Err(AppError::InvalidAgentConfig(format!(
            "Query exceeds maximum length of {} characters",
            MAX_QUERY_LENGTH
        )));
    }
    Ok(())
}

// Helper function to update agent status
async fn update_agent_status(
    state: &Arc<RwLock<AppState>>,
    agent_id: &AgentId,
    status: AgentStatus,
) {
    let mut state = state.write().await;
    state.update_agent_status(agent_id, status);
}

// Helper function to apply working directory context to an agent
fn apply_working_directory_context(agent: &mut Agent, state: &AppState) {
    if let Some(dir) = state.working_directory() {
        agent.config.working_dir = Some(dir.clone());
    }
}

// Helper function to create executor from config or default
fn create_executor(config: Option<&Config>) -> CliExecutor {
    let timeout = config
        .map(|c| c.execution.default_timeout_secs)
        .unwrap_or(30);
    CliExecutor::new(timeout)
}

// POST /api/agents/:id/query - Execute a query with the agent
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
    // Note: Config could be passed via State in the future for better modularity
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

// POST /api/query/stream - Stream query response using Server-Sent Events (simplified - auto-uses first Gemini agent)
pub async fn query_stream(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<QueryRequest>,
) -> Result<Response, AppError> {
    // Find first Gemini agent or create one, and apply working directory context
    let agent = {
        let state_read = state.read().await;
        // Try to find a Gemini agent
        let gemini_agent = state_read
            .agents
            .values()
            .find(|a| matches!(a.agent_type, AgentType::Gemini))
            .cloned();

        if let Some(mut agent) = gemini_agent {
            // Apply working directory context
            apply_working_directory_context(&mut agent, &state_read);
            agent
        } else {
            // Auto-create a Gemini agent if none exists
            drop(state_read);
            let mut state_write = state.write().await;
            let mut agent = Agent::new(
                Uuid::new_v4().to_string(),
                "Gemini Agent".to_string(),
                AgentType::Gemini,
            );
            // Apply working directory context
            apply_working_directory_context(&mut agent, &state_write);
            state_write.add_agent(agent.clone());
            agent
        }
    };

    // Validate query
    validate_query(&request.query)?;

    // Update agent status to Running
    update_agent_status(&state, &agent.id, AgentStatus::Running).await;

    // Create streaming executor
    let executor = StreamingCliExecutor::new(30);

    // Create SSE stream
    let stream = create_stream(executor, agent, request.query, state);

    let sse_stream = stream.map(|event_result| {
        let sse_text = match event_result {
            Ok(data) => format!("data: {}\n\n", data),
            Err(e) => format!("data: [ERROR] {}\n\n", e),
        };
        Ok::<_, std::io::Error>(sse_text)
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(sse_stream))
        .unwrap())
}

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
                yield Ok("[DONE]".to_string());
            }
            Err(e) => {
                update_agent_status(&app_state, &agent_id, AgentStatus::Error).await;
                yield Ok(format!("[ERROR] {}", e));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::state::{AgentType, AppState};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_state() -> Arc<RwLock<AppState>> {
        Arc::new(RwLock::new(AppState::new()))
    }

    #[tokio::test]
    async fn test_list_agents_empty() {
        let state = create_test_state();
        let result = list_agents(State(state)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.count, 0);
    }

    #[tokio::test]
    async fn test_create_agent() {
        let state = create_test_state();
        // Use Gemini type which has a default command, or Generic needs command set
        let request = CreateAgentRequest {
            name: "Test Agent".to_string(),
            agent_type: AgentType::Gemini, // Gemini has default command
        };
        let result = create_agent(State(state), Json(request)).await;
        assert!(result.is_ok(), "Agent creation should succeed");
        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response.name, "Test Agent");
    }

    #[tokio::test]
    async fn test_query_agent_not_found() {
        let state = create_test_state();
        let request = QueryRequest {
            query: "test query".to_string(),
        };
        let result = query_agent(
            State(state),
            Path("nonexistent-id".to_string()),
            Json(request),
        )
        .await;
        assert!(result.is_err(), "Query should fail for nonexistent agent");
        match result.unwrap_err() {
            AppError::AgentNotFound(_) => {
                // Expected error
            }
            other => {
                panic!("Expected AgentNotFound error, got: {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn test_query_agent_empty_query() {
        let state = create_test_state();
        // Create an agent first
        let create_request = CreateAgentRequest {
            name: "Test Agent".to_string(),
            agent_type: AgentType::Gemini,
        };
        let (_, agent_response) = create_agent(State(state.clone()), Json(create_request))
            .await
            .unwrap();

        // Try to query with empty string
        let query_request = QueryRequest {
            query: "   ".to_string(), // Whitespace only
        };
        let result = query_agent(
            State(state),
            Path(agent_response.id.clone()),
            Json(query_request),
        )
        .await;
        assert!(result.is_err(), "Query should fail with empty query");
        match result.unwrap_err() {
            AppError::InvalidAgentConfig(_) => {
                // Expected error
            }
            other => {
                panic!("Expected InvalidAgentConfig error, got: {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn test_query_agent_with_echo() {
        let state = create_test_state();
        // Create a generic agent with echo command
        let mut app_state = state.write().await;
        let agent = Agent::new(
            "test-echo-1".to_string(),
            "Echo Agent".to_string(),
            AgentType::Generic,
        );
        // Set command to echo
        let mut agent = agent;
        agent.config.command = "echo".to_string();
        agent.config.args = vec!["Hello from test".to_string()];
        app_state.add_agent(agent);
        drop(app_state);

        // Query the agent
        let query_request = QueryRequest {
            query: "test".to_string(),
        };
        let result = query_agent(
            State(state),
            Path("test-echo-1".to_string()),
            Json(query_request),
        )
        .await;

        // Should succeed (echo command should work)
        assert!(result.is_ok(), "Query should succeed with echo command");
        let response = result.unwrap();
        assert_eq!(response.agent_id, "test-echo-1");
        assert!(response.execution_time_ms > 0);
        // Echo output might vary, but should contain something
        assert!(!response.response.is_empty());
    }

    #[tokio::test]
    async fn test_query_agent_too_long() {
        let state = create_test_state();
        // Create an agent
        let create_request = CreateAgentRequest {
            name: "Test Agent".to_string(),
            agent_type: AgentType::Gemini,
        };
        let (_, agent_response) = create_agent(State(state.clone()), Json(create_request))
            .await
            .unwrap();

        // Try to query with very long string
        let query_request = QueryRequest {
            query: "a".repeat(MAX_QUERY_LENGTH + 1),
        };
        let result = query_agent(
            State(state),
            Path(agent_response.id.clone()),
            Json(query_request),
        )
        .await;
        assert!(result.is_err(), "Query should fail with too long query");
        match result.unwrap_err() {
            AppError::InvalidAgentConfig(_) => {
                // Expected error
            }
            other => {
                panic!("Expected InvalidAgentConfig error, got: {:?}", other);
            }
        }
    }

    #[test]
    fn test_validate_query() {
        // Test empty query
        assert!(validate_query("").is_err());
        assert!(validate_query("   ").is_err());

        // Test valid query
        assert!(validate_query("test").is_ok());
        assert!(validate_query("  test  ").is_ok());

        // Test too long query
        let long_query = "a".repeat(MAX_QUERY_LENGTH + 1);
        assert!(validate_query(&long_query).is_err());

        // Test max length query (should pass)
        let max_query = "a".repeat(MAX_QUERY_LENGTH);
        assert!(validate_query(&max_query).is_ok());
    }

    #[test]
    fn test_create_executor() {
        // Test with None config (uses default)
        let executor = create_executor(None);
        assert_eq!(executor.timeout().as_secs(), 30);

        // Test with config
        let config = Config {
            server: crate::config::ServerConfig {
                port: 8080,
                host: "0.0.0.0".to_string(),
            },
            persistence: crate::config::PersistenceConfig {
                data_dir: "/tmp".to_string(),
            },
            execution: crate::config::ExecutionConfig {
                default_timeout_secs: 60,
            },
        };
        let executor = create_executor(Some(&config));
        assert_eq!(executor.timeout().as_secs(), 60);
    }
}
