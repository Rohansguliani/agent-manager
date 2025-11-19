//! Bridge Manager
//!
//! Manages persistent Node.js bridge processes for each conversation.
//! Uses the sidecar architecture: one Node.js process per conversation that
//! uses @google/gemini-cli-core SDK directly instead of wrapping the CLI.

use super::bridge_session::BridgeSession;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Manages persistent bridge processes for conversations
///
/// One BridgeSession per conversation ID. Sessions are created on demand
/// and persist for the lifetime of the conversation.
pub struct BridgeManager {
    /// Map from conversation_id to BridgeSession
    sessions: Arc<RwLock<HashMap<String, Arc<BridgeSession>>>>,
    /// Path to the bridge script (stored for new session creation)
    #[allow(dead_code)]
    bridge_script_path: PathBuf,
}

impl BridgeManager {
    /// Create a new bridge manager
    pub fn new() -> Self {
        let bridge_script_path = BridgeSession::get_bridge_script_path();
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            bridge_script_path,
        }
    }

    /// Get or create a bridge session for a conversation
    ///
    /// # Arguments
    /// * `conversation_id` - ID of the conversation
    ///
    /// # Returns
    /// * `Result<Arc<BridgeSession>, String>` - Existing or new session
    pub async fn get_or_create_session(
        &self,
        conversation_id: &str,
    ) -> Result<Arc<BridgeSession>, String> {
        // Check if session already exists
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(conversation_id) {
                // Check if process is still running
                if session.is_running().await {
                    debug!(
                        conversation_id = %conversation_id,
                        "Reusing existing bridge session"
                    );
                    return Ok(session.clone());
                } else {
                    warn!(
                        conversation_id = %conversation_id,
                        "Existing session process has died, removing and creating new one"
                    );
                    // Remove dead session from map
                    drop(sessions);
                    let mut sessions = self.sessions.write().await;
                    sessions.remove(conversation_id);
                }
            }
        }

        // Create new session
        debug!(
            conversation_id = %conversation_id,
            "Creating new bridge session"
        );

        let session = Arc::new(
            BridgeSession::new(conversation_id.to_string(), self.bridge_script_path.clone())
                .await
                .map_err(|e| {
                    error!(
                        conversation_id = %conversation_id,
                        error = %e,
                        "Failed to create bridge session"
                    );
                    e
                })?,
        );

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(conversation_id.to_string(), session.clone());
        }

        info!(
            conversation_id = %conversation_id,
            "Bridge session created and stored"
        );

        Ok(session)
    }

    /// Send a message to a conversation's bridge session
    ///
    /// # Arguments
    /// * `conversation_id` - ID of the conversation
    /// * `content` - Message content
    /// * `model` - Optional model to use
    ///
    /// # Returns
    /// * `Result<String, String>` - Response text or error
    pub async fn send_message(
        &self,
        conversation_id: &str,
        content: &str,
        model: Option<&str>,
    ) -> Result<String, String> {
        let session = self.get_or_create_session(conversation_id).await?;
        session.send_message(content, model).await
    }

    /// Kill a process for a conversation
    ///
    /// # Arguments
    /// * `conversation_id` - ID of the conversation
    ///
    /// # Returns
    /// * `Result<(), String>` - Success or error
    pub async fn kill_process(&self, conversation_id: &str) -> Result<(), String> {
        debug!(
            conversation_id = %conversation_id,
            "Killing bridge process for conversation"
        );

        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.remove(conversation_id) {
            session.kill().await.map_err(|e| {
                error!(
                    conversation_id = %conversation_id,
                    error = %e,
                    "Failed to kill bridge process"
                );
                e
            })?;

            info!(
                conversation_id = %conversation_id,
                "Bridge process killed and removed"
            );
        } else {
            debug!(
                conversation_id = %conversation_id,
                "No bridge process found for conversation"
            );
        }

        Ok(())
    }

    /// Kill all processes (for graceful shutdown)
    pub async fn kill_all_processes(&self) {
        info!("Killing all bridge processes");

        let mut sessions = self.sessions.write().await;
        let conversation_ids: Vec<String> = sessions.keys().cloned().collect();

        for conversation_id in conversation_ids {
            if let Some(session) = sessions.remove(&conversation_id) {
                if let Err(e) = session.kill().await {
                    error!(
                        conversation_id = %conversation_id,
                        error = %e,
                        "Failed to kill bridge process during shutdown"
                    );
                }
            }
        }

        info!("All bridge processes killed");
    }

    /// Get the number of active sessions
    #[allow(dead_code)] // Will be used in Phase 4 for metrics/monitoring
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

impl Default for BridgeManager {
    fn default() -> Self {
        Self::new()
    }
}
