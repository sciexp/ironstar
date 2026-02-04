//! Event bus re-exported from the `ironstar-event-bus` crate.
//!
//! This module re-exports all public types from `ironstar_event_bus`
//! to maintain backward compatibility with existing import paths.

pub use ironstar_event_bus::{
    EventBus, ZenohEventBus, open_embedded_session, publish_events_fire_and_forget,
    zenoh_embedded_config,
};

pub mod workspace {
    //! Workspace subscriber factory re-exported from `ironstar-event-bus`.
    pub use ironstar_event_bus::workspace::{
        ALL_WORKSPACE_AGGREGATE_TYPES, DASHBOARD_TYPE, SAVED_QUERY_TYPE, USER_PREFERENCES_TYPE,
        WORKSPACE_TYPE, WorkspaceSubscriberFactory, ZenohSubscriber, dashboard_events_pattern,
        saved_query_events_pattern, user_preferences_events_pattern, workspace_cache_dependencies,
        workspace_events_pattern,
    };
}
