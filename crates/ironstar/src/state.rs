//! Application state shared across HTTP handlers.
//!
//! AppState is the central state container for the application. It holds all
//! infrastructure dependencies and is passed to handlers via axum's State extractor.
//!
//! # Design
//!
//! AppState uses optional fields for capabilities that may not be available:
//!
//! - `event_bus: Option<Arc<ZenohEventBus>>` — pub/sub for SSE feeds
//! - `session_store: Option<Arc<dyn SessionStore>>` — authentication sessions
//! - `analytics: Option<Arc<DuckDbPool>>` — OLAP queries
//!
//! This allows the application to start with reduced functionality when some
//! infrastructure is unavailable or disabled.
//!
//! # FromRef pattern
//!
//! Domain-specific states implement `FromRef<AppState>` to allow handlers to
//! extract only the dependencies they need:
//!
//! ```rust,ignore
//! async fn list_todos(State(todo_state): State<TodoAppState>) -> impl IntoResponse {
//!     // Handler receives only Todo-related state
//! }
//! ```

use crate::domain::todo::commands::TodoCommand;
use crate::domain::todo::events::TodoEvent;
use crate::infrastructure::{
    AssetManifest, SqliteEventRepository, SqliteSessionStore, ZenohEventBus,
};
use crate::presentation::health::HealthState;
use crate::presentation::todo::TodoAppState;
use axum::extract::FromRef;
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

// Re-export Session for convenience.
pub use crate::infrastructure::Session;

/// DuckDB pool wrapper for analytics queries.
///
/// This is a placeholder for the future analytics implementation.
/// Will wrap async-duckdb for OLAP query execution.
pub struct DuckDbPool {
    // Future: async-duckdb pool
}

/// Central application state container.
///
/// This struct holds all infrastructure dependencies needed by HTTP handlers.
/// Use `FromRef` implementations to extract domain-specific subsets of state.
#[derive(Clone)]
pub struct AppState {
    /// SQLite connection pool for event store and sessions.
    pub db_pool: SqlitePool,

    /// Asset manifest for static file resolution.
    pub assets: AssetManifest,

    /// Optional Zenoh event bus for pub/sub.
    ///
    /// When `None`, events are persisted but not published to subscribers.
    /// SSE feeds will still work via polling from the event store.
    pub event_bus: Option<Arc<ZenohEventBus>>,

    /// Optional session store for authentication.
    ///
    /// When `None`, authentication is disabled.
    pub session_store: Option<Arc<SqliteSessionStore>>,

    /// Optional DuckDB pool for analytics queries.
    ///
    /// When `None`, analytics endpoints return 503 Service Unavailable.
    pub analytics: Option<Arc<DuckDbPool>>,

    /// Shared Todo event repository.
    ///
    /// Cached here to avoid recreating for each request.
    todo_repo: Arc<SqliteEventRepository<TodoCommand, TodoEvent>>,
}

impl AppState {
    /// Create a new AppState with required dependencies.
    ///
    /// Optional capabilities (event_bus, session_store, analytics) can be set
    /// using the builder-style methods.
    #[must_use]
    pub fn new(db_pool: SqlitePool, assets: AssetManifest) -> Self {
        let todo_repo = Arc::new(SqliteEventRepository::new(db_pool.clone()));

        Self {
            db_pool,
            assets,
            event_bus: None,
            session_store: None,
            analytics: None,
            todo_repo,
        }
    }

    /// Set the Zenoh event bus.
    #[must_use]
    pub fn with_event_bus(mut self, event_bus: Arc<ZenohEventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Set the session store.
    #[must_use]
    pub fn with_session_store(mut self, session_store: Arc<SqliteSessionStore>) -> Self {
        self.session_store = Some(session_store);
        self
    }

    /// Set the analytics pool.
    #[must_use]
    pub fn with_analytics(mut self, analytics: Arc<DuckDbPool>) -> Self {
        self.analytics = Some(analytics);
        self
    }

    /// Check if the event bus is available.
    #[must_use]
    pub fn has_event_bus(&self) -> bool {
        self.event_bus.is_some()
    }

    /// Check if the session store is available.
    #[must_use]
    pub fn has_session_store(&self) -> bool {
        self.session_store.is_some()
    }

    /// Check if analytics is available.
    #[must_use]
    pub fn has_analytics(&self) -> bool {
        self.analytics.is_some()
    }
}

// =============================================================================
// FromRef implementations for domain-specific states
// =============================================================================

impl FromRef<AppState> for TodoAppState {
    fn from_ref(app_state: &AppState) -> Self {
        TodoAppState {
            repo: Arc::clone(&app_state.todo_repo),
            event_bus: app_state.event_bus.clone(),
        }
    }
}

impl FromRef<AppState> for HealthState {
    fn from_ref(app_state: &AppState) -> Self {
        HealthState {
            db_pool: app_state.db_pool.clone(),
        }
    }
}

impl FromRef<AppState> for AssetManifest {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.assets.clone()
    }
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.db_pool.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("test pool")
    }

    #[tokio::test]
    async fn app_state_creation() {
        let pool = create_test_pool().await;
        let assets = AssetManifest::default();

        let state = AppState::new(pool, assets);

        assert!(!state.has_event_bus());
        assert!(!state.has_session_store());
        assert!(!state.has_analytics());
    }

    #[tokio::test]
    async fn from_ref_todo_app_state() {
        let pool = create_test_pool().await;
        let assets = AssetManifest::default();
        let state = AppState::new(pool, assets);

        let todo_state: TodoAppState = TodoAppState::from_ref(&state);

        // TodoAppState should have None event_bus when AppState has None
        assert!(todo_state.event_bus.is_none());
    }

    #[tokio::test]
    async fn from_ref_health_state() {
        let pool = create_test_pool().await;
        let assets = AssetManifest::default();
        let state = AppState::new(pool, assets);

        let _health_state: HealthState = HealthState::from_ref(&state);
        // HealthState successfully extracted
    }
}
