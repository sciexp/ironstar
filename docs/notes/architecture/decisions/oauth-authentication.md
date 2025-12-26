# OAuth authentication architecture

This document records authentication and authorization technology selection decisions for the ironstar stack.
For session management fundamentals and security patterns, see the related documentation section below.

## Design principles

Ironstar uses OAuth-only authentication with no password-based login.
This architectural decision prioritizes minimal operational burden and security through delegation.

**Why OAuth-only:**

| Consideration | OAuth | Password-based |
|---------------|-------|----------------|
| Credential security | Provider responsibility | Your responsibility |
| MFA support | Provider handles (GitHub 2FA, Google Authenticator) | Must implement yourself |
| Password reset flows | Not needed | Must implement email flows |
| Credential storage | None locally | Must secure password hashes |
| Brute force protection | Provider handles | Must implement rate limiting |
| Security updates | Provider handles | Must track CVEs, update hashing |

Delegating identity verification to established providers (GitHub, Google) eliminates an entire class of security concerns while reducing operational burden.

**Algebraic justification:**

Authentication in the ironstar stack is an *effect at the boundary layer*.
The authenticated user context flows through handlers via the Reader monad pattern (axum extractors), keeping business logic pure and testable.

```rust
// OAuth authentication is a composition:
// AuthorizationCode -> Provider API -> UserProfile -> LocalUser -> Session

pub struct AuthContext {
    pub user: Option<User>,
    pub session_id: String,
}

// AuthContext flows as Reader environment
async fn protected_handler(
    auth: AuthContext,  // Extractor provides authenticated context
    State(store): State<EventStore>,
) -> Result<impl IntoResponse> {
    let user = auth.user.ok_or(AuthError::Unauthorized)?;
    // Business logic assumes authentication already validated
}
```

## Provider strategy

**Primary provider: GitHub OAuth**

GitHub OAuth is the initial implementation target for these reasons:

| Factor | GitHub | Google |
|--------|--------|--------|
| Protocol | Pure OAuth 2.0 | Full OIDC (more complex) |
| Setup complexity | Single OAuth app, one redirect URI | OIDC discovery, ID token validation |
| Target audience | Developer template → developers have GitHub | Broader consumer base |
| Crate requirement | `oauth2` crate sufficient | Needs `openidconnect` for proper OIDC |
| Profile endpoint | Single `/user` API call | Userinfo endpoint + ID token claims |

GitHub aligns with ironstar's developer-focused audience and provides the simplest path to working authentication.

**Planned extension: Google OIDC**

Google support is planned as a future extension using the `openidconnect` crate.
See the Google OIDC extension section below.

## Token handling

**Decision: Discard tokens after identity verification**

OAuth access tokens are used only during the callback flow to fetch user profile data.
Tokens are not stored in the database after session creation.

| Approach | Security | Complexity | Provider API access |
|----------|----------|------------|---------------------|
| Discard tokens (chosen) | Higher | Lower | None after login |
| Store tokens | Lower | Higher | Available for API calls |

**Rationale:**

- **Security**: No sensitive token data to protect at rest
- **Simplicity**: No token refresh logic, no encryption requirements
- **Minimal scope**: Ironstar authenticates identity, not integrates with provider APIs

If provider API access becomes needed (e.g., fetching user repositories from GitHub), token storage can be added as an extension with appropriate encryption at rest.

## OAuth flow

### Authorization redirect

User initiates login by clicking "Login with GitHub".
The server redirects to GitHub's authorization endpoint with state parameter for CSRF protection.

```rust
use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenUrl,
    basic::BasicClient,
};

pub fn create_github_client() -> BasicClient {
    BasicClient::new(ClientId::new(env!("GITHUB_CLIENT_ID").to_string()))
        .set_client_secret(ClientSecret::new(env!("GITHUB_CLIENT_SECRET").to_string()))
        .set_auth_uri(AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(env!("GITHUB_REDIRECT_URI").to_string()).unwrap())
}

async fn login_handler(
    State(app_state): State<AppState>,
    SessionExtractor(session): SessionExtractor,
) -> impl IntoResponse {
    let (auth_url, csrf_token) = app_state.oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    // Store CSRF token in session for validation on callback
    let mut data = session.data.clone();
    data["oauth_state"] = serde_json::json!(csrf_token.secret());
    app_state.session_service.update_data(&session.id, data).await.unwrap();

    Redirect::to(auth_url.as_str())
}
```

