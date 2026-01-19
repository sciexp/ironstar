//! Value objects with smart constructors for Session aggregate.
//!
//! Value objects are immutable, equality-compared by value (not identity),
//! and validated at construction time where applicable.
//!
//! # Types
//!
//! - `SessionId`: UUID wrapper for session identity
//! - `UserId`: UUID wrapper for user identity (Shared Kernel type)
//! - `OAuthProvider`: Authentication provider enumeration
//! - `SessionMetadata`: Audit trail data captured at session creation

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Unique identifier for a Session.
///
/// Wraps a UUID v4, providing type safety to prevent mixing up different
/// ID types (e.g., passing a `UserId` where a `SessionId` is expected).
///
/// # Construction
///
/// - `SessionId::new()` - Generate a new random ID
/// - `SessionId::from_uuid(uuid)` - Wrap an existing UUID (for deserialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Generate a new random SessionId.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID as a SessionId.
    ///
    /// Use this when deserializing from storage or parsing from input.
    /// For new sessions, prefer `SessionId::new()`.
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

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a User (Shared Kernel type).
///
/// Wraps a UUID v4, providing type safety. This is the canonical user identity
/// referenced across bounded contexts (Session, Workspace).
///
/// # Design Note
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

/// Metadata captured at session creation for security audit trail.
///
/// All fields are optional as they depend on boundary layer context.
/// Populated from HTTP request headers at the boundary.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub struct SessionMetadata {
    /// Client IP address (X-Forwarded-For or direct).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// User-Agent header value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl SessionMetadata {
    /// Create empty metadata.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create metadata with all fields.
    #[must_use]
    pub fn new(ip_address: Option<String>, user_agent: Option<String>) -> Self {
        Self {
            ip_address,
            user_agent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod session_id {
        use super::*;

        #[test]
        fn new_generates_unique_ids() {
            let id1 = SessionId::new();
            let id2 = SessionId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn from_uuid_roundtrips() {
            let uuid = Uuid::new_v4();
            let id = SessionId::from_uuid(uuid);
            assert_eq!(id.into_inner(), uuid);
        }

        #[test]
        fn serializes_as_string() {
            let id = SessionId::from_uuid(Uuid::nil());
            let json = serde_json::to_string(&id).unwrap();
            assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
        }
    }

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

    mod session_metadata {
        use super::*;

        #[test]
        fn empty_creates_default() {
            let meta = SessionMetadata::empty();
            assert_eq!(meta.ip_address, None);
            assert_eq!(meta.user_agent, None);
        }

        #[test]
        fn new_with_fields() {
            let meta = SessionMetadata::new(
                Some("192.168.1.1".to_string()),
                Some("Mozilla/5.0".to_string()),
            );
            assert_eq!(meta.ip_address, Some("192.168.1.1".to_string()));
            assert_eq!(meta.user_agent, Some("Mozilla/5.0".to_string()));
        }

        #[test]
        fn serializes_without_none_fields() {
            let meta = SessionMetadata::new(Some("192.168.1.1".to_string()), None);
            let json = serde_json::to_value(&meta).unwrap();
            assert!(json.get("ip_address").is_some());
            assert!(json.get("user_agent").is_none());
        }
    }
}
