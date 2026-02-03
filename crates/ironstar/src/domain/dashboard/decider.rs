//! Pure Dashboard Decider implementing fmodel-rust patterns.
//!
//! The Decider embodies the state machine from
//! `spec/Workspace/Dashboard.idr`. It is a pure function with no side
//! effects: all I/O (timestamps, validation) happens at boundaries.
//!
//! # State Machine
//!
//! ```text
//!                     ┌───────────────────┐
//!  CreateDashboard ──►│  DashboardExists  │
//!                     └────────┬──────────┘
//!                              │
//!          ┌───────────────────┼───────────────────┐
//!          │         │         │         │          │
//!       Rename   AddChart  RemoveChart  AddTab  RemoveTab  MoveChartToTab
//!          │         │         │         │          │
//!          └───────────────────┴───────────────────-┘
//!                              │
//!                              ▼
//!                     ┌───────────────────┐
//!                     │  DashboardExists  │ (updated fields)
//!                     └───────────────────┘
//! ```
//!
//! # Idempotency
//!
//! - RenameDashboard with same name returns `Ok(vec![])`
//! - AddChart with existing chart_id returns `Ok(vec![])`
//! - RemoveChart with missing chart_id returns `Ok(vec![])`
//! - AddTab with existing tab_id returns `Ok(vec![])`

use fmodel_rust::decider::Decider;

use super::commands::DashboardCommand;
use super::errors::DashboardError;
use super::events::DashboardEvent;
use super::state::DashboardState;
use super::values::ChartPlacement;

/// Type alias for the Dashboard Decider.
pub type DashboardDecider<'a> = Decider<
    'a,
    DashboardCommand,
    DashboardState,
    DashboardEvent,
    DashboardError,
>;

/// Factory function creating a pure Dashboard Decider.
///
/// Translates the specification from `spec/Workspace/Dashboard.idr`
/// into Rust, preserving the state machine transitions and idempotency invariants.
pub fn dashboard_decider<'a>() -> DashboardDecider<'a> {
    Decider {
        decide: Box::new(decide),
        evolve: Box::new(evolve),
        initial_state: Box::new(DashboardState::default),
    }
}