### Callback handling

GitHub redirects back with authorization code and state parameter.
Server validates state, exchanges code for token, fetches profile, creates/updates user, binds session.

```rust
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OAuthCallback {
    code: String,
    state: String,
}

async fn oauth_callback_handler(
    State(app_state): State<AppState>,
    SessionExtractor(session): SessionExtractor,
    Query(params): Query<OAuthCallback>,
) -> Result<impl IntoResponse, AuthError> {
    // 1. Validate CSRF state
    let stored_state = session.data.get("oauth_state")
        .and_then(|v| v.as_str())
        .ok_or(AuthError::InvalidState)?;

    if stored_state != params.state {
        return Err(AuthError::InvalidState);
    }

    // 2. Exchange code for token
    let token = app_state.oauth_client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|_| AuthError::TokenExchange)?;

    // 3. Fetch user profile from GitHub
    let profile = fetch_github_profile(token.access_token().secret()).await?;

    // 4. Create or update local user
    let user = app_state.user_service
        .upsert_from_oauth("github", &profile)
        .await?;

    // 5. Regenerate session (prevents fixation) and bind to user
    let new_session = regenerate_session(
        &app_state.session_service,
        &session.id,
        user.id.clone(),
    ).await?;

    // 6. Token is discarded here - not stored

    // 7. Set new session cookie and redirect
    let cookie = create_session_cookie(&new_session.id);
    Ok((
        [(axum::http::header::SET_COOKIE, cookie.to_string())],
        Redirect::to("/"),
    ))
}
```

### Fetching GitHub profile

```rust
#[derive(Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

async fn fetch_github_profile(access_token: &str) -> Result<GitHubUser, AuthError> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "ironstar")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| AuthError::ProfileFetch)?;

    response.json::<GitHubUser>().await
        .map_err(|_| AuthError::ProfileParse)
}
```

## User storage schema

Users are stored with provider identity information, designed for future multi-provider support.

```sql
-- Core user record (profile data)
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT,                       -- Primary email (may be NULL)
    display_name TEXT,
    avatar_url TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
) STRICT;

-- Provider identities (supports future multi-provider linking)
CREATE TABLE user_identities (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,           -- 'github' | 'google'
    provider_user_id TEXT NOT NULL,   -- External ID from provider
    provider_email TEXT,              -- Email from this specific provider
    created_at TEXT DEFAULT (datetime('now')),
    UNIQUE(provider, provider_user_id)
) STRICT;

CREATE INDEX idx_user_identities_user ON user_identities(user_id);
CREATE INDEX idx_user_identities_provider ON user_identities(provider, provider_user_id);
```

**Schema design notes:**

- **Separate `users` and `user_identities` tables**: Supports future account linking (one user, multiple providers)
- **No `password_hash` column**: OAuth-only by design
- **`provider_user_id`**: Immutable external ID from provider (GitHub user ID, Google sub claim)
- **`provider_email`**: Provider-specific email, may differ from primary user email
- **Unique constraint on `(provider, provider_user_id)`**: Prevents duplicate identity records

### User service

```rust
use uuid::Uuid;

pub struct UserService {
    pool: SqlitePool,
}

impl UserService {
    /// Create or update user from OAuth profile
    pub async fn upsert_from_oauth(
        &self,
        provider: &str,
        profile: &GitHubUser,
    ) -> Result<User> {
        let provider_user_id = profile.id.to_string();

        // Check if identity exists
        let existing = sqlx::query!(
            r#"
            SELECT user_id FROM user_identities
            WHERE provider = ? AND provider_user_id = ?
            "#,
            provider,
            provider_user_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(identity) = existing {
            // Update existing user profile
            let user = self.update_profile(&identity.user_id, profile).await?;
            return Ok(user);
        }

        // Create new user and identity
        let user_id = Uuid::new_v4().to_string();
        let identity_id = Uuid::new_v4().to_string();

        sqlx::query!(
            r#"
            INSERT INTO users (id, email, display_name, avatar_url)
            VALUES (?, ?, ?, ?)
            "#,
            user_id,
            profile.email,
            profile.name,
            profile.avatar_url,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO user_identities (id, user_id, provider, provider_user_id, provider_email)
            VALUES (?, ?, ?, ?, ?)
            "#,
            identity_id,
            user_id,
            provider,
            provider_user_id,
            profile.email,
        )
        .execute(&self.pool)
        .await?;

        self.get_by_id(&user_id).await?
            .ok_or_else(|| anyhow::anyhow!("User not found after creation"))
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<User>> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, display_name, avatar_url, created_at, updated_at
            FROM users WHERE id = ?
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query user")
    }
}
```

