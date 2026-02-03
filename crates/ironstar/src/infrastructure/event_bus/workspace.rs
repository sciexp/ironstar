//! Workspace bounded context event publishing and subscription.
//!
//! This module provides workspace-specific Zenoh integration following the pattern
//! established in ironstar-a9b.9. It covers four aggregate types within the Workspace
//! bounded context:
//!
//! - **Workspace**: Core workspace lifecycle (create, rename, visibility)
//! - **Dashboard**: Chart placements, tabs, and layout within a workspace
//! - **SavedQuery**: User-saved SQL queries scoped to workspaces
//! - **UserPreferences**: Per-user theme, locale, and UI state
//!
//! # Key expression routing
//!
//! Each aggregate type publishes to its own key expression namespace using the
//! standard `events/{aggregate_type}/{aggregate_id}` schema from [`key_expr`](super::super::key_expr).
//! Subscribers use type-level patterns to receive relevant events:
//!
//! | Pattern | Matches |
//! |---------|---------|
//! | `events/Workspace/**` | All workspace lifecycle events |
//! | `events/Dashboard/**` | All dashboard layout events |
//! | `events/SavedQuery/**` | All saved query events |
//! | `events/UserPreferences/**` | All user preferences events |
//!
//! # Subscriber factory
//!
//! [`WorkspaceSubscriberFactory`] creates typed Zenoh subscribers for each aggregate
//! type, following the subscribe-before-replay invariant required for SSE feeds.
//!
//! # Cache invalidation
//!
//! [`workspace_cache_dependencies`] registers cache invalidation hooks so that
//! workspace-related cache entries are invalidated when aggregate events arrive.

use crate::infrastructure::cache_dependency::CacheDependency;
use crate::infrastructure::error::InfrastructureError;
use crate::infrastructure::key_expr::aggregate_type_pattern;
use std::sync::Arc;
use zenoh::Session;

/// Aggregate type identifier for Workspace events.
pub const WORKSPACE_TYPE: &str = "Workspace";

/// Aggregate type identifier for Dashboard events.
pub const DASHBOARD_TYPE: &str = "Dashboard";

/// Aggregate type identifier for SavedQuery events.
pub const SAVED_QUERY_TYPE: &str = "SavedQuery";

/// Aggregate type identifier for UserPreferences events.
pub const USER_PREFERENCES_TYPE: &str = "UserPreferences";

/// All aggregate types in the Workspace bounded context.
pub const ALL_WORKSPACE_AGGREGATE_TYPES: [&str; 4] = [
    WORKSPACE_TYPE,
    DASHBOARD_TYPE,
    SAVED_QUERY_TYPE,
    USER_PREFERENCES_TYPE,
];

/// Key expression pattern for all Workspace lifecycle events.
pub fn workspace_events_pattern() -> String {
    aggregate_type_pattern(WORKSPACE_TYPE)
}

/// Key expression pattern for all Dashboard layout events.
pub fn dashboard_events_pattern() -> String {
    aggregate_type_pattern(DASHBOARD_TYPE)
}

/// Key expression pattern for all SavedQuery events.
pub fn saved_query_events_pattern() -> String {
    aggregate_type_pattern(SAVED_QUERY_TYPE)
}

/// Key expression pattern for all UserPreferences events.
pub fn user_preferences_events_pattern() -> String {
    aggregate_type_pattern(USER_PREFERENCES_TYPE)
}

/// Zenoh subscriber type alias for readability.
pub type ZenohSubscriber =
    zenoh::pubsub::Subscriber<zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>>;

/// Factory for creating typed Zenoh subscribers for Workspace bounded context aggregates.
///
/// Holds a reference to the Zenoh session and provides methods to create subscribers
/// for each aggregate type. Subscribers should be created before loading historical
/// events to satisfy the subscribe-before-replay invariant.
///
/// # Example
///
/// ```rust,ignore
/// let factory = WorkspaceSubscriberFactory::new(Arc::clone(&zenoh_session));
///
/// // Subscribe to workspace events for SSE feed
/// let subscriber = factory.subscribe_workspace().await?;
///
/// // Load historical events...
/// // Build SSE stream with zenoh_to_sse_stream(subscriber, converter)
/// ```
#[derive(Clone)]
pub struct WorkspaceSubscriberFactory {
    session: Arc<Session>,
}

