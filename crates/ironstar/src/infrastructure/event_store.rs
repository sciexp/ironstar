//! SQLite event store implementation for fmodel-rust.
//!
//! Implements the `EventRepository` trait from fmodel-rust for SQLite, enabling
//! event sourcing with optimistic locking via previous_id chain.
//!
//! # Schema
//!
//! See `migrations/001_events.sql` for the event store schema. Key features:
//! - Global monotonic `id` for SSE Last-Event-ID semantics
//! - `previous_id` chain for optimistic locking (first event has NULL)
//! - Triggers enforce immutability and chain integrity
//!
//! # Extension methods
//!
//! Beyond the `EventRepository` trait, this module provides:
//! - `query_all()` — projection rebuild on startup
//! - `query_since_sequence(since)` — SSE reconnection via Last-Event-ID
//! - `earliest_sequence()` / `latest_sequence()` — stream bounds

use crate::domain::traits::{DeciderType, EventType, Identifier, IsFinal};
use crate::infrastructure::error::InfrastructureError;
use fmodel_rust::aggregate::EventRepository;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::marker::PhantomData;
use uuid::Uuid;

/// Stored event with global sequence for SSE streaming.
#[derive(Debug, Clone)]
pub struct StoredEvent<E> {
    /// Global monotonic sequence (SSE Last-Event-ID)
    pub sequence: i64,
    /// Event UUID (version for optimistic locking)
    pub event_id: String,
    /// Aggregate type
    pub aggregate_type: String,
    /// Aggregate identifier
    pub aggregate_id: String,
    /// Event type name
    pub event_type: String,
    /// Schema version for upcasting
    pub schema_version: i64,
    /// Deserialized event payload
    pub event: E,
    /// Command that caused this event
    pub command_id: Option<String>,
    /// Whether this event finalizes the aggregate
    pub is_final: bool,
    /// Event creation timestamp (ISO 8601)
    pub created_at: String,
}

/// SQLite event repository implementing fmodel-rust's EventRepository trait.
///
/// Generic over command and event types to support multiple aggregates.
/// The repository uses the `Identifier`, `DeciderType`, `EventType`, and `IsFinal`
/// traits to extract routing and metadata from domain types.
///
/// # Type parameters
///
/// - `C`: Command type implementing `Identifier + DeciderType`
/// - `E`: Event type implementing `Identifier + EventType + DeciderType + IsFinal + Serialize + DeserializeOwned`
#[derive(Debug, Clone)]
pub struct SqliteEventRepository<C, E> {
    pool: SqlitePool,
    _phantom: PhantomData<(C, E)>,
}

impl<C, E> SqliteEventRepository<C, E> {
    /// Create a new event repository with the given connection pool.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            _phantom: PhantomData,
        }
    }

    /// Get a reference to the connection pool.
    #[must_use]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

impl<C, E> SqliteEventRepository<C, E>
where
    E: DeserializeOwned + Clone,
{
    /// Query all events across all aggregates, ordered by global sequence.
    ///
    /// Used for projection rebuild on application startup.
    pub async fn query_all(&self) -> Result<Vec<StoredEvent<E>>, InfrastructureError> {
        let rows = sqlx::query(
            r#"
            SELECT id, event_id, aggregate_type, aggregate_id, event_type,
                   schema_version, payload, command_id, final, created_at
            FROM events
            ORDER BY id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let payload: String = row.get("payload");
            let event: E = serde_json::from_str(&payload)?;
            events.push(StoredEvent {
                sequence: row.get("id"),
                event_id: row.get("event_id"),
                aggregate_type: row.get("aggregate_type"),
                aggregate_id: row.get("aggregate_id"),
                event_type: row.get("event_type"),
                schema_version: row.get("schema_version"),
                event,
                command_id: row.get("command_id"),
                is_final: row.get::<i64, _>("final") != 0,
                created_at: row.get("created_at"),
            });
        }
        Ok(events)
    }

    /// Query events since a global sequence (exclusive).
    ///
    /// Used for SSE reconnection via Last-Event-ID header.
    pub async fn query_since_sequence(
        &self,
        since: i64,
    ) -> Result<Vec<StoredEvent<E>>, InfrastructureError> {
        let rows = sqlx::query(
            r#"
            SELECT id, event_id, aggregate_type, aggregate_id, event_type,
                   schema_version, payload, command_id, final, created_at
            FROM events
            WHERE id > ?
            ORDER BY id
            "#,
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let payload: String = row.get("payload");
            let event: E = serde_json::from_str(&payload)?;
            events.push(StoredEvent {
                sequence: row.get("id"),
                event_id: row.get("event_id"),
                aggregate_type: row.get("aggregate_type"),
                aggregate_id: row.get("aggregate_id"),
                event_type: row.get("event_type"),
                schema_version: row.get("schema_version"),
                event,
                command_id: row.get("command_id"),
                is_final: row.get::<i64, _>("final") != 0,
                created_at: row.get("created_at"),
            });
        }
        Ok(events)
    }

    /// Get the earliest global sequence in the event store.
    ///
    /// Returns `None` if the event store is empty.
    pub async fn earliest_sequence(&self) -> Result<Option<i64>, InfrastructureError> {
        let row = sqlx::query("SELECT MIN(id) as min_id FROM events")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("min_id"))
    }

    /// Get the latest global sequence in the event store.
    ///
    /// Returns `None` if the event store is empty.
    pub async fn latest_sequence(&self) -> Result<Option<i64>, InfrastructureError> {
        let row = sqlx::query("SELECT MAX(id) as max_id FROM events")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("max_id"))
    }
}

