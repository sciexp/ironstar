//! Chart transformation from query results to ECharts configuration.
//!
//! This module provides types and traits for transforming DuckDB query results
//! into ECharts option JSON. This is a presentation concern: the transformation
//! is a view projection that maps tabular data to chart-specific configurations.
//!
//! # Architecture
//!
//! The transformation follows a quotient projection pattern where multiple
//! query result shapes can map to the same chart configuration. Each chart
//! type has specific data requirements that the transformer validates.
//!
//! ```text
//! QueryResult ──> ChartTransformer ──> serde_json::Value (ECharts option)
//!     │                  │
//!     │                  └── validates columns, types, data
//!     │
//!     └── columns: Vec<ColumnMetadata>
//!         rows: Vec<Vec<serde_json::Value>>
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use ironstar::presentation::chart_transformer::{
//!     ChartConfig, ChartTransformer, ChartType, QueryResult,
//! };
//!
//! let result = QueryResult {
//!     columns: vec![
//!         ColumnMetadata { name: "category".into(), data_type: "VARCHAR".into() },
//!         ColumnMetadata { name: "value".into(), data_type: "INTEGER".into() },
//!     ],
//!     rows: vec![
//!         vec![json!("A"), json!(10)],
//!         vec![json!("B"), json!(20)],
//!     ],
//! };
//!
//! let config = ChartConfig {
//!     chart_type: ChartType::Bar,
//!     title: Some("Sales by Category".into()),
//!     category_column: "category".into(),
//!     value_columns: vec!["value".into()],
//! };
//!
//! let transformer = BarChartTransformer;
//! let echarts_option = transformer.transform(&result, &config)?;
//! ```

use serde::{Deserialize, Serialize};

/// Column metadata from DuckDB query results.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnMetadata {
    /// Column name as returned by DuckDB.
    pub name: String,
    /// DuckDB type as string (e.g., "VARCHAR", "INTEGER", "DOUBLE").
    pub data_type: String,
}

/// Query result for chart transformation.
///
/// Captures DuckDB query output in a format suitable for chart transformers.
/// This is a presentation-layer type, not a domain type. It serves as the
/// input to chart transformers and is constructed from DuckDB query results.
#[derive(Clone, Debug)]
pub struct QueryResult {
    /// Column metadata describing the result schema.
    pub columns: Vec<ColumnMetadata>,
    /// Row data as JSON values, matching column order.
    pub rows: Vec<Vec<serde_json::Value>>,
}

impl QueryResult {
    /// Create a new query result.
    #[must_use]
    pub fn new(columns: Vec<ColumnMetadata>, rows: Vec<Vec<serde_json::Value>>) -> Self {
        Self { columns, rows }
    }

    /// Check if the result set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get the number of rows.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the number of columns.
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Find a column by name, returning its index if found.
    #[must_use]
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }

    /// Get column metadata by name.
    #[must_use]
    pub fn column(&self, name: &str) -> Option<&ColumnMetadata> {
        self.columns.iter().find(|c| c.name == name)
    }
}

/// Error type for chart transformation failures.
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    /// A required column is missing from the query result.
    #[error("missing required column: {0}")]
    MissingColumn(String),

    /// Column has wrong data type for the chart.
    #[error("invalid data type for column {column}: expected {expected}, got {actual}")]
    InvalidDataType {
        /// Column name.
        column: String,
        /// Expected DuckDB type.
        expected: String,
        /// Actual DuckDB type.
        actual: String,
    },

    /// Query returned no rows.
    #[error("empty result set")]
    EmptyResult,

    /// General transformation failure.
    #[error("transformation failed: {0}")]
    TransformFailed(String),
}

/// Configuration for chart transformation.
///
/// Specifies how to map query result columns to chart axes and series.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Type of chart to generate.
    pub chart_type: ChartType,
    /// Optional chart title.
    pub title: Option<String>,
    /// Column to use for category axis (x-axis for bar/line, labels for pie).
    pub category_column: String,
    /// Columns to use for value axis (y-axis for bar/line, values for pie).
    pub value_columns: Vec<String>,
}

/// Supported chart types for ECharts transformation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    /// Vertical bar chart.
    Bar,
    /// Line chart with optional area fill.
    Line,
    /// Pie or donut chart.
    Pie,
    /// Scatter plot for correlation analysis.
    Scatter,
}

impl ChartType {
    /// Get the ECharts series type string.
    #[must_use]
    pub fn echarts_type(&self) -> &'static str {
        match self {
            Self::Bar => "bar",
            Self::Line => "line",
            Self::Pie => "pie",
            Self::Scatter => "scatter",
        }
    }
}

