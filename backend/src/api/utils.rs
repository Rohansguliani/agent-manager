//! API utility functions
//!
//! Contains helper functions used by API handlers for validation,
//! agent status management, and executor creation.

use crate::config::Config;
use crate::error::AppError;
use crate::executor::CliExecutor;
use crate::state::{Agent, AgentId, AgentStatus, AppState};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Maximum query length in characters
pub const MAX_QUERY_LENGTH: usize = 10_000; // 10KB max query length

/// Validate query string
///
/// # Arguments
/// * `query` - Query string to validate
///
/// # Returns
/// * `Ok(())` - Query is valid
/// * `Err(AppError)` - Query is invalid (empty or too long)
pub fn validate_query(query: &str) -> Result<(), AppError> {
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

/// Update agent status in application state
///
/// # Arguments
/// * `state` - Application state
/// * `agent_id` - Agent ID to update
/// * `status` - New status
pub async fn update_agent_status(
    state: &Arc<RwLock<AppState>>,
    agent_id: &AgentId,
    status: AgentStatus,
) {
    let mut state = state.write().await;
    state.update_agent_status(agent_id, status);
}

/// Apply working directory context to an agent
///
/// # Arguments
/// * `agent` - Agent to modify
/// * `state` - Application state containing working directory
pub fn apply_working_directory_context(agent: &mut Agent, state: &AppState) {
    if let Some(dir) = state.working_directory() {
        agent.config.working_dir = Some(dir.clone());
    }
}

/// Create executor from config or use default
///
/// # Arguments
/// * `config` - Optional configuration
///
/// # Returns
/// * `CliExecutor` - Configured executor
pub fn create_executor(config: Option<&Config>) -> CliExecutor {
    let timeout = config
        .map(|c| c.execution.default_timeout_secs)
        .unwrap_or(30);
    CliExecutor::new(timeout)
}

/// Find or create a Gemini agent specifically for the planner (with JSON output)
///
/// The planner requires JSON output format, which is different from regular Gemini tasks
/// that return plain text. This function creates an agent with `--output-format json` flag.
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// * `Agent` - Gemini agent configured for planner use (JSON output)
pub async fn find_or_create_planner_agent(state: &Arc<RwLock<AppState>>) -> Agent {
    let mut agent = find_or_create_gemini_agent(state).await;

    // Add JSON output flag for planner (only if not already present)
    if !agent.config.args.iter().any(|arg| arg == "--output-format") {
        agent.config.args.push("--output-format".to_string());
        agent.config.args.push("json".to_string());
    }

    agent
}

/// Find or create a Gemini agent for general use (plain text output)
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// * `Agent` - Gemini agent (existing or newly created)
pub async fn find_or_create_gemini_agent(state: &Arc<RwLock<AppState>>) -> Agent {
    let state_read = state.read().await;
    // Try to find a Gemini agent
    let gemini_agent = state_read
        .agents
        .values()
        .find(|a| matches!(a.agent_type, crate::state::AgentType::Gemini))
        .cloned();

    if let Some(mut agent) = gemini_agent {
        drop(state_read);
        // Apply working directory context
        let state_read = state.read().await;
        let working_dir_before = agent.config.working_dir.clone();
        apply_working_directory_context(&mut agent, &state_read);
        if working_dir_before != agent.config.working_dir {
            tracing::debug!(
                agent_id = %agent.id,
                working_dir_before = ?working_dir_before,
                working_dir_after = ?agent.config.working_dir,
                "Applied working directory context to existing agent"
            );
        }
        agent
    } else {
        // Auto-create a Gemini agent if none exists
        drop(state_read);
        let mut state_write = state.write().await;
        let mut agent = Agent::new(
            uuid::Uuid::new_v4().to_string(),
            "Gemini Agent".to_string(),
            crate::state::AgentType::Gemini,
        );
        // Apply working directory context
        apply_working_directory_context(&mut agent, &state_write);
        tracing::debug!(
            agent_id = %agent.id,
            working_dir = ?agent.config.working_dir,
            "Created new Gemini agent with working directory context"
        );
        state_write.add_agent(agent.clone());
        agent
    }
}
