//! SavedQuery aggregate for user-defined reusable queries.
//!
//! Manages saved queries that reference datasets and contain SQL for
//! execution by DuckDB. Each query belongs to a workspace and can be
//! renamed, updated, or deleted.
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
//! # Aggregate ID pattern
//!
//! `saved_query_{query_id}` — one aggregate per saved query.
//!
//! # Terminal state
//!
//! Unlike most aggregates, SavedQuery has a terminal transition:
//! `DeleteQuery` returns the aggregate to `NoQuery`. After deletion,
//! `SaveQuery` can succeed again since the aggregate is back in its
//! initial state.
//!
//! # Idempotency
//!
//! All update operations are idempotent (setting the same value
//! returns `Ok(vec![])` with no events emitted).
//!
//! # Module organization
//!
//! - [`commands`]: SavedQueryCommand enum
//! - [`decider`]: saved_query_decider() factory with pure decide/evolve
//! - [`errors`]: SavedQueryError with UUID tracking
//! - [`events`]: SavedQueryEvent enum
//! - [`state`]: SavedQueryState enum (NoQuery | QueryExists)
//! - [`values`]: Value objects (SavedQueryId, QueryName)

pub mod commands;
pub mod decider;
pub mod errors;
pub mod events;
pub mod state;
pub mod values;

pub use commands::SavedQueryCommand;
pub use decider::{SavedQueryDecider, saved_query_decider};
pub use errors::{SavedQueryError, SavedQueryErrorKind};
pub use events::SavedQueryEvent;
pub use state::SavedQueryState;
pub use values::{QUERY_NAME_MAX_LENGTH, QUERY_NAME_MIN_LENGTH, QueryName, SavedQueryId};
