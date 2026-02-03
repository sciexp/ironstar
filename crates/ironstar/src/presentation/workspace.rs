//! Workspace bounded context HTTP handlers.
//!
//! This module provides axum handlers for Workspace command and query endpoints
//! covering all five aggregate types: Workspace, Dashboard, SavedQuery,
//! UserPreferences, and WorkspacePreferences.
//!
//! # Routes
//!
//! Query endpoints:
//! - `GET /api` - List all workspaces
//! - `GET /api/{id}/dashboard/{dashboard_id}` - Get dashboard layout
//! - `GET /api/{id}/queries` - List saved queries for a workspace
//! - `GET /api/user/preferences/{user_id}` - Get user preferences
//!
//! Workspace lifecycle:
//! - `POST /api` - Create a new workspace
//! - `POST /api/{id}/rename` - Rename a workspace
//! - `POST /api/{id}/visibility` - Change workspace visibility
//!
//! Dashboard management:
//! - `POST /api/{id}/dashboard` - Create a dashboard in a workspace
//! - `POST /api/{id}/dashboard/{dashboard_id}/chart` - Add a chart to a dashboard
//!
//! Saved queries:
//! - `POST /api/{id}/query` - Save a query in a workspace
//!
//! Workspace preferences:
//! - `POST /api/{id}/preferences/catalog` - Set default catalog
//! - `POST /api/{id}/preferences/catalog/clear` - Clear default catalog
//!
//! User preferences:
//! - `POST /api/user/preferences/theme` - Set user theme
//! - `POST /api/user/preferences/locale` - Set user locale

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::dashboard::handle_dashboard_command_zenoh;
use crate::application::error::CommandPipelineError;
use crate::application::saved_query::handle_saved_query_command_zenoh;
use crate::application::user_preferences::handle_user_preferences_command_zenoh;
use crate::application::workspace::{
    handle_workspace_command_zenoh, query_dashboard_layout, query_saved_query_list,
    query_user_preferences, query_workspace_list,
};
use crate::application::workspace_preferences::handle_workspace_preferences_command_zenoh;
use crate::domain::analytics::{DatasetRef, SqlQuery};
use crate::domain::common::DashboardTitle;
use crate::domain::dashboard::commands::DashboardCommand;
use crate::domain::dashboard::events::DashboardEvent;
use crate::domain::dashboard::values::{ChartPlacement, DashboardId};
use crate::domain::saved_query::commands::SavedQueryCommand;
use crate::domain::saved_query::events::SavedQueryEvent;
use crate::domain::saved_query::values::{QueryName, SavedQueryId};
use crate::domain::session::UserId;
use crate::domain::user_preferences::commands::UserPreferencesCommand;
use crate::domain::user_preferences::events::UserPreferencesEvent;
use crate::domain::user_preferences::values::{Locale, PreferencesId, Theme};
use crate::domain::workspace::commands::WorkspaceCommand;
use crate::domain::workspace::events::WorkspaceEvent;
use crate::domain::workspace::values::{Visibility, WorkspaceId, WorkspaceName};
use crate::domain::workspace_preferences::commands::WorkspacePreferencesCommand;
use crate::domain::workspace_preferences::events::WorkspacePreferencesEvent;
use crate::domain::workspace_preferences::values::CatalogUri;
use crate::infrastructure::event_bus::ZenohEventBus;
use crate::infrastructure::event_store::SqliteEventRepository;
use crate::presentation::error::AppError;
use crate::state::AppState;

/// Application state for Workspace bounded context handlers.
///
/// Contains event repositories for all five aggregate types and an optional
/// event bus for post-persist notification.
#[derive(Clone)]
pub struct WorkspaceAppState {
    pub workspace_repo: Arc<SqliteEventRepository<WorkspaceCommand, WorkspaceEvent>>,
    pub dashboard_repo: Arc<SqliteEventRepository<DashboardCommand, DashboardEvent>>,
    pub saved_query_repo: Arc<SqliteEventRepository<SavedQueryCommand, SavedQueryEvent>>,
    pub user_preferences_repo:
        Arc<SqliteEventRepository<UserPreferencesCommand, UserPreferencesEvent>>,
    pub workspace_preferences_repo:
        Arc<SqliteEventRepository<WorkspacePreferencesCommand, WorkspacePreferencesEvent>>,
    pub event_bus: Option<Arc<ZenohEventBus>>,
}

