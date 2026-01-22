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

    /// Validate that a name is a valid SQL identifier.
    ///
    /// Valid identifiers must:
    /// - Be non-empty
    /// - Start with a letter or underscore
    /// - Contain only alphanumeric characters and underscores
    fn is_valid_identifier(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let mut chars = name.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
            _ => return false,
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    /// Attach a DuckLake catalog from a ducklake: URI.
    ///
    /// Attaches a remote DuckLake catalog database, making its tables available
    /// for querying with the given alias. The attachment uses read-only mode
    /// since ducklake catalogs are versioned and immutable.
    ///
    /// # Arguments
    ///
    /// * `name` - The alias to use for the attached database (e.g., "space").
    ///   Must be a valid SQL identifier (alphanumeric + underscore, starting
    ///   with a letter or underscore).
    /// * `uri` - The ducklake URI (e.g., "ducklake:hf://datasets/sciexp/fixtures/lakes/frozen/space.db")
    ///
    /// # Errors
    ///
    /// Returns `InfrastructureError` if:
    /// - Analytics service is unavailable (pool is None)
    /// - The name is not a valid SQL identifier
    /// - The ATTACH statement fails (network error, invalid URI, auth failure)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// service.attach_catalog(
    ///     "space",
    ///     "ducklake:hf://datasets/sciexp/fixtures/lakes/frozen/space.db"
    /// ).await?;
    ///
    /// // Now query the attached catalog
    /// let astronauts = service.query(|conn| {
    ///     let mut stmt = conn.prepare(
    ///         "SELECT name, nationality FROM space.main.astronauts LIMIT 10"
    ///     )?;
    ///     // ... process results
    /// }).await?;
    /// ```
    pub async fn attach_catalog(&self, name: &str, uri: &str) -> Result<(), InfrastructureError> {
        // Validate name is a valid SQL identifier
        if !Self::is_valid_identifier(name) {
            return Err(InfrastructureError::analytics(format!(
                "invalid catalog name '{name}': must be a valid SQL identifier \
                 (alphanumeric + underscore, starting with letter or underscore)"
            )));
        }

        // Build and execute the ATTACH statement
        // Note: We use string formatting here because DuckDB's ATTACH doesn't support
        // parameterized queries for the URI or alias. The name is validated above.
        let sql = format!("ATTACH '{uri}' AS {name}");

        self.query_mut(move |conn| {
            conn.execute(&sql, [])?;
            Ok(())
        })
        .await
        .map_err(|e| {
            InfrastructureError::analytics(format!("failed to attach catalog '{name}': {e}"))
        })
    }

    /// Initialize DuckDB extensions on all pool connections.
    ///
    /// Installs httpfs and ducklake extensions (once, to ~/.duckdb/extensions/)
    /// and loads them on every connection in the pool. This enables:
    /// - `httpfs`: HTTP/HTTPS/S3 remote file access
    /// - `ducklake`: DuckLake catalog integration for versioned analytics
    ///
    /// # Errors
    ///
    /// Returns `InfrastructureError` if:
    /// - Analytics service is unavailable (pool is None)
    /// - Extension installation fails
    /// - Extension loading fails on any connection
    pub async fn initialize_extensions(&self) -> Result<(), InfrastructureError> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| InfrastructureError::analytics("analytics service unavailable"))?;

        // INSTALL runs once per extension (idempotent, writes to ~/.duckdb/extensions/)
        pool.conn(|conn| {
            conn.execute("INSTALL httpfs", [])?;
            conn.execute("INSTALL ducklake", [])?;
            Ok(())
        })
        .await
        .map_err(|e| InfrastructureError::analytics(format!("extension install failed: {e}")))?;

        // LOAD must run on every connection in the pool
        let results = pool
            .conn_for_each(|conn| {
                conn.execute("LOAD httpfs", [])?;
                conn.execute("LOAD ducklake", [])?;
                Ok(())
            })
            .await;

        // Check all connections loaded successfully
        for (i, result) in results.into_iter().enumerate() {
            result.map_err(|e| {
                InfrastructureError::analytics(format!(
                    "extension load failed on connection {i}: {e}"
                ))
            })?;
        }

        Ok(())
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

    #[tokio::test]
    async fn initialize_extensions_returns_error_when_unavailable() {
        let service = DuckDBService::new(None);
        let result = service.initialize_extensions().await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("analytics service unavailable"));
    }

    #[tokio::test]
    async fn initialize_extensions_succeeds_with_pool() {
        // Create an in-memory DuckDB pool for testing
        let pool = async_duckdb::PoolBuilder::new()
            .num_conns(2)
            .open()
            .await
            .expect("failed to create test pool");

        let service = DuckDBService::new(Some(pool.clone()));

        // Initialize extensions
        let result = service.initialize_extensions().await;
        assert!(result.is_ok(), "initialize_extensions failed: {result:?}");

        // Verify extensions are loaded by querying duckdb_extensions()
        let loaded_extensions: Vec<String> = service
            .query(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT extension_name FROM duckdb_extensions() \
                     WHERE extension_name IN ('httpfs', 'ducklake') AND loaded",
                )?;
                let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
                rows.collect::<Result<Vec<_>, _>>()
            })
            .await
            .expect("failed to query extensions");

        assert!(
            loaded_extensions.contains(&"httpfs".to_string()),
            "httpfs not loaded, found: {loaded_extensions:?}"
        );
        assert!(
            loaded_extensions.contains(&"ducklake".to_string()),
            "ducklake not loaded, found: {loaded_extensions:?}"
        );

        pool.close().await.expect("failed to close pool");
    }

    #[tokio::test]
    async fn initialize_extensions_loads_on_all_connections() {
        // Create pool with multiple connections to verify all get loaded
        let pool = async_duckdb::PoolBuilder::new()
            .num_conns(3)
            .open()
            .await
            .expect("failed to create test pool");

        let service = DuckDBService::new(Some(pool.clone()));
        service
            .initialize_extensions()
            .await
            .expect("initialize_extensions failed");

        // Query each connection multiple times to ensure round-robin hits all
        // The pool uses round-robin, so 3 queries on 3 connections covers all
        for _ in 0..3 {
            let count: i64 = service
                .query(|conn| {
                    let mut stmt = conn.prepare(
                        "SELECT COUNT(*) FROM duckdb_extensions() \
                         WHERE extension_name IN ('httpfs', 'ducklake') AND loaded",
                    )?;
                    stmt.query_row([], |row| row.get(0))
                })
                .await
                .expect("failed to count loaded extensions");

            assert_eq!(count, 2, "expected 2 extensions loaded on each connection");
        }

        pool.close().await.expect("failed to close pool");
    }

    #[tokio::test]
    async fn attach_catalog_returns_error_when_unavailable() {
        let service = DuckDBService::new(None);
        let result = service
            .attach_catalog("test", "ducklake:some/path.db")
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("analytics service unavailable"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn is_valid_identifier_accepts_valid_names() {
        // Valid identifiers
        assert!(DuckDBService::is_valid_identifier("space"));
        assert!(DuckDBService::is_valid_identifier("my_catalog"));
        assert!(DuckDBService::is_valid_identifier("_private"));
        assert!(DuckDBService::is_valid_identifier("Catalog1"));
        assert!(DuckDBService::is_valid_identifier("a"));
        assert!(DuckDBService::is_valid_identifier("_"));
        assert!(DuckDBService::is_valid_identifier("a1b2c3"));
        assert!(DuckDBService::is_valid_identifier("CamelCase"));
        assert!(DuckDBService::is_valid_identifier("snake_case_123"));
    }

    #[test]
    fn is_valid_identifier_rejects_invalid_names() {
        // Empty
        assert!(!DuckDBService::is_valid_identifier(""));

        // Starts with number
        assert!(!DuckDBService::is_valid_identifier("1catalog"));
        assert!(!DuckDBService::is_valid_identifier("123"));

        // Contains invalid characters
        assert!(!DuckDBService::is_valid_identifier("my-catalog"));
        assert!(!DuckDBService::is_valid_identifier("my catalog"));
        assert!(!DuckDBService::is_valid_identifier("my.catalog"));
        assert!(!DuckDBService::is_valid_identifier("catalog!"));
        assert!(!DuckDBService::is_valid_identifier("catalog@name"));

        // SQL injection attempts
        assert!(!DuckDBService::is_valid_identifier(
            "'; DROP TABLE users; --"
        ));
        assert!(!DuckDBService::is_valid_identifier("test; SELECT * FROM"));
    }

    #[tokio::test]
    async fn attach_catalog_rejects_invalid_identifier() {
        // Create a pool so we can test identifier validation (happens before query)
        let pool = async_duckdb::PoolBuilder::new()
            .num_conns(1)
            .open()
            .await
            .expect("failed to create test pool");

        let service = DuckDBService::new(Some(pool.clone()));

        // Test various invalid identifiers
        let invalid_names = ["1catalog", "my-catalog", "my catalog", "", "; DROP TABLE"];

        for name in invalid_names {
            let result = service.attach_catalog(name, "ducklake:some/path.db").await;
            assert!(result.is_err(), "expected error for invalid name '{name}'");
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("invalid catalog name"),
                "unexpected error for '{name}': {err}"
            );
        }

        pool.close().await.expect("failed to close pool");
    }
}
