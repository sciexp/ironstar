//! SQLite session store for authentication and SSE scoping.
//!
//! Sessions are lightweight identifiers for:
//! - Per-user event filtering via Zenoh key expressions
//! - State isolation between concurrent users
//! - Reconnection resilience with cookie-based session resumption
//!
//! Session IDs use 192 bits of entropy (24 bytes) encoded as URL-safe base64.

use crate::infrastructure::error::InfrastructureError;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::sqlite::SqlitePool;
use std::future::Future;

/// Session data stored in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session identifier (24-byte base64url token).
    pub id: String,
    /// Optional user ID (bound after OAuth).
    pub user_id: Option<String>,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp.
    pub last_seen_at: DateTime<Utc>,
    /// When the session expires.
    pub expires_at: DateTime<Utc>,
    /// Session-scoped application state.
    pub data: serde_json::Value,
}

/// Generate a cryptographically secure session ID.
///
/// Uses 24 bytes (192 bits) of entropy from the CSPRNG, encoded as URL-safe base64.
/// At 1 million sessions/second, collision probability remains negligible.
#[must_use]
pub fn generate_session_id() -> String {
    let mut bytes = [0u8; 24];
    rand::rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}

/// Session store trait for authentication and SSE scoping.
///
/// Implementations provide session CRUD operations with async methods.
/// The primary implementation is `SqliteSessionStore`.
pub trait SessionStore: Send + Sync {
    /// Create a new session, optionally bound to a user.
    fn create(
        &self,
        user_id: Option<&str>,
    ) -> impl Future<Output = Result<Session, InfrastructureError>> + Send;

    /// Get a session by ID. Returns None if not found or expired.
    fn get(
        &self,
        id: &str,
    ) -> impl Future<Output = Result<Option<Session>, InfrastructureError>> + Send;

    /// Update session data (JSON value).
    fn update_data(
        &self,
        id: &str,
        data: serde_json::Value,
    ) -> impl Future<Output = Result<(), InfrastructureError>> + Send;

    /// Touch session to update last_seen_at timestamp.
    fn touch(&self, id: &str) -> impl Future<Output = Result<(), InfrastructureError>> + Send;

    /// Delete a specific session.
    fn delete(&self, id: &str) -> impl Future<Output = Result<(), InfrastructureError>> + Send;

    /// Cleanup expired sessions, returning count deleted.
    fn cleanup_expired(&self) -> impl Future<Output = Result<u64, InfrastructureError>> + Send;

    /// Delete all sessions for a user (for logout).
    fn delete_user_sessions(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<u64, InfrastructureError>> + Send;
}

/// SQLite-backed session store.
#[derive(Debug, Clone)]
pub struct SqliteSessionStore {
    pool: SqlitePool,
    ttl: Duration,
}

impl SqliteSessionStore {
    /// Create a new session store with the given pool and TTL.
    #[must_use]
    pub fn new(pool: SqlitePool, ttl: Duration) -> Self {
        Self { pool, ttl }
    }

    /// Create with default TTL of 30 days.
    #[must_use]
    pub fn with_default_ttl(pool: SqlitePool) -> Self {
        Self::new(pool, Duration::days(30))
    }
}

impl SessionStore for SqliteSessionStore {
    fn create(
        &self,
        user_id: Option<&str>,
    ) -> impl Future<Output = Result<Session, InfrastructureError>> + Send {
        let pool = self.pool.clone();
        let ttl = self.ttl;
        let user_id = user_id.map(String::from);

        async move {
            let id = generate_session_id();
            let now = Utc::now();
            let expires_at = now + ttl;

            let session = Session {
                id: id.clone(),
                user_id: user_id.clone(),
                created_at: now,
                last_seen_at: now,
                expires_at,
                data: serde_json::json!({}),
            };

            // Format timestamps as ISO 8601 for SQLite TEXT columns
            let created_at_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
            let expires_at_str = expires_at.format("%Y-%m-%d %H:%M:%S").to_string();
            let data_str = session.data.to_string();

            sqlx::query(
                r#"
                INSERT INTO sessions (id, user_id, created_at, last_seen_at, expires_at, data)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&id)
            .bind(&user_id)
            .bind(&created_at_str)
            .bind(&created_at_str)
            .bind(&expires_at_str)
            .bind(&data_str)
            .execute(&pool)
            .await?;

            Ok(session)
        }
    }

