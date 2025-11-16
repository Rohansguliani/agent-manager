//! Orchestrator constants
//!
//! Centralized constants used throughout the orchestrator module.

/// SSE stream termination signal
pub const SSE_DONE_SIGNAL: &str = "[DONE]";

/// SSE error prefix
pub const SSE_ERROR_PREFIX: &str = "[ERROR]";

/// Default graph ID for plan execution
pub const DEFAULT_GRAPH_ID: &str = "plan_execution";

/// Suffix for step output keys in context
/// Format: "{step_id}{STEP_OUTPUT_SUFFIX}"
pub const STEP_OUTPUT_SUFFIX: &str = ".output";

/// Context key for working directory
pub const WORKING_DIR_KEY: &str = "working_dir";
