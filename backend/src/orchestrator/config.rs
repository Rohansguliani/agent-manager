//! Orchestrator configuration
//!
//! Centralized configuration for orchestrator components.

use crate::error::AppError;
use serde::{Deserialize, Serialize};

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize)]
pub struct OrchestratorConfig {
    /// Gemini API timeout in seconds
    #[allow(dead_code)] // Will be used when adding timeout configuration to API client
    pub gemini_timeout_secs: u64,
    /// Gemini model name
    pub gemini_model: String,
    /// Gemini API base URL
    #[allow(dead_code)] // Will be used when adding URL configuration to API client
    pub gemini_api_base_url: String,
    /// Maximum goal length in characters
    pub max_goal_length: usize,
    /// Plan execution timeout in seconds
    pub plan_timeout_secs: u64,
    /// Maximum number of parallel tasks (for concurrency limiting)
    #[allow(dead_code)] // Will be used when implementing concurrency configuration
    pub max_parallel_tasks: usize,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            gemini_timeout_secs: 30,
            gemini_model: "gemini-2.5-flash".to_string(),
            gemini_api_base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            max_goal_length: 10000, // 10KB
            plan_timeout_secs: 300, // 5 minutes
            max_parallel_tasks: 10, // Limit to 10 parallel tasks by default
        }
    }
}

/// Request body for updating orchestrator configuration
#[derive(Debug, Deserialize)]
pub struct ConfigUpdateRequest {
    /// Maximum number of parallel tasks (optional)
    pub max_parallel_tasks: Option<usize>,
    /// Gemini model name (optional)
    pub gemini_model: Option<String>,
    /// Maximum goal length in characters (optional)
    pub max_goal_length: Option<usize>,
    /// Plan execution timeout in seconds (optional)
    pub plan_timeout_secs: Option<u64>,
}

/// Validate and apply configuration updates
///
/// This function validates the update request and applies valid changes to the config.
/// Returns an error if any validation fails.
///
/// # Arguments
/// * `config` - The current config to update
/// * `request` - The update request with optional fields
///
/// # Returns
/// * `Ok(OrchestratorConfig)` - The updated configuration
/// * `Err(AppError)` - If validation fails
pub fn validate_and_apply_config_update(
    mut config: OrchestratorConfig,
    request: ConfigUpdateRequest,
) -> Result<OrchestratorConfig, AppError> {
    // Validate and apply max_parallel_tasks
    if let Some(max_parallel) = request.max_parallel_tasks {
        if max_parallel == 0 {
            return Err(AppError::Internal(anyhow::anyhow!(
                "max_parallel_tasks must be > 0"
            )));
        }
        config.max_parallel_tasks = max_parallel;
    }

    // Validate and apply gemini_model
    if let Some(model) = request.gemini_model {
        if model.is_empty() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "gemini_model cannot be empty"
            )));
        }
        config.gemini_model = model;
    }

    // Validate and apply max_goal_length
    if let Some(max_goal) = request.max_goal_length {
        if max_goal == 0 {
            return Err(AppError::Internal(anyhow::anyhow!(
                "max_goal_length must be > 0"
            )));
        }
        config.max_goal_length = max_goal;
    }

    // Validate and apply plan_timeout_secs
    if let Some(timeout) = request.plan_timeout_secs {
        if timeout == 0 {
            return Err(AppError::Internal(anyhow::anyhow!(
                "plan_timeout_secs must be > 0"
            )));
        }
        config.plan_timeout_secs = timeout;
    }

    Ok(config)
}
