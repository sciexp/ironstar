//! Value objects for the SavedQuery aggregate.
//!
//! - `SavedQueryId`: Unique identifier for a saved query (UUID newtype)
//! - `QueryName`: Validated name for a saved query (1-200 chars)

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::common::BoundedString;
use crate::domain::error::{ValidationError, ValidationErrorKind};

/// Maximum length for query names in characters.
pub const QUERY_NAME_MAX_LENGTH: usize = 200;

/// Minimum length for query names in characters.
pub const QUERY_NAME_MIN_LENGTH: usize = 1;

// ============================================================================
// SavedQueryId - Unique identifier for a saved query
// ============================================================================

/// Unique identifier for a saved query.
///
/// Wraps a UUID v4, providing type safety to prevent mixing up different
/// ID types (e.g., passing a `WorkspaceId` where a `SavedQueryId` is expected).
///
/// # Construction
///
/// - `SavedQueryId::new()` - Generate a new random ID
/// - `SavedQueryId::from_uuid(uuid)` - Wrap an existing UUID (for deserialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct SavedQueryId(Uuid);

impl SavedQueryId {
    /// Generate a new random SavedQueryId.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID as a SavedQueryId.
    ///
    /// Use this when deserializing from storage or parsing from input.
    /// For new queries, prefer `SavedQueryId::new()`.
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Extract the inner UUID.
    #[must_use]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl Default for SavedQueryId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SavedQueryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// QueryName - Validated name for a saved query
// ============================================================================

/// Validated query name.
///
/// Guarantees:
/// - Non-empty (at least 1 character)
/// - At most 200 characters
/// - Trimmed of leading/trailing whitespace
///
/// # Example
///
/// ```rust,ignore
/// let name = QueryName::new("Monthly Revenue by Region")?;
/// assert!(!name.as_str().is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(try_from = "String", into = "String")]
pub struct QueryName(BoundedString<QUERY_NAME_MIN_LENGTH, QUERY_NAME_MAX_LENGTH>);

impl QueryName {
    /// Create a new QueryName, validating and normalizing the input.
    ///
    /// # Errors
    ///
    /// - [`ValidationError`] with `TooShort` if the trimmed name is empty
    /// - [`ValidationError`] with `TooLong` if the name exceeds 200 characters
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        let bounded = BoundedString::new(name, "query_name")?;
        Ok(Self(bounded))
    }

    /// Get the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consume self and return the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0.into_inner()
    }
}

impl std::fmt::Display for QueryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for QueryName {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<QueryName> for String {
    fn from(name: QueryName) -> Self {
        name.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod saved_query_id {
        use super::*;

        #[test]
        fn new_generates_unique_ids() {
            let id1 = SavedQueryId::new();
            let id2 = SavedQueryId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn from_uuid_roundtrips() {
            let uuid = Uuid::new_v4();
            let id = SavedQueryId::from_uuid(uuid);
            assert_eq!(id.into_inner(), uuid);
        }

        #[test]
        fn serializes_as_string() {
            let id = SavedQueryId::from_uuid(Uuid::nil());
            let json = serde_json::to_string(&id).unwrap();
            assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
        }

        #[test]
        fn deserializes_from_string() {
            let json = "\"550e8400-e29b-41d4-a716-446655440000\"";
            let id: SavedQueryId = serde_json::from_str(json).unwrap();
            assert_eq!(
                id.into_inner(),
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
            );
        }

        #[test]
        fn copy_semantics() {
            let id = SavedQueryId::new();
            let copied = id;
            assert_eq!(id, copied);
        }
    }

    mod query_name {
        use super::*;

        #[test]
        fn accepts_valid_name() {
            let name = QueryName::new("Monthly Revenue").unwrap();
            assert_eq!(name.as_str(), "Monthly Revenue");
        }

        #[test]
        fn trims_whitespace() {
            let name = QueryName::new("  Revenue Query  ").unwrap();
            assert_eq!(name.as_str(), "Revenue Query");
        }

        #[test]
        fn rejects_empty_string() {
            let result = QueryName::new("");
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                ValidationErrorKind::TooShort { .. }
            ));
        }

        #[test]
        fn rejects_whitespace_only() {
            let result = QueryName::new("   \t\n  ");
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                ValidationErrorKind::TooShort { .. }
            ));
        }

        #[test]
        fn rejects_too_long_name() {
            let long_name = "a".repeat(QUERY_NAME_MAX_LENGTH + 1);
            let result = QueryName::new(&long_name);
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                ValidationErrorKind::TooLong { .. }
            ));
        }

        #[test]
        fn accepts_max_length_name() {
            let max_name = "a".repeat(QUERY_NAME_MAX_LENGTH);
            assert!(QueryName::new(&max_name).is_ok());
        }

        #[test]
        fn serde_roundtrip() {
            let original = QueryName::new("Test Query").unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: QueryName = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn serde_rejects_empty() {
            let json = r#""""#;
            let result: Result<QueryName, _> = serde_json::from_str(json);
            assert!(result.is_err());
        }
    }
}
