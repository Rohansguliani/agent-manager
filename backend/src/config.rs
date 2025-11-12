//! Application configuration
//!
//! Centralized configuration management with environment variable support
//! and sensible defaults.

use std::env;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Persistence configuration
    #[allow(dead_code)] // Will be used when persistence is fully implemented
    pub persistence: PersistenceConfig,
    /// Execution configuration
    pub execution: ExecutionConfig,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to bind the server to
    pub port: u16,
    /// Host address to bind to
    pub host: String,
}

/// Persistence configuration
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Base directory for storing agent data
    #[allow(dead_code)] // Will be used when persistence is fully implemented
    pub data_dir: String,
}

/// Execution configuration
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Default timeout for agent execution (in seconds)
    pub default_timeout_secs: u64,
}

impl Config {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Self {
        Self {
            server: ServerConfig {
                port: env::var("PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8080),
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            },
            persistence: PersistenceConfig {
                data_dir: env::var("DATA_DIR").unwrap_or_else(|_| {
                    // Default to ~/.agent-manager or current directory
                    if let Some(home) = env::var_os("HOME") {
                        format!("{}/.agent-manager", home.to_string_lossy())
                    } else {
                        ".agent-manager".to_string()
                    }
                }),
            },
            execution: ExecutionConfig {
                default_timeout_secs: env::var("EXECUTION_TIMEOUT_SECS")
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(30),
            },
        }
    }

    /// Get the server address as a string
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
