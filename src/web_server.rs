// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/web_server.rs
// Version: 1.0.1
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements the web server and WebSocket functionality for the
// real-time mining dashboard. It serves the HTML dashboard and broadcasts
// live mining statistics via WebSocket connections.
//
// Tree Location:
// - src/web_server.rs (web server and WebSocket handler)
// - Depends on: axum, tokio-tungstenite, serde, miner/stats

use axum::{
    Router,
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::{Html, Response},
    routing::get,
};
use log::{debug, error, info};
use sha3x_miner::miner::stats::MinerStats;
use std::sync::Arc;

const LOG_TARGET: &str = "tari::graxil::web_server";

/// Start the web server for the mining dashboard
///
/// Serves the dashboard at http://localhost:8080 and provides WebSocket
/// endpoint at ws://localhost:8080/ws for real-time data streaming
pub async fn start_web_server(stats: Arc<MinerStats>) {
    let app = Router::new()
        .route("/", get(dashboard_handler))
        .route("/ws", get(websocket_handler))
        .with_state(stats);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:8080").await {
        Ok(listener) => listener,
        Err(e) => {
            error!(target: LOG_TARGET,"❌ Failed to bind web server to port 8080: {}", e);
            error!(target: LOG_TARGET,"💡 Make sure port 8080 is not already in use");
            return;
        }
    };

    info!(target: LOG_TARGET,"🌐 Web dashboard available at: http://localhost:8080");
    info!(target: LOG_TARGET,"📊 Real-time charts at: http://localhost:8080 (Live Charts tab)");
    info!(target: LOG_TARGET,"🔗 WebSocket endpoint: ws://localhost:8080/ws");

    if let Err(e) = axum::serve(listener, app).await {
        error!(target: LOG_TARGET,"❌ Web server error: {}", e);
    }
}

/// Handler for the main dashboard page
///
/// Returns the HTML dashboard with embedded CSS and JavaScript
async fn dashboard_handler() -> Html<&'static str> {
    debug!(target: LOG_TARGET,"📄 Serving dashboard HTML");
    Html(include_str!("dashboard.html"))
}

/// WebSocket upgrade handler
///
/// Upgrades HTTP connections to WebSocket for real-time data streaming
async fn websocket_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(stats): axum::extract::State<Arc<MinerStats>>,
) -> Response {
    debug!(target: LOG_TARGET,"🔌 WebSocket connection request received");
    ws.on_upgrade(move |socket| handle_socket(socket, stats))
}

/// Handle WebSocket connections and stream mining data
///
/// Continuously sends mining statistics as JSON every 1 second
/// until the client disconnects or an error occurs
async fn handle_socket(mut socket: WebSocket, stats: Arc<MinerStats>) {
    use axum::extract::ws::Message;

    info!(target: LOG_TARGET,"✅ WebSocket client connected");

    loop {
        // Get current mining statistics
        let data = stats.to_websocket_data();

        // Serialize to JSON
        let json = match serde_json::to_string(&data) {
            Ok(json) => json,
            Err(e) => {
                error!(target: LOG_TARGET,"❌ Failed to serialize mining data: {}", e);
                break;
            }
        };

        // Send data to client
        if let Err(e) = socket.send(Message::Text(json)).await {
            debug!(target: LOG_TARGET,"🔌 WebSocket client disconnected: {}", e);
            break; // Client disconnected
        }

        // Wait 1 second before next update (changed from 2 seconds)
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    info!(target: LOG_TARGET,"🔌 WebSocket connection closed");
}

// Changelog:
// - v1.0.1 (2025-06-23): Updated for 1-second updates.
//   - Changed WebSocket update interval from 2 seconds to 1 second.
//   - Updated info message to mention "Live Charts tab" instead of "Analytics tab".
//   - Enhanced real-time responsiveness for dashboard updates.
// - v1.0.0 (2025-06-22): Initial web server implementation.
//   - Created web server with dashboard serving and WebSocket support.
//   - Implements real-time mining statistics broadcasting via WebSocket.
//   - Serves embedded HTML dashboard with tabbed interface.
//   - Added comprehensive error handling and logging.
//   - Updates mining data every 2 seconds to connected clients.
//   - Compatible with MinerStats v1.0.6+ WebSocket data serialization.
