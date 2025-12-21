# Session management

Session management for Ironstar, enabling per-user SSE subscriptions and state isolation in a Datastar-driven hypermedia application.

## Design principles

Sessions serve three critical purposes in Ironstar's architecture:

1. **Per-user event filtering**: Zenoh subscriptions use session IDs in key expressions to route events only to the correct SSE connections, avoiding broadcast-then-filter inefficiency.
2. **State isolation**: Each session maintains independent application state (e.g., TodoMVC data), enabling multiple concurrent users without interference.
3. **Reconnection resilience**: Session IDs in cookies allow SSE reconnections to resume the same logical session, preserving user context across network interruptions.

Unlike traditional session stores that hold authentication state, Ironstar sessions are lightweight identifiers for pub/sub scoping.
They exist in a single SQLite table alongside the event store, simplifying deployment without external session services.

## Session identification

### Session ID generation

Session IDs must be cryptographically secure to prevent enumeration attacks.
Use 192 bits (24 bytes) of entropy from a CSPRNG, encoded as URL-safe base64 without padding.

```rust
use rand::Rng;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

pub fn generate_session_id() -> String {
    let mut bytes = [0u8; 24]; // 192 bits of entropy
    rand::thread_rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}
```

**Rationale for 192 bits**: At 1 million sessions created per second, the probability of a collision remains negligible (< 2^-64) over the lifetime of the universe.
This eliminates the need for uniqueness checks in the database.

### Cookie configuration

Session cookies must use security flags to prevent common web attacks.

```rust
use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;

pub fn create_session_cookie(session_id: &str) -> Cookie<'static> {
    Cookie::build(("ironstar_session", session_id.to_owned()))
        .path("/")
        .http_only(true)        // Prevent XSS access to session ID
        .secure(true)           // Require HTTPS in production
        .same_site(SameSite::Lax) // CSRF protection with SSE compatibility
        .max_age(Duration::days(30))
        .build()
}

pub fn delete_session_cookie() -> Cookie<'static> {
    Cookie::build(("ironstar_session", ""))
        .path("/")
        .max_age(Duration::ZERO)
        .build()
}
```

**Security flags explained**:

- `http_only`: Prevents JavaScript access via `document.cookie`, mitigating XSS attacks.
- `secure`: Ensures transmission only over HTTPS (disable in dev mode).
- `same_site = Lax`: Allows cookies on top-level navigation (e.g., clicking links) but blocks third-party POST requests. SSE connections initiated by client-side JS are same-site, so `Lax` suffices.

**Why not `SameSite::Strict`?** Strict mode would block cookies when users navigate to the site from external links, breaking the initial SSE connection.

## Session storage

### SQLite schema

Sessions live in a dedicated table alongside the event store, avoiding external dependencies.

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT,                    -- Optional: link to authenticated user (NULL for anonymous)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    data JSON DEFAULT '{}'           -- Flexible session-scoped state storage
);

