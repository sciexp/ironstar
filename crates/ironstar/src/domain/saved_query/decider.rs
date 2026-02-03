//! Pure SavedQuery Decider implementing fmodel-rust patterns.
//!
//! The Decider embodies the state machine from
//! `spec/Workspace/SavedQuery.idr`. It is a pure function with
//! no side effects: all I/O (timestamps, validation) happens at boundaries.
//!
//! # State machine
//!
//! ```text
//!                 ┌───────────┐
//!  SaveQuery ────►│QueryExists│◄──── RenameQuery, UpdateSql, UpdateDatasetRef
//!                 └─────┬─────┘
//!                       │
//!                  DeleteQuery
//!                       │
//!                       ▼
//!                 ┌───────────┐
//!                 │  NoQuery  │ (terminal / can be re-created)
//!                 └───────────┘
//! ```
//!
//! # Idempotency
//!
//! All update operations are idempotent:
//! - RenameQuery with same name returns `Ok(vec![])`
//! - UpdateQuerySql with same SQL returns `Ok(vec![])`
//! - UpdateDatasetRef with same reference returns `Ok(vec![])`
//!
//! # Terminal state
//!
//! DeleteQuery transitions back to NoQuery. After deletion, SaveQuery
//! can succeed again since the aggregate is in NoQuery state.

use fmodel_rust::decider::Decider;

use super::commands::SavedQueryCommand;
use super::errors::SavedQueryError;
use super::events::SavedQueryEvent;
use super::state::SavedQueryState;

/// Type alias for the SavedQuery Decider.
pub type SavedQueryDecider<'a> = Decider<
    'a,
    SavedQueryCommand,
    SavedQueryState,
    SavedQueryEvent,
    SavedQueryError,
>;

/// Factory function creating a pure SavedQuery Decider.
///
/// Translates the specification from `spec/Workspace/SavedQuery.idr`
/// into Rust, preserving the state machine transitions and idempotency invariants.
pub fn saved_query_decider<'a>() -> SavedQueryDecider<'a> {
    Decider {
        decide: Box::new(decide),
        evolve: Box::new(evolve),
        initial_state: Box::new(SavedQueryState::default),
    }
}

/// Pure decide function: (Command, State) -> Result<Vec<Event>, Error>
fn decide(
    command: &SavedQueryCommand,
    state: &SavedQueryState,
) -> Result<Vec<SavedQueryEvent>, SavedQueryError> {
    match (command, state) {
        // SaveQuery: NoQuery -> QueryExists
        (
            SavedQueryCommand::SaveQuery {
                query_id,
                workspace_id,
                name,
                sql,
                dataset_ref,
                saved_at,
            },
            SavedQueryState::NoQuery,
        ) => Ok(vec![SavedQueryEvent::QuerySaved {
            query_id: *query_id,
            workspace_id: *workspace_id,
            name: name.clone(),
            sql: sql.clone(),
            dataset_ref: dataset_ref.clone(),
            saved_at: *saved_at,
        }]),

        // SaveQuery when already exists
        (
            SavedQueryCommand::SaveQuery { .. },
            SavedQueryState::QueryExists { .. },
        ) => Err(SavedQueryError::already_exists()),

        // DeleteQuery: QueryExists -> NoQuery (terminal)
        (
            SavedQueryCommand::DeleteQuery {
                query_id,
                deleted_at,
            },
            SavedQueryState::QueryExists { .. },
        ) => Ok(vec![SavedQueryEvent::QueryDeleted {
            query_id: *query_id,
            deleted_at: *deleted_at,
        }]),

        // DeleteQuery when no query exists
        (
            SavedQueryCommand::DeleteQuery { .. },
            SavedQueryState::NoQuery,
        ) => Err(SavedQueryError::not_found()),

        // RenameQuery: QueryExists -> QueryExists (idempotent if same name)
        (
            SavedQueryCommand::RenameQuery {
                query_id,
                name,
                renamed_at,
            },
            SavedQueryState::QueryExists {
                name: current_name, ..
            },
        ) => {
            if current_name == name {
                return Ok(vec![]);
            }

            Ok(vec![SavedQueryEvent::QueryRenamed {
                query_id: *query_id,
                name: name.clone(),
                renamed_at: *renamed_at,
            }])
        }

        // RenameQuery when no query exists
        (
            SavedQueryCommand::RenameQuery { .. },
            SavedQueryState::NoQuery,
        ) => Err(SavedQueryError::not_found()),

        // UpdateQuerySql: QueryExists -> QueryExists (idempotent if same SQL)
        (
            SavedQueryCommand::UpdateQuerySql {
                query_id,
                sql,
                updated_at,
            },
            SavedQueryState::QueryExists {
                sql: current_sql, ..
            },
        ) => {
            if current_sql == sql {
                return Ok(vec![]);
            }

            Ok(vec![SavedQueryEvent::QuerySqlUpdated {
                query_id: *query_id,
                sql: sql.clone(),
                updated_at: *updated_at,
            }])
        }

        // UpdateQuerySql when no query exists
        (
            SavedQueryCommand::UpdateQuerySql { .. },
            SavedQueryState::NoQuery,
        ) => Err(SavedQueryError::not_found()),

        // UpdateDatasetRef: QueryExists -> QueryExists (idempotent if same ref)
        (
            SavedQueryCommand::UpdateDatasetRef {
                query_id,
                dataset_ref,
                updated_at,
            },
            SavedQueryState::QueryExists {
                dataset_ref: current_ref,
                ..
            },
        ) => {
            if current_ref == dataset_ref {
                return Ok(vec![]);
            }

            Ok(vec![SavedQueryEvent::DatasetRefUpdated {
                query_id: *query_id,
                dataset_ref: dataset_ref.clone(),
                updated_at: *updated_at,
            }])
        }

        // UpdateDatasetRef when no query exists
        (
            SavedQueryCommand::UpdateDatasetRef { .. },
            SavedQueryState::NoQuery,
        ) => Err(SavedQueryError::not_found()),
    }
}

