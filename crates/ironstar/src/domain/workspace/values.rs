//! Value objects with smart constructors for Workspace aggregate.
//!
//! Value objects are immutable, equality-compared by value (not identity),
//! and validated at construction time where applicable.
//!
//! # Types
//!
//! - `WorkspaceId`: UUID wrapper for workspace identity

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

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
}
