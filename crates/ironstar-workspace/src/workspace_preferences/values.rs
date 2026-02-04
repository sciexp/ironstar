//! Value objects for the WorkspacePreferences aggregate.
//!
//! - `CatalogUri`: Validated URI referencing a DuckDB catalog
//! - `LayoutDefaults`: JSON string for workspace layout defaults
//!
//! Catalog existence validation is deferred to the boundary layer;
//! the domain only validates structural constraints (non-empty, max length).
//! JSON validation for LayoutDefaults is likewise deferred to the boundary.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::errors::WorkspacePreferencesError;
#[cfg(test)]
use super::errors::WorkspacePreferencesErrorKind;

/// Maximum length for a catalog URI in characters.
pub const CATALOG_URI_MAX_LENGTH: usize = 512;

/// URI referencing a DuckDB catalog.
///
/// Structural guarantees:
/// - Non-empty (at least one non-whitespace character)
/// - At most [`CATALOG_URI_MAX_LENGTH`] characters
/// - Trimmed of leading/trailing whitespace
///
/// Semantic guarantee (enforced at boundary):
/// - References a valid DuckDB catalog (e.g., `"ducklake:hf://datasets/sciexp"`)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(try_from = "String", into = "String")]
pub struct CatalogUri(String);

impl CatalogUri {
    /// Create a new CatalogUri, validating and normalizing the input.
    ///
    /// # Errors
    ///
    /// - [`WorkspacePreferencesError::EmptyCatalogUri`] if the trimmed URI is empty
    /// - [`WorkspacePreferencesError::CatalogUriTooLong`] if it exceeds [`CATALOG_URI_MAX_LENGTH`]
    pub fn new(uri: impl Into<String>) -> Result<Self, WorkspacePreferencesError> {
        let uri = uri.into();
        let trimmed = uri.trim();

        if trimmed.is_empty() {
            return Err(WorkspacePreferencesError::empty_catalog_uri());
        }

        let char_count = trimmed.chars().count();
        if char_count > CATALOG_URI_MAX_LENGTH {
            return Err(WorkspacePreferencesError::catalog_uri_too_long(
                CATALOG_URI_MAX_LENGTH,
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

impl std::fmt::Display for CatalogUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for CatalogUri {
    type Error = WorkspacePreferencesError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<CatalogUri> for String {
    fn from(uri: CatalogUri) -> Self {
        uri.0
    }
}

/// JSON string representing workspace layout defaults.
///
/// The domain layer treats this as an opaque string. JSON validation
/// is deferred to the boundary layer per Hoffman's Law 7 (work is a
/// side effect).
///
/// Default value is `"{}"` (empty JSON object).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct LayoutDefaults(String);

impl LayoutDefaults {
    /// Create LayoutDefaults from a string.
    ///
    /// No validation is performed in the domain layer; JSON validity
    /// is enforced at the boundary before command construction.
    #[must_use]
    pub fn new(json: impl Into<String>) -> Self {
        Self(json.into())
    }

    /// Get the JSON string as a slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for LayoutDefaults {
    fn default() -> Self {
        Self("{}".to_string())
    }
}

impl std::fmt::Display for LayoutDefaults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<LayoutDefaults> for String {
    fn from(ld: LayoutDefaults) -> Self {
        ld.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod catalog_uri {
        use super::*;

        #[test]
        fn accepts_valid_uri() {
            let uri = CatalogUri::new("ducklake:hf://datasets/sciexp").unwrap();
            assert_eq!(uri.as_str(), "ducklake:hf://datasets/sciexp");
        }

        #[test]
        fn trims_whitespace() {
            let uri = CatalogUri::new("  ducklake:local  ").unwrap();
            assert_eq!(uri.as_str(), "ducklake:local");
        }

        #[test]
        fn rejects_empty_string() {
            let result = CatalogUri::new("");
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                WorkspacePreferencesErrorKind::EmptyCatalogUri
            ));
        }

        #[test]
        fn rejects_whitespace_only() {
            let result = CatalogUri::new("   \t\n  ");
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                WorkspacePreferencesErrorKind::EmptyCatalogUri
            ));
        }

        #[test]
        fn rejects_too_long_uri() {
            let long_uri = "a".repeat(CATALOG_URI_MAX_LENGTH + 1);
            let result = CatalogUri::new(&long_uri);
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                WorkspacePreferencesErrorKind::CatalogUriTooLong { .. }
            ));
        }

        #[test]
        fn accepts_max_length_uri() {
            let max_uri = "a".repeat(CATALOG_URI_MAX_LENGTH);
            assert!(CatalogUri::new(&max_uri).is_ok());
        }

        #[test]
        fn serde_roundtrip() {
            let original = CatalogUri::new("ducklake:test").unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: CatalogUri = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn serde_rejects_empty() {
            let json = r#""""#;
            let result: Result<CatalogUri, _> = serde_json::from_str(json);
            assert!(result.is_err());
        }
    }

    mod layout_defaults {
        use super::*;

        #[test]
        fn default_is_empty_object() {
            assert_eq!(LayoutDefaults::default().as_str(), "{}");
        }

        #[test]
        fn accepts_any_string() {
            let ld = LayoutDefaults::new(r#"{"columns": 3}"#);
            assert_eq!(ld.as_str(), r#"{"columns": 3}"#);
        }

        #[test]
        fn serde_roundtrip() {
            let original = LayoutDefaults::new(r#"{"theme": "dark"}"#);
            let json = serde_json::to_string(&original).unwrap();
            let parsed: LayoutDefaults = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }
    }
}
