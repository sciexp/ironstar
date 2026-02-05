//! Health check HTTP handlers.
//!
//! This module provides standard health check endpoints for Kubernetes-style
//! health probes and process-compose integration.
//!
//! # Routes
//!
//! - `GET /health` - Combined health status (JSON)
//! - `GET /health/ready` - Readiness probe (200 when ready to serve traffic)
//! - `GET /health/live` - Liveness probe (200 when process is alive)
//!
//! # Kubernetes integration
//!
//! These endpoints follow Kubernetes health probe conventions:
//!
//! - **Liveness**: Returns 200 if the server is running. Failing this probe
//!   signals that the container should be restarted.
//!
//! - **Readiness**: Returns 200 if the server can handle traffic. Failing this
//!   probe removes the pod from service load balancing but does not restart it.
//!
//! # process-compose integration
//!
//! Configure health probes in `process-compose.yaml`:
//!
//! ```yaml
//! processes:
//!   ironstar:
//!     command: ./target/release/ironstar
//!     readiness_probe:
//!       http_get:
//!         path: /health/ready
//!         port: 3000
//!       initial_delay_seconds: 2
//!       period_seconds: 5
//!     liveness_probe:
//!       http_get:
//!         path: /health/live
//!         port: 3000
//!       period_seconds: 10
//! ```

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use tracing::instrument;

use crate::state::AppState;

/// Application state for health check handlers.
///
/// Contains references to infrastructure components that need to be checked
/// for readiness status.
#[derive(Clone)]
pub struct HealthState {
    /// SQLite connection pool for database health checks.
    pub db_pool: SqlitePool,
}

/// Combined health status response.
///
/// Returns detailed status of all infrastructure components.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Overall health status.
    pub status: HealthStatus,
    /// Individual component check results.
    pub checks: HealthChecks,
}

/// Overall health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All components healthy.
    Healthy,
    /// One or more components degraded but service is functional.
    Degraded,
    /// Service is unhealthy and cannot serve traffic.
    Unhealthy,
}

/// Individual component health check results.
#[derive(Debug, Serialize)]
pub struct HealthChecks {
    /// Database connection status.
    pub database: CheckStatus,
}

/// Individual check status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    /// Component is healthy and responsive.
    Ok,
    /// Component is degraded but functional.
    Degraded,
    /// Component has failed.
    Failed,
}

impl HealthResponse {
    /// Determine if the overall health is OK (suitable for readiness).
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.status == HealthStatus::Healthy || self.status == HealthStatus::Degraded
    }
}

/// GET /health - Combined health status.
///
/// Returns a JSON object with overall status and individual component checks.
/// This endpoint performs actual health checks against infrastructure.
///
/// # Response
///
/// - `200 OK` with JSON body when all checks pass
/// - `503 Service Unavailable` with JSON body when any check fails
///
/// # Example response
///
/// ```json
/// {
///   "status": "healthy",
///   "checks": {
///     "database": "ok"
///   }
/// }
/// ```
#[instrument(name = "handler.health.status", skip(state))]
pub async fn health(State(state): State<HealthState>) -> impl IntoResponse {
    let database_status = check_database(&state.db_pool).await;

    let overall_status = if database_status == CheckStatus::Ok {
        HealthStatus::Healthy
    } else if database_status == CheckStatus::Degraded {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };

    let response = HealthResponse {
        status: overall_status,
        checks: HealthChecks {
            database: database_status,
        },
    };

    let status_code = if response.is_ready() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(response))
}

/// GET /health/ready - Readiness probe.
///
/// Returns 200 OK if the server is ready to handle traffic.
/// This checks that all required infrastructure is available.
///
/// # Response
///
/// - `200 OK` with plain text "ready" when ready
/// - `503 Service Unavailable` with plain text "not ready" when not ready
#[instrument(name = "handler.health.ready", skip(state))]
pub async fn ready(State(state): State<HealthState>) -> impl IntoResponse {
    let database_status = check_database(&state.db_pool).await;

    if database_status == CheckStatus::Ok || database_status == CheckStatus::Degraded {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not ready")
    }
}

