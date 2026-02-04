//! Event store re-exported from the `ironstar-event-store` crate.
//!
//! This module re-exports all public types from `ironstar_event_store::event_store`
//! to maintain backward compatibility with existing import paths.

pub use ironstar_event_store::event_store::EVENTS_MIGRATION_SQL;
pub use ironstar_event_store::{
    EventStoreError, EventStoreErrorKind, SqliteEventRepository, StoredEvent,
};
