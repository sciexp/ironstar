//! Workspace aggregate state types.
//!
//! State is derived from events and represents the current status of a workspace.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::values::{Visibility, WorkspaceId, WorkspaceName};
use ironstar_shared_kernel::UserId;

/// Lifecycle status of a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorkspaceStatus {
    /// Initial state (before any events).
    #[default]
    NotCreated,
    /// Workspace is active and can be modified.
    Active,
}

/// State of a single workspace, derived from events.
///
/// All fields are `Option` because the decider starts empty (before the
/// `Created` event). After `Created`, the fields are guaranteed to be `Some`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorkspaceState {
    /// Unique identifier (set on Created).
    pub id: Option<WorkspaceId>,
    /// Current name (set on Created, updated on Renamed).
    pub name: Option<WorkspaceName>,
    /// Owner of the workspace.
    pub owner_id: Option<UserId>,
    /// Visibility setting.
    pub visibility: Option<Visibility>,
    /// When the workspace was created.
    pub created_at: Option<DateTime<Utc>>,
    /// Lifecycle status.
    pub status: WorkspaceStatus,
}

impl WorkspaceState {
    /// Check if the workspace exists (has been created).
    #[must_use]
    pub fn exists(&self) -> bool {
        self.status != WorkspaceStatus::NotCreated
    }

    /// Check if the workspace is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status == WorkspaceStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_not_created() {
        let state = WorkspaceState::default();
        assert_eq!(state.status, WorkspaceStatus::NotCreated);
        assert!(!state.exists());
        assert!(!state.is_active());
    }

    #[test]
    fn active_state() {
        let state = WorkspaceState {
            id: Some(WorkspaceId::new()),
            name: Some(WorkspaceName::new("Test").unwrap()),
            owner_id: Some(UserId::new()),
            visibility: Some(Visibility::Private),
            created_at: Some(Utc::now()),
            status: WorkspaceStatus::Active,
        };

        assert!(state.exists());
        assert!(state.is_active());
    }
}
