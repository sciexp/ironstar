//! Declarative cache invalidation via Zenoh key expression dependencies.
//!
//! Cache entries in the analytics layer declare which aggregate event streams
//! they depend on using Zenoh key expression patterns.
//! When an event arrives on the Zenoh bus, the cache invalidation subscriber
//! checks each cached entry's dependencies to determine whether invalidation
//! is required.
//!
//! # Pattern semantics
//!
//! Dependencies use simplified Zenoh key expression matching:
//!
//! | Pattern | Matches | Use case |
//! |---------|---------|----------|
//! | `events/Todo/**` | All Todo events | Aggregate-type dependency |
//! | `events/Todo/abc-123/*` | Events for one instance | Instance-level dependency |
//! | `events/Todo/abc-123/5` | One specific event | Exact match |
//!
//! # Example
//!
//! ```rust,ignore
//! let dep = CacheDependency::new("dashboard:summary")
//!     .depends_on_aggregate("Todo")
//!     .depends_on_instance("Session", "user-42");
//!
//! // When an event arrives, check if this cache entry should be invalidated.
//! let should_invalidate = dep.matches("events/Todo/abc-123/5");
//! ```

use crate::infrastructure::key_expr::EVENTS_ROOT;

/// Maps a cache key to the Zenoh key expression patterns it depends on.
///
/// When any event matching one of the dependency patterns is published,
/// the associated cache entry should be invalidated.
#[derive(Debug, Clone)]
pub struct CacheDependency {
    /// Cache key identifying the cached entry.
    pub cache_key: String,
    /// Zenoh key expression patterns this cache entry depends on.
    pub depends_on: Vec<String>,
}

impl CacheDependency {
    /// Create a new cache dependency for the given cache key with no dependencies.
    #[must_use]
    pub fn new(cache_key: impl Into<String>) -> Self {
        Self {
            cache_key: cache_key.into(),
            depends_on: Vec::new(),
        }
    }

    /// Add a dependency on all events for an aggregate type.
    ///
    /// Adds pattern `events/{aggregate_type}/**` which matches any event
    /// published for any instance of the given aggregate type.
    #[must_use]
    pub fn depends_on_aggregate(mut self, aggregate_type: &str) -> Self {
        let pattern = format!("{EVENTS_ROOT}/{aggregate_type}/**");
        self.depends_on.push(pattern);
        self
    }

    /// Add a dependency on events for a specific aggregate instance.
    ///
    /// Adds pattern `events/{aggregate_type}/{id}/*` which matches events
    /// at any sequence number for the given instance, but not sub-aggregates.
    #[must_use]
    pub fn depends_on_instance(mut self, aggregate_type: &str, id: &str) -> Self {
        let pattern = format!("{EVENTS_ROOT}/{aggregate_type}/{id}/*");
        self.depends_on.push(pattern);
        self
    }

    /// Check whether any dependency pattern matches the given key expression.
    ///
    /// Returns `true` if the cache entry should be invalidated in response
    /// to an event published on `key_expr`.
    #[must_use]
    pub fn matches(&self, key_expr: &str) -> bool {
        self.depends_on
            .iter()
            .any(|pattern| matches_key_expression(pattern, key_expr))
    }
}