/// Pure decide function: (Command, State) -> Result<Vec<Event>, Error>
fn decide(
    command: &DashboardCommand,
    state: &DashboardState,
) -> Result<Vec<DashboardEvent>, DashboardError> {
    match (command, state) {
        // CreateDashboard: NoDashboard -> DashboardExists
        (
            DashboardCommand::CreateDashboard {
                dashboard_id,
                workspace_id,
                name,
                created_at,
            },
            DashboardState::NoDashboard,
        ) => Ok(vec![DashboardEvent::DashboardCreated {
            dashboard_id: *dashboard_id,
            workspace_id: *workspace_id,
            name: name.clone(),
            created_at: *created_at,
        }]),

        // CreateDashboard when already exists
        (
            DashboardCommand::CreateDashboard { .. },
            DashboardState::DashboardExists { .. },
        ) => Err(DashboardError::already_exists()),

        // RenameDashboard: DashboardExists -> DashboardExists (idempotent if same name)
        (
            DashboardCommand::RenameDashboard {
                dashboard_id,
                name,
                renamed_at,
            },
            DashboardState::DashboardExists {
                name: current_name, ..
            },
        ) => {
            if current_name == name {
                return Ok(vec![]);
            }

            Ok(vec![DashboardEvent::DashboardRenamed {
                dashboard_id: *dashboard_id,
                name: name.clone(),
                renamed_at: *renamed_at,
            }])
        }

        // RenameDashboard when not created
        (
            DashboardCommand::RenameDashboard { .. },
            DashboardState::NoDashboard,
        ) => Err(DashboardError::not_found()),

        // AddChart: DashboardExists -> DashboardExists (idempotent on duplicate chart_id)
        (
            DashboardCommand::AddChart {
                dashboard_id,
                placement,
                added_at,
            },
            DashboardState::DashboardExists { placements, .. },
        ) => {
            if placements.iter().any(|p| p.chart_id == placement.chart_id) {
                return Ok(vec![]);
            }

            Ok(vec![DashboardEvent::ChartAdded {
                dashboard_id: *dashboard_id,
                placement: placement.clone(),
                added_at: *added_at,
            }])
        }

        // AddChart when not created
        (
            DashboardCommand::AddChart { .. },
            DashboardState::NoDashboard,
        ) => Err(DashboardError::not_found()),

        // RemoveChart: DashboardExists -> DashboardExists (idempotent on missing)
        (
            DashboardCommand::RemoveChart {
                dashboard_id,
                chart_id,
                removed_at,
            },
            DashboardState::DashboardExists { placements, .. },
        ) => {
            if !placements.iter().any(|p| p.chart_id == *chart_id) {
                return Ok(vec![]);
            }

            Ok(vec![DashboardEvent::ChartRemoved {
                dashboard_id: *dashboard_id,
                chart_id: *chart_id,
                removed_at: *removed_at,
            }])
        }

        // RemoveChart when not created
        (
            DashboardCommand::RemoveChart { .. },
            DashboardState::NoDashboard,
        ) => Err(DashboardError::not_found()),

        // AddTab: DashboardExists -> DashboardExists (idempotent on duplicate tab_id)
        (
            DashboardCommand::AddTab {
                dashboard_id,
                tab_info,
                added_at,
            },
            DashboardState::DashboardExists { tabs, .. },
        ) => {
            if tabs.iter().any(|t| t.tab_id == tab_info.tab_id) {
                return Ok(vec![]);
            }

            Ok(vec![DashboardEvent::TabAdded {
                dashboard_id: *dashboard_id,
                tab_info: tab_info.clone(),
                added_at: *added_at,
            }])
        }

        // AddTab when not created
        (
            DashboardCommand::AddTab { .. },
            DashboardState::NoDashboard,
        ) => Err(DashboardError::not_found()),

        // RemoveTab: DashboardExists -> error if not found, else remove
        (
            DashboardCommand::RemoveTab {
                dashboard_id,
                tab_id,
                removed_at,
            },
            DashboardState::DashboardExists { tabs, .. },
        ) => {
            if !tabs.iter().any(|t| t.tab_id == *tab_id) {
                return Err(DashboardError::tab_not_found());
            }

            Ok(vec![DashboardEvent::TabRemoved {
                dashboard_id: *dashboard_id,
                tab_id: *tab_id,
                removed_at: *removed_at,
            }])
        }

        // RemoveTab when not created
        (
            DashboardCommand::RemoveTab { .. },
            DashboardState::NoDashboard,
        ) => Err(DashboardError::not_found()),

        // MoveChartToTab: DashboardExists -> check chart and tab exist
        (
            DashboardCommand::MoveChartToTab {
                dashboard_id,
                chart_id,
                tab_id,
                moved_at,
            },
            DashboardState::DashboardExists {
                placements, tabs, ..
            },
        ) => {
            if !placements.iter().any(|p| p.chart_id == *chart_id) {
                return Err(DashboardError::chart_not_found());
            }

            if !tabs.iter().any(|t| t.tab_id == *tab_id) {
                return Err(DashboardError::tab_not_found());
            }

            Ok(vec![DashboardEvent::ChartMovedToTab {
                dashboard_id: *dashboard_id,
                chart_id: *chart_id,
                tab_id: *tab_id,
                moved_at: *moved_at,
            }])
        }

        // MoveChartToTab when not created
        (
            DashboardCommand::MoveChartToTab { .. },
            DashboardState::NoDashboard,
        ) => Err(DashboardError::not_found()),
    }
}

