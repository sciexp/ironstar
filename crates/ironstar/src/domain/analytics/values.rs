//! Analytics domain value objects with smart constructors.
//!
//! This module contains validated value objects for the analytics domain,
//! following the "parse, don't validate" principle. These types form the
//! vocabulary for the QuerySession aggregate and analytics workflows.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::errors::AnalyticsValidationError;

/// Maximum length for SQL query strings in characters.
pub const SQL_QUERY_MAX_LENGTH: usize = 10_000;

/// Maximum length for dataset reference strings in characters.
pub const DATASET_REF_MAX_LENGTH: usize = 1_000;

// ============================================================================
// QueryId - Unique identifier for analytics queries
// ============================================================================

/// Unique identifier for an analytics query.
///
/// Wraps a UUID v4, providing type safety to prevent mixing up different
/// ID types (e.g., passing a `SessionId` where a `QueryId` is expected).
///
/// # Construction
///
/// - `QueryId::new()` - Generate a new random ID
/// - `QueryId::from_uuid(uuid)` - Wrap an existing UUID (for deserialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct QueryId(Uuid);

impl QueryId {
    /// Generate a new random QueryId.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID as a QueryId.
    ///
    /// Use this when deserializing from storage or parsing from input.
    /// For new queries, prefer `QueryId::new()`.
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Extract the inner UUID.
    #[must_use]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl Default for QueryId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for QueryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// DatasetRef - Reference to a data source
// ============================================================================

/// Validated reference to a dataset.
///
/// Supports multiple URI schemes for different data sources:
/// - `hf://datasets/user/repo` - HuggingFace Hub dataset
/// - `s3://bucket/path` - S3-compatible object storage
/// - `gs://bucket/path` - Google Cloud Storage
/// - `file:///path/to/data` - Local file path (absolute)
/// - `./relative/path` - Local relative path (development)
///
/// The reference is validated at construction time to ensure it's non-empty,
/// within length limits, and follows a recognized format.
///
/// # Example
///
/// ```rust,ignore
/// let hf_dataset = DatasetRef::new("hf://datasets/user/repo")?;
/// let s3_dataset = DatasetRef::new("s3://my-bucket/data/file.parquet")?;
/// let local_file = DatasetRef::new("./fixtures/sample.csv")?;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(try_from = "String", into = "String")]
pub struct DatasetRef(String);

impl DatasetRef {
    /// Recognized URI scheme prefixes for dataset references.
    const VALID_PREFIXES: &'static [&'static str] = &[
        "hf://",     // HuggingFace Hub
        "s3://",     // S3-compatible storage
        "gs://",     // Google Cloud Storage
        "az://",     // Azure Blob Storage
        "file://",   // Local absolute path (file URI)
        "./",        // Local relative path
        "../",       // Local relative path (parent)
        "/",         // Local absolute path (Unix)
    ];

