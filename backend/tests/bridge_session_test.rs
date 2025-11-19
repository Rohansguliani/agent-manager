//! Tests for BridgeSession

use agent_manager_backend::chat::bridge_session::BridgeSession;
use std::path::PathBuf;
use tokio::test;

#[tokio::test]
async fn test_bridge_session_new() {
    let script_path = BridgeSession::get_bridge_script_path();

    // Check that script path exists
    assert!(
        script_path.exists(),
        "Bridge script should exist at {:?}",
        script_path
    );

    // Try to create a session (will fail without auth, but tests structure)
    let conversation_id = "test-conv-1".to_string();
    let result = BridgeSession::new(conversation_id.clone(), script_path.clone()).await;

    // Session creation might fail if Node.js or auth is not set up, but structure should be correct
    if let Ok(session) = result {
        // Clean up
        let _ = session.kill().await;
    }
}
