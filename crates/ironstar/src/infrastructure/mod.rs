//! Infrastructure layer - effect implementations for persistence and messaging.
//!
//! This layer provides concrete implementations of I/O operations. All functions
//! and methods here are `async` because they perform side effects: database
//! queries, network calls, file operations, or inter-process communication.
//!
//! # Async-only by design
//!
//! Every public function in this module must be `async`. This is a deliberate
//! architectural constraint that enforces the effect boundary:
//!
//! - **Domain layer**: sync, pure, no I/O
//! - **Application layer**: async orchestration around sync domain calls
//! - **Infrastructure layer**: async, effectful, all I/O lives here
//!
//! If you find yourself writing a sync function in infrastructure, it likely
//! belongs in the domain layer or is a private helper that should be inlined.
//!
//! # Port/adapter pattern
//!
//! Infrastructure implements port traits defined in the interfaces crate.
//! This enables dependency inversion: application code depends on abstract
//! interfaces, not concrete implementations.
//!
//! ```rust,ignore
//! // Port trait (in ironstar-interfaces)
//! #[async_trait]
//! pub trait EventStore: Send + Sync {
//!     async fn append(&self, events: Vec<NewEvent>) -> Result<(), EventStoreError>;
//!     async fn load_stream(&self, stream_id: &str) -> Result<Vec<StoredEvent>, EventStoreError>;
//! }
//!
//! // Adapter (in this module)
//! pub struct SqliteEventStore {
//!     pool: SqlitePool,
//! }
//!
//! #[async_trait]
//! impl EventStore for SqliteEventStore {
//!     async fn append(&self, events: Vec<NewEvent>) -> Result<(), EventStoreError> {
//!         // SQLite-specific implementation
//!     }
//!     // ...
//! }
//! ```
//!
//! # Effect categories
//!
//! This layer handles several categories of effects:
//!
//! | Category | Examples | Rust types |
//! |----------|----------|------------|
//! | Persistence | Event store, session store, projections | `sqlx::SqlitePool` |
//! | Messaging | Event bus, pub/sub | `zenoh::Publisher`, `tokio::broadcast` |
//! | Caching | Analytics cache, query results | `moka::future::Cache` |
//! | External APIs | OAuth providers, third-party services | `reqwest::Client` |
//!
//! # Testability
//!
//! Infrastructure adapters should be swappable for testing. Common patterns:
//!
//! - **In-memory implementations**: Fast, isolated unit tests
//! - **Test containers**: Integration tests with real databases
//! - **Mock servers**: External API testing with `wiremock`
//!
//! ```rust,ignore
//! // In-memory adapter for testing
//! pub struct InMemoryEventStore {
//!     events: Arc<RwLock<Vec<StoredEvent>>>,
//! }
//!
//! #[async_trait]
//! impl EventStore for InMemoryEventStore {
//!     async fn append(&self, events: Vec<NewEvent>) -> Result<(), EventStoreError> {
//!         let mut store = self.events.write().await;
//!         store.extend(events.into_iter().map(StoredEvent::from));
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # What belongs here
//!
//! - Database adapters (SQLite event store, session store, projections)
//! - Message bus adapters (Zenoh publisher/subscriber, broadcast channels)
//! - Cache implementations (moka async cache)
//! - External service clients (OAuth, HTTP APIs)
//! - File system operations (asset loading, configuration)
//!
//! # What does NOT belong here
//!
//! - Business logic (belongs in [`crate::domain`])
//! - Request/response handling (belongs in [`crate::presentation`])
//! - Orchestration of multiple services (belongs in [`crate::application`])
//! - Synchronous functions (by design, everything here is async)

pub mod error;
pub mod event_bus;
pub mod event_store;

pub use error::{InfrastructureError, InfrastructureErrorKind};
pub use event_bus::{
    EventBus, ZenohEventBus, open_embedded_session, publish_events_fire_and_forget,
    zenoh_embedded_config,
};
pub use event_store::{SqliteEventRepository, StoredEvent};
