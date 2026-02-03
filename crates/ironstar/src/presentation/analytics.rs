//! Analytics HTTP handlers for Catalog and QuerySession aggregates.
//!
//! Provides REST endpoints for catalog management and query execution,
//! plus a combined SSE feed for real-time analytics event streaming.
//!
//! # Routes
//!
//! Catalog:
//! - `POST /api/catalog/select` - Select a DuckLake catalog
//! - `POST /api/catalog/refresh` - Refresh catalog metadata
//! - `GET /api/catalog` - Query current catalog state
//!
//! QuerySession:
//! - `POST /api/queries` - Start a new query
//! - `DELETE /api/queries/{id}` - Cancel a running query
//! - `GET /api/queries/{id}` - Get specific query from history
//! - `GET /api/queries` - List query history
//!
//! SSE:
//! - `GET /api/feed` - Combined Catalog + QuerySession event stream

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, Sse};
use axum::routing::{delete as route_delete, get, post};
use chrono::Utc;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tracing::warn;
use uuid::Uuid;

use crate::application::catalog::{handle_catalog_command_zenoh, query_catalog_state};
use crate::application::query_session::{
    handle_query_session_command_zenoh, query_query_history, query_session_state,
};
use crate::domain::traits::EventType;
use crate::domain::views::{CatalogViewState, QueryHistoryEntry, QuerySessionViewState};
use crate::domain::{
    CatalogCommand, CatalogEvent, CatalogMetadata, CatalogRef, DatasetInfo, QueryId,
    QuerySessionCommand, QuerySessionEvent, SqlQuery,
};
use crate::infrastructure::event_bus::ZenohEventBus;
use crate::infrastructure::event_store::{SqliteEventRepository, StoredEvent};
use crate::infrastructure::key_expr::aggregate_type_pattern;
use crate::infrastructure::sse_stream::{
    SseStreamBuilder, stored_events_to_stream, zenoh_to_sse_stream,
};
use crate::presentation::error::AppError;
use crate::state::AppState;

/// Application state for Analytics handlers.
///
/// Contains event repositories for Catalog and QuerySession aggregates,
/// plus the optional event bus for SSE streaming and post-persist notification.
#[derive(Clone)]
pub struct AnalyticsAppState {
    pub catalog_repo: Arc<SqliteEventRepository<CatalogCommand, CatalogEvent>>,
    pub query_session_repo: Arc<SqliteEventRepository<QuerySessionCommand, QuerySessionEvent>>,
    pub event_bus: Option<Arc<ZenohEventBus>>,
}

// =============================================================================
// Route configuration
// =============================================================================

/// Creates the Analytics feature router with all endpoints.
pub fn routes() -> Router<AppState> {
    Router::new()
        // Catalog endpoints
        .route("/api/catalog/select", post(select_catalog))
        .route("/api/catalog/refresh", post(refresh_catalog))
        .route("/api/catalog", get(get_catalog))
        // QuerySession endpoints
        .route("/api/queries", post(start_query))
        .route("/api/queries", get(list_query_history))
        .route("/api/queries/{id}", get(get_query))
        .route("/api/queries/{id}", route_delete(cancel_query))
        // SSE feed
        .route("/api/feed", get(analytics_feed_handler))
}

// =============================================================================
// SSE feed
// =============================================================================

