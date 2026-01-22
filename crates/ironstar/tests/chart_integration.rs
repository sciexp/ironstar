//! Integration tests for chart SSE endpoints.
//!
//! These tests verify the full stack: DuckDB -> Transformer -> SSE.

#![expect(
    clippy::expect_used,
    reason = "test file with standard test assertions"
)]
#![expect(clippy::print_stdout, reason = "test output for manual verification")]

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
            let mut stmt =
                conn.prepare("SELECT nationality, count FROM astronauts ORDER BY count DESC")?;
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

/// End-to-end test with real HuggingFace DuckLake data.
///
/// This test requires network access to HuggingFace and the DuckLake extensions.
/// Run with: `cargo test --test chart_integration --ignored`
///
/// # What it tests
///
/// 1. DuckDB httpfs and ducklake extensions load successfully
/// 2. The sciexp/fixtures DuckLake catalog attaches
/// 3. The space.main.astronauts table is queryable
/// 4. The BarChartTransformer produces valid ECharts JSON
#[tokio::test]
#[ignore = "requires network access to HuggingFace"]
async fn e2e_huggingface_astronauts_chart() {
    // Create pool and initialize extensions
    let pool = async_duckdb::PoolBuilder::new()
        .num_conns(1)
        .open()
        .await
        .expect("failed to create DuckDB pool");

    let service = DuckDBService::new(Some(pool.clone()));

    // Initialize extensions (httpfs, ducklake)
    service
        .initialize_extensions()
        .await
        .expect("failed to initialize DuckDB extensions");

    // Attach the DuckLake catalog
    let catalog_uri = "ducklake:hf://datasets/sciexp/fixtures/lakes/frozen/space.db";
    service
        .attach_catalog("space", catalog_uri)
        .await
        .expect("failed to attach DuckLake catalog");

    // Query astronauts table
    let result = service
        .query(|conn| {
            let mut stmt = conn.prepare(
                "SELECT nationality, COUNT(*) as count
                 FROM space.main.astronauts
                 GROUP BY nationality
                 ORDER BY count DESC
                 LIMIT 10",
            )?;

            let rows: Vec<Vec<serde_json::Value>> = stmt
                .query_map([], |row| {
                    Ok(vec![
                        serde_json::Value::String(row.get::<_, String>(0)?),
                        serde_json::Value::Number(row.get::<_, i64>(1)?.into()),
                    ])
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(QueryResult::new(
                vec![
                    ColumnMetadata {
                        name: "nationality".to_string(),
                        data_type: "VARCHAR".to_string(),
                    },
                    ColumnMetadata {
                        name: "count".to_string(),
                        data_type: "BIGINT".to_string(),
                    },
                ],
                rows,
            ))
        })
        .await
        .expect("query failed");

    // Verify we got data
    assert!(
        !result.rows.is_empty(),
        "Expected astronauts data from HuggingFace"
    );
    println!(
        "Fetched {} nationalities from HuggingFace DuckLake",
        result.row_count()
    );

    // Transform to chart
    let config = ChartConfig {
        chart_type: ChartType::Bar,
        title: Some("Astronauts by Nationality (Real Data)".to_string()),
        category_column: "nationality".to_string(),
        value_columns: vec!["count".to_string()],
    };

    let option = BarChartTransformer
        .transform(&result, &config)
        .expect("transformation failed");

    // Verify ECharts structure
    assert!(option.get("xAxis").is_some());
    assert!(option.get("series").is_some());

    let categories = option["xAxis"]["data"].as_array().unwrap();
    assert!(!categories.is_empty(), "Expected nationality categories");

    // USA should be in the top 10
    let has_usa = categories.iter().any(|v| v == "USA");
    assert!(
        has_usa,
        "Expected USA in top nationalities: {:?}",
        categories
    );

    println!(
        "E2E test passed: {} nationalities, USA present",
        categories.len()
    );

    pool.close().await.expect("failed to close pool");
}
