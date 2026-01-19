//! Zenoh key expression patterns for event routing.
//!
//! This module defines the key expression schema for Zenoh event routing:
//! `events/{aggregate_type}/{aggregate_id}/{sequence}`
//!
//! # Subscription patterns
//!
//! | Pattern | Matches | Use case |
//! |---------|---------|----------|
//! | `events/**` | All events | Global audit log |
//! | `events/Todo/**` | All Todo events | Type-wide projections |
//! | `events/Todo/abc-123/**` | Specific aggregate | Entity SSE feed |
//! | `events/Todo/abc-123/5` | Specific event | Point lookup |
//!
//! # Key expression structure
//!
//! ```text
//! events/{aggregate_type}/{aggregate_id}/{sequence}
//!   │         │                │              │
//!   │         │                │              └─ Event sequence number (monotonic per aggregate)
//!   │         │                └─ Aggregate instance ID (e.g., UUID)
//!   │         └─ Aggregate type name (e.g., "Todo", "Session")
//!   └─ Root namespace for domain events
//! ```
//!
//! # Type safety
//!
//! The [`EventKeyExpr`] type provides validated key expressions with
//! construction from components and parsing back to extract metadata.

use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Root namespace for all domain events.
pub const EVENTS_ROOT: &str = "events";

/// Wildcard matching zero or more path segments.
pub const DOUBLE_WILD: &str = "**";

/// Wildcard matching exactly one path segment.
pub const SINGLE_WILD: &str = "*";

/// Key expression for subscribing to all events.
///
/// Pattern: `events/**`
pub const ALL_EVENTS: &str = "events/**";

/// Constructs a key expression for subscribing to all events of an aggregate type.
///
/// Pattern: `events/{aggregate_type}/**`
///
/// # Example
///
/// ```rust,ignore
/// let pattern = aggregate_type_pattern("Todo");
/// assert_eq!(pattern, "events/Todo/**");
/// ```
#[must_use]
pub fn aggregate_type_pattern(aggregate_type: &str) -> String {
    format!("{EVENTS_ROOT}/{aggregate_type}/{DOUBLE_WILD}")
}

/// Constructs a key expression for subscribing to all events of a specific aggregate instance.
///
/// Pattern: `events/{aggregate_type}/{aggregate_id}/**`
///
/// # Example
///
/// ```rust,ignore
/// let pattern = aggregate_instance_pattern("Todo", "abc-123");
/// assert_eq!(pattern, "events/Todo/abc-123/**");
/// ```
#[must_use]
pub fn aggregate_instance_pattern(aggregate_type: &str, aggregate_id: &str) -> String {
    format!("{EVENTS_ROOT}/{aggregate_type}/{aggregate_id}/{DOUBLE_WILD}")
}

/// Constructs a key expression for a specific event.
///
/// Pattern: `events/{aggregate_type}/{aggregate_id}/{sequence}`
///
/// This is the full key used when publishing events.
///
/// # Example
///
/// ```rust,ignore
/// let key = event_key("Todo", "abc-123", 5);
/// assert_eq!(key, "events/Todo/abc-123/5");
/// ```
#[must_use]
pub fn event_key(aggregate_type: &str, aggregate_id: &str, sequence: u64) -> String {
    format!("{EVENTS_ROOT}/{aggregate_type}/{aggregate_id}/{sequence}")
}

/// Constructs a key expression for publishing without sequence (legacy compatibility).
///
/// Pattern: `events/{aggregate_type}/{aggregate_id}`
///
/// This pattern is used by the current `EventBus` implementation which publishes
/// without sequence numbers. Use [`event_key`] for the full pattern.
#[must_use]
pub fn event_key_without_sequence(aggregate_type: &str, aggregate_id: &str) -> String {
    format!("{EVENTS_ROOT}/{aggregate_type}/{aggregate_id}")
}

/// Parsed event key expression components.
///
/// Represents a fully-qualified event key with all components extracted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventKeyExpr {
    /// Aggregate type name (e.g., "Todo", "Session").
    pub aggregate_type: String,
    /// Aggregate instance identifier.
    pub aggregate_id: String,
    /// Event sequence number within the aggregate stream.
    pub sequence: Option<u64>,
}

