//! Value objects for the Catalog aggregate.
//!
//! Value objects are immutable, equality-compared by value, and validated
//! at construction time following the "parse, don't validate" principle.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::errors::CatalogError;
#[cfg(test)]
use super::errors::CatalogErrorKind;

/// Maximum length for a catalog URI in characters.
pub const CATALOG_REF_MAX_LENGTH: usize = 1024;

/// Reference to a DuckLake catalog.
///
/// Guarantees:
/// - Non-empty (at least one non-whitespace character)
/// - At most [`CATALOG_REF_MAX_LENGTH`] characters
/// - Trimmed of leading/trailing whitespace
///
/// Example values: `"ducklake://hf/sciexp/fixtures"`, `"ducklake://local/analytics"`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(try_from = "String", into = "String")]
pub struct CatalogRef(String);

impl CatalogRef {
    /// Create a new CatalogRef, validating and normalizing the input.
    ///
    /// # Errors
    ///
    /// - [`CatalogError::EmptyRef`] if the trimmed URI is empty
    /// - [`CatalogError::RefTooLong`] if the URI exceeds [`CATALOG_REF_MAX_LENGTH`]
    pub fn new(uri: impl Into<String>) -> Result<Self, CatalogError> {
        let uri = uri.into();
        let trimmed = uri.trim();

        if trimmed.is_empty() {
            return Err(CatalogError::empty_ref());
        }

        let char_count = trimmed.chars().count();
        if char_count > CATALOG_REF_MAX_LENGTH {
            return Err(CatalogError::ref_too_long(
                CATALOG_REF_MAX_LENGTH,
                char_count,
            ));
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Get the URI as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CatalogRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for CatalogRef {
    type Error = CatalogError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<CatalogRef> for String {
    fn from(r: CatalogRef) -> Self {
        r.0
    }
}

/// Information about a dataset in the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub struct DatasetInfo {
    /// Dataset name.
    pub name: String,
    /// Number of tables in the dataset.
    pub table_count: usize,
    /// Schema version string.
    pub schema_version: String,
}

/// Metadata about a DuckLake catalog.
///
/// Contains the list of datasets and the timestamp of the last refresh.
/// This is populated by the application layer after introspecting the catalog
/// via DuckDB and injected into the `RefreshCatalogMetadata` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub struct CatalogMetadata {
    /// Datasets available in the catalog.
    pub datasets: Vec<DatasetInfo>,
    /// When the metadata was last refreshed.
    pub last_refreshed: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod catalog_ref {
        use super::*;

        #[test]
        fn accepts_valid_uri() {
            let r = CatalogRef::new("ducklake://hf/sciexp/fixtures").unwrap();
            assert_eq!(r.as_str(), "ducklake://hf/sciexp/fixtures");
        }

        #[test]
        fn trims_whitespace() {
            let r = CatalogRef::new("  ducklake://local  ").unwrap();
            assert_eq!(r.as_str(), "ducklake://local");
        }

        #[test]
        fn rejects_empty_string() {
            let result = CatalogRef::new("");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind(), &CatalogErrorKind::EmptyRef);
        }

        #[test]
        fn rejects_whitespace_only() {
            let result = CatalogRef::new("   \t  ");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind(), &CatalogErrorKind::EmptyRef);
        }

        #[test]
        fn rejects_too_long_uri() {
            let long = "a".repeat(CATALOG_REF_MAX_LENGTH + 1);
            let result = CatalogRef::new(&long);
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().kind(),
                &CatalogErrorKind::RefTooLong {
                    max: CATALOG_REF_MAX_LENGTH,
                    actual: CATALOG_REF_MAX_LENGTH + 1,
                }
            );
        }

        #[test]
        fn accepts_max_length_uri() {
            let max = "a".repeat(CATALOG_REF_MAX_LENGTH);
            assert!(CatalogRef::new(&max).is_ok());
        }

        #[test]
        fn serde_roundtrip() {
            let original = CatalogRef::new("ducklake://test").unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: CatalogRef = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn serde_rejects_invalid() {
            let json = r#""""#;
            let result: Result<CatalogRef, _> = serde_json::from_str(json);
            assert!(result.is_err());
        }
    }
}
