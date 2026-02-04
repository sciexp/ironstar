//! Event-driven cache invalidation via Zenoh subscriptions.
//!
//! Subscribes to domain events on the Zenoh bus and invalidates analytics
//! cache entries whose [`CacheDependency`] patterns match the incoming key
//! expression.
//!
//! # Architecture
//!
//! The invalidation subscriber runs as a background tokio task spawned at
//! application startup.
//! It holds a registry of [`CacheDependency`] entries describing which cache
//! keys depend on which event streams.
//! When an event arrives, each dependency is tested via
//! [`CacheDependency::matches`] and matching cache entries are invalidated
//! by prefix.
//!
//! # Example
//!
//! ```rust,ignore
//! let registry = CacheInvalidationRegistry::new(cached_service.clone())
//!     .register(
//!         CacheDependency::new("embedded:space:0.1.0:astronauts")
//!             .depends_on_aggregate("Catalog")
//!     );
//!
//! spawn_cache_invalidation(event_bus.session().clone(), registry);
//! ```

use std::sync::Arc;

use zenoh::Session;

use ironstar_event_bus::ALL_EVENTS;
use ironstar_event_bus::CacheDependency;

use crate::cached_analytics::CachedAnalyticsService;

/// Registry of cache dependencies for event-driven invalidation.
///
/// Holds a [`CachedAnalyticsService`] and a list of [`CacheDependency`] entries.
/// When an event key expression matches a dependency, all cache entries with
/// the corresponding cache key prefix are invalidated.
#[derive(Clone)]
pub struct CacheInvalidationRegistry {
    cached_service: CachedAnalyticsService,
    dependencies: Vec<CacheDependency>,
}

impl CacheInvalidationRegistry {
    /// Create a new empty registry for the given cached analytics service.
    #[must_use]
    pub fn new(cached_service: CachedAnalyticsService) -> Self {
        Self {
            cached_service,
            dependencies: Vec::new(),
        }
    }

    /// Register a cache dependency for event-driven invalidation.
    #[must_use]
    pub fn register(mut self, dep: CacheDependency) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Register multiple cache dependencies at once.
    #[must_use]
    pub fn register_all(mut self, deps: impl IntoIterator<Item = CacheDependency>) -> Self {
        self.dependencies.extend(deps);
        self
    }

    /// Return the registered dependencies.
    #[must_use]
    pub fn dependencies(&self) -> &[CacheDependency] {
        &self.dependencies
    }

    /// Process an incoming event key expression, invalidating matching cache entries.
    ///
    /// Returns the number of cache dependencies that matched and triggered
    /// invalidation.
    pub fn process_event(&self, key_expr: &str) -> usize {
        let mut invalidated = 0;
        for dep in &self.dependencies {
            if dep.matches(key_expr) {
                tracing::debug!(
                    cache_key = dep.cache_key(),
                    event_key = key_expr,
                    "Invalidating cache entry"
                );
                self.cached_service.invalidate_for_prefix(dep.cache_key());
                invalidated += 1;
            }
        }
        invalidated
    }
}

