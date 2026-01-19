//! Todo HTTP handlers.
//!
//! This module provides axum handlers for Todo query endpoints.
//! Handlers call the application layer query functions and format
//! responses as JSON.
//!
//! # Routes
//!
//! - `GET /api/todos` - List all todos
//! - `GET /api/todos/:id` - Get a single todo by ID

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::todo::{query_all_todos, query_todo_state};
use crate::domain::signals::TodoItemView;
use crate::domain::todo::commands::TodoCommand;
use crate::domain::todo::events::TodoEvent;
use crate::domain::todo::values::TodoId;
use crate::infrastructure::event_store::SqliteEventRepository;
use crate::presentation::error::AppError;

/// Application state for Todo handlers.
///
/// Contains the event repository needed for query operations.
#[derive(Clone)]
pub struct TodoAppState {
    pub repo: Arc<SqliteEventRepository<TodoCommand, TodoEvent>>,
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
pub async fn list_todos(
    State(state): State<TodoAppState>,
) -> Result<impl IntoResponse, AppError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::todo::handle_todo_command;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use axum::Router;
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
            .with_state(TodoAppState { repo })
    }

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
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

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
            TodoCommand::Create {
                id: id1,
                text: "First".to_string(),
                created_at: now,
            },
        )
        .await
        .unwrap();

        handle_todo_command(
            Arc::clone(&repo),
            TodoCommand::Create {
                id: id2,
                text: "Second".to_string(),
                created_at: now,
            },
        )
        .await
        .unwrap();

        handle_todo_command(
            Arc::clone(&repo),
            TodoCommand::Complete {
                id: id1,
                completed_at: now,
            },
        )
        .await
        .unwrap();

        let app = create_router(repo);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/todos")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

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
            TodoCommand::Create {
                id,
                text: "Test todo".to_string(),
                created_at: now,
            },
        )
        .await
        .unwrap();

        let app = create_router(repo);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/todos/{}", id.into_inner()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

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
                    .uri(&format!("/api/todos/{}", nonexistent_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

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
            TodoCommand::Create {
                id,
                text: "To be deleted".to_string(),
                created_at: now,
            },
        )
        .await
        .unwrap();

        handle_todo_command(
            Arc::clone(&repo),
            TodoCommand::Delete {
                id,
                deleted_at: now,
            },
        )
        .await
        .unwrap();

        let app = create_router(repo);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/todos/{}", id.into_inner()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
