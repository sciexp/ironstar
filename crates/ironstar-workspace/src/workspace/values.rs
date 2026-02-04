//! Value objects with smart constructors for Workspace aggregate.
//!
//! Value objects are immutable, equality-compared by value (not identity),
//! and validated at construction time where applicable.
//!
//! # Types
//!
//! - `WorkspaceId`: UUID wrapper for workspace identity
//! - `WorkspaceName`: Validated workspace name (non-empty, max 255 chars)
//! - `Visibility`: Workspace visibility (Private or Public)

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::errors::WorkspaceError;
#[cfg(test)]
use super::errors::WorkspaceErrorKind;

/// Maximum length for workspace name in characters.
pub const WORKSPACE_NAME_MAX_LENGTH: usize = 255;

/// Unique identifier for a Workspace.
///
/// Wraps a UUID v4, providing type safety to prevent mixing up different
/// ID types (e.g., passing a `UserId` where a `WorkspaceId` is expected).
///
/// # Construction
///
/// - `WorkspaceId::new()` - Generate a new random ID
/// - `WorkspaceId::from_uuid(uuid)` - Wrap an existing UUID (for deserialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct WorkspaceId(Uuid);

impl WorkspaceId {
    /// Generate a new random WorkspaceId.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID as a WorkspaceId.
    ///
    /// Use this when deserializing from storage or parsing from input.
    /// For new workspaces, prefer `WorkspaceId::new()`.
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

impl Default for WorkspaceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validated workspace name.
///
/// Guarantees:
/// - Non-empty (at least one non-whitespace character)
/// - At most [`WORKSPACE_NAME_MAX_LENGTH`] characters
/// - Trimmed of leading/trailing whitespace
///
/// # Example
///
/// ```rust,ignore
/// let name = WorkspaceName::new("  My Workspace  ")?;
/// assert_eq!(name.as_str(), "My Workspace"); // Trimmed
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(try_from = "String", into = "String")]
pub struct WorkspaceName(String);

impl WorkspaceName {
    /// Create a new WorkspaceName, validating and normalizing the input.
    ///
    /// # Errors
    ///
    /// - [`WorkspaceError::InvalidName`] if the trimmed name is empty
    /// - [`WorkspaceError::InvalidName`] if the name exceeds [`WORKSPACE_NAME_MAX_LENGTH`]
    pub fn new(name: impl Into<String>) -> Result<Self, WorkspaceError> {
        let name = name.into();
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(WorkspaceError::invalid_name(
                "workspace name cannot be empty",
            ));
        }

        let char_count = trimmed.chars().count();
        if char_count > WORKSPACE_NAME_MAX_LENGTH {
            return Err(WorkspaceError::invalid_name(format!(
                "workspace name cannot exceed {} characters (got {})",
                WORKSPACE_NAME_MAX_LENGTH, char_count
            )));
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Get the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume self and return the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for WorkspaceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for WorkspaceName {
    type Error = WorkspaceError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<WorkspaceName> for String {
    fn from(name: WorkspaceName) -> Self {
        name.0
    }
}

/// Workspace visibility controls access permissions.
///
/// - `Private`: Visible only to owner (default)
/// - `Public`: Visible to all authenticated users
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Visible only to owner.
    #[default]
    Private,
    /// Visible to all authenticated users.
    Public,
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Private => write!(f, "private"),
            Self::Public => write!(f, "public"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod workspace_id {
        use super::*;

        #[test]
        fn new_generates_unique_ids() {
            let id1 = WorkspaceId::new();
            let id2 = WorkspaceId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn from_uuid_roundtrips() {
            let uuid = Uuid::new_v4();
            let id = WorkspaceId::from_uuid(uuid);
            assert_eq!(id.into_inner(), uuid);
        }

        #[test]
        fn serializes_as_string() {
            let id = WorkspaceId::from_uuid(Uuid::nil());
            let json = serde_json::to_string(&id).unwrap();
            assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
        }
    }

    mod workspace_name {
        use super::*;

        #[test]
        fn accepts_valid_name() {
            let name = WorkspaceName::new("My Workspace").unwrap();
            assert_eq!(name.as_str(), "My Workspace");
        }

        #[test]
        fn trims_whitespace() {
            let name = WorkspaceName::new("  My Workspace  ").unwrap();
            assert_eq!(name.as_str(), "My Workspace");
        }

        #[test]
        fn rejects_empty_string() {
            let result = WorkspaceName::new("");
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                WorkspaceErrorKind::InvalidName(_)
            ));
        }

        #[test]
        fn rejects_whitespace_only() {
            let result = WorkspaceName::new("   \t\n  ");
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                WorkspaceErrorKind::InvalidName(_)
            ));
        }

        #[test]
        fn rejects_too_long_name() {
            let long_name = "a".repeat(WORKSPACE_NAME_MAX_LENGTH + 1);
            let result = WorkspaceName::new(&long_name);
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                WorkspaceErrorKind::InvalidName(_)
            ));
        }

        #[test]
        fn accepts_max_length_name() {
            let max_name = "a".repeat(WORKSPACE_NAME_MAX_LENGTH);
            let result = WorkspaceName::new(&max_name);
            assert!(result.is_ok());
        }

        #[test]
        fn serde_roundtrip_valid() {
            let original = WorkspaceName::new("Test Workspace").unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: WorkspaceName = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn serde_rejects_invalid() {
            let json = r#""""#; // Empty string
            let result: Result<WorkspaceName, _> = serde_json::from_str(json);
            assert!(result.is_err());
        }
    }

    mod visibility {
        use super::*;

        #[test]
        fn default_is_private() {
            assert_eq!(Visibility::default(), Visibility::Private);
        }

        #[test]
        fn serializes_lowercase() {
            let private = serde_json::to_string(&Visibility::Private).unwrap();
            let public = serde_json::to_string(&Visibility::Public).unwrap();
            assert_eq!(private, "\"private\"");
            assert_eq!(public, "\"public\"");
        }

        #[test]
        fn deserializes_lowercase() {
            let private: Visibility = serde_json::from_str("\"private\"").unwrap();
            let public: Visibility = serde_json::from_str("\"public\"").unwrap();
            assert_eq!(private, Visibility::Private);
            assert_eq!(public, Visibility::Public);
        }

        #[test]
        fn display_lowercase() {
            assert_eq!(Visibility::Private.to_string(), "private");
            assert_eq!(Visibility::Public.to_string(), "public");
        }
    }
}
