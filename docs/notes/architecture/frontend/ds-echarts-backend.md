# ds-echarts Backend Integration

This document covers the Rust backend patterns for integrating ds-echarts with axum, DuckDB analytics, and Zenoh event distribution.

See `ds-echarts-integration-guide.md` for the complete integration overview and frontend patterns.

## Axum Handler Patterns

### Chart Data Handler (SSE)

```rust
// src/presentation/handlers/charts.rs
use axum::{
    extract::{Path, State},
    response::{sse::{Event, Sse}, IntoResponse},
};
use datastar::{PatchElements, PatchSignals, ReadSignals};
use futures::stream::{self, Stream};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio_stream::StreamExt;

use crate::{
    domain::signals::{ChartSignals, ChartSelection},
    infrastructure::{analytics::DuckDBService, event_bus::EventBus},
    presentation::templates::charts::echarts_chart,
};

pub struct AppState {
    pub analytics: Arc<DuckDBService>,
    pub event_bus: Arc<EventBus>,
}

/// GET /api/charts/:id/data - Initial chart load via SSE
pub async fn chart_data_sse(
    State(state): State<Arc<AppState>>,
    Path(chart_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        // 1. Query initial data from DuckDB
        let query_result = state.analytics
            .query_chart_data(&chart_id)
            .await;

        match query_result {
            Ok(data) => {
                // 2. Build chart option from data
                let chart_option = build_echarts_option(&data);
                let signals = ChartSignals {
                    chart_option,
                    selected: None,
                    loading: false,
                };

                // 3. Render chart HTML
                let html = echarts_chart(&chart_id, &signals, "400px")
                    .render()
                    .into_inner();

                // 4. Send PatchElements to replace container
                let patch = PatchElements::new(html)
                    .selector(&format!("#{}-container", chart_id));
                yield Ok(patch.write_as_axum_sse_event());

                // 5. Subscribe to real-time updates via Zenoh
                let mut subscription = state.event_bus
                    .subscribe(&format!("charts/{}/updates", chart_id))
                    .await;

                while let Some(update) = subscription.next().await {
                    let updated_option = build_echarts_option(&update);
                    let signals_json = serde_json::json!({
                        "chartOption": updated_option,
                        "loading": false,
                    }).to_string();

                    let patch = PatchSignals::new(signals_json);
                    yield Ok(patch.write_as_axum_sse_event());
                }
            }
            Err(e) => {
                // Send error state
                let error_html = format!(
                    r#"<div id="{}-container" class="alert error">Failed to load chart: {}</div>"#,
                    chart_id, e
                );
                let patch = PatchElements::new(error_html)
                    .selector(&format!("#{}-container", chart_id));
                yield Ok(patch.write_as_axum_sse_event());
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// POST /api/chart/select - Handle chart click interactions
pub async fn chart_select(
    State(state): State<Arc<AppState>>,
    ReadSignals(selection): ReadSignals<ChartSelection>,
) -> impl IntoResponse {
    let stream = async_stream::stream! {
        // 1. Signal loading state
        let loading_patch = PatchSignals::new(r#"{"loading": true}"#);
        yield Ok::<_, Infallible>(loading_patch.write_as_axum_sse_event());

        // 2. Query drill-down data based on selection
        let drill_down = state.analytics
            .query_drill_down(&selection.series_name, selection.data_index)
            .await;

        match drill_down {
            Ok(detail_data) => {
                // 3. Publish to Zenoh for other subscribers
                state.event_bus
                    .publish(&format!("charts/detail/{}", selection.name), &detail_data)
                    .await;

                // 4. Return updated signals
                let signals_json = serde_json::json!({
                    "detailData": detail_data,
                    "loading": false,
                }).to_string();

                let patch = PatchSignals::new(signals_json);
                yield Ok(patch.write_as_axum_sse_event());
            }
            Err(e) => {
                let error_patch = PatchSignals::new(
                    serde_json::json!({
                        "loading": false,
                        "error": e.to_string(),
                    }).to_string()
                );
                yield Ok(error_patch.write_as_axum_sse_event());
            }
        }
    };

    Sse::new(stream)
}

fn build_echarts_option(data: &AnalyticsData) -> serde_json::Value {
    serde_json::json!({
        "title": { "text": &data.title },
        "tooltip": { "trigger": "axis" },
        "xAxis": {
            "type": "category",
            "data": data.labels
        },
        "yAxis": { "type": "value" },
        "series": [{
            "data": data.values,
            "type": "bar",
            "emphasis": { "focus": "series" }
        }]
    })
}
```

