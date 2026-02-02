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
//! 8. Attach DuckLake catalogs (embedded first, network fallback)
//! 9. Initialize analytics cache layer
//! 10. Spawn cache invalidation subscriber
//! 11. Construct AppState
//! 12. Compose router
//! 13. Start server with graceful shutdown

use ironstar::config::Config;
use ironstar::infrastructure::{
    AnalyticsCache, AssetManifest, CachedAnalyticsService, DuckDBService, ZenohEventBus,
    embedded_catalogs, open_embedded_session, spawn_cache_invalidation,
};
use ironstar::presentation::app_router;
use ironstar::state::AppState;
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
        let mut builder = async_duckdb::PoolBuilder::new().num_conns(config.analytics_num_conns);
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

                        // Attach embedded DuckLake catalogs (zero-latency, from binary)
                        match embedded_catalogs::attach_all(&service).await {
                            Ok(names) if !names.is_empty() => {
                                tracing::info!(
                                    catalogs = ?names,
                                    source = "embedded",
                                    "Attached embedded DuckLake catalogs"
                                );
                            }
                            Ok(_) => {
                                // No embedded catalogs — fall back to network ATTACH
                                tracing::debug!("No embedded catalogs, trying network ATTACH");
                                let catalog_uri =
                                    "ducklake:hf://datasets/sciexp/fixtures/lakes/frozen/space.db";
                                match service.attach_catalog("space", catalog_uri).await {
                                    Ok(()) => {
                                        tracing::info!(
                                            catalog = "space",
                                            source = "hf://sciexp/fixtures",
                                            "Attached DuckLake catalog via network"
                                        );
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            error = %e,
                                            catalog = "space",
                                            "Failed to attach DuckLake catalog, \
                                             continuing without demo data"
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    "Failed to attach embedded catalogs"
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

    // 9. Initialize analytics cache layer (if analytics available)
    let cached_analytics = analytics.as_ref().map(|pool| {
        let service = DuckDBService::new(Some(pool.clone()));
        let cache = AnalyticsCache::new();
        let cached = CachedAnalyticsService::new(service, cache);
        tracing::info!("Analytics cache layer initialized");
        cached
    });

    // 10. Spawn cache invalidation subscriber (if both Zenoh and cached analytics)
    if let (Some(bus), Some(cached)) = (&event_bus, &cached_analytics) {
        let registry =
            ironstar::infrastructure::CacheInvalidationRegistry::new(cached.clone());
        // Dependencies will be registered as domain aggregates are added.
        // For now, the registry is empty and ready for 3gd domain integration.
        let _handle = spawn_cache_invalidation(bus.session().clone(), registry);
        tracing::info!("Cache invalidation subscriber spawned");
    }

    // 11. Construct AppState
    let mut app_state = AppState::new(db_pool.clone(), assets);
    if let Some(bus) = event_bus {
        app_state = app_state.with_event_bus(bus);
    }
    if let Some(pool) = analytics {
        app_state = app_state.with_analytics(pool);
    }
    if let Some(cached) = cached_analytics {
        app_state = app_state.with_cached_analytics(cached);
    }

    // 12. Compose router
    let app = app_router(app_state);

    // 13. Start server with graceful shutdown
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
