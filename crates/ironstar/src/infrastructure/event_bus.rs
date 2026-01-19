//! Event bus abstraction and Zenoh implementation for post-persist event publishing.
//!
//! The EventBus trait provides a publish-only interface for broadcasting events after
//! they have been successfully persisted to the event store. This follows the CQRS
//! pattern where the event store is the source of truth and the event bus is used
//! only for notification.
//!
//! # Key design decisions
//!
//! - **Fire-and-forget**: Publish errors are logged but do not fail the command.
//!   The event store is the source of truth; event bus failures are recoverable
//!   via replay from the event store.
//!
//! - **Post-persist only**: Events are published AFTER successful persistence,
//!   never before. This ensures subscribers never receive events that weren't
//!   durably stored.
//!
//! - **Key expressions**: Zenoh uses hierarchical key expressions for routing:
//!   `events/{aggregate_type}/{aggregate_id}`. Subscribers use wildcards like
//!   `events/Todo/**` to receive all Todo events.
//!
//! # Usage pattern
//!
//! ```rust,ignore
//! // After EventSourcedAggregate::handle() succeeds
//! for (event, _version) in &saved_events {
//!     if let Err(e) = event_bus.publish(event).await {
//!         tracing::warn!(error = %e, "Failed to publish event to bus");
//!     }
//! }
//! ```

use crate::domain::traits::{DeciderType, Identifier};
use crate::infrastructure::error::InfrastructureError;
use crate::infrastructure::key_expr::event_key_without_sequence;
use serde::Serialize;
use std::future::Future;
use std::sync::Arc;
use tracing::warn;
use zenoh::Session;

/// Event bus trait for publishing domain events after persistence.
///
/// Implementations are expected to be fire-and-forget: callers should log
/// failures but not propagate them as command errors.
///
/// # Type constraints
///
/// Events must implement:
/// - `Identifier`: Provides the aggregate ID for key expression routing
/// - `DeciderType`: Provides the aggregate type for key expression routing
/// - `Serialize`: Enables JSON payload serialization
///
/// # Design: publish-only interface
///
/// This trait intentionally omits a `subscribe` method. Zenoh's `Subscriber` type
/// has its lifecycle tied to its `Drop` implementation — when dropped, the subscription
/// ends. This ownership model doesn't translate cleanly to a trait abstraction where
/// the returned subscriber would need to be a concrete type or boxed trait object.
///
/// Instead, implementations expose their underlying session (e.g., [`ZenohEventBus::session`])
/// for direct subscription access. This design:
/// - Preserves Zenoh's zero-copy efficiency and type-safe key expressions
/// - Allows subscribers to leverage Zenoh-specific features like key expression wildcards
/// - Keeps the trait focused on the common publish interface across implementations
///
/// For subscription patterns, see the [`key_expr`](super::key_expr) module which provides
/// helper functions for constructing subscription patterns.
pub trait EventBus: Send + Sync {
    /// Publish an event to the event bus.
    ///
    /// Returns `Ok(())` on success, or an error that should be logged but
    /// not propagated to fail the command.
    fn publish<E>(&self, event: &E) -> impl Future<Output = Result<(), InfrastructureError>> + Send
    where
        E: Identifier + DeciderType + Serialize + Sync;
}

/// Zenoh-based event bus using key expression routing.
///
/// Events are published to key expressions of the form:
/// `events/{aggregate_type}/{aggregate_id}`
///
/// Subscribers can use wildcards to filter:
/// - `events/Todo/**` - All Todo events
/// - `events/Todo/abc-123` - Events for specific Todo
/// - `events/**` - All events (global audit log)
#[derive(Clone)]
pub struct ZenohEventBus {
    session: Arc<Session>,
}

impl ZenohEventBus {
    /// Create a new Zenoh event bus with the given session.
    ///
    /// Use `zenoh_embedded_config()` to create a session configured for
    /// embedded (in-process) mode with no network communication.
    #[must_use]
    pub fn new(session: Arc<Session>) -> Self {
        Self { session }
    }