/// Transforms DuckDB query results into ECharts option JSON.
///
/// Implements the quotient projection from QueryResult to ECharts configuration.
/// Each implementation handles a specific chart type's data requirements.
///
/// # Implementors
///
/// Implementations should:
/// - Validate that required columns exist in the query result
/// - Check data types are compatible with the chart type
/// - Return `TransformError` for invalid inputs rather than panicking
/// - Produce valid ECharts option JSON that can be passed directly to `setOption()`
pub trait ChartTransformer {
    /// Transform query result into ECharts option JSON.
    ///
    /// # Arguments
    ///
    /// * `result` - Query result from DuckDB
    /// * `config` - Chart configuration specifying column mappings
    ///
    /// # Errors
    ///
    /// Returns `TransformError` if:
    /// - Required columns are missing
    /// - Column data types are incompatible
    /// - Result set is empty (when chart requires data)
    fn transform(
        &self,
        result: &QueryResult,
        config: &ChartConfig,
    ) -> Result<serde_json::Value, TransformError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn query_result_construction() {
        let columns = vec![
            ColumnMetadata {
                name: "category".into(),
                data_type: "VARCHAR".into(),
            },
            ColumnMetadata {
                name: "value".into(),
                data_type: "INTEGER".into(),
            },
        ];
        let rows = vec![
            vec![json!("A"), json!(10)],
            vec![json!("B"), json!(20)],
            vec![json!("C"), json!(30)],
        ];

        let result = QueryResult::new(columns, rows);

        assert_eq!(result.column_count(), 2);
        assert_eq!(result.row_count(), 3);
        assert!(!result.is_empty());
    }

    #[test]
    fn query_result_empty() {
        let columns = vec![ColumnMetadata {
            name: "x".into(),
            data_type: "INTEGER".into(),
        }];
        let rows = vec![];

        let result = QueryResult::new(columns, rows);

        assert!(result.is_empty());
        assert_eq!(result.row_count(), 0);
    }

    #[test]
    fn query_result_column_lookup() {
        let columns = vec![
            ColumnMetadata {
                name: "a".into(),
                data_type: "VARCHAR".into(),
            },
            ColumnMetadata {
                name: "b".into(),
                data_type: "INTEGER".into(),
            },
            ColumnMetadata {
                name: "c".into(),
                data_type: "DOUBLE".into(),
            },
        ];
        let result = QueryResult::new(columns, vec![]);

        assert_eq!(result.column_index("a"), Some(0));
        assert_eq!(result.column_index("b"), Some(1));
        assert_eq!(result.column_index("c"), Some(2));
        assert_eq!(result.column_index("nonexistent"), None);

        let col_b = result.column("b").unwrap();
        assert_eq!(col_b.data_type, "INTEGER");
    }

    #[test]
    fn chart_type_echarts_string() {
        assert_eq!(ChartType::Bar.echarts_type(), "bar");
        assert_eq!(ChartType::Line.echarts_type(), "line");
        assert_eq!(ChartType::Pie.echarts_type(), "pie");
        assert_eq!(ChartType::Scatter.echarts_type(), "scatter");
    }

    #[test]
    fn chart_config_serialization() {
        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: Some("Test Chart".into()),
            category_column: "month".into(),
            value_columns: vec!["sales".into(), "revenue".into()],
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"chart_type\":\"bar\""));
        assert!(json.contains("\"title\":\"Test Chart\""));
    }

    #[test]
    fn column_metadata_equality() {
        let col1 = ColumnMetadata {
            name: "x".into(),
            data_type: "INTEGER".into(),
        };
        let col2 = ColumnMetadata {
            name: "x".into(),
            data_type: "INTEGER".into(),
        };
        let col3 = ColumnMetadata {
            name: "x".into(),
            data_type: "VARCHAR".into(),
        };

        assert_eq!(col1, col2);
        assert_ne!(col1, col3);
    }

    #[test]
    fn transform_error_display() {
        let err = TransformError::MissingColumn("category".into());
        assert_eq!(err.to_string(), "missing required column: category");

        let err = TransformError::InvalidDataType {
            column: "value".into(),
            expected: "INTEGER".into(),
            actual: "VARCHAR".into(),
        };
        assert_eq!(
            err.to_string(),
            "invalid data type for column value: expected INTEGER, got VARCHAR"
        );

        let err = TransformError::EmptyResult;
        assert_eq!(err.to_string(), "empty result set");

        let err = TransformError::TransformFailed("custom error".into());
        assert_eq!(err.to_string(), "transformation failed: custom error");
    }
}