/// Test whether a Zenoh-style key expression pattern matches a concrete key.
///
/// Supports three matching modes:
///
/// - `/**` suffix: matches zero or more path segments after the prefix.
/// - `/*` suffix: matches exactly one additional path segment (no further `/` separators).
/// - Otherwise: exact string equality.
///
/// This is a simplified subset of Zenoh's key expression matching, sufficient
/// for cache invalidation routing where patterns are constructed by the builder
/// methods on [`CacheDependency`].
#[must_use]
pub fn matches_key_expression(pattern: &str, key: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix("/**") {
        // Double wild: key must start with prefix, then either end or continue with '/'.
        key == prefix || key.strip_prefix(prefix).is_some_and(|rest| rest.starts_with('/'))
    } else if let Some(prefix) = pattern.strip_suffix("/*") {
        // Single wild: key must have exactly one more segment after prefix.
        key.strip_prefix(prefix)
            .and_then(|rest| rest.strip_prefix('/'))
            .is_some_and(|remainder| !remainder.is_empty() && !remainder.contains('/'))
    } else {
        // Exact match.
        pattern == key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- CacheDependency construction and builder --

    #[test]
    fn new_creates_empty_dependency() {
        let dep = CacheDependency::new("my-key");
        assert_eq!(dep.cache_key, "my-key");
        assert!(dep.depends_on.is_empty());
    }

    #[test]
    fn depends_on_aggregate_adds_double_wild_pattern() {
        let dep = CacheDependency::new("k").depends_on_aggregate("Todo");
        assert_eq!(dep.depends_on, vec!["events/Todo/**"]);
    }

    #[test]
    fn depends_on_instance_adds_single_wild_pattern() {
        let dep = CacheDependency::new("k").depends_on_instance("Todo", "abc-123");
        assert_eq!(dep.depends_on, vec!["events/Todo/abc-123/*"]);
    }

    #[test]
    fn chaining_multiple_dependencies() {
        let dep = CacheDependency::new("dashboard:summary")
            .depends_on_aggregate("Todo")
            .depends_on_instance("Session", "user-42")
            .depends_on_aggregate("Workspace");

        assert_eq!(dep.depends_on.len(), 3);
        assert_eq!(dep.depends_on[0], "events/Todo/**");
        assert_eq!(dep.depends_on[1], "events/Session/user-42/*");
        assert_eq!(dep.depends_on[2], "events/Workspace/**");
    }

    // -- matches_key_expression: double wild --

    #[test]
    fn double_wild_matches_nested_key() {
        assert!(matches_key_expression(
            "events/Todo/**",
            "events/Todo/abc/5"
        ));
    }

    #[test]
    fn double_wild_matches_immediate_child() {
        assert!(matches_key_expression("events/Todo/**", "events/Todo/abc"));
    }

    #[test]
    fn double_wild_matches_prefix_exactly() {
        // Zero additional segments — the prefix itself should match.
        assert!(matches_key_expression("events/Todo/**", "events/Todo"));
    }

    #[test]
    fn double_wild_rejects_different_aggregate() {
        assert!(!matches_key_expression(
            "events/Todo/**",
            "events/Session/abc/1"
        ));
    }

    #[test]
    fn double_wild_rejects_partial_prefix() {
        assert!(!matches_key_expression(
            "events/Todo/**",
            "events/TodoItem/abc/1"
        ));
    }

    // -- matches_key_expression: single wild --

    #[test]
    fn single_wild_matches_one_segment() {
        assert!(matches_key_expression(
            "events/Todo/abc-123/*",
            "events/Todo/abc-123/5"
        ));
    }

    #[test]
    fn single_wild_matches_non_numeric_segment() {
        assert!(matches_key_expression(
            "events/Todo/abc-123/*",
            "events/Todo/abc-123/latest"
        ));
    }

    #[test]
    fn single_wild_rejects_zero_segments() {
        assert!(!matches_key_expression(
            "events/Todo/abc-123/*",
            "events/Todo/abc-123"
        ));
    }

    #[test]
    fn single_wild_rejects_two_segments() {
        assert!(!matches_key_expression(
            "events/Todo/abc-123/*",
            "events/Todo/abc-123/5/extra"
        ));
    }

    #[test]
    fn single_wild_rejects_different_instance() {
        assert!(!matches_key_expression(
            "events/Todo/abc-123/*",
            "events/Todo/xyz-789/5"
        ));
    }

    // -- matches_key_expression: exact match --

    #[test]
    fn exact_match_succeeds() {
        assert!(matches_key_expression(
            "events/Todo/abc-123/5",
            "events/Todo/abc-123/5"
        ));
    }

    #[test]
    fn exact_match_rejects_different_key() {
        assert!(!matches_key_expression(
            "events/Todo/abc-123/5",
            "events/Todo/abc-123/6"
        ));
    }

    // -- CacheDependency::matches --

    #[test]
    fn matches_returns_true_when_any_pattern_matches() {
        let dep = CacheDependency::new("k")
            .depends_on_aggregate("Todo")
            .depends_on_instance("Session", "user-42");

        assert!(dep.matches("events/Todo/abc/1"));
        assert!(dep.matches("events/Session/user-42/3"));
    }

    #[test]
    fn matches_returns_false_when_no_pattern_matches() {
        let dep = CacheDependency::new("k").depends_on_aggregate("Todo");

        assert!(!dep.matches("events/Session/abc/1"));
    }

    #[test]
    fn matches_returns_false_for_empty_dependencies() {
        let dep = CacheDependency::new("k");
        assert!(!dep.matches("events/Todo/abc/1"));
    }
}