    /// Create a new DatasetRef, validating the input.
    ///
    /// # Errors
    ///
    /// - [`AnalyticsValidationError::EmptyDatasetRef`] if empty after trimming
    /// - [`AnalyticsValidationError::DatasetRefTooLong`] if exceeds max length
    /// - [`AnalyticsValidationError::InvalidDatasetRefFormat`] if format unrecognized
    pub fn new(reference: impl Into<String>) -> Result<Self, AnalyticsValidationError> {
        let reference = reference.into();
        let trimmed = reference.trim();

        if trimmed.is_empty() {
            return Err(AnalyticsValidationError::EmptyDatasetRef);
        }

        let char_count = trimmed.chars().count();
        if char_count > DATASET_REF_MAX_LENGTH {
            return Err(AnalyticsValidationError::DatasetRefTooLong {
                max: DATASET_REF_MAX_LENGTH,
                actual: char_count,
            });
        }

        // Validate that the reference starts with a recognized prefix
        let has_valid_prefix = Self::VALID_PREFIXES
            .iter()
            .any(|prefix| trimmed.starts_with(prefix));

        if !has_valid_prefix {
            return Err(AnalyticsValidationError::InvalidDatasetRefFormat {
                reason: "must start with hf://, s3://, gs://, az://, file://, or a path",
            });
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Get the reference as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume self and return the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Check if this is a HuggingFace Hub reference.
    #[must_use]
    pub fn is_huggingface(&self) -> bool {
        self.0.starts_with("hf://")
    }

    /// Check if this is an S3 reference.
    #[must_use]
    pub fn is_s3(&self) -> bool {
        self.0.starts_with("s3://")
    }

    /// Check if this is a local file reference.
    #[must_use]
    pub fn is_local(&self) -> bool {
        self.0.starts_with("./")
            || self.0.starts_with("../")
            || self.0.starts_with('/')
            || self.0.starts_with("file://")
    }
}

impl std::fmt::Display for DatasetRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for DatasetRef {
    type Error = AnalyticsValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<DatasetRef> for String {
    fn from(reference: DatasetRef) -> Self {
        reference.0
    }
}

// ============================================================================
// SqlQuery - Validated SQL query string
// ============================================================================

/// Validated SQL query string.
///
/// Guarantees:
/// - Non-empty (at least one non-whitespace character)
/// - At most [`SQL_QUERY_MAX_LENGTH`] characters
/// - Trimmed of leading/trailing whitespace
///
/// Note: This performs basic validation only. Full SQL parsing and
/// authorization checks happen at the DuckDB execution layer.
///
/// # Example
///
/// ```rust,ignore
/// let query = SqlQuery::new("SELECT * FROM dataset LIMIT 10")?;
/// assert!(!query.as_str().is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(try_from = "String", into = "String")]
pub struct SqlQuery(String);

impl SqlQuery {
    /// Create a new SqlQuery, validating and normalizing the input.
    ///
    /// # Errors
    ///
    /// - [`AnalyticsValidationError::EmptySql`] if the trimmed query is empty
    /// - [`AnalyticsValidationError::SqlTooLong`] if the query exceeds max length
    pub fn new(sql: impl Into<String>) -> Result<Self, AnalyticsValidationError> {
        let sql = sql.into();
        let trimmed = sql.trim();

        if trimmed.is_empty() {
            return Err(AnalyticsValidationError::EmptySql);
        }

        let char_count = trimmed.chars().count();
        if char_count > SQL_QUERY_MAX_LENGTH {
            return Err(AnalyticsValidationError::SqlTooLong {
                max: SQL_QUERY_MAX_LENGTH,
                actual: char_count,
            });
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Get the SQL query as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume self and return the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for SqlQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for SqlQuery {
    type Error = AnalyticsValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SqlQuery> for String {
    fn from(query: SqlQuery) -> Self {
        query.0
    }
}

// ============================================================================
// ChartType - Type of chart visualization
// ============================================================================

/// Type of chart visualization.
///
/// Maps to Apache ECharts chart types. This is a sum type (enum) representing
/// the discrete choices available for chart visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    /// Line chart for time series or continuous data.
    #[default]
    Line,
    /// Bar chart for categorical comparisons.
    Bar,
    /// Scatter plot for correlation analysis.
    Scatter,
    /// Pie chart for part-to-whole relationships.
    Pie,
    /// Area chart (filled line chart).
    Area,
    /// Heatmap for matrix/grid data.
    Heatmap,
    /// Box plot for statistical distributions.
    Boxplot,
    /// Candlestick for financial data (OHLC).
    Candlestick,
}

impl ChartType {
    /// Get the ECharts series type name.
    #[must_use]
    pub fn echarts_type(&self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Bar => "bar",
            Self::Scatter => "scatter",
            Self::Pie => "pie",
            Self::Area => "line", // Area is line with areaStyle
            Self::Heatmap => "heatmap",
            Self::Boxplot => "boxplot",
            Self::Candlestick => "candlestick",
        }
    }

    /// Check if this chart type requires areaStyle in ECharts.
    #[must_use]
    pub fn requires_area_style(&self) -> bool {
        matches!(self, Self::Area)
    }
}

impl std::fmt::Display for ChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.echarts_type())
    }
}

// ============================================================================
// ChartConfig - Configuration for chart visualization
// ============================================================================

/// Configuration for ECharts visualization.
///
/// This is a product type containing all configuration options for rendering
/// a chart. Optional fields use `Option` to allow partial configuration
/// with sensible defaults applied at render time.
///
/// # Example
///
/// ```rust,ignore
/// let config = ChartConfig::new(ChartType::Line)
///     .with_x_axis("date")
///     .with_y_axis("value")
///     .with_title("Sales Over Time");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub struct ChartConfig {
    /// Type of chart to render.
    pub chart_type: ChartType,

    /// Column name for x-axis data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis: Option<String>,

    /// Column name for y-axis data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis: Option<String>,

    /// Chart title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Column name for series grouping (multiple lines/bars).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_column: Option<String>,

    /// Maximum number of data points to display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Whether to enable data zoom (scrollable x-axis).
    #[serde(default)]
    pub enable_zoom: bool,

    /// Whether to show legend.
    #[serde(default = "default_true")]
    pub show_legend: bool,
}

fn default_true() -> bool {
    true
}

impl ChartConfig {
    /// Create a new ChartConfig with the specified chart type.
    #[must_use]
    pub fn new(chart_type: ChartType) -> Self {
        Self {
            chart_type,
            show_legend: true,
            ..Default::default()
        }
    }

