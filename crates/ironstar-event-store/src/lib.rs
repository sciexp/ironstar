//! SQLite event store and SSE stream composition for ironstar.
//!
//! This crate provides event persistence via `SqliteEventRepository` implementing
//! fmodel-rust's `EventRepository` trait, plus SSE stream utilities for composing
//! historical replay with live Zenoh subscription streams.

pub mod error;
pub mod event_store;
pub mod sse_stream;

pub use error::{EventStoreError, EventStoreErrorKind};
pub use event_store::{EVENTS_MIGRATION_SQL, SqliteEventRepository, StoredEvent};
pub use sse_stream::{
    DEFAULT_KEEP_ALIVE_SECS, KEEP_ALIVE_COMMENT, KeepAliveStream, SseStreamBuilder,
    event_with_sequence, stored_events_to_stream, zenoh_to_sse_stream,
};
