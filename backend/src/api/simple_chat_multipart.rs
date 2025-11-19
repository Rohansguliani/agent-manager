//! Simple Chat API with Multipart Support
//!
//! Handles image uploads and passes them to Gemini CLI using @filename syntax.
//! This is a separate endpoint that accepts multipart form data.

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{error, info, warn};

use super::simple_chat::simple_chat_internal;
use crate::api::simple_chat::SimpleChatResponse;
use crate::api::utils::RouterState;

/// Temporary directory for uploaded images
const TEMP_IMAGE_DIR: &str = "/tmp/gemini-chat-images";

/// Ensure temp directory exists
async fn ensure_temp_dir() -> Result<PathBuf, StatusCode> {
    let temp_dir = PathBuf::from(TEMP_IMAGE_DIR);
    if let Err(e) = fs::create_dir_all(&temp_dir).await {
        error!("Failed to create temp directory: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok(temp_dir)
}

/// Clean up temporary image files
async fn cleanup_images(filenames: &[String]) {
    for filename in filenames {
        let path = PathBuf::from(TEMP_IMAGE_DIR).join(filename);
        if let Err(e) = fs::remove_file(&path).await {
            warn!("Failed to cleanup temp image {}: {}", filename, e);
        }
    }
}

/// Simple chat endpoint with multipart support for image uploads
///
/// Accepts multipart form data with:
/// - message: text message
/// - conversation_id: optional conversation ID
/// - images: one or more image files
pub async fn simple_chat_multipart(
    State((_, chat_db, bridge_manager)): State<RouterState>,
    mut multipart: Multipart,
) -> Result<Json<SimpleChatResponse>, StatusCode> {
    let temp_dir = ensure_temp_dir().await?;

    let mut message = String::new();
    let mut conversation_id: Option<String> = None;
    let mut model: Option<String> = None;
    let mut image_filenames = Vec::new();
    let mut saved_files = Vec::new();

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        let field_name = field.name().unwrap_or("");

        match field_name {
            "message" => {
                let text = field.text().await.map_err(|e| {
                    error!("Failed to read message field: {}", e);
                    StatusCode::BAD_REQUEST
                })?;
                message = text;
            }
            "conversation_id" => {
                let text = field.text().await.map_err(|e| {
                    error!("Failed to read conversation_id field: {}", e);
                    StatusCode::BAD_REQUEST
                })?;
                if !text.is_empty() {
                    conversation_id = Some(text);
                }
            }
            "model" => {
                let text = field.text().await.map_err(|e| {
                    error!("Failed to read model field: {}", e);
                    StatusCode::BAD_REQUEST
                })?;
                if !text.is_empty() {
                    model = Some(text);
                }
            }
            "images" => {
                // Handle image file uploads
                // Get filename first (before moving field)
                let original_filename = field.file_name().map(|s| s.to_string());
                let data = field.bytes().await.map_err(|e| {
                    error!("Failed to read image data: {}", e);
                    StatusCode::BAD_REQUEST
                })?;

                if let Some(filename) = original_filename {
                    // Validate file size (7MB limit)
                    if data.len() > 7 * 1024 * 1024 {
                        error!("Image file too large: {} bytes", data.len());
                        cleanup_images(&saved_files).await;
                        return Err(StatusCode::PAYLOAD_TOO_LARGE);
                    }

                    // Generate unique filename
                    let path_buf = PathBuf::from(&filename);
                    let ext_str = path_buf
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("bin");
                    let unique_filename = format!(
                        "{}-{}.{}",
                        uuid::Uuid::new_v4(),
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        ext_str
                    );

                    let file_path = temp_dir.join(&unique_filename);

                    // Save file - handle errors by cleaning up and returning
                    let mut file = match fs::File::create(&file_path).await {
                        Ok(f) => f,
                        Err(e) => {
                            error!("Failed to create temp file: {}", e);
                            cleanup_images(&saved_files).await;
                            return Err(StatusCode::INTERNAL_SERVER_ERROR);
                        }
                    };

                    if let Err(e) = file.write_all(&data).await {
                        error!("Failed to write image file: {}", e);
                        cleanup_images(&saved_files).await;
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }

                    if let Err(e) = file.sync_all().await {
                        error!("Failed to sync image file: {}", e);
                        cleanup_images(&saved_files).await;
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }

                    let filename_clone = unique_filename.clone();
                    image_filenames.push(unique_filename.clone());
                    saved_files.push(filename_clone);

                    info!(
                        "Saved uploaded image: {} ({} bytes)",
                        unique_filename,
                        data.len()
                    );
                }
            }
            _ => {
                warn!("Unknown multipart field: {}", field_name);
            }
        }
    }

    // Validate message is not empty
    if message.trim().is_empty() {
        cleanup_images(&saved_files).await;
        return Err(StatusCode::BAD_REQUEST);
    }

    // Call internal chat function with images
    let result = simple_chat_internal(
        message,
        conversation_id,
        Some(image_filenames),
        model,
        &chat_db,
        &bridge_manager,
    )
    .await;

    // Cleanup images after processing (whether success or failure)
    cleanup_images(&saved_files).await;

    result
}
