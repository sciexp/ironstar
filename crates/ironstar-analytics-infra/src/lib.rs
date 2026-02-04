//! DuckDB analytics, moka caching, and cache invalidation infrastructure for ironstar.
//!
//! This crate provides analytics query execution via `DuckDBService`, in-memory
//! caching with `AnalyticsCache` (moka + rkyv), composed `CachedAnalyticsService`,
//! event-driven cache invalidation via Zenoh subscriptions, and embedded DuckLake
//! catalog management.

pub mod analytics;
pub mod analytics_cache;
pub mod cache_invalidation;
pub mod cached_analytics;
pub mod embedded_catalogs;
pub mod error;

pub use analytics::{AnalyticsState, DuckDBService, DuckDbPool};
pub use analytics_cache::AnalyticsCache;
pub use cache_invalidation::{CacheInvalidationRegistry, spawn_cache_invalidation};
pub use cached_analytics::{CachedAnalyticsService, cache_key, query_hash};
pub use embedded_catalogs::{DuckLakeCatalogs, embedded_cache_key_prefix};
pub use error::{AnalyticsInfraError, AnalyticsInfraErrorKind};
