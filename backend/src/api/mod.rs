//! API module
//!
//! Contains HTTP request handlers for agent management endpoints

pub mod agents;
pub mod files;
pub mod orchestrator;
pub mod orchestrator_graph;
pub mod queries;
pub mod streaming;
pub mod utils;

// Re-export file API for convenience (used by main.rs)
pub use files::*;
