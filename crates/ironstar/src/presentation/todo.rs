//! Todo HTTP handlers.
//!
//! This module provides axum handlers for Todo query and command endpoints.
//! Handlers call the application layer functions and format responses as JSON.
//!
//! # Routes
//!
//! Query endpoints:
//! - `GET /api/todos` - List all todos
//! - `GET /api/todos/:id` - Get a single todo by ID
//!
//! Command endpoints:
//! - `POST /api/todos` - Create a new todo
//! - `POST /api/todos/:id/complete` - Complete a todo
//! - `DELETE /api/todos/:id` - Delete a todo

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{
    Html, IntoResponse,
    sse::{Event, Sse},
};
use axum::routing::{delete as route_delete, get, post};
use chrono::Utc;
use futures::stream;
use hypertext::Renderable;
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::todo::{handle_todo_command_zenoh, query_all_todos, query_todo_state};
use crate::domain::signals::TodoItemView;
use crate::domain::todo::commands::TodoCommand;
use crate::domain::todo::events::TodoEvent;
use crate::domain::todo::values::TodoId;
use crate::infrastructure::assets::AssetManifest;
use crate::infrastructure::event_bus::ZenohEventBus;
use crate::infrastructure::event_store::SqliteEventRepository;
use crate::presentation::error::AppError;
use crate::presentation::todo_templates::todo_page;
use crate::state::AppState;

/// Application state for Todo handlers.
///
/// Contains the event repository for query and command operations,
/// and an optional event bus for post-persist event notification.
#[derive(Clone)]
pub struct TodoAppState {
    /// Event repository for persisting and fetching Todo events.
    pub repo: Arc<SqliteEventRepository<TodoCommand, TodoEvent>>,
    /// Optional event bus for publishing events after persistence.
    ///
    /// When `None`, events are persisted but not published to subscribers.
    /// Use `None` in tests that don't require event bus integration.
    pub event_bus: Option<Arc<ZenohEventBus>>,
}

// =============================================================================
// Route configuration
// =============================================================================

/// Creates the Todo feature router with all endpoints.
///
/// # Routes
///
/// - `GET /` - Render todo page (HTML)
/// - `GET /api` - List todos (JSON)
/// - `GET /api/:id` - Get single todo (JSON)
/// - `POST /api` - Create todo (JSON)
/// - `POST /api/:id/complete` - Complete todo (JSON)
/// - `DELETE /api/:id` - Delete todo (JSON)
/// - `GET /api/feed` - SSE feed (placeholder)
pub fn routes() -> Router<AppState> {
    Router::new()
        // HTML page endpoint
        .route("/", get(todo_page_handler))
        // JSON API endpoints (separate routes for each method)
        .route("/api", get(list_todos))
        .route("/api", post(create_todo))
        .route("/api/{id}", get(get_todo))
        .route("/api/{id}", route_delete(delete_todo))
        .route("/api/{id}/complete", post(complete_todo))
        // SSE feed endpoint (placeholder for now)
        .route("/api/feed", get(todo_feed_handler))
}

/// GET / - Render the todo page with current todos.
async fn todo_page_handler(
    State(state): State<TodoAppState>,
    State(manifest): State<AssetManifest>,
) -> Result<impl IntoResponse, AppError> {
    // Query current todos
    let view_state = query_all_todos(&state.repo).await?;

    // Render page with manifest-resolved asset paths
    let html = todo_page(&manifest, &view_state.todos).render();

    Ok(Html(html.into_inner()))
}

/// GET /api/feed - SSE stream for real-time todo updates.
///
/// Placeholder implementation that sends a single "connected" event.
/// Full implementation will subscribe to Zenoh and stream updates.
async fn todo_feed_handler() -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    // Placeholder: single connected event, then keep-alive
    let stream = stream::once(async {
        Ok(Event::default()
            .event("connected")
            .data("Todo feed connected"))
    });

    Sse::new(stream)
}

/// Response type for the todo list endpoint.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoListResponse {
    /// List of all non-deleted todos.
    pub todos: Vec<TodoItemView>,
    /// Total count of todos.
    pub count: usize,
    /// Count of completed todos.
    pub completed_count: usize,
    /// Count of active (non-completed) todos.
    pub active_count: usize,
}

