//! Dashboard aggregate state types.
//!
//! State is derived from events via replay. Uses a sum type enum following
//! the WorkspacePreferences aggregate pattern for clean state machine semantics.

use super::values::{ChartPlacement, DashboardId, TabInfo};
use crate::workspace::WorkspaceId;
use ironstar_core::DashboardTitle;

/// State of a dashboard, derived from events.
///
/// ```text
///                     ┌───────────────────┐
///  CreateDashboard ──►│  DashboardExists  │
///                     └────────┬──────────┘
///                              │
///          ┌───────────────────┼───────────────────┐
///          │         │         │         │          │
///       Rename   AddChart  RemoveChart  AddTab  RemoveTab  MoveChartToTab
///          │         │         │         │          │
///          └───────────────────┴───────────────────-┘
///                              │
///                              ▼
///                     ┌───────────────────┐
///                     │  DashboardExists  │ (updated fields)
///                     └───────────────────┘
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub enum DashboardState {
    /// Initial state before creation.
    #[default]
    NoDashboard,

    /// Dashboard exists with chart placements and tabs.
    DashboardExists {
        /// Unique identifier for this dashboard.
        dashboard_id: DashboardId,
        /// The workspace this dashboard belongs to.
        workspace_id: WorkspaceId,
        /// Display name of the dashboard.
        name: DashboardTitle,
        /// Chart placements on this dashboard.
        placements: Vec<ChartPlacement>,
        /// Tabs for organizing charts.
        tabs: Vec<TabInfo>,
    },
}

impl DashboardState {
    /// Check if the dashboard exists.
    #[must_use]
    pub fn exists(&self) -> bool {
        matches!(self, Self::DashboardExists { .. })
    }

    /// Get the dashboard ID, if it exists.
    #[must_use]
    pub fn dashboard_id(&self) -> Option<&DashboardId> {
        match self {
            Self::NoDashboard => None,
            Self::DashboardExists { dashboard_id, .. } => Some(dashboard_id),
        }
    }

    /// Get the workspace ID, if it exists.
    #[must_use]
    pub fn workspace_id(&self) -> Option<&WorkspaceId> {
        match self {
            Self::NoDashboard => None,
            Self::DashboardExists { workspace_id, .. } => Some(workspace_id),
        }
    }

    /// Get the dashboard name, if it exists.
    #[must_use]
    pub fn name(&self) -> Option<&DashboardTitle> {
        match self {
            Self::NoDashboard => None,
            Self::DashboardExists { name, .. } => Some(name),
        }
    }

    /// Get the chart placements, if dashboard exists.
    #[must_use]
    pub fn placements(&self) -> Option<&[ChartPlacement]> {
        match self {
            Self::NoDashboard => None,
            Self::DashboardExists { placements, .. } => Some(placements),
        }
    }

    /// Get the tabs, if dashboard exists.
    #[must_use]
    pub fn tabs(&self) -> Option<&[TabInfo]> {
        match self {
            Self::NoDashboard => None,
            Self::DashboardExists { tabs, .. } => Some(tabs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::WorkspaceId;
    use ironstar_core::DashboardTitle;

    #[test]
    fn default_state_is_no_dashboard() {
        let state = DashboardState::default();
        assert!(!state.exists());
        assert!(state.dashboard_id().is_none());
        assert!(state.workspace_id().is_none());
        assert!(state.name().is_none());
        assert!(state.placements().is_none());
        assert!(state.tabs().is_none());
    }

    #[test]
    fn dashboard_exists_state() {
        let dash_id = DashboardId::from_uuid(uuid::Uuid::nil());
        let ws_id = WorkspaceId::from_uuid(uuid::Uuid::nil());
        let title = DashboardTitle::new("Test Dashboard").unwrap();

        let state = DashboardState::DashboardExists {
            dashboard_id: dash_id,
            workspace_id: ws_id,
            name: title.clone(),
            placements: vec![],
            tabs: vec![],
        };

        assert!(state.exists());
        assert_eq!(state.dashboard_id(), Some(&dash_id));
        assert_eq!(state.workspace_id(), Some(&ws_id));
        assert_eq!(state.name(), Some(&title));
        assert_eq!(state.placements().unwrap().len(), 0);
        assert_eq!(state.tabs().unwrap().len(), 0);
    }
}