/// GET /api/feed - Combined SSE stream for all Analytics events.
///
/// Streams both Catalog and QuerySession events on a single SSE connection.
/// Supports `Last-Event-ID` reconnection. Subscribe-before-replay invariant
/// is maintained: Zenoh subscriptions are established before querying historical
/// events.
async fn analytics_feed_handler(
    State(state): State<AnalyticsAppState>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>> + Send>, StatusCode> {
    let event_bus = state.event_bus.as_ref().ok_or_else(|| {
        warn!("Analytics feed requested but event bus not configured");
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    let last_event_id: i64 = headers
        .get("Last-Event-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Subscribe BEFORE loading historical events (critical invariant).
    let catalog_sub = event_bus
        .session()
        .declare_subscriber(aggregate_type_pattern("Catalog"))
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to create Zenoh subscriber for Catalog feed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let qs_sub = event_bus
        .session()
        .declare_subscriber(aggregate_type_pattern("QuerySession"))
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to create Zenoh subscriber for QuerySession feed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Query historical events AFTER subscribing, from both repos.
    let catalog_events = state
        .catalog_repo
        .query_since_sequence(last_event_id)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to query historical Catalog events");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let qs_events = state
        .query_session_repo
        .query_since_sequence(last_event_id)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to query historical QuerySession events");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Merge historical events sorted by global sequence for correct SSE ordering.
    let mut all_sse: Vec<(i64, Event)> = Vec::with_capacity(catalog_events.len() + qs_events.len());
    for stored in catalog_events {
        let seq = stored.sequence;
        all_sse.push((seq, stored_catalog_event_to_sse(stored)));
    }
    for stored in qs_events {
        let seq = stored.sequence;
        all_sse.push((seq, stored_qs_event_to_sse(stored)));
    }
    all_sse.sort_by_key(|(seq, _)| *seq);

    let replay_stream =
        stored_events_to_stream(all_sse.into_iter().map(|(_, event)| event).collect(), |e| e);

    // Merge live streams from both subscribers.
    let catalog_live = zenoh_to_sse_stream(catalog_sub, live_catalog_event_to_sse);
    let qs_live = zenoh_to_sse_stream(qs_sub, live_qs_event_to_sse);
    let combined_live = futures::stream::select(catalog_live, qs_live);

    let builder = SseStreamBuilder::new().with_keep_alive_secs(15);
    let stream = builder.build_with_streams(replay_stream, combined_live);

    Ok(Sse::new(stream))
}

fn stored_catalog_event_to_sse(stored: StoredEvent<CatalogEvent>) -> Event {
    Event::default()
        .id(stored.sequence.to_string())
        .event(stored.event_type)
        .data(serde_json::to_string(&stored.event).unwrap_or_else(|e| {
            warn!(error = %e, "Failed to serialize stored CatalogEvent");
            "{}".to_string()
        }))
}

fn stored_qs_event_to_sse(stored: StoredEvent<QuerySessionEvent>) -> Event {
    Event::default()
        .id(stored.sequence.to_string())
        .event(stored.event_type)
        .data(serde_json::to_string(&stored.event).unwrap_or_else(|e| {
            warn!(error = %e, "Failed to serialize stored QuerySessionEvent");
            "{}".to_string()
        }))
}

fn live_catalog_event_to_sse(event: CatalogEvent) -> Event {
    Event::default()
        .event(event.event_type())
        .data(serde_json::to_string(&event).unwrap_or_else(|e| {
            warn!(error = %e, "Failed to serialize live CatalogEvent");
            "{}".to_string()
        }))
}

fn live_qs_event_to_sse(event: QuerySessionEvent) -> Event {
    Event::default()
        .event(event.event_type())
        .data(serde_json::to_string(&event).unwrap_or_else(|e| {
            warn!(error = %e, "Failed to serialize live QuerySessionEvent");
            "{}".to_string()
        }))
}

// =============================================================================
// Catalog query handlers
// =============================================================================

/// GET /api/catalog - Query current catalog state.
pub async fn get_catalog(
    State(state): State<AnalyticsAppState>,
) -> Result<Json<CatalogStateResponse>, AppError> {
    let view_state = query_catalog_state(&state.catalog_repo).await?;
    Ok(Json(CatalogStateResponse::from(view_state)))
}

/// Response for the catalog state query.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogStateResponse {
    pub has_catalog: bool,
    pub catalog_ref: Option<String>,
    pub metadata: Option<CatalogMetadata>,
    pub dataset_count: usize,
}

impl From<CatalogViewState> for CatalogStateResponse {
    fn from(state: CatalogViewState) -> Self {
        let has_catalog = state.has_catalog();
        let dataset_count = state.dataset_count();
        let catalog_ref = state.catalog_ref.as_ref().map(|r| r.as_str().to_string());
        Self {
            has_catalog,
            catalog_ref,
            dataset_count,
            metadata: state.metadata,
        }
    }
}

// =============================================================================
// Catalog command handlers
// =============================================================================

/// Request body for selecting a catalog.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectCatalogRequest {
    pub catalog_ref: String,
}

/// Response body for successful command operations.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsCommandResponse {
    pub events_count: usize,
}

/// POST /api/catalog/select - Select a DuckLake catalog.
pub async fn select_catalog(
    State(state): State<AnalyticsAppState>,
    Json(request): Json<SelectCatalogRequest>,
) -> Result<(StatusCode, Json<AnalyticsCommandResponse>), AppError> {
    let catalog_ref = CatalogRef::new(request.catalog_ref)?;
    let command = CatalogCommand::SelectCatalog {
        catalog_ref,
        selected_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events =
        handle_catalog_command_zenoh(Arc::clone(&state.catalog_repo), event_bus_ref, command)
            .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(AnalyticsCommandResponse {
            events_count: events.len(),
        }),
    ))
}

