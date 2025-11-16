//! Orchestrator configuration
//!
//! Centralized configuration for orchestrator components.

/// Orchestrator configuration
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when refactoring to use config struct
pub struct OrchestratorConfig {
    /// Gemini API timeout in seconds
    pub gemini_timeout_secs: u64,
    /// Gemini model name
    pub gemini_model: String,
    /// Gemini API base URL
    pub gemini_api_base_url: String,
    /// Maximum goal length in characters
    pub max_goal_length: usize,
    /// Plan execution timeout in seconds
    pub plan_timeout_secs: u64,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            gemini_timeout_secs: 30,
            gemini_model: "gemini-2.5-flash".to_string(),
            gemini_api_base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            max_goal_length: 10000, // 10KB
            plan_timeout_secs: 300, // 5 minutes
        }
    }
}
