//! View modules for read-side projections.
//!
//! Views are the read side of CQRS: pure functions that project events into
//! queryable state. Unlike Deciders, Views have no commands or errors — they
//! simply evolve state by folding events.
//!
//! # View vs Decider
//!
//! - **Decider**: `(Command, State) -> Result<Vec<Event>, Error>` + `(State, Event) -> State`
//! - **View**: `(State, Event) -> State` only
//!
//! Views reuse the evolve function pattern but drop command handling entirely.

pub mod catalog;
pub mod query_session;
pub mod todo;
pub mod workspace;

pub use catalog::{CatalogView, CatalogViewState, catalog_view};
pub use query_session::{
    QueryHistoryEntry, QueryOutcome, QuerySessionView, QuerySessionViewState, query_session_view,
};
pub use todo::{TodoView, TodoViewState, todo_view};
pub use workspace::{
    DashboardLayoutView, DashboardLayoutViewState, SavedQueryListEntry, SavedQueryListView,
    SavedQueryListViewState, UserPreferencesView, UserPreferencesViewState, WorkspaceListEntry,
    WorkspaceListView, WorkspaceListViewState, dashboard_layout_view, saved_query_list_view,
    user_preferences_view, workspace_list_view,
};
