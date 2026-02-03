//! Workspace bounded context Views for read-side projections.
//!
//! Four views materialize events from the five workspace aggregates into
//! queryable read models optimized for rendering:
//!
//! - `WorkspaceListView`: All workspaces with metadata, filterable by owner
//! - `DashboardLayoutView`: Full dashboard state with charts and tabs
//! - `SavedQueryListView`: All saved queries, filterable by workspace
//! - `UserPreferencesView`: Per-user preferences singleton

use chrono::{DateTime, Utc};
use fmodel_rust::view::View;

use crate::domain::common::DashboardTitle;
use crate::domain::dashboard::events::DashboardEvent;
use crate::domain::dashboard::values::{ChartPlacement, DashboardId, TabInfo};
use crate::domain::saved_query::events::SavedQueryEvent;
use crate::domain::saved_query::values::{QueryName, SavedQueryId};
use crate::domain::session::UserId;
use crate::domain::user_preferences::events::UserPreferencesEvent;
use crate::domain::user_preferences::values::{Locale, PreferencesId, Theme, UiState};
use crate::domain::workspace::events::WorkspaceEvent;
use crate::domain::workspace::values::{Visibility, WorkspaceId, WorkspaceName};

// ============================================================================
// WorkspaceListView
// ============================================================================

/// A single workspace entry in the list view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceListEntry {
    pub workspace_id: WorkspaceId,
    pub name: WorkspaceName,
    pub owner_id: UserId,
    pub visibility: Visibility,
    pub created_at: DateTime<Utc>,
}

/// State materialized by the workspace list view.
///
/// Contains all workspaces in creation order. Use `workspaces_for_user` to
/// filter by owner.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceListViewState {
    pub workspaces: Vec<WorkspaceListEntry>,
    /// Invariant: `count == workspaces.len()`
    pub count: usize,
}

impl WorkspaceListViewState {
    /// Filter workspaces owned by a specific user.
    #[must_use]
    pub fn workspaces_for_user(&self, user_id: &UserId) -> Vec<&WorkspaceListEntry> {
        self.workspaces
            .iter()
            .filter(|w| &w.owner_id == user_id)
            .collect()
    }
}

pub type WorkspaceListView<'a> = View<'a, WorkspaceListViewState, WorkspaceEvent>;

/// Factory function creating a pure workspace list view.
pub fn workspace_list_view<'a>() -> WorkspaceListView<'a> {
    View {
        evolve: Box::new(evolve_workspace_list),
        initial_state: Box::new(WorkspaceListViewState::default),
    }
}

fn evolve_workspace_list(
    state: &WorkspaceListViewState,
    event: &WorkspaceEvent,
) -> WorkspaceListViewState {
    match event {
        WorkspaceEvent::Created {
            workspace_id,
            name,
            owner_id,
            visibility,
            created_at,
        } => {
            let mut workspaces = state.workspaces.clone();
            workspaces.push(WorkspaceListEntry {
                workspace_id: *workspace_id,
                name: name.clone(),
                owner_id: *owner_id,
                visibility: *visibility,
                created_at: *created_at,
            });
            WorkspaceListViewState {
                workspaces,
                count: state.count + 1,
            }
        }

        WorkspaceEvent::Renamed {
            workspace_id,
            new_name,
            ..
        } => {
            let mut workspaces = state.workspaces.clone();
            if let Some(ws) = workspaces
                .iter_mut()
                .find(|w| w.workspace_id == *workspace_id)
            {
                ws.name = new_name.clone();
            }
            WorkspaceListViewState {
                workspaces,
                count: state.count,
            }
        }

        WorkspaceEvent::VisibilityChanged {
            workspace_id,
            new_visibility,
            ..
        } => {
            let mut workspaces = state.workspaces.clone();
            if let Some(ws) = workspaces
                .iter_mut()
                .find(|w| w.workspace_id == *workspace_id)
            {
                ws.visibility = *new_visibility;
            }
            WorkspaceListViewState {
                workspaces,
                count: state.count,
            }
        }
    }
}

// ============================================================================
// DashboardLayoutView
// ============================================================================

