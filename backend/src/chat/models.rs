//! Chat data models
//!
//! Defines structures for conversations and messages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Role of a message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Message from the user
    User,
    /// Message from the assistant/AI
    Assistant,
}

impl MessageRole {
    /// Convert the role to its string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        }
    }
}

impl From<&str> for MessageRole {
    fn from(s: &str) -> Self {
        match s {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            _ => MessageRole::User,
        }
    }
}

/// A conversation thread
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    /// Unique identifier for the conversation
    pub id: String,
    /// Title of the conversation (auto-generated from first message or user-set)
    pub title: String,
    /// When the conversation was created (Unix timestamp)
    pub created_at: i64,
    /// When the conversation was last updated (Unix timestamp)
    pub updated_at: i64,
}

impl Conversation {
    /// Create a new conversation
    pub fn new(id: String, title: String) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id,
            title,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get created_at as DateTime
    #[allow(dead_code)]
    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_at, 0).unwrap_or_else(Utc::now)
    }

    /// Get updated_at as DateTime
    #[allow(dead_code)]
    pub fn updated_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.updated_at, 0).unwrap_or_else(Utc::now)
    }
}

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    /// Unique identifier for the message
    pub id: String,
    /// ID of the conversation this message belongs to
    pub conversation_id: String,
    /// Role of the message sender
    pub role: String, // Stored as "user" or "assistant" in DB
    /// Content of the message
    pub content: String,
    /// When the message was created (Unix timestamp)
    pub created_at: i64,
}

impl Message {
    /// Create a new message
    pub fn new(id: String, conversation_id: String, role: MessageRole, content: String) -> Self {
        Self {
            id,
            conversation_id,
            role: role.as_str().to_string(),
            content,
            created_at: Utc::now().timestamp(),
        }
    }

    /// Get the message role as enum
    #[allow(dead_code)]
    pub fn role_enum(&self) -> MessageRole {
        MessageRole::from(self.role.as_str())
    }

    /// Get created_at as DateTime
    #[allow(dead_code)]
    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_at, 0).unwrap_or_else(Utc::now)
    }
}