    fn get(
        &self,
        id: &str,
    ) -> impl Future<Output = Result<Option<Session>, InfrastructureError>> + Send {
        let pool = self.pool.clone();
        let id = id.to_string();

        async move {
            let now = Utc::now();
            let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

            let row = sqlx::query(
                r#"
                SELECT id, user_id, created_at, last_seen_at, expires_at, data
                FROM sessions
                WHERE id = ? AND expires_at > ?
                "#,
            )
            .bind(&id)
            .bind(&now_str)
            .fetch_optional(&pool)
            .await?;

            match row {
                Some(row) => {
                    let session = parse_session_row(&row)?;
                    Ok(Some(session))
                }
                None => Ok(None),
            }
        }
    }

    fn update_data(
        &self,
        id: &str,
        data: serde_json::Value,
    ) -> impl Future<Output = Result<(), InfrastructureError>> + Send {
        let pool = self.pool.clone();
        let id = id.to_string();

        async move {
            let now = Utc::now();
            let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
            let data_str = data.to_string();

            sqlx::query(
                r#"
                UPDATE sessions
                SET data = ?, last_seen_at = ?
                WHERE id = ?
                "#,
            )
            .bind(&data_str)
            .bind(&now_str)
            .bind(&id)
            .execute(&pool)
            .await?;

            Ok(())
        }
    }

    fn touch(&self, id: &str) -> impl Future<Output = Result<(), InfrastructureError>> + Send {
        let pool = self.pool.clone();
        let id = id.to_string();

        async move {
            let now = Utc::now();
            let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

            sqlx::query(
                r#"
                UPDATE sessions
                SET last_seen_at = ?
                WHERE id = ?
                "#,
            )
            .bind(&now_str)
            .bind(&id)
            .execute(&pool)
            .await?;

            Ok(())
        }
    }

    fn delete(&self, id: &str) -> impl Future<Output = Result<(), InfrastructureError>> + Send {
        let pool = self.pool.clone();
        let id = id.to_string();

        async move {
            sqlx::query(
                r#"
                DELETE FROM sessions
                WHERE id = ?
                "#,
            )
            .bind(&id)
            .execute(&pool)
            .await?;

            Ok(())
        }
    }

    fn cleanup_expired(&self) -> impl Future<Output = Result<u64, InfrastructureError>> + Send {
        let pool = self.pool.clone();

        async move {
            let now = Utc::now();
            let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

            let result = sqlx::query(
                r#"
                DELETE FROM sessions
                WHERE expires_at <= ?
                "#,
            )
            .bind(&now_str)
            .execute(&pool)
            .await?;

            Ok(result.rows_affected())
        }
    }

    fn delete_user_sessions(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<u64, InfrastructureError>> + Send {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();

        async move {
            let result = sqlx::query(
                r#"
                DELETE FROM sessions
                WHERE user_id = ?
                "#,
            )
            .bind(&user_id)
            .execute(&pool)
            .await?;

            Ok(result.rows_affected())
        }
    }
}

/// Parse a SQLite row into a Session struct.
fn parse_session_row(row: &sqlx::sqlite::SqliteRow) -> Result<Session, InfrastructureError> {
    let id: String = row.get("id");
    let user_id: Option<String> = row.get("user_id");
    let created_at: String = row.get("created_at");
    let last_seen_at: String = row.get("last_seen_at");
    let expires_at: String = row.get("expires_at");
    let data: String = row.get("data");

    // Parse timestamps from SQLite TEXT format
    let created_at = parse_sqlite_datetime(&created_at)?;
    let last_seen_at = parse_sqlite_datetime(&last_seen_at)?;
    let expires_at = parse_sqlite_datetime(&expires_at)?;

    let data: serde_json::Value = serde_json::from_str(&data)?;

    Ok(Session {
        id,
        user_id,
        created_at,
        last_seen_at,
        expires_at,
        data,
    })
}

/// Parse SQLite datetime TEXT to chrono DateTime<Utc>.
fn parse_sqlite_datetime(s: &str) -> Result<DateTime<Utc>, InfrastructureError> {
    use chrono::NaiveDateTime;
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| InfrastructureError::database(format!("Invalid datetime format: {e}")))?;
    Ok(naive.and_utc())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("test pool");

