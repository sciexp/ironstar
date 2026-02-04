//! Workspace domain: 5 aggregates (Workspace, Dashboard, SavedQuery, UserPreferences,
//! WorkspacePreferences) with views.
//!
//! This crate contains the workspace bounded context for ironstar, managing
//! user workspaces, dashboards, saved queries, and user/workspace preferences.
//! It depends on ironstar-shared-kernel for UserId and ironstar-analytics for
//! chart-related value objects (Customer-Supplier relationship).

pub mod dashboard;
pub mod saved_query;
pub mod user_preferences;
pub mod views;
pub mod workspace;
pub mod workspace_preferences;

// Re-export aggregate types for ergonomic imports
pub use dashboard::{
    DashboardCommand, DashboardDecider, DashboardError, DashboardErrorKind, DashboardEvent,
    DashboardState, dashboard_decider,
};
pub use saved_query::{
    SavedQueryCommand, SavedQueryDecider, SavedQueryError, SavedQueryErrorKind, SavedQueryEvent,
    SavedQueryState, saved_query_decider,
};
pub use user_preferences::{
    UserPreferencesCommand, UserPreferencesDecider, UserPreferencesError, UserPreferencesErrorKind,
    UserPreferencesEvent, UserPreferencesState, user_preferences_decider,
};
pub use workspace::{
    WorkspaceCommand, WorkspaceDecider, WorkspaceError, WorkspaceErrorKind, WorkspaceEvent,
    WorkspaceState, WorkspaceStatus, workspace_decider,
};
pub use workspace_preferences::{
    WorkspacePreferencesCommand, WorkspacePreferencesDecider, WorkspacePreferencesError,
    WorkspacePreferencesErrorKind, WorkspacePreferencesEvent, WorkspacePreferencesState,
    workspace_preferences_decider,
};

// Re-export views
pub use views::workspace::{
    DashboardLayoutView, DashboardLayoutViewState, SavedQueryListEntry, SavedQueryListView,
    SavedQueryListViewState, UserPreferencesView, UserPreferencesViewState, WorkspaceListEntry,
    WorkspaceListView, WorkspaceListViewState, dashboard_layout_view, saved_query_list_view,
    user_preferences_view, workspace_list_view,
};
