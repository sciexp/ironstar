//! Prometheus metrics scrape endpoint.
//!
//! Exposes a `GET /metrics` handler that returns the Prometheus text
//! exposition format for scraping by Prometheus or compatible collectors.
//!
//! # Routes
//!
//! - `GET /metrics` - Prometheus text exposition format
//!
//! # Integration
//!
//! Configure Prometheus to scrape this endpoint:
//!
//! ```yaml
//! scrape_configs:
//!   - job_name: ironstar
//!     static_configs:
//!       - targets: ['127.0.0.1:3000']
//!     scrape_interval: 15s
//! ```

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use metrics_exporter_prometheus::PrometheusHandle;
use tracing::instrument;

use crate::state::AppState;

/// Application state subset for the metrics endpoint.
///
/// Contains the Prometheus handle used to render metric values on demand.
#[derive(Clone)]
pub struct MetricsState {
    /// Handle to the Prometheus recorder for rendering exposition format.
    pub prometheus_handle: PrometheusHandle,
}

/// GET /metrics - Prometheus text exposition format.
///
/// Returns all registered metrics in the Prometheus text format with
/// `Content-Type: text/plain; version=0.0.4; charset=utf-8` as required
/// by the Prometheus exposition format specification.
#[instrument(name = "handler.metrics", skip(state))]
pub async fn metrics_handler(State(state): State<MetricsState>) -> impl IntoResponse {
    let body = state.prometheus_handle.render();
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
}

/// Creates the metrics feature router.
///
/// # Routes
///
/// - `GET /metrics` - Prometheus text exposition format
///
/// Handlers extract `MetricsState` via `FromRef<AppState>`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/metrics", get(metrics_handler))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::infrastructure::metrics::test_prometheus_handle;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn create_test_router() -> Router {
        let state = MetricsState {
            prometheus_handle: test_prometheus_handle(),
        };
        Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(state)
    }

    #[tokio::test]
    async fn metrics_returns_ok() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn metrics_returns_prometheus_content_type() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("content-type header");
        assert_eq!(content_type, "text/plain; version=0.0.4; charset=utf-8");
    }

    #[tokio::test]
    async fn metrics_body_is_valid_text() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body read");

        // Body should be valid UTF-8 (Prometheus text format is text/plain)
        let text = std::str::from_utf8(&body).expect("valid utf-8");
        // Empty metrics or with descriptions is valid
        assert!(
            text.is_empty() || text.len() < 1_000_000,
            "metrics output should be reasonable size"
        );
    }
}
