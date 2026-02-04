//! Domain events for the Dashboard aggregate.
//!
//! Events represent facts that have occurred. They are immutable, past-tense,
//! and self-describing per Hoffman's Law 1.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{ChartId, ChartPlacement, DashboardId, TabId, TabInfo};
use crate::workspace::WorkspaceId;
use ironstar_core::DashboardTitle;
use ironstar_core::{DeciderType, EventType, Identifier, IsFinal};

/// Events emitted by the Dashboard aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "events/")]
pub enum DashboardEvent {
    /// A new dashboard was created.
    DashboardCreated {
        dashboard_id: DashboardId,
        workspace_id: WorkspaceId,
        name: DashboardTitle,
        created_at: DateTime<Utc>,
    },

    /// A dashboard was renamed.
    DashboardRenamed {
        dashboard_id: DashboardId,
        name: DashboardTitle,
        renamed_at: DateTime<Utc>,
    },

    /// A chart was added to the dashboard.
    ChartAdded {
        dashboard_id: DashboardId,
        placement: ChartPlacement,
        added_at: DateTime<Utc>,
    },

    /// A chart was removed from the dashboard.
    ChartRemoved {
        dashboard_id: DashboardId,
        chart_id: ChartId,
        removed_at: DateTime<Utc>,
    },

    /// A tab was added to the dashboard.
    TabAdded {
        dashboard_id: DashboardId,
        tab_info: TabInfo,
        added_at: DateTime<Utc>,
    },

    /// A tab was removed from the dashboard.
    TabRemoved {
        dashboard_id: DashboardId,
        tab_id: TabId,
        removed_at: DateTime<Utc>,
    },

    /// A chart was moved to a specific tab.
    ChartMovedToTab {
        dashboard_id: DashboardId,
        chart_id: ChartId,
        tab_id: TabId,
        moved_at: DateTime<Utc>,
    },
}

impl DashboardEvent {
    /// Extract the dashboard ID this event belongs to.
    #[must_use]
    pub fn dashboard_id(&self) -> DashboardId {
        match self {
            Self::DashboardCreated { dashboard_id, .. }
            | Self::DashboardRenamed { dashboard_id, .. }
            | Self::ChartAdded { dashboard_id, .. }
            | Self::ChartRemoved { dashboard_id, .. }
            | Self::TabAdded { dashboard_id, .. }
            | Self::TabRemoved { dashboard_id, .. }
            | Self::ChartMovedToTab { dashboard_id, .. } => *dashboard_id,
        }
    }

    /// Get the event type name for storage and routing.
    #[must_use]
    pub fn event_type_str(&self) -> &'static str {
        match self {
            Self::DashboardCreated { .. } => "DashboardCreated",
            Self::DashboardRenamed { .. } => "DashboardRenamed",
            Self::ChartAdded { .. } => "ChartAdded",
            Self::ChartRemoved { .. } => "ChartRemoved",
            Self::TabAdded { .. } => "TabAdded",
            Self::TabRemoved { .. } => "TabRemoved",
            Self::ChartMovedToTab { .. } => "ChartMovedToTab",
        }
    }

    /// Get the event version for schema evolution.
    #[must_use]
    pub fn event_version(&self) -> &'static str {
        "1"
    }
}

impl Identifier for DashboardEvent {
    fn identifier(&self) -> String {
        format!("dashboard_{}", self.dashboard_id())
    }
}

impl EventType for DashboardEvent {
    fn event_type(&self) -> String {
        self.event_type_str().to_string()
    }
}

impl DeciderType for DashboardEvent {
    fn decider_type(&self) -> String {
        "Dashboard".to_string()
    }
}

impl IsFinal for DashboardEvent {
    fn is_final(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ironstar_core::DashboardTitle;

    fn sample_dash_id() -> DashboardId {
        DashboardId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn created_event_serializes_with_type_tag() {
        let event = DashboardEvent::DashboardCreated {
            dashboard_id: sample_dash_id(),
            workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
            name: DashboardTitle::new("Test").unwrap(),
            created_at: sample_time(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "DashboardCreated");
        assert_eq!(json["dashboard_id"], "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn identifier_follows_aggregate_id_pattern() {
        let event = DashboardEvent::DashboardCreated {
            dashboard_id: sample_dash_id(),
            workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
            name: DashboardTitle::new("Test").unwrap(),
            created_at: sample_time(),
        };

        assert_eq!(
            event.identifier(),
            "dashboard_00000000-0000-0000-0000-000000000000"
        );
    }

    #[test]
    fn event_type_matches_serde_tag() {
        let events: Vec<(DashboardEvent, &str)> = vec![
            (
                DashboardEvent::DashboardCreated {
                    dashboard_id: sample_dash_id(),
                    workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
                    name: DashboardTitle::new("Test").unwrap(),
                    created_at: sample_time(),
                },
                "DashboardCreated",
            ),
            (
                DashboardEvent::DashboardRenamed {
                    dashboard_id: sample_dash_id(),
                    name: DashboardTitle::new("New Name").unwrap(),
                    renamed_at: sample_time(),
                },
                "DashboardRenamed",
            ),
            (
                DashboardEvent::ChartRemoved {
                    dashboard_id: sample_dash_id(),
                    chart_id: ChartId::from_uuid(uuid::Uuid::nil()),
                    removed_at: sample_time(),
                },
                "ChartRemoved",
            ),
            (
                DashboardEvent::TabRemoved {
                    dashboard_id: sample_dash_id(),
                    tab_id: TabId::from_uuid(uuid::Uuid::nil()),
                    removed_at: sample_time(),
                },
                "TabRemoved",
            ),
            (
                DashboardEvent::ChartMovedToTab {
                    dashboard_id: sample_dash_id(),
                    chart_id: ChartId::from_uuid(uuid::Uuid::nil()),
                    tab_id: TabId::from_uuid(uuid::Uuid::nil()),
                    moved_at: sample_time(),
                },
                "ChartMovedToTab",
            ),
        ];

        for (event, expected_type) in events {
            assert_eq!(event.event_type_str(), expected_type);

            let json = serde_json::to_value(&event).unwrap();
            assert_eq!(json["type"], expected_type);
        }
    }

    #[test]
    fn is_final_returns_false_for_all_events() {
        let events = vec![
            DashboardEvent::DashboardCreated {
                dashboard_id: sample_dash_id(),
                workspace_id: WorkspaceId::from_uuid(uuid::Uuid::nil()),
                name: DashboardTitle::new("Test").unwrap(),
                created_at: sample_time(),
            },
            DashboardEvent::ChartRemoved {
                dashboard_id: sample_dash_id(),
                chart_id: ChartId::from_uuid(uuid::Uuid::nil()),
                removed_at: sample_time(),
            },
        ];

        for event in events {
            assert!(!event.is_final());
        }
    }
}
