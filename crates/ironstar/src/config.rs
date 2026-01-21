//! Application configuration loaded from environment variables.
//!
//! Configuration follows the twelve-factor app methodology: all configuration
//! is loaded from environment variables with sensible defaults for development.
//!
//! # Environment variables
//!
//! All ironstar-specific variables use the `IRONSTAR_` prefix:
//!
//! | Variable | Default | Description |
//! |----------|---------|-------------|
//! | `IRONSTAR_PORT` | 3000 | HTTP server port |
//! | `IRONSTAR_DATABASE_URL` | `sqlite:./data/ironstar.db?mode=rwc` | SQLite database path |
//! | `IRONSTAR_ENABLE_ZENOH` | true | Enable Zenoh event bus |
//! | `IRONSTAR_ENABLE_ANALYTICS` | true | Enable DuckDB analytics pool |
//! | `IRONSTAR_ANALYTICS_PATH` | (none) | DuckDB database path (in-memory if unset) |
//! | `IRONSTAR_ANALYTICS_NUM_CONNS` | 4 | Number of DuckDB connections in pool |
//! | `IRONSTAR_SHUTDOWN_TIMEOUT_SECS` | 30 | Graceful shutdown timeout |
//!
//! Standard variables (no prefix):
//!
//! | Variable | Default | Description |
//! |----------|---------|-------------|
//! | `RUST_LOG` | `ironstar=debug,tower_http=debug` | Tracing filter |

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

/// Application configuration loaded from environment variables.
///
/// Use [`Config::from_env()`] to load configuration at startup.
#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP server port.
    pub port: u16,

    /// SQLite database URL (path for file database, or `:memory:` for in-memory).
    pub database_url: String,

    /// Whether to enable the Zenoh event bus.
    ///
    /// When disabled, event publishing is skipped but commands still work.
    /// Useful for testing or resource-constrained environments.
    pub enable_zenoh: bool,

    /// Whether to enable the DuckDB analytics pool.
    ///
    /// When disabled, analytics endpoints return 503 Service Unavailable.
    pub enable_analytics: bool,

    /// DuckDB database path for analytics.
    ///
    /// When `None`, DuckDB uses an in-memory database (ephemeral).
    /// Set to a file path for persistent analytics storage.
    pub analytics_database_path: Option<String>,

    /// Number of DuckDB connections in the analytics pool.
    pub analytics_num_conns: usize,

    /// Graceful shutdown timeout.
    ///
    /// When shutdown is signaled, the server waits this long for in-flight
    /// requests to complete before forcefully terminating.
    pub shutdown_timeout: Duration,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Missing variables use sensible defaults for local development.
    /// Invalid values are logged as warnings and fall back to defaults.
    #[must_use]
    pub fn from_env() -> Self {
        let port = env::var("IRONSTAR_PORT")
            .ok()
            .and_then(|s| {
                s.parse().ok().or_else(|| {
                    tracing::warn!(
                        value = %s,
                        "Invalid IRONSTAR_PORT value, using default"
                    );
                    None
                })
            })
            .unwrap_or(3000);

        let database_url = env::var("IRONSTAR_DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:./data/ironstar.db?mode=rwc".to_string());

        let enable_zenoh = env::var("IRONSTAR_ENABLE_ZENOH")
            .map(|s| !matches!(s.to_lowercase().as_str(), "false" | "0" | "no"))
            .unwrap_or(true);

        let enable_analytics = env::var("IRONSTAR_ENABLE_ANALYTICS")
            .map(|s| !matches!(s.to_lowercase().as_str(), "false" | "0" | "no"))
            .unwrap_or(true);

        let analytics_database_path = env::var("IRONSTAR_ANALYTICS_PATH").ok();

        let analytics_num_conns: usize = env::var("IRONSTAR_ANALYTICS_NUM_CONNS")
            .ok()
            .and_then(|s| {
                s.parse().ok().or_else(|| {
                    tracing::warn!(
                        value = %s,
                        "Invalid IRONSTAR_ANALYTICS_NUM_CONNS value, using default"
                    );
                    None
                })
            })
            .unwrap_or(4);

        let shutdown_timeout_secs: u64 = env::var("IRONSTAR_SHUTDOWN_TIMEOUT_SECS")
            .ok()
            .and_then(|s| {
                s.parse().ok().or_else(|| {
                    tracing::warn!(
                        value = %s,
                        "Invalid IRONSTAR_SHUTDOWN_TIMEOUT_SECS value, using default"
                    );
                    None
                })
            })
            .unwrap_or(30);

        Self {
            port,
            database_url,
            enable_zenoh,
            enable_analytics,
            analytics_database_path,
            analytics_num_conns,
            shutdown_timeout: Duration::from_secs(shutdown_timeout_secs),
        }
    }

    /// Get the socket address to bind the HTTP server to.
    #[must_use]
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], self.port))
    }

    /// Get the database path for directory creation.
    ///
    /// Returns `None` for in-memory databases (`:memory:` or URLs starting with
    /// `sqlite::memory:`).
    #[must_use]
    pub fn database_dir(&self) -> Option<PathBuf> {
        if self.database_url == ":memory:" || self.database_url.starts_with("sqlite::memory:") {
            return None;
        }

        // Handle sqlite:// URL scheme
        let path_str = if let Some(stripped) = self.database_url.strip_prefix("sqlite://") {
            stripped
        } else if let Some(stripped) = self.database_url.strip_prefix("sqlite:") {
            stripped
        } else {
            &self.database_url
        };

        PathBuf::from(path_str).parent().map(PathBuf::from)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 3000,
            database_url: "sqlite:./data/ironstar.db?mode=rwc".to_string(),
            enable_zenoh: true,
            enable_analytics: true,
            analytics_database_path: None,
            analytics_num_conns: 4,
            shutdown_timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.port, 3000);
        assert_eq!(config.database_url, "sqlite:./data/ironstar.db?mode=rwc");
        assert!(config.enable_zenoh);
        assert!(config.enable_analytics);
        assert!(config.analytics_database_path.is_none());
        assert_eq!(config.analytics_num_conns, 4);
        assert_eq!(config.shutdown_timeout, Duration::from_secs(30));
    }

    #[test]
    fn socket_addr_binding() {
        let config = Config {
            port: 8080,
            ..Default::default()
        };
        assert_eq!(config.socket_addr(), SocketAddr::from(([0, 0, 0, 0], 8080)));
    }

    #[test]
    fn database_dir_extraction() {
        // Relative path
        let config = Config {
            database_url: "./data/ironstar.db".to_string(),
            ..Default::default()
        };
        assert_eq!(config.database_dir(), Some(PathBuf::from("./data")));

        // In-memory (no directory)
        let config = Config {
            database_url: ":memory:".to_string(),
            ..Default::default()
        };
        assert_eq!(config.database_dir(), None);

        // sqlite::memory: URL
        let config = Config {
            database_url: "sqlite::memory:".to_string(),
            ..Default::default()
        };
        assert_eq!(config.database_dir(), None);

        // sqlite:// URL scheme
        let config = Config {
            database_url: "sqlite://./data/app.db".to_string(),
            ..Default::default()
        };
        assert_eq!(config.database_dir(), Some(PathBuf::from("./data")));
    }
}