// =============================================================================
// Route configuration
// =============================================================================

/// Creates the Workspace feature router with query and command endpoints.
pub fn routes() -> Router<AppState> {
    Router::new()
        // Query endpoints
        .route("/api", get(list_workspaces))
        .route(
            "/api/{id}/dashboard/{dashboard_id}",
            get(get_dashboard_layout),
        )
        .route("/api/{id}/queries", get(list_saved_queries))
        .route("/api/user/preferences/{user_id}", get(get_user_preferences))
        // Workspace lifecycle
        .route("/api", post(create_workspace))
        .route("/api/{id}/rename", post(rename_workspace))
        .route("/api/{id}/visibility", post(set_visibility))
        // Dashboard management
        .route("/api/{id}/dashboard", post(create_dashboard))
        .route("/api/{id}/dashboard/{dashboard_id}/chart", post(add_chart))
        // Saved queries
        .route("/api/{id}/query", post(save_query))
        // Workspace preferences
        .route("/api/{id}/preferences/catalog", post(set_default_catalog))
        .route(
            "/api/{id}/preferences/catalog/clear",
            post(clear_default_catalog),
        )
        // User preferences
        .route("/api/user/preferences/theme", post(set_theme))
        .route("/api/user/preferences/locale", post(set_locale))
}

// =============================================================================
// Response types
// =============================================================================

/// Response body for successful workspace command operations.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResponse {
    /// The ID of the affected aggregate.
    pub id: Uuid,
    /// Number of events produced by this command.
    pub events_count: usize,
}

// =============================================================================
// Query response types
// =============================================================================

/// A single workspace entry in the list response.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceListItem {
    pub workspace_id: WorkspaceId,
    pub name: WorkspaceName,
    pub owner_id: UserId,
    pub visibility: Visibility,
    pub created_at: chrono::DateTime<Utc>,
}

/// Response body for the workspace list query.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceListResponse {
    pub workspaces: Vec<WorkspaceListItem>,
    pub count: usize,
}

/// Response body for the dashboard layout query.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardLayoutResponse {
    pub dashboard_id: Option<DashboardId>,
    pub workspace_id: Option<WorkspaceId>,
    pub name: Option<DashboardTitle>,
    pub placements: Vec<ChartPlacement>,
    pub chart_count: usize,
    pub tab_count: usize,
}

/// A single saved query entry in the list response.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedQueryListItem {
    pub query_id: SavedQueryId,
    pub workspace_id: WorkspaceId,
    pub name: QueryName,
    pub sql: String,
    pub dataset_ref: String,
    pub saved_at: chrono::DateTime<Utc>,
}

/// Response body for the saved query list query.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedQueryListResponse {
    pub queries: Vec<SavedQueryListItem>,
    pub count: usize,
}

/// Response body for the user preferences query.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferencesResponse {
    pub preferences_id: Option<PreferencesId>,
    pub user_id: Option<UserId>,
    pub theme: Theme,
    pub locale: Locale,
    pub initialized: bool,
}

// =============================================================================
// Query handlers
// =============================================================================

/// GET /api - List all workspaces.
pub async fn list_workspaces(
    State(state): State<WorkspaceAppState>,
) -> Result<impl IntoResponse, AppError> {
    let view_state = query_workspace_list::<WorkspaceCommand>(&state.workspace_repo).await?;

    let workspaces: Vec<WorkspaceListItem> = view_state
        .workspaces
        .into_iter()
        .map(|w| WorkspaceListItem {
            workspace_id: w.workspace_id,
            name: w.name,
            owner_id: w.owner_id,
            visibility: w.visibility,
            created_at: w.created_at,
        })
        .collect();
    let count = workspaces.len();

    Ok(Json(WorkspaceListResponse { workspaces, count }))
}

