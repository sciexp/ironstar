//! Ironstar application entry point.
//!
//! This is the composition root where all dependencies are wired together.
//! The initialization sequence follows a deterministic order with proper
//! error handling at each step.
//!
//! # Startup sequence
//!
//! 1. Load configuration from environment variables
//! 2. Initialize tracing subscriber
//! 3. Create parent directories and SQLite pool
//! 4. Run database migrations
//! 5. Load asset manifest (graceful fallback)
//! 6. Initialize Zenoh event bus (optional, graceful fallback)
//! 7. Initialize DuckDB analytics pool (optional, graceful fallback)
//! 8. Construct AppState
//! 9. Compose router
//! 10. Start server with graceful shutdown

use ironstar::config::Config;
use ironstar::infrastructure::{
    AssetManifest, DuckDBService, ZenohEventBus, create_static_router, open_embedded_session,
};
use ironstar::presentation::health::{health, live, ready};
use ironstar::presentation::chart_routes;
use ironstar::presentation::todo::routes as todo_routes;
use ironstar::state::AppState;

use axum::Router;
use axum::routing::get;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Application startup errors.
///
/// These errors are fatal and prevent the application from starting.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("Failed to create database directory: {0}")]
    CreateDir(#[from] std::io::Error),

    #[error("Failed to connect to database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Failed to run migrations: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Failed to bind to address: {0}")]
    Bind(std::io::Error),
}

#[tokio::main]
async fn main() -> Result<(), StartupError> {
    // 1. Load configuration
    let config = Config::from_env();

    // 2. Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ironstar=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(
        port = config.port,
        database_url = %config.database_url,
        enable_zenoh = config.enable_zenoh,
        enable_analytics = config.enable_analytics,
        shutdown_timeout_secs = config.shutdown_timeout.as_secs(),
        "Starting ironstar"
    );

    // 3. Create parent directories if needed
    if let Some(dir) = config.database_dir()
        && !dir.exists()
    {
        tracing::info!(path = %dir.display(), "Creating database directory");
        std::fs::create_dir_all(&dir)?;
    }

    // 4. Create SQLite pool
    tracing::debug!(url = %config.database_url, "Connecting to database");
    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // 5. Run migrations
    tracing::info!("Running database migrations");
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    tracing::info!("Database migrations complete");

    // 6. Load asset manifest (graceful fallback)
    let assets = AssetManifest::load();
    if assets.is_empty() {
        tracing::warn!(
            "Asset manifest empty - frontend build may not have run. \
             Static assets will use fallback paths."
        );
    } else {
        tracing::debug!("Asset manifest loaded");
    }

    // 7. Initialize Zenoh event bus (optional)
    let event_bus = if config.enable_zenoh {
        match open_embedded_session().await {
            Ok(session) => {
                tracing::info!("Zenoh event bus initialized in embedded mode");
                Some(Arc::new(ZenohEventBus::new(Arc::new(session))))
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to initialize Zenoh event bus, continuing without pub/sub"
                );
                None
            }
        }
    } else {
        tracing::info!("Zenoh event bus disabled by configuration");
        None
    };

    // 8. Initialize DuckDB analytics pool (optional)
    let analytics = if config.enable_analytics {
        let mut builder =
            async_duckdb::PoolBuilder::new().num_conns(config.analytics_num_conns);
        if let Some(ref path) = config.analytics_database_path {
            builder = builder.path(path);
        }
        match builder.open().await {
            Ok(pool) => {
                tracing::info!(
                    num_conns = config.analytics_num_conns,
                    path = ?config.analytics_database_path,
                    "DuckDB analytics pool initialized"
                );

                // Load httpfs and ducklake extensions on all connections
                let service = DuckDBService::new(Some(pool.clone()));
                match service.initialize_extensions().await {
                    Ok(()) => {
                        tracing::info!("DuckDB extensions loaded (httpfs, ducklake)");

                        // Attach demo DuckLake catalog from HuggingFace
                        // This provides example data for development and testing
                        let catalog_uri =
                            "ducklake:hf://datasets/sciexp/fixtures/lakes/frozen/space.db";
                        match service.attach_catalog("space", catalog_uri).await {
                            Ok(()) => {
                                tracing::info!(
                                    catalog = "space",
                                    source = "sciexp/fixtures",
                                    "Attached DuckLake catalog"
                                );
                            }
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    catalog = "space",
                                    "Failed to attach DuckLake catalog, continuing without demo data"
                                );
                            }
                        }

                        Some(pool)
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "Failed to load DuckDB extensions, continuing without analytics"
                        );
                        None
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to initialize DuckDB analytics pool, continuing without analytics"
                );
                None
            }
        }
    } else {
        tracing::info!("DuckDB analytics disabled by configuration");
        None
    };

    // 9. Construct AppState
    let mut app_state = AppState::new(db_pool.clone(), assets);
    if let Some(bus) = event_bus {
        app_state = app_state.with_event_bus(bus);
    }
    if let Some(pool) = analytics {
        app_state = app_state.with_analytics(pool);
    }

    // 10. Compose router
    let app = compose_router(app_state);

    // 11. Start server with graceful shutdown
    let addr = config.socket_addr();
    tracing::info!(addr = %addr, "Listening");

    let listener = TcpListener::bind(addr).await.map_err(StartupError::Bind)?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(config.shutdown_timeout))
        .await
        .map_err(StartupError::Bind)?;

    tracing::info!("Shutdown complete");
    Ok(())
}

/// Compose the application router from all feature routers.
///
/// Router composition follows a clear hierarchy:
/// - Health endpoints at root (/health/*)
/// - Todo feature at /todos
/// - Static assets at /static
///
/// Each feature router is composed with its own state type derived from AppState
/// via FromRef, then converted to a stateless router for merging.
fn compose_router(state: AppState) -> Router {
    use axum::extract::FromRef;
    use ironstar::presentation::health::HealthState;

    // Extract domain-specific states via FromRef
    let health_state = HealthState::from_ref(&state);

    // Health endpoints with HealthState
    let health_routes = Router::new()
        .route("/health", get(health))
        .route("/health/ready", get(ready))
        .route("/health/live", get(live))
        .with_state(health_state);

    // Todo routes with AppState (handlers extract via FromRef)
    let todo_routes = todo_routes().with_state(state.clone());

    // Chart routes with AppState
    let chart_routes = chart_routes().with_state(state.clone());

    // Static asset serving (stateless)
    let static_routes = create_static_router();

    // Merge all routes into the final router
    Router::new()
        .merge(health_routes)
        .nest("/todos", todo_routes)
        .nest("/charts", chart_routes)
        .merge(static_routes)
}

/// Wait for shutdown signal (SIGINT or SIGTERM).
///
/// Returns when a shutdown signal is received, then waits for the configured
/// timeout to allow in-flight requests to complete.
#[expect(clippy::expect_used, reason = "signal handlers must succeed or panic")]
async fn shutdown_signal(timeout: std::time::Duration) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            tracing::info!("Received SIGINT, starting graceful shutdown");
        }
        () = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown");
        }
    }

    // Give in-flight requests time to complete
    tracing::info!(
        timeout_secs = timeout.as_secs(),
        "Waiting for in-flight requests to complete"
    );
}
