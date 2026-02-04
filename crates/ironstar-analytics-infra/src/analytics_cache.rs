//! Moka analytics cache with rkyv serialization.
//!
//! Provides TTL-based caching for analytics query results, memoizing the query
//! profunctor via time-based invalidation.
//! Cached values are stored as rkyv-serialized bytes (`Vec<u8>`), enabling
//! zero-copy deserialization and compact memory representation.
//!
//! # Cache key composition
//!
//! Keys are composed from query identifiers, dataset references, and chart
//! configuration hashes.
//! Callers construct key strings; this module stores and retrieves opaque bytes.
//!
//! # TTL policy
//!
//! - `time_to_live`: 5 minutes from insertion
//! - `time_to_idle`: 60 seconds from last access
//!
//! Entries are evicted by whichever TTL expires first, ensuring both stale data
//! and unused entries are cleaned up.
//!
//! # Serialization
//!
//! Values are serialized to `Vec<u8>` via rkyv before insertion and deserialized
//! on retrieval.
//! The caller is responsible for ensuring types implement the required rkyv traits.
//! Helper methods `serialize` and `deserialize` encapsulate the rkyv API.

use crate::error::AnalyticsInfraError;
use moka::future::Cache;
use std::future::Future;
use std::time::Duration;

/// Default time-to-live for cache entries (5 minutes).
const DEFAULT_TTL: Duration = Duration::from_secs(300);

/// Default time-to-idle for cache entries (60 seconds).
const DEFAULT_TTI: Duration = Duration::from_secs(60);

/// Default maximum cache capacity (number of entries).
const DEFAULT_MAX_CAPACITY: u64 = 1_000;

/// Analytics cache wrapping `moka::future::Cache<String, Vec<u8>>`.
///
/// Stores rkyv-serialized query results with TTL-based eviction.
/// The cache uses `String` keys composed by callers and `Vec<u8>` values
/// containing rkyv-serialized data.
///
/// # Examples
///
/// ```rust,ignore
/// use ironstar_analytics_infra::AnalyticsCache;
///
/// let cache = AnalyticsCache::new();
///
/// // Serialize and insert
/// let bytes = AnalyticsCache::serialize(&my_value)?;
/// cache.insert("query:dataset:config_hash".into(), bytes).await;
///
/// // Retrieve and deserialize
/// if let Some(bytes) = cache.get("query:dataset:config_hash").await {
///     let value: MyType = AnalyticsCache::deserialize(&bytes)?;
/// }
/// ```
#[derive(Clone)]
pub struct AnalyticsCache {
    cache: Cache<String, Vec<u8>>,
}

