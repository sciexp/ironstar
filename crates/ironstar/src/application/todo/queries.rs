//! Todo query handlers for View-based state computation.
//!
//! This module provides query handlers that fetch events and fold them through
//! the View to compute current state on demand. This implements the compute-on-demand
//! pattern where state is materialized at query time rather than pre-computed.
//!
//! # Compute-on-demand vs. persistent projections
//!
//! The query handlers in this module replay events through the View on each query.
//! This is appropriate for:
//! - Low-traffic read paths where freshness matters more than latency
//! - Development and testing where projection infrastructure isn't yet available
//! - Small aggregates with few events
//!
//! For high-traffic read paths, consider persistent projections that maintain
//! pre-computed state updated via event subscriptions.

use crate::domain::todo::events::TodoEvent;
use crate::domain::todo::values::TodoId;
use crate::domain::views::{TodoViewState, todo_view};
use crate::infrastructure::error::InfrastructureError;
use crate::infrastructure::event_store::SqliteEventRepository;

/// Query the current state of a Todo aggregate by replaying events through the View.
///
/// This function implements compute-on-demand state materialization:
/// 1. Fetch all events for the aggregate from the event store
/// 2. Fold events through the View's evolve function
/// 3. Return the computed state
///
/// # Arguments
///
/// * `repo` - The event repository to fetch events from
/// * `todo_id` - The ID of the Todo aggregate to query
///
/// # Returns
///
/// The current `TodoViewState` computed by replaying all events for this aggregate.
/// Returns the View's initial state (empty) if no events exist for the aggregate.
///
/// # Errors
///
/// Returns `InfrastructureError` if event fetching fails (database error, deserialization error).
/// The View's evolve function is infallible, so no domain errors are possible.
///
/// # Example
///
/// ```rust,ignore
/// use ironstar::application::todo::query_todo_state;
/// use ironstar::domain::TodoId;
///
/// let state = query_todo_state(&repo, &todo_id).await?;
/// println!("Todo count: {}", state.count);
/// ```
pub async fn query_todo_state<C>(
    repo: &SqliteEventRepository<C, TodoEvent>,
    todo_id: &TodoId,
) -> Result<TodoViewState, InfrastructureError> {
    // Fetch events by aggregate type and ID
    let events = repo
        .fetch_events_by_aggregate("Todo", &todo_id.to_string())
        .await?;

    // Create the View and compute state by folding events
    let view = todo_view();
    let initial_state = (view.initial_state)();

    let state = events
        .iter()
        .fold(initial_state, |state, (event, _version)| {
            (view.evolve)(&state, event)
        });

    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::todo::handle_todo_command;
    use crate::domain::todo::commands::TodoCommand;
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

    #[tokio::test]
    async fn query_nonexistent_returns_empty_state() {
        let pool = create_test_pool().await;
        let repo: SqliteEventRepository<TodoCommand, TodoEvent> =
            SqliteEventRepository::new(pool);

        let state = query_todo_state(&repo, &TodoId::new()).await.unwrap();

        assert!(state.todos.is_empty());
        assert_eq!(state.count, 0);
        assert_eq!(state.completed_count, 0);
    }

    #[tokio::test]
    async fn query_after_create_returns_todo() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let id = TodoId::new();
        let command = TodoCommand::Create {
            id,
            text: "Test todo".to_string(),
            created_at: Utc::now(),
        };
        handle_todo_command(Arc::clone(&repo), command).await.unwrap();

        let state = query_todo_state(&repo, &id).await.unwrap();

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.count, 1);
        assert_eq!(state.todos[0].text, "Test todo");
        assert!(!state.todos[0].completed);
    }

    #[tokio::test]
    async fn query_reflects_full_lifecycle() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let id = TodoId::new();
        let now = Utc::now();

        // Create
        let create = TodoCommand::Create {
            id,
            text: "Lifecycle test".to_string(),
            created_at: now,
        };
        handle_todo_command(Arc::clone(&repo), create).await.unwrap();

        // Complete
        let complete = TodoCommand::Complete {
            id,
            completed_at: now,
        };
        handle_todo_command(Arc::clone(&repo), complete).await.unwrap();

        // Query should show completed state
        let state = query_todo_state(&repo, &id).await.unwrap();
        assert_eq!(state.count, 1);
        assert_eq!(state.completed_count, 1);
        assert!(state.todos[0].completed);

        // Uncomplete
        let uncomplete = TodoCommand::Uncomplete {
            id,
            uncompleted_at: now,
        };
        handle_todo_command(Arc::clone(&repo), uncomplete).await.unwrap();

        // Query should show active state
        let state = query_todo_state(&repo, &id).await.unwrap();
        assert_eq!(state.completed_count, 0);
        assert!(!state.todos[0].completed);
    }

    #[tokio::test]
    async fn query_after_delete_returns_empty() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let id = TodoId::new();
        let now = Utc::now();

        // Create then delete
        let create = TodoCommand::Create {
            id,
            text: "To be deleted".to_string(),
            created_at: now,
        };
        handle_todo_command(Arc::clone(&repo), create).await.unwrap();

        let delete = TodoCommand::Delete {
            id,
            deleted_at: now,
        };
        handle_todo_command(Arc::clone(&repo), delete).await.unwrap();

        // Query should show empty state (todo removed from list)
        let state = query_todo_state(&repo, &id).await.unwrap();
        assert!(state.todos.is_empty());
        assert_eq!(state.count, 0);
    }
}
