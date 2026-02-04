//! Shared kernel types for ironstar.
//!
//! This crate contains identity and classification types referenced across
//! multiple bounded contexts (Session, Workspace, Analytics). These types
//! form the shared kernel in DDD terms: a small, stable contract that
//! multiple contexts depend on without coupling to each other.
//!
//! Maps to `spec/SharedKernel/*` in the Idris2 specification.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Unique identifier for a User (Shared Kernel type).
///
/// Wraps a UUID v4, providing type safety. This is the canonical user identity
/// referenced across bounded contexts (Session, Workspace).
///
/// # Design note
///
/// The provider+externalId lookup (OAuth identity mapping) is an infrastructure
/// concern handled by the user_identities table. This domain type uses UUID as
/// the canonical identity reference.
///
/// # Construction
///
/// - `UserId::new()` - Generate a new random ID
/// - `UserId::from_uuid(uuid)` - Wrap an existing UUID (for deserialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct UserId(Uuid);

impl UserId {
    /// Generate a new random UserId.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID as a UserId.
    ///
    /// Use this when deserializing from storage or parsing from input.
    /// For new users, prefer `UserId::new()`.
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

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// OAuth authentication provider.
///
/// GitHub is the primary provider; Google is planned for future extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    /// GitHub OAuth provider (primary).
    GitHub,
    /// Google OAuth/OIDC provider (future).
    Google,
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
            Self::Google => write!(f, "google"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod user_id {
        use super::*;

        #[test]
        fn new_generates_unique_ids() {
            let id1 = UserId::new();
            let id2 = UserId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn from_uuid_roundtrips() {
            let uuid = Uuid::new_v4();
            let id = UserId::from_uuid(uuid);
            assert_eq!(id.into_inner(), uuid);
        }

        #[test]
        fn serializes_as_string() {
            let id = UserId::from_uuid(Uuid::nil());
            let json = serde_json::to_string(&id).unwrap();
            assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
        }
    }

    mod oauth_provider {
        use super::*;

        #[test]
        fn serializes_lowercase() {
            let json = serde_json::to_string(&OAuthProvider::GitHub).unwrap();
            assert_eq!(json, "\"github\"");

            let json = serde_json::to_string(&OAuthProvider::Google).unwrap();
            assert_eq!(json, "\"google\"");
        }

        #[test]
        fn deserializes_lowercase() {
            let provider: OAuthProvider = serde_json::from_str("\"github\"").unwrap();
            assert_eq!(provider, OAuthProvider::GitHub);

            let provider: OAuthProvider = serde_json::from_str("\"google\"").unwrap();
            assert_eq!(provider, OAuthProvider::Google);
        }

        #[test]
        fn display_lowercase() {
            assert_eq!(OAuthProvider::GitHub.to_string(), "github");
            assert_eq!(OAuthProvider::Google.to_string(), "google");
        }
    }
}
