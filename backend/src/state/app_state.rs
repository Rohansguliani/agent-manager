// Application state management
// Contains agent registry, selected agent, and UI state

use crate::state::config::{AgentConfig, AgentType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for an agent
pub type AgentId = String;

/// Agent status enumeration
/// Represents the current lifecycle state of an agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is not running
    Idle,
    /// Agent is currently running
    Running,
    /// Agent has been stopped
    Stopped,
    /// Agent encountered an error
    Error,
}

/// Agent structure
/// Represents a CLI agent with its configuration and state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    /// Unique identifier for the agent
    pub id: AgentId,
    /// Display name of the agent
    pub name: String,
    /// Type of the agent (Gemini, ClaudeCode, Generic, etc.)
    pub agent_type: AgentType,
    /// Current status of the agent
    pub status: AgentStatus,
    /// Agent configuration (command, args, env vars, etc.)
    pub config: AgentConfig,
}

impl Agent {
    /// Create a new agent with the given ID, name, and type
    /// Uses default configuration for the agent type
    pub fn new(id: AgentId, name: String, agent_type: AgentType) -> Self {
        Self {
            id,
            name,
            agent_type: agent_type.clone(),
            status: AgentStatus::Idle,
            config: AgentConfig::for_type(&agent_type),
        }
    }

    /// Create a new agent with a custom configuration
    #[allow(dead_code)] // Reserved for future use
    pub fn with_config(
        id: AgentId,
        name: String,
        agent_type: AgentType,
        config: AgentConfig,
    ) -> Self {
        Self {
            id,
            name,
            agent_type,
            status: AgentStatus::Idle,
            config,
        }
    }

    /// Generate a new unique ID for an agent
    /// Uses UUID v4 for uniqueness
    pub fn generate_id() -> AgentId {
        Uuid::new_v4().to_string()
    }

    /// Validate the agent's configuration
    /// Returns Ok(()) if valid, Err with message if invalid
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }
        self.config.validate()?;
        Ok(())
    }
}

/// Main application state
/// Manages all application-wide state including agents and UI preferences
#[derive(Debug, Clone, Default)]
pub struct AppState {
    /// Registry of all agents (id -> Agent)
    pub agents: HashMap<AgentId, Agent>,
    /// ID of the currently selected agent, if any
    pub selected_agent_id: Option<AgentId>,
    /// UI state preferences
    pub ui_state: UiState,
}

/// UI-specific state
#[derive(Debug, Clone)]
pub struct UiState {
    /// Whether the sidebar is visible
    #[allow(dead_code)] // Reserved for future UI features
    pub sidebar_visible: bool,
    /// Whether to show terminal output
    #[allow(dead_code)] // Reserved for future UI features
    pub terminal_visible: bool,
    /// Current working directory context for AI operations
    pub working_directory: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            terminal_visible: true,
            working_directory: None, // Default to no context (current directory)
        }
    }
}

impl AppState {
    /// Create a new application state with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a reference to the selected agent, if any
    #[allow(dead_code)] // Prepared for Phase 3 (Agent Management Core)
    pub fn selected_agent(&self) -> Option<&Agent> {
        self.selected_agent_id
            .as_ref()
            .and_then(|id| self.agents.get(id))
    }

    /// Select an agent by ID
    /// Returns true if the agent was found and selected
    #[allow(dead_code)] // Reserved for future UI features
    pub fn select_agent(&mut self, id: &AgentId) -> bool {
        if self.agents.contains_key(id) {
            self.selected_agent_id = Some(id.clone());
            true
        } else {
            false
        }
    }

    /// Deselect the current agent
    #[allow(dead_code)] // Reserved for future UI features
    pub fn deselect_agent(&mut self) {
        self.selected_agent_id = None;
    }

    /// Add an agent to the registry
    /// Returns true if the agent was added (false if ID already exists)
    pub fn add_agent(&mut self, agent: Agent) -> bool {
        if self.agents.contains_key(&agent.id) {
            false
        } else {
            self.agents.insert(agent.id.clone(), agent);
            true
        }
    }

    /// Remove an agent from the registry
    /// If the removed agent was selected, selection is cleared
    /// Returns the removed agent if it existed
    #[allow(dead_code)] // Prepared for Phase 3 (Agent Management Core) - Delete agent UI
    pub fn remove_agent(&mut self, id: &AgentId) -> Option<Agent> {
        let removed = self.agents.remove(id);
        if self.selected_agent_id.as_ref() == Some(id) {
            self.selected_agent_id = None;
        }
        removed
    }

    /// Get all agents as a vector, sorted by name
    pub fn agents_list(&self) -> Vec<&Agent> {
        let mut agents: Vec<&Agent> = self.agents.values().collect();
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        agents
    }

    /// Update an agent's status
    /// Returns true if the agent was found and updated
    pub fn update_agent_status(&mut self, id: &AgentId, status: AgentStatus) -> bool {
        if let Some(agent) = self.agents.get_mut(id) {
            agent.status = status;
            true
        } else {
            false
        }
    }

    /// Update an agent in the registry
    /// Replaces the agent with the given ID if it exists
    /// Returns true if the agent was found and updated
    #[allow(dead_code)] // Reserved for future UI features
    pub fn update_agent(&mut self, id: &AgentId, updated_agent: Agent) -> bool {
        if !self.agents.contains_key(id) {
            return false;
        }
        // Ensure the ID matches
        if updated_agent.id != *id {
            return false;
        }
        self.agents.insert(id.clone(), updated_agent);
        true
    }