## DuckDB Analytics Service

```rust
// src/infrastructure/analytics.rs
use duckdb::{Connection, Result as DuckResult};
use std::sync::Mutex;

pub struct DuckDBService {
    pool: Arc<async_duckdb::Pool>,
}

pub struct AnalyticsData {
    pub title: String,
    pub labels: Vec<String>,
    pub values: Vec<f64>,
}

impl DuckDBService {
    pub async fn new() -> Result<Self, Error> {
        // Initialize pool with httpfs extension
        let pool = async_duckdb::PoolBuilder::new()
            .path(":memory:")
            .num_conns(2)
            .open()
            .await?;

        // Install required extensions
        pool.conn(|conn| {
            conn.execute_batch(
                r#"
                INSTALL httpfs; LOAD httpfs;
                INSTALL parquet; LOAD parquet;
                "#
            )
        }).await?;

        Ok(Self { pool: Arc::new(pool) })
    }

    pub async fn query_chart_data(&self, chart_id: &str) -> Result<AnalyticsData, Error> {
        let chart_id = chart_id.to_string();

        // Non-blocking query via async-duckdb pool
        self.pool.conn(move |conn| {
            // Example: query from HuggingFace dataset
            let mut stmt = conn.prepare(
                r#"
                SELECT category, SUM(value) as total
                FROM read_parquet('hf://datasets/org/data.parquet')
                WHERE chart_id = ?
                GROUP BY category
                ORDER BY total DESC
                "#
            )?;

            let rows = stmt.query_map([&chart_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })?;

            let mut labels = Vec::new();
            let mut values = Vec::new();
            for row in rows {
                let (label, value) = row?;
                labels.push(label);
                values.push(value);
            }

            Ok(AnalyticsData {
                title: format!("Chart {}", chart_id),
                labels,
                values,
            })
        }).await
    }

    pub async fn query_drill_down(
        &self,
        series: &str,
        index: i32,
    ) -> Result<serde_json::Value, Error> {
        // Similar pattern for drill-down queries using pool.conn()
        todo!()
    }
}
```

## Zenoh Event Bus

```rust
// src/infrastructure/event_bus.rs
use std::sync::Arc;
use zenoh::{Config, Session};
use tokio_stream::Stream;

pub struct EventBus {
    session: Arc<Session>,
}

impl EventBus {
    pub async fn new() -> Result<Self, zenoh::Error> {
        // Embedded mode - no external server
        let mut config = Config::default();
        config.set_mode(Some(zenoh::config::WhatAmI::Peer))?;

        let session = Arc::new(zenoh::open(config).await?);
        Ok(Self { session })
    }

    pub async fn publish(&self, key: &str, data: &impl serde::Serialize) -> Result<(), Error> {
        let json = serde_json::to_string(data)?;
        self.session
            .put(key, json)
            .encoding(zenoh::bytes::Encoding::APPLICATION_JSON)
            .await?;
        Ok(())
    }

    pub async fn subscribe(&self, pattern: &str) -> impl Stream<Item = serde_json::Value> {
        let subscriber = self.session
            .declare_subscriber(pattern)
            .await
            .expect("Failed to create subscriber");

        async_stream::stream! {
            loop {
                match subscriber.recv_async().await {
                    Ok(sample) => {
                        if let Ok(json) = sample.payload().try_to_string() {
                            if let Ok(value) = serde_json::from_str(&json) {
                                yield value;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
}
```

## Router Configuration

```rust
// src/presentation/routes.rs
use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use crate::presentation::handlers::charts::{chart_data_sse, chart_select, AppState};

pub fn chart_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/charts/:id/data", get(chart_data_sse))
        .route("/api/chart/select", post(chart_select))
        .with_state(state)
}
```

## Related Documentation

- **Integration overview**: `ds-echarts-integration-guide.md` — Component properties, hypertext templates, critical notes
- **Build and testing**: `ds-echarts-build-test.md` — TypeScript/Rolldown config, Vitest setup, data flow diagrams
- **Zenoh architecture**: `../infrastructure/zenoh-event-bus.md` — Key expression patterns, embedded config, migration from tokio broadcast
- **Analytics cache**: `../infrastructure/analytics-cache-architecture.md` — moka cache with TTL, Zenoh-based invalidation
- **SSE patterns**: `../cqrs/sse-connection-lifecycle.md` — Client subscription, reconnection resilience, Last-Event-ID
