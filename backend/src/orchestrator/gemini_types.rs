//! Gemini API response types
//!
//! Structs that mirror the Gemini API JSON response format.
//! Used to deserialize API responses into typed Rust structs.

use serde::{Deserialize, Serialize};

/// Top-level Gemini API response
#[derive(Deserialize, Debug)]
pub struct GeminiApiResponse {
    /// List of candidate responses from the model
    pub candidates: Vec<Candidate>,
    /// Optional feedback about the prompt (e.g., if it was blocked)
    #[serde(default)]
    pub prompt_feedback: Option<PromptFeedback>,
}

/// A single candidate response from the model
#[derive(Deserialize, Debug)]
pub struct Candidate {
    /// The content of this candidate
    pub content: Content,
    /// Why the model stopped generating (if applicable)
    #[serde(default)]
    #[allow(dead_code)] // Part of API response format, may be used in future
    pub finish_reason: Option<String>,
}

/// Content structure containing parts of the response
#[derive(Deserialize, Debug)]
pub struct Content {
    /// List of content parts (typically one text part)
    pub parts: Vec<Part>,
    /// Role of the content (e.g., "model")
    #[allow(dead_code)] // Part of API response format, may be used in future
    pub role: String,
}

/// A single part of content (typically text)
#[derive(Deserialize, Debug)]
pub struct Part {
    /// The text content of this part
    pub text: String,
}

/// Feedback about the prompt (e.g., if it was blocked)
#[derive(Deserialize, Debug)]
pub struct PromptFeedback {
    /// Reason the prompt was blocked (if applicable)
    #[serde(default)]
    pub block_reason: Option<String>,
}

/// Request structure for Gemini API
#[allow(dead_code)] // Used by api_client module
#[derive(Serialize, Debug)]
pub struct GeminiApiRequest {
    /// List of content items to send
    pub contents: Vec<RequestContent>,
    /// Optional generation configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

/// Content structure for requests
#[allow(dead_code)] // Used by api_client module
#[derive(Serialize, Debug)]
pub struct RequestContent {
    /// List of content parts
    pub parts: Vec<RequestPart>,
}

/// A single part for requests (typically text)
#[allow(dead_code)] // Used by api_client module
#[derive(Serialize, Debug)]
pub struct RequestPart {
    /// The text content
    pub text: String,
}

/// Generation configuration for requests
#[allow(dead_code)] // Used by api_client module
#[derive(Serialize, Debug)]
pub struct GenerationConfig {
    /// MIME type to force for response (e.g., "application/json")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,
}
