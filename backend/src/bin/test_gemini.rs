//! Test binary for verifying Gemini CLI installation and functionality
//! This is a utility binary, not part of the main application

use std::env;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Gemini CLI invocation from Rust...\n");

    // Test 1: Check if gemini command exists
    println!("1. Checking if 'gemini' command is available...");
    let check_output = Command::new("which").arg("gemini").output().await?;

    if check_output.status.success() {
        let path = String::from_utf8_lossy(&check_output.stdout);
        println!("   ✓ Found gemini at: {}", path.trim());
    } else {
        eprintln!("   ✗ 'gemini' command not found in PATH");
        eprintln!("   Make sure you've installed: npm install -g @google/gemini-cli");
        return Err("Gemini CLI not found".into());
    }

    // Test 2: Get version
    println!("\n2. Getting Gemini CLI version...");
    let version_output = Command::new("gemini").arg("--version").output().await?;

    if version_output.status.success() {
        let version = String::from_utf8_lossy(&version_output.stdout);
        println!("   ✓ Version: {}", version.trim());
    } else {
        let error = String::from_utf8_lossy(&version_output.stderr);
        eprintln!("   ✗ Failed to get version: {}", error);
    }

    // Test 3: Check if API key is available
    println!("\n3. Checking for GEMINI_API_KEY environment variable...");
    match env::var("GEMINI_API_KEY") {
        Ok(key) => {
            println!("   ✓ GEMINI_API_KEY is set (length: {} chars)", key.len());
        }
        Err(_) => {
            eprintln!("   ⚠ GEMINI_API_KEY not found in environment");
            eprintln!("   Make sure to export it: export GEMINI_API_KEY=\"your-key\"");
            eprintln!("   Or load from .env file");
        }
    }

    // Test 4: Execute a query (async with timeout)
    println!("\n4. Executing test query...");
    println!("   Query: 'What is 2+2? Answer in one sentence.'");

    let mut cmd = Command::new("gemini");
    cmd.arg("What is 2+2? Answer in one sentence.");

    // Pass through environment variables (including GEMINI_API_KEY if set)
    cmd.envs(env::vars());

    // Execute with 30 second timeout
    match timeout(Duration::from_secs(30), cmd.output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                println!("   ✓ Response received:");
                println!("   {}", response.trim());

                // Show stderr if present (for debugging)
                if !output.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("\n   Stderr (for debugging):");
                    println!("   {}", stderr.trim());
                }
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                eprintln!("   ✗ Query failed:");
                eprintln!("   {}", error);
                eprintln!("   Exit code: {:?}", output.status.code());
                eprintln!("\n   Troubleshooting:");
                eprintln!("   - Make sure GEMINI_API_KEY is set: echo $GEMINI_API_KEY");
                eprintln!("   - Test manually: gemini 'What is 2+2?'");
            }
        }
        Ok(Err(e)) => {
            eprintln!("   ✗ Failed to execute command: {}", e);
        }
        Err(_) => {
            eprintln!("   ✗ Command timed out after 30 seconds");
        }
    }

    println!("\n✓ All tests completed!");
    Ok(())
}