/// State materialized by the dashboard layout view.
///
/// Represents the full rendering state of a single dashboard including
/// all chart placements and tab organization.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DashboardLayoutViewState {
    pub dashboard_id: Option<DashboardId>,
    pub workspace_id: Option<WorkspaceId>,
    pub name: Option<DashboardTitle>,
    pub placements: Vec<ChartPlacement>,
    pub tabs: Vec<TabInfo>,
    pub chart_count: usize,
    pub tab_count: usize,
}

pub type DashboardLayoutView<'a> = View<'a, DashboardLayoutViewState, DashboardEvent>;

/// Factory function creating a pure dashboard layout view.
pub fn dashboard_layout_view<'a>() -> DashboardLayoutView<'a> {
    View {
        evolve: Box::new(evolve_dashboard_layout),
        initial_state: Box::new(DashboardLayoutViewState::default),
    }
}

fn evolve_dashboard_layout(
    state: &DashboardLayoutViewState,
    event: &DashboardEvent,
) -> DashboardLayoutViewState {
    match event {
        DashboardEvent::DashboardCreated {
            dashboard_id,
            workspace_id,
            name,
            ..
        } => DashboardLayoutViewState {
            dashboard_id: Some(*dashboard_id),
            workspace_id: Some(*workspace_id),
            name: Some(name.clone()),
            placements: Vec::new(),
            tabs: Vec::new(),
            chart_count: 0,
            tab_count: 0,
        },

        DashboardEvent::DashboardRenamed { name, .. } => DashboardLayoutViewState {
            name: Some(name.clone()),
            ..state.clone()
        },

        DashboardEvent::ChartAdded { placement, .. } => {
            let mut placements = state.placements.clone();
            placements.push(placement.clone());
            DashboardLayoutViewState {
                placements,
                chart_count: state.chart_count + 1,
                ..state.clone()
            }
        }

        DashboardEvent::ChartRemoved { chart_id, .. } => {
            if let Some(idx) = state
                .placements
                .iter()
                .position(|p| p.chart_id == *chart_id)
            {
                let mut placements = state.placements.clone();
                placements.remove(idx);
                DashboardLayoutViewState {
                    placements,
                    chart_count: state.chart_count.saturating_sub(1),
                    ..state.clone()
                }
            } else {
                state.clone()
            }
        }

        DashboardEvent::TabAdded { tab_info, .. } => {
            let mut tabs = state.tabs.clone();
            tabs.push(tab_info.clone());
            DashboardLayoutViewState {
                tabs,
                tab_count: state.tab_count + 1,
                ..state.clone()
            }
        }

        DashboardEvent::TabRemoved { tab_id, .. } => {
            if let Some(idx) = state.tabs.iter().position(|t| t.tab_id == *tab_id) {
                let mut tabs = state.tabs.clone();
                tabs.remove(idx);
                DashboardLayoutViewState {
                    tabs,
                    tab_count: state.tab_count.saturating_sub(1),
                    ..state.clone()
                }
            } else {
                state.clone()
            }
        }

        DashboardEvent::ChartMovedToTab {
            chart_id, tab_id, ..
        } => {
            let mut placements = state.placements.clone();
            if let Some(placement) = placements
                .iter_mut()
                .find(|p| p.chart_id == *chart_id)
            {
                placement.tab_id = Some(*tab_id);
            }
            DashboardLayoutViewState {
                placements,
                ..state.clone()
            }
        }
    }
}

// ============================================================================
// SavedQueryListView
// ============================================================================

/// A single saved query entry in the list view.
#[derive(Debug, Clone, PartialEq)]
pub struct SavedQueryListEntry {
    pub query_id: SavedQueryId,
    pub workspace_id: WorkspaceId,
    pub name: QueryName,
    pub sql: String,
    pub dataset_ref: String,
    pub saved_at: DateTime<Utc>,
}

/// State materialized by the saved query list view.
///
/// Contains all non-deleted queries. Use `queries_for_workspace` to filter
/// by workspace scope.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SavedQueryListViewState {
    pub queries: Vec<SavedQueryListEntry>,
    /// Invariant: `count == queries.len()`
    pub count: usize,
}

