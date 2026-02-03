//! Spawn-after-persist pattern for async DuckDB query execution.
//!
//! After a `QueryStarted` event is persisted, this module spawns a background
//! tokio task that:
//!
//! 1. Issues `BeginExecution` command through the Decider (persists `ExecutionBegan`)
//! 2. Executes the DuckDB query via `DuckDBService`
//! 3. Issues `CompleteQuery` or `FailQuery` command through the Decider
//!
//! All state transitions flow through the Decider, preserving the aggregate
//! invariant. The spawned task is just an async command issuer.
//!
//! # Error handling
//!
//! Errors in the spawned task are logged but do not propagate to the caller.
//! The caller has already received 202 Accepted after the initial `StartQuery`
//! command succeeded. Clients observe query progress via SSE events published
//! by the Decider's event bus integration.

use crate::domain::analytics::{QueryId, SqlQuery};
use crate::domain::query_session::{QuerySessionCommand, QuerySessionEvent};
use crate::infrastructure::analytics::DuckDBService;
use crate::infrastructure::event_bus::ZenohEventBus;
use crate::infrastructure::event_store::SqliteEventRepository;
use chrono::Utc;
use std::sync::Arc;
use tokio::task::JoinHandle;

use super::handlers::handle_query_session_command_zenoh;

/// Parameters extracted from a persisted `QueryStarted` event for spawning
/// async DuckDB execution.
#[derive(Debug, Clone)]
pub struct QueryExecutionParams {
    /// The query ID from the `QueryStarted` event.
    pub query_id: QueryId,
    /// The SQL query to execute.
    pub sql: SqlQuery,
}

impl QueryExecutionParams {
    /// Extract execution parameters from a `QueryStarted` event.
    ///
    /// Returns `None` if the event is not a `QueryStarted` variant.
    #[must_use]
    pub fn from_event(event: &QuerySessionEvent) -> Option<Self> {
        match event {
            QuerySessionEvent::QueryStarted {
                query_id, sql, ..
            } => Some(Self {
                query_id: *query_id,
                sql: sql.clone(),
            }),
            _ => None,
        }
    }
}

/// Spawn a background task for DuckDB query execution.
///
/// This function implements the spawn-after-persist pattern:
/// - Called after `QueryStarted` event is persisted
/// - Spawns a `tokio::spawn` task that issues commands back through the Decider
/// - Returns a `JoinHandle` for optional abort capability
///
/// The spawned task:
/// 1. Issues `BeginExecution` → Decider persists `ExecutionBegan`
/// 2. Runs the SQL query against DuckDB
/// 3. On success: issues `CompleteQuery` → Decider persists `QueryCompleted`
/// 4. On failure: issues `FailQuery` → Decider persists `QueryFailed`
///
/// # Arguments
///
/// * `event_repository` - Shared SQLite event repository for command handling
/// * `event_bus` - Optional Zenoh event bus for post-persist notification
/// * `duckdb_service` - DuckDB service for query execution
/// * `params` - Query parameters extracted from the `QueryStarted` event
pub fn spawn_query_execution(
    event_repository: Arc<SqliteEventRepository<QuerySessionCommand, QuerySessionEvent>>,
    event_bus: Option<Arc<ZenohEventBus>>,
    duckdb_service: DuckDBService,
    params: QueryExecutionParams,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let query_id = params.query_id;
        let sql_str = params.sql.as_str().to_string();

        // Step 1: Issue BeginExecution command
        let begin_cmd = QuerySessionCommand::BeginExecution {
            query_id,
            began_at: Utc::now(),
        };

        let bus_ref = event_bus.as_deref();
        if let Err(e) = handle_query_session_command_zenoh(
            Arc::clone(&event_repository),
            bus_ref,
            begin_cmd,
        )
        .await
        {
            tracing::error!(
                query_id = %query_id,
                error = %e,
                "Failed to issue BeginExecution command"
            );
            return;
        }

        tracing::info!(query_id = %query_id, "Query execution began");

        // Step 2: Execute the DuckDB query
        let start_time = std::time::Instant::now();
        let query_result = duckdb_service
            .query(move |conn| {
                let mut stmt = conn.prepare(&sql_str)?;
                let mut rows = stmt.query([])?;
                let mut row_count: usize = 0;
                while rows.next()?.is_some() {
                    row_count += 1;
                }
                Ok(row_count)
            })
            .await;
        let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(0);

        // Step 3: Issue completion or failure command
        match query_result {
            Ok(row_count) => {
                let complete_cmd = QuerySessionCommand::CompleteQuery {
                    query_id,
                    row_count,
                    duration_ms,
                    completed_at: Utc::now(),
                };

                if let Err(e) = handle_query_session_command_zenoh(
                    Arc::clone(&event_repository),
                    bus_ref,
                    complete_cmd,
                )
                .await
                {
                    tracing::error!(
                        query_id = %query_id,
                        error = %e,
                        "Failed to issue CompleteQuery command"
                    );
                    return;
                }

                tracing::info!(
                    query_id = %query_id,
                    row_count,
                    duration_ms,
                    "Query completed successfully"
                );
            }
            Err(e) => {
                let fail_cmd = QuerySessionCommand::FailQuery {
                    query_id,
                    error: e.to_string(),
                    failed_at: Utc::now(),
                };

                if let Err(cmd_err) = handle_query_session_command_zenoh(
                    Arc::clone(&event_repository),
                    bus_ref,
                    fail_cmd,
                )
                .await
                {
                    tracing::error!(
                        query_id = %query_id,
                        original_error = %e,
                        command_error = %cmd_err,
                        "Failed to issue FailQuery command after query execution failure"
                    );
                    return;
                }

                tracing::warn!(
                    query_id = %query_id,
                    error = %e,
                    "Query execution failed"
                );
            }
        }
    })
}