CREATE INDEX idx_sessions_expires ON sessions(expires_at);
CREATE INDEX idx_sessions_user ON sessions(user_id) WHERE user_id IS NOT NULL;
```

**Schema notes**:

- `id`: Session ID (primary key), generated via `generate_session_id()`.
- `user_id`: Optional foreign key to a users table. NULL for anonymous sessions.
- `last_seen_at`: Updated on every request for activity tracking and idle timeout.
- `expires_at`: Absolute expiration timestamp (e.g., `created_at + 30 days`).
- `data`: JSON blob for session-scoped application state (e.g., TodoMVC data, UI preferences).

**Why JSON for data?** Allows schema-less extension without migrations. Serialize using `serde_json::Value` or typed structs.

### Session service

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

## Zenoh key expression patterns for sessions

### Per-session event routing

See `zenoh-event-bus.md` "Key expression design" for wildcard patterns and structure.
Session-scoped keys follow the pattern `sessions/{session_id}/**`.

**Example session-scoped key expressions**:

```
sessions/Xy9Kp2Lm3nO4qR5sT6uV7wX/events/Todo/42
sessions/Xy9Kp2Lm3nO4qR5sT6uV7wX/events/notification
```

**Subscriber patterns**:

```rust
// Single session, all events
let key = format!("sessions/{}/**", session_id);

// Single session, specific aggregate type
let key = format!("sessions/{}/events/Todo/**", session_id);

// Single session, specific aggregate instance
let key = format!("sessions/{}/events/Todo/{}", session_id, todo_id);
```

### Global vs. session-scoped events

Some events are broadcast globally (e.g., system notifications), others are session-scoped (e.g., user-specific state updates).

```
events/global/announcement        → All sessions
events/session/{id}/Todo/42       → Single session only
```

**Command handlers decide routing**:

```rust
async fn handle_create_todo(
    session_id: &str,
    cmd: CreateTodo,
    zenoh: &Arc<zenoh::Session>,
) -> Result<()> {
    // Create event
    let event = TodoCreated { id: cmd.id, text: cmd.text };
    let payload = serde_json::to_vec(&event)?;

    // Publish to session-specific key
    let key = format!("events/session/{}/Todo/{}", session_id, cmd.id);
    zenoh.put(key, payload).await?;

    Ok(())
}
```

### Publishing session-scoped events

Convenience function for publishing to a session's key space.

```rust
pub async fn publish_to_session(
    zenoh: &zenoh::Session,
    session_id: &str,
    event_type: &str,
    aggregate_id: Option<&str>,
    payload: &[u8],
) -> Result<()> {
    let key = match aggregate_id {
        Some(id) => format!("events/session/{}/{}/{}", session_id, event_type, id),
        None => format!("events/session/{}/{}", session_id, event_type),
    };

    zenoh.put(key, payload)
        .await
        .map_err(|e| anyhow::anyhow!("Zenoh publish failed: {}", e))?;

    Ok(())
}
```

**Usage**:

```rust
publish_to_session(
    &zenoh,
    &session.id,
    "Todo",
    Some("42"),
    &event_payload,
).await?;
```

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

## Security considerations

### Session fixation prevention

Session fixation attacks occur when an attacker sets a victim's session ID to a known value, then hijacks the session after authentication.

**Mitigation**: Regenerate session ID on privilege escalation (e.g., login).

```rust
pub async fn regenerate_session(
    session_service: &SessionService,
    old_session_id: &str,
    user_id: String,
) -> Result<Session> {
    // Get old session data
    let old_session = session_service.get(old_session_id).await?
        .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

    // Create new session with user_id
    let new_session = session_service.create(Some(user_id)).await?;

    // Copy data to new session
    session_service.update_data(&new_session.id, old_session.data).await?;

    // Delete old session
    session_service.delete(old_session_id).await?;

    Ok(new_session)
}
```

**Usage in login handler**:

```rust
async fn login_handler(
    State(app_state): State<AppState>,
    SessionExtractor(session): SessionExtractor,
    Form(credentials): Form<LoginForm>,
) -> Result<Response> {
    let user_id = authenticate(&credentials)?;

    // Regenerate session to prevent fixation
    let new_session = regenerate_session(
        &app_state.session_service,
        &session.id,
        user_id,
    ).await?;

    // Return new cookie
    let cookie = create_session_cookie(&new_session.id);
    Ok(([(SET_COOKIE, cookie.to_string())], "Login successful").into_response())
}
```

### Session hijacking mitigation

Session hijacking involves stealing a valid session ID (e.g., via XSS or network sniffing).

**Mitigations**:

1. **HttpOnly cookies**: Prevents `document.cookie` access via XSS.
2. **Secure flag**: Ensures cookies only transmit over HTTPS.
3. **SameSite=Lax**: Blocks CSRF attacks while allowing top-level navigation.
4. **Short TTLs**: Limits window for stolen session use (e.g., 30 days max).
5. **User-Agent binding (optional)**: Store User-Agent header in session, validate on each request. Adds fragility (users change browsers) but increases hijacking difficulty.

```rust
// Optional: User-Agent validation
pub async fn validate_user_agent(
    session: &Session,
    current_user_agent: &str,
) -> Result<()> {
    let stored_ua = session.data.get("user_agent")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("No user agent in session"))?;

    if stored_ua != current_user_agent {
        return Err(anyhow::anyhow!("User agent mismatch"));
    }

    Ok(())
}
```

### Rate limiting session creation

Prevent session exhaustion attacks (filling database with sessions).

```rust
use governor::{Quota, RateLimiter};
use std::net::IpAddr;

pub struct SessionRateLimiter {
    limiter: RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>,
}

impl SessionRateLimiter {
    pub fn new() -> Self {
        // 10 sessions per IP per minute
        let quota = Quota::per_minute(10);
        Self {
            limiter: RateLimiter::keyed(quota),
        }
    }

    pub fn check(&self, ip: IpAddr) -> bool {
        self.limiter.check_key(&ip).is_ok()
    }
}
```

**Usage in extractor**:

```rust
async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    // ... cookie extraction ...

    // Rate limit session creation by IP
    if cookie.is_none() {
        let ip = parts.extensions.get::<IpAddr>()
            .ok_or(SessionError::NoIpAddress)?;

        if !app_state.session_rate_limiter.check(*ip) {
            return Err(SessionError::RateLimited);
        }
    }

    // ... create session ...
}
```

## References

- [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- Northstar session pattern: `/Users/crs58/projects/lakescope-workspace/datastar-go-nats-template-northstar/features/index/services/todo_service.go` (lines 165-182)
- Northstar SSE subscription: `/Users/crs58/projects/lakescope-workspace/datastar-go-nats-template-northstar/features/index/handlers.go` (lines 32-71)
- axum cookie handling: `axum-extra` crate documentation
- Zenoh key expressions: `~/projects/rust-workspace/zenoh` documentation

## Related documentation

- SSE connection lifecycle patterns: `sse-connection-lifecycle.md`
- Event replay and consistency: `event-replay-consistency.md`
- Zenoh configuration and key expressions: `zenoh-event-bus.md`