impl SavedQueryListViewState {
    /// Filter queries belonging to a specific workspace.
    #[must_use]
    pub fn queries_for_workspace(&self, workspace_id: &WorkspaceId) -> Vec<&SavedQueryListEntry> {
        self.queries
            .iter()
            .filter(|q| &q.workspace_id == workspace_id)
            .collect()
    }
}

pub type SavedQueryListView<'a> = View<'a, SavedQueryListViewState, SavedQueryEvent>;

/// Factory function creating a pure saved query list view.
pub fn saved_query_list_view<'a>() -> SavedQueryListView<'a> {
    View {
        evolve: Box::new(evolve_saved_query_list),
        initial_state: Box::new(SavedQueryListViewState::default),
    }
}

fn evolve_saved_query_list(
    state: &SavedQueryListViewState,
    event: &SavedQueryEvent,
) -> SavedQueryListViewState {
    match event {
        SavedQueryEvent::QuerySaved {
            query_id,
            workspace_id,
            name,
            sql,
            dataset_ref,
            saved_at,
        } => {
            let mut queries = state.queries.clone();
            queries.push(SavedQueryListEntry {
                query_id: *query_id,
                workspace_id: *workspace_id,
                name: name.clone(),
                sql: sql.to_string(),
                dataset_ref: dataset_ref.to_string(),
                saved_at: *saved_at,
            });
            SavedQueryListViewState {
                queries,
                count: state.count + 1,
            }
        }

        SavedQueryEvent::QueryDeleted { query_id, .. } => {
            if let Some(idx) = state.queries.iter().position(|q| q.query_id == *query_id) {
                let mut queries = state.queries.clone();
                queries.remove(idx);
                SavedQueryListViewState {
                    queries,
                    count: state.count.saturating_sub(1),
                }
            } else {
                state.clone()
            }
        }

        SavedQueryEvent::QueryRenamed {
            query_id, name, ..
        } => {
            let mut queries = state.queries.clone();
            if let Some(q) = queries.iter_mut().find(|q| q.query_id == *query_id) {
                q.name = name.clone();
            }
            SavedQueryListViewState {
                queries,
                count: state.count,
            }
        }

        SavedQueryEvent::QuerySqlUpdated { query_id, sql, .. } => {
            let mut queries = state.queries.clone();
            if let Some(q) = queries.iter_mut().find(|q| q.query_id == *query_id) {
                q.sql = sql.to_string();
            }
            SavedQueryListViewState {
                queries,
                count: state.count,
            }
        }

        SavedQueryEvent::DatasetRefUpdated {
            query_id,
            dataset_ref,
            ..
        } => {
            let mut queries = state.queries.clone();
            if let Some(q) = queries.iter_mut().find(|q| q.query_id == *query_id) {
                q.dataset_ref = dataset_ref.to_string();
            }
            SavedQueryListViewState {
                queries,
                count: state.count,
            }
        }
    }
}

// ============================================================================
// UserPreferencesView
// ============================================================================

/// State materialized by the user preferences view.
///
/// Represents the current preferences for a single user. Singleton per
/// user-scoped aggregate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserPreferencesViewState {
    pub preferences_id: Option<PreferencesId>,
    pub user_id: Option<UserId>,
    pub theme: Theme,
    pub locale: Locale,
    pub ui_state: UiState,
    pub initialized: bool,
}

impl Default for UserPreferencesViewState {
    fn default() -> Self {
        Self {
            preferences_id: None,
            user_id: None,
            theme: Theme::default(),
            locale: Locale::default(),
            ui_state: UiState::default(),
            initialized: false,
        }
    }
}

pub type UserPreferencesView<'a> = View<'a, UserPreferencesViewState, UserPreferencesEvent>;

/// Factory function creating a pure user preferences view.
pub fn user_preferences_view<'a>() -> UserPreferencesView<'a> {
    View {
        evolve: Box::new(evolve_user_preferences),
        initial_state: Box::new(UserPreferencesViewState::default),
    }
}