    /// Set the x-axis column.
    #[must_use]
    pub fn with_x_axis(mut self, column: impl Into<String>) -> Self {
        self.x_axis = Some(column.into());
        self
    }

    /// Set the y-axis column.
    #[must_use]
    pub fn with_y_axis(mut self, column: impl Into<String>) -> Self {
        self.y_axis = Some(column.into());
        self
    }

    /// Set the chart title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the series grouping column.
    #[must_use]
    pub fn with_series_column(mut self, column: impl Into<String>) -> Self {
        self.series_column = Some(column.into());
        self
    }

    /// Set the data point limit.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Enable data zoom.
    #[must_use]
    pub fn with_zoom(mut self) -> Self {
        self.enable_zoom = true;
        self
    }

    /// Hide the legend.
    #[must_use]
    pub fn without_legend(mut self) -> Self {
        self.show_legend = false;
        self
    }

    /// Validate the configuration is complete enough for rendering.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing for the chart type.
    pub fn validate(&self) -> Result<(), AnalyticsValidationError> {
        match self.chart_type {
            ChartType::Pie => {
                // Pie charts need a value column (y_axis used for this)
                if self.y_axis.is_none() {
                    return Err(AnalyticsValidationError::InvalidChartConfig {
                        reason: "pie chart requires y_axis (value column)",
                    });
                }
            }
            ChartType::Line | ChartType::Bar | ChartType::Area | ChartType::Scatter => {
                // These need both axes for meaningful display
                if self.x_axis.is_none() || self.y_axis.is_none() {
                    return Err(AnalyticsValidationError::InvalidChartConfig {
                        reason: "chart requires both x_axis and y_axis",
                    });
                }
            }
            ChartType::Heatmap => {
                // Heatmap needs x, y, and implicitly a value
                if self.x_axis.is_none() || self.y_axis.is_none() {
                    return Err(AnalyticsValidationError::InvalidChartConfig {
                        reason: "heatmap requires x_axis and y_axis",
                    });
                }
            }
            ChartType::Boxplot | ChartType::Candlestick => {
                // These have specialized data requirements
                // For now, just require x_axis
                if self.x_axis.is_none() {
                    return Err(AnalyticsValidationError::InvalidChartConfig {
                        reason: "chart requires x_axis",
                    });
                }
            }
        }
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod query_id {
        use super::*;

        #[test]
        fn new_generates_unique_ids() {
            let id1 = QueryId::new();
            let id2 = QueryId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn from_uuid_roundtrips() {
            let uuid = Uuid::new_v4();
            let id = QueryId::from_uuid(uuid);
            assert_eq!(id.into_inner(), uuid);
        }

        #[test]
        fn serializes_as_string() {
            let id = QueryId::from_uuid(Uuid::nil());
            let json = serde_json::to_string(&id).unwrap();
            assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
        }

        #[test]
        fn deserializes_from_string() {
            let json = "\"550e8400-e29b-41d4-a716-446655440000\"";
            let id: QueryId = serde_json::from_str(json).unwrap();
            assert_eq!(
                id.into_inner(),
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
            );
        }
    }

    mod dataset_ref {
        use super::*;

        #[test]
        fn accepts_huggingface_uri() {
            let dataset = DatasetRef::new("hf://datasets/user/repo").unwrap();
            assert_eq!(dataset.as_str(), "hf://datasets/user/repo");
            assert!(dataset.is_huggingface());
            assert!(!dataset.is_s3());
            assert!(!dataset.is_local());
        }

        #[test]
        fn accepts_s3_uri() {
            let dataset = DatasetRef::new("s3://bucket/path/to/data.parquet").unwrap();
            assert!(dataset.is_s3());
            assert!(!dataset.is_huggingface());
        }

        #[test]
        fn accepts_local_relative_path() {
            let dataset = DatasetRef::new("./fixtures/data.csv").unwrap();
            assert!(dataset.is_local());
            assert!(!dataset.is_huggingface());
        }

        #[test]
        fn accepts_local_absolute_path() {
            let dataset = DatasetRef::new("/var/data/dataset.parquet").unwrap();
            assert!(dataset.is_local());
        }

        #[test]
        fn accepts_file_uri() {
            let dataset = DatasetRef::new("file:///home/user/data.csv").unwrap();
            assert!(dataset.is_local());
        }

        #[test]
        fn trims_whitespace() {
            let dataset = DatasetRef::new("  hf://datasets/user/repo  ").unwrap();
            assert_eq!(dataset.as_str(), "hf://datasets/user/repo");
        }

        #[test]
        fn rejects_empty_string() {
            let result = DatasetRef::new("");
            assert_eq!(result, Err(AnalyticsValidationError::EmptyDatasetRef));
        }

        #[test]
        fn rejects_whitespace_only() {
            let result = DatasetRef::new("   \t\n  ");
            assert_eq!(result, Err(AnalyticsValidationError::EmptyDatasetRef));
        }

        #[test]
        fn rejects_invalid_prefix() {
            let result = DatasetRef::new("http://example.com/data");
            assert!(matches!(
                result,
                Err(AnalyticsValidationError::InvalidDatasetRefFormat { .. })
            ));
        }

        #[test]
        fn rejects_too_long() {
            let long_path = format!("hf://{}", "a".repeat(DATASET_REF_MAX_LENGTH));
            let result = DatasetRef::new(&long_path);
            assert!(matches!(
                result,
                Err(AnalyticsValidationError::DatasetRefTooLong { .. })
            ));
        }

        #[test]
        fn serde_roundtrip() {
            let original = DatasetRef::new("s3://bucket/data").unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: DatasetRef = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn serde_rejects_invalid() {
            let json = r#""http://invalid.com/data""#;
            let result: Result<DatasetRef, _> = serde_json::from_str(json);
            assert!(result.is_err());
        }
    }

    mod sql_query {
        use super::*;

        #[test]
        fn accepts_valid_sql() {
            let query = SqlQuery::new("SELECT * FROM dataset").unwrap();
            assert_eq!(query.as_str(), "SELECT * FROM dataset");
        }

        #[test]
        fn trims_whitespace() {
            let query = SqlQuery::new("  SELECT * FROM t  ").unwrap();
            assert_eq!(query.as_str(), "SELECT * FROM t");
        }

        #[test]
        fn rejects_empty_string() {
            let result = SqlQuery::new("");
            assert_eq!(result, Err(AnalyticsValidationError::EmptySql));
        }

        #[test]
        fn rejects_whitespace_only() {
            let result = SqlQuery::new("   \t\n  ");
            assert_eq!(result, Err(AnalyticsValidationError::EmptySql));
        }

        #[test]
        fn rejects_too_long() {
            let long_sql = "SELECT ".to_string() + &"a".repeat(SQL_QUERY_MAX_LENGTH);
            let result = SqlQuery::new(&long_sql);
            assert!(matches!(
                result,
                Err(AnalyticsValidationError::SqlTooLong { .. })
            ));
        }

        #[test]
        fn accepts_max_length() {
            let max_sql = "a".repeat(SQL_QUERY_MAX_LENGTH);
            let result = SqlQuery::new(&max_sql);
            assert!(result.is_ok());
        }

        #[test]
        fn serde_roundtrip() {
            let original = SqlQuery::new("SELECT 1").unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: SqlQuery = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn serde_rejects_empty() {
            let json = r#""""#;
            let result: Result<SqlQuery, _> = serde_json::from_str(json);
            assert!(result.is_err());
        }
    }

    mod chart_type {
        use super::*;

        #[test]
        fn echarts_type_mapping() {
            assert_eq!(ChartType::Line.echarts_type(), "line");
            assert_eq!(ChartType::Bar.echarts_type(), "bar");
            assert_eq!(ChartType::Scatter.echarts_type(), "scatter");
            assert_eq!(ChartType::Pie.echarts_type(), "pie");
            assert_eq!(ChartType::Area.echarts_type(), "line");
            assert_eq!(ChartType::Heatmap.echarts_type(), "heatmap");
        }

        #[test]
        fn area_requires_area_style() {
            assert!(ChartType::Area.requires_area_style());
            assert!(!ChartType::Line.requires_area_style());
        }

        #[test]
        fn default_is_line() {
            assert_eq!(ChartType::default(), ChartType::Line);
        }

        #[test]
        fn serializes_lowercase() {
            let json = serde_json::to_string(&ChartType::Bar).unwrap();
            assert_eq!(json, "\"bar\"");
        }

        #[test]
        fn deserializes_lowercase() {
            let chart: ChartType = serde_json::from_str("\"scatter\"").unwrap();
            assert_eq!(chart, ChartType::Scatter);
        }
    }

    mod chart_config {
        use super::*;

        #[test]
        fn builder_pattern() {
            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("date")
                .with_y_axis("value")
                .with_title("My Chart")
                .with_limit(100)
                .with_zoom();

            assert_eq!(config.chart_type, ChartType::Line);
            assert_eq!(config.x_axis, Some("date".to_string()));
            assert_eq!(config.y_axis, Some("value".to_string()));
            assert_eq!(config.title, Some("My Chart".to_string()));
            assert_eq!(config.limit, Some(100));
            assert!(config.enable_zoom);
            assert!(config.show_legend); // default true
        }

        #[test]
        fn validate_line_chart_requires_both_axes() {
            let config = ChartConfig::new(ChartType::Line).with_x_axis("date");
            assert!(config.validate().is_err());

            let config = ChartConfig::new(ChartType::Line)
                .with_x_axis("date")
                .with_y_axis("value");
            assert!(config.validate().is_ok());
        }

        #[test]
        fn validate_pie_chart_requires_y_axis() {
            let config = ChartConfig::new(ChartType::Pie);
            assert!(config.validate().is_err());

            let config = ChartConfig::new(ChartType::Pie).with_y_axis("count");
            assert!(config.validate().is_ok());
        }

        #[test]
        fn serde_skips_none_fields() {
            let config = ChartConfig::new(ChartType::Bar);
            let json = serde_json::to_string(&config).unwrap();
            // Should not contain "x_axis" since it's None
            assert!(!json.contains("x_axis"));
            // Should contain chart_type
            assert!(json.contains("\"chart_type\":\"bar\""));
        }

        #[test]
        fn serde_roundtrip() {
            let original = ChartConfig::new(ChartType::Scatter)
                .with_x_axis("x")
                .with_y_axis("y")
                .with_title("Test");

            let json = serde_json::to_string(&original).unwrap();
            let parsed: ChartConfig = serde_json::from_str(&json).unwrap();

            assert_eq!(original, parsed);
        }
    }
}
