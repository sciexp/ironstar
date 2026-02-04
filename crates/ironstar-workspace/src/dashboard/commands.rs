//! Commands for the Dashboard aggregate.
//!
//! Commands represent requests to change dashboard state.
//! Timestamps are injected at the boundary layer; the pure decider
//! does not call `Utc::now()`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{ChartId, ChartPlacement, DashboardId, TabId, TabInfo};
use crate::workspace::WorkspaceId;
use ironstar_core::DashboardTitle;
use ironstar_core::{DeciderType, Identifier};

/// Commands that can be sent to the Dashboard aggregate.
///
/// The aggregate ID follows the pattern `dashboard_{dashboard_id}`,
/// supporting multiple dashboards per workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "commands/")]
pub enum DashboardCommand {
    /// Create a new dashboard for a workspace.
    ///
    /// Can only succeed when the dashboard does not yet exist.
    CreateDashboard {
        dashboard_id: DashboardId,
        workspace_id: WorkspaceId,
        name: DashboardTitle,
        created_at: DateTime<Utc>,
    },

    /// Rename an existing dashboard.
    ///
    /// Idempotent when setting the same name.
    RenameDashboard {
        dashboard_id: DashboardId,
        name: DashboardTitle,
        renamed_at: DateTime<Utc>,
    },

    /// Add a chart to the dashboard.
    ///
    /// Idempotent when the chart_id already exists in placements.
    AddChart {
        dashboard_id: DashboardId,
        placement: ChartPlacement,
        added_at: DateTime<Utc>,
    },

    /// Remove a chart from the dashboard.
    ///
    /// Idempotent when the chart does not exist.
    RemoveChart {
        dashboard_id: DashboardId,
        chart_id: ChartId,
        removed_at: DateTime<Utc>,
    },

    /// Add a tab to the dashboard.
    ///
    /// Idempotent when the tab_id already exists.
    AddTab {
        dashboard_id: DashboardId,
        tab_info: TabInfo,
        added_at: DateTime<Utc>,
    },

    /// Remove a tab from the dashboard.
    ///
    /// Fails if the tab does not exist.
    RemoveTab {
        dashboard_id: DashboardId,
        tab_id: TabId,
        removed_at: DateTime<Utc>,
    },

    /// Move a chart to a specific tab.
    ///
    /// Fails if the chart or tab does not exist.
    MoveChartToTab {
        dashboard_id: DashboardId,
        chart_id: ChartId,
        tab_id: TabId,
        moved_at: DateTime<Utc>,
    },
}

impl DashboardCommand {
    /// Extract the dashboard ID from the command.
    #[must_use]
    pub fn dashboard_id(&self) -> DashboardId {
        match self {
            Self::CreateDashboard { dashboard_id, .. }
            | Self::RenameDashboard { dashboard_id, .. }
            | Self::AddChart { dashboard_id, .. }
            | Self::RemoveChart { dashboard_id, .. }
            | Self::AddTab { dashboard_id, .. }
            | Self::RemoveTab { dashboard_id, .. }
            | Self::MoveChartToTab { dashboard_id, .. } => *dashboard_id,
        }
    }

    /// Get the command type name for logging and metrics.
    #[must_use]
    pub fn command_type(&self) -> &'static str {
        match self {
            Self::CreateDashboard { .. } => "CreateDashboard",
            Self::RenameDashboard { .. } => "RenameDashboard",
            Self::AddChart { .. } => "AddChart",
            Self::RemoveChart { .. } => "RemoveChart",
            Self::AddTab { .. } => "AddTab",
            Self::RemoveTab { .. } => "RemoveTab",
            Self::MoveChartToTab { .. } => "MoveChartToTab",
        }
    }
}

impl Identifier for DashboardCommand {
    fn identifier(&self) -> String {
        format!("dashboard_{}", self.dashboard_id())
    }
}

impl DeciderType for DashboardCommand {
    fn decider_type(&self) -> String {
        "Dashboard".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ironstar_core::DashboardTitle;

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn create_command_serializes_with_type_tag() {
        let dash_id = DashboardId::from_uuid(uuid::Uuid::nil());
        let ws_id = WorkspaceId::from_uuid(uuid::Uuid::nil());
        let cmd = DashboardCommand::CreateDashboard {
            dashboard_id: dash_id,
            workspace_id: ws_id,
            name: DashboardTitle::new("Test").unwrap(),
            created_at: sample_time(),
        };

        let json = serde_json::to_value(&cmd).unwrap();
        assert_eq!(json["type"], "CreateDashboard");
        assert_eq!(json["dashboard_id"], "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn identifier_follows_aggregate_id_pattern() {
        let dash_id = DashboardId::from_uuid(uuid::Uuid::nil());
        let ws_id = WorkspaceId::from_uuid(uuid::Uuid::nil());
        let cmd = DashboardCommand::CreateDashboard {
            dashboard_id: dash_id,
            workspace_id: ws_id,
            name: DashboardTitle::new("Test").unwrap(),
            created_at: sample_time(),
        };

        assert_eq!(
            cmd.identifier(),
            "dashboard_00000000-0000-0000-0000-000000000000"
        );
    }

    #[test]
    fn dashboard_id_extracts_correctly() {
        let dash_id = DashboardId::from_uuid(uuid::Uuid::nil());
        let ts = sample_time();

        let commands = vec![
            DashboardCommand::CreateDashboard {
                dashboard_id: dash_id,
                workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
                name: DashboardTitle::new("Test").unwrap(),
                created_at: ts,
            },
            DashboardCommand::RenameDashboard {
                dashboard_id: dash_id,
                name: DashboardTitle::new("New Name").unwrap(),
                renamed_at: ts,
            },
            DashboardCommand::RemoveChart {
                dashboard_id: dash_id,
                chart_id: ChartId::from_uuid(uuid::Uuid::nil()),
                removed_at: ts,
            },
        ];

        for cmd in commands {
            assert_eq!(cmd.dashboard_id(), dash_id);
        }
    }
}