/// GET /api/{id}/dashboard/{dashboard_id} - Get dashboard layout.
pub async fn get_dashboard_layout(
    State(state): State<WorkspaceAppState>,
    Path((_workspace_id, dashboard_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let view_state = query_dashboard_layout::<DashboardCommand>(
        &state.dashboard_repo,
        &dashboard_id.to_string(),
    )
    .await?;

    if view_state.dashboard_id.is_none() {
        return Err(AppError::not_found("Dashboard", dashboard_id.to_string()));
    }

    Ok(Json(DashboardLayoutResponse {
        dashboard_id: view_state.dashboard_id,
        workspace_id: view_state.workspace_id,
        name: view_state.name,
        placements: view_state.placements,
        chart_count: view_state.chart_count,
        tab_count: view_state.tab_count,
    }))
}

/// GET /api/{id}/queries - List saved queries for a workspace.
pub async fn list_saved_queries(
    State(state): State<WorkspaceAppState>,
    Path(workspace_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let view_state = query_saved_query_list::<SavedQueryCommand>(&state.saved_query_repo).await?;
    let ws_id = WorkspaceId::from_uuid(workspace_id);
    let filtered = view_state.queries_for_workspace(&ws_id);

    let queries: Vec<SavedQueryListItem> = filtered
        .into_iter()
        .map(|q| SavedQueryListItem {
            query_id: q.query_id,
            workspace_id: q.workspace_id,
            name: q.name.clone(),
            sql: q.sql.clone(),
            dataset_ref: q.dataset_ref.clone(),
            saved_at: q.saved_at,
        })
        .collect();
    let count = queries.len();

    Ok(Json(SavedQueryListResponse { queries, count }))
}

/// GET /api/user/preferences/{user_id} - Get user preferences.
pub async fn get_user_preferences(
    State(state): State<WorkspaceAppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let uid = UserId::from_uuid(user_id);
    let view_state =
        query_user_preferences::<UserPreferencesCommand>(&state.user_preferences_repo, &uid)
            .await?;

    Ok(Json(UserPreferencesResponse {
        preferences_id: view_state.preferences_id,
        user_id: view_state.user_id,
        theme: view_state.theme,
        locale: view_state.locale,
        initialized: view_state.initialized,
    }))
}

// =============================================================================
// Request payload types
// =============================================================================

/// Request body for creating a new workspace.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub owner_id: Uuid,
    pub visibility: Visibility,
}

/// Request body for renaming a workspace.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameWorkspaceRequest {
    pub new_name: String,
}

/// Request body for changing workspace visibility.
#[derive(Debug, Deserialize)]
pub struct SetVisibilityRequest {
    pub visibility: Visibility,
}

/// Request body for creating a dashboard.
#[derive(Debug, Deserialize)]
pub struct CreateDashboardRequest {
    pub name: String,
}

/// Request body for adding a chart to a dashboard.
#[derive(Debug, Deserialize)]
pub struct AddChartRequest {
    pub placement: ChartPlacement,
}

/// Request body for saving a query.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveQueryRequest {
    pub name: String,
    pub sql: String,
    #[serde(default)]
    pub dataset_ref: Option<String>,
}

/// Request body for setting the default catalog.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDefaultCatalogRequest {
    pub catalog_uri: String,
}

/// Request body for setting user theme.
#[derive(Debug, Deserialize)]
pub struct SetThemeRequest {
    pub theme: Theme,
}

/// Request body for setting user locale.
#[derive(Debug, Deserialize)]
pub struct SetLocaleRequest {
    pub locale: String,
}

// =============================================================================
// Workspace command handlers
// =============================================================================

