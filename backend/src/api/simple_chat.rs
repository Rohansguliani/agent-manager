//! Simple Chat API
//!
//! Provides a straightforward chat endpoint using the bridge architecture.
//! Flow: user message -> backend -> Node.js bridge process -> GeminiChat (with context) -> response
//!
//! Context Management:
//! - Each conversation gets a persistent Node.js bridge process that maintains conversation state
//! - GeminiChat (from @google/gemini-cli-core) manages conversation history internally
//! - Messages are also persisted to SQLite for UI display and cross-restart recovery
//! - No manual history formatting needed - the bridge handles context automatically

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::api::utils::RouterState;
use crate::chat::models::{Conversation, Message, MessageRole};

#[allow(missing_docs)]
#[derive(Deserialize)]
pub struct SimpleChatRequest {
    pub message: String,
    /// Optional conversation ID for maintaining context
    /// If provided, the same bridge process (with conversation history) will be used
    #[serde(default)]
    pub conversation_id: Option<String>,
    /// Optional list of image filenames (for multipart requests)
    #[serde(default)]
    pub image_filenames: Option<Vec<String>>,
    /// Optional model name (e.g., "gemini-2.5-flash", "gemini-2.5-pro")
    /// If not provided, uses default from bridge config
    #[serde(default)]
    pub model: Option<String>,
}

#[allow(missing_docs)]
#[derive(Serialize)]
pub struct SimpleChatResponse {
    pub response: String,
    pub success: bool,
    /// The conversation ID (same as input or newly generated)
    pub conversation_id: String,
}

/// Internal function that handles the actual chat logic
/// This is shared between JSON and multipart endpoints
pub async fn simple_chat_internal(
    message: String,
    conversation_id: Option<String>,
    image_filenames: Option<Vec<String>>,
    model: Option<String>,
    chat_db: &crate::chat::ChatDb,
    bridge_manager: &crate::chat::BridgeManager,
) -> Result<Json<SimpleChatResponse>, StatusCode> {
    // Generate or use provided conversation_id
    let conversation_id = conversation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Ensure conversation exists in database
    let conversation_exists = chat_db
        .get_conversation(&conversation_id)
        .await
        .map_err(|e| {
            error!("Failed to check conversation existence: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if conversation_exists.is_none() {
        // Create new conversation
        let title = if message.len() > 50 {
            format!("{}...", &message[..47])
        } else {
            message.clone()
        };
        let conversation = Conversation::new(conversation_id.clone(), title);
        chat_db
            .create_conversation(&conversation)
            .await
            .map_err(|e| {
                error!("Failed to create conversation: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    info!(
        "Simple chat request received (conversation_id: {}): {}",
        conversation_id, message
    );

    // Validate message is not empty
    if message.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Note: Images are not yet supported in bridge approach (will be added later)
    if image_filenames.is_some() {
        warn!("Image support not yet implemented in bridge approach, ignoring images");
    }

    // Send message to bridge process
    // The bridge process maintains conversation state internally via GeminiChat
    // No need to format conversation history - GeminiChat handles it
    let model_name = model.as_deref();
    let response_text = bridge_manager
        .send_message(&conversation_id, &message, model_name)
        .await
        .map_err(|e| {
            error!(
                conversation_id = %conversation_id,
                error = %e,
                "Failed to send message to bridge"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(
        conversation_id = %conversation_id,
        response_len = response_text.len(),
        "Bridge response received"
    );

    // Store messages in database for persistence across restarts
    let user_message = Message::new(
        uuid::Uuid::new_v4().to_string(),
        conversation_id.clone(),
        MessageRole::User,
        message.clone(),
    );
    chat_db.add_message(&user_message).await.map_err(|e| {
        error!("Failed to save user message: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let assistant_message = Message::new(
        uuid::Uuid::new_v4().to_string(),
        conversation_id.clone(),
        MessageRole::Assistant,
        response_text.clone(),
    );
    chat_db.add_message(&assistant_message).await.map_err(|e| {
        error!("Failed to save assistant message: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(SimpleChatResponse {
        response: response_text,
        success: true,
        conversation_id,
    }))
}

/// Simple chat endpoint using the bridge architecture (JSON version)
///
/// This endpoint:
/// 1. Receives a message from the frontend (optionally with conversation_id)
/// 2. Gets or creates a persistent bridge process for the conversation
/// 3. Sends the message to the bridge (which maintains context via GeminiChat)
/// 4. Stores the message pair in SQLite (for UI display and persistence)
/// 5. Returns the response to the frontend
pub async fn simple_chat(
    State((_, chat_db, bridge_manager)): State<RouterState>,
    Json(request): Json<SimpleChatRequest>,
) -> Result<Json<SimpleChatResponse>, StatusCode> {
    simple_chat_internal(
        request.message,
        request.conversation_id,
        request.image_filenames,
        request.model,
        &chat_db,
        &bridge_manager,
    )
    .await
}