/// GET /health/live - Liveness probe.
///
/// Returns 200 OK if the server process is alive and responsive.
/// This is a fast check that does not verify infrastructure connectivity.
///
/// # Response
///
/// - `200 OK` with plain text "alive"
///
/// This endpoint always returns 200 as long as the HTTP server is running.
/// Kubernetes uses liveness failures to trigger container restarts.
#[instrument(name = "handler.health.live")]
pub async fn live() -> impl IntoResponse {
    (StatusCode::OK, "alive")
}

/// Check database connectivity by executing a simple query.
///
/// Uses `SELECT 1` as a fast ping query that validates the connection pool
/// can acquire a connection and execute queries.
async fn check_database(pool: &SqlitePool) -> CheckStatus {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
    {
        Ok(_) => CheckStatus::Ok,
        Err(_) => CheckStatus::Failed,
    }
}

/// Creates the health feature router with all endpoints.
///
/// # Routes
///
/// - `GET /health` - Combined health status (JSON)
/// - `GET /health/ready` - Readiness probe
/// - `GET /health/live` - Liveness probe
///
/// Handlers extract `HealthState` via `FromRef<AppState>`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/health/ready", get(ready))
        .route("/health/live", get(live))
}

/// Create a router with health check endpoints.
///
/// Returns a router that can be merged with the main application router.
/// The routes are:
///
/// - `GET /health` - Combined health status
/// - `GET /health/ready` - Readiness probe
/// - `GET /health/live` - Liveness probe
///
/// # Example
///
/// ```rust,ignore
/// use ironstar::presentation::health::{health_router, HealthState};
///
/// let health_state = HealthState {
///     db_pool: pool.clone(),
/// };
///
/// let app = Router::new()
///     .merge(health_router(health_state))
///     .merge(api_router);
/// ```
pub fn health_router(state: HealthState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/health/ready", get(ready))
        .route("/health/live", get(live))
        .with_state(state)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool")
    }

    fn create_router(pool: SqlitePool) -> Router {
        health_router(HealthState { db_pool: pool })
    }

    #[tokio::test]
    async fn health_returns_healthy_status() {
        let pool = create_test_pool().await;
        let app = create_router(pool);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body read");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json parse");

        assert_eq!(json["status"], "healthy");
        assert_eq!(json["checks"]["database"], "ok");
    }

    #[tokio::test]
    async fn ready_returns_ok() {
        let pool = create_test_pool().await;
        let app = create_router(pool);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body read");
        assert_eq!(&body[..], b"ready");
    }

    #[tokio::test]
    async fn live_returns_ok() {
        let pool = create_test_pool().await;
        let app = create_router(pool);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(Body::empty())
                    .expect("request body"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body read");
        assert_eq!(&body[..], b"alive");
    }

    #[tokio::test]
    async fn health_response_serialization() {
        let response = HealthResponse {
            status: HealthStatus::Healthy,
            checks: HealthChecks {
                database: CheckStatus::Ok,
            },
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"database\":\"ok\""));
    }

    #[tokio::test]
    async fn health_response_is_ready() {
        let healthy = HealthResponse {
            status: HealthStatus::Healthy,
            checks: HealthChecks {
                database: CheckStatus::Ok,
            },
        };
        assert!(healthy.is_ready());

        let degraded = HealthResponse {
            status: HealthStatus::Degraded,
            checks: HealthChecks {
                database: CheckStatus::Degraded,
            },
        };
        assert!(degraded.is_ready());

        let unhealthy = HealthResponse {
            status: HealthStatus::Unhealthy,
            checks: HealthChecks {
                database: CheckStatus::Failed,
            },
        };
        assert!(!unhealthy.is_ready());
    }
}