impl AnalyticsCache {
    /// Create a new analytics cache with default TTL and capacity settings.
    ///
    /// Defaults:
    /// - `time_to_live`: 5 minutes
    /// - `time_to_idle`: 60 seconds
    /// - `max_capacity`: 1,000 entries
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(DEFAULT_MAX_CAPACITY, DEFAULT_TTL, DEFAULT_TTI)
    }

    /// Create a new analytics cache with custom configuration.
    ///
    /// Use this constructor when default TTL or capacity values are not appropriate,
    /// such as in tests requiring shorter TTL for deterministic expiration.
    #[must_use]
    pub fn with_config(max_capacity: u64, time_to_live: Duration, time_to_idle: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(time_to_live)
            .time_to_idle(time_to_idle)
            .support_invalidation_closures()
            .build();
        Self { cache }
    }

    /// Get a cached value by key.
    ///
    /// Returns the raw rkyv-serialized bytes if the key exists and has not expired.
    /// Use `deserialize` to convert the bytes back to a typed value.
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.cache.get(key).await
    }

    /// Insert a value into the cache.
    ///
    /// The value should be rkyv-serialized bytes produced by `serialize`.
    /// Overwrites any existing entry for the given key, resetting TTL timers.
    pub async fn insert(&self, key: String, value: Vec<u8>) {
        self.cache.insert(key, value).await;
    }

    /// Get a cached value or compute and insert it on cache miss.
    ///
    /// On cache hit, returns the cached bytes.
    /// On cache miss, calls the provided async closure to compute the value,
    /// inserts the result, and returns the bytes.
    ///
    /// # Errors
    ///
    /// Returns `AnalyticsInfraError::cache` if the computation closure fails.
    pub async fn get_or_insert_with<F, Fut>(
        &self,
        key: String,
        compute: F,
    ) -> Result<Vec<u8>, AnalyticsInfraError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<u8>, AnalyticsInfraError>>,
    {
        if let Some(cached) = self.cache.get(&key).await {
            return Ok(cached);
        }

        let value = compute().await?;
        self.cache.insert(key, value.clone()).await;
        Ok(value)
    }

    /// Invalidate a specific cache entry by key.
    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    /// Invalidate all cache entries whose keys match a predicate.
    ///
    /// This schedules invalidation of matching entries.
    /// Moka processes invalidation asynchronously during subsequent cache operations.
    pub fn invalidate_where<F>(&self, predicate: F)
    where
        F: Fn(&String, &Vec<u8>) -> bool + Send + Sync + 'static,
    {
        let _ = self
            .cache
            .invalidate_entries_if(move |k, v| predicate(k, v));
    }

    /// Return the current estimated entry count.
    #[must_use]
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Run pending maintenance tasks (eviction, expiration).
    ///
    /// Moka runs maintenance lazily during cache operations.
    /// Call this method to force immediate cleanup, which is useful in tests
    /// or when precise eviction timing is required.
    pub async fn run_pending_tasks(&self) {
        self.cache.run_pending_tasks().await;
    }

    /// Serialize a value to rkyv bytes.
    ///
    /// # Errors
    ///
    /// Returns `AnalyticsInfraError::serialization` if serialization fails.
    pub fn serialize<T>(value: &T) -> Result<Vec<u8>, AnalyticsInfraError>
    where
        T: for<'a> rkyv::Serialize<
                rkyv::api::high::HighSerializer<
                    rkyv::util::AlignedVec,
                    rkyv::ser::allocator::ArenaHandle<'a>,
                    rkyv::rancor::Error,
                >,
            >,
    {
        let aligned = rkyv::to_bytes::<rkyv::rancor::Error>(value).map_err(|e| {
            AnalyticsInfraError::serialization(format!("rkyv serialization failed: {e}"))
        })?;
        Ok(aligned.to_vec())
    }

    /// Deserialize a value from rkyv bytes.
    ///
    /// # Errors
    ///
    /// Returns `AnalyticsInfraError::serialization` if deserialization or validation fails.
    pub fn deserialize<T>(bytes: &[u8]) -> Result<T, AnalyticsInfraError>
    where
        T: rkyv::Archive,
        T::Archived: for<'a> rkyv::bytecheck::CheckBytes<
                rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>,
            > + rkyv::Deserialize<T, rkyv::rancor::Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
    {
        rkyv::from_bytes::<T, rkyv::rancor::Error>(bytes).map_err(|e| {
            AnalyticsInfraError::serialization(format!("rkyv deserialization failed: {e}"))
        })
    }
}