/// GET /api/todos - List all todos.
///
/// Returns all non-deleted todos across all aggregates by replaying
/// events through the View.
///
/// # Response
///
/// - `200 OK` with JSON body containing todos list and counts
/// - `500 Internal Server Error` on database/infrastructure failure
///
/// # Example response
///
/// ```json
/// {
///   "todos": [
///     { "id": "550e8400-...", "text": "Buy groceries", "completed": false },
///     { "id": "6ba7b810-...", "text": "Walk the dog", "completed": true }
///   ],
///   "count": 2,
///   "completedCount": 1,
///   "activeCount": 1
/// }
/// ```
pub async fn list_todos(State(state): State<TodoAppState>) -> Result<impl IntoResponse, AppError> {
    let view_state = query_all_todos(&state.repo).await?;

    let active_count = view_state.active_count();
    let response = TodoListResponse {
        todos: view_state.todos,
        count: view_state.count,
        completed_count: view_state.completed_count,
        active_count,
    };

    Ok(Json(response))
}

/// GET /api/todos/:id - Get a single todo by ID.
///
/// Returns the specified todo if it exists and is not deleted.
///
/// # Path parameters
///
/// - `id` - UUID of the todo to retrieve
///
/// # Response
///
/// - `200 OK` with JSON body containing the todo
/// - `404 Not Found` if the todo doesn't exist or is deleted
/// - `400 Bad Request` if the ID is not a valid UUID
/// - `500 Internal Server Error` on database/infrastructure failure
///
/// # Example response
///
/// ```json
/// { "id": "550e8400-...", "text": "Buy groceries", "completed": false }
/// ```
pub async fn get_todo(
    State(state): State<TodoAppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let todo_id = TodoId::from_uuid(id);
    let view_state = query_todo_state(&state.repo, &todo_id).await?;

    // The view state for a single aggregate will have at most one todo
    // If the todo was deleted, the list will be empty
    match view_state.todos.into_iter().next() {
        Some(todo) => Ok((StatusCode::OK, Json(todo))),
        None => Err(AppError::not_found("Todo", id.to_string())),
    }
}

// =============================================================================
// Command handlers
// =============================================================================

/// Request body for creating a new todo.
#[derive(Debug, Deserialize)]
pub struct CreateTodoRequest {
    /// The text content of the todo item.
    pub text: String,
}

/// Response body for successful command operations.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResponse {
    /// The ID of the affected todo.
    pub id: Uuid,
    /// Number of events produced by this command.
    pub events_count: usize,
}

