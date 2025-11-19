//! Agent management API handlers
//!
//! Contains HTTP request handlers for agent CRUD operations.

use crate::api::utils::RouterState;
use crate::error::AppError;
use crate::state::{Agent, AgentId, AgentStatus, AgentType};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

/// Agent response type
#[derive(Debug, Serialize)]
pub struct AgentResponse {
    /// Unique identifier for the agent
    pub id: AgentId,
    /// Human-readable name of the agent
    pub name: String,
    /// Type of agent (e.g., Gemini, OpenAI)
    pub agent_type: AgentType,
    /// Current status of the agent (Running, Stopped, etc.)
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

/// Agents list response
#[derive(Serialize)]
pub struct AgentsListResponse {
    /// List of all agents
    pub agents: Vec<AgentResponse>,
    /// Total number of agents
    pub count: usize,
}

/// Message response
#[derive(Serialize)]
pub struct MessageResponse {
    /// Human-readable message
    pub message: String,
    /// Status indicator (e.g., "ok", "error")
    pub status: String,
}

/// Create agent request
#[derive(Deserialize)]
pub struct CreateAgentRequest {
    /// Name for the new agent
    pub name: String,
    /// Type of agent to create
    pub agent_type: AgentType,
}

/// Update agent request
#[derive(Deserialize)]
pub struct UpdateAgentRequest {
    /// New name for the agent (optional)
    pub name: Option<String>,
    /// New agent type (optional)
    pub agent_type: Option<AgentType>,
    /// New status for the agent (optional)
    pub status: Option<AgentStatus>,
}

/// GET /api/agents - List all agents
pub async fn list_agents(
    State((state, _, _)): State<RouterState>,
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

/// GET /api/agents/:id - Get a specific agent
pub async fn get_agent(
    State((state, _, _)): State<RouterState>,
    Path(id): Path<AgentId>,
) -> Result<Json<AgentResponse>, AppError> {
    let state = state.read().await;
    let agent = state
        .agents
        .get(&id)
        .ok_or_else(|| AppError::AgentNotFound(id.clone()))?;

    Ok(Json(AgentResponse::from(agent)))
}

/// POST /api/agents - Create a new agent
pub async fn create_agent(
    State((state, _, _)): State<RouterState>,
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

/// PUT /api/agents/:id - Update an agent
pub async fn update_agent(
    State((state, _, _)): State<RouterState>,
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

/// DELETE /api/agents/:id - Delete an agent
pub async fn delete_agent(
    State((state, _, _)): State<RouterState>,
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

/// POST /api/agents/:id/start - Start an agent
pub async fn start_agent(
    State((state, _, _)): State<RouterState>,
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

/// POST /api/agents/:id/stop - Stop an agent
pub async fn stop_agent(
    State((state, _, _)): State<RouterState>,
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
    async fn test_list_agents_empty() {
        let router_state = create_test_router_state().await;
        let result = list_agents(State(router_state)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.count, 0);
        assert_eq!(response.agents.len(), 0);
    }

    #[tokio::test]
    async fn test_create_agent() {
        let router_state = create_test_router_state().await;
        // Use Gemini type which has a default command configured
        let request = CreateAgentRequest {
            name: "Test Agent".to_string(),
            agent_type: AgentType::Gemini,
        };

        let result = create_agent(State(router_state.clone()), Json(request)).await;
        assert!(
            result.is_ok(),
            "Agent creation should succeed with Gemini type"
        );
        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response.name, "Test Agent");

        // Verify agent is in list
        let list_result = list_agents(State(router_state.clone())).await;
        assert!(list_result.is_ok());
        let list_response = list_result.unwrap();
        assert_eq!(list_response.count, 1);
    }

    #[tokio::test]
    async fn test_get_agent_not_found() {
        let router_state = create_test_router_state().await;
        let result = get_agent(State(router_state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::AgentNotFound(_) => {
                // Expected error
            }
            other => {
                panic!("Expected AgentNotFound error, got: {:?}", other);
            }
        }
    }
}
