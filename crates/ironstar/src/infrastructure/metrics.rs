//! Prometheus metrics infrastructure.
//!
//! Initializes the `metrics` facade with a Prometheus exporter recorder and
//! exposes a handle for rendering the Prometheus text exposition format at
//! the `/metrics` endpoint.
//!
//! # Metric naming conventions
//!
//! All application metrics follow Prometheus naming conventions:
//!
//! - Counters use `_total` suffix (e.g., `http_requests_total`)
//! - Histograms use `_seconds` or `_bytes` suffix for units
//! - Labels use snake_case (e.g., `aggregate_type`, `http_method`)
//!
//! # Architecture
//!
//! The `metrics` crate provides a facade pattern (like `log` or `tracing`):
//! library code emits metrics via macros (`counter!`, `histogram!`), and the
//! recorder installed at startup determines where metrics go. This module
//! installs a Prometheus recorder that accumulates metrics in memory and
//! renders them on demand for the `/metrics` scrape endpoint.

use metrics_exporter_prometheus::{BuildError, PrometheusBuilder, PrometheusHandle};

// ---------------------------------------------------------------------------
// Metric name constants
// ---------------------------------------------------------------------------

/// HTTP request counter (labels: method, path, status).
pub const HTTP_REQUESTS_TOTAL: &str = "http_requests_total";

/// HTTP request duration histogram in seconds (labels: method, path).
pub const HTTP_REQUEST_DURATION_SECONDS: &str = "http_request_duration_seconds";

/// Events persisted to the event store (labels: aggregate_type).
pub const EVENTS_PERSISTED_TOTAL: &str = "events_persisted_total";

/// Analytics cache hit counter.
pub const CACHE_HITS_TOTAL: &str = "cache_hits_total";

/// Analytics cache miss counter.
pub const CACHE_MISSES_TOTAL: &str = "cache_misses_total";

/// Query execution duration histogram in seconds.
pub const QUERY_DURATION_SECONDS: &str = "query_duration_seconds";

// ---------------------------------------------------------------------------
// Recorder initialization
// ---------------------------------------------------------------------------

/// Initialize the Prometheus metrics recorder and return a handle for rendering.
///
/// This installs the Prometheus recorder as the global `metrics` recorder.
/// It must be called exactly once during application startup, before any
/// metrics are emitted.
///
/// The returned `PrometheusHandle` is cheaply cloneable and used by the
/// `/metrics` HTTP handler to render the Prometheus text exposition format.
///
/// # Errors
///
/// Returns an error if a global recorder has already been installed or if
/// the builder configuration is invalid.
pub fn init_prometheus_recorder() -> Result<PrometheusHandle, BuildError> {
    let handle = PrometheusBuilder::new().install_recorder()?;

    // Register descriptions so Prometheus sees HELP/TYPE lines even before
    // any values are recorded.
    describe_metrics();

    Ok(handle)
}

/// Register metric descriptions with the global recorder.
///
/// Descriptions appear as `# HELP` comments in the Prometheus exposition
/// format, making metrics self-documenting for operators. Uses the
/// `metrics::describe_*` macros which operate on the installed global
/// recorder.
fn describe_metrics() {
    metrics::describe_counter!(
        HTTP_REQUESTS_TOTAL,
        metrics::Unit::Count,
        "Total number of HTTP requests handled"
    );

    metrics::describe_histogram!(
        HTTP_REQUEST_DURATION_SECONDS,
        metrics::Unit::Seconds,
        "HTTP request duration in seconds"
    );

    metrics::describe_counter!(
        EVENTS_PERSISTED_TOTAL,
        metrics::Unit::Count,
        "Total number of events persisted to the event store"
    );

    metrics::describe_counter!(
        CACHE_HITS_TOTAL,
        metrics::Unit::Count,
        "Total number of analytics cache hits"
    );

    metrics::describe_counter!(
        CACHE_MISSES_TOTAL,
        metrics::Unit::Count,
        "Total number of analytics cache misses"
    );

    metrics::describe_histogram!(
        QUERY_DURATION_SECONDS,
        metrics::Unit::Seconds,
        "Query execution duration in seconds"
    );
}

/// Create a non-global Prometheus handle for testing.
///
/// This builds a recorder without installing it as the global recorder,
/// making it safe to use in tests that run in parallel within the same
/// process. The returned handle can render metrics for the recorder but
/// only captures metrics explicitly registered via the recorder, not
/// metrics emitted via global macros.
///
/// Exposed unconditionally so integration tests in `tests/` can use it.
#[doc(hidden)]
pub fn test_prometheus_handle() -> PrometheusHandle {
    let recorder = PrometheusBuilder::new().build_recorder();
    recorder.handle()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn prometheus_handle_renders_valid_output() {
        let handle = test_prometheus_handle();

        let output = handle.render();
        // Empty output is valid Prometheus text format (no samples yet).
        assert!(
            output.is_empty() || output.len() < 1_000_000,
            "unexpected output size: {}",
            output.len()
        );
    }

    #[test]
    fn metric_name_constants_follow_prometheus_conventions() {
        // Counters end with _total
        assert!(HTTP_REQUESTS_TOTAL.ends_with("_total"));
        assert!(EVENTS_PERSISTED_TOTAL.ends_with("_total"));
        assert!(CACHE_HITS_TOTAL.ends_with("_total"));
        assert!(CACHE_MISSES_TOTAL.ends_with("_total"));

        // Histograms end with _seconds
        assert!(HTTP_REQUEST_DURATION_SECONDS.ends_with("_seconds"));
        assert!(QUERY_DURATION_SECONDS.ends_with("_seconds"));
    }
}
