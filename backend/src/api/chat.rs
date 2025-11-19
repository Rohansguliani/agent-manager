//! Chat API endpoints
//!
//! Handles HTTP requests for chat conversations and messages.

use crate::api::utils::RouterState;
use crate::chat::Conversation;
use crate::error::AppError;
use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to create a new conversation
#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    /// Optional title (auto-generated from first message if not provided)
    pub title: Option<String>,
}

/// Request to send a message
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SendMessageRequest {
    /// Message content
    pub content: String,
}

/// Request to update conversation title
#[derive(Debug, Deserialize)]
pub struct UpdateTitleRequest {
    /// New title
    pub title: String,
}

/// Conversation response
#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    /// Conversation unique identifier
    pub id: String,
    /// Conversation title
    pub title: String,
    /// Unix timestamp when conversation was created
    pub created_at: i64,
    /// Unix timestamp when conversation was last updated
    pub updated_at: i64,
}

/// Message response
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    /// Message unique identifier
    pub id: String,
    /// ID of the conversation this message belongs to
    pub conversation_id: String,
    /// Message role ("user" or "assistant")
    pub role: String,
    /// Message content
    pub content: String,
    /// Unix timestamp when message was created
    pub created_at: i64,
}

/// Conversation with messages response
#[derive(Debug, Serialize)]
pub struct ConversationWithMessagesResponse {
    /// The conversation
    pub conversation: ConversationResponse,
    /// List of messages in the conversation
    pub messages: Vec<MessageResponse>,
}

