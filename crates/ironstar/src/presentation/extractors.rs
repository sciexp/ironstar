//! Session extractor for axum handlers.
//!
//! This module provides a [`SessionExtractor`] that extracts session information
//! from cookies and loads the associated session from the session store.
//!
//! # Usage
//!
//! ```rust,ignore
//! async fn protected_handler(
//!     SessionExtractor(session): SessionExtractor,
//! ) -> impl IntoResponse {
//!     // Session is guaranteed to be valid and loaded
//!     if let Some(user_id) = &session.user_id {
//!         format!("Hello, user {user_id}!")
//!     } else {
//!         "Hello, anonymous session!".to_string()
//!     }
//! }
//! ```
//!
//! # Cookie configuration
//!
//! Use [`session_cookie`] to create properly configured session cookies:
//!
//! ```rust,ignore
//! use axum_extra::extract::cookie::{Cookie, CookieJar};
//!
//! let session = store.create(None).await?;
//! let cookie = session_cookie(&session.id, true); // secure=true for HTTPS
//! let jar = jar.add(cookie);
//! ```

use crate::infrastructure::{InfrastructureError, Session, SessionStore};
use crate::state::AppState;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use std::fmt;

/// Cookie name for session identification.
pub const SESSION_COOKIE_NAME: &str = "ironstar_session";

/// Session extractor that loads a valid session from cookies.
///
/// This extractor fails if:
/// - No session store is configured in [`AppState`]
/// - No session cookie is present in the request
/// - The session cookie references an expired or deleted session
/// - A database error occurs during session lookup
///
/// Handlers that need sessions should use this extractor. To create new sessions,
/// use the session store directly and set the cookie in the response.
#[derive(Debug, Clone)]
pub struct SessionExtractor(pub Session);

impl SessionExtractor {
    /// Get the inner session.
    #[must_use]
    pub fn into_inner(self) -> Session {
        self.0
    }

    /// Get a reference to the session.
    #[must_use]
    pub fn session(&self) -> &Session {
        &self.0
    }
}

/// Rejection type for session extraction failures.
#[derive(Debug)]
pub enum SessionRejection {
    /// Session store is not configured in AppState.
    NoSessionStore,
    /// No session cookie in request.
    NoCookie,
    /// Session cookie exists but session not found (expired or deleted).
    SessionNotFound,
    /// Database error during session lookup.
    StoreError(InfrastructureError),
}

impl fmt::Display for SessionRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSessionStore => write!(f, "session store not configured"),
            Self::NoCookie => write!(f, "no session cookie"),
            Self::SessionNotFound => write!(f, "session not found or expired"),
            Self::StoreError(e) => write!(f, "session store error: {e}"),
        }
    }
}

impl std::error::Error for SessionRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::StoreError(e) => Some(e),
            _ => None,
        }
    }
}

impl IntoResponse for SessionRejection {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::NoSessionStore => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Session service unavailable",
            ),
            Self::NoCookie => (StatusCode::UNAUTHORIZED, "Session required"),
            Self::SessionNotFound => (StatusCode::UNAUTHORIZED, "Session expired or invalid"),
            Self::StoreError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Session lookup failed"),
        };

        (status, message).into_response()
    }
}

impl<S> FromRequestParts<S> for SessionExtractor
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = SessionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract AppState using FromRef
        let app_state = AppState::from_ref(state);

        // Get session store from AppState
        let session_store = app_state
            .session_store
            .as_ref()
            .ok_or(SessionRejection::NoSessionStore)?;

        // Extract cookies from request headers
        let jar = CookieJar::from_headers(&parts.headers);

        // Look for session cookie
        let session_id = jar
            .get(SESSION_COOKIE_NAME)
            .map(Cookie::value)
            .ok_or(SessionRejection::NoCookie)?;

        // Load session from store
        let session = session_store
            .get(session_id)
            .await
            .map_err(SessionRejection::StoreError)?
            .ok_or(SessionRejection::SessionNotFound)?;

        Ok(Self(session))
    }
}

/// Create a session cookie with proper security attributes.
///
/// # Arguments
///
/// * `session_id` - The session identifier to store in the cookie
/// * `secure` - Whether to set the `Secure` flag (should be `true` for HTTPS)
///
/// # Cookie attributes
///
/// - `HttpOnly`: Prevents JavaScript access (XSS protection)
/// - `SameSite::Lax`: Allows top-level navigation but blocks cross-origin requests
/// - `Path=/`: Cookie valid for all paths
/// - `Secure`: Only sent over HTTPS when `secure=true`
///
/// # Example
///
/// ```rust,ignore
/// let cookie = session_cookie(&session.id, true);
/// let jar = CookieJar::new().add(cookie);
/// (jar, Html(content)).into_response()
/// ```
#[must_use]
pub fn session_cookie(session_id: &str, secure: bool) -> Cookie<'static> {
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, session_id.to_owned());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    if secure {
        cookie.set_secure(true);
    }
    cookie
}

