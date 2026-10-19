use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use dashmap::DashMap;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::fmt::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;

const SESSION_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(2 * 60 * 60);

struct Session {
    created: Instant,
    last_seen: Instant,
}

impl Session {
    fn is_expired(&self, now: Instant) -> bool {
        now.duration_since(self.created) >= SESSION_MAX_AGE
            || now.duration_since(self.last_seen) >= SESSION_IDLE_TIMEOUT
    }
}

struct AuthInner {
    password_hash: String,
    sessions: DashMap<String, Session>,
}

#[derive(Clone)]
pub struct AuthState(Arc<AuthInner>);

#[derive(Debug, PartialEq, Eq)]
pub enum LoginError {
    InvalidPassword,
}

impl AuthState {
    pub fn new(secret: &str) -> Self {
        Self(Arc::new(AuthInner {
            password_hash: sha512_hex(secret),
            sessions: DashMap::new(),
        }))
    }

    /// Checks a client supplied sha512 hash and, if it matches, opens a new session.
    /// Returns the session token.
    pub fn login(&self, hash: &str) -> Result<String, LoginError> {
        let now = Instant::now();

        let submitted = hash.trim().to_ascii_lowercase();
        let matches: bool = submitted
            .as_bytes()
            .ct_eq(self.0.password_hash.as_bytes())
            .into();
        if !matches {
            return Err(LoginError::InvalidPassword);
        }

        let token = generate_token();
        self.0.sessions.insert(
            token.clone(),
            Session {
                created: now,
                last_seen: now,
            },
        );
        Ok(token)
    }

    /// Checks that a token belongs to a live session
    pub fn validate(&self, token: &str) -> bool {
        let now = Instant::now();
        let Some(mut session) = self.0.sessions.get_mut(token) else {
            return false;
        };
        if session.is_expired(now) {
            drop(session);
            self.0.sessions.remove(token);
            return false;
        }
        session.last_seen = now;
        true
    }

    pub fn logout(&self, token: &str) {
        self.0.sessions.remove(token);
    }

    /// Drops expired sessions
    pub fn sweep(&self) {
        let now = Instant::now();
        self.0
            .sessions
            .retain(|_, session| !session.is_expired(now));
    }
}

fn sha512_hex(input: &str) -> String {
    let digest = Sha512::digest(input.as_bytes());
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn generate_token() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Pulls a bearer token out of the `Authorization` header, if there is one.
pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    scheme
        .eq_ignore_ascii_case("bearer")
        .then_some(token.trim())
        .filter(|token| !token.is_empty())
}

#[derive(Deserialize)]
pub struct LoginRequest {
    /// Lowercase or uppercase hex encoded sha512 of the dashboard secret.
    hash: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    expires_in: u64,
}

pub async fn login_handler(
    State(auth): State<AuthState>,
    Json(request): Json<LoginRequest>,
) -> Response {
    match auth.login(&request.hash) {
        Ok(token) => Json(LoginResponse {
            token,
            expires_in: SESSION_MAX_AGE.as_secs(),
        })
        .into_response(),
        Err(LoginError::InvalidPassword) => {
            (StatusCode::UNAUTHORIZED, "Invalid password").into_response()
        }
    }
}

pub async fn logout_handler(State(auth): State<AuthState>, headers: HeaderMap) -> StatusCode {
    if let Some(token) = bearer_token(&headers) {
        auth.logout(token);
    }
    StatusCode::NO_CONTENT
}

/// Lets a client check whether its token is still valid without opening a websocket.
pub async fn session_handler(State(auth): State<AuthState>, headers: HeaderMap) -> StatusCode {
    match bearer_token(&headers) {
        Some(token) if auth.validate(token) => StatusCode::NO_CONTENT,
        _ => StatusCode::UNAUTHORIZED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha512_hex_matches_known_vector() {
        assert_eq!(
            sha512_hex("abc"),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }

    #[test]
    fn login_with_correct_hash_creates_session() {
        let auth = AuthState::new("hunter2");
        let token = auth.login(&sha512_hex("hunter2")).unwrap();
        assert!(auth.validate(&token));
    }

    #[test]
    fn login_accepts_uppercase_hash() {
        let auth = AuthState::new("hunter2");
        let hash = sha512_hex("hunter2").to_ascii_uppercase();
        assert!(auth.login(&hash).is_ok());
    }

    #[test]
    fn login_with_wrong_hash_fails() {
        let auth = AuthState::new("hunter2");
        assert_eq!(
            auth.login(&sha512_hex("wrong")),
            Err(LoginError::InvalidPassword)
        );
        assert_eq!(auth.login(""), Err(LoginError::InvalidPassword));
    }

    #[test]
    fn unknown_and_logged_out_tokens_are_invalid() {
        let auth = AuthState::new("hunter2");
        assert!(!auth.validate("nope"));
        let token = auth.login(&sha512_hex("hunter2")).unwrap();
        auth.logout(&token);
        assert!(!auth.validate(&token));
    }

    #[test]
    fn tokens_are_unique() {
        let auth = AuthState::new("hunter2");
        let hash = sha512_hex("hunter2");
        let a = auth.login(&hash).unwrap();
        let b = auth.login(&hash).unwrap();
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn expired_sessions_are_rejected_and_swept() {
        let auth = AuthState::new("hunter2");
        let token = auth.login(&sha512_hex("hunter2")).unwrap();
        auth.0.sessions.get_mut(&token).unwrap().last_seen -=
            SESSION_IDLE_TIMEOUT + Duration::from_secs(1);
        auth.sweep();
        assert!(!auth.validate(&token));
        assert!(auth.0.sessions.is_empty());
    }

    #[test]
    fn bearer_token_parsing() {
        let mut headers = HeaderMap::new();
        assert_eq!(bearer_token(&headers), None);
        headers.insert(header::AUTHORIZATION, "Bearer abc123".parse().unwrap());
        assert_eq!(bearer_token(&headers), Some("abc123"));
        headers.insert(header::AUTHORIZATION, "bearer abc123".parse().unwrap());
        assert_eq!(bearer_token(&headers), Some("abc123"));
        headers.insert(header::AUTHORIZATION, "Basic abc123".parse().unwrap());
        assert_eq!(bearer_token(&headers), None);
        headers.insert(header::AUTHORIZATION, "Bearer ".parse().unwrap());
        assert_eq!(bearer_token(&headers), None);
    }
}
