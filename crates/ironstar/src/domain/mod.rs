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

pub mod analytics;
pub mod catalog;
pub mod common;
pub mod error;
pub mod query_session;
pub mod session;
pub mod signals;
pub mod todo;
pub mod traits;
pub mod views;
pub mod workspace;

// Trait re-exports (Identifier re-exported from fmodel_rust via traits module)
pub use traits::{DeciderType, EventType, IsFinal};

// Todo re-exports (from todo/)
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

// Session re-exports (from session/)
pub use session::{
    OAuthProvider, SessionCommand, SessionDecider, SessionError, SessionErrorKind, SessionEvent,
    SessionId, SessionMetadata, SessionState, SessionStatus, UserId, session_decider,
};

// Workspace re-exports (from workspace/)
pub use workspace::{
    Visibility, WORKSPACE_NAME_MAX_LENGTH, WorkspaceCommand, WorkspaceDecider, WorkspaceError,
    WorkspaceErrorKind, WorkspaceEvent, WorkspaceId, WorkspaceName, WorkspaceState,
    WorkspaceStatus, workspace_decider,
};
