// Agent persistence module
// Handles saving and loading agent configurations to/from files

use super::app_state::{Agent, AgentId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Error types for persistence operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceError {
    /// File I/O error
    IoError(String),
    /// JSON serialization/deserialization error
    JsonError(String),
    /// Invalid data format
    InvalidData(String),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::IoError(msg) => write!(f, "IO Error: {}", msg),
            PersistenceError::JsonError(msg) => write!(f, "JSON Error: {}", msg),
            PersistenceError::InvalidData(msg) => write!(f, "Invalid Data: {}", msg),
        }
    }
}

impl std::error::Error for PersistenceError {}

/// Serializable structure for agent registry
/// Used for saving/loading agents to/from JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentRegistryData {
    /// Version of the registry format (for future migration support)
    version: u32,
    /// Map of agent ID to agent data
    agents: HashMap<AgentId, Agent>,
}

/// Agent registry persistence operations
pub struct AgentRegistry;

impl AgentRegistry {
    /// Save agents to a JSON file
    ///
    /// # Arguments
    /// * `agents` - HashMap of agents to save
    /// * `path` - Path to the JSON file
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(PersistenceError)` if an error occurred
    #[allow(dead_code)] // Reserved for future persistence features
    pub fn save_to_file<P: AsRef<Path>>(
        agents: &HashMap<AgentId, Agent>,
        path: P,
    ) -> Result<(), PersistenceError> {
        let data = AgentRegistryData {
            version: 1,
            agents: agents.clone(),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| PersistenceError::JsonError(e.to_string()))?;

        fs::write(path.as_ref(), json).map_err(|e| PersistenceError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Load agents from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to the JSON file
    ///
    /// # Returns
    /// * `Ok(HashMap<AgentId, Agent>)` if successful
    /// * `Err(PersistenceError)` if an error occurred
    pub fn load_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<HashMap<AgentId, Agent>, PersistenceError> {
        if !path.as_ref().exists() {
            return Ok(HashMap::new());
        }

        let json = fs::read_to_string(path.as_ref())
            .map_err(|e| PersistenceError::IoError(e.to_string()))?;

        let data: AgentRegistryData =
            serde_json::from_str(&json).map_err(|e| PersistenceError::JsonError(e.to_string()))?;

        // Validate version (for future migration support)
        if data.version != 1 {
            return Err(PersistenceError::InvalidData(format!(
                "Unsupported registry version: {}",
                data.version
            )));
        }

        Ok(data.agents)
    }

    /// Get the default path for the agent registry file
    /// Returns a path in the user's home directory or current directory
    pub fn default_path() -> std::path::PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = std::path::PathBuf::from(home);
            path.push(".agent-manager");
            path.push("agents.json");
            path
        } else {
            std::path::PathBuf::from("agents.json")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::config::AgentType;
    use tempfile::NamedTempFile;

    #[test]
    fn test_agent_registry_serialization() {
        let mut agents = HashMap::new();
        let mut agent = Agent::new(
            "agent-1".to_string(),
            "Test Agent".to_string(),
            AgentType::Generic,
        );
        agent.config.command = "test-command".to_string();
        agents.insert("agent-1".to_string(), agent);

        let data = AgentRegistryData {
            version: 1,
            agents: agents.clone(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: AgentRegistryData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, 1);
        assert_eq!(deserialized.agents.len(), 1);
        assert!(deserialized.agents.contains_key("agent-1"));
    }

    #[test]
    fn test_save_and_load_from_file() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Create test agents
        let mut agents = HashMap::new();
        let mut agent1 = Agent::new(
            "agent-1".to_string(),
            "Agent 1".to_string(),
            AgentType::Gemini,
        );
        agent1.config.command = "gemini".to_string();
        agents.insert("agent-1".to_string(), agent1);

        let mut agent2 = Agent::new(
            "agent-2".to_string(),
            "Agent 2".to_string(),
            AgentType::ClaudeCode,
        );
        agent2.config.command = "claude".to_string();
        agents.insert("agent-2".to_string(), agent2);

        // Save to file
        AgentRegistry::save_to_file(&agents, path).unwrap();

        // Load from file
        let loaded_agents = AgentRegistry::load_from_file(path).unwrap();

        assert_eq!(loaded_agents.len(), 2);
        assert!(loaded_agents.contains_key("agent-1"));
        assert!(loaded_agents.contains_key("agent-2"));
        assert_eq!(loaded_agents.get("agent-1").unwrap().name, "Agent 1");
        assert_eq!(loaded_agents.get("agent-2").unwrap().name, "Agent 2");
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        // Delete the file
        std::fs::remove_file(path).unwrap();

        // Should return empty HashMap for non-existent file
        let agents = AgentRegistry::load_from_file(path).unwrap();
        assert!(agents.is_empty());
    }
}
