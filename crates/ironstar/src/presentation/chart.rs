//! Chart presentation handlers.
//!
//! Provides SSE endpoints for streaming chart data from DuckDB analytics.
//! Charts are rendered as HTML fragments containing ds-echarts web components
//! with Datastar signal bindings, or as PatchSignals for direct signal updates.
//!
//! # Architecture
//!
//! Chart handlers follow the Datastar SSE pattern:
//! 1. Query DuckDB for analytics data
//! 2. Transform results using chart-specific transformers
//! 3. Render chart template with embedded signals
//! 4. Stream HTML fragment via SSE `datastar-patch-elements` event
//!
//! The chart feed endpoint uses PatchSignals to stream ChartSignals directly,
//! allowing clients to receive structured signal updates with keep-alive
//! support for proxy compatibility.
//!
//! # Routes
//!
//! - `GET /charts/astronauts` - Page with astronaut demographics chart
//! - `GET /charts/api/astronauts/data` - SSE endpoint streaming chart data
//! - `GET /charts/api/{chart_id}/feed` - SSE endpoint streaming ChartSignals via PatchSignals

use std::convert::Infallible;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    response::{Html, IntoResponse},
    routing::get,
};
use datastar::prelude::{PatchElements, PatchSignals};
use futures::stream::{self, Stream, StreamExt};
use hypertext::Renderable;
use tracing::{instrument, warn};

use crate::domain::signals::ChartSignals;
use crate::infrastructure::analytics::AnalyticsState;
use crate::infrastructure::assets::AssetManifest;
use crate::infrastructure::sse_stream::SseStreamBuilder;
use crate::infrastructure::{cache_key, embedded_cache_key_prefix};
use crate::presentation::bar_chart_transformer::BarChartTransformer;
use crate::presentation::chart_templates::echarts_chart;
use crate::presentation::chart_transformer::{
    ChartConfig, ChartTransformer, ChartType, ColumnMetadata, QueryResult,
};
use crate::state::AppState;

/// SSE endpoint for astronaut nationality chart data.
///
/// Queries `space.main.astronauts` table for nationality counts,
/// transforms to bar chart configuration, and streams the rendered
/// chart template via SSE.
///
/// # Route
///
/// GET /charts/api/astronauts/data
///
/// # SSE events
///
/// - `datastar-patch-elements`: HTML fragment containing ds-echarts component
///   with `data-signals` attribute embedding chart configuration
///
/// # Error handling
///
/// Query or transformation errors are communicated via the `error` signal field
/// rather than HTTP error codes, allowing the chart UI to display error state.
///
/// # SSE pattern
///
/// This endpoint uses `stream::once()` for one-shot delivery rather than
/// `SseStreamBuilder` with keep-alive. This is intentional:
///
/// - Chart data is computed once from a DuckDB query
/// - No real-time updates are needed for this chart
/// - The client receives the data and renders immediately
///
/// For charts requiring live updates (e.g., real-time metrics), use
/// `SseStreamBuilder` with Zenoh subscription for continuous streaming.
/// See `infrastructure/sse_stream.rs` for the streaming pattern.
#[instrument(name = "handler.chart.astronauts_sse", skip(analytics))]
pub async fn astronauts_chart_sse(
    State(analytics): State<AnalyticsState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let signals = astronauts_chart_signals(&analytics).await;

    // Render chart template with embedded signals
    let html = echarts_chart("astronauts-chart", &signals, "400px").render();

    // Create SSE event via SDK PatchElements (datastar-patch-elements event type)
    let event: Event = PatchElements::new(html.into_inner()).into();

    Sse::new(stream::once(async move { Ok(event) }))
}

