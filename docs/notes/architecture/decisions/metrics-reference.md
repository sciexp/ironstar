# Prometheus metrics reference

This document provides the comprehensive metrics reference for ironstar's Prometheus integration.
For architectural context including core observability principles, logging configuration, and health checks, see `observability-decisions.md`.

## Dependencies

```toml
[dependencies]
prometheus = { version = "0.13", features = ["process"] }
once_cell = "1.20"
```

## Metrics registry

```rust
use once_cell::sync::Lazy;
use prometheus::{
    CounterVec, GaugeVec, HistogramVec, IntCounter, IntGauge, Registry, Opts, HistogramOpts,
};

/// Global Prometheus registry
pub static METRICS_REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

/// Event store metrics
pub static EVENTS_APPENDED_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new(
        "events_appended_total",
        "Total number of events appended to event store"
    );
    let counter = CounterVec::new(opts, &["aggregate_type"]).unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

pub static APPEND_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "append_duration_seconds",
        "Duration of event append operations"
    )
    .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]);
    let histogram = HistogramVec::new(opts, &["aggregate_type"]).unwrap();
    METRICS_REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

/// SSE connection metrics
pub static SSE_CONNECTIONS: Lazy<IntGauge> = Lazy::new(|| {
    let opts = Opts::new(
        "sse_connections",
        "Current number of active SSE connections"
    );
    let gauge = IntGauge::with_opts(opts).unwrap();
    METRICS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub static SSE_EMIT_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "sse_emit_duration_seconds",
        "Duration of SSE event rendering and emission"
    )
    .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5]);
    let histogram = HistogramVec::new(opts, &["event_type"]).unwrap();
    METRICS_REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

pub static BROADCAST_LAGS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let opts = Opts::new(
        "broadcast_lags_total",
        "Total number of broadcast lag events (slow consumers)"
    );
    let counter = IntCounter::with_opts(opts).unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

/// Projection metrics
pub static PROJECTION_LAG_SECONDS: Lazy<GaugeVec> = Lazy::new(|| {
    let opts = Opts::new(
        "projection_lag_seconds",
        "Time lag between event creation and projection update"
    );
    let gauge = GaugeVec::new(opts, &["projection_name"]).unwrap();
    METRICS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub static PROJECTION_EVENTS_PROCESSED: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new(
        "projection_events_processed_total",
        "Total events processed by each projection"
    );
    let counter = CounterVec::new(opts, &["projection_name"]).unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

/// Cache metrics
pub static CACHE_HIT_RATIO: Lazy<GaugeVec> = Lazy::new(|| {
    let opts = Opts::new(
        "cache_hit_ratio",
        "Cache hit ratio (0.0 to 1.0)"
    );
    let gauge = GaugeVec::new(opts, &["cache_name"]).unwrap();
    METRICS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub static CACHE_EVICTIONS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new(
        "cache_evictions_total",
        "Total number of cache evictions"
    );
    let counter = CounterVec::new(opts, &["cache_name", "reason"]).unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});
```

## Instrumented event store

```rust
use prometheus::Histogram;

impl SqliteEventStore {
    pub async fn append_events(
        &self,
        aggregate_id: &str,
        events: &[Event],
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let timer = APPEND_DURATION_SECONDS
            .with_label_values(&[events.first().unwrap().aggregate_type()])
            .start_timer();

        // Perform append operation
        let stored_events = self.append_events_inner(aggregate_id, events).await?;

        timer.observe_duration();
        EVENTS_APPENDED_TOTAL
            .with_label_values(&[events.first().unwrap().aggregate_type()])
            .inc_by(events.len() as f64);

        Ok(stored_events)
    }
}
```

## Instrumented SSE handler

```rust
use axum::response::sse::Event;

pub async fn sse_feed(
    State(state): State<AppState>,
    Extension(session_id): Extension<SessionId>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    SSE_CONNECTIONS.inc();

    let rx = state.event_bus.subscribe();
    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            let timer = SSE_EMIT_DURATION_SECONDS
                .with_label_values(&[&event.event_type])
                .start_timer();

            yield Ok(convert_to_sse(event));

            timer.observe_duration();
        }

        SSE_CONNECTIONS.dec();
    };

    Sse::new(stream)
}
```