## Session binding after OAuth

After OAuth callback completes, the session is bound to the authenticated user.
This follows the pattern in `../infrastructure/session-security.md` for session fixation prevention.

```
OAuth Callback Flow:

1. Validate state parameter (CSRF)
2. Exchange code for access token
3. Fetch user profile from provider
4. Upsert local user record
5. Regenerate session ID (fixation prevention)
6. Bind session to user_id
7. Discard access token
8. Set new session cookie
9. Redirect to application
```

The session extractor loads the authenticated user from the session's `user_id` field:

```rust
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
        let session = SessionExtractor::from_request_parts(parts, state).await?.0;

        let user = if let Some(user_id) = &session.user_id {
            app_state.user_service.get_by_id(user_id).await?
        } else {
            None
        };

        Ok(AuthContext {
            user,
            session_id: session.id,
        })
    }
}
```

## Logout

Logout deletes the session, effectively de-authenticating the user.
No provider-side logout is performed (user remains logged into GitHub/Google).

```rust
async fn logout_handler(
    State(app_state): State<AppState>,
    SessionExtractor(session): SessionExtractor,
) -> impl IntoResponse {
    // Delete session
    app_state.session_service.delete(&session.id).await.unwrap();

    // Clear cookie
    let cookie = delete_session_cookie();
    (
        [(axum::http::header::SET_COOKIE, cookie.to_string())],
        Redirect::to("/"),
    )
}
```

## GitHub OAuth implementation notes

### Required scopes

| Scope | Purpose |
|-------|---------|
| `user:email` | Access user's primary email address |

The default scope (no explicit scope) provides read-only access to public profile data.
`user:email` adds access to the primary email, useful for display and notification purposes.

### Configuration

Environment variables:

```bash
GITHUB_CLIENT_ID=your_client_id
GITHUB_CLIENT_SECRET=your_client_secret
GITHUB_REDIRECT_URI=http://localhost:3000/auth/github/callback
```

Create OAuth app at: https://github.com/settings/developers

### GitHub API rate limits

Authenticated requests: 5,000 requests per hour per user.
Profile fetch is a single request per login, well within limits.

## Google OIDC extension (future)

Google authentication requires OIDC (OpenID Connect) rather than plain OAuth 2.0.
This adds complexity but provides standardized identity claims.

### Why OIDC for Google

| Aspect | Plain OAuth 2.0 | OIDC |
|--------|-----------------|------|
| Identity claims | Requires separate userinfo call | ID token contains claims |
| Token validation | Trust bearer token | Cryptographic signature verification |
| Standard | Provider-specific APIs | Standardized claim names |

### Implementation approach

```toml
# Additional dependency for Google
openidconnect = "3.3"
```

```rust
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    IssuerUrl, ClientId, ClientSecret, RedirectUrl,
};

pub async fn create_google_client() -> CoreClient {
    let issuer = IssuerUrl::new("https://accounts.google.com".to_string()).unwrap();

    // OIDC discovery fetches provider metadata automatically
    let provider_metadata = CoreProviderMetadata::discover_async(
        issuer,
        oauth2::reqwest::async_http_client,
    )
    .await
    .unwrap();

    CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(env!("GOOGLE_CLIENT_ID").to_string()),
        Some(ClientSecret::new(env!("GOOGLE_CLIENT_SECRET").to_string())),
    )
    .set_redirect_uri(RedirectUrl::new(env!("GOOGLE_REDIRECT_URI").to_string()).unwrap())
}
```

The callback handler validates the ID token signature using JWKS from the OIDC discovery document, then extracts standardized claims (`sub`, `email`, `name`, `picture`).

## Fallback evaluation: managed auth services

If the Rust OAuth ecosystem proves insufficient for ironstar's needs, two managed services have been evaluated as potential fallbacks.

### Trigger criteria for fallback

Consider managed auth when:

