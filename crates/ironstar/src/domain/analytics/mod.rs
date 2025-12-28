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

// Re-export errors
pub use errors::AnalyticsValidationError;

// Re-export values
pub use values::{
    ChartConfig, ChartType, DatasetRef, QueryId, SqlQuery, DATASET_REF_MAX_LENGTH,
    SQL_QUERY_MAX_LENGTH,
};