/// Produce chart signals for the astronaut nationality chart.
///
/// Uses the cached analytics service when available to memoize the DuckDB
/// query result as JSON bytes.
/// On cache hit, the cached JSON is deserialized directly without querying DuckDB.
/// On cache miss, the query executes, the result is cached, and signals are returned.
async fn astronauts_chart_signals(analytics: &AnalyticsState) -> ChartSignals {
    let key = cache_key(
        &embedded_cache_key_prefix("space", "astronauts"),
        &"nationality_counts_top10",
    );

    // Try cached path first.
    if let Some(cached) = &analytics.cached
        && let Some(bytes) = cached.cache().get(&key).await
        && let Ok(chart_option) = serde_json::from_slice::<serde_json::Value>(&bytes)
    {
        return ChartSignals {
            chart_option,
            selected: None,
            loading: false,
            error: None,
        };
    }

    // Cache miss or no cache: execute query.
    let query_result = analytics
        .service
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
        .await;

    match query_result {
        Ok(result) => {
            let config = ChartConfig {
                chart_type: ChartType::Bar,
                title: Some("Astronauts by Nationality".to_string()),
                category_column: "nationality".to_string(),
                value_columns: vec!["count".to_string()],
            };

            match BarChartTransformer.transform(&result, &config) {
                Ok(chart_option) => {
                    // Cache the chart option as JSON bytes.
                    if let Some(cached) = &analytics.cached
                        && let Ok(bytes) = serde_json::to_vec(&chart_option)
                    {
                        cached.cache().insert(key, bytes).await;
                    }
                    ChartSignals {
                        chart_option,
                        selected: None,
                        loading: false,
                        error: None,
                    }
                }
                Err(e) => ChartSignals {
                    chart_option: serde_json::json!({}),
                    selected: None,
                    loading: false,
                    error: Some(format!("Transform error: {e}")),
                },
            }
        }
        Err(e) => ChartSignals {
            chart_option: serde_json::json!({}),
            selected: None,
            loading: false,
            error: Some(format!("Query error: {e}")),
        },
    }
}

/// Page handler for astronaut chart demo.
///
/// Renders the chart page shell which establishes an SSE connection
/// to `/api/charts/astronauts/data` on load.
///
/// # Route
///
/// GET /charts/astronauts
#[instrument(name = "handler.chart.astronauts_page", skip(manifest))]
pub async fn astronauts_chart_page(State(manifest): State<AssetManifest>) -> impl IntoResponse {
    use crate::presentation::chart_templates::chart_page;

    let html = chart_page(
        &manifest,
        "Astronaut Demographics",
        "astronauts",
        "/charts/api/astronauts/data",
    )
    .render();

    Html(html.into_inner())
}

