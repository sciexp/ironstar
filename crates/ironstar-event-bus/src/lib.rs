//! Zenoh event bus, key expression routing, and cache invalidation for ironstar.
//!
//! This crate provides event publishing via `ZenohEventBus` implementing the `EventBus`
//! trait, key expression utilities for CQRS routing, and cache dependency declarations
//! for invalidation via Zenoh subscriptions.

pub mod cache_dependency;
pub mod error;
pub mod event_bus;
pub mod key_expr;
pub mod workspace;

pub use cache_dependency::{CacheDependency, matches_key_expression};
pub use error::{EventBusError, EventBusErrorKind};
pub use event_bus::{
    EventBus, ZenohEventBus, open_embedded_session, publish_events_fire_and_forget,
    zenoh_embedded_config,
};
pub use key_expr::{
    ALL_EVENTS, DOUBLE_WILD, EVENTS_ROOT, EventKeyExpr, ParseError as KeyExprParseError,
    SINGLE_WILD, aggregate_instance_pattern, aggregate_type_pattern, event_key,
    event_key_without_sequence,
};
pub use workspace::{
    ALL_WORKSPACE_AGGREGATE_TYPES, DASHBOARD_TYPE, SAVED_QUERY_TYPE, USER_PREFERENCES_TYPE,
    WORKSPACE_TYPE, WorkspaceSubscriberFactory, ZenohSubscriber, dashboard_events_pattern,
    saved_query_events_pattern, user_preferences_events_pattern, workspace_cache_dependencies,
    workspace_events_pattern,
};
