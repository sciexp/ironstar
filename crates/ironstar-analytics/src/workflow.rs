//! Analytics workflow as pure function pipeline.
//!
//! Pattern: Railway-oriented programming with effect boundaries.
//!
//! ```text
//! validate_inputs -> [load_schema] -> validate_compatibility -> [execute_query] -> transform_for_chart
//!       pure            async              pure                  async              pure
//! ```
//!
//! - Pure functions validate and transform
//! - Async traits define effect boundaries (implemented in infrastructure)
//! - All functions compose via `Result<T, AnalyticsError>`

use std::collections::HashMap;
use std::future::Future;

use crate::errors::{AnalyticsError, AnalyticsValidationError};
use crate::values::{ChartConfig, ChartType, DatasetRef, QueryId, SqlQuery};

// ============================================================================
// Effect Boundary Traits (implemented in infrastructure layer)
// ============================================================================

/// Schema information for a dataset.
#[derive(Debug, Clone)]
pub struct DatasetSchema {
    /// Column name to SQL type mapping.
    pub columns: HashMap<String, String>,
}

/// Query execution result.
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Column names in order.
    pub columns: Vec<String>,
    /// Row data as JSON-compatible values.
    pub rows: Vec<Vec<serde_json::Value>>,
    /// Total row count.
    pub row_count: usize,
    /// Execution time in milliseconds.
    pub execution_time_ms: u64,
}

/// Loads schema from a dataset reference.
///
/// Implemented by infrastructure layer (DuckDB httpfs, local file, etc.)
pub trait SchemaLoader {
    /// Load schema asynchronously.
    fn load_schema(
        &self,
        dataset: &DatasetRef,
    ) -> impl Future<Output = Result<DatasetSchema, AnalyticsError>> + Send;
}

/// Executes a SQL query against a dataset.
///
/// Implemented by infrastructure layer (async-duckdb).
pub trait QueryExecutor {
    /// Execute query asynchronously.
    fn execute(
        &self,
        dataset: &DatasetRef,
        query: &SqlQuery,
    ) -> impl Future<Output = Result<QueryResult, AnalyticsError>> + Send;
}

// ============================================================================
// Pure Validation Functions
// ============================================================================

/// Validates that all inputs are well-formed.
///
/// This is a convenience function that combines individual validations.
/// Note: DatasetRef and SqlQuery are already validated at construction time.
/// This function performs additional cross-cutting validation.
pub fn validate_workflow_inputs(
    _dataset: &DatasetRef,
    _query: &SqlQuery,
    chart_config: Option<&ChartConfig>,
) -> Result<(), AnalyticsError> {
    // DatasetRef and SqlQuery are pre-validated by smart constructors.
    // Validate chart config if present.
    if let Some(config) = chart_config {
        config.validate().map_err(AnalyticsError::validation)?;
    }

    // Additional cross-cutting validation could go here:
    // - Query references columns that exist in schema (requires schema)
    // - Chart axes match query SELECT columns (requires schema)

    Ok(())
}

/// Validates that query columns are compatible with chart configuration.
///
/// Called after schema is loaded to verify column references.
pub fn validate_schema_compatibility(
    schema: &DatasetSchema,
    chart_config: &ChartConfig,
) -> Result<(), AnalyticsError> {
    // Verify x_axis column exists
    if let Some(x_col) = chart_config
        .x_axis()
        .filter(|col| !schema.columns.contains_key(*col))
    {
        return Err(AnalyticsError::validation(
            AnalyticsValidationError::schema_incompatible(format!(
                "x_axis column '{}' not found in schema",
                x_col
            )),
        ));
    }

    // Verify y_axis column exists
    if let Some(y_col) = chart_config
        .y_axis()
        .filter(|col| !schema.columns.contains_key(*col))
    {
        return Err(AnalyticsError::validation(
            AnalyticsValidationError::schema_incompatible(format!(
                "y_axis column '{}' not found in schema",
                y_col
            )),
        ));
    }

    // Verify series column exists if specified
    if let Some(series_col) = chart_config
        .series_column()
        .filter(|col| !schema.columns.contains_key(*col))
    {
        return Err(AnalyticsError::validation(
            AnalyticsValidationError::schema_incompatible(format!(
                "series column '{}' not found in schema",
                series_col
            )),
        ));
    }

    Ok(())
}

