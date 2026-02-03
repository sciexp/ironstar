//! WorkspacePreferences aggregate state types.
//!
//! State is derived from events via replay. Uses a sum type enum following
//! the Catalog aggregate pattern for clean state machine semantics.

use super::values::{CatalogUri, LayoutDefaults};
use crate::domain::workspace::WorkspaceId;

/// State of workspace preferences, derived from events.
///
/// ```text
///                    ┌──────────────────┐
///  Initialize ──────►│   Initialized    │
///                    └────────┬─────────┘
///                             │
///          ┌──────────────────┼──────────────────┐
///          │                  │                   │
///   SetDefaultCatalog  ClearDefaultCatalog  UpdateLayoutDefaults
///          │                  │                   │
///          └──────────────────┴──────────────────-┘
///                             │
///                             ▼
///                    ┌──────────────────┐
///                    │   Initialized    │ (updated fields)
///                    └──────────────────┘
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub enum WorkspacePreferencesState {
    /// Initial state before initialization.
    #[default]
    NotInitialized,

    /// Preferences exist for the workspace.
    Initialized {
        /// The workspace these preferences belong to.
        workspace_id: WorkspaceId,
        /// Default DuckDB catalog URI, if set.
        default_catalog: Option<CatalogUri>,
        /// Layout defaults as JSON string.
        layout_defaults: LayoutDefaults,
    },
}

impl WorkspacePreferencesState {
    /// Check if preferences have been initialized.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        matches!(self, Self::Initialized { .. })
    }

    /// Get the workspace ID, if initialized.
    #[must_use]
    pub fn workspace_id(&self) -> Option<&WorkspaceId> {
        match self {
            Self::NotInitialized => None,
            Self::Initialized { workspace_id, .. } => Some(workspace_id),
        }
    }

    /// Get the default catalog URI, if set.
    #[must_use]
    pub fn default_catalog(&self) -> Option<&CatalogUri> {
        match self {
            Self::NotInitialized => None,
            Self::Initialized {
                default_catalog, ..
            } => default_catalog.as_ref(),
        }
    }

    /// Get the layout defaults, if initialized.
    #[must_use]
    pub fn layout_defaults(&self) -> Option<&LayoutDefaults> {
        match self {
            Self::NotInitialized => None,
            Self::Initialized {
                layout_defaults, ..
            } => Some(layout_defaults),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_not_initialized() {
        let state = WorkspacePreferencesState::default();
        assert!(!state.is_initialized());
        assert!(state.workspace_id().is_none());
        assert!(state.default_catalog().is_none());
        assert!(state.layout_defaults().is_none());
    }

    #[test]
    fn initialized_state() {
        let ws_id = WorkspaceId::new();
        let state = WorkspacePreferencesState::Initialized {
            workspace_id: ws_id,
            default_catalog: Some(CatalogUri::new("ducklake:test").unwrap()),
            layout_defaults: LayoutDefaults::default(),
        };

        assert!(state.is_initialized());
        assert_eq!(state.workspace_id(), Some(&ws_id));
        assert_eq!(state.default_catalog().unwrap().as_str(), "ducklake:test");
        assert_eq!(state.layout_defaults().unwrap().as_str(), "{}");
    }
}