impl Default for AnalyticsCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic, reason = "test assertions")]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    #[rkyv(compare(PartialEq))]
    struct TestResult {
        count: u64,
        label: String,
    }

    #[test]
    fn cache_creation_with_defaults() {
        let cache = AnalyticsCache::new();
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn cache_creation_with_custom_config() {
        let cache =
            AnalyticsCache::with_config(500, Duration::from_secs(60), Duration::from_secs(10));
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let value = TestResult {
            count: 42,
            label: "test query".to_string(),
        };

        let bytes = AnalyticsCache::serialize(&value).expect("serialization failed");
        assert!(!bytes.is_empty());

        let deserialized: TestResult =
            AnalyticsCache::deserialize(&bytes).expect("deserialization failed");
        assert_eq!(deserialized, value);
    }

    #[test]
    fn deserialize_invalid_bytes_returns_error() {
        let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
        let result = AnalyticsCache::deserialize::<TestResult>(&invalid_bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("rkyv deserialization failed"),
            "unexpected error message: {err}"
        );
    }

    #[tokio::test]
    async fn insert_and_get_roundtrip() {
        let cache = AnalyticsCache::new();
        let value = TestResult {
            count: 7,
            label: "cached".to_string(),
        };

        let bytes = AnalyticsCache::serialize(&value).expect("serialization failed");
        cache.insert("key-1".to_string(), bytes).await;

        let retrieved = cache.get("key-1").await.expect("entry not found");
        let deserialized: TestResult =
            AnalyticsCache::deserialize(&retrieved).expect("deserialization failed");
        assert_eq!(deserialized, value);
    }

    #[tokio::test]
    async fn get_nonexistent_key_returns_none() {
        let cache = AnalyticsCache::new();
        assert!(cache.get("missing").await.is_none());
    }

    #[tokio::test]
    async fn invalidate_removes_entry() {
        let cache = AnalyticsCache::new();
        let bytes = AnalyticsCache::serialize(&TestResult {
            count: 1,
            label: "remove me".to_string(),
        })
        .expect("serialization failed");

        cache.insert("to-remove".to_string(), bytes).await;
        assert!(cache.get("to-remove").await.is_some());

        cache.invalidate("to-remove").await;
        assert!(cache.get("to-remove").await.is_none());
    }

    #[tokio::test]
    async fn invalidate_where_removes_matching_entries() {
        let cache = AnalyticsCache::new();

        let bytes1 = AnalyticsCache::serialize(&TestResult {
            count: 1,
            label: "a".to_string(),
        })
        .expect("serialization failed");
        let bytes2 = AnalyticsCache::serialize(&TestResult {
            count: 2,
            label: "b".to_string(),
        })
        .expect("serialization failed");

        cache.insert("dataset:alpha:1".to_string(), bytes1).await;
        cache.insert("dataset:beta:2".to_string(), bytes2).await;

        // Invalidate entries with keys starting with "dataset:alpha"
        cache.invalidate_where(|k, _v| k.starts_with("dataset:alpha"));
        cache.run_pending_tasks().await;

        assert!(cache.get("dataset:alpha:1").await.is_none());
        assert!(cache.get("dataset:beta:2").await.is_some());
    }

    #[tokio::test]
    async fn get_or_insert_with_returns_cached_on_hit() {
        let cache = AnalyticsCache::new();
        let value = TestResult {
            count: 99,
            label: "original".to_string(),
        };
        let bytes = AnalyticsCache::serialize(&value).expect("serialization failed");
        cache.insert("hit-key".to_string(), bytes).await;

        // The compute closure should not be called on cache hit.
        let result = cache
            .get_or_insert_with("hit-key".to_string(), || async {
                panic!("compute closure should not be called on cache hit")
            })
            .await
            .expect("get_or_insert_with failed");

        let deserialized: TestResult =
            AnalyticsCache::deserialize(&result).expect("deserialization failed");
        assert_eq!(deserialized, value);
    }

    #[tokio::test]
    async fn get_or_insert_with_computes_on_miss() {
        let cache = AnalyticsCache::new();
        let value = TestResult {
            count: 42,
            label: "computed".to_string(),
        };

        let result = cache
            .get_or_insert_with("miss-key".to_string(), || {
                let v = value.clone();
                async move { AnalyticsCache::serialize(&v) }
            })
            .await
            .expect("get_or_insert_with failed");

        let deserialized: TestResult =
            AnalyticsCache::deserialize(&result).expect("deserialization failed");
        assert_eq!(deserialized, value);

        // Verify it was cached
        assert!(cache.get("miss-key").await.is_some());
    }

    #[tokio::test]
    async fn get_or_insert_with_propagates_compute_error() {
        let cache = AnalyticsCache::new();

        let result = cache
            .get_or_insert_with("err-key".to_string(), || async {
                Err(AnalyticsInfraError::cache("compute failed"))
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("compute failed"),
            "unexpected error: {err}"
        );

        // Verify nothing was cached
        assert!(cache.get("err-key").await.is_none());
    }

    #[tokio::test]
    async fn entry_count_tracks_insertions() {
        let cache = AnalyticsCache::new();
        assert_eq!(cache.entry_count(), 0);

        let bytes = AnalyticsCache::serialize(&TestResult {
            count: 1,
            label: "a".to_string(),
        })
        .expect("serialization failed");

        cache.insert("k1".to_string(), bytes.clone()).await;
        cache.insert("k2".to_string(), bytes).await;
        cache.run_pending_tasks().await;
        assert_eq!(cache.entry_count(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn ttl_expiration() {
        // Use real time with short TTL for expiration testing.
        // moka uses std::time::Instant internally, requiring real elapsed time.
        let cache =
            AnalyticsCache::with_config(100, Duration::from_millis(200), Duration::from_secs(60));

        let bytes = AnalyticsCache::serialize(&TestResult {
            count: 1,
            label: "expiring".to_string(),
        })
        .expect("serialization failed");

        cache.insert("ttl-key".to_string(), bytes).await;
        assert!(cache.get("ttl-key").await.is_some());

        // Wait past time_to_live using real time
        tokio::time::sleep(Duration::from_millis(350)).await;
        cache.run_pending_tasks().await;

        assert!(cache.get("ttl-key").await.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn tti_expiration() {
        // Use real time with short TTI for expiration testing.
        // moka uses std::time::Instant internally, requiring real elapsed time.
        let cache =
            AnalyticsCache::with_config(100, Duration::from_secs(60), Duration::from_millis(200));

        let bytes = AnalyticsCache::serialize(&TestResult {
            count: 1,
            label: "idle".to_string(),
        })
        .expect("serialization failed");

        cache.insert("tti-key".to_string(), bytes).await;
        assert!(cache.get("tti-key").await.is_some());

        // Wait past time_to_idle without accessing
        tokio::time::sleep(Duration::from_millis(350)).await;
        cache.run_pending_tasks().await;

        assert!(cache.get("tti-key").await.is_none());
    }
}
