//! DuckDB analytics service for OLAP queries.
//!
//! This module provides a service wrapper around async-duckdb's Pool for executing
//! analytics queries. The service handles the Option<Pool> pattern, returning
//! appropriate errors when analytics is unavailable.
//!
//! # Design
//!
//! DuckDBService wraps `Option<DuckDbPool>` to centralize availability checking.
//! Handlers extract `AnalyticsState` via axum's `FromRef` and call typed query methods.
//!
//! # Usage
//!
//! ```rust,ignore
//! async fn analytics_handler(
//!     State(analytics): State<AnalyticsState>,
//! ) -> Result<impl IntoResponse, AppError> {
//!     let result = analytics.service.query(|conn| {
//!         let mut stmt = conn.prepare("SELECT COUNT(*) FROM events")?;
//!         stmt.query_row([], |row| row.get::<_, i64>(0))
//!     }).await?;
//!
//!     Ok(Json(result))
//! }
//! ```
//!
//! # Error handling
//!
//! When analytics is unavailable (pool is None), methods return
//! `InfrastructureError::analytics("analytics service unavailable")` which maps
//! to HTTP 503 Service Unavailable.

use crate::infrastructure::InfrastructureError;
use crate::state::DuckDbPool;

// Re-export duckdb types from async_duckdb for public API consumers.
pub use async_duckdb::duckdb;

/// DuckDB analytics service wrapper.
///
/// Provides a clean async interface for analytics queries, handling the
/// optional nature of the DuckDB pool at the service layer.
#[derive(Clone)]
pub struct DuckDBService {
    pool: Option<DuckDbPool>,
}

impl DuckDBService {
    /// Create a new DuckDBService with an optional pool.
    ///
    /// When `pool` is `None`, all query methods will return service unavailable errors.
    #[must_use]
    pub fn new(pool: Option<DuckDbPool>) -> Self {
        Self { pool }
    }

    /// Check if analytics is available.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.pool.is_some()
    }

    /// Execute a read-only query against the analytics database.
    ///
    /// The closure receives an immutable reference to a DuckDB connection.
    /// Use this for SELECT queries and read operations.
    ///
    /// # Errors
    ///
    /// Returns `InfrastructureError` if:
    /// - Analytics service is unavailable (pool is None)
    /// - The query execution fails
    /// - The connection is closed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let count = service.query(|conn| {
    ///     let mut stmt = conn.prepare("SELECT COUNT(*) FROM events")?;
    ///     stmt.query_row([], |row| row.get::<_, i64>(0))
    /// }).await?;
    /// ```
    pub async fn query<F, T>(&self, func: F) -> Result<T, InfrastructureError>
    where
        F: FnOnce(&duckdb::Connection) -> Result<T, duckdb::Error> + Send + 'static,
        T: Send + 'static,
    {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| InfrastructureError::analytics("analytics service unavailable"))?;

        pool.conn(func)
            .await
            .map_err(|e| InfrastructureError::analytics(e.to_string()))
    }

    /// Execute a query that may modify the database.
    ///
    /// The closure receives a mutable reference to a DuckDB connection.
    /// Use this for DDL operations, COPY statements, or other mutations.
    ///
    /// Note: By default, async-duckdb opens connections in read-only mode.
    /// Mutable operations require appropriate pool configuration.
    ///
    /// # Errors
    ///
    /// Returns `InfrastructureError` if:
    /// - Analytics service is unavailable (pool is None)
    /// - The query execution fails
    /// - The connection is closed
    pub async fn query_mut<F, T>(&self, func: F) -> Result<T, InfrastructureError>
    where
        F: FnOnce(&mut duckdb::Connection) -> Result<T, duckdb::Error> + Send + 'static,
        T: Send + 'static,
    {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| InfrastructureError::analytics("analytics service unavailable"))?;

        pool.conn_mut(func)
            .await
            .map_err(|e| InfrastructureError::analytics(e.to_string()))
    }
}

/// State container for analytics handlers.
///
/// Extract this via axum's `State` extractor. Implements `FromRef<AppState>`
/// for automatic extraction from the application state.
#[derive(Clone)]
pub struct AnalyticsState {
    /// The DuckDB analytics service.
    pub service: DuckDBService,
}

impl AnalyticsState {
    /// Create a new AnalyticsState with the given service.
    #[must_use]
    pub fn new(service: DuckDBService) -> Self {
        Self { service }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_unavailable_when_no_pool() {
        let service = DuckDBService::new(None);
        assert!(!service.is_available());
    }

    #[tokio::test]
    async fn query_returns_error_when_unavailable() {
        let service = DuckDBService::new(None);
        let result: Result<i64, _> = service.query(|_conn| Ok(42)).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("analytics service unavailable"));
    }

    #[tokio::test]
    async fn query_mut_returns_error_when_unavailable() {
        let service = DuckDBService::new(None);
        let result: Result<(), _> = service.query_mut(|_conn| Ok(())).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("analytics service unavailable"));
    }

    // Integration tests with actual DuckDB pool would go in tests/
}