/// Implementation of fmodel-rust's EventRepository trait for SQLite.
///
/// The `Version` type is `String` representing the event UUID, used for
/// optimistic locking via the previous_id chain.
impl<C, E> EventRepository<C, E, String, InfrastructureError> for SqliteEventRepository<C, E>
where
    C: Identifier + DeciderType + Sync,
    E: Identifier + EventType + DeciderType + IsFinal + Serialize + DeserializeOwned + Clone + Sync,
{
    /// Fetch all events for the aggregate identified by the command.
    ///
    /// Returns events ordered by global sequence (id), with each event
    /// paired with its event_id (version) for optimistic locking.
    async fn fetch_events(&self, command: &C) -> Result<Vec<(E, String)>, InfrastructureError> {
        let aggregate_id = command.identifier();
        let aggregate_type = command.decider_type();

        let rows = sqlx::query(
            r#"
            SELECT event_id, payload
            FROM events
            WHERE aggregate_type = ? AND aggregate_id = ?
            ORDER BY id
            "#,
        )
        .bind(&aggregate_type)
        .bind(&aggregate_id)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let event_id: String = row.get("event_id");
            let payload: String = row.get("payload");
            let event: E = serde_json::from_str(&payload)?;
            events.push((event, event_id));
        }
        Ok(events)
    }

    /// Save events to the event store with optimistic locking.
    ///
    /// Events are saved with a previous_id chain: the first event in an
    /// aggregate has NULL previous_id, subsequent events reference their
    /// predecessor's event_id.
    ///
    /// The chain is maintained by fetching the latest event_id before each
    /// insert and using it as previous_id for the new event.
    async fn save(&self, events: &[E]) -> Result<Vec<(E, String)>, InfrastructureError> {
        if events.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(events.len());

        for event in events {
            let event_id = Uuid::new_v4().to_string();
            let aggregate_id = event.identifier();
            let aggregate_type = event.decider_type();
            let event_type = event.event_type();
            let is_final = if event.is_final() { 1_i64 } else { 0_i64 };
            let payload = serde_json::to_string(event)?;

            // Fetch latest version for this aggregate to use as previous_id
            let previous_id = self.version_provider(event).await?;

            sqlx::query(
                r#"
                INSERT INTO events (
                    event_id, aggregate_type, aggregate_id, previous_id,
                    event_type, payload, final
                )
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&event_id)
            .bind(&aggregate_type)
            .bind(&aggregate_id)
            .bind(&previous_id)
            .bind(&event_type)
            .bind(&payload)
            .bind(is_final)
            .execute(&self.pool)
            .await?;

            results.push((event.clone(), event_id));
        }

        Ok(results)
    }

    /// Get the latest event_id (version) for the aggregate this event belongs to.
    ///
    /// Returns `None` if this would be the first event in the aggregate.
    /// Used for optimistic locking: the returned version becomes the
    /// previous_id of the next event.
    async fn version_provider(&self, event: &E) -> Result<Option<String>, InfrastructureError> {
        let aggregate_id = event.identifier();
        let aggregate_type = event.decider_type();

        let row = sqlx::query(
            r#"
            SELECT event_id
            FROM events
            WHERE aggregate_type = ? AND aggregate_id = ?
            ORDER BY id DESC
            LIMIT 1
            "#,
        )
        .bind(&aggregate_type)
        .bind(&aggregate_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.get("event_id")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    // Test helpers - minimal event/command types for testing
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

    impl EventType for TestEvent {
        fn event_type(&self) -> String {
            "TestEvent".to_string()
        }
    }

    impl DeciderType for TestEvent {
        fn decider_type(&self) -> String {
            "Test".to_string()
        }
    }

    impl IsFinal for TestEvent {
        fn is_final(&self) -> bool {
            false
        }
    }

    #[derive(Debug, Clone)]
    struct TestCommand {
        id: String,
    }

    impl Identifier for TestCommand {
        fn identifier(&self) -> String {
            self.id.clone()
        }
    }

    impl DeciderType for TestCommand {
        fn decider_type(&self) -> String {
            "Test".to_string()
        }
    }

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool");

        // Run migration
        sqlx::query(include_str!("../../migrations/001_events.sql"))
            .execute(&pool)
            .await
            .expect("Failed to run migration");

        pool
    }

    #[tokio::test]
    async fn test_save_and_fetch_events() {
        let pool = create_test_pool().await;
        let repo: SqliteEventRepository<TestCommand, TestEvent> = SqliteEventRepository::new(pool);

        let event = TestEvent {
            id: "agg-1".to_string(),
            data: "test data".to_string(),
        };

        // Save event
        let saved = repo.save(&[event.clone()]).await.unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].0, event);

        // Fetch events
        let command = TestCommand {
            id: "agg-1".to_string(),
        };
        let fetched = repo.fetch_events(&command).await.unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].0, event);
    }

    #[tokio::test]
    async fn test_previous_id_chain() {
        let pool = create_test_pool().await;
        let repo: SqliteEventRepository<TestCommand, TestEvent> = SqliteEventRepository::new(pool);

        let event1 = TestEvent {
            id: "agg-1".to_string(),
            data: "first".to_string(),
        };
        let event2 = TestEvent {
            id: "agg-1".to_string(),
            data: "second".to_string(),
        };

        // Save first event (previous_id should be NULL)
        let saved1 = repo.save(&[event1]).await.unwrap();
        let version1 = saved1[0].1.clone();

        // Save second event (previous_id should be version1)
        let saved2 = repo.save(&[event2]).await.unwrap();
        let version2 = saved2[0].1.clone();

        // Verify chain via version_provider
        let latest_version = repo.version_provider(&saved2[0].0).await.unwrap();
        assert_eq!(latest_version, Some(version2.clone()));

        // Fetch all events - should be ordered
        let command = TestCommand {
            id: "agg-1".to_string(),
        };
        let events = repo.fetch_events(&command).await.unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].1, version1);
        assert_eq!(events[1].1, version2);
    }

    #[tokio::test]
    async fn test_query_all_and_since_sequence() {
        let pool = create_test_pool().await;
        let repo: SqliteEventRepository<TestCommand, TestEvent> = SqliteEventRepository::new(pool);

        // Save events for two aggregates
        let event1 = TestEvent {
            id: "agg-1".to_string(),
            data: "first".to_string(),
        };
        let event2 = TestEvent {
            id: "agg-2".to_string(),
            data: "second".to_string(),
        };
        let event3 = TestEvent {
            id: "agg-1".to_string(),
            data: "third".to_string(),
        };

        repo.save(&[event1]).await.unwrap();
        repo.save(&[event2]).await.unwrap();
        repo.save(&[event3]).await.unwrap();

        // Query all
        let all_events = repo.query_all().await.unwrap();
        assert_eq!(all_events.len(), 3);

        // Query since sequence 1 (should get events 2 and 3)
        let since_events = repo.query_since_sequence(1).await.unwrap();
        assert_eq!(since_events.len(), 2);
        assert_eq!(since_events[0].sequence, 2);
        assert_eq!(since_events[1].sequence, 3);
    }

    #[tokio::test]
    async fn test_sequence_bounds() {
        let pool = create_test_pool().await;
        let repo: SqliteEventRepository<TestCommand, TestEvent> = SqliteEventRepository::new(pool);

        // Empty store
        assert_eq!(repo.earliest_sequence().await.unwrap(), None);
        assert_eq!(repo.latest_sequence().await.unwrap(), None);

        // Add events
        let event = TestEvent {
            id: "agg-1".to_string(),
            data: "test".to_string(),
        };
        repo.save(&[event]).await.unwrap();

        assert_eq!(repo.earliest_sequence().await.unwrap(), Some(1));
        assert_eq!(repo.latest_sequence().await.unwrap(), Some(1));
    }
}