    /// Get a reference to the underlying Zenoh session.
    ///
    /// This method exposes the Zenoh session for creating subscribers and other
    /// Zenoh-specific operations that don't fit the [`EventBus`] trait abstraction.
    ///
    /// # Why subscription is via session, not the trait
    ///
    /// Zenoh's `Subscriber` type has its lifecycle tied to `Drop` — the subscription
    /// automatically ends when the subscriber is dropped. This ownership model requires
    /// callers to hold the `Subscriber` value, which doesn't fit cleanly into a trait
    /// method that would need to return an abstract type.
    ///
    /// Direct session access provides:
    /// - Full access to Zenoh's key expression wildcards (`events/Todo/**`)
    /// - Zero-copy message handling via Zenoh's `Sample` type
    /// - Natural Rust ownership semantics for subscription lifecycle
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::infrastructure::{ZenohEventBus, aggregate_type_pattern};
    ///
    /// let event_bus = ZenohEventBus::new(session);
    ///
    /// // Subscribe to all Todo events using key expression pattern
    /// let subscriber = event_bus.session()
    ///     .declare_subscriber(aggregate_type_pattern("Todo"))
    ///     .await?;
    ///
    /// // Subscriber lives until dropped
    /// while let Ok(sample) = subscriber.recv_async().await {
    ///     // Process event...
    /// }
    /// ```
    ///
    /// See [`aggregate_type_pattern`](super::key_expr::aggregate_type_pattern) and
    /// [`aggregate_instance_pattern`](super::key_expr::aggregate_instance_pattern)
    /// for subscription pattern helpers.
    #[must_use]
    pub fn session(&self) -> &Arc<Session> {
        &self.session
    }
}

impl EventBus for ZenohEventBus {
    async fn publish<E>(&self, event: &E) -> Result<(), InfrastructureError>
    where
        E: Identifier + DeciderType + Serialize + Sync,
    {
        let aggregate_type = event.decider_type();
        let aggregate_id = event.identifier();
        let key_expr = event_key_without_sequence(&aggregate_type, &aggregate_id);

        let payload = serde_json::to_vec(event)?;

        self.session
            .put(&key_expr, payload)
            .await
            .map_err(|e| InfrastructureError::event_bus(e.to_string()))?;

        Ok(())
    }
}

/// Create a Zenoh configuration for embedded (in-process) mode.
///
/// This configuration disables all network communication:
/// - No listening for incoming connections
/// - No connecting to remote endpoints
/// - No multicast or gossip discovery
///
/// Zenoh runs entirely in-process, suitable for single-binary deployments.
/// When multi-node distribution is needed, configure endpoints and scouting
/// via environment variables or configuration files.
///
/// # Panics
///
/// Panics if JSON configuration insertion fails, which indicates a bug
/// in the configuration keys.
#[allow(clippy::expect_used)] // Panic on invalid config keys is documented programming bug
#[must_use]
pub fn zenoh_embedded_config() -> zenoh::Config {
    let mut config = zenoh::Config::default();

    // Disable network communication for embedded mode
    config
        .insert_json5("listen/endpoints", "[]")
        .expect("valid config key: listen/endpoints");
    config
        .insert_json5("connect/endpoints", "[]")
        .expect("valid config key: connect/endpoints");
    config
        .insert_json5("scouting/multicast/enabled", "false")
        .expect("valid config key: scouting/multicast/enabled");
    config
        .insert_json5("scouting/gossip/enabled", "false")
        .expect("valid config key: scouting/gossip/enabled");

    config
}

/// Open a Zenoh session in embedded mode.
///
/// This is a convenience function that creates the embedded configuration
/// and opens a session. For production use, consider creating the session
/// once at application startup and sharing it via `Arc<Session>`.
///
/// # Errors
///
/// Returns an error if the Zenoh session fails to open.
pub async fn open_embedded_session() -> Result<Session, InfrastructureError> {
    let config = zenoh_embedded_config();
    zenoh::open(config)
        .await
        .map_err(|e| InfrastructureError::event_bus(format!("failed to open zenoh session: {e}")))
}

