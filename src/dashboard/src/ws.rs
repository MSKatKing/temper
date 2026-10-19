use crate::auth::{bearer_token, AuthState};
use crate::handshake::Handshake;
use crate::telemetry::DashboardEvent;
use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::{interval, timeout};

/// How long a client has to authenticate after connecting without a bearer header.
const AUTH_TIMEOUT: Duration = Duration::from_secs(10);
/// How often a connected client's session is re-validated.
const SESSION_RECHECK_INTERVAL: Duration = Duration::from_secs(30);
/// Close code sent when authentication fails or the session ends.
const CLOSE_UNAUTHORIZED: u16 = 4401;

/// The state needed for the websocket
#[derive(Clone)]
pub struct WsState {
    pub tx: broadcast::Sender<DashboardEvent>,
    pub handshake: Handshake,
    pub auth: AuthState,
}

/// Messages a client can send to the server.
#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
enum ClientMessage {
    Auth(String),
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsState>,
    headers: HeaderMap,
) -> Response {
    // Clients that can set headers authenticate during the upgrade, browsers
    // send an Auth message as their first frame instead.
    let token = match bearer_token(&headers) {
        Some(token) if state.auth.validate(token) => Some(token.to_string()),
        Some(_) => return StatusCode::UNAUTHORIZED.into_response(),
        None => None,
    };
    ws.on_upgrade(|socket| handle_socket(socket, state, token))
}

async fn close_unauthorized(mut socket: WebSocket, reason: &'static str) {
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: CLOSE_UNAUTHORIZED,
            reason: Utf8Bytes::from_static(reason),
        })))
        .await;
}

/// Waits for the client's first message and checks it is a valid Auth message.
async fn await_auth(socket: &mut WebSocket, auth: &AuthState) -> Option<String> {
    let first = timeout(AUTH_TIMEOUT, socket.recv()).await.ok()??.ok()?;
    let Message::Text(text) = first else {
        return None;
    };
    match serde_json::from_str::<ClientMessage>(&text) {
        Ok(ClientMessage::Auth(token)) if auth.validate(&token) => Some(token),
        _ => None,
    }
}

async fn handle_socket(mut socket: WebSocket, state: WsState, token: Option<String>) {
    let token = match token {
        Some(token) => token,
        None => match await_auth(&mut socket, &state.auth).await {
            Some(token) => token,
            None => return close_unauthorized(socket, "Unauthorized").await,
        },
    };

    // Send handshake once authenticated
    let handshake = DashboardEvent::Handshake(state.handshake.clone());
    if let Ok(json) = serde_json::to_string(&handshake) {
        if socket
            .send(Message::Text(Utf8Bytes::from(json)))
            .await
            .is_err()
        {
            // Client disconnected during handshake
            return;
        }
    }

    // Subscribe to the telemetry channel for ongoing metrics
    let mut rx = state.tx.subscribe();
    let mut recheck = interval(SESSION_RECHECK_INTERVAL);

    loop {
        tokio::select! {
            event = rx.recv() => {
                let Ok(event) = event else { break };
                if let Ok(json) = serde_json::to_string(&event) {
                    if socket
                        .send(Message::Text(Utf8Bytes::from(json)))
                        .await
                        .is_err()
                    {
                        // Client disconnected
                        break;
                    }
                }
            }
            _ = recheck.tick() => {
                if !state.auth.validate(&token) {
                    return close_unauthorized(socket, "Session expired").await;
                }
            }
            incoming = socket.recv() => {
                // Nothing is accepted from authenticated clients yet, only watch for closure
                if !matches!(incoming, Some(Ok(_))) {
                    break;
                }
            }
        }
    }
}
