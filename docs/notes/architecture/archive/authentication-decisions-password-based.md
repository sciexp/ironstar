# Authentication architecture decisions (archived)

> **Archive notice**: This document describes a password-based authentication approach that has been superseded.
> Ironstar now uses OAuth-only authentication.
> See `../oauth-authentication.md` for the current authentication architecture.
> This document is retained for reference only.

---

*Original content below*

---

This document records authentication and authorization technology selection decisions for the ironstar stack.
For session management fundamentals and security patterns, see the related documentation section below.

## Session-based authentication

**Algebraic justification:**

Authentication in the ironstar stack is an *effect at the boundary layer*.
The authenticated user context flows through handlers via the Reader monad pattern (axum extractors), keeping business logic pure and testable.

```rust
// Authentication is a morphism: Credentials -> Result<User, AuthError>
// Authorization is a predicate: (User, Resource) -> bool
// Session is a lookup: SessionID -> Option<User>

pub struct AuthContext {
    pub user: Option<User>,
    pub session_id: String,
}

// AuthContext flows as Reader environment
async fn protected_handler(
    auth: AuthContext,  // Extractor provides authenticated context
    State(store): State<EventStore>,
) -> Result<impl IntoResponse> {
    // Business logic assumes authentication already validated
    let user = auth.user.ok_or(AuthError::Unauthorized)?;
    // ...
}
```

**Why session-based over JWT:**

| Consideration | Session-based | JWT |
|---------------|---------------|-----|
| Revocation | Immediate (delete session) | Requires blacklist or short expiry |
| Storage | Server-side in SQLite | Client-side in cookie/localStorage |
| Stateful/stateless | Stateful (session table) | Stateless (self-contained token) |
| Token size | Small opaque ID (~32 bytes) | Large signed payload (~200+ bytes) |
| Rotation | Trivial (regenerate session ID) | Complex (refresh token flow) |
| Single-node suitability | Optimal | Overkill for embedded stack |

For ironstar's single-node deployment target with embedded SQLite, session-based authentication aligns better with the stack's simplicity principle.
Sessions already exist for SSE connection scoping (see `session-management.md`), so authentication extends the existing session infrastructure without introducing JWTs.

**Authentication flow:**

```
1. User submits credentials → POST /auth/login
2. Validate credentials → password hash verification
3. Create/regenerate session → SessionService::regenerate_session()
4. Link session to user → UPDATE sessions SET user_id = ?
5. Set session cookie → HttpOnly, Secure, SameSite=Lax
6. Subsequent requests → SessionExtractor reads cookie, loads user
```

## Password hashing with argon2

**Algebraic justification:**

Password hashing is a *one-way function* (non-invertible morphism):

```rust
// Hash: Password -> Hash (one-way)
// Verify: (Password, Hash) -> bool (comparison in hash space)
hash("password") = "$argon2id$v=19$m=19456,t=2,p=1$..."
hash("password") ≠ hash("password")  // Different salt each time

// Preimage resistance: given hash, cannot find password
// Second preimage resistance: given (password, hash), cannot find password' where hash(password') = hash
```

**Why argon2id over alternatives:**

| Algorithm | Year | Security | Performance | Rationale |
|-----------|------|----------|-------------|-----------|
| bcrypt | 2000 | Good | Fast (~100ms) | Older, limited parallelism resistance |
| scrypt | 2009 | Better | Moderate | Memory-hard but superseded |
| **argon2id** | 2015 | Best | Tunable | Winner of Password Hashing Competition 2015 |
| PBKDF2 | 2000 | Weak | Fast | Insufficient resistance to GPU attacks |

argon2id combines argon2i (data-independent, resistant to side-channel attacks) and argon2d (data-dependent, resistant to GPU/ASIC attacks).
The hybrid approach provides defense against both timing attacks and hardware acceleration.

**Implementation with argon2 crate:**