impl WorkspaceSubscriberFactory {
    /// Create a new subscriber factory wrapping the given Zenoh session.
    #[must_use]
    pub fn new(session: Arc<Session>) -> Self {
        Self { session }
    }

    /// Get a reference to the underlying Zenoh session.
    #[must_use]
    pub fn session(&self) -> &Arc<Session> {
        &self.session
    }

    /// Subscribe to all Workspace lifecycle events.
    ///
    /// Pattern: `events/Workspace/**`
    pub async fn subscribe_workspace(&self) -> Result<ZenohSubscriber, InfrastructureError> {
        self.subscribe_aggregate(WORKSPACE_TYPE).await
    }

    /// Subscribe to all Dashboard layout events.
    ///
    /// Pattern: `events/Dashboard/**`
    pub async fn subscribe_dashboard(&self) -> Result<ZenohSubscriber, InfrastructureError> {
        self.subscribe_aggregate(DASHBOARD_TYPE).await
    }

    /// Subscribe to all SavedQuery events.
    ///
    /// Pattern: `events/SavedQuery/**`
    pub async fn subscribe_saved_query(&self) -> Result<ZenohSubscriber, InfrastructureError> {
        self.subscribe_aggregate(SAVED_QUERY_TYPE).await
    }

    /// Subscribe to all UserPreferences events.
    ///
    /// Pattern: `events/UserPreferences/**`
    pub async fn subscribe_user_preferences(&self) -> Result<ZenohSubscriber, InfrastructureError> {
        self.subscribe_aggregate(USER_PREFERENCES_TYPE).await
    }

    /// Subscribe to events for all aggregate types in the Workspace bounded context.
    ///
    /// Returns one subscriber per aggregate type. Useful for creating a unified
    /// workspace SSE feed that streams all workspace-related events.
    pub async fn subscribe_all(
        &self,
    ) -> Result<Vec<(&'static str, ZenohSubscriber)>, InfrastructureError> {
        let mut subscribers = Vec::with_capacity(ALL_WORKSPACE_AGGREGATE_TYPES.len());
        for &aggregate_type in &ALL_WORKSPACE_AGGREGATE_TYPES {
            let subscriber = self.subscribe_aggregate(aggregate_type).await?;
            subscribers.push((aggregate_type, subscriber));
        }
        Ok(subscribers)
    }

    /// Subscribe to events for a specific aggregate type.
    async fn subscribe_aggregate(
        &self,
        aggregate_type: &str,
    ) -> Result<ZenohSubscriber, InfrastructureError> {
        let pattern = aggregate_type_pattern(aggregate_type);
        self.session
            .declare_subscriber(&pattern)
            .await
            .map_err(|e| {
                InfrastructureError::event_bus(format!(
                    "failed to create subscriber for {aggregate_type}: {e}"
                ))
            })
    }
}