| Trigger | Description |
|---------|-------------|
| Multi-provider complexity | Account linking bugs, email conflicts across providers |
| Token lifecycle burden | Refresh token handling, token encryption at rest |
| Pre-built UI desire | Want hosted login pages, reduce frontend work |
| MFA requirements | Need MFA beyond what OAuth providers offer natively |
| Compliance requirements | SOC2, HIPAA require audit trails beyond SQLite |

### clerk-rs: Recommended fallback

**clerk-rs** is an SDK for Clerk, a managed authentication SaaS.

| Aspect | Assessment |
|--------|------------|
| axum integration | First-class middleware and extractors |
| OAuth handling | Clerk manages OAuth dance, tokens, providers |
| User management | Full CRUD, metadata, organizations |
| Pre-built UI | Hosted login pages, embeddable components |
| Pricing | Free tier available, usage-based scaling |

**When to use:**

Clerk is appropriate if OAuth complexity exceeds acceptable maintenance burden, particularly when adding multiple providers or needing pre-built authentication UI.

**Trade-offs:**

- Adds external service dependency (contradicts embedded philosophy)
- Requires internet connectivity for all auth operations
- Session state managed by Clerk, not local SQLite

**Integration pattern:**

```rust
// clerk-rs provides ClerkLayer middleware
use clerk_rs::{ClerkConfiguration, validators::axum::ClerkLayer};

let config = ClerkConfiguration::new(
    None, None,
    Some("your_secret_key".to_string()),
    None,
);

let app = Router::new()
    .route("/protected", get(protected_handler))
    .layer(ClerkLayer::new(
        MemoryCacheJwksProvider::new(Clerk::new(config)),
        Some(vec![String::from("/protected")]),
        true,
    ));
```

### workos: Enterprise-only

**workos** is an SDK for WorkOS, focused on enterprise authentication.

| Aspect | Assessment |
|--------|------------|
| axum integration | None built-in (HTTP client only) |
| Target use case | Enterprise SSO (SAML, SCIM, directory sync) |
| OAuth handling | Available but not primary focus |
| Complexity | Higher than clerk-rs for simple OAuth |

**When to use:**

WorkOS is only appropriate if ironstar needs enterprise features like SAML SSO integration with customer identity providers or SCIM directory synchronization.

**Not recommended for:**

Simple OAuth authentication. WorkOS adds unnecessary complexity for GitHub/Google login.

## RBAC authorization patterns

Authorization is separate from authentication.
OAuth establishes *who* the user is; RBAC determines *what* they can do.

Role-Based Access Control (RBAC) is a predicate over `(User, Resource, Action)`:

```rust
pub trait Authorizer {
    fn can(&self, user: &User, resource: &str, action: &str) -> bool;
}
```

### Simple role-based implementation

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
        // Roles stored in user metadata or separate table
        // Implementation depends on authorization requirements
        vec![Role::Viewer] // Default
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

### Authorization extractor

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
    format!("Hello, {}", user.display_name.unwrap_or_default())
}
```

## Cookie security

Session cookies use security attributes to prevent common web attacks.
See `../infrastructure/session-management.md` for complete cookie configuration.

| Attribute | Value | Protection |
|-----------|-------|------------|
| `HttpOnly` | `true` | Prevents JavaScript access, mitigates XSS |
| `Secure` | `true` (prod) | HTTPS-only transmission |
| `SameSite` | `Lax` | CSRF protection, allows SSE connections |
| `Path` | `/` | Scopes cookie to entire application |
| `Max-Age` | `30 days` | Session lifetime |

## Related documentation

- **Session management fundamentals**: `../infrastructure/session-management.md` — Session ID generation, cookie configuration, SQLite schema
- **Session security patterns**: `../infrastructure/session-security.md` — Zenoh key expressions, rate limiting, fixation prevention
- **Session implementation**: `../infrastructure/session-implementation.md` — Session service CRUD, axum extractors, SSE handler integration
- **Infrastructure decisions**: `infrastructure-decisions.md` — SQLite integration, embedded services philosophy
- **Event sourcing core**: `../cqrs/event-sourcing-core.md` — Command handling patterns for user actions

## References

- [OWASP OAuth Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/OAuth_Cheat_Sheet.html)
- [GitHub OAuth Documentation](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps)
- [oauth2 crate documentation](https://docs.rs/oauth2)
- [openidconnect crate documentation](https://docs.rs/openidconnect)