/// GET /api/chat/conversations - List all conversations
pub async fn list_conversations(
    State((_, chat_db, _)): State<RouterState>,
) -> Result<Json<Vec<ConversationResponse>>, AppError> {
    let conversations = chat_db.get_conversations().await?;

    let responses: Vec<ConversationResponse> = conversations
        .into_iter()
        .map(|c| ConversationResponse {
            id: c.id,
            title: c.title,
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
        .collect();

    Ok(Json(responses))
}

/// Generate a title from message content
/// Truncates to first sentence or 50 characters, whichever comes first
pub fn generate_title_from_message(content: &str) -> String {
    let trimmed = content.trim();

    // Try to find first sentence (ending with . ! or ?)
    if let Some(sentence_end) = trimmed.find(['.', '!', '?']) {
        let sentence = &trimmed[..=sentence_end];
        if sentence.len() <= 60 {
            return sentence.trim().to_string();
        }
    }

    // Otherwise truncate to 50 characters
    if trimmed.len() > 50 {
        format!("{}...", &trimmed[..47])
    } else {
        trimmed.to_string()
    }
}

/// POST /api/chat/conversations - Create a new conversation
pub async fn create_conversation(
    State((_, chat_db, _)): State<RouterState>,
    Json(request): Json<CreateConversationRequest>,
) -> Result<Json<ConversationResponse>, AppError> {
    let id = Uuid::new_v4().to_string();
    let title = request.title.unwrap_or_else(|| "New Chat".to_string());

    let conversation = Conversation::new(id.clone(), title.clone());
    chat_db.create_conversation(&conversation).await?;

    Ok(Json(ConversationResponse {
        id: conversation.id,
        title: conversation.title,
        created_at: conversation.created_at,
        updated_at: conversation.updated_at,
    }))
}

/// GET /api/chat/conversations/:id - Get conversation with messages
pub async fn get_conversation(
    State((_, chat_db, _)): State<RouterState>,
    Path(id): Path<String>,
) -> Result<Json<ConversationWithMessagesResponse>, AppError> {
    let conversation = chat_db
        .get_conversation(&id)
        .await?
        .ok_or_else(|| AppError::FileNotFound(format!("Conversation not found: {}", id)))?;

    let messages = chat_db.get_messages(&id).await?;

    let conversation_response = ConversationResponse {
        id: conversation.id.clone(),
        title: conversation.title,
        created_at: conversation.created_at,
        updated_at: conversation.updated_at,
    };

    let message_responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(|m| MessageResponse {
            id: m.id,
            conversation_id: m.conversation_id,
            role: m.role,
            content: m.content,
            created_at: m.created_at,
        })
        .collect();

    Ok(Json(ConversationWithMessagesResponse {
        conversation: conversation_response,
        messages: message_responses,
    }))
}

/// DELETE /api/chat/conversations/:id - Delete a conversation
pub async fn delete_conversation(
    State((_, chat_db, bridge_manager)): State<RouterState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check if conversation exists
    chat_db
        .get_conversation(&id)
        .await?
        .ok_or_else(|| AppError::FileNotFound(format!("Conversation not found: {}", id)))?;

    // Kill process for this conversation before deleting
    if let Err(e) = bridge_manager.kill_process(&id).await {
        tracing::warn!(
            conversation_id = %id,
            error = %e,
            "Failed to kill process for conversation, continuing with deletion"
        );
        // Continue with deletion even if process kill fails
    }

    chat_db.delete_conversation(&id).await?;

    Ok(Json(serde_json::json!({
        "message": "Conversation deleted successfully",
        "id": id
    })))
}

/// PUT /api/chat/conversations/:id/title - Update conversation title
pub async fn update_conversation_title(
    State((_, chat_db, _)): State<RouterState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateTitleRequest>,
) -> Result<Json<ConversationResponse>, AppError> {
    // Validate title is not empty
    if request.title.trim().is_empty() {
        return Err(AppError::InvalidAgentConfig(
            "Title cannot be empty".to_string(),
        ));
    }

    // Check if conversation exists
    let conversation = chat_db
        .get_conversation(&id)
        .await?
        .ok_or_else(|| AppError::FileNotFound(format!("Conversation not found: {}", id)))?;

    chat_db.update_conversation(&id, &request.title).await?;

    Ok(Json(ConversationResponse {
        id: conversation.id,
        title: request.title,
        created_at: conversation.created_at,
        updated_at: chrono::Utc::now().timestamp(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::utils::RouterState;
    use crate::chat::{BridgeManager, ChatDb, Conversation, Message, MessageRole};
    use crate::state::AppState;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::RwLock;

    async fn create_test_router_state() -> (RouterState, TempDir) {
        let app_state = Arc::new(RwLock::new(AppState::new()));
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let chat_db = ChatDb::new(db_path.to_str().unwrap())
            .await
            .expect("Failed to create test database");
        let bridge_manager = Arc::new(BridgeManager::new());
        ((app_state, Arc::new(chat_db), bridge_manager), temp_dir)
    }

    #[tokio::test]
    async fn test_list_conversations_empty() {
        let (router_state, _temp_dir) = create_test_router_state().await;
        let result = list_conversations(State(router_state)).await;
        assert!(result.is_ok());
        let conversations = result.unwrap().0;
        assert!(conversations.is_empty());
    }

    #[tokio::test]
    async fn test_create_conversation() {
        let (router_state, _temp_dir) = create_test_router_state().await;
        let request = CreateConversationRequest {
            title: Some("Test Chat".to_string()),
        };
        let result = create_conversation(State(router_state), Json(request)).await;
        if let Err(e) = &result {
            eprintln!("Error creating conversation: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Failed to create conversation: {:?}",
            result
        );
        let conversation = result.unwrap().0;
        assert_eq!(conversation.title, "Test Chat");
        assert!(!conversation.id.is_empty());
    }

    #[tokio::test]
    async fn test_create_conversation_default_title() {
        let (router_state, _temp_dir) = create_test_router_state().await;
        let request = CreateConversationRequest { title: None };
        let result = create_conversation(State(router_state), Json(request)).await;
        assert!(result.is_ok());
        let conversation = result.unwrap().0;
        assert_eq!(conversation.title, "New Chat");
    }

    #[tokio::test]
    async fn test_get_conversation_not_found() {
        let (router_state, _temp_dir) = create_test_router_state().await;
        let result = get_conversation(State(router_state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::FileNotFound(_) => {}
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_conversation_with_messages() {
        let (router_state, _temp_dir) = create_test_router_state().await;
        let (_, chat_db, _) = &router_state;

        // Create conversation
        let conv = Conversation::new(Uuid::new_v4().to_string(), "Test".to_string());
        chat_db.create_conversation(&conv).await.unwrap();

        // Add messages
        let msg1 = Message::new(
            Uuid::new_v4().to_string(),
            conv.id.clone(),
            MessageRole::User,
            "Hello".to_string(),
        );
        let msg2 = Message::new(
            Uuid::new_v4().to_string(),
            conv.id.clone(),
            MessageRole::Assistant,
            "Hi there!".to_string(),
        );
        chat_db.add_message(&msg1).await.unwrap();
        chat_db.add_message(&msg2).await.unwrap();

        // Get conversation
        let result = get_conversation(State(router_state.clone()), Path(conv.id.clone())).await;
        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.conversation.id, conv.id);
        assert_eq!(response.messages.len(), 2);
        assert_eq!(response.messages[0].content, "Hello");
        assert_eq!(response.messages[1].content, "Hi there!");
    }

    #[tokio::test]
    async fn test_delete_conversation() {
        let (router_state, _temp_dir) = create_test_router_state().await;
        let (_, chat_db, _) = &router_state;

        // Create conversation
        let conv = Conversation::new(Uuid::new_v4().to_string(), "Test".to_string());
        chat_db.create_conversation(&conv).await.unwrap();

        // Delete it
        let result = delete_conversation(State(router_state.clone()), Path(conv.id.clone())).await;
        assert!(result.is_ok());

        // Verify it's gone
        let result = get_conversation(State(router_state), Path(conv.id)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_conversation_not_found() {
        let (router_state, _temp_dir) = create_test_router_state().await;
        let result =
            delete_conversation(State(router_state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_conversation_title() {
        let (router_state, _temp_dir) = create_test_router_state().await;
        let (_, chat_db, _) = &router_state;

        // Create conversation
        let conv = Conversation::new(Uuid::new_v4().to_string(), "Old Title".to_string());
        chat_db.create_conversation(&conv).await.unwrap();

        // Update title
        let request = UpdateTitleRequest {
            title: "New Title".to_string(),
        };
        let result = update_conversation_title(
            State(router_state.clone()),
            Path(conv.id.clone()),
            Json(request),
        )
        .await;
        assert!(result.is_ok());
        let updated = result.unwrap().0;
        assert_eq!(updated.title, "New Title");

        // Verify in database
        let conv_from_db = chat_db.get_conversation(&conv.id).await.unwrap().unwrap();
        assert_eq!(conv_from_db.title, "New Title");
    }

    #[tokio::test]
    async fn test_update_conversation_title_empty() {
        let (router_state, _temp_dir) = create_test_router_state().await;
        let (_, chat_db, _) = &router_state;
        let conv = Conversation::new(Uuid::new_v4().to_string(), "Test".to_string());
        chat_db.create_conversation(&conv).await.unwrap();

        let request = UpdateTitleRequest {
            title: "   ".to_string(),
        };
        let result =
            update_conversation_title(State(router_state), Path(conv.id), Json(request)).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidAgentConfig(_) => {}
            _ => panic!("Expected InvalidAgentConfig error"),
        }
    }

    #[tokio::test]
    async fn test_update_conversation_title_not_found() {
        let (router_state, _temp_dir) = create_test_router_state().await;
        let request = UpdateTitleRequest {
            title: "New Title".to_string(),
        };
        let result = update_conversation_title(
            State(router_state),
            Path("nonexistent".to_string()),
            Json(request),
        )
        .await;
        assert!(result.is_err());
    }
}
