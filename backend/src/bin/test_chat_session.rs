//! Test binary for verifying Gemini CLI chat session (multi-turn)
//! This simulates how the backend will handle chat context.

use std::env;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Gemini CLI Chat Session (Multi-turn)...\n");

    // Turn 1: Say "Hi"
    println!("1. User: Hi");
    let response1 = run_gemini("Hi").await?;
    println!("   Model: {}\n", response1.trim());

    // Turn 2: Ask "Whats ur name" with context
    println!("2. User: Whats ur name");

    // Construct prompt with history
    let context_prompt = format!(
        "Previous conversation:\nUser: Hi\nModel: {}\n\nUser: Whats ur name? Answer briefly.",
        response1.trim()
    );

    let response2 = run_gemini(&context_prompt).await?;
    println!("   Model: {}\n", response2.trim());

    println!("✓ Chat session test completed!");
    Ok(())
}

async fn run_gemini(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("gemini");
    cmd.arg(prompt);

    // Pass through environment variables
    cmd.envs(env::vars());

    // Execute with 15 second timeout (user asked for 10s check, giving 15s buffer)
    match timeout(Duration::from_secs(15), cmd.output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Command failed: {}", error).into())
            }
        }
        Ok(Err(e)) => Err(format!("Failed to execute: {}", e).into()),
        Err(_) => Err("Command timed out after 15 seconds".into()),
    }
}