/// POST /api - Create a new workspace.
///
/// Returns 202 Accepted; state changes are delivered via SSE feeds.
pub async fn create_workspace(
    State(state): State<WorkspaceAppState>,
    Json(request): Json<CreateWorkspaceRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let id = WorkspaceId::new();
    let command = WorkspaceCommand::Create {
        workspace_id: id,
        name: request.name,
        owner_id: UserId::from_uuid(request.owner_id),
        visibility: request.visibility,
        created_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events =
        handle_workspace_command_zenoh(Arc::clone(&state.workspace_repo), event_bus_ref, command)
            .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CommandResponse {
            id: id.into_inner(),
            events_count: events.len(),
        }),
    ))
}

/// POST /api/{id}/rename - Rename a workspace.
pub async fn rename_workspace(
    State(state): State<WorkspaceAppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<RenameWorkspaceRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let workspace_id = WorkspaceId::from_uuid(id);
    let command = WorkspaceCommand::Rename {
        workspace_id,
        new_name: request.new_name,
        renamed_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events =
        handle_workspace_command_zenoh(Arc::clone(&state.workspace_repo), event_bus_ref, command)
            .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CommandResponse {
            id,
            events_count: events.len(),
        }),
    ))
}

/// POST /api/{id}/visibility - Change workspace visibility.
pub async fn set_visibility(
    State(state): State<WorkspaceAppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<SetVisibilityRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let workspace_id = WorkspaceId::from_uuid(id);
    let command = WorkspaceCommand::SetVisibility {
        workspace_id,
        visibility: request.visibility,
        changed_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events =
        handle_workspace_command_zenoh(Arc::clone(&state.workspace_repo), event_bus_ref, command)
            .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CommandResponse {
            id,
            events_count: events.len(),
        }),
    ))
}

// =============================================================================
// Dashboard command handlers
// =============================================================================

/// POST /api/{id}/dashboard - Create a dashboard in a workspace.
pub async fn create_dashboard(
    State(state): State<WorkspaceAppState>,
    Path(workspace_id): Path<Uuid>,
    Json(request): Json<CreateDashboardRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let dashboard_id = DashboardId::new();
    let command = DashboardCommand::CreateDashboard {
        dashboard_id,
        workspace_id: WorkspaceId::from_uuid(workspace_id),
        name: DashboardTitle::new(request.name)?,
        created_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events =
        handle_dashboard_command_zenoh(Arc::clone(&state.dashboard_repo), event_bus_ref, command)
            .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CommandResponse {
            id: dashboard_id.into_inner(),
            events_count: events.len(),
        }),
    ))
}

/// POST /api/{id}/dashboard/{dashboard_id}/chart - Add a chart to a dashboard.
pub async fn add_chart(
    State(state): State<WorkspaceAppState>,
    Path((_workspace_id, dashboard_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<AddChartRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let db_id = DashboardId::from_uuid(dashboard_id);
    let command = DashboardCommand::AddChart {
        dashboard_id: db_id,
        placement: request.placement,
        added_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events =
        handle_dashboard_command_zenoh(Arc::clone(&state.dashboard_repo), event_bus_ref, command)
            .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CommandResponse {
            id: dashboard_id,
            events_count: events.len(),
        }),
    ))
}

// =============================================================================
// SavedQuery command handlers
// =============================================================================

/// POST /api/{id}/query - Save a query in a workspace.
pub async fn save_query(
    State(state): State<WorkspaceAppState>,
    Path(workspace_id): Path<Uuid>,
    Json(request): Json<SaveQueryRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let query_id = SavedQueryId::new();
    let dataset_ref = match request.dataset_ref {
        Some(r) => DatasetRef::new(r)?,
        None => DatasetRef::new("./local")?,
    };
    let command = SavedQueryCommand::SaveQuery {
        query_id,
        workspace_id: WorkspaceId::from_uuid(workspace_id),
        name: QueryName::new(request.name)?,
        sql: SqlQuery::new(request.sql)?,
        dataset_ref,
        saved_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events = handle_saved_query_command_zenoh(
        Arc::clone(&state.saved_query_repo),
        event_bus_ref,
        command,
    )
    .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CommandResponse {
            id: query_id.into_inner(),
            events_count: events.len(),
        }),
    ))
}

