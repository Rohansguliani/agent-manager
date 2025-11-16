//! Agent Manager Backend Library
//!
//! This library exposes modules for testing and external use.
//! The main binary is in `src/main.rs`.

pub mod api;
pub mod config;
pub mod error;
pub mod executor;
pub mod orchestrator;
pub mod services;
/// Application state management
///
/// Handles agent registry, working directory context, and persistence.
pub mod state;
pub mod websocket;