        // Run migrations
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

    #[tokio::test]
    async fn create_and_get_session() {
        let pool = create_test_pool().await;
        let store = SqliteSessionStore::with_default_ttl(pool);

        let session = store.create(None).await.unwrap();
        assert_eq!(session.id.len(), 32); // 24 bytes base64 = 32 chars
        assert!(session.user_id.is_none());

        let fetched = store.get(&session.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, session.id);
    }

    #[tokio::test]
    async fn create_with_user_id() {
        let pool = create_test_pool().await;
        let store = SqliteSessionStore::with_default_ttl(pool);

        let session = store.create(Some("user-123")).await.unwrap();
        assert_eq!(session.user_id, Some("user-123".to_string()));
    }

    #[tokio::test]
    async fn get_expired_returns_none() {
        let pool = create_test_pool().await;
        // TTL of -1 day means session is already expired
        let store = SqliteSessionStore::new(pool, Duration::days(-1));

        let session = store.create(None).await.unwrap();
        let fetched = store.get(&session.id).await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn touch_updates_last_seen() {
        let pool = create_test_pool().await;
        let store = SqliteSessionStore::with_default_ttl(pool);

        let session = store.create(None).await.unwrap();
        let original_last_seen = session.last_seen_at;

        // Wait for at least 1 second since SQLite TEXT timestamps have second precision
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        store.touch(&session.id).await.unwrap();

        let fetched = store.get(&session.id).await.unwrap().unwrap();
        // With 1 second delay, last_seen should be strictly greater
        assert!(
            fetched.last_seen_at > original_last_seen,
            "last_seen_at should be updated after touch"
        );
    }

    #[tokio::test]
    async fn update_data() {
        let pool = create_test_pool().await;
        let store = SqliteSessionStore::with_default_ttl(pool);

        let session = store.create(None).await.unwrap();
        let new_data = serde_json::json!({"key": "value"});

        store
            .update_data(&session.id, new_data.clone())
            .await
            .unwrap();

        let fetched = store.get(&session.id).await.unwrap().unwrap();
        assert_eq!(fetched.data, new_data);
    }

    #[tokio::test]
    async fn delete_session() {
        let pool = create_test_pool().await;
        let store = SqliteSessionStore::with_default_ttl(pool);

        let session = store.create(None).await.unwrap();
        store.delete(&session.id).await.unwrap();

        let fetched = store.get(&session.id).await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn cleanup_expired() {
        let pool = create_test_pool().await;
        let store_expired = SqliteSessionStore::new(pool.clone(), Duration::days(-1));
        let store_valid = SqliteSessionStore::with_default_ttl(pool.clone());

        // Create expired and valid sessions
        store_expired.create(None).await.unwrap();
        store_expired.create(None).await.unwrap();
        store_valid.create(None).await.unwrap();

        let deleted = store_valid.cleanup_expired().await.unwrap();
        assert_eq!(deleted, 2);
    }

    #[tokio::test]
    async fn delete_user_sessions() {
        let pool = create_test_pool().await;
        let store = SqliteSessionStore::with_default_ttl(pool);

        store.create(Some("user-1")).await.unwrap();
        store.create(Some("user-1")).await.unwrap();
        store.create(Some("user-2")).await.unwrap();

        let deleted = store.delete_user_sessions("user-1").await.unwrap();
        assert_eq!(deleted, 2);
    }

    #[test]
    fn session_id_length() {
        let id = generate_session_id();
        assert_eq!(id.len(), 32); // 24 bytes -> 32 base64 chars
    }

    #[test]
    fn session_id_url_safe() {
        let id = generate_session_id();
        assert!(!id.contains('+'));
        assert!(!id.contains('/'));
        assert!(!id.contains('='));
    }
}