/// Pure evolve function: (State, Event) -> State
fn evolve(state: &DashboardState, event: &DashboardEvent) -> DashboardState {
    match event {
        DashboardEvent::DashboardCreated {
            dashboard_id,
            workspace_id,
            name,
            ..
        } => DashboardState::DashboardExists {
            dashboard_id: *dashboard_id,
            workspace_id: *workspace_id,
            name: name.clone(),
            placements: vec![],
            tabs: vec![],
        },

        DashboardEvent::DashboardRenamed { name, .. } => match state {
            DashboardState::DashboardExists {
                dashboard_id,
                workspace_id,
                placements,
                tabs,
                ..
            } => DashboardState::DashboardExists {
                dashboard_id: *dashboard_id,
                workspace_id: *workspace_id,
                name: name.clone(),
                placements: placements.clone(),
                tabs: tabs.clone(),
            },
            DashboardState::NoDashboard => state.clone(),
        },

        DashboardEvent::ChartAdded { placement, .. } => match state {
            DashboardState::DashboardExists {
                dashboard_id,
                workspace_id,
                name,
                placements,
                tabs,
            } => {
                let mut new_placements = placements.clone();
                new_placements.push(placement.clone());
                DashboardState::DashboardExists {
                    dashboard_id: *dashboard_id,
                    workspace_id: *workspace_id,
                    name: name.clone(),
                    placements: new_placements,
                    tabs: tabs.clone(),
                }
            }
            DashboardState::NoDashboard => state.clone(),
        },

        DashboardEvent::ChartRemoved { chart_id, .. } => match state {
            DashboardState::DashboardExists {
                dashboard_id,
                workspace_id,
                name,
                placements,
                tabs,
            } => DashboardState::DashboardExists {
                dashboard_id: *dashboard_id,
                workspace_id: *workspace_id,
                name: name.clone(),
                placements: placements
                    .iter()
                    .filter(|p| p.chart_id != *chart_id)
                    .cloned()
                    .collect(),
                tabs: tabs.clone(),
            },
            DashboardState::NoDashboard => state.clone(),
        },

        DashboardEvent::TabAdded { tab_info, .. } => match state {
            DashboardState::DashboardExists {
                dashboard_id,
                workspace_id,
                name,
                placements,
                tabs,
            } => {
                let mut new_tabs = tabs.clone();
                new_tabs.push(tab_info.clone());
                DashboardState::DashboardExists {
                    dashboard_id: *dashboard_id,
                    workspace_id: *workspace_id,
                    name: name.clone(),
                    placements: placements.clone(),
                    tabs: new_tabs,
                }
            }
            DashboardState::NoDashboard => state.clone(),
        },

        DashboardEvent::TabRemoved { tab_id, .. } => match state {
            DashboardState::DashboardExists {
                dashboard_id,
                workspace_id,
                name,
                placements,
                tabs,
            } => DashboardState::DashboardExists {
                dashboard_id: *dashboard_id,
                workspace_id: *workspace_id,
                name: name.clone(),
                placements: placements
                    .iter()
                    .filter(|p| p.tab_id != Some(*tab_id))
                    .cloned()
                    .collect(),
                tabs: tabs
                    .iter()
                    .filter(|t| t.tab_id != *tab_id)
                    .cloned()
                    .collect(),
            },
            DashboardState::NoDashboard => state.clone(),
        },

        DashboardEvent::ChartMovedToTab {
            chart_id, tab_id, ..
        } => match state {
            DashboardState::DashboardExists {
                dashboard_id,
                workspace_id,
                name,
                placements,
                tabs,
            } => DashboardState::DashboardExists {
                dashboard_id: *dashboard_id,
                workspace_id: *workspace_id,
                name: name.clone(),
                placements: placements
                    .iter()
                    .map(|p| {
                        if p.chart_id == *chart_id {
                            ChartPlacement {
                                tab_id: Some(*tab_id),
                                ..p.clone()
                            }
                        } else {
                            p.clone()
                        }
                    })
                    .collect(),
                tabs: tabs.clone(),
            },
            DashboardState::NoDashboard => state.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::values::{
        ChartDefinitionRef, ChartId, ChartPlacement, DashboardId, GridPosition, TabId, TabInfo,
    };
    use chrono::{DateTime, Utc};
    use fmodel_rust::specification::DeciderTestSpecification;

    use crate::domain::analytics::ChartType;
    use crate::domain::common::{DashboardTitle, GridSize, TabTitle};
    use crate::domain::workspace::WorkspaceId;

    fn sample_dashboard_id() -> DashboardId {
        DashboardId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_workspace_id() -> WorkspaceId {
        WorkspaceId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn sample_title() -> DashboardTitle {
        DashboardTitle::new("Sales Dashboard").unwrap()
    }

    fn sample_chart_id() -> ChartId {
        ChartId::from_uuid(uuid::Uuid::from_u128(1))
    }

    fn sample_tab_id() -> TabId {
        TabId::from_uuid(uuid::Uuid::from_u128(2))
    }

    fn sample_placement() -> ChartPlacement {
        ChartPlacement {
            chart_id: sample_chart_id(),
            chart_def_ref: ChartDefinitionRef {
                ref_id: "chart-def-001".to_string(),
                chart_type_hint: Some(ChartType::Bar),
            },
            position: GridPosition { row: 0, col: 0 },
            size: GridSize::new(4, 3).unwrap(),
            tab_id: None,
        }
    }

    fn sample_tab_info() -> TabInfo {
        TabInfo {
            tab_id: sample_tab_id(),
            name: TabTitle::new("Overview").unwrap(),
        }
    }

    fn created_event() -> DashboardEvent {
        DashboardEvent::DashboardCreated {
            dashboard_id: sample_dashboard_id(),
            workspace_id: sample_workspace_id(),
            name: sample_title(),
            created_at: sample_time(),
        }
    }

    // --- CreateDashboard transitions ---

    #[test]
    fn create_dashboard_from_no_dashboard_succeeds() {
        let dash_id = sample_dashboard_id();
        let ws_id = sample_workspace_id();
        let ts = sample_time();
        let title = sample_title();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![])
            .when(DashboardCommand::CreateDashboard {
                dashboard_id: dash_id,
                workspace_id: ws_id,
                name: title.clone(),
                created_at: ts,
            })
            .then(vec![DashboardEvent::DashboardCreated {
                dashboard_id: dash_id,
                workspace_id: ws_id,
                name: title,
                created_at: ts,
            }]);
    }

    #[test]
    fn create_dashboard_when_already_exists_fails() {
        let dash_id = sample_dashboard_id();
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![created_event()])
            .when(DashboardCommand::CreateDashboard {
                dashboard_id: dash_id,
                workspace_id: ws_id,
                name: sample_title(),
                created_at: ts,
            })
            .then_error(DashboardError::already_exists());
    }

    // --- RenameDashboard transitions ---

    #[test]
    fn rename_dashboard_succeeds() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();
        let new_name = DashboardTitle::new("Revenue Dashboard").unwrap();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![created_event()])
            .when(DashboardCommand::RenameDashboard {
                dashboard_id: dash_id,
                name: new_name.clone(),
                renamed_at: ts,
            })
            .then(vec![DashboardEvent::DashboardRenamed {
                dashboard_id: dash_id,
                name: new_name,
                renamed_at: ts,
            }]);
    }

    #[test]
    fn rename_dashboard_same_name_is_idempotent() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![created_event()])
            .when(DashboardCommand::RenameDashboard {
                dashboard_id: dash_id,
                name: sample_title(),
                renamed_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn rename_dashboard_not_found_fails() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![])
            .when(DashboardCommand::RenameDashboard {
                dashboard_id: dash_id,
                name: DashboardTitle::new("New Name").unwrap(),
                renamed_at: ts,
            })
            .then_error(DashboardError::not_found());
    }

    // --- AddChart transitions ---

    #[test]
    fn add_chart_succeeds() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();
        let placement = sample_placement();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![created_event()])
            .when(DashboardCommand::AddChart {
                dashboard_id: dash_id,
                placement: placement.clone(),
                added_at: ts,
            })
            .then(vec![DashboardEvent::ChartAdded {
                dashboard_id: dash_id,
                placement,
                added_at: ts,
            }]);
    }

    #[test]
    fn add_chart_duplicate_chart_id_is_idempotent() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();
        let placement = sample_placement();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![
                created_event(),
                DashboardEvent::ChartAdded {
                    dashboard_id: dash_id,
                    placement: placement.clone(),
                    added_at: ts,
                },
            ])
            .when(DashboardCommand::AddChart {
                dashboard_id: dash_id,
                placement,
                added_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn add_chart_not_found_fails() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![])
            .when(DashboardCommand::AddChart {
                dashboard_id: dash_id,
                placement: sample_placement(),
                added_at: ts,
            })
            .then_error(DashboardError::not_found());
    }

    // --- RemoveChart transitions ---

    #[test]
    fn remove_chart_succeeds() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();
        let placement = sample_placement();
        let chart_id = placement.chart_id;

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![
                created_event(),
                DashboardEvent::ChartAdded {
                    dashboard_id: dash_id,
                    placement,
                    added_at: ts,
                },
            ])
            .when(DashboardCommand::RemoveChart {
                dashboard_id: dash_id,
                chart_id,
                removed_at: ts,
            })
            .then(vec![DashboardEvent::ChartRemoved {
                dashboard_id: dash_id,
                chart_id,
                removed_at: ts,
            }]);
    }

    #[test]
    fn remove_chart_missing_is_idempotent() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![created_event()])
            .when(DashboardCommand::RemoveChart {
                dashboard_id: dash_id,
                chart_id: sample_chart_id(),
                removed_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn remove_chart_not_found_fails() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![])
            .when(DashboardCommand::RemoveChart {
                dashboard_id: dash_id,
                chart_id: sample_chart_id(),
                removed_at: ts,
            })
            .then_error(DashboardError::not_found());
    }

    // --- AddTab transitions ---

    #[test]
    fn add_tab_succeeds() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();
        let tab_info = sample_tab_info();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![created_event()])
            .when(DashboardCommand::AddTab {
                dashboard_id: dash_id,
                tab_info: tab_info.clone(),
                added_at: ts,
            })
            .then(vec![DashboardEvent::TabAdded {
                dashboard_id: dash_id,
                tab_info,
                added_at: ts,
            }]);
    }

    #[test]
    fn add_tab_duplicate_tab_id_is_idempotent() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();
        let tab_info = sample_tab_info();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![
                created_event(),
                DashboardEvent::TabAdded {
                    dashboard_id: dash_id,
                    tab_info: tab_info.clone(),
                    added_at: ts,
                },
            ])
            .when(DashboardCommand::AddTab {
                dashboard_id: dash_id,
                tab_info,
                added_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn add_tab_not_found_fails() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![])
            .when(DashboardCommand::AddTab {
                dashboard_id: dash_id,
                tab_info: sample_tab_info(),
                added_at: ts,
            })
            .then_error(DashboardError::not_found());
    }

    // --- RemoveTab transitions ---

    #[test]
    fn remove_tab_succeeds() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();
        let tab_info = sample_tab_info();
        let tab_id = tab_info.tab_id;

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![
                created_event(),
                DashboardEvent::TabAdded {
                    dashboard_id: dash_id,
                    tab_info,
                    added_at: ts,
                },
            ])
            .when(DashboardCommand::RemoveTab {
                dashboard_id: dash_id,
                tab_id,
                removed_at: ts,
            })
            .then(vec![DashboardEvent::TabRemoved {
                dashboard_id: dash_id,
                tab_id,
                removed_at: ts,
            }]);
    }

    #[test]
    fn remove_tab_missing_fails() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![created_event()])
            .when(DashboardCommand::RemoveTab {
                dashboard_id: dash_id,
                tab_id: sample_tab_id(),
                removed_at: ts,
            })
            .then_error(DashboardError::tab_not_found());
    }

    #[test]
    fn remove_tab_not_found_fails() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![])
            .when(DashboardCommand::RemoveTab {
                dashboard_id: dash_id,
                tab_id: sample_tab_id(),
                removed_at: ts,
            })
            .then_error(DashboardError::not_found());
    }

    // --- MoveChartToTab transitions ---

    #[test]
    fn move_chart_to_tab_succeeds() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();
        let placement = sample_placement();
        let chart_id = placement.chart_id;
        let tab_info = sample_tab_info();
        let tab_id = tab_info.tab_id;

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![
                created_event(),
                DashboardEvent::ChartAdded {
                    dashboard_id: dash_id,
                    placement,
                    added_at: ts,
                },
                DashboardEvent::TabAdded {
                    dashboard_id: dash_id,
                    tab_info,
                    added_at: ts,
                },
            ])
            .when(DashboardCommand::MoveChartToTab {
                dashboard_id: dash_id,
                chart_id,
                tab_id,
                moved_at: ts,
            })
            .then(vec![DashboardEvent::ChartMovedToTab {
                dashboard_id: dash_id,
                chart_id,
                tab_id,
                moved_at: ts,
            }]);
    }

    #[test]
    fn move_chart_to_tab_missing_chart_fails() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();
        let tab_info = sample_tab_info();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![
                created_event(),
                DashboardEvent::TabAdded {
                    dashboard_id: dash_id,
                    tab_info,
                    added_at: ts,
                },
            ])
            .when(DashboardCommand::MoveChartToTab {
                dashboard_id: dash_id,
                chart_id: sample_chart_id(),
                tab_id: sample_tab_id(),
                moved_at: ts,
            })
            .then_error(DashboardError::chart_not_found());
    }

    #[test]
    fn move_chart_to_tab_missing_tab_fails() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();
        let placement = sample_placement();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![
                created_event(),
                DashboardEvent::ChartAdded {
                    dashboard_id: dash_id,
                    placement,
                    added_at: ts,
                },
            ])
            .when(DashboardCommand::MoveChartToTab {
                dashboard_id: dash_id,
                chart_id: sample_chart_id(),
                tab_id: sample_tab_id(),
                moved_at: ts,
            })
            .then_error(DashboardError::tab_not_found());
    }

    #[test]
    fn move_chart_to_tab_not_found_fails() {
        let dash_id = sample_dashboard_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(dashboard_decider())
            .given(vec![])
            .when(DashboardCommand::MoveChartToTab {
                dashboard_id: dash_id,
                chart_id: sample_chart_id(),
                tab_id: sample_tab_id(),
                moved_at: ts,
            })
            .then_error(DashboardError::not_found());
    }

    // --- Full lifecycle ---

    #[test]
    fn full_lifecycle() {
        let dash_id = sample_dashboard_id();
        let ws_id = sample_workspace_id();
        let ts = sample_time();
        let title = sample_title();

        // Create dashboard
        let events = decide(
            &DashboardCommand::CreateDashboard {
                dashboard_id: dash_id,
                workspace_id: ws_id,
                name: title,
                created_at: ts,
            },
            &DashboardState::default(),
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&DashboardState::default(), &events[0]);
        assert!(state.exists());
        assert_eq!(state.placements().unwrap().len(), 0);
        assert_eq!(state.tabs().unwrap().len(), 0);

        // Add a tab
        let tab_info = sample_tab_info();
        let events = decide(
            &DashboardCommand::AddTab {
                dashboard_id: dash_id,
                tab_info: tab_info.clone(),
                added_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.tabs().unwrap().len(), 1);

        // Add a chart
        let placement = sample_placement();
        let chart_id = placement.chart_id;
        let events = decide(
            &DashboardCommand::AddChart {
                dashboard_id: dash_id,
                placement: placement.clone(),
                added_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.placements().unwrap().len(), 1);

        // Idempotent: add same chart again
        let events = decide(
            &DashboardCommand::AddChart {
                dashboard_id: dash_id,
                placement,
                added_at: ts,
            },
            &state,
        )
        .unwrap();
        assert!(events.is_empty());

        // Move chart to tab
        let tab_id = tab_info.tab_id;
        let events = decide(
            &DashboardCommand::MoveChartToTab {
                dashboard_id: dash_id,
                chart_id,
                tab_id,
                moved_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(
            state.placements().unwrap()[0].tab_id,
            Some(tab_id)
        );

        // Rename
        let new_name = DashboardTitle::new("Revenue Dashboard").unwrap();
        let events = decide(
            &DashboardCommand::RenameDashboard {
                dashboard_id: dash_id,
                name: new_name.clone(),
                renamed_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.name().unwrap(), &new_name);

        // Idempotent rename
        let events = decide(
            &DashboardCommand::RenameDashboard {
                dashboard_id: dash_id,
                name: new_name,
                renamed_at: ts,
            },
            &state,
        )
        .unwrap();
        assert!(events.is_empty());

        // Remove tab (should also remove chart placements on that tab)
        let events = decide(
            &DashboardCommand::RemoveTab {
                dashboard_id: dash_id,
                tab_id,
                removed_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.tabs().unwrap().len(), 0);
        // Chart was on the removed tab, so it should be filtered out
        assert_eq!(state.placements().unwrap().len(), 0);
    }
}