impl EventKeyExpr {
    /// Create a new event key expression with all components.
    #[must_use]
    pub fn new(
        aggregate_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        sequence: u64,
    ) -> Self {
        Self {
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
            sequence: Some(sequence),
        }
    }

    /// Create an event key expression without sequence (for matching the current EventBus format).
    #[must_use]
    pub fn without_sequence(
        aggregate_type: impl Into<String>,
        aggregate_id: impl Into<String>,
    ) -> Self {
        Self {
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
            sequence: None,
        }
    }

    /// Convert to a Zenoh key expression string.
    #[must_use]
    pub fn to_key_expr(&self) -> String {
        match self.sequence {
            Some(seq) => event_key(&self.aggregate_type, &self.aggregate_id, seq),
            None => event_key_without_sequence(&self.aggregate_type, &self.aggregate_id),
        }
    }

    /// Parse a Zenoh key expression string into components.
    ///
    /// Accepts both formats:
    /// - `events/{type}/{id}` (without sequence)
    /// - `events/{type}/{id}/{sequence}` (with sequence)
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the key expression is malformed.
    pub fn parse(key_expr: &str) -> Result<Self, ParseError> {
        let parts: Vec<&str> = key_expr.split('/').collect();

        // Validate minimum structure: events/{type}/{id}
        if parts.len() < 3 {
            return Err(ParseError::TooFewSegments {
                expected: 3,
                found: parts.len(),
            });
        }

        // Validate root namespace
        if parts[0] != EVENTS_ROOT {
            return Err(ParseError::InvalidRoot {
                expected: EVENTS_ROOT.to_string(),
                found: parts[0].to_string(),
            });
        }

        let aggregate_type = parts[1].to_string();
        let aggregate_id = parts[2].to_string();

        // Validate non-empty components
        if aggregate_type.is_empty() {
            return Err(ParseError::EmptyComponent("aggregate_type".to_string()));
        }
        if aggregate_id.is_empty() {
            return Err(ParseError::EmptyComponent("aggregate_id".to_string()));
        }

        // Parse optional sequence
        let sequence = if parts.len() >= 4 && !parts[3].is_empty() {
            Some(
                parts[3]
                    .parse::<u64>()
                    .map_err(|_| ParseError::InvalidSequence {
                        value: parts[3].to_string(),
                    })?,
            )
        } else {
            None
        };

        Ok(Self {
            aggregate_type,
            aggregate_id,
            sequence,
        })
    }
}

impl fmt::Display for EventKeyExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_key_expr())
    }
}

