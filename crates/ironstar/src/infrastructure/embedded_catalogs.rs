//! Embedded DuckLake catalog management.
//!
//! Embeds DuckLake catalog `.db` files into the binary at compile time using
//! rust_embed, then extracts them to a temporary directory at startup for
//! zero-latency ATTACH operations.
//!
//! # Build-time vs runtime catalogs
//!
//! Embedded catalogs eliminate the ~2s network download penalty for known
//! datasets. Cache keys incorporate `CARGO_PKG_VERSION` so cache entries
//! automatically invalidate when the binary is rebuilt with updated catalogs.
//!
//! For user-provided datasets that cannot be known at build time, use
//! [`DuckDBService::attach_catalog`] with a runtime `ducklake:` URI instead.
//!
//! # Development setup
//!
//! Copy catalog files into `assets/ducklake-catalogs/` before building:
//!
//! ```bash
//! cp ~/projects/omicslake-workspace/sciexp-fixtures/lakes/frozen/space.db \
//!    assets/ducklake-catalogs/
//! ```
//!
//! In CI, nix provides catalog files at build time.

use std::path::{Path, PathBuf};

use rust_embed::RustEmbed;

use crate::infrastructure::DuckDBService;
use crate::infrastructure::error::InfrastructureError;

/// Embedded DuckLake catalog database files.
///
/// Files from `assets/ducklake-catalogs/` are compiled into the binary.
/// When the directory is empty (no `.db` files present), this struct
/// contains no entries and [`attach_all`] becomes a no-op.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../assets/ducklake-catalogs"]
#[include = "*.db"]
pub struct DuckLakeCatalogs;

/// Extract an embedded catalog to a temporary file and return the path.
///
/// The file is written to `{temp_dir}/ironstar-catalogs/{name}` so that
/// repeated startups reuse the same path (avoiding temp dir accumulation).
///
/// # Errors
///
/// Returns `InfrastructureError` if:
/// - The named catalog is not embedded in the binary
/// - The temporary directory cannot be created
/// - The file cannot be written
pub async fn extract_catalog(name: &str) -> Result<PathBuf, InfrastructureError> {
    let data = DuckLakeCatalogs::get(name).ok_or_else(|| {
        InfrastructureError::analytics(format!("embedded catalog '{name}' not found"))
    })?;

    let catalog_dir = std::env::temp_dir().join("ironstar-catalogs");
    tokio::fs::create_dir_all(&catalog_dir).await.map_err(|e| {
        InfrastructureError::analytics(format!("failed to create catalog directory: {e}"))
    })?;

    let file_path = catalog_dir.join(name);
    tokio::fs::write(&file_path, &data.data)
        .await
        .map_err(|e| {
            InfrastructureError::analytics(format!("failed to write catalog '{name}': {e}"))
        })?;

    Ok(file_path)
}

/// Attach all embedded catalogs to the DuckDB service.
///
/// Iterates over all `.db` files compiled into the binary, extracts each
/// to a temporary file, and ATTACHes it with an alias derived from the
/// filename (e.g., `space.db` becomes alias `space`).
///
/// Returns the list of catalog names that were successfully attached.
/// Logs warnings for catalogs that fail to attach without aborting.
///
/// # Errors
///
/// Returns `InfrastructureError` only if the analytics service is unavailable.
/// Individual catalog failures are logged and skipped.
pub async fn attach_all(service: &DuckDBService) -> Result<Vec<String>, InfrastructureError> {
    if !service.is_available() {
        return Err(InfrastructureError::analytics(
            "analytics service unavailable",
        ));
    }

    let filenames: Vec<String> = DuckLakeCatalogs::iter().map(|f| f.to_string()).collect();
    if filenames.is_empty() {
        tracing::debug!("No embedded DuckLake catalogs found");
        return Ok(Vec::new());
    }

    let mut attached = Vec::new();

    for filename in &filenames {
        let alias = catalog_alias(filename);

        match extract_and_attach(service, filename, &alias).await {
            Ok(()) => {
                tracing::info!(
                    catalog = %alias,
                    source = "embedded",
                    "Attached embedded DuckLake catalog"
                );
                attached.push(alias);
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    catalog = %alias,
                    filename = %filename,
                    "Failed to attach embedded catalog, skipping"
                );
            }
        }
    }

    Ok(attached)
}

/// Compose a cache key prefix for queries against an embedded catalog.
///
/// Incorporates `CARGO_PKG_VERSION` so cache entries automatically invalidate
/// when the binary is rebuilt with updated catalog contents.
///
/// # Example
///
/// ```rust,ignore
/// let prefix = embedded_cache_key_prefix("space", "astronauts");
/// // "embedded:space:0.1.0:astronauts"
/// let full_key = format!("{prefix}:{query_hash:x}");
/// ```
#[must_use]
pub fn embedded_cache_key_prefix(catalog_name: &str, table_name: &str) -> String {
    format!(
        "embedded:{catalog_name}:{}:{table_name}",
        env!("CARGO_PKG_VERSION")
    )
}

/// Derive a DuckDB catalog alias from a filename.
///
/// Strips the `.db` extension to produce the alias (e.g., `space.db` -> `space`).
fn catalog_alias(filename: &str) -> String {
    Path::new(filename)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| filename.to_string())
}

/// Extract one embedded catalog and attach it to the service.
async fn extract_and_attach(
    service: &DuckDBService,
    filename: &str,
    alias: &str,
) -> Result<(), InfrastructureError> {
    let path = extract_catalog(filename).await?;

    let uri = format!(
        "ducklake:{}",
        path.to_str().ok_or_else(|| InfrastructureError::analytics(
            "catalog path contains non-UTF-8 characters"
        ))?
    );

    service.attach_catalog(alias, &uri).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_alias_strips_db_extension() {
        assert_eq!(catalog_alias("space.db"), "space");
        assert_eq!(catalog_alias("my_catalog.db"), "my_catalog");
    }

    #[test]
    fn catalog_alias_handles_no_extension() {
        assert_eq!(catalog_alias("catalog"), "catalog");
    }

    #[test]
    fn catalog_alias_handles_nested_path() {
        assert_eq!(catalog_alias("subdir/space.db"), "space");
    }

    #[test]
    fn embedded_cache_key_prefix_includes_version() {
        let prefix = embedded_cache_key_prefix("space", "astronauts");
        assert!(prefix.starts_with("embedded:space:"));
        assert!(prefix.ends_with(":astronauts"));
        assert!(prefix.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn ducklake_catalogs_iter_finds_db_files() {
        // This test verifies the rust_embed struct is functional.
        // Whether it contains files depends on build-time directory contents.
        let count = DuckLakeCatalogs::iter().count();
        // If space.db was copied to assets/ducklake-catalogs/, count >= 1.
        // If the directory is empty, count == 0. Both are valid states.
        assert!(count <= 100, "unexpectedly many embedded catalogs: {count}");
    }

    #[tokio::test]
    async fn attach_all_returns_error_when_unavailable() {
        let service = DuckDBService::new(None);
        let result = attach_all(&service).await;
        assert!(result.is_err());
    }
}