/// Spawn a background task that subscribes to all domain events and
/// invalidates cache entries matching registered dependencies.
///
/// The task runs until the Zenoh session is closed or the tokio runtime
/// shuts down.
/// Subscription or processing errors are logged but do not cause the
/// task to terminate.
///
/// # Panics
///
/// Does not panic.
/// If the Zenoh subscriber cannot be created, the error is logged and
/// the task exits without crashing the application.
pub fn spawn_cache_invalidation(
    session: Arc<Session>,
    registry: CacheInvalidationRegistry,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let subscriber = match session.declare_subscriber(ALL_EVENTS).await {
            Ok(sub) => sub,
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "Failed to create cache invalidation subscriber, \
                     cache will not be automatically invalidated"
                );
                return;
            }
        };

        tracing::info!(
            pattern = ALL_EVENTS,
            dependency_count = registry.dependencies().len(),
            "Cache invalidation subscriber started"
        );

        loop {
            match subscriber.recv_async().await {
                Ok(sample) => {
                    let key_expr = sample.key_expr().as_str();
                    let count = registry.process_event(key_expr);
                    if count > 0 {
                        tracing::debug!(
                            key_expr = key_expr,
                            invalidated = count,
                            "Cache entries invalidated by event"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "Cache invalidation subscriber channel closed"
                    );
                    break;
                }
            }
        }

        tracing::info!("Cache invalidation subscriber stopped");
    })
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::analytics::DuckDBService;
    use crate::analytics_cache::AnalyticsCache;
    use std::time::Duration;

    fn test_registry() -> CacheInvalidationRegistry {
        let service = DuckDBService::new(None);
        let cache = AnalyticsCache::new();
        let cached = CachedAnalyticsService::new(service, cache);
        CacheInvalidationRegistry::new(cached)
    }

    #[test]
    fn empty_registry_matches_nothing() {
        let registry = test_registry();
        assert_eq!(registry.process_event("events/Todo/abc/1"), 0);
    }

    #[test]
    fn registry_matches_aggregate_dependency() {
        let registry = test_registry()
            .register(CacheDependency::new("dashboard:todo").depends_on_aggregate("Todo"));

        assert_eq!(registry.process_event("events/Todo/abc/1"), 1);
        assert_eq!(registry.process_event("events/Session/abc/1"), 0);
    }

    #[test]
    fn registry_matches_instance_dependency() {
        let registry = test_registry().register(
            CacheDependency::new("user:session:42").depends_on_instance("Session", "user-42"),
        );

        assert_eq!(registry.process_event("events/Session/user-42/3"), 1);
        assert_eq!(registry.process_event("events/Session/user-99/3"), 0);
    }

    #[test]
    fn registry_matches_multiple_dependencies() {
        let registry = test_registry()
            .register(CacheDependency::new("summary:all").depends_on_aggregate("Todo"))
            .register(CacheDependency::new("counts:todo").depends_on_aggregate("Todo"));

        // Both deps match Todo events.
        assert_eq!(registry.process_event("events/Todo/abc/1"), 2);
    }

    #[test]
    fn register_all_adds_multiple_at_once() {
        let deps = vec![
            CacheDependency::new("a").depends_on_aggregate("Todo"),
            CacheDependency::new("b").depends_on_aggregate("Session"),
        ];

        let registry = test_registry().register_all(deps);
        assert_eq!(registry.dependencies().len(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn spawn_invalidation_processes_events() {
        use ironstar_event_bus::open_embedded_session;

        let session = Arc::new(open_embedded_session().await.expect("session"));

        let pool = async_duckdb::PoolBuilder::new()
            .num_conns(1)
            .open()
            .await
            .expect("pool");
        let duckdb_service = DuckDBService::new(Some(pool.clone()));
        let cache = AnalyticsCache::new();
        let cached = CachedAnalyticsService::new(duckdb_service, cache.clone());

        // Pre-populate cache with an entry.
        let bytes = AnalyticsCache::serialize(&42u64).expect("serialize");
        cache.insert("todo:count:abc".to_string(), bytes).await;
        cache.run_pending_tasks().await;
        assert_eq!(cache.entry_count(), 1);

        // Register dependency: "todo:" prefix depends on Todo events.
        let registry = CacheInvalidationRegistry::new(cached)
            .register(CacheDependency::new("todo:").depends_on_aggregate("Todo"));

        let _handle = spawn_cache_invalidation(session.clone(), registry);

        // Give subscriber time to start.
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Publish a Todo event via Zenoh directly.
        session
            .put("events/Todo/abc/1", "event-payload")
            .await
            .expect("publish");

        // Allow invalidation to process.
        tokio::time::sleep(Duration::from_millis(100)).await;
        cache.run_pending_tasks().await;

        assert_eq!(cache.entry_count(), 0, "cache entry should be invalidated");

        pool.close().await.expect("close");
    }
}