// =============================================================================
// Workspace preferences command handlers
// =============================================================================

/// POST /api/{id}/preferences/catalog - Set default catalog for workspace.
pub async fn set_default_catalog(
    State(state): State<WorkspaceAppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<SetDefaultCatalogRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let workspace_id = WorkspaceId::from_uuid(id);

    // Ensure workspace preferences are initialized first, then set catalog
    let init_command = WorkspacePreferencesCommand::InitializeWorkspacePreferences {
        workspace_id,
        initialized_at: Utc::now(),
    };
    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();

    // Initialize is idempotent — will return empty events if already initialized
    let _ = handle_workspace_preferences_command_zenoh(
        Arc::clone(&state.workspace_preferences_repo),
        event_bus_ref,
        init_command,
    )
    .await?;

    let command = WorkspacePreferencesCommand::SetDefaultCatalog {
        workspace_id,
        catalog_uri: CatalogUri::new(request.catalog_uri)
            .map_err(|e| AppError::from(CommandPipelineError::from(e)))?,
        set_at: Utc::now(),
    };

    let events = handle_workspace_preferences_command_zenoh(
        Arc::clone(&state.workspace_preferences_repo),
        event_bus_ref,
        command,
    )
    .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CommandResponse {
            id,
            events_count: events.len(),
        }),
    ))
}

/// POST /api/{id}/preferences/catalog/clear - Clear default catalog.
pub async fn clear_default_catalog(
    State(state): State<WorkspaceAppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let workspace_id = WorkspaceId::from_uuid(id);
    let command = WorkspacePreferencesCommand::ClearDefaultCatalog {
        workspace_id,
        cleared_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events = handle_workspace_preferences_command_zenoh(
        Arc::clone(&state.workspace_preferences_repo),
        event_bus_ref,
        command,
    )
    .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CommandResponse {
            id,
            events_count: events.len(),
        }),
    ))
}

// =============================================================================
// User preferences command handlers
// =============================================================================

/// POST /api/user/preferences/theme - Set user theme.
pub async fn set_theme(
    State(state): State<WorkspaceAppState>,
    Json(request): Json<SetThemeRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    // Generate a user ID for now; will come from auth context later (jqv epic)
    let user_id = UserId::new();
    let prefs_id = PreferencesId::new();

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();

    // Initialize preferences if needed (idempotent)
    let init_command = UserPreferencesCommand::InitializePreferences {
        preferences_id: prefs_id,
        user_id,
        initialized_at: Utc::now(),
    };
    let _ = handle_user_preferences_command_zenoh(
        Arc::clone(&state.user_preferences_repo),
        event_bus_ref,
        init_command,
    )
    .await?;

    let command = UserPreferencesCommand::SetTheme {
        user_id,
        theme: request.theme,
        set_at: Utc::now(),
    };

    let events = handle_user_preferences_command_zenoh(
        Arc::clone(&state.user_preferences_repo),
        event_bus_ref,
        command,
    )
    .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CommandResponse {
            id: user_id.into_inner(),
            events_count: events.len(),
        }),
    ))
}

