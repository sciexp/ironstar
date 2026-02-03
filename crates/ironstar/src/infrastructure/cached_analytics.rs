//! Cache-aside pattern for DuckDB analytics queries.
//!
//! Composes [`DuckDBService`] and [`AnalyticsCache`] into a single service that
//! transparently caches query results using rkyv serialization.
//! On cache hit, the stored bytes are deserialized directly without executing
//! the DuckDB query.
//! On cache miss, the query runs, results are serialized and cached, then
//! returned to the caller.
//!
//! # Cache key composition
//!
//! Cache keys are structured as `{prefix}:{query_hash:x}` where:
//! - `prefix` identifies the query context (e.g., `embedded:space:0.1.0:astronauts`)
//! - `query_hash` is a 64-bit hash of the query parameters (hex-encoded)
//!
//! Use [`embedded_cache_key_prefix`](super::embedded_catalogs::embedded_cache_key_prefix)
//! for embedded catalogs or construct prefixes manually for runtime sources.
//!
//! # Invalidation
//!
//! Cache entries are invalidated by prefix using [`invalidate_for_aggregate`],
//! which removes all entries whose keys start with a given prefix.
//! This integrates with the Zenoh-based event-driven invalidation in 3gd.2.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::infrastructure::analytics::DuckDBService;
use crate::infrastructure::analytics_cache::AnalyticsCache;
use crate::infrastructure::error::InfrastructureError;

/// Analytics service with transparent cache-aside caching.
///
/// Wraps a DuckDB query service and an in-memory cache to provide typed,
/// cached analytics queries.
/// The cache uses rkyv for zero-copy serialization of query results.
#[derive(Clone)]
pub struct CachedAnalyticsService {
    service: DuckDBService,
    cache: AnalyticsCache,
}

impl CachedAnalyticsService {
    /// Create a new cached analytics service.
    #[must_use]
    pub fn new(service: DuckDBService, cache: AnalyticsCache) -> Self {
        Self { service, cache }
    }

    /// Access the underlying DuckDB service for uncached queries.
    #[must_use]
    pub fn service(&self) -> &DuckDBService {
        &self.service
    }

    /// Access the underlying cache for direct manipulation.
    #[must_use]
    pub fn cache(&self) -> &AnalyticsCache {
        &self.cache
    }

    /// Execute a cached analytics query.
    ///
    /// Checks the cache for `key` first.
    /// On hit, deserializes and returns the cached result.
    /// On miss, executes `query_fn` against DuckDB, serializes the result
    /// into the cache, and returns it.
    ///
    /// # Type requirements
    ///
    /// `T` must implement rkyv's `Archive`, `Serialize`, and `Deserialize`
    /// traits, plus `bytecheck::CheckBytes` for safe deserialization.
    ///
    /// # Errors
    ///
    /// Returns `InfrastructureError` if:
    /// - The DuckDB query fails (analytics or connection error)
    /// - Serialization or deserialization fails (rkyv error)
    pub async fn query_cached<F, T>(&self, key: &str, query_fn: F) -> Result<T, InfrastructureError>
    where
        F: FnOnce(&async_duckdb::duckdb::Connection) -> Result<T, async_duckdb::duckdb::Error>
            + Send
            + 'static,
        T: Send
            + 'static
            + for<'a> rkyv::Serialize<
                rkyv::api::high::HighSerializer<
                    rkyv::util::AlignedVec,
                    rkyv::ser::allocator::ArenaHandle<'a>,
                    rkyv::rancor::Error,
                >,
            >
            + rkyv::Archive,
        T::Archived: for<'a> rkyv::bytecheck::CheckBytes<
                rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>,
            > + rkyv::Deserialize<T, rkyv::rancor::Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
    {
        // Cache hit: deserialize and return.
        if let Some(bytes) = self.cache.get(key).await {
            return AnalyticsCache::deserialize::<T>(&bytes);
        }

        // Cache miss: execute query, serialize, cache, return.
        let result = self.service.query(query_fn).await?;
        let bytes = AnalyticsCache::serialize(&result)?;
        self.cache.insert(key.to_string(), bytes).await;
        Ok(result)
    }

    /// Invalidate all cache entries whose keys start with the given prefix.
    ///
    /// Used for aggregate-level invalidation when events arrive via Zenoh.
    /// For example, invalidating prefix `"embedded:space"` removes all
    /// cached queries for the `space` catalog.
    pub fn invalidate_for_prefix(&self, prefix: &str) {
        let prefix = prefix.to_string();
        self.cache
            .invalidate_where(move |k, _v| k.starts_with(&prefix));
    }

    /// Return the current estimated cache entry count.
    #[must_use]
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }
}

/// Compute a query hash from hashable parameters.
///
/// Produces a deterministic 64-bit hash suitable for cache key composition.
/// Combine with a prefix to form the full cache key:
///
/// ```rust,ignore
/// let hash = query_hash(&("SELECT COUNT(*)", "astronauts"));
/// let key = format!("{prefix}:{hash:x}");
/// ```
#[must_use]
pub fn query_hash(params: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    params.hash(&mut hasher);
    hasher.finish()
}