    /// Get an agent by ID
    /// Returns a mutable reference to the agent if found
    #[allow(dead_code)] // Reserved for future UI features
    pub fn get_agent_mut(&mut self, id: &AgentId) -> Option<&mut Agent> {
        self.agents.get_mut(id)
    }

    /// Get the number of agents in the registry
    #[allow(dead_code)] // Will be used in future UI features (footer, statistics, etc.)
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Set the working directory context
    pub fn set_working_directory(&mut self, path: Option<String>) {
        self.ui_state.working_directory = path;
    }

    /// Get the current working directory context
    pub fn working_directory(&self) -> Option<&String> {
        self.ui_state.working_directory.as_ref()
    }

    /// Load agents from a file
    /// Replaces all current agents with those loaded from the file
    /// Returns the number of agents loaded, or an error if loading failed
    pub fn load_agents<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<usize, super::persistence::PersistenceError> {
        let loaded_agents = super::persistence::AgentRegistry::load_from_file(path)?;
        let count = loaded_agents.len();
        self.agents = loaded_agents;
        Ok(count)
    }

    /// Save agents to a file
    /// Returns Ok(()) if successful, or an error if saving failed
    #[allow(dead_code)] // Reserved for future persistence features
    pub fn save_agents<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), super::persistence::PersistenceError> {
        super::persistence::AgentRegistry::save_to_file(&self.agents, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert_eq!(state.agent_count(), 0);
        assert!(state.selected_agent().is_none());
    }

    #[test]
    fn test_agent_new() {
        use crate::state::config::AgentType;
        let agent = Agent::new(
            "1".to_string(),
            "Test Agent".to_string(),
            AgentType::Generic,
        );
        assert_eq!(agent.id, "1");
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.agent_type, AgentType::Generic);
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn test_agent_generate_id() {
        let id1 = Agent::generate_id();
        let id2 = Agent::generate_id();
        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
    }

    #[test]
    fn test_agent_validate() {
        use crate::state::config::AgentType;
        // Create agent with valid config (Generic type needs a command set)
        let mut agent = Agent::new(
            "1".to_string(),
            "Test Agent".to_string(),
            AgentType::Generic,
        );
        // Set a valid command for Generic type
        agent.config.command = "test-command".to_string();
        assert!(agent.validate().is_ok());

        agent.name = "".to_string();
        assert!(agent.validate().is_err());

        agent.name = "Test Agent".to_string();
        agent.config.command = "".to_string();
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_add_agent() {
        use crate::state::config::AgentType;
        let mut state = AppState::new();
        let agent = Agent::new(
            "1".to_string(),
            "Test Agent".to_string(),
            AgentType::Generic,
        );

        assert!(state.add_agent(agent.clone()));
        assert_eq!(state.agent_count(), 1);
        assert!(!state.add_agent(agent)); // Duplicate ID should fail
        assert_eq!(state.agent_count(), 1);
    }

    #[test]
    fn test_select_agent() {
        use crate::state::config::AgentType;
        let mut state = AppState::new();
        let agent = Agent::new(
            "1".to_string(),
            "Test Agent".to_string(),
            AgentType::Generic,
        );
        state.add_agent(agent);

        assert!(state.select_agent(&"1".to_string()));
        assert!(state.selected_agent().is_some());
        assert_eq!(state.selected_agent().unwrap().name, "Test Agent");

        assert!(!state.select_agent(&"999".to_string())); // Non-existent ID
    }

    #[test]
    fn test_remove_agent() {
        use crate::state::config::AgentType;
        let mut state = AppState::new();
        let agent = Agent::new(
            "1".to_string(),
            "Test Agent".to_string(),
            AgentType::Generic,
        );
        state.add_agent(agent);
        state.select_agent(&"1".to_string());

        let removed = state.remove_agent(&"1".to_string());
        assert!(removed.is_some());
        assert_eq!(state.agent_count(), 0);
        assert!(state.selected_agent().is_none()); // Selection should be cleared
    }

    #[test]
    fn test_update_agent_status() {
        use crate::state::config::AgentType;
        let mut state = AppState::new();
        let agent = Agent::new(
            "1".to_string(),
            "Test Agent".to_string(),
            AgentType::Generic,
        );
        state.add_agent(agent);

        assert!(state.update_agent_status(&"1".to_string(), AgentStatus::Running));
        assert_eq!(state.agents.get("1").unwrap().status, AgentStatus::Running);

        assert!(!state.update_agent_status(&"999".to_string(), AgentStatus::Running));
    }

    #[test]
    fn test_agents_list_sorted() {
        use crate::state::config::AgentType;
        let mut state = AppState::new();
        state.add_agent(Agent::new(
            "2".to_string(),
            "Beta Agent".to_string(),
            AgentType::Generic,
        ));
        state.add_agent(Agent::new(
            "1".to_string(),
            "Alpha Agent".to_string(),
            AgentType::Generic,
        ));
        state.add_agent(Agent::new(
            "3".to_string(),
            "Gamma Agent".to_string(),
            AgentType::Generic,
        ));

        let agents = state.agents_list();
        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0].name, "Alpha Agent");
        assert_eq!(agents[1].name, "Beta Agent");
        assert_eq!(agents[2].name, "Gamma Agent");
    }
}
