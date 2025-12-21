# Session implementation

Rust implementation patterns for session management in Ironstar: session service CRUD, axum extractors, and SSE handler integration.

See `session-management.md` for design principles and storage schema.
See `session-security.md` for Zenoh key expressions, rate limiting, and security considerations.

## Session service

The session service encapsulates all session CRUD operations with sqlx's compile-time query validation.

```rust
use sqlx::{SqlitePool, FromRow};
use serde::{Serialize, Deserialize};
use time::OffsetDateTime;
use anyhow::{Result, Context};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_seen_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct SessionService {
    pool: SqlitePool,
    ttl: time::Duration,
}

impl SessionService {
    pub fn new(pool: SqlitePool, ttl: time::Duration) -> Self {
        Self { pool, ttl }
    }

    /// Create a new session with generated ID
    pub async fn create(&self, user_id: Option<String>) -> Result<Session> {
        let id = generate_session_id();
        let now = OffsetDateTime::now_utc();
        let expires_at = now + self.ttl;

        let session = Session {
            id: id.clone(),
            user_id,
            created_at: now,
            last_seen_at: now,
            expires_at,
            data: serde_json::json!({}),
        };

        sqlx::query!(
            r#"
            INSERT INTO sessions (id, user_id, created_at, last_seen_at, expires_at, data)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            session.id,
            session.user_id,
            session.created_at,
            session.last_seen_at,
            session.expires_at,
            session.data,
        )
        .execute(&self.pool)
        .await
        .context("Failed to insert session")?;

        Ok(session)
    }

    /// Get session by ID, returning None if expired or not found
    pub async fn get(&self, id: &str) -> Result<Option<Session>> {
        let now = OffsetDateTime::now_utc();

        let session = sqlx::query_as!(
            Session,
            r#"
            SELECT id, user_id, created_at, last_seen_at, expires_at, data as "data: serde_json::Value"
            FROM sessions
            WHERE id = ? AND expires_at > ?
            "#,
            id,
            now
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query session")?;

        Ok(session)
    }

    /// Update last_seen_at timestamp
    pub async fn touch(&self, id: &str) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        sqlx::query!(
            r#"
            UPDATE sessions
            SET last_seen_at = ?
            WHERE id = ?
            "#,
            now,
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to touch session")?;

        Ok(())
    }

    /// Update session data
    pub async fn update_data(&self, id: &str, data: serde_json::Value) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE sessions
            SET data = ?, last_seen_at = ?
            WHERE id = ?
            "#,
            data,
            OffsetDateTime::now_utc(),
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update session data")?;

        Ok(())
    }

    /// Delete a specific session
    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM sessions
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete session")?;

        Ok(())
    }

    /// Cleanup expired sessions, returning count deleted
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query!(
            r#"
            DELETE FROM sessions
            WHERE expires_at <= ?
            "#,
            now
        )
        .execute(&self.pool)
        .await
        .context("Failed to cleanup expired sessions")?;

        Ok(result.rows_affected())
    }

    /// Delete all sessions for a specific user (for logout)
    pub async fn delete_user_sessions(&self, user_id: &str) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM sessions
            WHERE user_id = ?
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete user sessions")?;

        Ok(result.rows_affected())
    }
}
```

**Key methods**:

- `create`: Generates ID, sets expiration, returns Session.
- `get`: Validates expiration before returning.
- `touch`: Updates `last_seen_at` for idle timeout tracking.
- `update_data`: Stores session-scoped application state.
- `cleanup_expired`: Background task support.

## Axum integration

### Session extractor

A custom extractor retrieves or creates sessions from cookies, integrating seamlessly with axum's handler signature.

```rust
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{CookieJar, Cookie};

pub struct SessionExtractor(pub Session);

#[async_trait]
impl<S> FromRequestParts<S> for SessionExtractor
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = SessionError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| SessionError::CookieExtraction)?;

        // Try to get existing session from cookie
        if let Some(cookie) = jar.get("ironstar_session") {
            if let Some(session) = app_state.session_service.get(cookie.value()).await? {
                // Touch session to update last_seen_at
                app_state.session_service.touch(&session.id).await?;
                return Ok(SessionExtractor(session));
            }
        }

        // No valid session found, create new one
        let session = app_state.session_service.create(None).await?;

        // Set cookie in response (stored in extensions for middleware)
        parts.extensions.insert(create_session_cookie(&session.id));

        Ok(SessionExtractor(session))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Cookie extraction failed")]
    CookieExtraction,
    #[error("Session service error: {0}")]
    Service(#[from] anyhow::Error),
}

impl IntoResponse for SessionError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            SessionError::CookieExtraction => (StatusCode::BAD_REQUEST, "Invalid cookies"),
            SessionError::Service(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Session error"),
        };
        (status, message).into_response()
    }
}
```

**Cookie setting middleware**:

The extractor stores the cookie in request extensions. A layer extracts it and adds to the response.

```rust
use axum::{
    middleware::Next,
    response::Response,
};

pub async fn session_cookie_layer(
    mut req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let cookie = req.extensions_mut().remove::<Cookie>();
    let mut response = next.run(req).await;

    if let Some(cookie) = cookie {
        response.headers_mut().append(
            axum::http::header::SET_COOKIE,
            cookie.to_string().parse().unwrap(),
        );
    }

    response
}
```

**Router integration**:

```rust
use axum::{Router, routing::get, middleware};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/sse", get(sse_handler))
        .layer(middleware::from_fn(session_cookie_layer))
        .with_state(state)
}
```

### Handler pattern

Using the session extractor in SSE handlers for per-session event subscriptions.

```rust
use axum::{
    response::sse::{Event, KeepAlive, Sse},
    extract::State,
    http::HeaderMap,
};
use futures::stream::Stream;
use std::convert::Infallible;

async fn sse_handler(
    State(app_state): State<AppState>,
    SessionExtractor(session): SessionExtractor,
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Extract Last-Event-ID for reconnection
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());

    // Subscribe to session-specific events
    let key_expr = format!("events/session/{}/**", session.id);
    let subscriber = app_state.zenoh
        .declare_subscriber(&key_expr)
        .await
        .expect("Failed to create Zenoh subscriber");

    // Replay missed events if reconnecting
    if let Some(since_id) = last_event_id {
        let events = app_state.event_store
            .get_events_since(since_id)
            .await
            .expect("Failed to replay events");

        for event in events {
            // Send missed events...
        }
    }

    // Stream future events
    let stream = futures::stream::unfold(subscriber, |sub| async move {
        match sub.recv_async().await {
            Ok(sample) => {
                let payload = sample.payload().to_bytes();
                let event = Event::default().data(payload);
                Some((Ok(event), sub))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

**Key patterns**:

- Session ID automatically injected via extractor.
- Zenoh subscription scoped to `events/session/{session_id}/**`.
- `Last-Event-ID` header enables reconnection with event replay.

## Session lifecycle

### State diagram

```
┌─────────────┐   First Request    ┌─────────────┐
│  No Cookie  │ ──────────────────►│   Created   │
└─────────────┘                    └──────┬──────┘
                                          │
                                   Set-Cookie header
                                          │
                                          ▼
                                   ┌─────────────┐
                               ┌──►│   Active    │◄──┐
                               │   └──────┬──────┘   │
                               │          │          │
                            touch()       │       touch()
                               │          │          │
                               └──────────┴──────────┘
                                          │
                                   Expiration / Logout
                                          │
                                          ▼
                                   ┌─────────────┐
                                   │   Expired   │
                                   └─────────────┘
                                          │
                                  Background cleanup
                                          │
                                          ▼
                                   ┌─────────────┐
                                   │   Deleted   │
                                   └─────────────┘
```

**Transitions**:

1. **No Cookie → Created**: First request triggers session creation, cookie set in response.
2. **Created → Active**: Subsequent requests with valid cookie transition to active.
3. **Active → Active**: `touch()` updates `last_seen_at` on every request.
4. **Active → Expired**: TTL expiration or explicit logout.
5. **Expired → Deleted**: Background cleanup task removes row.

### Background cleanup task

Periodic deletion of expired sessions prevents unbounded table growth.

```rust
use tokio::time::{interval, Duration};
use std::sync::Arc;

pub async fn spawn_session_cleanup(
    session_service: Arc<SessionService>,
    cleanup_interval: Duration,
) {
    tokio::spawn(async move {
        let mut ticker = interval(cleanup_interval);
        loop {
            ticker.tick().await;
            match session_service.cleanup_expired().await {
                Ok(count) if count > 0 => {
                    tracing::info!(deleted = count, "Cleaned up expired sessions");
                }
                Ok(_) => {
                    tracing::trace!("Session cleanup ran, no expired sessions");
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Session cleanup failed");
                }
            }
        }
    });
}
```

**Startup integration**:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect("sqlite:ironstar.db").await?;
    let session_service = Arc::new(SessionService::new(pool, Duration::days(30)));

    // Spawn cleanup task (run every hour)
    spawn_session_cleanup(session_service.clone(), Duration::from_secs(3600)).await;

    // Start server...
    Ok(())
}
```

## Related documentation

- **Design principles and schema**: `session-management.md` — Session ID generation, cookie configuration, SQLite schema
- **Security and Zenoh patterns**: `session-security.md` — Key expressions, rate limiting, security mitigations
- **SSE connection lifecycle**: `../cqrs/sse-connection-lifecycle.md` — Client subscription, reconnection resilience
- **Event replay**: `../cqrs/event-replay-consistency.md` — Snapshot + delta patterns, Last-Event-ID handling
