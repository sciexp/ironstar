//! fmodel-rust identifier trait re-export and ironstar-specific marker traits.
//!
//! - `Identifier`: Re-exported from fmodel-rust, used for aggregate identity in event sourcing
//! - `EventType`: Event type discriminator for JSON schema evolution
//! - `DeciderType`: Aggregate type name for polymorphic routing
//! - `IsFinal`: Terminal state marker for aggregate lifecycle

pub use fmodel_rust::Identifier;

/// Event type discriminator for JSON schema evolution.
///
/// Returns the event variant name matching the serde tag for deserialization.
pub trait EventType {
    fn event_type(&self) -> String;
}

/// Aggregate type name for polymorphic routing.
///
/// Returns the aggregate type identifier (e.g., "Todo", "QuerySession").
pub trait DeciderType {
    fn decider_type(&self) -> String;
}

/// Terminal state marker - aggregate won't accept further commands.
///
/// When true, the aggregate has reached a final state (e.g., deleted).
pub trait IsFinal {
    fn is_final(&self) -> bool;
}
