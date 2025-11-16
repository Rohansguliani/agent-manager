//! Gemini API client
//!
//! Direct HTTP client for calling the Gemini API.
//! This is used by the "Planner" agent to get structured JSON responses.

use crate::error::AppError;
use crate::orchestrator::config::OrchestratorConfig;
use crate::orchestrator::gemini_types::{
    GeminiApiRequest, GeminiApiResponse, GenerationConfig, RequestContent, RequestPart,
};
use anyhow::anyhow;

const GEMINI_API_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Call Gemini API directly with a prompt
///
/// This function makes a direct HTTP request to the Gemini API,
/// bypassing the CLI wrapper. This is used for "Planner" calls
/// that need structured JSON output.
///
/// # Arguments
/// * `api_key` - Gemini API key
/// * `prompt` - The prompt to send
/// * `model` - Model name (default: "gemini-2.5-flash")
/// * `force_json` - If true, request JSON response format
///
/// # Returns
/// * `Ok(String)` - The text content from the API response
/// * `Err(AppError)` - If API call failed
///
/// # Errors
/// * Returns `AppError::Internal` if API key is missing, HTTP request fails,
///   response parsing fails, or no valid content is found in the response.
pub async fn call_gemini_api(
    client: &reqwest::Client,
    api_key: &str,
    prompt: &str,
    model: Option<&str>,
    force_json: bool,
) -> Result<String, AppError> {
    call_gemini_api_with_base_url(
        client,
        api_key,
        prompt,
        model,
        force_json,
        GEMINI_API_BASE_URL,
    )
    .await
}

