//! Bar chart transformer for DuckDB query results.
//!
//! Transforms tabular DuckDB results into ECharts bar chart configuration.
//! This is a presentation concern: mapping query results to chart-specific JSON.

use crate::presentation::chart_transformer::{
    ChartConfig, ChartTransformer, ChartType, QueryResult, TransformError,
};

/// Transforms query results into ECharts bar chart configuration.
///
/// Expects `QueryResult` with:
/// - One category column (for X-axis labels)
/// - One or more value columns (for bar heights)
///
/// Produces ECharts option with:
/// - `xAxis`: category type with data from category column
/// - `yAxis`: value type
/// - `series`: one bar series per value column
///
/// # Example
///
/// ```rust,ignore
/// use ironstar::presentation::{
///     BarChartTransformer, ChartConfig, ChartTransformer, ChartType,
///     ColumnMetadata, QueryResult,
/// };
/// use serde_json::json;
///
/// let result = QueryResult::new(
///     vec![
///         ColumnMetadata { name: "nationality".into(), data_type: "VARCHAR".into() },
///         ColumnMetadata { name: "count".into(), data_type: "BIGINT".into() },
///     ],
///     vec![
///         vec![json!("USA"), json!(123)],
///         vec![json!("Russia"), json!(72)],
///     ],
/// );
///
/// let config = ChartConfig {
///     chart_type: ChartType::Bar,
///     title: Some("Astronauts by Nationality".into()),
///     category_column: "nationality".into(),
///     value_columns: vec!["count".into()],
/// };
///
/// let transformer = BarChartTransformer;
/// let echarts_option = transformer.transform(&result, &config).unwrap();
/// ```
pub struct BarChartTransformer;