/// Compose a full cache key from a prefix and hashable query parameters.
///
/// Convenience function combining [`query_hash`] with string formatting.
///
/// ```rust,ignore
/// let key = cache_key("embedded:space:0.1.0:astronauts", &("nationality", "American"));
/// // "embedded:space:0.1.0:astronauts:a1b2c3d4e5f6"
/// ```
#[must_use]
pub fn cache_key(prefix: &str, params: &impl Hash) -> String {
    format!("{prefix}:{:x}", query_hash(params))
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Hash)]
    #[rkyv(compare(PartialEq))]
    struct QueryResult {
        count: u64,
        name: String,
    }

    fn test_cache() -> AnalyticsCache {
        AnalyticsCache::with_config(100, Duration::from_secs(300), Duration::from_secs(60))
    }

    #[test]
    fn query_hash_is_deterministic() {
        let h1 = query_hash(&("SELECT *", "astronauts"));
        let h2 = query_hash(&("SELECT *", "astronauts"));
        assert_eq!(h1, h2);
    }

    #[test]
    fn query_hash_differs_for_different_params() {
        let h1 = query_hash(&("SELECT *", "astronauts"));
        let h2 = query_hash(&("SELECT *", "missions"));
        assert_ne!(h1, h2);
    }

    #[test]
    fn cache_key_includes_prefix_and_hash() {
        let key = cache_key("embedded:space:0.1.0:astronauts", &"test");
        assert!(key.starts_with("embedded:space:0.1.0:astronauts:"));
        assert!(key.len() > "embedded:space:0.1.0:astronauts:".len());
    }

    #[tokio::test]
    async fn query_cached_returns_result_on_miss() {
        let pool = async_duckdb::PoolBuilder::new()
            .num_conns(1)
            .open()
            .await
            .expect("pool");
        let service = DuckDBService::new(Some(pool.clone()));
        let cached = CachedAnalyticsService::new(service, test_cache());

        let result: QueryResult = cached
            .query_cached("test:miss", |conn| {
                let mut stmt = conn.prepare("SELECT 42 AS count, 'hello' AS name")?;
                stmt.query_row([], |row| {
                    Ok(QueryResult {
                        count: u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
                        name: row.get(1)?,
                    })
                })
            })
            .await
            .expect("query_cached failed");

        assert_eq!(result.count, 42);
        assert_eq!(result.name, "hello");

        // Verify it was cached.
        cached.cache().run_pending_tasks().await;
        assert_eq!(cached.entry_count(), 1);

        pool.close().await.expect("close");
    }

    #[tokio::test]
    async fn query_cached_returns_cached_on_hit() {
        let pool = async_duckdb::PoolBuilder::new()
            .num_conns(1)
            .open()
            .await
            .expect("pool");
        let service = DuckDBService::new(Some(pool.clone()));
        let cached = CachedAnalyticsService::new(service, test_cache());

        // Prime the cache.
        let _: QueryResult = cached
            .query_cached("test:hit", |conn| {
                let mut stmt = conn.prepare("SELECT 1 AS count, 'first' AS name")?;
                stmt.query_row([], |row| {
                    Ok(QueryResult {
                        count: u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
                        name: row.get(1)?,
                    })
                })
            })
            .await
            .expect("first query_cached failed");

        // Second call should return cached result, not execute the query.
        // If the query ran, it would return 'second' instead of 'first'.
        let result: QueryResult = cached
            .query_cached("test:hit", |conn| {
                let mut stmt = conn.prepare("SELECT 2 AS count, 'second' AS name")?;
                stmt.query_row([], |row| {
                    Ok(QueryResult {
                        count: u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
                        name: row.get(1)?,
                    })
                })
            })
            .await
            .expect("second query_cached failed");

        assert_eq!(result.count, 1);
        assert_eq!(result.name, "first");

        pool.close().await.expect("close");
    }

    #[tokio::test]
    async fn invalidate_for_prefix_clears_matching_entries() {
        let pool = async_duckdb::PoolBuilder::new()
            .num_conns(1)
            .open()
            .await
            .expect("pool");
        let service = DuckDBService::new(Some(pool.clone()));
        let cached = CachedAnalyticsService::new(service, test_cache());

        // Insert entries with different prefixes.
        let _: QueryResult = cached
            .query_cached("space:astronauts:abc", |conn| {
                conn.prepare("SELECT 1, 'a'")?.query_row([], |row| {
                    Ok(QueryResult {
                        count: u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
                        name: row.get(1)?,
                    })
                })
            })
            .await
            .expect("query failed");

        let _: QueryResult = cached
            .query_cached("other:missions:xyz", |conn| {
                conn.prepare("SELECT 2, 'b'")?.query_row([], |row| {
                    Ok(QueryResult {
                        count: u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
                        name: row.get(1)?,
                    })
                })
            })
            .await
            .expect("query failed");

        cached.cache().run_pending_tasks().await;
        assert_eq!(cached.entry_count(), 2);

        // Invalidate only space-prefixed entries.
        cached.invalidate_for_prefix("space:");
        cached.cache().run_pending_tasks().await;

        assert_eq!(cached.entry_count(), 1);

        pool.close().await.expect("close");
    }

    #[tokio::test]
    async fn query_cached_returns_error_when_unavailable() {
        let service = DuckDBService::new(None);
        let cached = CachedAnalyticsService::new(service, test_cache());

        let result: Result<QueryResult, _> = cached
            .query_cached("test:unavailable", |conn| {
                conn.prepare("SELECT 1, 'x'")?.query_row([], |row| {
                    Ok(QueryResult {
                        count: u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
                        name: row.get(1)?,
                    })
                })
            })
            .await;

        assert!(result.is_err());
    }
}
