//! Key expression utilities re-exported from the `ironstar-event-bus` crate.

pub use ironstar_event_bus::{
    ALL_EVENTS, DOUBLE_WILD, EVENTS_ROOT, EventKeyExpr, KeyExprParseError as ParseError,
    SINGLE_WILD, aggregate_instance_pattern, aggregate_type_pattern, event_key,
    event_key_without_sequence,
};
