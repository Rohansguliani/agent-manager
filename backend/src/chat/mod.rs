//! Chat module
//!
//! Handles chat conversations and messages storage using SQLite database.

pub mod bridge_manager;
pub mod bridge_session;
pub mod db;
pub mod models;

pub use bridge_manager::BridgeManager;
#[allow(unused_imports)] // Will be used in Phase 4 for metrics/monitoring
pub use bridge_session::BridgeSession;
pub use db::ChatDb;
pub use models::{Conversation, Message, MessageRole};