/// POST /api/todos - Create a new todo.
///
/// Creates a new todo item with the given text. The ID is generated server-side.
/// Timestamps are injected at this boundary layer.
///
/// # Request body
///
/// ```json
/// { "text": "Buy groceries" }
/// ```
///
/// # Response
///
/// - `202 Accepted` with JSON body containing the created todo ID
/// - `400 Bad Request` if text validation fails (empty or too long)
/// - `500 Internal Server Error` on infrastructure failure
///
/// Returns 202 (not 201) because full state is delivered via SSE streams.
/// The response acknowledges the command was accepted; clients observe
/// state changes through their SSE subscriptions.
///
/// # Example response
///
/// ```json
/// { "id": "550e8400-...", "eventsCount": 1 }
/// ```
pub async fn create_todo(
    State(state): State<TodoAppState>,
    Json(request): Json<CreateTodoRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let id = TodoId::new();
    let command = TodoCommand::Create {
        id,
        text: request.text,
        created_at: Utc::now(),
    };

    // Convert Arc<ZenohEventBus> to &ZenohEventBus for the generic call
    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events = handle_todo_command_zenoh(Arc::clone(&state.repo), event_bus_ref, command).await?;

    let response = CommandResponse {
        id: id.into_inner(),
        events_count: events.len(),
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// POST /api/todos/:id/complete - Complete a todo.
///
/// Marks the specified todo as completed. Idempotent: completing an already
/// completed todo returns success with no new events.
///
/// # Path parameters
///
/// - `id` - UUID of the todo to complete
///
/// # Response
///
/// - `202 Accepted` with JSON body containing event count
/// - `400 Bad Request` if the ID is not a valid UUID
/// - `404 Not Found` if the todo doesn't exist
/// - `409 Conflict` if the todo is in a state that cannot be completed (deleted)
/// - `500 Internal Server Error` on infrastructure failure
///
/// # Example response
///
/// ```json
/// { "id": "550e8400-...", "eventsCount": 1 }
/// ```
pub async fn complete_todo(
    State(state): State<TodoAppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let todo_id = TodoId::from_uuid(id);
    let command = TodoCommand::Complete {
        id: todo_id,
        completed_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events = handle_todo_command_zenoh(Arc::clone(&state.repo), event_bus_ref, command).await?;

    let response = CommandResponse {
        id,
        events_count: events.len(),
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// DELETE /api/todos/:id - Delete a todo.
///
/// Soft deletes the specified todo. Idempotent: deleting an already deleted
/// todo returns success with no new events.
///
/// # Path parameters
///
/// - `id` - UUID of the todo to delete
///
/// # Response
///
/// - `202 Accepted` with JSON body containing event count
/// - `400 Bad Request` if the ID is not a valid UUID
/// - `404 Not Found` if the todo doesn't exist
/// - `500 Internal Server Error` on infrastructure failure
///
/// # Example response
///
/// ```json
/// { "id": "550e8400-...", "eventsCount": 1 }
/// ```
pub async fn delete_todo(
    State(state): State<TodoAppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    let todo_id = TodoId::from_uuid(id);
    let command = TodoCommand::Delete {
        id: todo_id,
        deleted_at: Utc::now(),
    };

    let event_bus_ref: Option<&ZenohEventBus> = state.event_bus.as_deref();
    let events = handle_todo_command_zenoh(Arc::clone(&state.repo), event_bus_ref, command).await?;

    let response = CommandResponse {
        id,
        events_count: events.len(),
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::application::todo::handle_todo_command;
    use crate::infrastructure::event_bus::ZenohEventBus;
    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use chrono::Utc;
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

    fn create_router(repo: Arc<SqliteEventRepository<TodoCommand, TodoEvent>>) -> Router {
        Router::new()
            .route("/api/todos", get(list_todos))
            .route("/api/todos/{id}", get(get_todo))
            .with_state(TodoAppState {
                repo,
                event_bus: None,
            })
    }

    // Type alias for None event bus to satisfy generic constraint
    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    #[tokio::test]
    async fn list_todos_empty() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let app = create_router(repo);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/todos")
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

        assert_eq!(json["todos"], serde_json::json!([]));
        assert_eq!(json["count"], 0);
        assert_eq!(json["completedCount"], 0);
        assert_eq!(json["activeCount"], 0);
    }

    #[tokio::test]
    async fn list_todos_returns_all() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let now = Utc::now();

        // Create some todos
        let id1 = TodoId::new();
        let id2 = TodoId::new();

        handle_todo_command(
            Arc::clone(&repo),
            NO_EVENT_BUS,
            TodoCommand::Create {
                id: id1,
                text: "First".to_string(),
                created_at: now,
            },
        )
        .await
        .expect("create first todo");

        handle_todo_command(
            Arc::clone(&repo),
            NO_EVENT_BUS,
            TodoCommand::Create {
                id: id2,
                text: "Second".to_string(),
                created_at: now,
            },
        )
        .await
        .expect("create second todo");

        handle_todo_command(
            Arc::clone(&repo),
            NO_EVENT_BUS,
            TodoCommand::Complete {
                id: id1,
                completed_at: now,
            },
        )
        .await
        .expect("complete first todo");

        let app = create_router(repo);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/todos")
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

        assert_eq!(json["count"], 2);
        assert_eq!(json["completedCount"], 1);
        assert_eq!(json["activeCount"], 1);
    }

    #[tokio::test]
    async fn get_todo_found() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let now = Utc::now();

        let id = TodoId::new();
        handle_todo_command(
            Arc::clone(&repo),
            NO_EVENT_BUS,
            TodoCommand::Create {
                id,
                text: "Test todo".to_string(),
                created_at: now,
            },
        )
        .await
        .expect("create todo");

        let app = create_router(repo);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/todos/{}", id.into_inner()))
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

        assert_eq!(json["text"], "Test todo");
        assert_eq!(json["completed"], false);
    }

    #[tokio::test]
    async fn get_todo_not_found() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let app = create_router(repo);

        let nonexistent_id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/todos/{}", nonexistent_id))
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_todo_deleted_returns_not_found() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let now = Utc::now();

        let id = TodoId::new();
        handle_todo_command(
            Arc::clone(&repo),
            NO_EVENT_BUS,
            TodoCommand::Create {
                id,
                text: "To be deleted".to_string(),
                created_at: now,
            },
        )
        .await
        .expect("create todo");

        handle_todo_command(
            Arc::clone(&repo),
            NO_EVENT_BUS,
            TodoCommand::Delete {
                id,
                deleted_at: now,
            },
        )
        .await
        .expect("delete todo");

        let app = create_router(repo);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/todos/{}", id.into_inner()))
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // TODO(ironstar-a9b.11): Add integration tests for command handlers
    // The command handlers compile correctly but have an issue with axum's
    // Handler trait bounds in unit test context. Router-based tests will be
    // added as integration tests in crates/ironstar/tests/todo_commands.rs
    // where the full application context is available.
}