```rust
use argon2::{
    Argon2,
    PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{rand_core::OsRng, SaltString},
};
use anyhow::Result;

pub struct PasswordService {
    argon2: Argon2<'static>,
}

impl PasswordService {
    pub fn new() -> Self {
        // Use default params: m=19456 KiB, t=2, p=1
        // Adjust based on your security/performance requirements
        Self {
            argon2: Argon2::default(),
        }
    }

    /// Hash password with random salt
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?;
        Ok(hash.to_string())
    }

    /// Verify password against stored hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;

        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(anyhow::anyhow!("Password verification failed: {}", e)),
        }
    }
}
```

**Parameter tuning:**

The default argon2 parameters are conservative.
Adjust based on your threat model:

```rust
use argon2::{Argon2, Params, Algorithm, Version};

// Custom parameters for higher security
let params = Params::new(
    65536,  // m: 64 MiB memory
    3,      // t: 3 iterations
    4,      // p: 4 parallelism
    None,   // output length (default 32 bytes)
).expect("Invalid argon2 parameters");

let argon2 = Argon2::new(
    Algorithm::Argon2id,
    Version::V0x13,
    params,
);
```

**Timing attack resistance:**

The argon2 crate uses constant-time comparison for hash verification, preventing timing side-channel attacks.
Ensure you never short-circuit based on partial hash comparison in your own code.

## User storage schema

Users are stored in a dedicated `users` table with password hashes, linked to sessions via `user_id` foreign key.

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE,
    metadata JSON DEFAULT '{}'
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);

-- Sessions reference users (optional link)
-- See session-management.md for complete sessions table schema
ALTER TABLE sessions ADD CONSTRAINT fk_sessions_user
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
```

**Schema notes:**

- `id`: UUID v4 or ULID for user identifier.
- `password_hash`: argon2id hash string (e.g., `$argon2id$v=19$m=19456,t=2,p=1$...`).
- `is_active`: Soft delete flag for account deactivation without removing data.
- `metadata`: JSON blob for extensibility (e.g., profile data, preferences).

## Authentication service

Encapsulate authentication logic in a service layer, keeping handlers thin.

```rust
use sqlx::SqlitePool;
use uuid::Uuid;
use anyhow::{Result, Context};

#[derive(Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub is_active: bool,
    pub metadata: serde_json::Value,
}

pub struct AuthService {
    pool: SqlitePool,
    password_service: PasswordService,
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            password_service: PasswordService::new(),
        }
    }

    /// Register new user with hashed password
    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<User> {
        // Hash password before storing
        let password_hash = self.password_service.hash_password(&password)?;
        let id = Uuid::new_v4().to_string();

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, username, email, password_hash)
            VALUES (?, ?, ?, ?)
            RETURNING id, username, email, password_hash, created_at, updated_at,
                      is_active, metadata as "metadata: serde_json::Value"
            "#,
            id,
            username,
            email,
            password_hash,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to insert user")?;

        Ok(user)
    }

    /// Authenticate user by username and password
    pub async fn authenticate(
        &self,
        username: String,
        password: String,
    ) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at,
                   is_active, metadata as "metadata: serde_json::Value"
            FROM users
            WHERE username = ? AND is_active = TRUE
            "#,
            username,
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query user")?;

        let Some(user) = user else {
            return Ok(None);
        };

        // Verify password against stored hash
        let valid = self.password_service
            .verify_password(&password, &user.password_hash)?;

        if valid {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Get user by ID (for session loading)
    pub async fn get_by_id(&self, id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at,
                   is_active, metadata as "metadata: serde_json::Value"
            FROM users
            WHERE id = ? AND is_active = TRUE
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query user by ID")?;

        Ok(user)
    }

    /// Change password (requires old password verification)
    pub async fn change_password(
        &self,
        user_id: &str,
        old_password: String,
        new_password: String,
    ) -> Result<bool> {
        let user = self.get_by_id(user_id).await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Verify old password
        let valid = self.password_service
            .verify_password(&old_password, &user.password_hash)?;
        if !valid {
            return Ok(false);
        }

        // Hash new password
        let new_hash = self.password_service.hash_password(&new_password)?;

        // Update password hash and updated_at timestamp
        sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = ?, updated_at = ?
            WHERE id = ?
            "#,
            new_hash,
            time::OffsetDateTime::now_utc(),
            user_id,
        )
        .execute(&self.pool)
        .await
        .context("Failed to update password")?;

        Ok(true)
    }
}
```

## Authentication extractor

Extend the session extractor to provide authenticated user context.

```rust
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};