## Instrumented cache

```rust
use moka::future::Cache;

pub struct AnalyticsCache {
    cache: Cache<String, Vec<u8>>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl AnalyticsCache {
    pub async fn get_or_compute<F, Fut>(
        &self,
        key: String,
        compute: F,
    ) -> Result<Vec<u8>, Error>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<u8>, Error>>,
    {
        if let Some(value) = self.cache.get(&key).await {
            self.hits.fetch_add(1, Ordering::Relaxed);
            self.update_hit_ratio();
            return Ok(value);
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        self.update_hit_ratio();

        let value = compute().await?;
        self.cache.insert(key, value.clone()).await;
        Ok(value)
    }

    fn update_hit_ratio(&self) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total > 0 {
            let ratio = hits as f64 / total as f64;
            CACHE_HIT_RATIO
                .with_label_values(&["analytics"])
                .set(ratio);
        }
    }

    pub fn on_eviction(&self, reason: &str) {
        CACHE_EVICTIONS_TOTAL
            .with_label_values(&["analytics", reason])
            .inc();
    }
}
```

## Metrics endpoint

```rust
use axum::{routing::get, Router};
use prometheus::{Encoder, TextEncoder};

async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = METRICS_REGISTRY.gather();
    let mut buffer = Vec::new();

    encoder.encode(&metric_families, &mut buffer).unwrap();

    (
        axum::http::StatusCode::OK,
        [("content-type", encoder.format_type())],
        buffer,
    )
}

pub fn metrics_routes() -> Router<AppState> {
    Router::new().route("/metrics", get(metrics_handler))
}
```

## Key metrics summary

| Metric | Type | Purpose | Alert Threshold |
|--------|------|---------|-----------------|
| `events_appended_total` | Counter | Track write throughput | N/A (informational) |
| `append_duration_seconds` | Histogram | Event store performance | p99 > 100ms (SQLite tuning) |
| `sse_connections` | Gauge | Active client count | > 1000 (scale up) |
| `sse_emit_duration_seconds` | Histogram | SSE rendering performance | p99 > 50ms (optimize templates) |
| `broadcast_lags_total` | Counter | Slow consumer detection | > 10/min (investigate) |
| `projection_lag_seconds` | Gauge | Projection freshness | > 60s (stale projections) |
| `projection_events_processed_total` | Counter | Projection throughput | Decreasing (backlog growing) |
| `cache_hit_ratio` | Gauge | Cache effectiveness | < 0.5 (poor cache utilization) |
| `cache_evictions_total` | Counter | Cache pressure | High rate (increase capacity) |

## Alerting thresholds

Recommended Prometheus alerting rules for production deployments.

### Critical alerts

```yaml
groups:
  - name: ironstar_critical
    interval: 30s
    rules:
      - alert: DatabaseDown
        expr: up{job="ironstar"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Ironstar database is down"
          description: "Database health check failed for more than 1 minute"

      - alert: HighSSEConnectionDrop
        expr: rate(sse_connections[5m]) < -10
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High rate of SSE disconnections"
          description: "More than 10 SSE clients disconnecting per minute"
```

### Warning alerts

```yaml
      - alert: HighProjectionLag
        expr: projection_lag_seconds > 60
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Projection lag exceeds 60 seconds"
          description: "Projection {{ $labels.projection_name }} is lagging behind event stream"

      - alert: LowCacheHitRatio
        expr: cache_hit_ratio < 0.5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Cache hit ratio below 50%"
          description: "Cache {{ $labels.cache_name }} has poor hit ratio, consider tuning"

      - alert: FrequentBroadcastLags
        expr: rate(broadcast_lags_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Frequent broadcast lag events"
          description: "Slow consumers detected, consider rate limiting or capacity increase"
```

## Related documentation

- **Core observability architecture**: `observability-decisions.md` (principles, logging, health checks)
- **Performance tuning metrics**: `../cqrs/performance-tuning.md` (broadcast lags, batching, rate limiting)
- **Zenoh monitoring**: `../infrastructure/zenoh-event-bus.md` (key expression debugging, subscriber health)
- **Cache invalidation**: `../infrastructure/analytics-cache-architecture.md` (Pattern 4: Zenoh-based invalidation)
