# Session security

Security considerations and operational patterns for Ironstar session management: Zenoh key expressions, rate limiting, cleanup tasks, and attack mitigations.

See `session-management.md` for design principles and storage schema.
See `session-implementation.md` for Rust implementation patterns.

## Zenoh key expression patterns for sessions

### Per-session event routing

Session IDs appear in Zenoh key expressions to scope events to specific users.
For detailed Zenoh configuration and key expression fundamentals, see `zenoh-event-bus.md`.

```
events/session/{session_id}/notification
events/session/{session_id}/state_update
events/session/{session_id}/{aggregate_type}/{aggregate_id}
```

**Example key expressions**:

```
events/session/Xy9Kp2Lm3nO4qR5sT6uV7wX/notification
events/session/Xy9Kp2Lm3nO4qR5sT6uV7wX/Todo/42
events/session/Xy9Kp2Lm3nO4qR5sT6uV7wX/Todo/*  (all todos for session)
```

**Subscriber patterns**:

```rust
// Single session, all events
let key = format!("events/session/{}/**", session_id);

// Single session, specific aggregate type
let key = format!("events/session/{}/Todo/**", session_id);

// Single session, specific aggregate instance
let key = format!("events/session/{}/Todo/{}", session_id, todo_id);
```

### Global vs. session-scoped events

Some events are broadcast globally (e.g., system notifications), others are session-scoped (e.g., user-specific state updates).

```
events/global/announcement        → All sessions
events/session/{id}/Todo/42       → Single session only
```

**Command handlers decide routing**.
For the complete command handling pattern including validation, event emission, and persistence, see `event-sourcing-core.md`.

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

**Usage in OAuth callback handler**:

Ironstar uses OAuth-only authentication.
See `oauth-authentication.md` for the complete OAuth flow.

```rust
async fn oauth_callback_handler(
    State(app_state): State<AppState>,
    SessionExtractor(session): SessionExtractor,
    Query(params): Query<OAuthCallback>,
) -> Result<impl IntoResponse, AuthError> {
    // ... validate state, exchange code, fetch profile ...

    let user = app_state.user_service
        .upsert_from_oauth("github", &profile)
        .await?;

    // Regenerate session to prevent fixation
    let new_session = regenerate_session(
        &app_state.session_service,
        &session.id,
        user.id.clone(),
    ).await?;

    // Return new cookie and redirect
    let cookie = create_session_cookie(&new_session.id);
    Ok((
        [(SET_COOKIE, cookie.to_string())],
        Redirect::to("/"),
    ))
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

## Related documentation

- **Design principles**: `session-management.md` — Session ID generation, cookie configuration, SQLite schema
- **Implementation patterns**: `session-implementation.md` — Session service CRUD, axum extractors, SSE handlers
- **OAuth authentication**: `oauth-authentication.md` — OAuth flow, user schema, provider configuration
- **Zenoh configuration**: `zenoh-event-bus.md` — Key expression patterns, embedded setup, subscriber lifecycle
- **SSE connection lifecycle**: `sse-connection-lifecycle.md` — Client subscription, reconnection resilience

## References

- [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- Zenoh key expressions: `~/projects/rust-workspace/zenoh` documentation