fn evolve_user_preferences(
    state: &UserPreferencesViewState,
    event: &UserPreferencesEvent,
) -> UserPreferencesViewState {
    match event {
        UserPreferencesEvent::PreferencesInitialized {
            preferences_id,
            user_id,
            ..
        } => UserPreferencesViewState {
            preferences_id: Some(*preferences_id),
            user_id: Some(*user_id),
            theme: Theme::default(),
            locale: Locale::default(),
            ui_state: UiState::default(),
            initialized: true,
        },

        UserPreferencesEvent::ThemeSet { theme, .. } => UserPreferencesViewState {
            theme: theme.clone(),
            ..state.clone()
        },

        UserPreferencesEvent::LocaleSet { locale, .. } => UserPreferencesViewState {
            locale: locale.clone(),
            ..state.clone()
        },

        UserPreferencesEvent::UiStateUpdated { ui_state, .. } => UserPreferencesViewState {
            ui_state: ui_state.clone(),
            ..state.clone()
        },
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::analytics::{DatasetRef, SqlQuery};
    use crate::domain::common::{DashboardTitle, GridSize, TabTitle};
    use crate::domain::dashboard::values::{ChartDefinitionRef, GridPosition};
    use fmodel_rust::view::ViewStateComputation;

    fn sample_workspace_id() -> WorkspaceId {
        WorkspaceId::from_uuid(Uuid::nil())
    }

    fn sample_workspace_id_2() -> WorkspaceId {
        WorkspaceId::from_uuid(Uuid::from_u128(1))
    }

    fn sample_owner() -> UserId {
        UserId::from_uuid(Uuid::nil())
    }

    fn sample_owner_2() -> UserId {
        UserId::from_uuid(Uuid::from_u128(2))
    }

    fn sample_name() -> WorkspaceName {
        WorkspaceName::new("Test Workspace").unwrap()
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn as_refs<T>(events: &[T]) -> Vec<&T> {
        events.iter().collect()
    }

    // --- WorkspaceListView ---

    mod workspace_list {
        use super::*;

        #[test]
        fn initial_state_is_empty() {
            let view = workspace_list_view();
            let state = (view.initial_state)();
            assert!(state.workspaces.is_empty());
            assert_eq!(state.count, 0);
        }

        #[test]
        fn created_adds_workspace() {
            let view = workspace_list_view();
            let event = WorkspaceEvent::Created {
                workspace_id: sample_workspace_id(),
                name: sample_name(),
                owner_id: sample_owner(),
                visibility: Visibility::Private,
                created_at: sample_time(),
            };

            let state = view.compute_new_state(None, &[&event]);

            assert_eq!(state.workspaces.len(), 1);
            assert_eq!(state.count, 1);
            assert_eq!(state.workspaces[0].name, sample_name());
        }

        #[test]
        fn renamed_updates_name() {
            let view = workspace_list_view();
            let new_name = WorkspaceName::new("Renamed").unwrap();
            let events = vec![
                WorkspaceEvent::Created {
                    workspace_id: sample_workspace_id(),
                    name: sample_name(),
                    owner_id: sample_owner(),
                    visibility: Visibility::Private,
                    created_at: sample_time(),
                },
                WorkspaceEvent::Renamed {
                    workspace_id: sample_workspace_id(),
                    old_name: sample_name(),
                    new_name: new_name.clone(),
                    renamed_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.workspaces[0].name, new_name);
            assert_eq!(state.count, 1);
        }

        #[test]
        fn visibility_changed_updates_visibility() {
            let view = workspace_list_view();
            let events = vec![
                WorkspaceEvent::Created {
                    workspace_id: sample_workspace_id(),
                    name: sample_name(),
                    owner_id: sample_owner(),
                    visibility: Visibility::Private,
                    created_at: sample_time(),
                },
                WorkspaceEvent::VisibilityChanged {
                    workspace_id: sample_workspace_id(),
                    old_visibility: Visibility::Private,
                    new_visibility: Visibility::Public,
                    changed_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.workspaces[0].visibility, Visibility::Public);
        }

        #[test]
        fn renamed_nonexistent_is_noop() {
            let view = workspace_list_view();
            let events = vec![WorkspaceEvent::Renamed {
                workspace_id: sample_workspace_id(),
                old_name: sample_name(),
                new_name: WorkspaceName::new("New").unwrap(),
                renamed_at: sample_time(),
            }];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert!(state.workspaces.is_empty());
            assert_eq!(state.count, 0);
        }

        #[test]
        fn filter_by_user() {
            let view = workspace_list_view();
            let events = vec![
                WorkspaceEvent::Created {
                    workspace_id: sample_workspace_id(),
                    name: sample_name(),
                    owner_id: sample_owner(),
                    visibility: Visibility::Private,
                    created_at: sample_time(),
                },
                WorkspaceEvent::Created {
                    workspace_id: sample_workspace_id_2(),
                    name: WorkspaceName::new("Other").unwrap(),
                    owner_id: sample_owner_2(),
                    visibility: Visibility::Private,
                    created_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.count, 2);
            let user1_workspaces = state.workspaces_for_user(&sample_owner());
            assert_eq!(user1_workspaces.len(), 1);
            assert_eq!(user1_workspaces[0].workspace_id, sample_workspace_id());
        }
    }

    // --- DashboardLayoutView ---

    mod dashboard_layout {
        use super::*;

        fn sample_dash_id() -> DashboardId {
            DashboardId::from_uuid(Uuid::nil())
        }

        fn sample_chart_id() -> ChartId {
            ChartId::from_uuid(Uuid::nil())
        }

        fn sample_chart_id_2() -> ChartId {
            ChartId::from_uuid(Uuid::from_u128(1))
        }

        fn sample_tab_id() -> TabId {
            TabId::from_uuid(Uuid::nil())
        }

        fn sample_placement(chart_id: ChartId) -> ChartPlacement {
            ChartPlacement {
                chart_id,
                chart_def_ref: ChartDefinitionRef {
                    ref_id: "ref-1".to_string(),
                    chart_type_hint: None,
                },
                position: GridPosition { row: 0, col: 0 },
                size: GridSize::new(4, 3).unwrap(),
                tab_id: None,
            }
        }

        #[test]
        fn initial_state_is_empty() {
            let view = dashboard_layout_view();
            let state = (view.initial_state)();
            assert!(state.dashboard_id.is_none());
            assert!(state.placements.is_empty());
            assert!(state.tabs.is_empty());
            assert_eq!(state.chart_count, 0);
            assert_eq!(state.tab_count, 0);
        }

        #[test]
        fn created_initializes_dashboard() {
            let view = dashboard_layout_view();
            let event = DashboardEvent::DashboardCreated {
                dashboard_id: sample_dash_id(),
                workspace_id: sample_workspace_id(),
                name: DashboardTitle::new("Main").unwrap(),
                created_at: sample_time(),
            };

            let state = view.compute_new_state(None, &[&event]);

            assert_eq!(state.dashboard_id, Some(sample_dash_id()));
            assert_eq!(state.workspace_id, Some(sample_workspace_id()));
            assert!(state.placements.is_empty());
        }

        #[test]
        fn chart_added_increments_count() {
            let view = dashboard_layout_view();
            let events = vec![
                DashboardEvent::DashboardCreated {
                    dashboard_id: sample_dash_id(),
                    workspace_id: sample_workspace_id(),
                    name: DashboardTitle::new("Main").unwrap(),
                    created_at: sample_time(),
                },
                DashboardEvent::ChartAdded {
                    dashboard_id: sample_dash_id(),
                    placement: sample_placement(sample_chart_id()),
                    added_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.placements.len(), 1);
            assert_eq!(state.chart_count, 1);
        }

        #[test]
        fn chart_removed_decrements_count() {
            let view = dashboard_layout_view();
            let events = vec![
                DashboardEvent::DashboardCreated {
                    dashboard_id: sample_dash_id(),
                    workspace_id: sample_workspace_id(),
                    name: DashboardTitle::new("Main").unwrap(),
                    created_at: sample_time(),
                },
                DashboardEvent::ChartAdded {
                    dashboard_id: sample_dash_id(),
                    placement: sample_placement(sample_chart_id()),
                    added_at: sample_time(),
                },
                DashboardEvent::ChartRemoved {
                    dashboard_id: sample_dash_id(),
                    chart_id: sample_chart_id(),
                    removed_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert!(state.placements.is_empty());
            assert_eq!(state.chart_count, 0);
        }

        #[test]
        fn chart_removed_nonexistent_is_noop() {
            let view = dashboard_layout_view();
            let events = vec![
                DashboardEvent::DashboardCreated {
                    dashboard_id: sample_dash_id(),
                    workspace_id: sample_workspace_id(),
                    name: DashboardTitle::new("Main").unwrap(),
                    created_at: sample_time(),
                },
                DashboardEvent::ChartAdded {
                    dashboard_id: sample_dash_id(),
                    placement: sample_placement(sample_chart_id()),
                    added_at: sample_time(),
                },
                DashboardEvent::ChartRemoved {
                    dashboard_id: sample_dash_id(),
                    chart_id: sample_chart_id_2(),
                    removed_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.placements.len(), 1);
            assert_eq!(state.chart_count, 1);
        }

        #[test]
        fn tab_lifecycle() {
            let view = dashboard_layout_view();
            let events = vec![
                DashboardEvent::DashboardCreated {
                    dashboard_id: sample_dash_id(),
                    workspace_id: sample_workspace_id(),
                    name: DashboardTitle::new("Main").unwrap(),
                    created_at: sample_time(),
                },
                DashboardEvent::TabAdded {
                    dashboard_id: sample_dash_id(),
                    tab_info: TabInfo {
                        tab_id: sample_tab_id(),
                        name: TabTitle::new("Overview").unwrap(),
                    },
                    added_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));
            assert_eq!(state.tabs.len(), 1);
            assert_eq!(state.tab_count, 1);

            let remove = vec![DashboardEvent::TabRemoved {
                dashboard_id: sample_dash_id(),
                tab_id: sample_tab_id(),
                removed_at: sample_time(),
            }];
            let state = view.compute_new_state(Some(state), &as_refs(&remove));
            assert!(state.tabs.is_empty());
            assert_eq!(state.tab_count, 0);
        }

        #[test]
        fn chart_moved_to_tab() {
            let view = dashboard_layout_view();
            let events = vec![
                DashboardEvent::DashboardCreated {
                    dashboard_id: sample_dash_id(),
                    workspace_id: sample_workspace_id(),
                    name: DashboardTitle::new("Main").unwrap(),
                    created_at: sample_time(),
                },
                DashboardEvent::ChartAdded {
                    dashboard_id: sample_dash_id(),
                    placement: sample_placement(sample_chart_id()),
                    added_at: sample_time(),
                },
                DashboardEvent::ChartMovedToTab {
                    dashboard_id: sample_dash_id(),
                    chart_id: sample_chart_id(),
                    tab_id: sample_tab_id(),
                    moved_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.placements[0].tab_id, Some(sample_tab_id()));
        }
    }

    // --- SavedQueryListView ---

    mod saved_query_list {
        use super::*;

        fn sample_query_id() -> SavedQueryId {
            SavedQueryId::from_uuid(Uuid::nil())
        }

        fn sample_query_id_2() -> SavedQueryId {
            SavedQueryId::from_uuid(Uuid::from_u128(1))
        }

        #[test]
        fn initial_state_is_empty() {
            let view = saved_query_list_view();
            let state = (view.initial_state)();
            assert!(state.queries.is_empty());
            assert_eq!(state.count, 0);
        }

        #[test]
        fn saved_adds_query() {
            let view = saved_query_list_view();
            let event = SavedQueryEvent::QuerySaved {
                query_id: sample_query_id(),
                workspace_id: sample_workspace_id(),
                name: QueryName::new("Revenue").unwrap(),
                sql: SqlQuery::new("SELECT 1").unwrap(),
                dataset_ref: DatasetRef::new("hf://datasets/test").unwrap(),
                saved_at: sample_time(),
            };

            let state = view.compute_new_state(None, &[&event]);

            assert_eq!(state.queries.len(), 1);
            assert_eq!(state.count, 1);
        }

        #[test]
        fn deleted_removes_query() {
            let view = saved_query_list_view();
            let events = vec![
                SavedQueryEvent::QuerySaved {
                    query_id: sample_query_id(),
                    workspace_id: sample_workspace_id(),
                    name: QueryName::new("Revenue").unwrap(),
                    sql: SqlQuery::new("SELECT 1").unwrap(),
                    dataset_ref: DatasetRef::new("hf://datasets/test").unwrap(),
                    saved_at: sample_time(),
                },
                SavedQueryEvent::QueryDeleted {
                    query_id: sample_query_id(),
                    deleted_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert!(state.queries.is_empty());
            assert_eq!(state.count, 0);
        }

        #[test]
        fn deleted_nonexistent_is_noop() {
            let view = saved_query_list_view();
            let events = vec![SavedQueryEvent::QueryDeleted {
                query_id: sample_query_id(),
                deleted_at: sample_time(),
            }];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert!(state.queries.is_empty());
            assert_eq!(state.count, 0);
        }

        #[test]
        fn renamed_updates_name() {
            let view = saved_query_list_view();
            let new_name = QueryName::new("Updated").unwrap();
            let events = vec![
                SavedQueryEvent::QuerySaved {
                    query_id: sample_query_id(),
                    workspace_id: sample_workspace_id(),
                    name: QueryName::new("Original").unwrap(),
                    sql: SqlQuery::new("SELECT 1").unwrap(),
                    dataset_ref: DatasetRef::new("hf://datasets/test").unwrap(),
                    saved_at: sample_time(),
                },
                SavedQueryEvent::QueryRenamed {
                    query_id: sample_query_id(),
                    name: new_name.clone(),
                    renamed_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.queries[0].name, new_name);
        }

        #[test]
        fn sql_updated_changes_sql() {
            let view = saved_query_list_view();
            let events = vec![
                SavedQueryEvent::QuerySaved {
                    query_id: sample_query_id(),
                    workspace_id: sample_workspace_id(),
                    name: QueryName::new("Test").unwrap(),
                    sql: SqlQuery::new("SELECT 1").unwrap(),
                    dataset_ref: DatasetRef::new("hf://datasets/test").unwrap(),
                    saved_at: sample_time(),
                },
                SavedQueryEvent::QuerySqlUpdated {
                    query_id: sample_query_id(),
                    sql: SqlQuery::new("SELECT 2").unwrap(),
                    updated_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.queries[0].sql, "SELECT 2");
        }

        #[test]
        fn filter_by_workspace() {
            let view = saved_query_list_view();
            let events = vec![
                SavedQueryEvent::QuerySaved {
                    query_id: sample_query_id(),
                    workspace_id: sample_workspace_id(),
                    name: QueryName::new("WS1 Query").unwrap(),
                    sql: SqlQuery::new("SELECT 1").unwrap(),
                    dataset_ref: DatasetRef::new("hf://datasets/test").unwrap(),
                    saved_at: sample_time(),
                },
                SavedQueryEvent::QuerySaved {
                    query_id: sample_query_id_2(),
                    workspace_id: sample_workspace_id_2(),
                    name: QueryName::new("WS2 Query").unwrap(),
                    sql: SqlQuery::new("SELECT 2").unwrap(),
                    dataset_ref: DatasetRef::new("hf://datasets/other").unwrap(),
                    saved_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.count, 2);
            let ws1_queries = state.queries_for_workspace(&sample_workspace_id());
            assert_eq!(ws1_queries.len(), 1);
            assert_eq!(ws1_queries[0].query_id, sample_query_id());
        }

        #[test]
        fn count_invariant_after_delete_nonexistent() {
            let view = saved_query_list_view();
            let events = vec![SavedQueryEvent::QuerySaved {
                query_id: sample_query_id(),
                workspace_id: sample_workspace_id(),
                name: QueryName::new("Keep").unwrap(),
                sql: SqlQuery::new("SELECT 1").unwrap(),
                dataset_ref: DatasetRef::new("hf://datasets/test").unwrap(),
                saved_at: sample_time(),
            }];
            let state = view.compute_new_state(None, &as_refs(&events));

            let delete = vec![SavedQueryEvent::QueryDeleted {
                query_id: sample_query_id_2(),
                deleted_at: sample_time(),
            }];
            let state = view.compute_new_state(Some(state), &as_refs(&delete));

            assert_eq!(state.queries.len(), 1);
            assert_eq!(state.count, state.queries.len());
        }
    }

    // --- UserPreferencesView ---

    mod user_preferences {
        use super::*;

        fn sample_pref_id() -> PreferencesId {
            PreferencesId::from_uuid(Uuid::nil())
        }

        #[test]
        fn initial_state_is_not_initialized() {
            let view = user_preferences_view();
            let state = (view.initial_state)();
            assert!(!state.initialized);
            assert!(state.preferences_id.is_none());
            assert!(state.user_id.is_none());
            assert_eq!(state.theme, Theme::System);
            assert_eq!(state.locale, Locale::default());
        }

        #[test]
        fn initialized_sets_defaults() {
            let view = user_preferences_view();
            let event = UserPreferencesEvent::PreferencesInitialized {
                preferences_id: sample_pref_id(),
                user_id: sample_owner(),
                initialized_at: sample_time(),
            };

            let state = view.compute_new_state(None, &[&event]);

            assert!(state.initialized);
            assert_eq!(state.preferences_id, Some(sample_pref_id()));
            assert_eq!(state.user_id, Some(sample_owner()));
            assert_eq!(state.theme, Theme::System);
        }

        #[test]
        fn theme_set_updates_theme() {
            let view = user_preferences_view();
            let events = vec![
                UserPreferencesEvent::PreferencesInitialized {
                    preferences_id: sample_pref_id(),
                    user_id: sample_owner(),
                    initialized_at: sample_time(),
                },
                UserPreferencesEvent::ThemeSet {
                    user_id: sample_owner(),
                    theme: Theme::Dark,
                    set_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.theme, Theme::Dark);
        }

        #[test]
        fn locale_set_updates_locale() {
            let view = user_preferences_view();
            let locale = Locale::new("ja-JP").unwrap();
            let events = vec![
                UserPreferencesEvent::PreferencesInitialized {
                    preferences_id: sample_pref_id(),
                    user_id: sample_owner(),
                    initialized_at: sample_time(),
                },
                UserPreferencesEvent::LocaleSet {
                    user_id: sample_owner(),
                    locale: locale.clone(),
                    set_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.locale, locale);
        }

        #[test]
        fn ui_state_updated() {
            let view = user_preferences_view();
            let ui_state = UiState::new(r#"{"sidebar":"collapsed"}"#);
            let events = vec![
                UserPreferencesEvent::PreferencesInitialized {
                    preferences_id: sample_pref_id(),
                    user_id: sample_owner(),
                    initialized_at: sample_time(),
                },
                UserPreferencesEvent::UiStateUpdated {
                    user_id: sample_owner(),
                    ui_state: ui_state.clone(),
                    updated_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert_eq!(state.ui_state, ui_state);
        }

        #[test]
        fn full_lifecycle() {
            let view = user_preferences_view();
            let events = vec![
                UserPreferencesEvent::PreferencesInitialized {
                    preferences_id: sample_pref_id(),
                    user_id: sample_owner(),
                    initialized_at: sample_time(),
                },
                UserPreferencesEvent::ThemeSet {
                    user_id: sample_owner(),
                    theme: Theme::Dark,
                    set_at: sample_time(),
                },
                UserPreferencesEvent::LocaleSet {
                    user_id: sample_owner(),
                    locale: Locale::new("fr-FR").unwrap(),
                    set_at: sample_time(),
                },
                UserPreferencesEvent::ThemeSet {
                    user_id: sample_owner(),
                    theme: Theme::Light,
                    set_at: sample_time(),
                },
            ];

            let state = view.compute_new_state(None, &as_refs(&events));

            assert!(state.initialized);
            assert_eq!(state.theme, Theme::Light);
            assert_eq!(state.locale, Locale::new("fr-FR").unwrap());
        }
    }
}