/// SSE feed endpoint for chart signal updates via PatchSignals.
///
/// Streams `ChartSignals` as Datastar `PatchSignals` events for the
/// specified chart. The stream first sends a loading indicator, then
/// computes and sends the chart data, followed by keep-alive comments
/// for proxy compatibility.
///
/// # Route
///
/// GET /charts/api/{chart_id}/feed
///
/// # SSE events
///
/// - `datastar-patch-signals`: JSON-encoded `ChartSignals` with loading state
/// - `datastar-patch-signals`: JSON-encoded `ChartSignals` with chart data
/// - `: keepalive` comments at 15-second intervals
///
/// # Supported chart IDs
///
/// - `astronauts` - Astronaut demographics bar chart
///
/// Returns 404 for unrecognized chart IDs.
///
/// # Error handling
///
/// Query or transformation errors are communicated via the `error` field
/// in `ChartSignals` rather than HTTP error codes, allowing the chart UI
/// to display error state gracefully.
#[instrument(name = "handler.chart.feed", skip(analytics), fields(chart_id = %chart_id))]
pub async fn chart_feed_handler(
    Path(chart_id): Path<String>,
    State(analytics): State<AnalyticsState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>> + Send>, StatusCode> {
    match chart_id.as_str() {
        "astronauts" => {}
        _ => {
            warn!(chart_id = %chart_id, "Unknown chart_id requested for feed");
            return Err(StatusCode::NOT_FOUND);
        }
    }

    // Build a finite stream: loading signal, then computed chart data.
    let data_stream = stream::once(async move {
        // Send loading indicator first.
        let loading_signals = ChartSignals {
            chart_option: serde_json::json!({}),
            selected: None,
            loading: true,
            error: None,
        };
        let loading_json = serde_json::to_string(&loading_signals).unwrap_or_else(|e| {
            warn!(error = %e, "Failed to serialize loading ChartSignals");
            r#"{"loading":true}"#.to_string()
        });
        let loading_event: Event = PatchSignals::new(loading_json).into();

        loading_event
    })
    .chain(stream::once(async move {
        // Compute chart signals from DuckDB.
        let signals = astronauts_chart_signals(&analytics).await;
        let signals_json = serde_json::to_string(&signals).unwrap_or_else(|e| {
            warn!(error = %e, "Failed to serialize ChartSignals");
            r#"{"chartOption":{},"loading":false,"error":"Serialization error"}"#.to_string()
        });
        let data_event: Event = PatchSignals::new(signals_json).into();

        data_event
    }))
    .map(Ok);

    // Wrap with keep-alive for proxy compatibility. Use build_live_only
    // since the data stream is finite and there is no replay/live split.
    let builder = SseStreamBuilder::new().with_keep_alive_secs(15);
    let stream = builder.build_live_only(data_stream);

    Ok(Sse::new(stream))
}

/// Creates the Chart feature router with all endpoints.
///
/// # Routes (relative to /charts nest)
///
/// - `GET /astronauts` - Astronaut demographics chart page (HTML)
/// - `GET /api/astronauts/data` - Astronaut chart SSE endpoint
/// - `GET /api/{chart_id}/feed` - Chart signal feed SSE endpoint
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/astronauts", get(astronauts_chart_page))
        .route("/api/astronauts/data", get(astronauts_chart_sse))
        .route("/api/{chart_id}/feed", get(chart_feed_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Verify QueryResult construction logic produces valid structure.
    #[test]
    fn query_result_construction_matches_expected_format() {
        // Simulate the row format produced by the DuckDB query
        let rows: Vec<Vec<serde_json::Value>> = vec![
            vec![json!("USA"), json!(123)],
            vec![json!("Russia"), json!(72)],
            vec![json!("China"), json!(18)],
        ];

        let result = QueryResult::new(
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
        );

        // Verify structure
        assert_eq!(result.column_count(), 2);
        assert_eq!(result.row_count(), 3);
        assert_eq!(result.column_index("nationality"), Some(0));
        assert_eq!(result.column_index("count"), Some(1));

        // Verify first row data
        assert_eq!(result.rows[0][0], json!("USA"));
        assert_eq!(result.rows[0][1], json!(123));
    }

    /// Verify ChartConfig matches bar chart transformer requirements.
    #[test]
    fn chart_config_valid_for_bar_transformer() {
        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: Some("Astronauts by Nationality".to_string()),
            category_column: "nationality".to_string(),
            value_columns: vec!["count".to_string()],
        };

        assert_eq!(config.chart_type, ChartType::Bar);
        assert_eq!(config.category_column, "nationality");
        assert_eq!(config.value_columns.len(), 1);
    }

    /// Verify transformation produces valid ECharts option structure.
    #[test]
    fn transformation_produces_valid_echarts_option() {
        let result = QueryResult::new(
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
            vec![
                vec![json!("USA"), json!(123)],
                vec![json!("Russia"), json!(72)],
            ],
        );

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
        assert_eq!(option["xAxis"]["type"], "category");
        assert_eq!(option["xAxis"]["data"], json!(["USA", "Russia"]));
    }

    /// Verify ChartSignals construction for success case.
    #[test]
    fn chart_signals_success_case() {
        let chart_option = json!({
            "xAxis": {"type": "category", "data": ["A", "B"]},
            "yAxis": {"type": "value"},
            "series": [{"type": "bar", "data": [10, 20]}]
        });

        let signals = ChartSignals {
            chart_option: chart_option.clone(),
            selected: None,
            loading: false,
            error: None,
        };

        assert_eq!(signals.chart_option, chart_option);
        assert!(signals.error.is_none());
        assert!(!signals.loading);
    }

    /// Verify ChartSignals construction for error case.
    #[test]
    fn chart_signals_error_case() {
        let signals = ChartSignals {
            chart_option: json!({}),
            selected: None,
            loading: false,
            error: Some("Query error: analytics service unavailable".to_string()),
        };

        assert!(signals.error.is_some());
        assert!(signals.error.as_ref().unwrap().contains("Query error"));
        assert_eq!(signals.chart_option, json!({}));
    }

    /// Verify empty result handling.
    #[test]
    fn transformation_handles_empty_result() {
        let result = QueryResult::new(
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
            vec![], // Empty rows
        );

        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: None,
            category_column: "nationality".to_string(),
            value_columns: vec!["count".to_string()],
        };

        // BarChartTransformer should return EmptyResult error
        let err = BarChartTransformer.transform(&result, &config).unwrap_err();
        assert!(matches!(
            err,
            crate::presentation::chart_transformer::TransformError::EmptyResult
        ));
    }

    /// Verify chart_feed_handler returns 404 for unknown chart IDs.
    #[tokio::test]
    async fn chart_feed_returns_not_found_for_unknown_id() {
        use crate::infrastructure::analytics::{AnalyticsState, DuckDBService};

        let service = DuckDBService::new(None);
        let analytics = AnalyticsState::new(service);

        let result = chart_feed_handler(Path("nonexistent".to_string()), State(analytics)).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    /// Verify chart_feed_handler accepts "astronauts" as a valid chart ID.
    ///
    /// When analytics is unavailable, the handler still returns an SSE stream
    /// (with error signals) rather than an HTTP error, because query failures
    /// are communicated via ChartSignals.error.
    #[tokio::test]
    async fn chart_feed_accepts_astronauts_chart_id() {
        use crate::infrastructure::analytics::{AnalyticsState, DuckDBService};

        let service = DuckDBService::new(None);
        let analytics = AnalyticsState::new(service);

        let result = chart_feed_handler(Path("astronauts".to_string()), State(analytics)).await;

        // Should succeed (returns SSE stream) even with unavailable analytics.
        // Query errors surface in ChartSignals.error, not HTTP status.
        assert!(result.is_ok());
    }

    /// Verify PatchSignals produces the correct Datastar event type.
    #[test]
    fn patch_signals_creates_valid_event() {
        let signals = ChartSignals {
            chart_option: json!({"xAxis": {"type": "category"}}),
            selected: None,
            loading: false,
            error: None,
        };
        let signals_json = serde_json::to_string(&signals).unwrap();
        let patch = PatchSignals::new(signals_json);

        // Verify conversion to axum Event succeeds without panic.
        let _event: Event = patch.into();
    }

    /// Verify loading ChartSignals serializes correctly for PatchSignals.
    #[test]
    fn loading_chart_signals_serializes_for_patch() {
        let signals = ChartSignals {
            chart_option: json!({}),
            selected: None,
            loading: true,
            error: None,
        };

        let json = serde_json::to_string(&signals).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["loading"], true);
        assert_eq!(parsed["chartOption"], json!({}));
    }

    /// Verify graceful degradation when analytics service is unavailable.
    ///
    /// When DuckLake catalog attachment fails at startup (e.g., network error,
    /// invalid URI), the DuckDBService is created with None pool. This test
    /// verifies the error path surfaces properly in ChartSignals.
    #[tokio::test]
    async fn analytics_unavailable_surfaces_error_in_signals() {
        use crate::infrastructure::analytics::{AnalyticsState, DuckDBService};

        // Create service with no pool (simulates failed catalog attachment)
        let service = DuckDBService::new(None);
        assert!(!service.is_available());

        let analytics = AnalyticsState::new(service);

        // Simulate the query that would happen in astronauts_chart_sse
        let query_result = analytics
            .service
            .query(|conn| {
                // This closure won't execute — service returns error immediately
                let mut stmt = conn.prepare("SELECT 1")?;
                stmt.query_row([], |row| row.get::<_, i64>(0))
            })
            .await;

        // Verify error is returned
        assert!(query_result.is_err());
        let err = query_result.unwrap_err();
        assert!(
            err.to_string().contains("analytics service unavailable"),
            "unexpected error message: {err}"
        );

        // Verify ChartSignals construction matches the handler's error path
        let signals = ChartSignals {
            chart_option: json!({}),
            selected: None,
            loading: false,
            error: Some(format!("Query error: {err}")),
        };

        assert!(signals.error.is_some());
        assert!(
            signals.error.as_ref().unwrap().contains("unavailable"),
            "error message should contain 'unavailable': {:?}",
            signals.error
        );
        assert_eq!(signals.chart_option, json!({}));
        assert!(!signals.loading);
    }
}
