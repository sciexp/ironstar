//! Domain layer - algebraic types and pure domain logic.
//!
//! This module contains the pure core of the application: types that
//! represent domain concepts and functions that implement business rules.
//! Everything here is synchronous, deterministic, and side-effect-free.
//!
//! # Module Organization
//!
//! - [`analytics`]: Analytics value objects (QueryId, DatasetRef, SqlQuery, ChartConfig)
//! - [`common`]: Shared value objects (BoundedString, DashboardTitle, TabTitle, GridSize)
//! - [`query_session`]: QuerySession decider (commands, events, state, errors)
//! - [`session`]: Session decider for authentication lifecycle (Shared Kernel: UserId)
//! - [`todo`]: Todo decider (commands, events, state, values, errors)
//! - [`views`]: Read-side projections (View pattern for CQRS read models)
//! - [`workspace`]: Workspace decider for user workspaces (imports UserId from session)
//!
//! # Design Principles
//!
//! 1. **Parse, don't validate**: Value objects enforce invariants at
//!    construction time. If you have a `TodoText`, it's guaranteed valid.
//!
//! 2. **Make invalid states unrepresentable**: Sum types (enums) ensure
//!    only valid state combinations can exist.
//!
//! 3. **Pure functions at the core**: Aggregates are pure state machines.
//!    All I/O happens at boundaries (application/infrastructure layers).
//!
//! 4. **Effects at boundaries**: The async/sync boundary marks the effect
//!    boundary. Domain functions are sync; I/O functions are async.
//!
//! # Example
//!
//! ```rust,ignore
//! use fmodel_rust::decider::EventComputation;
//! use ironstar::domain::{todo_decider, TodoCommand, TodoId};
//! use chrono::Utc;
//!
//! // Create the decider (pure function, no state)
//! let decider = todo_decider();
//! let id = TodoId::new();
//! let now = Utc::now();
//!
//! // Compute new events from command
//! let events = decider.compute_new_events(
//!     &[],
//!     &TodoCommand::Create { id, text: "Buy groceries".to_string(), created_at: now }
//! )?;
//!
//! // Events are returned; state is computed by folding events
//! assert_eq!(events.len(), 1);
//! ```

// Inline re-export modules for extracted domain crates.
// Each `pub mod` preserves the `crate::domain::X` import path while
// the subdirectory shim files have been removed.

pub mod todo {
    //! Todo domain re-exports from `ironstar-todo` crate.
    pub use ironstar_todo::*;
}

pub mod session {
    //! Session domain aggregate re-exports from `ironstar-session` crate.
    pub use ironstar_session::*;
}

pub mod analytics {
    //! Analytics domain value objects re-exports from `ironstar-analytics` crate.
    pub use ironstar_analytics::combined;
    pub use ironstar_analytics::workflow;
    pub use ironstar_analytics::*;
}

pub mod catalog {
    //! Catalog Decider module re-exports from `ironstar-analytics` crate.
    pub use ironstar_analytics::catalog::errors;
    pub use ironstar_analytics::catalog::values;
    pub use ironstar_analytics::catalog::*;
}

pub mod query_session {
    //! QuerySession Decider module re-exports from `ironstar-analytics` crate.
    pub use ironstar_analytics::query_session::errors;
    pub use ironstar_analytics::query_session::*;
}

pub mod dashboard {
    //! Dashboard aggregate re-exports from `ironstar-workspace` crate.
    pub use ironstar_workspace::dashboard::*;
}

pub mod saved_query {
    //! SavedQuery aggregate re-exports from `ironstar-workspace` crate.
    pub use ironstar_workspace::saved_query::*;
}

pub mod user_preferences {
    //! UserPreferences aggregate re-exports from `ironstar-workspace` crate.
    pub use ironstar_workspace::user_preferences::*;
}

pub mod workspace {
    //! Workspace aggregate re-exports from `ironstar-workspace` crate.
    pub use ironstar_workspace::workspace::*;
}

pub mod workspace_preferences {
    //! WorkspacePreferences aggregate re-exports from `ironstar-workspace` crate.
    pub use ironstar_workspace::workspace_preferences::*;
}

pub mod common {
    //! Common domain value objects re-exported from `ironstar-core`.
    pub mod values {
        pub use ironstar_core::{
            BoundedString, DASHBOARD_TITLE_MAX_LENGTH, DASHBOARD_TITLE_MIN_LENGTH, DashboardTitle,
            GRID_HEIGHT_MIN, GRID_WIDTH_MIN, GridSize, TAB_TITLE_MAX_LENGTH, TAB_TITLE_MIN_LENGTH,
            TabTitle,
        };
    }

    pub use values::{
        BoundedString, DASHBOARD_TITLE_MAX_LENGTH, DASHBOARD_TITLE_MIN_LENGTH, DashboardTitle,
        GRID_HEIGHT_MIN, GRID_WIDTH_MIN, GridSize, TAB_TITLE_MAX_LENGTH, TAB_TITLE_MIN_LENGTH,
        TabTitle,
    };
}

pub mod error {
    //! Domain layer error types re-exported from `ironstar-core`.
    pub use ironstar_core::error::{
        DomainError, DomainErrorKind, ValidationError, ValidationErrorKind,
    };
}

pub mod traits {
    //! fmodel-rust identifier trait and ironstar-specific marker traits.
    pub use ironstar_core::traits::{DeciderType, EventType, Identifier, IsFinal};
}

pub mod views {
    //! View modules for read-side projections.

