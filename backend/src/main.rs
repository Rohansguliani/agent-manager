//! Agent Manager Backend
//!
//! A REST API and WebSocket server for managing CLI-based AI agents.
//! Provides endpoints for agent CRUD operations and real-time status updates.

mod api;
mod config;
mod error;
mod executor;
mod orchestrator;
mod services;
mod state;
mod websocket;

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use config::Config;
use serde::Serialize;
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, info_span, Instrument};
use uuid::Uuid;

#[derive(Serialize)]
struct HelloResponse {
    message: String,
    status: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    message: String,
}

/// Request ID middleware - adds unique ID to each request for tracing
async fn request_id_middleware(request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let span = info_span!(
        "request",
        request_id = %request_id,
        method = %method,
        uri = %uri,
    );

    let response = next.run(request).instrument(span).await;

    let duration = start.elapsed();
    info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        status = %response.status().as_u16(),
        duration_ms = duration.as_millis(),
        "Request completed"
    );

    response
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = Config::from_env();
    info!("Configuration loaded: {:?}", config);

    // Initialize application state
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Try to load agents from default path
    let default_path = state::persistence::AgentRegistry::default_path();
    if default_path.exists() {
        match app_state.write().await.load_agents(&default_path) {
            Ok(count) => info!("Loaded {} agents from {}", count, default_path.display()),
            Err(e) => tracing::warn!("Failed to load agents: {}", e),
        }
    }

    // Build our application with routes
    let app = Router::new()
        // Health check and hello world
        .route("/", get(hello_world))
        .route("/api/health", get(health_check))
        // Agent management API
        .route(
            "/api/agents",
            get(api::agents::list_agents).post(api::agents::create_agent),
        )
        .route(
            "/api/agents/:id",
            get(api::agents::get_agent)
                .put(api::agents::update_agent)
                .delete(api::agents::delete_agent),
        )
        .route("/api/agents/:id/start", post(api::agents::start_agent))
        .route("/api/agents/:id/stop", post(api::agents::stop_agent))
        .route("/api/agents/:id/query", post(api::queries::query_agent))
        .route("/api/query/stream", post(api::queries::query_stream))
        // File system API
        .route("/api/files", get(api::list_files))
        .route(
            "/api/files/working-directory",
            get(api::get_working_directory).post(api::set_working_directory),
        )
        // Orchestration API
        .route(
            "/api/orchestrate/poem",
            post(api::orchestrator::orchestrate_poem),
        )
        .route("/api/orchestrate", post(api::orchestrator::orchestrate))
        // Phase 6.1: Pre-flight check - Plan + Optimizer
        .route("/api/plan", post(api::orchestrator::plan_with_analysis))
        // Phase 6.2: Graph visualization
        .route(
            "/api/orchestrate/graph",
            get(api::orchestrator_graph::get_graph_structure),
        )
        // Phase 6.4: Settings Panel
        .route(
            "/api/config",
            get(api::orchestrator::get_config).post(api::orchestrator::update_config),
        )
        // WebSocket for real-time updates
        .route("/ws", get(websocket::websocket_handler))
        // Middleware (order matters - request_id should be first)
        .layer(axum::middleware::from_fn(request_id_middleware))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        )
        .layer(CorsLayer::permissive()) // Allow CORS for development
        .with_state(app_state);

    // Bind to address from config
    let addr: SocketAddr = config
        .server_addr()
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid server address: {}", e))?;

    info!("🚀 Server running on http://{}", addr);
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Setup graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Handle graceful shutdown signals (Ctrl+C, SIGTERM)
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down gracefully...");
        },
    }
}

async fn hello_world() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello from Agent Manager Backend!".to_string(),
        status: "ok".to_string(),
    })
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        message: "Backend is healthy".to_string(),
    })
}
