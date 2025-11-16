//! Orchestrator utility functions
//!
//! Common utilities for orchestrator operations including hashing, validation, and helpers.

use crate::orchestrator::plan_types::Plan;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Compute a short hash for a goal string
///
/// Returns an 8-character hexadecimal hash suitable for logging and tracing.
///
/// # Arguments
/// * `goal` - The goal string to hash
///
/// # Returns
/// * `String` - 8-character hexadecimal hash
pub fn hash_goal(goal: &str) -> String {
    let mut hasher = DefaultHasher::new();
    goal.hash(&mut hasher);
    format!("{:x}", hasher.finish())[..8].to_string()
}

/// Compute a short hash for a plan
///
/// Returns an 8-character hexadecimal hash based on the plan's step count and step IDs.
/// Used for plan identification in logging and tracing.
///
/// # Arguments
/// * `plan` - The plan to hash
///
/// # Returns
/// * `String` - 8-character hexadecimal hash
pub fn hash_plan(plan: &Plan) -> String {
    let mut hasher = DefaultHasher::new();
    plan.steps.len().hash(&mut hasher);
    for step in &plan.steps {
        step.id.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())[..8].to_string()
}
