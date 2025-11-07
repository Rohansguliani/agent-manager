// Application state management
// Contains agent registry, selected agent, and UI state

use std::collections::HashMap;

/// Unique identifier for an agent
pub type AgentId = String;

/// Agent status enumeration
/// Represents the current lifecycle state of an agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentStatus {
    /// Agent is not running
    Idle,
    /// Agent is currently running
    Running,
    /// Agent has been stopped
    Stopped,
    /// Agent encountered an error
    #[allow(dead_code)] // Will be used in Phase 4 (CLI Integration) for error handling
    Error,
}

/// Placeholder agent structure
/// Full agent model will be implemented in Phase 3
#[derive(Debug, Clone)]
pub struct Agent {
    /// Unique identifier for the agent
    pub id: AgentId,
    /// Display name of the agent
    pub name: String,
    /// Current status of the agent
    pub status: AgentStatus,
}

impl Agent {
    /// Create a new agent with the given ID and name
    pub fn new(id: AgentId, name: String) -> Self {
        Self {
            id,
            name,
            status: AgentStatus::Idle,
        }
    }
}

/// Main application state
/// Manages all application-wide state including agents and UI preferences
#[derive(Debug, Clone)]
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
    pub sidebar_visible: bool,
    /// Whether to show terminal output
    pub terminal_visible: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            terminal_visible: true,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            agents: HashMap::new(),
            selected_agent_id: None,
            ui_state: UiState::default(),
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
    pub fn select_agent(&mut self, id: &AgentId) -> bool {
        if self.agents.contains_key(id) {
            self.selected_agent_id = Some(id.clone());
            true
        } else {
            false
        }
    }

    /// Deselect the current agent
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

    /// Get the number of agents in the registry
    #[allow(dead_code)] // Will be used in future UI features (footer, statistics, etc.)
    pub fn agent_count(&self) -> usize {
        self.agents.len()
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
    fn test_add_agent() {
        let mut state = AppState::new();
        let agent = Agent::new("1".to_string(), "Test Agent".to_string());
        
        assert!(state.add_agent(agent.clone()));
        assert_eq!(state.agent_count(), 1);
        assert!(!state.add_agent(agent)); // Duplicate ID should fail
        assert_eq!(state.agent_count(), 1);
    }

    #[test]
    fn test_select_agent() {
        let mut state = AppState::new();
        let agent = Agent::new("1".to_string(), "Test Agent".to_string());
        state.add_agent(agent);
        
        assert!(state.select_agent(&"1".to_string()));
        assert!(state.selected_agent().is_some());
        assert_eq!(state.selected_agent().unwrap().name, "Test Agent");
        
        assert!(!state.select_agent(&"999".to_string())); // Non-existent ID
    }

    #[test]
    fn test_remove_agent() {
        let mut state = AppState::new();
        let agent = Agent::new("1".to_string(), "Test Agent".to_string());
        state.add_agent(agent);
        state.select_agent(&"1".to_string());
        
        let removed = state.remove_agent(&"1".to_string());
        assert!(removed.is_some());
        assert_eq!(state.agent_count(), 0);
        assert!(state.selected_agent().is_none()); // Selection should be cleared
    }

    #[test]
    fn test_update_agent_status() {
        let mut state = AppState::new();
        let agent = Agent::new("1".to_string(), "Test Agent".to_string());
        state.add_agent(agent);
        
        assert!(state.update_agent_status(&"1".to_string(), AgentStatus::Running));
        assert_eq!(state.agents.get(&"1".to_string()).unwrap().status, AgentStatus::Running);
        
        assert!(!state.update_agent_status(&"999".to_string(), AgentStatus::Running));
    }

    #[test]
    fn test_agents_list_sorted() {
        let mut state = AppState::new();
        state.add_agent(Agent::new("2".to_string(), "Beta Agent".to_string()));
        state.add_agent(Agent::new("1".to_string(), "Alpha Agent".to_string()));
        state.add_agent(Agent::new("3".to_string(), "Gamma Agent".to_string()));
        
        let agents = state.agents_list();
        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0].name, "Alpha Agent");
        assert_eq!(agents[1].name, "Beta Agent");
        assert_eq!(agents[2].name, "Gamma Agent");
    }
}