// ============================================================================
// Pure Transformation Functions
// ============================================================================

/// Chart-ready data structure.
#[derive(Debug, Clone)]
pub struct ChartData {
    /// Chart type for rendering.
    pub chart_type: ChartType,
    /// X-axis values (labels).
    pub x_values: Vec<serde_json::Value>,
    /// Y-axis values (data points).
    pub y_values: Vec<serde_json::Value>,
    /// Optional series grouping.
    pub series: Option<HashMap<String, Vec<serde_json::Value>>>,
    /// Chart title.
    pub title: Option<String>,
}

/// Transforms query results into chart-ready data.
///
/// Pure transformation based on chart configuration.
pub fn transform_for_chart(
    result: &QueryResult,
    config: &ChartConfig,
) -> Result<ChartData, AnalyticsError> {
    let x_col_idx = config
        .x_axis()
        .and_then(|name| result.columns.iter().position(|c| c == name));

    let y_col_idx = config
        .y_axis()
        .and_then(|name| result.columns.iter().position(|c| c == name));

    let x_values: Vec<serde_json::Value> = match x_col_idx {
        Some(idx) => result
            .rows
            .iter()
            .map(|row| row.get(idx).cloned().unwrap_or(serde_json::Value::Null))
            .collect(),
        None => (0..result.row_count)
            .map(|i| serde_json::Value::Number(i.into()))
            .collect(),
    };

    let y_values: Vec<serde_json::Value> = match y_col_idx {
        Some(idx) => result
            .rows
            .iter()
            .map(|row| row.get(idx).cloned().unwrap_or(serde_json::Value::Null))
            .collect(),
        None => vec![],
    };

    // Apply limit if configured
    let limit = config.limit().unwrap_or(result.row_count);
    let x_values: Vec<_> = x_values.into_iter().take(limit).collect();
    let y_values: Vec<_> = y_values.into_iter().take(limit).collect();

    Ok(ChartData {
        chart_type: config.chart_type(),
        x_values,
        y_values,
        series: None, // Series grouping not yet implemented
        title: config.title().map(String::from),
    })
}

// ============================================================================
// Pipeline Composition
// ============================================================================

/// Result of a complete analytics workflow execution.
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    /// Query identifier for correlation.
    pub query_id: QueryId,
    /// Query execution metrics.
    pub execution_time_ms: u64,
    /// Total rows returned.
    pub row_count: usize,
    /// Chart-ready data (if chart config provided).
    pub chart_data: Option<ChartData>,
    /// Raw query results.
    pub raw_result: QueryResult,
}

