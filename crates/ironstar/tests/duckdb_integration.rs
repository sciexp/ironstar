//! DuckDB integration tests.
//!
//! These tests require network access to HuggingFace and are marked with `#[ignore]`.
//! Run with: `cargo test -p ironstar --ignored`

use ironstar::infrastructure::DuckDBService;

/// Test attaching the sciexp/fixtures DuckLake catalog and querying astronaut data.
///
/// This test verifies the full integration path:
/// 1. Create DuckDB pool
/// 2. Initialize httpfs and ducklake extensions
/// 3. Attach remote DuckLake catalog via hf:// protocol
/// 4. Execute query against attached catalog
/// 5. Verify expected data structure
#[tokio::test]
#[ignore = "requires network access to HuggingFace"]
async fn attach_and_query_ducklake_catalog() {
    // Create an in-memory DuckDB pool
    let pool = async_duckdb::PoolBuilder::new()
        .num_conns(1)
        .open()
        .await
        .expect("failed to create DuckDB pool");

    let service = DuckDBService::new(Some(pool.clone()));

    // Initialize required extensions
    service
        .initialize_extensions()
        .await
        .expect("failed to initialize extensions");

    // Attach the sciexp/fixtures DuckLake catalog
    let catalog_uri = "ducklake:hf://datasets/sciexp/fixtures/lakes/frozen/space.db";
    service
        .attach_catalog("space", catalog_uri)
        .await
        .expect("failed to attach DuckLake catalog");

    // Query astronaut data from the attached catalog
    let astronauts: Vec<(String, String, i64, i64)> = service
        .query(|conn| {
            let mut stmt = conn.prepare(
                "SELECT name, nationality, total_space_days, total_flights \
                 FROM space.main.astronauts \
                 ORDER BY total_space_days DESC \
                 LIMIT 10",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .await
        .expect("failed to query astronauts");

    // Verify we got results
    assert!(
        !astronauts.is_empty(),
        "expected astronaut data from DuckLake catalog"
    );

    // Verify data structure (first result should have non-empty name and nationality)
    let (name, nationality, space_days, flights) = &astronauts[0];
    assert!(!name.is_empty(), "astronaut name should not be empty");
    assert!(
        !nationality.is_empty(),
        "astronaut nationality should not be empty"
    );
    assert!(*space_days > 0, "top astronaut should have positive space days");
    assert!(*flights > 0, "top astronaut should have at least one flight");

    // Log the top astronaut for manual verification
    eprintln!(
        "Top astronaut: {} ({}) - {} days in space across {} flights",
        name, nationality, space_days, flights
    );

    pool.close().await.expect("failed to close pool");
}

/// Test that tables from attached catalog are visible.
#[tokio::test]
#[ignore = "requires network access to HuggingFace"]
async fn attached_catalog_tables_visible() {
    let pool = async_duckdb::PoolBuilder::new()
        .num_conns(1)
        .open()
        .await
        .expect("failed to create DuckDB pool");

    let service = DuckDBService::new(Some(pool.clone()));

    service
        .initialize_extensions()
        .await
        .expect("failed to initialize extensions");

    let catalog_uri = "ducklake:hf://datasets/sciexp/fixtures/lakes/frozen/space.db";
    service
        .attach_catalog("space", catalog_uri)
        .await
        .expect("failed to attach DuckLake catalog");

    // Query available tables in the space catalog
    let tables: Vec<String> = service
        .query(|conn| {
            let mut stmt = conn.prepare(
                "SELECT table_name FROM information_schema.tables \
                 WHERE table_catalog = 'space' AND table_schema = 'main'",
            )?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .await
        .expect("failed to query tables");

    // Verify expected tables exist
    let expected_tables = ["astronauts", "missions", "mission_crew", "spacecraft"];
    for expected in expected_tables {
        assert!(
            tables.iter().any(|t| t == expected),
            "expected table '{}' in catalog, found: {:?}",
            expected,
            tables
        );
    }

    pool.close().await.expect("failed to close pool");
}