    pub mod catalog {
        //! Catalog View re-exports from `ironstar-analytics` crate.
        pub use ironstar_analytics::views::catalog::*;
    }

    pub mod query_session {
        //! QuerySession View re-exports from `ironstar-analytics` crate.
        pub use ironstar_analytics::views::query_session::*;
    }

    pub mod workspace {
        //! Workspace views re-exports from `ironstar-workspace` crate.
        pub use ironstar_workspace::views::workspace::*;
    }

    pub use catalog::{CatalogView, CatalogViewState, catalog_view};
    pub use ironstar_todo::{TodoItemView, TodoView, TodoViewState, todo_view};
    pub use query_session::{
        QueryHistoryEntry, QueryOutcome, QuerySessionView, QuerySessionViewState,
        query_session_view,
    };
    pub use workspace::{
        DashboardLayoutView, DashboardLayoutViewState, SavedQueryListEntry, SavedQueryListView,
        SavedQueryListViewState, UserPreferencesView, UserPreferencesViewState, WorkspaceListEntry,
        WorkspaceListView, WorkspaceListViewState, dashboard_layout_view, saved_query_list_view,
        user_preferences_view, workspace_list_view,
    };
}

// Signals module kept as a real file (429 lines of original code)
pub mod signals;

// Trait re-exports (Identifier re-exported from fmodel_rust via traits module)
pub use traits::{DeciderType, EventType, IsFinal};

// Todo re-exports
pub use todo::{
    TODO_TEXT_MAX_LENGTH, TodoCommand, TodoDecider, TodoError, TodoErrorKind, TodoEvent, TodoId,
    TodoState, TodoStatus, TodoText, todo_decider,
};

// Analytics re-exports
pub use analytics::{
    AnalyticsError, AnalyticsErrorKind, AnalyticsValidationError, AnalyticsValidationErrorKind,
    ChartConfig, ChartType, DATASET_REF_MAX_LENGTH, DatasetRef, QueryId, SQL_QUERY_MAX_LENGTH,
    SqlQuery,
};

// Catalog re-exports
pub use catalog::{
    CATALOG_REF_MAX_LENGTH, CatalogCommand, CatalogDecider, CatalogError, CatalogErrorKind,
    CatalogEvent, CatalogMetadata, CatalogRef, CatalogState, DatasetInfo, catalog_decider,
};

// QuerySession re-exports
pub use query_session::{
    QuerySessionCommand, QuerySessionDecider, QuerySessionError, QuerySessionErrorKind,
    QuerySessionEvent, QuerySessionState, QuerySessionStatus, query_session_decider,
};

// Signal re-exports
pub use signals::{ChartSelection, ChartSignals, TodoFilter, TodoItemView, TodoSignals};

// Error re-exports
pub use error::{DomainError, DomainErrorKind, ValidationError, ValidationErrorKind};

// Common value object re-exports
pub use common::{
    BoundedString, DASHBOARD_TITLE_MAX_LENGTH, DASHBOARD_TITLE_MIN_LENGTH, DashboardTitle,
    GRID_HEIGHT_MIN, GRID_WIDTH_MIN, GridSize, TAB_TITLE_MAX_LENGTH, TAB_TITLE_MIN_LENGTH,
    TabTitle,
};

// View re-exports
pub use views::{
    CatalogView, CatalogViewState, QueryHistoryEntry, QueryOutcome, QuerySessionView,
    QuerySessionViewState, TodoView, TodoViewState, catalog_view, query_session_view, todo_view,
};

// Session re-exports
pub use session::{
    OAuthProvider, SessionCommand, SessionDecider, SessionError, SessionErrorKind, SessionEvent,
    SessionId, SessionMetadata, SessionState, SessionStatus, UserId, session_decider,
};

// Workspace re-exports
pub use workspace::{
    Visibility, WORKSPACE_NAME_MAX_LENGTH, WorkspaceCommand, WorkspaceDecider, WorkspaceError,
    WorkspaceErrorKind, WorkspaceEvent, WorkspaceId, WorkspaceName, WorkspaceState,
    WorkspaceStatus, workspace_decider,
};

// SavedQuery re-exports
pub use saved_query::{
    QUERY_NAME_MAX_LENGTH, QUERY_NAME_MIN_LENGTH, QueryName, SavedQueryCommand, SavedQueryDecider,
    SavedQueryError, SavedQueryErrorKind, SavedQueryEvent, SavedQueryId, SavedQueryState,
    saved_query_decider,
};

// UserPreferences re-exports
pub use user_preferences::{
    LOCALE_MAX_LENGTH, Locale, PreferencesId, Theme, UiState, UserPreferencesCommand,
    UserPreferencesDecider, UserPreferencesError, UserPreferencesErrorKind, UserPreferencesEvent,
    UserPreferencesState, user_preferences_decider,
};

// Dashboard re-exports
pub use dashboard::{
    ChartDefinitionRef, ChartId, ChartPlacement, DashboardCommand, DashboardDecider,
    DashboardError, DashboardErrorKind, DashboardEvent, DashboardId, DashboardState, GridPosition,
    TabId, TabInfo, dashboard_decider,
};

// WorkspacePreferences re-exports
pub use workspace_preferences::{
    CATALOG_URI_MAX_LENGTH, CatalogUri, LayoutDefaults, WorkspacePreferencesCommand,
    WorkspacePreferencesDecider, WorkspacePreferencesError, WorkspacePreferencesErrorKind,
    WorkspacePreferencesEvent, WorkspacePreferencesState, workspace_preferences_decider,
};