/// Executes the complete analytics workflow.
///
/// This is the main entry point for analytics operations.
/// Composes pure validation, async execution, and pure transformation.
pub async fn execute_workflow<S, E>(
    schema_loader: &S,
    query_executor: &E,
    query_id: QueryId,
    dataset: &DatasetRef,
    query: &SqlQuery,
    chart_config: Option<&ChartConfig>,
) -> Result<WorkflowResult, AnalyticsError>
where
    S: SchemaLoader,
    E: QueryExecutor,
{
    // 1. Validate inputs (pure)
    validate_workflow_inputs(dataset, query, chart_config)?;

    // 2. Load schema (async effect boundary)
    let schema = schema_loader.load_schema(dataset).await?;

    // 3. Validate schema compatibility (pure)
    if let Some(config) = chart_config {
        validate_schema_compatibility(&schema, config)?;
    }

    // 4. Execute query (async effect boundary)
    let result = query_executor.execute(dataset, query).await?;

    // 5. Transform for chart (pure)
    let chart_data = match chart_config {
        Some(config) => Some(transform_for_chart(&result, config)?),
        None => None,
    };

    Ok(WorkflowResult {
        query_id,
        execution_time_ms: result.execution_time_ms,
        row_count: result.row_count,
        chart_data,
        raw_result: result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod validate_workflow_inputs {
        use super::*;

        #[test]
        fn accepts_valid_inputs_without_chart() {
            let dataset = DatasetRef::new("./test.csv").unwrap();
            let query = SqlQuery::new("SELECT * FROM test").unwrap();
            assert!(validate_workflow_inputs(&dataset, &query, None).is_ok());
        }

        #[test]
        fn accepts_valid_inputs_with_valid_chart() {
            let dataset = DatasetRef::new("./test.csv").unwrap();
            let query = SqlQuery::new("SELECT * FROM test").unwrap();
            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("date")
                .with_y_axis("value");
            assert!(validate_workflow_inputs(&dataset, &query, Some(&config)).is_ok());
        }

        #[test]
        fn rejects_invalid_chart_config() {
            let dataset = DatasetRef::new("./test.csv").unwrap();
            let query = SqlQuery::new("SELECT * FROM test").unwrap();
            // Line chart requires both axes
            let config = ChartConfig::new(ChartType::Line).with_x_axis("date");
            let result = validate_workflow_inputs(&dataset, &query, Some(&config));
            assert!(result.is_err());
        }
    }

    mod validate_schema_compatibility {
        use super::*;

        fn sample_schema() -> DatasetSchema {
            let mut columns = HashMap::new();
            columns.insert("date".to_string(), "DATE".to_string());
            columns.insert("value".to_string(), "DOUBLE".to_string());
            columns.insert("category".to_string(), "VARCHAR".to_string());
            DatasetSchema { columns }
        }

        #[test]
        fn accepts_matching_columns() {
            let schema = sample_schema();
            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("date")
                .with_y_axis("value");
            assert!(validate_schema_compatibility(&schema, &config).is_ok());
        }

        #[test]
        fn accepts_with_series_column() {
            let schema = sample_schema();
            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("date")
                .with_y_axis("value")
                .with_series_column("category");
            assert!(validate_schema_compatibility(&schema, &config).is_ok());
        }

        #[test]
        fn rejects_missing_x_axis_column() {
            let schema = sample_schema();
            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("nonexistent")
                .with_y_axis("value");
            let result = validate_schema_compatibility(&schema, &config);
            assert!(result.is_err());
        }

        #[test]
        fn rejects_missing_y_axis_column() {
            let schema = sample_schema();
            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("date")
                .with_y_axis("nonexistent");
            let result = validate_schema_compatibility(&schema, &config);
            assert!(result.is_err());
        }

        #[test]
        fn rejects_missing_series_column() {
            let schema = sample_schema();
            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("date")
                .with_y_axis("value")
                .with_series_column("nonexistent");
            let result = validate_schema_compatibility(&schema, &config);
            assert!(result.is_err());
        }
    }

    mod transform_for_chart {
        use super::*;
        use serde_json::json;

        fn sample_result() -> QueryResult {
            QueryResult {
                columns: vec!["date".to_string(), "value".to_string()],
                rows: vec![
                    vec![json!("2024-01-01"), json!(100)],
                    vec![json!("2024-01-02"), json!(150)],
                    vec![json!("2024-01-03"), json!(120)],
                ],
                row_count: 3,
                execution_time_ms: 42,
            }
        }

        #[test]
        fn extracts_x_and_y_values() {
            let result = sample_result();
            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("date")
                .with_y_axis("value");
            let chart_data = transform_for_chart(&result, &config).unwrap();

            assert_eq!(chart_data.chart_type, ChartType::Line);
            assert_eq!(chart_data.x_values.len(), 3);
            assert_eq!(chart_data.y_values.len(), 3);
            assert_eq!(chart_data.x_values[0], json!("2024-01-01"));
            assert_eq!(chart_data.y_values[0], json!(100));
        }

        #[test]
        fn uses_indices_when_no_x_axis() {
            let result = sample_result();
            let config = ChartConfig::new(ChartType::Bar).with_y_axis("value");
            let chart_data = transform_for_chart(&result, &config).unwrap();

            assert_eq!(chart_data.x_values[0], json!(0));
            assert_eq!(chart_data.x_values[1], json!(1));
            assert_eq!(chart_data.x_values[2], json!(2));
        }

        #[test]
        fn applies_limit() {
            let result = sample_result();
            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("date")
                .with_y_axis("value")
                .with_limit(2);
            let chart_data = transform_for_chart(&result, &config).unwrap();

            assert_eq!(chart_data.x_values.len(), 2);
            assert_eq!(chart_data.y_values.len(), 2);
        }

        #[test]
        fn includes_title_when_configured() {
            let result = sample_result();
            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("date")
                .with_y_axis("value")
                .with_title("My Chart");
            let chart_data = transform_for_chart(&result, &config).unwrap();

            assert_eq!(chart_data.title, Some("My Chart".to_string()));
        }
    }
}