/// Pure evolve function: (State, Event) -> State
fn evolve(
    state: &SavedQueryState,
    event: &SavedQueryEvent,
) -> SavedQueryState {
    match event {
        SavedQueryEvent::QuerySaved {
            query_id,
            workspace_id,
            name,
            sql,
            dataset_ref,
            ..
        } => SavedQueryState::QueryExists {
            query_id: *query_id,
            workspace_id: *workspace_id,
            name: name.clone(),
            sql: sql.clone(),
            dataset_ref: dataset_ref.clone(),
        },

        SavedQueryEvent::QueryDeleted { .. } => SavedQueryState::NoQuery,

        SavedQueryEvent::QueryRenamed { name, .. } => match state {
            SavedQueryState::QueryExists {
                query_id,
                workspace_id,
                sql,
                dataset_ref,
                ..
            } => SavedQueryState::QueryExists {
                query_id: *query_id,
                workspace_id: *workspace_id,
                name: name.clone(),
                sql: sql.clone(),
                dataset_ref: dataset_ref.clone(),
            },
            SavedQueryState::NoQuery => state.clone(),
        },

        SavedQueryEvent::QuerySqlUpdated { sql, .. } => match state {
            SavedQueryState::QueryExists {
                query_id,
                workspace_id,
                name,
                dataset_ref,
                ..
            } => SavedQueryState::QueryExists {
                query_id: *query_id,
                workspace_id: *workspace_id,
                name: name.clone(),
                sql: sql.clone(),
                dataset_ref: dataset_ref.clone(),
            },
            SavedQueryState::NoQuery => state.clone(),
        },

        SavedQueryEvent::DatasetRefUpdated { dataset_ref, .. } => match state {
            SavedQueryState::QueryExists {
                query_id,
                workspace_id,
                name,
                sql,
                ..
            } => SavedQueryState::QueryExists {
                query_id: *query_id,
                workspace_id: *workspace_id,
                name: name.clone(),
                sql: sql.clone(),
                dataset_ref: dataset_ref.clone(),
            },
            SavedQueryState::NoQuery => state.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use fmodel_rust::specification::DeciderTestSpecification;

    use super::super::values::{QueryName, SavedQueryId};
    use crate::domain::analytics::{DatasetRef, SqlQuery};
    use crate::domain::workspace::WorkspaceId;

    fn sample_query_id() -> SavedQueryId {
        SavedQueryId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_workspace_id() -> WorkspaceId {
        WorkspaceId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn sample_name() -> QueryName {
        QueryName::new("Monthly Revenue").unwrap()
    }

    fn sample_sql() -> SqlQuery {
        SqlQuery::new("SELECT * FROM sales GROUP BY month").unwrap()
    }

    fn sample_dataset_ref() -> DatasetRef {
        DatasetRef::new("hf://datasets/sciexp/sales-data").unwrap()
    }

    fn saved_event() -> SavedQueryEvent {
        SavedQueryEvent::QuerySaved {
            query_id: sample_query_id(),
            workspace_id: sample_workspace_id(),
            name: sample_name(),
            sql: sample_sql(),
            dataset_ref: sample_dataset_ref(),
            saved_at: sample_time(),
        }
    }

    // --- SaveQuery transitions ---

    #[test]
    fn save_query_from_no_query_succeeds() {
        let qid = sample_query_id();
        let ws_id = sample_workspace_id();
        let ts = sample_time();
        let name = sample_name();
        let sql = sample_sql();
        let dataset = sample_dataset_ref();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![])
            .when(SavedQueryCommand::SaveQuery {
                query_id: qid,
                workspace_id: ws_id,
                name: name.clone(),
                sql: sql.clone(),
                dataset_ref: dataset.clone(),
                saved_at: ts,
            })
            .then(vec![SavedQueryEvent::QuerySaved {
                query_id: qid,
                workspace_id: ws_id,
                name,
                sql,
                dataset_ref: dataset,
                saved_at: ts,
            }]);
    }

    #[test]
    fn save_query_when_already_exists_fails() {
        let qid = sample_query_id();
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![saved_event()])
            .when(SavedQueryCommand::SaveQuery {
                query_id: qid,
                workspace_id: ws_id,
                name: QueryName::new("Other").unwrap(),
                sql: SqlQuery::new("SELECT 1").unwrap(),
                dataset_ref: DatasetRef::new("hf://datasets/other/data").unwrap(),
                saved_at: ts,
            })
            .then_error(SavedQueryError::already_exists());
    }

    // --- DeleteQuery transitions ---

    #[test]
    fn delete_query_succeeds() {
        let qid = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![saved_event()])
            .when(SavedQueryCommand::DeleteQuery {
                query_id: qid,
                deleted_at: ts,
            })
            .then(vec![SavedQueryEvent::QueryDeleted {
                query_id: qid,
                deleted_at: ts,
            }]);
    }

    #[test]
    fn delete_query_when_no_query_fails() {
        let qid = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![])
            .when(SavedQueryCommand::DeleteQuery {
                query_id: qid,
                deleted_at: ts,
            })
            .then_error(SavedQueryError::not_found());
    }

    // --- RenameQuery transitions ---

    #[test]
    fn rename_query_succeeds() {
        let qid = sample_query_id();
        let ts = sample_time();
        let new_name = QueryName::new("Quarterly Revenue").unwrap();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![saved_event()])
            .when(SavedQueryCommand::RenameQuery {
                query_id: qid,
                name: new_name.clone(),
                renamed_at: ts,
            })
            .then(vec![SavedQueryEvent::QueryRenamed {
                query_id: qid,
                name: new_name,
                renamed_at: ts,
            }]);
    }

    #[test]
    fn rename_query_same_name_is_idempotent() {
        let qid = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![saved_event()])
            .when(SavedQueryCommand::RenameQuery {
                query_id: qid,
                name: sample_name(),
                renamed_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn rename_query_when_no_query_fails() {
        let qid = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![])
            .when(SavedQueryCommand::RenameQuery {
                query_id: qid,
                name: QueryName::new("Any Name").unwrap(),
                renamed_at: ts,
            })
            .then_error(SavedQueryError::not_found());
    }

    // --- UpdateQuerySql transitions ---

    #[test]
    fn update_sql_succeeds() {
        let qid = sample_query_id();
        let ts = sample_time();
        let new_sql = SqlQuery::new("SELECT COUNT(*) FROM sales").unwrap();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![saved_event()])
            .when(SavedQueryCommand::UpdateQuerySql {
                query_id: qid,
                sql: new_sql.clone(),
                updated_at: ts,
            })
            .then(vec![SavedQueryEvent::QuerySqlUpdated {
                query_id: qid,
                sql: new_sql,
                updated_at: ts,
            }]);
    }

    #[test]
    fn update_sql_same_value_is_idempotent() {
        let qid = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![saved_event()])
            .when(SavedQueryCommand::UpdateQuerySql {
                query_id: qid,
                sql: sample_sql(),
                updated_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn update_sql_when_no_query_fails() {
        let qid = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![])
            .when(SavedQueryCommand::UpdateQuerySql {
                query_id: qid,
                sql: SqlQuery::new("SELECT 1").unwrap(),
                updated_at: ts,
            })
            .then_error(SavedQueryError::not_found());
    }

    // --- UpdateDatasetRef transitions ---

    #[test]
    fn update_dataset_ref_succeeds() {
        let qid = sample_query_id();
        let ts = sample_time();
        let new_ref = DatasetRef::new("s3://bucket/new-data").unwrap();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![saved_event()])
            .when(SavedQueryCommand::UpdateDatasetRef {
                query_id: qid,
                dataset_ref: new_ref.clone(),
                updated_at: ts,
            })
            .then(vec![SavedQueryEvent::DatasetRefUpdated {
                query_id: qid,
                dataset_ref: new_ref,
                updated_at: ts,
            }]);
    }

    #[test]
    fn update_dataset_ref_same_value_is_idempotent() {
        let qid = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![saved_event()])
            .when(SavedQueryCommand::UpdateDatasetRef {
                query_id: qid,
                dataset_ref: sample_dataset_ref(),
                updated_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn update_dataset_ref_when_no_query_fails() {
        let qid = sample_query_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![])
            .when(SavedQueryCommand::UpdateDatasetRef {
                query_id: qid,
                dataset_ref: DatasetRef::new("hf://datasets/other/data").unwrap(),
                updated_at: ts,
            })
            .then_error(SavedQueryError::not_found());
    }

    // --- Full lifecycle ---

    #[test]
    fn full_lifecycle() {
        let qid = sample_query_id();
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        // Save query
        let events = decide(
            &SavedQueryCommand::SaveQuery {
                query_id: qid,
                workspace_id: ws_id,
                name: sample_name(),
                sql: sample_sql(),
                dataset_ref: sample_dataset_ref(),
                saved_at: ts,
            },
            &SavedQueryState::default(),
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&SavedQueryState::default(), &events[0]);
        assert!(state.exists());
        assert_eq!(state.name().unwrap(), &sample_name());
        assert_eq!(state.sql().unwrap(), &sample_sql());
        assert_eq!(state.dataset_ref().unwrap(), &sample_dataset_ref());

        // Rename
        let new_name = QueryName::new("Updated Revenue Report").unwrap();
        let events = decide(
            &SavedQueryCommand::RenameQuery {
                query_id: qid,
                name: new_name.clone(),
                renamed_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.name().unwrap(), &new_name);

        // Update SQL
        let new_sql = SqlQuery::new("SELECT SUM(revenue) FROM sales").unwrap();
        let events = decide(
            &SavedQueryCommand::UpdateQuerySql {
                query_id: qid,
                sql: new_sql.clone(),
                updated_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.sql().unwrap(), &new_sql);

        // Idempotent: update same SQL
        let events = decide(
            &SavedQueryCommand::UpdateQuerySql {
                query_id: qid,
                sql: new_sql,
                updated_at: ts,
            },
            &state,
        )
        .unwrap();
        assert!(events.is_empty());

        // Delete (terminal)
        let events = decide(
            &SavedQueryCommand::DeleteQuery {
                query_id: qid,
                deleted_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert!(!state.exists());
        assert_eq!(state, SavedQueryState::NoQuery);
    }

    // --- Terminal state: re-creation after deletion ---

    #[test]
    fn save_succeeds_after_deletion() {
        let qid = sample_query_id();
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(saved_query_decider())
            .given(vec![
                saved_event(),
                SavedQueryEvent::QueryDeleted {
                    query_id: qid,
                    deleted_at: ts,
                },
            ])
            .when(SavedQueryCommand::SaveQuery {
                query_id: qid,
                workspace_id: ws_id,
                name: QueryName::new("Re-created Query").unwrap(),
                sql: SqlQuery::new("SELECT 1").unwrap(),
                dataset_ref: DatasetRef::new("hf://datasets/new/data").unwrap(),
                saved_at: ts,
            })
            .then(vec![SavedQueryEvent::QuerySaved {
                query_id: qid,
                workspace_id: ws_id,
                name: QueryName::new("Re-created Query").unwrap(),
                sql: SqlQuery::new("SELECT 1").unwrap(),
                dataset_ref: DatasetRef::new("hf://datasets/new/data").unwrap(),
                saved_at: ts,
            }]);
    }
}
