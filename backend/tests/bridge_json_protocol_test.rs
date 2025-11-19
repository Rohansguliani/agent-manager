//! Tests for JSON protocol in BridgeSession

use agent_manager_backend::chat::bridge_session::{BridgeRequest, BridgeResponse};
use serde_json;

#[test]
fn test_bridge_request_serialization() {
    // Test serialization of bridge request
    let request = BridgeRequest {
        request_type: "message".to_string(),
        content: Some("Hello, world!".to_string()),
        model: Some("gemini-2.5-flash".to_string()),
    };

    let json = serde_json::to_string(&request).unwrap();

    // Verify JSON structure
    assert!(json.contains(r#""type":"message""#));
    assert!(json.contains(r#""content":"Hello, world!""#));
    assert!(json.contains(r#""model":"gemini-2.5-flash""#));
}

#[test]
fn test_bridge_response_deserialization_success() {
    // Test deserialization of success response
    let json = r#"{"status":"success","data":"Hello, assistant!"}"#;
    let response: BridgeResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.status, "success");
    assert_eq!(response.data, Some("Hello, assistant!".to_string()));
    assert_eq!(response.message, None);
}

#[test]
fn test_bridge_response_deserialization_error() {
    // Test deserialization of error response
    let json = r#"{"status":"error","message":"Invalid request"}"#;
    let response: BridgeResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.status, "error");
    assert_eq!(response.message, Some("Invalid request".to_string()));
    assert_eq!(response.data, None);
}

#[test]
fn test_bridge_request_without_model() {
    // Test request without optional model field
    let request = BridgeRequest {
        request_type: "message".to_string(),
        content: Some("Test message".to_string()),
        model: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Model should be null or absent
    assert!(parsed["model"].is_null() || !parsed.as_object().unwrap().contains_key("model"));
}