/// Create cache invalidation dependencies for Workspace bounded context events.
///
/// Returns a set of `CacheDependency` entries that map workspace-related cache keys
/// to the aggregate event streams they depend on. Register these with a
/// `CacheInvalidationRegistry` to enable automatic invalidation when workspace
/// events are published.
///
/// # Cache key conventions
///
/// | Cache key prefix | Invalidated by | Use case |
/// |-----------------|----------------|----------|
/// | `workspace:list` | Workspace events | Workspace list view |
/// | `dashboard:layout` | Dashboard events | Dashboard layout projections |
/// | `saved_query:list` | SavedQuery events | Saved query list view |
/// | `user_preferences` | UserPreferences events | User preferences view |
///
/// # Example
///
/// ```rust,ignore
/// let deps = workspace_cache_dependencies();
/// let registry = CacheInvalidationRegistry::new(cached_service)
///     .register_all(deps);
/// spawn_cache_invalidation(session, registry);
/// ```
#[must_use]
pub fn workspace_cache_dependencies() -> Vec<CacheDependency> {
    vec![
        CacheDependency::new("workspace:list").depends_on_aggregate(WORKSPACE_TYPE),
        CacheDependency::new("dashboard:layout").depends_on_aggregate(DASHBOARD_TYPE),
        CacheDependency::new("saved_query:list").depends_on_aggregate(SAVED_QUERY_TYPE),
        CacheDependency::new("user_preferences").depends_on_aggregate(USER_PREFERENCES_TYPE),
    ]
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::traits::{DeciderType, Identifier};
    use crate::infrastructure::event_bus::{ZenohEventBus, open_embedded_session};
    use crate::infrastructure::key_expr::aggregate_type_pattern;
    use std::time::Duration;

    // Minimal test event for verifying pub/sub routing
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    struct MockWorkspaceEvent {
        id: String,
        aggregate: String,
    }

    impl Identifier for MockWorkspaceEvent {
        fn identifier(&self) -> String {
            self.id.clone()
        }
    }

    impl DeciderType for MockWorkspaceEvent {
        fn decider_type(&self) -> String {
            self.aggregate.clone()
        }
    }

    #[test]
    fn aggregate_type_constants_match_domain_decider_types() {
        assert_eq!(WORKSPACE_TYPE, "Workspace");
        assert_eq!(DASHBOARD_TYPE, "Dashboard");
        assert_eq!(SAVED_QUERY_TYPE, "SavedQuery");
        assert_eq!(USER_PREFERENCES_TYPE, "UserPreferences");
    }

    #[test]
    fn all_workspace_aggregate_types_contains_four_types() {
        assert_eq!(ALL_WORKSPACE_AGGREGATE_TYPES.len(), 4);
        assert!(ALL_WORKSPACE_AGGREGATE_TYPES.contains(&"Workspace"));
        assert!(ALL_WORKSPACE_AGGREGATE_TYPES.contains(&"Dashboard"));
        assert!(ALL_WORKSPACE_AGGREGATE_TYPES.contains(&"SavedQuery"));
        assert!(ALL_WORKSPACE_AGGREGATE_TYPES.contains(&"UserPreferences"));
    }

    #[test]
    fn key_expression_patterns_follow_schema() {
        assert_eq!(workspace_events_pattern(), "events/Workspace/**");
        assert_eq!(dashboard_events_pattern(), "events/Dashboard/**");
        assert_eq!(saved_query_events_pattern(), "events/SavedQuery/**");
        assert_eq!(
            user_preferences_events_pattern(),
            "events/UserPreferences/**"
        );
    }

    #[test]
    fn workspace_cache_dependencies_covers_all_aggregate_types() {
        let deps = workspace_cache_dependencies();
        assert_eq!(deps.len(), 4);

        let cache_keys: Vec<&str> = deps.iter().map(|d| d.cache_key()).collect();
        assert!(cache_keys.contains(&"workspace:list"));
        assert!(cache_keys.contains(&"dashboard:layout"));
        assert!(cache_keys.contains(&"saved_query:list"));
        assert!(cache_keys.contains(&"user_preferences"));
    }

    #[test]
    fn cache_dependency_matches_correct_aggregate_events() {
        let deps = workspace_cache_dependencies();

        // Workspace dependency matches workspace events
        let ws_dep = deps
            .iter()
            .find(|d| d.cache_key() == "workspace:list")
            .unwrap();
        assert!(ws_dep.matches("events/Workspace/some-id"));
        assert!(ws_dep.matches("events/Workspace/some-id/1"));
        assert!(!ws_dep.matches("events/Dashboard/some-id"));

        // Dashboard dependency matches dashboard events
        let db_dep = deps
            .iter()
            .find(|d| d.cache_key() == "dashboard:layout")
            .unwrap();
        assert!(db_dep.matches("events/Dashboard/dashboard_abc"));
        assert!(!db_dep.matches("events/Workspace/some-id"));

        // SavedQuery dependency
        let sq_dep = deps
            .iter()
            .find(|d| d.cache_key() == "saved_query:list")
            .unwrap();
        assert!(sq_dep.matches("events/SavedQuery/saved_query_abc"));
        assert!(!sq_dep.matches("events/Dashboard/abc"));

        // UserPreferences dependency
        let up_dep = deps
            .iter()
            .find(|d| d.cache_key() == "user_preferences")
            .unwrap();
        assert!(up_dep.matches("events/UserPreferences/user_abc/preferences"));
        assert!(!up_dep.matches("events/Workspace/abc"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn subscriber_factory_creates_workspace_subscriber() {
        let session = Arc::new(open_embedded_session().await.expect("session should open"));
        let factory = WorkspaceSubscriberFactory::new(Arc::clone(&session));

        let subscriber = factory
            .subscribe_workspace()
            .await
            .expect("subscriber should be created");

        // Publish a workspace event via the event bus
        let event_bus = ZenohEventBus::new(Arc::clone(&session));
        let event = MockWorkspaceEvent {
            id: "ws-1".to_string(),
            aggregate: "Workspace".to_string(),
        };

        crate::infrastructure::event_bus::EventBus::publish(&event_bus, &event)
            .await
            .expect("publish should succeed");

        let sample = tokio::time::timeout(Duration::from_millis(100), subscriber.recv_async())
            .await
            .expect("should receive within timeout")
            .expect("recv should succeed");

        assert_eq!(sample.key_expr().as_str(), "events/Workspace/ws-1");

        let received: MockWorkspaceEvent =
            serde_json::from_slice(&sample.payload().to_bytes()).expect("should deserialize");
        assert_eq!(received, event);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn subscriber_factory_filters_by_aggregate_type() {
        let session = Arc::new(open_embedded_session().await.expect("session should open"));
        let factory = WorkspaceSubscriberFactory::new(Arc::clone(&session));

        // Subscribe only to Dashboard events
        let subscriber = factory
            .subscribe_dashboard()
            .await
            .expect("subscriber should be created");

        let event_bus = ZenohEventBus::new(Arc::clone(&session));

        // Publish a Workspace event (should NOT be received)
        let ws_event = MockWorkspaceEvent {
            id: "ws-1".to_string(),
            aggregate: "Workspace".to_string(),
        };
        crate::infrastructure::event_bus::EventBus::publish(&event_bus, &ws_event)
            .await
            .expect("publish should succeed");

        // Publish a Dashboard event (should be received)
        let db_event = MockWorkspaceEvent {
            id: "dashboard_db-1".to_string(),
            aggregate: "Dashboard".to_string(),
        };
        crate::infrastructure::event_bus::EventBus::publish(&event_bus, &db_event)
            .await
            .expect("publish should succeed");

        let sample = tokio::time::timeout(Duration::from_millis(100), subscriber.recv_async())
            .await
            .expect("should receive within timeout")
            .expect("recv should succeed");

        // Should receive the Dashboard event, not the Workspace event
        let received: MockWorkspaceEvent =
            serde_json::from_slice(&sample.payload().to_bytes()).expect("should deserialize");
        assert_eq!(received.aggregate, "Dashboard");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn subscribe_all_creates_subscribers_for_all_types() {
        let session = Arc::new(open_embedded_session().await.expect("session should open"));
        let factory = WorkspaceSubscriberFactory::new(Arc::clone(&session));

        let subscribers = factory
            .subscribe_all()
            .await
            .expect("all subscribers should be created");

        assert_eq!(subscribers.len(), 4);

        let types: Vec<&str> = subscribers.iter().map(|(t, _)| *t).collect();
        assert!(types.contains(&"Workspace"));
        assert!(types.contains(&"Dashboard"));
        assert!(types.contains(&"SavedQuery"));
        assert!(types.contains(&"UserPreferences"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn subscriber_factory_session_accessor() {
        let session = Arc::new(open_embedded_session().await.expect("session should open"));
        let factory = WorkspaceSubscriberFactory::new(Arc::clone(&session));

        // Direct session access for custom subscriptions
        let custom_sub = factory
            .session()
            .declare_subscriber(aggregate_type_pattern("Workspace"))
            .await
            .expect("custom subscriber should be created");

        drop(custom_sub);
    }
}
