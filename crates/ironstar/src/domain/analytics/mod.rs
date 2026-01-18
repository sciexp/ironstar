//! Analytics domain value objects with smart constructors.
//!
//! This module contains validated value objects for the analytics domain,
//! following the same "parse, don't validate" principle as Todo values.
//! These types form the vocabulary for the QuerySession aggregate and
//! analytics workflows.
//!
//! # Value Objects
//!
//! - [`QueryId`]: Unique identifier for analytics queries (UUID wrapper)
//! - [`DatasetRef`]: Reference to a dataset (HuggingFace, S3, or local path)
//! - [`SqlQuery`]: Validated SQL query string
//! - [`ChartConfig`]: Configuration for ECharts visualization
//!
//! # Workflow
//!
//! The [`workflow`] module provides a pure function pipeline for analytics
//! operations using railway-oriented programming:
//!
//! - [`execute_workflow`]: Main entry point composing validation and execution
//! - [`SchemaLoader`], [`QueryExecutor`]: Effect boundary traits for infrastructure
//! - [`validate_schema_compatibility`], [`transform_for_chart`]: Pure functions
//!
//! # Example
//!
//! ```rust,ignore
//! use ironstar::domain::analytics::{QueryId, DatasetRef, SqlQuery};
//!
//! let query_id = QueryId::new();
//! let dataset = DatasetRef::new("hf://datasets/user/repo")?;
//! let sql = SqlQuery::new("SELECT * FROM dataset LIMIT 10")?;
//! ```

mod errors;
mod values;
pub mod workflow;

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