/// Request body for refreshing catalog metadata.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshCatalogRequest {
    pub datasets: Vec<DatasetInfo>,
}

/// POST /api/catalog/refresh - Refresh catalog metadata.
pub async fn refresh_catalog(
    State(state): State<AnalyticsAppState>,
    Json(request): Json<RefreshCatalogRequest>,
) -> Result<(StatusCode, Json<AnalyticsCommandResponse>), AppError> {
    let now = Utc::now();
    let metadata = CatalogMetadata {
        datasets: request.datasets,
        last_refreshed: now,
    };
    let command = CatalogCommand::RefreshCatalogMetadata {
        metadata,
        refreshed_at: now,
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events =
        handle_catalog_command_zenoh(Arc::clone(&state.catalog_repo), event_bus_ref, command)
            .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(AnalyticsCommandResponse {
            events_count: events.len(),
        }),
    ))
}

// =============================================================================
// QuerySession query handlers
// =============================================================================

/// Response for the session state query.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStateResponse {
    pub is_idle: bool,
    pub is_in_progress: bool,
    pub status: String,
    pub completed_count: usize,
    pub failed_count: usize,
    pub cancelled_count: usize,
    pub total_finished: usize,
}

impl From<&QuerySessionViewState> for SessionStateResponse {
    fn from(state: &QuerySessionViewState) -> Self {
        Self {
            is_idle: state.is_idle(),
            is_in_progress: state.is_in_progress(),
            status: format!("{:?}", state.status),
            completed_count: state.completed_count,
            failed_count: state.failed_count,
            cancelled_count: state.cancelled_count,
            total_finished: state.total_finished(),
        }
    }
}

/// Response for query history entries.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryHistoryResponse {
    pub history: Vec<QueryHistoryEntryResponse>,
    pub session: SessionStateResponse,
}

/// A single query history entry for JSON serialization.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryHistoryEntryResponse {
    pub query_id: Uuid,
    pub sql: String,
    pub dataset_ref: Option<String>,
    pub started_at: chrono::DateTime<Utc>,
    pub outcome: String,
}

impl From<&QueryHistoryEntry> for QueryHistoryEntryResponse {
    fn from(entry: &QueryHistoryEntry) -> Self {
        Self {
            query_id: entry.query_id.into_inner(),
            sql: entry.sql.as_str().to_string(),
            dataset_ref: entry.dataset_ref.as_ref().map(|r| r.as_str().to_string()),
            started_at: entry.started_at,
            outcome: format!("{:?}", entry.outcome),
        }
    }
}

/// GET /api/queries - List query history.
pub async fn list_query_history(
    State(state): State<AnalyticsAppState>,
) -> Result<Json<QueryHistoryResponse>, AppError> {
    let view_state = query_session_state(&state.query_session_repo).await?;
    let history = view_state
        .query_history
        .iter()
        .map(QueryHistoryEntryResponse::from)
        .collect();

    Ok(Json(QueryHistoryResponse {
        history,
        session: SessionStateResponse::from(&view_state),
    }))
}

/// GET /api/queries/{id} - Get specific query from history.
pub async fn get_query(
    State(state): State<AnalyticsAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<QueryHistoryEntryResponse>, AppError> {
    let history = query_query_history(&state.query_session_repo).await?;
    let query_id = QueryId::from_uuid(id);

    let entry = history
        .iter()
        .find(|e| e.query_id == query_id)
        .ok_or_else(|| AppError::not_found("Query", id.to_string()))?;

    Ok(Json(QueryHistoryEntryResponse::from(entry)))
}

// =============================================================================
// QuerySession command handlers
// =============================================================================

/// Request body for starting a new query.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartQueryRequest {
    pub sql: String,
    pub dataset_ref: Option<String>,
}

