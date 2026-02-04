//! SSE stream utilities re-exported from the `ironstar-event-store` crate.
//!
//! This module re-exports all public types from `ironstar_event_store::sse_stream`
//! to maintain backward compatibility with existing import paths.

pub use ironstar_event_store::{
    DEFAULT_KEEP_ALIVE_SECS, KEEP_ALIVE_COMMENT, KeepAliveStream, SseStreamBuilder,
    event_with_sequence, stored_events_to_stream, zenoh_to_sse_stream,
};