pub struct AuthContext {
    pub user: Option<User>,
    pub session_id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract session (reuse SessionExtractor logic)
        let session = SessionExtractor::from_request_parts(parts, state).await?;

        // Load user if session has user_id
        let user = if let Some(user_id) = &session.0.user_id {
            app_state.auth_service.get_by_id(user_id).await?
        } else {
            None
        };

        Ok(AuthContext {
            user,
            session_id: session.0.id,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Session error: {0}")]
    Session(#[from] SessionError),
    #[error("Service error: {0}")]
    Service(#[from] anyhow::Error),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self {
            AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
            AuthError::Session(_) => StatusCode::BAD_REQUEST,
            AuthError::Service(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
```

**Require authenticated user:**

For handlers that require authentication, use a newtype wrapper:

```rust
pub struct RequireAuth(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthContext::from_request_parts(parts, state).await?;
        let user = auth.user.ok_or(AuthError::Unauthorized)?;
        Ok(RequireAuth(user))
    }
}

// Usage in handler
async fn protected_handler(
    RequireAuth(user): RequireAuth,
    State(store): State<EventStore>,
) -> impl IntoResponse {
    // user is guaranteed to exist
    format!("Hello, {}", user.username)
}
```

## Authentication handlers

Login, logout, and registration handlers using hypertext templates and datastar SSE.

```rust
use axum::{
    extract::State,
    response::{sse::{Event, Sse}, IntoResponse, Response},
    Form,
};
use datastar::prelude::*;
use serde::Deserialize;
use std::convert::Infallible;

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    username: String,
    email: String,
    password: String,
}

/// POST /auth/login
pub async fn login_handler(
    State(app_state): State<AppState>,
    SessionExtractor(session): SessionExtractor,
    Form(form): Form<LoginForm>,
) -> Result<Response, AuthError> {
    // Authenticate user
    let user = app_state.auth_service
        .authenticate(form.username, form.password)
        .await?
        .ok_or(AuthError::Unauthorized)?;

    // Regenerate session to prevent fixation
    let new_session = regenerate_session(
        &app_state.session_service,
        &session.id,
        user.id.clone(),
    ).await?;

    // Send SSE response with new session cookie
    let cookie = create_session_cookie(&new_session.id);
    let sse_stream = futures::stream::once(async move {
        Ok::<_, Infallible>(
            Event::default()
                .event("login-success")
                .data(serde_json::json!({
                    "username": user.username,
                    "redirect": "/"
                }).to_string())
        )
    });

    Ok((
        [(axum::http::header::SET_COOKIE, cookie.to_string())],
        Sse::new(sse_stream),
    ).into_response())
}

/// POST /auth/logout
pub async fn logout_handler(
    State(app_state): State<AppState>,
    SessionExtractor(session): SessionExtractor,
) -> Result<Response, AuthError> {
    // Delete session from database
    app_state.session_service.delete(&session.id).await?;

    // Clear session cookie
    let cookie = delete_session_cookie();
    let sse_stream = futures::stream::once(async move {
        Ok::<_, Infallible>(
            Event::default()
                .event("logout-success")
                .data(serde_json::json!({"redirect": "/login"}).to_string())
        )
    });

    Ok((
        [(axum::http::header::SET_COOKIE, cookie.to_string())],
        Sse::new(sse_stream),
    ).into_response())
}

/// POST /auth/register
pub async fn register_handler(
    State(app_state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Result<Response, AuthError> {
    // Register new user
    let user = app_state.auth_service
        .register(form.username, form.email, form.password)
        .await?;

    // Auto-login: create session
    let session = app_state.session_service
        .create(Some(user.id.clone()))
        .await?;

    let cookie = create_session_cookie(&session.id);
    let sse_stream = futures::stream::once(async move {
        Ok::<_, Infallible>(
            Event::default()
                .event("register-success")
                .data(serde_json::json!({
                    "username": user.username,
                    "redirect": "/"
                }).to_string())
        )
    });

    Ok((
        [(axum::http::header::SET_COOKIE, cookie.to_string())],
        Sse::new(sse_stream),
    ).into_response())
}
```

## RBAC authorization patterns

Role-Based Access Control (RBAC) is a *predicate* over (User, Resource, Action):

```rust
// Authorization: (User, Resource, Action) -> bool
// Implemented as a trait for extensibility
pub trait Authorizer {
    fn can(&self, user: &User, resource: &str, action: &str) -> bool;
}
```

**Simple role-based implementation:**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Editor,
    Viewer,
}

impl User {
    pub fn roles(&self) -> Vec<Role> {
        // Parse from metadata JSON
        self.metadata.get("roles")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(Vec::new)
    }

    pub fn has_role(&self, role: Role) -> bool {
        self.roles().contains(&role)
    }
}

pub struct RbacAuthorizer;

impl Authorizer for RbacAuthorizer {
    fn can(&self, user: &User, _resource: &str, action: &str) -> bool {
        match action {
            "read" => user.has_role(Role::Viewer)
                   || user.has_role(Role::Editor)
                   || user.has_role(Role::Admin),
            "write" => user.has_role(Role::Editor)
                    || user.has_role(Role::Admin),
            "delete" => user.has_role(Role::Admin),
            _ => false,
        }
    }
}
```

**Authorization extractor:**

```rust
pub struct RequirePermission {
    pub action: &'static str,
}

#[async_trait]
impl<S> FromRequestParts<S> for RequirePermission
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthContext::from_request_parts(parts, state).await?;
        let user = auth.user.ok_or(AuthError::Unauthorized)?;

        // Store action in request extensions for later check
        let action = parts.extensions.get::<&'static str>()
            .copied()
            .ok_or(AuthError::Unauthorized)?;

        let app_state = AppState::from_ref(state);
        if !app_state.authorizer.can(&user, "", action) {
            return Err(AuthError::Unauthorized);
        }

        Ok(RequirePermission { action })
    }
}
```

## Cookie security hardening

Session cookies must use security attributes to prevent common web attacks.
See `session-security.md` for complete security considerations and attack mitigations.

```rust
use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;

pub fn create_session_cookie(session_id: &str) -> Cookie<'static> {
    Cookie::build(("ironstar_session", session_id.to_owned()))
        .path("/")
        .http_only(true)        // Prevent XSS access via document.cookie
        .secure(true)           // HTTPS-only in production (disable in dev)
        .same_site(SameSite::Lax) // CSRF protection, allows top-level navigation
        .max_age(Duration::days(30))
        .build()
}
```

**Security attributes explained:**

| Attribute | Value | Protection |
|-----------|-------|------------|
| `HttpOnly` | `true` | Prevents JavaScript access, mitigates XSS cookie theft |
| `Secure` | `true` (prod) | Ensures cookies only transmitted over HTTPS |
| `SameSite` | `Lax` | Blocks CSRF while allowing SSE connections initiated by client JS |
| `Path` | `/` | Scopes cookie to entire application |
| `Max-Age` | `30 days` | Session lifetime (matches SQLite session TTL) |

**Why SameSite=Lax, not Strict:**

`SameSite::Strict` would block cookies when users navigate to the site from external links, breaking the initial SSE connection.
`Lax` mode allows cookies on top-level navigation while blocking third-party POST requests, providing CSRF protection without breaking legitimate use cases.

## Related documentation

- **Session management fundamentals**: `session-management.md` — Session ID generation, cookie configuration, SQLite schema
- **Session security patterns**: `session-security.md` — Zenoh key expressions, rate limiting, security mitigations
- **Session implementation**: `session-implementation.md` — Session service CRUD, axum extractors, SSE handler integration
- **Backend core decisions**: `backend-core-decisions.md` — axum extractors as Reader monad, effect boundaries
- **Infrastructure decisions**: `infrastructure-decisions.md` — SQLite sessions table integration
- **Event sourcing core**: `event-sourcing-core.md` — Command handling patterns for user actions
