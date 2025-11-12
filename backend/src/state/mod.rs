// State management module
// Handles application state, agent registry, and persistence

pub mod app_state;
pub mod config;
pub mod persistence;

pub use app_state::{Agent, AgentId, AgentStatus, AppState};
pub use config::{AgentConfig, AgentType};
pub use persistence::PersistenceError;