/// POST /api/queries - Start a new analytics query.
pub async fn start_query(
    State(state): State<AnalyticsAppState>,
    Json(request): Json<StartQueryRequest>,
) -> Result<(StatusCode, Json<StartQueryResponse>), AppError> {
    let query_id = QueryId::new();
    let sql = SqlQuery::new(&request.sql)?;
    let dataset_ref = request
        .dataset_ref
        .map(|r| crate::domain::DatasetRef::new(r))
        .transpose()?;

    let command = QuerySessionCommand::StartQuery {
        query_id,
        sql,
        dataset_ref,
        chart_config: None,
        started_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events = handle_query_session_command_zenoh(
        Arc::clone(&state.query_session_repo),
        event_bus_ref,
        command,
    )
    .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(StartQueryResponse {
            query_id: query_id.into_inner(),
            events_count: events.len(),
        }),
    ))
}

/// Response for the start query command.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartQueryResponse {
    pub query_id: Uuid,
    pub events_count: usize,
}

/// Request body for cancelling a query.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelQueryRequest {
    #[serde(default)]
    pub reason: Option<String>,
}

/// DELETE /api/queries/{id} - Cancel a running query.
pub async fn cancel_query(
    State(state): State<AnalyticsAppState>,
    Path(id): Path<Uuid>,
    body: Option<Json<CancelQueryRequest>>,
) -> Result<(StatusCode, Json<AnalyticsCommandResponse>), AppError> {
    let query_id = QueryId::from_uuid(id);
    let reason = body.and_then(|b| b.0.reason);
    let command = QuerySessionCommand::CancelQuery {
        query_id,
        reason,
        cancelled_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events = handle_query_session_command_zenoh(
        Arc::clone(&state.query_session_repo),
        event_bus_ref,
        command,
    )
    .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(AnalyticsCommandResponse {
            events_count: events.len(),
        }),
    ))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::application::catalog::handle_catalog_command;
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

    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    fn create_analytics_state(pool: sqlx::SqlitePool) -> AnalyticsAppState {
        AnalyticsAppState {
            catalog_repo: Arc::new(SqliteEventRepository::new(pool.clone())),
            query_session_repo: Arc::new(SqliteEventRepository::new(pool)),
            event_bus: None,
        }
    }

    fn create_router(state: AnalyticsAppState) -> Router {
        Router::new()
            .route("/api/catalog", get(get_catalog))
            .route("/api/queries", get(list_query_history))
            .route("/api/queries/{id}", get(get_query))
            .with_state(state)
    }

    #[tokio::test]
    async fn get_catalog_empty_returns_no_catalog() {
        let pool = create_test_pool().await;
        let state = create_analytics_state(pool);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/catalog")
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body read");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json parse");

        assert_eq!(json["hasCatalog"], false);
        assert!(json["catalogRef"].is_null());
        assert_eq!(json["datasetCount"], 0);
    }

    #[tokio::test]
    async fn get_catalog_after_select_returns_catalog() {
        let pool = create_test_pool().await;
        let state = create_analytics_state(pool);

        let command = CatalogCommand::SelectCatalog {
            catalog_ref: CatalogRef::new("ducklake:test").unwrap(),
            selected_at: Utc::now(),
        };
        handle_catalog_command(Arc::clone(&state.catalog_repo), NO_EVENT_BUS, command)
            .await
            .expect("select should succeed");

        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/catalog")
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body read");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json parse");

        assert_eq!(json["hasCatalog"], true);
        assert_eq!(json["catalogRef"], "ducklake:test");
    }

    #[tokio::test]
    async fn list_queries_empty_returns_idle() {
        let pool = create_test_pool().await;
        let state = create_analytics_state(pool);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/queries")
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body read");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json parse");

        assert_eq!(json["session"]["isIdle"], true);
        assert_eq!(json["history"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn get_query_not_found() {
        let pool = create_test_pool().await;
        let state = create_analytics_state(pool);
        let app = create_router(state);

        let nonexistent = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/queries/{nonexistent}"))
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
