//! Plan optimization utilities
//!
//! This module provides plan optimization functions such as:
//! - Merging compatible steps
//! - Identifying bottlenecks
//! - Cost estimation (token usage prediction)

use crate::orchestrator::plan_types::Plan;
use serde::Serialize;
use std::collections::HashMap;

/// Estimate token usage for a plan
///
/// This provides a rough estimate based on task types and prompt lengths.
/// Used for cost estimation and planning optimization.
#[allow(dead_code)] // Will be used when implementing cost estimation endpoints
pub fn estimate_token_usage(plan: &Plan) -> usize {
    let mut total_tokens = 0;

    for step in &plan.steps {
        match step.task.as_str() {
            "run_gemini" => {
                // Rough estimate: 1.3 tokens per character for prompts
                // Plus ~100 tokens for API overhead
                if let Some(ref prompt) = step.params.prompt {
                    total_tokens += (prompt.len() as f64 * 1.3) as usize + 100;
                }
            }
            "create_file" => {
                // File creation has minimal token cost (just task description)
                total_tokens += 50;
            }
            _ => {
                // Unknown task type - conservative estimate
                total_tokens += 100;
            }
        }
    }

    total_tokens
}

/// Get execution time estimate for a plan
///
/// Returns estimated time in seconds based on task types.
/// This is a rough estimate for planning purposes.
#[allow(dead_code)] // Will be used when implementing time estimation endpoints
pub fn estimate_execution_time(plan: &Plan) -> usize {
    let mut total_seconds = 0;

    for step in &plan.steps {
        match step.task.as_str() {
            "run_gemini" => {
                // Gemini API calls typically take 1-5 seconds
                // Use conservative estimate of 3 seconds per call
                total_seconds += 3;
            }
            "create_file" => {
                // File operations are fast (< 1 second)
                total_seconds += 1;
            }
            _ => {
                // Unknown task type - conservative estimate
                total_seconds += 2;
            }
        }
    }

    total_seconds
}

/// Analyze plan for bottlenecks
///
/// Returns information about potential bottlenecks in the plan,
/// such as long sequential chains or steps with many dependencies.
#[derive(Debug, Clone, Serialize)]
pub struct BottleneckAnalysis {
    /// Steps that have many dependencies (potential bottlenecks)
    pub high_dependency_steps: Vec<String>,
    /// Longest sequential chain length
    pub longest_chain_length: usize,
    /// Total number of independent steps (can run in parallel)
    pub independent_steps: usize,
}

/// Analyze plan bottlenecks and execution characteristics
///
/// Identifies steps with high dependency counts, calculates longest sequential chain,
/// and counts independent steps that can run in parallel.
///
/// # Arguments
/// * `plan` - The plan to analyze
///
/// # Returns
/// * `BottleneckAnalysis` - Analysis results with high-dependency steps, chain length, and parallelization info
#[allow(dead_code)] // Will be used when implementing optimization endpoints
pub fn analyze_bottlenecks(plan: &Plan) -> BottleneckAnalysis {
    let mut high_dependency_steps = Vec::new();
    let mut independent_steps = 0;

    // Find steps with many dependencies
    for step in &plan.steps {
        if step.dependencies.len() >= 3 {
            high_dependency_steps.push(step.id.clone());
        }
        if step.dependencies.is_empty() {
            independent_steps += 1;
        }
    }

    // Calculate longest chain using memoized depth calculation (O(n) instead of O(n²))
    // Build step lookup map for efficient access
    let step_map: HashMap<&str, &crate::orchestrator::plan_types::Step> = plan
        .steps
        .iter()
        .map(|step| (step.id.as_str(), step))
        .collect();

    // Memoized depth calculation (use String keys to avoid lifetime issues)
    let mut depth_cache: HashMap<String, usize> = HashMap::new();
    let mut max_depth = 0;
    for step in &plan.steps {
        let depth = calculate_step_depth_memoized(&step.id, &step_map, &mut depth_cache);
        if depth > max_depth {
            max_depth = depth;
        }
    }
    let longest_chain_length = max_depth;

    BottleneckAnalysis {
        high_dependency_steps,
        longest_chain_length,
        independent_steps,
    }
}

/// Calculate the depth of a step in the dependency graph using memoization
///
/// This function uses a cache to avoid recalculating depths for the same steps,
/// reducing complexity from O(n²) to O(n) for the bottleneck analysis.
///
/// # Arguments
/// * `step_id` - The ID of the step to calculate depth for (owned String)
/// * `step_map` - HashMap for O(1) step lookup
/// * `depth_cache` - Mutable cache to store calculated depths (String keys to avoid lifetime issues)
///
/// # Returns
/// * `usize` - The depth of the step (1 for steps with no dependencies)
fn calculate_step_depth_memoized(
    step_id: &str,
    step_map: &HashMap<&str, &crate::orchestrator::plan_types::Step>,
    depth_cache: &mut HashMap<String, usize>,
) -> usize {
    // Check cache first (use String key)
    if let Some(&depth) = depth_cache.get(step_id) {
        return depth;
    }

    // Get step from map
    let step = match step_map.get(step_id) {
        Some(s) => s,
        None => {
            // Step not found (shouldn't happen after validation, but handle gracefully)
            tracing::warn!(
                "Step '{}' not found in step_map during depth calculation",
                step_id
            );
            return 1;
        }
    };

    // Base case: step with no dependencies has depth 1
    if step.dependencies.is_empty() {
        depth_cache.insert(step_id.to_string(), 1);
        return 1;
    }

    // Recursive case: find max depth of dependencies
    let mut max_dependency_depth = 0;
    for dep_id in &step.dependencies {
        let dep_depth = calculate_step_depth_memoized(dep_id, step_map, depth_cache);
        if dep_depth > max_dependency_depth {
            max_dependency_depth = dep_depth;
        }
    }

    let depth = max_dependency_depth + 1;
    depth_cache.insert(step_id.to_string(), depth);
    depth
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::plan_types::{Plan, Step, StepParams};

    #[test]
    fn test_estimate_token_usage() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Test prompt with 30 chars".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams::default(),
                    dependencies: vec!["step_1".to_string()],
                },
            ],
        };

        let tokens = estimate_token_usage(&plan);
        assert!(tokens > 0);
        // Should include tokens for both steps
        assert!(tokens > 150); // At least prompt tokens + overhead
    }

    #[test]
    fn test_estimate_execution_time() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Test".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams::default(),
                    dependencies: vec!["step_1".to_string()],
                },
            ],
        };

        let time = estimate_execution_time(&plan);
        assert_eq!(time, 4); // 3 seconds for Gemini + 1 second for file
    }

    #[test]
    fn test_analyze_bottlenecks() {
        let plan = Plan {
            version: "1.0".to_string(),
            steps: vec![
                Step {
                    id: "step_1".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Test 1".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_2".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Test 2".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_3".to_string(),
                    task: "run_gemini".to_string(),
                    params: StepParams {
                        prompt: Some("Test 3".to_string()),
                        ..Default::default()
                    },
                    dependencies: vec![],
                },
                Step {
                    id: "step_4".to_string(),
                    task: "create_file".to_string(),
                    params: StepParams::default(),
                    dependencies: vec![
                        "step_1".to_string(),
                        "step_2".to_string(),
                        "step_3".to_string(),
                    ],
                },
            ],
        };

        let analysis = analyze_bottlenecks(&plan);
        assert_eq!(analysis.independent_steps, 3); // step_1, step_2, step_3
        assert!(analysis
            .high_dependency_steps
            .contains(&"step_4".to_string()));
    }
}