/// POST /api/user/preferences/locale - Set user locale.
pub async fn set_locale(
    State(state): State<WorkspaceAppState>,
    Json(request): Json<SetLocaleRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let user_id = UserId::new();
    let prefs_id = PreferencesId::new();

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();

    let init_command = UserPreferencesCommand::InitializePreferences {
        preferences_id: prefs_id,
        user_id,
        initialized_at: Utc::now(),
    };
    let _ = handle_user_preferences_command_zenoh(
        Arc::clone(&state.user_preferences_repo),
        event_bus_ref,
        init_command,
    )
    .await?;

    let command = UserPreferencesCommand::SetLocale {
        user_id,
        locale: Locale::new(request.locale)?,
        set_at: Utc::now(),
    };

    let events = handle_user_preferences_command_zenoh(
        Arc::clone(&state.user_preferences_repo),
        event_bus_ref,
        command,
    )
    .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CommandResponse {
            id: user_id.into_inner(),
            events_count: events.len(),
        }),
    ))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    async fn create_test_pool() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool");

        sqlx::query(include_str!("../../migrations/001_events.sql"))
            .execute(&pool)
            .await
            .expect("Failed to run migration");

        pool
    }

    fn create_workspace_router(
        workspace_repo: Arc<SqliteEventRepository<WorkspaceCommand, WorkspaceEvent>>,
        dashboard_repo: Arc<SqliteEventRepository<DashboardCommand, DashboardEvent>>,
        saved_query_repo: Arc<SqliteEventRepository<SavedQueryCommand, SavedQueryEvent>>,
    ) -> Router {
        let state = WorkspaceAppState {
            workspace_repo,
            dashboard_repo,
            saved_query_repo,
            user_preferences_repo: Arc::new(SqliteEventRepository::new(
                SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect_lazy("sqlite::memory:")
                    .expect("lazy pool"),
            )),
            workspace_preferences_repo: Arc::new(SqliteEventRepository::new(
                SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect_lazy("sqlite::memory:")
                    .expect("lazy pool"),
            )),
            event_bus: None,
        };

        Router::new()
            .route("/api", post(create_workspace))
            .route("/api/{id}/rename", post(rename_workspace))
            .route("/api/{id}/visibility", post(set_visibility))
            .route("/api/{id}/dashboard", post(create_dashboard))
            .route("/api/{id}/query", post(save_query))
            .with_state(state)
    }

    #[tokio::test]
    async fn create_workspace_returns_accepted() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool.clone()));
        let db_repo = Arc::new(SqliteEventRepository::new(pool.clone()));
        let sq_repo = Arc::new(SqliteEventRepository::new(pool));
        let app = create_workspace_router(repo, db_repo, sq_repo);

        let owner_id = Uuid::new_v4();
        let body = serde_json::json!({
            "name": "Test Workspace",
            "ownerId": owner_id.to_string(),
            "visibility": "private"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let resp: CommandResponse = serde_json::from_slice(&body).expect("valid JSON response");
        assert_eq!(resp.events_count, 1);
    }

    #[tokio::test]
    async fn rename_nonexistent_workspace_returns_error() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool.clone()));
        let db_repo = Arc::new(SqliteEventRepository::new(pool.clone()));
        let sq_repo = Arc::new(SqliteEventRepository::new(pool));
        let app = create_workspace_router(repo, db_repo, sq_repo);

        let body = serde_json::json!({ "newName": "New Name" });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/{}/rename", Uuid::new_v4()))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .expect("request should succeed");

        // Should return an error status (domain error → NotFound)
        assert_ne!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn create_dashboard_returns_accepted() {
        let pool = create_test_pool().await;
        let ws_repo = Arc::new(SqliteEventRepository::new(pool.clone()));
        let db_repo = Arc::new(SqliteEventRepository::new(pool.clone()));
        let sq_repo = Arc::new(SqliteEventRepository::new(pool));
        let app = create_workspace_router(ws_repo, db_repo, sq_repo);

        let workspace_id = Uuid::new_v4();
        let body = serde_json::json!({ "name": "My Dashboard" });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/{workspace_id}/dashboard"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let resp: CommandResponse = serde_json::from_slice(&body).expect("valid JSON response");
        assert_eq!(resp.events_count, 1);
    }

    #[tokio::test]
    async fn save_query_returns_accepted() {
        let pool = create_test_pool().await;
        let ws_repo = Arc::new(SqliteEventRepository::new(pool.clone()));
        let db_repo = Arc::new(SqliteEventRepository::new(pool.clone()));
        let sq_repo = Arc::new(SqliteEventRepository::new(pool));
        let app = create_workspace_router(ws_repo, db_repo, sq_repo);

        let workspace_id = Uuid::new_v4();
        let body = serde_json::json!({
            "name": "Monthly Sales",
            "sql": "SELECT * FROM sales"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/{workspace_id}/query"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }
}