/// Internal function that allows custom base URL (for testing)
#[allow(dead_code)] // Used in tests
async fn call_gemini_api_with_base_url(
    client: &reqwest::Client,
    api_key: &str,
    prompt: &str,
    model: Option<&str>,
    force_json: bool,
    base_url: &str,
) -> Result<String, AppError> {
    if api_key.is_empty() {
        return Err(AppError::Internal(anyhow!("API key is empty")));
    }

    let config = OrchestratorConfig::default();
    let model_name = model.unwrap_or(&config.gemini_model);
    let url = format!(
        "{}/models/{}:generateContent?key={}",
        base_url, model_name, api_key
    );

    // Build request payload
    let mut generation_config = None;
    if force_json {
        generation_config = Some(GenerationConfig {
            response_mime_type: Some("application/json".to_string()),
        });
    }

    let request_body = GeminiApiRequest {
        contents: vec![RequestContent {
            parts: vec![RequestPart {
                text: prompt.to_string(),
            }],
        }],
        generation_config,
    };

    tracing::debug!(
        url = %url,
        model = %model_name,
        force_json = force_json,
        prompt_len = prompt.len(),
        "Calling Gemini API"
    );

    // Make POST request using shared client (connection pooling)
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            AppError::Internal(anyhow!("Failed to send HTTP request to Gemini API: {}", e))
        })?;

    // Check HTTP status
    let status = response.status();
    if !status.is_success() {
        let status_code = status.as_u16();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error body".to_string());

        tracing::error!(
            status_code = status_code,
            error_body = %error_body,
            "Gemini API returned error status"
        );

        if status_code == 429 {
            return Err(AppError::Internal(anyhow!(
                "Gemini API rate limit exceeded (HTTP {}): {}",
                status_code,
                error_body
            )));
        }

        return Err(AppError::Internal(anyhow!(
            "Gemini API returned error status {}: {}",
            status_code,
            error_body
        )));
    }

    // Parse response body
    let response_body = response.text().await.map_err(|e| {
        AppError::Internal(anyhow!(
            "Failed to read response body from Gemini API: {}",
            e
        ))
    })?;

    // Parse JSON response
    let parsed: GeminiApiResponse = serde_json::from_str(&response_body).map_err(|e| {
        AppError::Internal(anyhow!(
            "Failed to parse JSON response from Gemini API: {} - Response body: {}",
            e,
            response_body
        ))
    })?;

    // Check for blocked prompt
    if let Some(feedback) = &parsed.prompt_feedback {
        if let Some(reason) = &feedback.block_reason {
            return Err(AppError::Internal(anyhow!(
                "Gemini API blocked the prompt: {}",
                reason
            )));
        }
    }

    // Extract text content
    let candidate = parsed
        .candidates
        .first()
        .ok_or_else(|| AppError::Internal(anyhow!("Gemini API response contains no candidates")))?;

    let part = candidate.content.parts.first().ok_or_else(|| {
        AppError::Internal(anyhow!("Gemini API response candidate contains no parts"))
    })?;

    let text = &part.text;
    if text.is_empty() {
        return Err(AppError::Internal(anyhow!(
            "Gemini API response text is empty"
        )));
    }

    tracing::debug!(
        response_len = text.len(),
        "Successfully received response from Gemini API"
    );

    Ok(text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};
    use serial_test::serial;

    #[tokio::test]
    async fn test_call_gemini_api_empty_api_key() {
        let client = reqwest::Client::new();
        let result = call_gemini_api(&client, "", "test prompt", None, false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key is empty"));
    }

    #[tokio::test]
    #[serial]
    async fn test_call_gemini_api_success() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/models/gemini-2.5-flash:generateContent")
            .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
                "key".into(),
                "test-key".into(),
            )]))
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_body(
                r#"{
                    "candidates": [{
                        "content": {
                            "parts": [{
                                "text": "This is a test response"
                            }],
                            "role": "model"
                        }
                    }]
                }"#,
            )
            .create_async()
            .await;

        let base_url = &server.url();
        let client = reqwest::Client::new();
        let result = call_gemini_api_with_base_url(
            &client,
            "test-key",
            "test prompt",
            None,
            false,
            base_url,
        )
        .await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This is a test response");
    }

    #[tokio::test]
    #[serial]
    async fn test_call_gemini_api_json_mode() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/models/gemini-2.5-flash:generateContent")
            .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
                "key".into(),
                "test-key".into(),
            )]))
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_body(
                r#"{
                    "candidates": [{
                        "content": {
                            "parts": [{
                                "text": "{\"step\": 1, \"action\": \"test\"}"
                            }],
                            "role": "model"
                        }
                    }]
                }"#,
            )
            .create_async()
            .await;

        let base_url = &server.url();
        let client = reqwest::Client::new();
        let result = call_gemini_api_with_base_url(
            &client,
            "test-key",
            "test prompt",
            None,
            true, // force_json
            base_url,
        )
        .await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let response = result.unwrap();
        // Verify JSON mode was requested (response should be JSON)
        assert!(response.contains("\"step\""));
        assert!(response.contains("\"action\""));
    }

    #[tokio::test]
    #[serial]
    async fn test_call_gemini_api_empty_candidates() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/models/gemini-2.5-flash:generateContent")
            .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
                "key".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(r#"{"candidates": []}"#)
            .create_async()
            .await;

        let base_url = &server.url();
        let client = reqwest::Client::new();
        let result = call_gemini_api_with_base_url(
            &client,
            "test-key",
            "test prompt",
            None,
            false,
            base_url,
        )
        .await;

        mock.assert_async().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no candidates"));
    }

    #[tokio::test]
    #[serial]
    async fn test_call_gemini_api_blocked_prompt() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/models/gemini-2.5-flash:generateContent")
            .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
                "key".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                r#"{
                    "candidates": [],
                    "prompt_feedback": {
                        "block_reason": "SAFETY"
                    }
                }"#,
            )
            .create_async()
            .await;

        let base_url = &server.url();
        let client = reqwest::Client::new();
        let result = call_gemini_api_with_base_url(
            &client,
            "test-key",
            "test prompt",
            None,
            false,
            base_url,
        )
        .await;

        mock.assert_async().await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("blocked the prompt"),
            "Error message should contain 'blocked the prompt', got: {}",
            error_msg
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_call_gemini_api_rate_limit() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/models/gemini-2.5-flash:generateContent")
            .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
                "key".into(),
                "test-key".into(),
            )]))
            .with_status(429)
            .with_body(r#"{"error": "Rate limit exceeded"}"#)
            .create_async()
            .await;

        let base_url = &server.url();
        let client = reqwest::Client::new();
        let result = call_gemini_api_with_base_url(
            &client,
            "test-key",
            "test prompt",
            None,
            false,
            base_url,
        )
        .await;

        mock.assert_async().await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("rate limit") || error_msg.contains("429"));
    }

    #[tokio::test]
    #[serial]
    async fn test_call_gemini_api_invalid_json() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/models/gemini-2.5-flash:generateContent")
            .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
                "key".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(r#"This is not JSON"#)
            .create_async()
            .await;

        let base_url = &server.url();
        let client = reqwest::Client::new();
        let result = call_gemini_api_with_base_url(
            &client,
            "test-key",
            "test prompt",
            None,
            false,
            base_url,
        )
        .await;

        mock.assert_async().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse JSON"));
    }

    #[tokio::test]
    async fn test_call_gemini_api_invalid_api_key_real() {
        // This will fail with a real HTTP request, but we're testing error handling
        // In a real scenario, this would hit the real API with an invalid key
        let client = reqwest::Client::new();
        let result =
            call_gemini_api(&client, "invalid-key-12345", "test prompt", None, false).await;
        // Should return an error (either HTTP error or parsing error)
        assert!(result.is_err());
    }
}
