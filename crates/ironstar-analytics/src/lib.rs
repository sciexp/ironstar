//! Analytics domain: Catalog aggregate, QuerySession aggregate, Chart values, combined decider.
//!
//! This crate contains the analytics bounded context for ironstar, including
//! value objects (QueryId, DatasetRef, SqlQuery, ChartConfig), the Catalog and
//! QuerySession aggregates, the combined analytics decider, workflow pipeline,
//! and read-side views.

pub mod catalog;
pub mod combined;
pub mod errors;
pub mod query_session;
pub mod values;
pub mod views;
pub mod workflow;

// Re-export combined decider
pub use combined::{
    AnalyticsCommand, AnalyticsDecider, AnalyticsEvent, AnalyticsState, CombinedDeciderError,
    analytics_decider,
};

// Re-export errors
pub use errors::{
    AnalyticsError, AnalyticsErrorKind, AnalyticsValidationError, AnalyticsValidationErrorKind,
};

// Re-export values
pub use values::{
    ChartConfig, ChartType, DATASET_REF_MAX_LENGTH, DatasetRef, QueryId, SQL_QUERY_MAX_LENGTH,
    SqlQuery,
};

// Re-export workflow types and functions
pub use workflow::{
    ChartData, DatasetSchema, QueryExecutor, QueryResult, SchemaLoader, WorkflowResult,
    execute_workflow, transform_for_chart, validate_schema_compatibility, validate_workflow_inputs,
};
