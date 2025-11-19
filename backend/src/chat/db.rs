//! Chat database operations
//!
//! Handles all database interactions for conversations and messages.

use crate::chat::models::{Conversation, Message};
use crate::error::AppError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::{debug, info};

/// Database connection pool for chat operations
pub struct ChatDb {
    pool: SqlitePool,
}

impl ChatDb {
    /// Initialize database connection pool
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file
    ///
    /// # Returns
    /// * `Ok(ChatDb)` if successful
    /// * `Err(AppError)` if connection failed
    pub async fn new(db_path: &str) -> Result<Self, AppError> {
        // Ensure parent directory exists
        if let Some(parent) = PathBuf::from(db_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to create db directory: {}", e))
            })?;
        }

        // SQLite connection string format: sqlite://path/to/db.db
        let connection_string = if db_path.starts_with("sqlite:") {
            db_path.to_string()
        } else {
            format!("sqlite:{}", db_path)
        };

        let options = SqliteConnectOptions::from_str(&connection_string)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid database path: {}", e)))?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to connect to database: {}", e))
            })?;

        info!("Connected to SQLite database at: {}", db_path);

        let db = Self { pool };
        db.run_migrations().await?;

        Ok(db)
    }

    /// Run database migrations
    async fn run_migrations(&self) -> Result<(), AppError> {
        info!("Running database migrations...");

        // Read migration file
        let migration_sql = include_str!("../../migrations/001_create_chats.sql");

        // Remove comments (lines starting with --) and normalize whitespace
        let mut cleaned_sql = String::new();
        for line in migration_sql.lines() {
            let trimmed = line.trim();
            // Skip empty lines and comment-only lines
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }
            // Remove inline comments (everything after --)
            let without_comments = if let Some(comment_pos) = trimmed.find("--") {
                &trimmed[..comment_pos]
            } else {
                trimmed
            };
            cleaned_sql.push_str(without_comments.trim());
            cleaned_sql.push(' ');
        }

        // Split by semicolon and filter out empty statements
        let statements: Vec<&str> = cleaned_sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        // Execute each statement separately
        for statement in statements {
            sqlx::query(statement)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    AppError::Internal(anyhow::anyhow!(
                        "Migration failed: {} - Statement: {}",
                        e,
                        statement.chars().take(100).collect::<String>()
                    ))
                })?;
        }

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Get all conversations, ordered by most recently updated
    pub async fn get_conversations(&self) -> Result<Vec<Conversation>, AppError> {
        let conversations = sqlx::query_as::<_, Conversation>(
            "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to fetch conversations: {}", e)))?;

        Ok(conversations)
    }

    /// Get a conversation by ID
    pub async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>, AppError> {
        let conversation = sqlx::query_as::<_, Conversation>(
            "SELECT id, title, created_at, updated_at FROM conversations WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to fetch conversation: {}", e)))?;

        Ok(conversation)
    }

    /// Create a new conversation
    pub async fn create_conversation(&self, conversation: &Conversation) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&conversation.id)
        .bind(&conversation.title)
        .bind(conversation.created_at)
        .bind(conversation.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create conversation: {}", e)))?;

        debug!("Created conversation: {}", conversation.id);
        Ok(())
    }

    /// Update conversation title and updated_at timestamp
    pub async fn update_conversation(&self, id: &str, title: &str) -> Result<(), AppError> {
        let updated_at = chrono::Utc::now().timestamp();
        sqlx::query("UPDATE conversations SET title = ?, updated_at = ? WHERE id = ?")
            .bind(title)
            .bind(updated_at)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to update conversation: {}", e))
            })?;

        debug!("Updated conversation: {}", id);
        Ok(())
    }

    /// Update conversation's updated_at timestamp (when new message is added)
    pub async fn touch_conversation(&self, id: &str) -> Result<(), AppError> {
        let updated_at = chrono::Utc::now().timestamp();
        sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
            .bind(updated_at)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to touch conversation: {}", e))
            })?;

        Ok(())
    }

    /// Delete a conversation (cascades to messages)
    pub async fn delete_conversation(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM conversations WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to delete conversation: {}", e))
            })?;

        debug!("Deleted conversation: {}", id);
        Ok(())
    }

    /// Get all messages for a conversation, ordered by creation time
    pub async fn get_messages(&self, conversation_id: &str) -> Result<Vec<Message>, AppError> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT id, conversation_id, role, content, created_at FROM messages WHERE conversation_id = ? ORDER BY created_at ASC"
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to fetch messages: {}", e)))?;

        Ok(messages)
    }

    /// Add a message to a conversation
    pub async fn add_message(&self, message: &Message) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&message.id)
        .bind(&message.conversation_id)
        .bind(&message.role)
        .bind(&message.content)
        .bind(message.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to add message: {}", e)))?;

        // Update conversation's updated_at timestamp
        self.touch_conversation(&message.conversation_id).await?;

        debug!(
            "Added message {} to conversation {}",
            message.id, message.conversation_id
        );
        Ok(())
    }

    /// Get the database pool (for advanced operations if needed)
    #[allow(dead_code)]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