impl FromStr for EventKeyExpr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Error parsing an event key expression.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Key expression has too few path segments.
    #[error("key expression has too few segments: expected at least {expected}, found {found}")]
    TooFewSegments { expected: usize, found: usize },

    /// Key expression does not start with the expected root namespace.
    #[error("invalid root namespace: expected '{expected}', found '{found}'")]
    InvalidRoot { expected: String, found: String },

    /// A required component is empty.
    #[error("empty component: {0}")]
    EmptyComponent(String),

    /// Sequence number is not a valid u64.
    #[error("invalid sequence number: '{value}' is not a valid u64")]
    InvalidSequence { value: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_type_pattern_constructs_correctly() {
        assert_eq!(aggregate_type_pattern("Todo"), "events/Todo/**");
        assert_eq!(aggregate_type_pattern("Session"), "events/Session/**");
        assert_eq!(
            aggregate_type_pattern("QuerySession"),
            "events/QuerySession/**"
        );
    }

    #[test]
    fn aggregate_instance_pattern_constructs_correctly() {
        assert_eq!(
            aggregate_instance_pattern("Todo", "abc-123"),
            "events/Todo/abc-123/**"
        );
        assert_eq!(
            aggregate_instance_pattern("Session", "550e8400-e29b-41d4-a716-446655440000"),
            "events/Session/550e8400-e29b-41d4-a716-446655440000/**"
        );
    }

    #[test]
    fn event_key_constructs_correctly() {
        assert_eq!(event_key("Todo", "abc-123", 0), "events/Todo/abc-123/0");
        assert_eq!(event_key("Todo", "abc-123", 5), "events/Todo/abc-123/5");
        assert_eq!(event_key("Todo", "abc-123", 42), "events/Todo/abc-123/42");
    }

    #[test]
    fn event_key_without_sequence_matches_current_event_bus() {
        // This matches the current format in event_bus.rs line 102
        assert_eq!(
            event_key_without_sequence("Todo", "abc-123"),
            "events/Todo/abc-123"
        );
    }

    #[test]
    fn event_key_expr_new_creates_with_sequence() {
        let key = EventKeyExpr::new("Todo", "abc-123", 5);
        assert_eq!(key.aggregate_type, "Todo");
        assert_eq!(key.aggregate_id, "abc-123");
        assert_eq!(key.sequence, Some(5));
        assert_eq!(key.to_key_expr(), "events/Todo/abc-123/5");
    }

    #[test]
    fn event_key_expr_without_sequence_creates_legacy_format() {
        let key = EventKeyExpr::without_sequence("Todo", "abc-123");
        assert_eq!(key.sequence, None);
        assert_eq!(key.to_key_expr(), "events/Todo/abc-123");
    }

    #[test]
    fn event_key_expr_parse_with_sequence() {
        let key = EventKeyExpr::parse("events/Todo/abc-123/5").unwrap();
        assert_eq!(key.aggregate_type, "Todo");
        assert_eq!(key.aggregate_id, "abc-123");
        assert_eq!(key.sequence, Some(5));
    }

    #[test]
    fn event_key_expr_parse_without_sequence() {
        let key = EventKeyExpr::parse("events/Todo/abc-123").unwrap();
        assert_eq!(key.aggregate_type, "Todo");
        assert_eq!(key.aggregate_id, "abc-123");
        assert_eq!(key.sequence, None);
    }

    #[test]
    fn event_key_expr_parse_uuid_aggregate_id() {
        let key =
            EventKeyExpr::parse("events/Session/550e8400-e29b-41d4-a716-446655440000/42").unwrap();
        assert_eq!(key.aggregate_type, "Session");
        assert_eq!(key.aggregate_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(key.sequence, Some(42));
    }

    #[test]
    fn event_key_expr_parse_rejects_invalid_root() {
        let err = EventKeyExpr::parse("invalid/Todo/abc-123").unwrap_err();
        assert!(matches!(err, ParseError::InvalidRoot { .. }));
    }

    #[test]
    fn event_key_expr_parse_rejects_too_few_segments() {
        let err = EventKeyExpr::parse("events/Todo").unwrap_err();
        assert!(matches!(err, ParseError::TooFewSegments { .. }));

        let err = EventKeyExpr::parse("events").unwrap_err();
        assert!(matches!(err, ParseError::TooFewSegments { .. }));
    }

    #[test]
    fn event_key_expr_parse_rejects_invalid_sequence() {
        let err = EventKeyExpr::parse("events/Todo/abc-123/not-a-number").unwrap_err();
        assert!(matches!(err, ParseError::InvalidSequence { .. }));
    }

    #[test]
    fn event_key_expr_roundtrip_with_sequence() {
        let original = EventKeyExpr::new("Workspace", "ws-42", 100);
        let key_str = original.to_key_expr();
        let parsed = EventKeyExpr::parse(&key_str).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn event_key_expr_roundtrip_without_sequence() {
        let original = EventKeyExpr::without_sequence("Workspace", "ws-42");
        let key_str = original.to_key_expr();
        let parsed = EventKeyExpr::parse(&key_str).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn event_key_expr_display_trait() {
        let key = EventKeyExpr::new("Todo", "abc", 1);
        assert_eq!(format!("{key}"), "events/Todo/abc/1");
    }

    #[test]
    fn event_key_expr_from_str_trait() {
        let key: EventKeyExpr = "events/Todo/abc/1".parse().unwrap();
        assert_eq!(key.aggregate_type, "Todo");
        assert_eq!(key.aggregate_id, "abc");
        assert_eq!(key.sequence, Some(1));
    }

    #[test]
    fn all_events_constant_is_valid_pattern() {
        assert_eq!(ALL_EVENTS, "events/**");
    }

    #[test]
    fn event_key_expr_parse_handles_large_sequence() {
        let key = EventKeyExpr::parse("events/Todo/abc/18446744073709551615").unwrap();
        assert_eq!(key.sequence, Some(u64::MAX));
    }

    #[test]
    fn event_key_expr_parse_handles_zero_sequence() {
        let key = EventKeyExpr::parse("events/Todo/abc/0").unwrap();
        assert_eq!(key.sequence, Some(0));
    }
}