impl ChartTransformer for BarChartTransformer {
    fn transform(
        &self,
        result: &QueryResult,
        config: &ChartConfig,
    ) -> Result<serde_json::Value, TransformError> {
        // Validate chart type
        if config.chart_type != ChartType::Bar {
            return Err(TransformError::TransformFailed(format!(
                "BarChartTransformer requires ChartType::Bar, got {:?}",
                config.chart_type
            )));
        }

        // Validate non-empty result
        if result.rows.is_empty() {
            return Err(TransformError::EmptyResult);
        }

        // Find category column index
        let category_idx = result
            .column_index(&config.category_column)
            .ok_or_else(|| TransformError::MissingColumn(config.category_column.clone()))?;

        // Find value column indices
        let value_indices: Vec<usize> = config
            .value_columns
            .iter()
            .map(|name| {
                result
                    .column_index(name)
                    .ok_or_else(|| TransformError::MissingColumn(name.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Extract category labels (X-axis)
        let categories: Vec<String> = result
            .rows
            .iter()
            .map(|row| {
                row.get(category_idx)
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default()
            })
            .collect();

        // Build series (one per value column)
        let series: Vec<serde_json::Value> = config
            .value_columns
            .iter()
            .zip(value_indices.iter())
            .map(|(name, &idx)| {
                let data: Vec<serde_json::Value> = result
                    .rows
                    .iter()
                    .map(|row| row.get(idx).cloned().unwrap_or(serde_json::Value::Null))
                    .collect();

                serde_json::json!({
                    "name": name,
                    "type": "bar",
                    "data": data
                })
            })
            .collect();

        // Build complete ECharts option
        let mut option = serde_json::json!({
            "xAxis": {
                "type": "category",
                "data": categories
            },
            "yAxis": {
                "type": "value"
            },
            "series": series,
            "tooltip": {
                "trigger": "axis"
            }
        });

        // Add title if provided
        if let Some(title) = &config.title
            && let Some(obj) = option.as_object_mut()
        {
            obj.insert(
                "title".to_string(),
                serde_json::json!({
                    "text": title
                }),
            );
        }

        Ok(option)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::chart_transformer::ColumnMetadata;
    use serde_json::json;

    fn astronaut_result() -> QueryResult {
        QueryResult::new(
            vec![
                ColumnMetadata {
                    name: "nationality".into(),
                    data_type: "VARCHAR".into(),
                },
                ColumnMetadata {
                    name: "count".into(),
                    data_type: "BIGINT".into(),
                },
            ],
            vec![
                vec![json!("USA"), json!(123)],
                vec![json!("Russia"), json!(72)],
                vec![json!("China"), json!(18)],
            ],
        )
    }

    fn astronaut_config() -> ChartConfig {
        ChartConfig {
            chart_type: ChartType::Bar,
            title: Some("Astronauts by Nationality".into()),
            category_column: "nationality".into(),
            value_columns: vec!["count".into()],
        }
    }

    #[test]
    fn bar_chart_single_value_column() {
        let transformer = BarChartTransformer;
        let result = astronaut_result();
        let config = astronaut_config();

        let option = transformer.transform(&result, &config).unwrap();

        // Verify xAxis
        assert_eq!(option["xAxis"]["type"], "category");
        assert_eq!(option["xAxis"]["data"], json!(["USA", "Russia", "China"]));

        // Verify yAxis
        assert_eq!(option["yAxis"]["type"], "value");

        // Verify series
        let series = option["series"].as_array().unwrap();
        assert_eq!(series.len(), 1);
        assert_eq!(series[0]["name"], "count");
        assert_eq!(series[0]["type"], "bar");
        assert_eq!(series[0]["data"], json!([123, 72, 18]));

        // Verify title
        assert_eq!(option["title"]["text"], "Astronauts by Nationality");

        // Verify tooltip
        assert_eq!(option["tooltip"]["trigger"], "axis");
    }

    #[test]
    fn bar_chart_multiple_value_columns() {
        let transformer = BarChartTransformer;
        let result = QueryResult::new(
            vec![
                ColumnMetadata {
                    name: "year".into(),
                    data_type: "INTEGER".into(),
                },
                ColumnMetadata {
                    name: "sales".into(),
                    data_type: "DOUBLE".into(),
                },
                ColumnMetadata {
                    name: "revenue".into(),
                    data_type: "DOUBLE".into(),
                },
            ],
            vec![
                vec![json!(2022), json!(100.5), json!(150.0)],
                vec![json!(2023), json!(120.3), json!(180.5)],
                vec![json!(2024), json!(140.0), json!(210.2)],
            ],
        );

        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: None,
            category_column: "year".into(),
            value_columns: vec!["sales".into(), "revenue".into()],
        };

        let option = transformer.transform(&result, &config).unwrap();

        // Verify xAxis categories (numeric converted to string)
        assert_eq!(option["xAxis"]["data"], json!(["2022", "2023", "2024"]));

        // Verify two series
        let series = option["series"].as_array().unwrap();
        assert_eq!(series.len(), 2);

        assert_eq!(series[0]["name"], "sales");
        assert_eq!(series[0]["data"], json!([100.5, 120.3, 140.0]));

        assert_eq!(series[1]["name"], "revenue");
        assert_eq!(series[1]["data"], json!([150.0, 180.5, 210.2]));

        // Verify no title when not provided
        assert!(option.get("title").is_none());
    }

    #[test]
    fn bar_chart_missing_category_column() {
        let transformer = BarChartTransformer;
        let result = astronaut_result();
        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: None,
            category_column: "nonexistent".into(),
            value_columns: vec!["count".into()],
        };

        let err = transformer.transform(&result, &config).unwrap_err();
        assert!(
            matches!(&err, TransformError::MissingColumn(col) if col == "nonexistent"),
            "expected MissingColumn error, got {err:?}"
        );
    }

    #[test]
    fn bar_chart_missing_value_column() {
        let transformer = BarChartTransformer;
        let result = astronaut_result();
        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: None,
            category_column: "nationality".into(),
            value_columns: vec!["missing_column".into()],
        };

        let err = transformer.transform(&result, &config).unwrap_err();
        assert!(
            matches!(&err, TransformError::MissingColumn(col) if col == "missing_column"),
            "expected MissingColumn error, got {err:?}"
        );
    }

    #[test]
    fn bar_chart_empty_result() {
        let transformer = BarChartTransformer;
        let result = QueryResult::new(
            vec![
                ColumnMetadata {
                    name: "nationality".into(),
                    data_type: "VARCHAR".into(),
                },
                ColumnMetadata {
                    name: "count".into(),
                    data_type: "BIGINT".into(),
                },
            ],
            vec![], // empty rows
        );
        let config = astronaut_config();

        let err = transformer.transform(&result, &config).unwrap_err();
        assert!(
            matches!(err, TransformError::EmptyResult),
            "expected EmptyResult error, got {err:?}"
        );
    }

    #[test]
    fn bar_chart_wrong_chart_type() {
        let transformer = BarChartTransformer;
        let result = astronaut_result();
        let config = ChartConfig {
            chart_type: ChartType::Line, // wrong type
            title: None,
            category_column: "nationality".into(),
            value_columns: vec!["count".into()],
        };

        let err = transformer.transform(&result, &config).unwrap_err();
        assert!(
            matches!(
                &err,
                TransformError::TransformFailed(msg)
                if msg.contains("BarChartTransformer requires ChartType::Bar") && msg.contains("Line")
            ),
            "expected TransformFailed error with correct message, got {err:?}"
        );
    }

    #[test]
    fn bar_chart_echarts_structure_validity() {
        let transformer = BarChartTransformer;
        let result = astronaut_result();
        let config = astronaut_config();

        let option = transformer.transform(&result, &config).unwrap();

        // Verify the structure is valid ECharts JSON
        assert!(option.is_object());
        assert!(option.get("xAxis").is_some());
        assert!(option.get("yAxis").is_some());
        assert!(option.get("series").is_some());
        assert!(option.get("tooltip").is_some());

        // xAxis must have type and data
        let x_axis = &option["xAxis"];
        assert!(x_axis.get("type").is_some());
        assert!(x_axis.get("data").is_some());
        assert!(x_axis["data"].is_array());

        // yAxis must have type
        let y_axis = &option["yAxis"];
        assert!(y_axis.get("type").is_some());

        // series must be array with proper structure
        let series = option["series"].as_array().unwrap();
        for s in series {
            assert!(s.get("name").is_some());
            assert!(s.get("type").is_some());
            assert!(s.get("data").is_some());
            assert_eq!(s["type"], "bar");
        }
    }

    #[test]
    fn bar_chart_null_values_handled() {
        let transformer = BarChartTransformer;
        let result = QueryResult::new(
            vec![
                ColumnMetadata {
                    name: "category".into(),
                    data_type: "VARCHAR".into(),
                },
                ColumnMetadata {
                    name: "value".into(),
                    data_type: "INTEGER".into(),
                },
            ],
            vec![
                vec![json!("A"), json!(10)],
                vec![json!("B"), serde_json::Value::Null],
                vec![json!("C"), json!(30)],
            ],
        );

        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: None,
            category_column: "category".into(),
            value_columns: vec!["value".into()],
        };

        let option = transformer.transform(&result, &config).unwrap();

        // Null values should be preserved in the data array
        let series = option["series"].as_array().unwrap();
        assert_eq!(series[0]["data"], json!([10, null, 30]));
    }
}
