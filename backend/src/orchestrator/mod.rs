//! Orchestrator module
//!
//! Contains reusable primitives for orchestrating multi-step operations.
//! This module provides building blocks that can be composed to create
//! orchestration workflows.
//!
//! The primitives are intentionally designed to be reusable and testable,
//! making it easy to refactor to a generic orchestrator in future versions.

pub mod api_client;
pub mod config;
pub mod gemini_types;
pub mod graph_executor;
// TODO(Phase 4D): Implement full GraphFlow-rs integration
pub mod graph_flow_adapter;
pub mod plan_to_graph;
pub mod plan_types;
pub mod primitives;
pub mod tasks;
