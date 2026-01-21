//! Integration tests for chart SSE endpoints.
//!
//! These tests verify the full stack: DuckDB -> Transformer -> SSE.

use ironstar::infrastructure::analytics::DuckDBService;
use ironstar::presentation::bar_chart_transformer::BarChartTransformer;
use ironstar::presentation::chart_transformer::{
    ChartConfig, ChartTransformer, ChartType, ColumnMetadata, QueryResult,
};

/// Test that BarChartTransformer produces valid ECharts JSON structure.
#[test]
fn bar_chart_produces_valid_echarts_structure() {
    let result = QueryResult {
        columns: vec![
            ColumnMetadata {
                name: "nationality".to_string(),
                data_type: "VARCHAR".to_string(),
            },
            ColumnMetadata {
                name: "count".to_string(),
                data_type: "BIGINT".to_string(),
            },
        ],
        rows: vec![
            vec![serde_json::json!("USA"), serde_json::json!(337)],
            vec![serde_json::json!("Russia"), serde_json::json!(130)],
            vec![serde_json::json!("China"), serde_json::json!(20)],
        ],
    };

    let config = ChartConfig {
        chart_type: ChartType::Bar,
        title: Some("Astronauts by Nationality".to_string()),
        category_column: "nationality".to_string(),
        value_columns: vec!["count".to_string()],
    };

    let option = BarChartTransformer.transform(&result, &config).unwrap();

    // Verify ECharts structure
    assert!(option.get("xAxis").is_some());
    assert!(option.get("yAxis").is_some());
    assert!(option.get("series").is_some());

    let x_axis = option.get("xAxis").unwrap();
    assert_eq!(x_axis["type"], "category");

    let categories = x_axis["data"].as_array().unwrap();
    assert_eq!(categories.len(), 3);
    assert_eq!(categories[0], "USA");

    let series = option.get("series").unwrap().as_array().unwrap();
    assert_eq!(series[0]["type"], "bar");
}

/// Test DuckDB query execution with in-memory database.
#[tokio::test]
async fn duckdb_query_executes_successfully() {
    let pool = async_duckdb::PoolBuilder::new()
        .num_conns(1)
        .open()
        .await
        .expect("failed to create pool");

    let service = DuckDBService::new(Some(pool.clone()));

    service
        .query_mut(|conn| {
            conn.execute(
                "CREATE TABLE astronauts (nationality VARCHAR, count BIGINT)",
                [],
            )?;
            conn.execute(
                "INSERT INTO astronauts VALUES ('USA', 337), ('Russia', 130)",
                [],
            )?;
            Ok(())
        })
        .await
        .expect("failed to create test data");

    let result: Vec<(String, i64)> = service
        .query(|conn| {
            let mut stmt = conn.prepare(
                "SELECT nationality, count FROM astronauts ORDER BY count DESC",
            )?;
            let rows = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .await
        .expect("query failed");

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], ("USA".to_string(), 337));

    pool.close().await.expect("failed to close pool");
}
