//! Workspace query handlers for View-based state computation.
//!
//! Query handlers fetch events and fold them through the workspace Views to
//! compute current state on demand. This follows the same compute-on-demand
//! pattern as the catalog and todo query handlers.

use crate::domain::dashboard::events::DashboardEvent;
use crate::domain::saved_query::events::SavedQueryEvent;
use crate::domain::session::UserId;
use crate::domain::user_preferences::events::UserPreferencesEvent;
use crate::domain::views::{
    DashboardLayoutViewState, SavedQueryListViewState, UserPreferencesViewState,
    WorkspaceListViewState, dashboard_layout_view, saved_query_list_view, user_preferences_view,
    workspace_list_view,
};
use crate::domain::workspace::events::WorkspaceEvent;
use crate::infrastructure::error::InfrastructureError;
use crate::infrastructure::event_store::SqliteEventRepository;

/// Query workspace list state by replaying all workspace events through the view.
///
/// Returns the full list of workspaces. Use `state.workspaces_for_user()` to
/// filter by owner.
pub async fn query_workspace_list<C>(
    repo: &SqliteEventRepository<C, WorkspaceEvent>,
) -> Result<WorkspaceListViewState, InfrastructureError> {
    let events = repo.fetch_all_events_by_type("Workspace").await?;

    let view = workspace_list_view();
    let initial_state = (view.initial_state)();

    let state = events
        .iter()
        .fold(initial_state, |state, (event, _version)| {
            (view.evolve)(&state, event)
        });

    Ok(state)
}

/// Query workspaces owned by a specific user.
///
/// Convenience wrapper over `query_workspace_list` that filters by owner.
pub async fn query_workspaces_for_user<C>(
    repo: &SqliteEventRepository<C, WorkspaceEvent>,
    user_id: &UserId,
) -> Result<WorkspaceListViewState, InfrastructureError> {
    let full_state = query_workspace_list(repo).await?;
    let filtered: Vec<_> = full_state
        .workspaces
        .into_iter()
        .filter(|w| &w.owner_id == user_id)
        .collect();
    let count = filtered.len();
    Ok(WorkspaceListViewState {
        workspaces: filtered,
        count,
    })
}

/// Query the layout state for a specific dashboard by replaying its events.
pub async fn query_dashboard_layout<C>(
    repo: &SqliteEventRepository<C, DashboardEvent>,
    dashboard_id: &str,
) -> Result<DashboardLayoutViewState, InfrastructureError> {
    let events = repo
        .fetch_events_by_aggregate("Dashboard", dashboard_id)
        .await?;

    let view = dashboard_layout_view();
    let initial_state = (view.initial_state)();

    let state = events
        .iter()
        .fold(initial_state, |state, (event, _version)| {
            (view.evolve)(&state, event)
        });

    Ok(state)
}

/// Query all saved queries by replaying events through the list view.
///
/// Returns the full list. Use `state.queries_for_workspace()` to filter
/// by workspace scope.
pub async fn query_saved_query_list<C>(
    repo: &SqliteEventRepository<C, SavedQueryEvent>,
) -> Result<SavedQueryListViewState, InfrastructureError> {
    let events = repo.fetch_all_events_by_type("SavedQuery").await?;

    let view = saved_query_list_view();
    let initial_state = (view.initial_state)();

    let state = events
        .iter()
        .fold(initial_state, |state, (event, _version)| {
            (view.evolve)(&state, event)
        });

    Ok(state)
}

/// Query user preferences by replaying events for the user's preferences aggregate.
pub async fn query_user_preferences<C>(
    repo: &SqliteEventRepository<C, UserPreferencesEvent>,
    user_id: &UserId,
) -> Result<UserPreferencesViewState, InfrastructureError> {
    let aggregate_id = format!("user_{user_id}/preferences");
    let events = repo
        .fetch_events_by_aggregate("UserPreferences", &aggregate_id)
        .await?;

    let view = user_preferences_view();
    let initial_state = (view.initial_state)();

    let state = events
        .iter()
        .fold(initial_state, |state, (event, _version)| {
            (view.evolve)(&state, event)
        });

    Ok(state)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::application::workspace::handle_workspace_command;
    use crate::domain::workspace::commands::WorkspaceCommand;
    use crate::domain::workspace::values::{Visibility, WorkspaceId};
    use crate::infrastructure::event_bus::ZenohEventBus;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Arc;

    async fn create_test_pool() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool");

        sqlx::query(include_str!("../../../migrations/001_events.sql"))
            .execute(&pool)
            .await
            .expect("Failed to run migration");

        pool
    }

    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    #[tokio::test]
    async fn query_empty_returns_no_workspaces() {
        let pool = create_test_pool().await;
        let repo: SqliteEventRepository<WorkspaceCommand, WorkspaceEvent> =
            SqliteEventRepository::new(pool);

        let state = query_workspace_list(&repo)
            .await
            .expect("query should succeed");

        assert!(state.workspaces.is_empty());
        assert_eq!(state.count, 0);
    }

    #[tokio::test]
    async fn query_after_create_returns_workspace() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = WorkspaceCommand::Create {
            workspace_id: WorkspaceId::new(),
            name: "Test Workspace".to_string(),
            owner_id: UserId::new(),
            visibility: Visibility::Private,
            created_at: Utc::now(),
        };
        handle_workspace_command(Arc::clone(&repo), NO_EVENT_BUS, command)
            .await
            .expect("create should succeed");

        let state = query_workspace_list(&repo)
            .await
            .expect("query should succeed");

        assert_eq!(state.workspaces.len(), 1);
        assert_eq!(state.count, 1);
    }

    #[tokio::test]
    async fn query_for_user_filters_by_owner() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let user1 = UserId::new();
        let user2 = UserId::new();

        let cmd1 = WorkspaceCommand::Create {
            workspace_id: WorkspaceId::new(),
            name: "User1 WS".to_string(),
            owner_id: user1,
            visibility: Visibility::Private,
            created_at: Utc::now(),
        };
        let cmd2 = WorkspaceCommand::Create {
            workspace_id: WorkspaceId::new(),
            name: "User2 WS".to_string(),
            owner_id: user2,
            visibility: Visibility::Private,
            created_at: Utc::now(),
        };

        handle_workspace_command(Arc::clone(&repo), NO_EVENT_BUS, cmd1)
            .await
            .expect("create should succeed");
        handle_workspace_command(Arc::clone(&repo), NO_EVENT_BUS, cmd2)
            .await
            .expect("create should succeed");

        let state = query_workspaces_for_user(&repo, &user1)
            .await
            .expect("query should succeed");

        assert_eq!(state.count, 1);
        assert_eq!(state.workspaces[0].owner_id, user1);
    }
}
