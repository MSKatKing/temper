//! # Dashboard WebSocket API
//!
//! This module provides a WebSocket-based API for the temper dashboard.
//!
//! ## WebSocket Protocol
//!
//! Connect to `/ws` to receive server events. The connection must be authenticated, see
//! [Authentication](#authentication). The protocol uses JSON messages
//! with a `type` field indicating the event type.
//!
//! ## Authentication
//!
//! The password is the `secret` from the `[dashboard]` config section. Clients never send it
//! directly, they send its lowercase hex encoded SHA-512 hash.
//!
//! - `POST /api/login` with `{ "hash": "<sha512 hex>" }` returns
//!   `{ "token": "...", "expires_in": <seconds> }`. Responds `401` on a wrong hash.
//! - `GET /api/session` returns `204` if the `Authorization: Bearer <token>` is still valid,
//!   `401` otherwise.
//! - `POST /api/logout` with the bearer header revokes the session.
//!
//! The WebSocket accepts the token in one of two ways:
//!
//! - an `Authorization: Bearer <token>` header on the upgrade request (rejected with `401` if
//!   invalid), or
//! - for clients that can't set headers (browsers), a first text frame of
//!   `{ "type": "Auth", "data": "<token>" }` within 10 seconds of connecting.
//!
//! Nothing is sent until the client is authenticated. On failure, or when the session expires
//! mid-connection, the server closes the socket with code `4401`.
//!
//! ### Handshake (sent once after authenticating)
//!
//! ```json
//! {
//!   "type": "Handshake",
//!   "data": {
//!     "system": { "cpu_model": "...", "cpu_cores": 12, "cpu_threads": 24 },
//!     "config": { "max_players": 100 }
//!   }
//! }
//! ```
//!
//! ### Metrics (sent every second)
//!
//! ```json
//! {
//!   "type": "Metric",
//!   "data": {
//!     "cpu_usage": 15.5,
//!     "ram_usage": 1073741824,
//!     "total_ram": 17179869184,
//!     "uptime": 3600,
//!     "storage_used": 536870912,
//!     "player_count": 42
//!   }
//! }
//! ```

use crate::telemetry::DashboardEvent;
use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use include_dir::{include_dir, Dir};
use std::net::SocketAddr;
use std::time::Duration;
use temper_state::GlobalState;
use tokio::sync::broadcast;
use tracing::{debug, info, trace};

mod auth;
mod handshake;
mod telemetry;
mod ws;

pub fn start_dashboard(state: GlobalState) {
    std::thread::Builder::new()
        .name("temper-dashboard".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .expect("Failed to build dashboard runtime");

            rt.block_on(start_webserver(state));
        })
        .expect("Failed to spawn dashboard thread");
}

/// The text the default config uses as a placeholder until a real secret is generated.
const UNSET_SECRET_PREFIX: &str = "This will be replaced";

/// The HTTP protocol used by the dashboard.
/// (I made this variable cuz rust rover was complaining about insecure url 💔😔)
const PROTOCOL: &str = "http";

/// This macro runs at COMPILE TIME.
/// It embeds the entire dashboard directory into the binary.
/// Uses OUT_DIR if available, otherwise falls back to CARGO_MANIFEST_DIR.
#[cfg(dashboard_in_out_dir)]
static DASHBOARD_DIR: Dir<'_> = include_dir!("$OUT_DIR/dashboard");

#[cfg(dashboard_in_manifest_dir)]
static DASHBOARD_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/dashboard-dist");

// Fallback if neither cfg is set (shouldn't happen, but prevents compile errors)
#[cfg(not(any(dashboard_in_out_dir, dashboard_in_manifest_dir)))]
static DASHBOARD_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/dashboard-dist");

async fn start_webserver(state: GlobalState) {
    debug!("Starting temper dashboard webserver...");

    // Gather handshake data once at startup
    let handshake = handshake::Handshake::gather(state.config.max_players);
    trace!("Handshake data gathered: {:?}", handshake);

    // Create a rx/tx (with max 100 messages buffered) for telemetry events
    let (tx, _rx) = broadcast::channel::<DashboardEvent>(100);

    // Spawn the Telemetry Loop in the background with GlobalState for player count
    let tx_clone = tx.clone();
    let state_clone = state.clone();
    tokio::spawn(telemetry::start_telemetry_loop(tx_clone, state_clone));

    let secret = &state.config.dashboard.secret;
    if secret.is_empty() || secret.starts_with(UNSET_SECRET_PREFIX) {
        tracing::error!("Dashboard secret is not set, refusing to start the dashboard");
        return;
    }
    let auth = auth::AuthState::new(secret);

    // Drop expired sessions and stale lockouts in the background
    let sweep_auth = auth.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(300));
        loop {
            ticker.tick().await;
            sweep_auth.sweep();
        }
    });

    // WebSocket state includes broadcast channel, handshake data and auth
    let ws_state = ws::WsState {
        tx,
        handshake,
        auth: auth.clone(),
    };

    let auth_routes = Router::new()
        .route("/api/login", post(auth::login_handler))
        .route("/api/logout", post(auth::logout_handler))
        .route("/api/session", get(auth::session_handler))
        .with_state(auth);

    // axum app/router
    let mut app = Router::new()
        .route("/ws", get(ws::ws_handler))
        .with_state(ws_state)
        .merge(auth_routes);

    if state.config.dashboard.serve_page {
        app = app
            .route("/", get(index_handler))
            .route("/{*path}", get(static_handler));
    }
    let addr = format!("{}:{}", state.config.host, state.config.dashboard.port);

    match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            info!("Dashboard listening on {PROTOCOL}://{}", addr);
            if let Err(e) = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            {
                tracing::error!("Dashboard server error: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to bind dashboard to {}: {}", addr, e);
        }
    }
}

async fn index_handler() -> impl IntoResponse {
    match DASHBOARD_DIR.get_file("index.html") {
        Some(file) => Html(file.contents()).into_response(),
        None => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

async fn static_handler(Path(path): Path<String>) -> Response {
    // Try to find the file in the embedded directory
    let path = path.trim_start_matches('/');

    // Try multiple fallbacks for the path
    let file = DASHBOARD_DIR
        .get_file(path)
        // Try with index.html appended (for directory-style URLs like /admin/)
        .or_else(|| {
            let with_index = if path.is_empty() {
                "index.html".to_string()
            } else if path.ends_with('/') {
                format!("{}index.html", path)
            } else {
                format!("{}/index.html", path)
            };
            DASHBOARD_DIR.get_file(&with_index)
        })
        // Try with .html extension (for clean URLs like /about -> /about.html)
        .or_else(|| {
            if !path.contains('.') {
                DASHBOARD_DIR.get_file(format!("{}.html", path))
            } else {
                None
            }
        });

    match file {
        Some(file) => {
            let file_path = file.path().to_string_lossy();
            let mime_type = mime_guess::from_path(file_path.as_ref())
                .first_or_octet_stream()
                .to_string();

            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime_type)],
                file.contents(),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}
