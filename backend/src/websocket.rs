//! WebSocket handlers for real-time updates
//!
//! This module handles WebSocket connections for streaming agent status updates
//! and output to connected clients. Supports ping/pong for connection keepalive.

use crate::state::{AgentId, AgentStatus, AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "agent_status_update")]
    AgentStatusUpdate {
        agent_id: AgentId,
        status: AgentStatus,
    },
    #[serde(rename = "agent_output")]
    AgentOutput { agent_id: AgentId, output: String },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}

// WebSocket upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<RwLock<AppState>>) {
    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket client connected");

    // Send initial state
    let initial_state = {
        let state = state.read().await;
        let agents: Vec<_> = state
            .agents_list()
            .iter()
            .map(|agent| {
                serde_json::json!({
                    "id": agent.id,
                    "name": agent.name,
                    "status": agent.status,
                })
            })
            .collect();

        serde_json::json!({
            "type": "initial_state",
            "agents": agents,
        })
    };

    if let Err(e) = sender.send(Message::Text(initial_state.to_string())).await {
        error!("Failed to send initial state: {}", e);
        return;
    }

    // Use a channel to send messages from receiver to sender
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Task to forward messages from channel to sender
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = sender.send(msg).await {
                error!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // Task to send periodic pings
    let ping_tx = tx.clone();
    let mut ping_task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            if ping_tx.send(Message::Ping(vec![])).is_err() {
                break;
            }
        }
    });

    // Receive messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Handle text messages (could be commands from client)
                    if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                        match ws_msg {
                            WebSocketMessage::Ping => {
                                // Respond to ping
                                if let Ok(pong_msg) = serde_json::to_string(&WebSocketMessage::Pong)
                                {
                                    if tx.send(Message::Text(pong_msg)).is_err() {
                                        break;
                                    }
                                }
                            }
                            _ => {
                                warn!("Received unhandled WebSocket message: {:?}", ws_msg);
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket client disconnected");
                    break;
                }
                Ok(Message::Pong(_)) => {
                    // Client responded to ping
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for any task to complete
    tokio::select! {
        _ = &mut send_task => {
            ping_task.abort();
            recv_task.abort();
        }
        _ = &mut ping_task => {
            send_task.abort();
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
            ping_task.abort();
        }
    }

    info!("WebSocket connection closed");
}

// Helper function to broadcast agent status updates
#[allow(dead_code)] // Reserved for future WebSocket functionality
pub async fn broadcast_agent_status(
    state: &Arc<RwLock<AppState>>,
    agent_id: AgentId,
    status: AgentStatus,
) {
    // In a real implementation, you'd maintain a list of connected WebSocket clients
    // and broadcast to all of them. For now, this is a placeholder.
    let _ = (state, agent_id, status);
    // TODO: Implement broadcast mechanism when we have client management
}
