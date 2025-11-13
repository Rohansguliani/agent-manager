//! Agent configuration module
//!
//! Defines the structure and types for agent configurations.
//!
//! This module handles agent-level configuration (agent types, agent configs).
//! For application-level configuration (server settings, persistence settings,
//! execution settings), see `config`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent type enumeration
/// Represents the different types of CLI agents supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    /// Gemini CLI agent
    Gemini,
    /// Claude Code agent
    ClaudeCode,
    /// Generic CLI agent (custom command)
    Generic,
    /// Other/custom agent type
    Other(String),
}

impl AgentType {
    /// Get a display name for the agent type
    #[allow(dead_code)] // Reserved for future UI features
    pub fn display_name(&self) -> String {
        match self {
            AgentType::Gemini => "Gemini CLI".to_string(),
            AgentType::ClaudeCode => "Claude Code".to_string(),
            AgentType::Generic => "Generic CLI".to_string(),
            AgentType::Other(name) => name.clone(),
        }
    }

    /// Get all available agent types (for UI dropdowns)
    #[allow(dead_code)] // Reserved for future UI features
    pub fn available_types() -> Vec<AgentType> {
        vec![AgentType::Gemini, AgentType::ClaudeCode, AgentType::Generic]
    }
}

impl Default for AgentType {
    #[allow(clippy::derivable_impls)] // Cannot derive Default with Other(String) variant
    fn default() -> Self {
        AgentType::Generic
    }
}

/// Agent configuration structure
/// Contains all configurable settings for an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgentConfig {
    /// Command to execute the agent (e.g., "gemini", "claude", or custom command)
    pub command: String,
    /// Command-line arguments for the agent
    pub args: Vec<String>,
    /// Environment variables for the agent process
    pub env_vars: HashMap<String, String>,
    /// Working directory for the agent process (None = current directory)
    pub working_dir: Option<String>,
    /// Additional configuration options (key-value pairs)
    /// Used for agent-type-specific settings
    pub options: HashMap<String, String>,
}

impl AgentConfig {
    /// Create a new agent configuration with a command
    pub fn new(command: String) -> Self {
        Self {
            command,
            args: Vec::new(),
            env_vars: HashMap::new(),
            working_dir: None,
            options: HashMap::new(),
        }
    }

    /// Create a default configuration for a specific agent type
    pub fn for_type(agent_type: &AgentType) -> Self {
        match agent_type {
            AgentType::Gemini => {
                // Try to find gemini in PATH, fallback to common locations
                let gemini_path = if let Ok(path) = std::env::var("GEMINI_CLI_PATH") {
                    path
                } else {
                    // Check common installation paths (including npm global install location)
                    // Order matters: check npm global location first (Docker), then host locations
                    let common_paths = [
                        "/usr/local/bin/gemini",    // npm global install (Docker/Linux)
                        "/opt/homebrew/bin/gemini", // Homebrew on macOS (host)
                        "/usr/bin/gemini",          // System-wide (Linux)
                    ];
                    let found = common_paths
                        .iter()
                        .find(|path| std::path::Path::new(path).exists());
                    found
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "gemini".to_string())
                };

                Self {
                    command: gemini_path,
                    args: vec!["--yolo".to_string()], // Auto-approve all actions (no prompts)
                    env_vars: HashMap::new(),
                    working_dir: None,
                    options: HashMap::new(),
                }
            }
            AgentType::ClaudeCode => Self {
                command: "claude".to_string(),
                args: Vec::new(),
                env_vars: HashMap::new(),
                working_dir: None,
                options: HashMap::new(),
            },
            AgentType::Generic => Self::default(),
            AgentType::Other(cmd) => Self::new(cmd.clone()),
        }
    }

    /// Validate the configuration
    /// Returns Ok(()) if valid, Err with message if invalid
    pub fn validate(&self) -> Result<(), String> {
        if self.command.is_empty() {
            return Err("Command cannot be empty".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_display_name() {
        assert_eq!(AgentType::Gemini.display_name(), "Gemini CLI");
        assert_eq!(AgentType::ClaudeCode.display_name(), "Claude Code");
        assert_eq!(AgentType::Generic.display_name(), "Generic CLI");
        assert_eq!(
            AgentType::Other("Custom".to_string()).display_name(),
            "Custom"
        );
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert!(config.command.is_empty());
        assert!(config.args.is_empty());
        assert!(config.env_vars.is_empty());
        assert!(config.working_dir.is_none());
        assert!(config.options.is_empty());
    }

    #[test]
    fn test_agent_config_new() {
        let config = AgentConfig::new("test-command".to_string());
        assert_eq!(config.command, "test-command");
        assert!(config.args.is_empty());
    }

    #[test]
    fn test_agent_config_for_type() {
        let gemini_config = AgentConfig::for_type(&AgentType::Gemini);
        // Command might be a full path (e.g., "/opt/homebrew/bin/gemini") or just "gemini"
        // depending on where it's installed on the system
        assert!(
            gemini_config.command == "gemini" || gemini_config.command.ends_with("/gemini"),
            "Command should be 'gemini' or end with '/gemini', got: {}",
            gemini_config.command
        );
        assert_eq!(gemini_config.args, vec!["--yolo"]);

        let claude_config = AgentConfig::for_type(&AgentType::ClaudeCode);
        assert_eq!(claude_config.command, "claude");

        let generic_config = AgentConfig::for_type(&AgentType::Generic);
        assert!(generic_config.command.is_empty());
    }

    #[test]
    fn test_agent_config_validate() {
        let mut config = AgentConfig::default();
        assert!(config.validate().is_err());

        config.command = "test".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig::new("test-command".to_string());
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}
