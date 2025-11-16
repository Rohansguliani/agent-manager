//! GraphFlow-rs adapter module
//!
//! This module provides adapters to bridge between our PlanTask trait
//! and GraphFlow-rs's Task trait. This allows us to use GraphFlow-rs
//! for parallel DAG execution while keeping our primitives decoupled.

use crate::orchestrator::tasks::ExecutionContext;
use crate::state::AppState;
#[allow(unused_imports)] // Will be used when fully implementing GraphFlow-rs integration
use async_trait::async_trait;
#[allow(unused_imports)] // Will be used when fully implementing GraphFlow-rs integration
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// GraphFlow-rs imports - using dynamic types to avoid tight coupling
// We'll handle context conversion manually

/// Shared state container for GraphFlow execution
///
/// This struct holds both our ExecutionContext and AppState,
/// allowing GraphFlow tasks to access them through the context.
///
/// Phase 4D: Currently unused, reserved for future GraphFlow-rs integration
#[allow(dead_code)] // Reserved for Phase 4D+ GraphFlow-rs integration
#[derive(Clone)]
pub struct GraphFlowState {
    /// Our execution context (step outputs, working directory)
    pub execution_context: Arc<RwLock<ExecutionContext>>,
    /// Application state (for agent management)
    pub app_state: Arc<RwLock<AppState>>,
}

impl GraphFlowState {
    /// Create a new GraphFlow state container
    #[allow(dead_code)] // Reserved for Phase 4D+ GraphFlow-rs integration
    pub fn new(execution_context: ExecutionContext, app_state: Arc<RwLock<AppState>>) -> Self {
        Self {
            execution_context: Arc::new(RwLock::new(execution_context)),
            app_state,
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO(Phase 4D): Add tests once GraphFlow-rs integration is complete
    // Tests should verify:
    // - Context conversion works correctly
    // - Task execution preserves state
    // - Parallel execution works as expected
}