/// Publish events to the event bus with fire-and-forget semantics.
///
/// This helper function iterates over saved events and publishes each to the
/// event bus. Errors are logged but not propagated, following the fire-and-forget
/// pattern where event bus failures do not fail commands.
///
/// # Arguments
///
/// * `event_bus` - The event bus to publish to
/// * `events` - Iterator of (event, version) tuples from the event store
pub async fn publish_events_fire_and_forget<'a, E, B>(
    event_bus: &B,
    events: impl IntoIterator<Item = &'a (E, String)>,
) where
    E: Identifier + DeciderType + Serialize + Sync + 'a,
    B: EventBus,
{
    for (event, _version) in events {
        if let Err(e) = event_bus.publish(event).await {
            warn!(
                error = %e,
                aggregate_type = %event.decider_type(),
                aggregate_id = %event.identifier(),
                "Failed to publish event to bus"
            );
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::traits::{DeciderType, Identifier};
    use std::time::Duration;

    // Test event type
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    struct TestEvent {
        id: String,
        data: String,
    }

    impl Identifier for TestEvent {
        fn identifier(&self) -> String {
            self.id.clone()
        }
    }

    impl DeciderType for TestEvent {
        fn decider_type(&self) -> String {
            "Test".to_string()
        }
    }

    // Zenoh requires multi-threaded runtime for its internal task scheduling.
    // Use `flavor = "multi_thread"` with `worker_threads = 1` for minimal overhead.

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn zenoh_embedded_config_creates_valid_config() {
        let config = zenoh_embedded_config();
        // Config should be usable to open a session
        let session = zenoh::open(config).await;
        assert!(
            session.is_ok(),
            "Should create valid session from embedded config"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn publish_and_subscribe_roundtrip() {
        let session = Arc::new(open_embedded_session().await.expect("session should open"));
        let event_bus = ZenohEventBus::new(Arc::clone(&session));

        // Set up subscriber before publishing
        let subscriber = session
            .declare_subscriber("events/Test/**")
            .await
            .expect("subscriber should be created");

        let test_event = TestEvent {
            id: "test-123".to_string(),
            data: "hello world".to_string(),
        };

        // Publish event
        event_bus
            .publish(&test_event)
            .await
            .expect("publish should succeed");

        // Receive with timeout
        let sample = tokio::time::timeout(Duration::from_millis(100), subscriber.recv_async())
            .await
            .expect("should receive within timeout")
            .expect("recv should succeed");

        // Verify key expression
        assert_eq!(sample.key_expr().as_str(), "events/Test/test-123");

        // Verify payload deserializes correctly
        let received: TestEvent =
            serde_json::from_slice(&sample.payload().to_bytes()).expect("should deserialize");
        assert_eq!(received, test_event);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn key_expression_filtering_works() {
        let session = Arc::new(open_embedded_session().await.expect("session should open"));
        let event_bus = ZenohEventBus::new(Arc::clone(&session));

        // Subscribe only to Todo events, not Test events
        let subscriber = session
            .declare_subscriber("events/Todo/**")
            .await
            .expect("subscriber should be created");

        let test_event = TestEvent {
            id: "test-456".to_string(),
            data: "should not receive".to_string(),
        };

        // Publish to Test aggregate (not Todo)
        event_bus
            .publish(&test_event)
            .await
            .expect("publish should succeed");

        // Should timeout because subscriber is filtering for Todo, not Test
        let result = tokio::time::timeout(Duration::from_millis(50), subscriber.recv_async()).await;

        assert!(result.is_err(), "Should timeout - event was filtered out");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn fire_and_forget_logs_errors_but_continues() {
        // This test verifies the publish_events_fire_and_forget function
        // doesn't panic on errors and processes all events
        let session = Arc::new(open_embedded_session().await.expect("session should open"));
        let event_bus = ZenohEventBus::new(session);

        let events = vec![
            (
                TestEvent {
                    id: "a".to_string(),
                    data: "first".to_string(),
                },
                "v1".to_string(),
            ),
            (
                TestEvent {
                    id: "b".to_string(),
                    data: "second".to_string(),
                },
                "v2".to_string(),
            ),
        ];

        // Should complete without panicking
        publish_events_fire_and_forget(&event_bus, &events).await;
    }
}
