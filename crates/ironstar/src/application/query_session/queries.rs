//! QuerySession query handlers for View-based state computation.
//!
//! Query handlers fetch events and fold them through the QuerySessionView
//! to compute current state on demand. QuerySession is a singleton aggregate
//! ("default-session"), so queries do not require an aggregate ID parameter.

use crate::domain::views::{QueryHistoryEntry, QuerySessionViewState, query_session_view};
use crate::domain::QuerySessionEvent;
use crate::infrastructure::error::InfrastructureError;
use crate::infrastructure::event_store::SqliteEventRepository;

/// Query the current session state by replaying events through the View.
///
/// Returns the full `QuerySessionViewState` including current status,
/// query history, and counters. Returns the View's initial state (idle,
/// empty history) if no events exist.
pub async fn query_session_state<C>(
    repo: &SqliteEventRepository<C, QuerySessionEvent>,
) -> Result<QuerySessionViewState, InfrastructureError> {
    let events = repo
        .fetch_events_by_aggregate("QuerySession", "default-session")
        .await?;

    let view = query_session_view();
    let initial_state = (view.initial_state)();

    let state = events
        .iter()
        .fold(initial_state, |state, (event, _version)| {
            (view.evolve)(&state, event)
        });

    Ok(state)
}

/// Query the history of completed, failed, and cancelled queries.
///
/// Convenience wrapper over `query_session_state` that extracts just the
/// history entries. Returns an empty Vec if no queries have reached a
/// terminal state.
pub async fn query_query_history<C>(
    repo: &SqliteEventRepository<C, QuerySessionEvent>,
) -> Result<Vec<QueryHistoryEntry>, InfrastructureError> {
    let state = query_session_state(repo).await?;
    Ok(state.query_history)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::application::query_session::handle_query_session_command;
    use crate::domain::QuerySessionCommand;
    use crate::domain::{QueryId, SqlQuery};
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
    async fn query_empty_returns_idle() {
        let pool = create_test_pool().await;
        let repo: SqliteEventRepository<QuerySessionCommand, QuerySessionEvent> =
            SqliteEventRepository::new(pool);

        let state = query_session_state(&repo)
            .await
            .expect("query should succeed");

        assert!(state.is_idle());
        assert!(state.query_history.is_empty());
        assert_eq!(state.total_finished(), 0);
    }

    #[tokio::test]
    async fn query_after_start_shows_pending() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = QuerySessionCommand::StartQuery {
            query_id: QueryId::new(),
            sql: SqlQuery::new("SELECT 1").unwrap(),
            dataset_ref: None,
            chart_config: None,
            started_at: Utc::now(),
        };
        handle_query_session_command(Arc::clone(&repo), NO_EVENT_BUS, command)
            .await
            .expect("start should succeed");

        let state = query_session_state(&repo)
            .await
            .expect("query should succeed");

        assert!(state.is_in_progress());
        assert!(state.query_history.is_empty());
    }

    #[tokio::test]
    async fn query_history_empty_before_terminal() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = QuerySessionCommand::StartQuery {
            query_id: QueryId::new(),
            sql: SqlQuery::new("SELECT 1").unwrap(),
            dataset_ref: None,
            chart_config: None,
            started_at: Utc::now(),
        };
        handle_query_session_command(Arc::clone(&repo), NO_EVENT_BUS, command)
            .await
            .expect("start should succeed");

        let history = query_query_history(&repo)
            .await
            .expect("query should succeed");

        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn query_history_populated_after_completion() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let qid = QueryId::new();

        // Start query
        let start = QuerySessionCommand::StartQuery {
            query_id: qid,
            sql: SqlQuery::new("SELECT 1").unwrap(),
            dataset_ref: None,
            chart_config: None,
            started_at: Utc::now(),
        };
        handle_query_session_command(Arc::clone(&repo), NO_EVENT_BUS, start)
            .await
            .expect("start should succeed");

        // Begin execution
        let began = QuerySessionCommand::BeginExecution {
            query_id: qid,
            began_at: Utc::now(),
        };
        handle_query_session_command(Arc::clone(&repo), NO_EVENT_BUS, began)
            .await
            .expect("began should succeed");

        // Complete query
        let complete = QuerySessionCommand::CompleteQuery {
            query_id: qid,
            row_count: 42,
            duration_ms: 150,
            completed_at: Utc::now(),
        };
        handle_query_session_command(Arc::clone(&repo), NO_EVENT_BUS, complete)
            .await
            .expect("complete should succeed");

        let history = query_query_history(&repo)
            .await
            .expect("query should succeed");

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].query_id, qid);
    }
}
