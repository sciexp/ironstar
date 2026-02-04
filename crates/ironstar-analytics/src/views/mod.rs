//! View modules for analytics read-side projections.

pub mod catalog;
pub mod query_session;

pub use catalog::{CatalogView, CatalogViewState, catalog_view};
pub use query_session::{
    QueryHistoryEntry, QueryOutcome, QuerySessionView, QuerySessionViewState, query_session_view,
};