/// Create a cookie that clears the session (for logout).
///
/// Sets the cookie value to empty and max-age to zero, causing the browser
/// to delete the cookie.
///
/// # Example
///
/// ```rust,ignore
/// let jar = CookieJar::new().add(clear_session_cookie());
/// (jar, Redirect::to("/")).into_response()
/// ```
#[must_use]
pub fn clear_session_cookie() -> Cookie<'static> {
    // Use Cookie::build to set max_age with the time crate re-exported by cookie
    Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .max_age(cookie::time::Duration::ZERO)
        .build()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::infrastructure::{AssetManifest, SqliteSessionStore};
    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use chrono::Duration;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn create_test_pool() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("test pool");

        sqlx::query(include_str!("../../migrations/001_events.sql"))
            .execute(&pool)
            .await
            .expect("events migration");

        sqlx::query(include_str!("../../migrations/002_sessions.sql"))
            .execute(&pool)
            .await
            .expect("sessions migration");

        pool
    }

    fn create_app_state(pool: sqlx::SqlitePool) -> AppState {
        let session_store = Arc::new(SqliteSessionStore::new(pool.clone(), Duration::days(30)));
        AppState::new(pool, AssetManifest::default()).with_session_store(session_store)
    }

    async fn test_handler(SessionExtractor(session): SessionExtractor) -> String {
        format!("session_id={}", session.id)
    }

    #[test]
    fn session_cookie_has_correct_attributes() {
        let cookie = session_cookie("test-session-id", true);

        assert_eq!(cookie.name(), SESSION_COOKIE_NAME);
        assert_eq!(cookie.value(), "test-session-id");
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.secure(), Some(true));
    }

    #[test]
    fn session_cookie_without_secure() {
        let cookie = session_cookie("test-session-id", false);

        assert_eq!(cookie.secure(), None);
    }

    #[test]
    fn clear_session_cookie_has_empty_value() {
        let cookie = clear_session_cookie();

        assert_eq!(cookie.name(), SESSION_COOKIE_NAME);
        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.max_age(), Some(cookie::time::Duration::ZERO));
    }

    #[test]
    fn session_rejection_display() {
        assert_eq!(
            SessionRejection::NoSessionStore.to_string(),
            "session store not configured"
        );
        assert_eq!(SessionRejection::NoCookie.to_string(), "no session cookie");
        assert_eq!(
            SessionRejection::SessionNotFound.to_string(),
            "session not found or expired"
        );
    }

    #[tokio::test]
    async fn extractor_returns_session_when_valid_cookie() {
        let pool = create_test_pool().await;
        let state = create_app_state(pool);

        // Create a session
        let session = state
            .session_store
            .as_ref()
            .unwrap()
            .create(None)
            .await
            .expect("create session");

        let app = Router::new()
            .route("/test", get(test_handler))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Cookie", format!("{}={}", SESSION_COOKIE_NAME, session.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains(&session.id));
    }

    #[tokio::test]
    async fn extractor_returns_unauthorized_when_no_cookie() {
        let pool = create_test_pool().await;
        let state = create_app_state(pool);

        let app = Router::new()
            .route("/test", get(test_handler))
            .with_state(state);

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn extractor_returns_unauthorized_when_session_not_found() {
        let pool = create_test_pool().await;
        let state = create_app_state(pool);

        let app = Router::new()
            .route("/test", get(test_handler))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header(
                        "Cookie",
                        format!("{}=nonexistent-session-id", SESSION_COOKIE_NAME),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn extractor_returns_service_unavailable_when_no_store() {
        let pool = create_test_pool().await;
        // Create AppState without session store
        let state = AppState::new(pool, AssetManifest::default());

        let app = Router::new()
            .route("/test", get(test_handler))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Cookie", format!("{}=some-session-id", SESSION_COOKIE_NAME))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn extractor_returns_unauthorized_for_expired_session() {
        let pool = create_test_pool().await;
        // Create session store with negative TTL (sessions expire immediately)
        let session_store = Arc::new(SqliteSessionStore::new(pool.clone(), Duration::days(-1)));
        let state = AppState::new(pool, AssetManifest::default()).with_session_store(session_store);

        // Create a session (which is already expired due to negative TTL)
        let session = state
            .session_store
            .as_ref()
            .unwrap()
            .create(None)
            .await
            .expect("create session");

        let app = Router::new()
            .route("/test", get(test_handler))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Cookie", format!("{}={}", SESSION_COOKIE_NAME, session.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Expired session should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
