//! Workspace domain implementation using fmodel-rust Decider pattern.
//!
//! The Workspace bounded context manages user workspaces, dashboards, tabs,
//! and their hierarchical relationships. It depends on the Session context's
//! `UserId` as a Shared Kernel type for ownership attribution.
//!
//! # State Machine
//!
//! ```text
//!                    ┌──────────────┐
//!     Create ───────►│    Active    │
//!                    └──────┬───────┘
//!                           │
//!            ┌──────────────┼──────────────┐
//!            │              │              │
//!         Rename     SetVisibility    (future: Delete)
//!            │              │              │
//!            └──────────────┴──────────────┘
//!                           │
//!                           ▼
//!                    ┌──────────────┐
//!                    │    Active    │ (same state, updated fields)
//!                    └──────────────┘
//! ```
//!
//! # Shared Kernel Pattern
//!
//! `UserId` is imported from the parent domain module (defined in Session context).
//! This establishes the cross-context dependency through explicit imports rather
//! than tight coupling between bounded contexts.
//!
//! # Idempotency
//!
//! Operations that would result in the same state return `Ok(vec![])`:
//! - Rename with the same name
//! - SetVisibility with the same visibility
//!
//! # Module Organization
//!
//! - [`commands`]: WorkspaceCommand enum
//! - [`decider`]: workspace_decider() factory with pure decide/evolve
//! - [`errors`]: WorkspaceError with UUID tracking
//! - [`events`]: WorkspaceEvent enum with audit trail
//! - [`state`]: WorkspaceState and WorkspaceStatus
//! - [`values`]: Value objects (WorkspaceId, WorkspaceName, Visibility)

pub mod commands;
pub mod decider;
pub mod errors;
pub mod events;
pub mod state;
pub mod values;

// Re-export UserId from Shared Kernel (Session context)
// This demonstrates the shared kernel pattern: Workspace depends on Session's UserId
pub use crate::domain::UserId;

// Re-export public types for ergonomic imports
pub use commands::WorkspaceCommand;
pub use decider::{WorkspaceDecider, workspace_decider};
pub use errors::{WorkspaceError, WorkspaceErrorKind};
pub use events::WorkspaceEvent;
pub use state::{WorkspaceState, WorkspaceStatus};
pub use values::{WORKSPACE_NAME_MAX_LENGTH, Visibility, WorkspaceId, WorkspaceName};

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that UserId can be imported from the shared kernel.
    ///
    /// This test validates the cross-context dependency pattern:
    /// Workspace context imports UserId from Session context via domain re-exports.
    #[test]
    fn shared_kernel_userid_import() {
        // UserId should be available through the workspace module
        let user_id = UserId::new();
        // Type should be correct (demonstrates successful import)
        let _: UserId = user_id;
    }

    /// Verify WorkspaceId and UserId are distinct types.
    ///
    /// The type system should prevent mixing up workspace and user identities.
    #[test]
    fn workspace_id_and_user_id_are_distinct() {
        let workspace_id = WorkspaceId::new();
        let user_id = UserId::new();

        // These should be different types (compile-time guarantee)
        // We verify at runtime that they have different Display output
        assert_ne!(workspace_id.to_string(), user_id.to_string());
    }
}
