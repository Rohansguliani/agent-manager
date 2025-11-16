//! Plan manipulation utilities
//!
//! Common utilities for working with Plans including validation, structure extraction,
//! and analysis helpers that can be shared across modules.

use crate::orchestrator::plan_types::Plan;
use std::collections::HashSet;

/// Extract task IDs from a plan
///
/// Returns a vector of all step IDs in the plan, maintaining order.
///
/// # Arguments
/// * `plan` - The plan to extract task IDs from
///
/// # Returns
/// * `Vec<String>` - Task IDs in order
pub fn extract_task_ids(plan: &Plan) -> Vec<String> {
    plan.steps.iter().map(|step| step.id.clone()).collect()
}

/// Extract edges from a plan
///
/// Returns a vector of (from, to) tuples representing dependencies.
/// Each dependency becomes an edge in the execution graph.
///
/// # Arguments
/// * `plan` - The plan to extract edges from
///
/// # Returns
/// * `Vec<(String, String)>` - Edges as (from_step_id, to_step_id) pairs
pub fn extract_edges(plan: &Plan) -> Vec<(String, String)> {
    let mut edges = Vec::new();
    for step in &plan.steps {
        for dep in &step.dependencies {
            edges.push((dep.clone(), step.id.clone()));
        }
    }
    edges
}

/// Find the first step with no dependencies
///
/// Returns the step ID of the first step that has no dependencies,
/// or the first step in the plan if all steps have dependencies.
///
/// # Arguments
/// * `plan` - The plan to analyze
///
/// # Returns
/// * `Option<&str>` - Step ID of the start step, or None if plan is empty
pub fn find_start_step_id(plan: &Plan) -> Option<&str> {
    plan.steps
        .iter()
        .find(|step| step.dependencies.is_empty())
        .map(|step| step.id.as_str())
        .or_else(|| plan.steps.first().map(|step| step.id.as_str()))
}

/// Get all step IDs that have no dependencies (can run in parallel at start)
///
/// # Arguments
/// * `plan` - The plan to analyze
///
/// # Returns
/// * `Vec<&str>` - Step IDs that have no dependencies
#[allow(dead_code)] // Reserved for future plan analysis features
pub fn find_independent_steps(plan: &Plan) -> Vec<&str> {
    plan.steps
        .iter()
        .filter(|step| step.dependencies.is_empty())
        .map(|step| step.id.as_str())
        .collect()
}

/// Get all step IDs that depend on a specific step
///
/// # Arguments
/// * `plan` - The plan to analyze
/// * `step_id` - The step ID to find dependents for
///
/// # Returns
/// * `Vec<String>` - Step IDs that depend on the given step
#[allow(dead_code)] // Reserved for future plan analysis features
pub fn find_dependents(plan: &Plan, step_id: &str) -> Vec<String> {
    plan.steps
        .iter()
        .filter(|step| step.dependencies.contains(&step_id.to_string()))
        .map(|step| step.id.clone())
        .collect()
}

/// Count the total number of dependencies across all steps
///
/// # Arguments
/// * `plan` - The plan to analyze
///
/// # Returns
/// * `usize` - Total number of dependency relationships
#[allow(dead_code)] // Reserved for future plan analysis features
pub fn count_total_dependencies(plan: &Plan) -> usize {
    plan.steps.iter().map(|step| step.dependencies.len()).sum()
}

/// Check if a plan has any steps
///
/// # Arguments
/// * `plan` - The plan to check
///
/// # Returns
/// * `bool` - True if plan has at least one step
#[allow(dead_code)] // Reserved for future plan analysis features
pub fn has_steps(plan: &Plan) -> bool {
    !plan.steps.is_empty()
}

/// Get unique set of all step IDs referenced in dependencies
///
/// This includes both step IDs that exist in the plan and any invalid references.
/// Useful for validation checks.
///
/// # Arguments
/// * `plan` - The plan to analyze
///
/// # Returns
/// * `HashSet<String>` - All unique step IDs referenced in dependencies
#[allow(dead_code)] // Reserved for future plan analysis features
pub fn get_all_referenced_step_ids(plan: &Plan) -> HashSet<String> {
    let mut referenced = HashSet::new();
    for step in &plan.steps {
        for dep in &step.dependencies {
            referenced.insert(dep.clone());
        }
    }
    referenced
}
