//! Session aggregate state types.
//!
//! State is derived from events and represents the current status of a session.
//!
//! # State Machine
//!
//! ```text
//!                    ┌───────────────┐
//!     Create ───────►│    Active     │◄────────┐
//!                    └───────┬───────┘         │
//!                            │                 │
//!            ┌───────────────┼───────────────┐ │
//!            │               │               │ │
//!        Expire          Invalidate      Refresh
//!            │               │               │ │
//!            ▼               ▼               │ │
//!     ┌──────────┐    ┌─────────────┐        │ │
//!     │ Expired  │    │ Invalidated │────────┘ │
//!     └──────────┘    └─────────────┘          │
//!                                              │
//!                     (Refresh extends TTL)────┘
//! ```
//!
//! - `NoSession`: Initial state (before any events)
//! - `Active`: Session is valid; expires_at > current time (boundary check)
//! - `Expired`: Session TTL exceeded (terminal)
//! - `Invalidated`: Session explicitly terminated (terminal)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::values::{SessionId, UserId};

/// Session aggregate state (sum type).
///
/// Represents the current state of a session, derived from events.
/// The state machine is: NoSession → Active → (Expired | Invalidated)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SessionState {
    /// Initial state: no session exists.
    #[default]
    NoSession,

    /// Session is active and valid.
    Active {
        /// Session identifier.
        session_id: SessionId,
        /// User who owns this session.
        user_id: UserId,
        /// When the session expires.
        expires_at: DateTime<Utc>,
    },

    /// Session has expired (TTL exceeded).
    Expired {
        /// Session that expired.
        session_id: SessionId,
    },

    /// Session was explicitly invalidated (logout).
    Invalidated {
        /// Session that was invalidated.
        session_id: SessionId,
    },
}

impl SessionState {
    /// Check if the session is in the active state.
    ///
    /// Note: This only checks the state machine position; boundary layer
    /// must additionally verify expires_at > current time.
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active { .. })
    }

    /// Check if the session is terminated (expired or invalidated).
    #[must_use]
    pub fn is_terminated(&self) -> bool {
        matches!(self, Self::Expired { .. } | Self::Invalidated { .. })
    }

    /// Check if no session exists.
    #[must_use]
    pub fn is_no_session(&self) -> bool {
        matches!(self, Self::NoSession)
    }

    /// Extract the SessionId if one exists.
    #[must_use]
    pub fn session_id(&self) -> Option<SessionId> {
        match self {
            Self::NoSession => None,
            Self::Active { session_id, .. }
            | Self::Expired { session_id }
            | Self::Invalidated { session_id } => Some(*session_id),
        }
    }

    /// Extract the UserId if the session is active.
    #[must_use]
    pub fn user_id(&self) -> Option<UserId> {
        match self {
            Self::Active { user_id, .. } => Some(*user_id),
            _ => None,
        }
    }

    /// Extract the expiration time if the session is active.
    #[must_use]
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Active { expires_at, .. } => Some(*expires_at),
            _ => None,
        }
    }
}

/// Lifecycle status of a session (simple enum for projections).
///
/// Maps to SessionState but as a simple enum without associated data,
/// useful for serialization and projection queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Initial state (before any events).
    #[default]
    NoSession,
    /// Session is active and valid.
    Active,
    /// Session has expired.
    Expired,
    /// Session was explicitly invalidated.
    Invalidated,
}

impl From<&SessionState> for SessionStatus {
    fn from(state: &SessionState) -> Self {
        match state {
            SessionState::NoSession => Self::NoSession,
            SessionState::Active { .. } => Self::Active,
            SessionState::Expired { .. } => Self::Expired,
            SessionState::Invalidated { .. } => Self::Invalidated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_session_id() -> SessionId {
        SessionId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_user_id() -> UserId {
        UserId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_expires() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T22:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn default_is_no_session() {
        assert_eq!(SessionState::default(), SessionState::NoSession);
    }

    #[test]
    fn is_active_only_for_active_state() {
        let no_session = SessionState::NoSession;
        let active = SessionState::Active {
            session_id: sample_session_id(),
            user_id: sample_user_id(),
            expires_at: sample_expires(),
        };
        let expired = SessionState::Expired {
            session_id: sample_session_id(),
        };
        let invalidated = SessionState::Invalidated {
            session_id: sample_session_id(),
        };

        assert!(!no_session.is_active());
        assert!(active.is_active());
        assert!(!expired.is_active());
        assert!(!invalidated.is_active());
    }

    #[test]
    fn is_terminated_for_expired_and_invalidated() {
        let no_session = SessionState::NoSession;
        let active = SessionState::Active {
            session_id: sample_session_id(),
            user_id: sample_user_id(),
            expires_at: sample_expires(),
        };
        let expired = SessionState::Expired {
            session_id: sample_session_id(),
        };
        let invalidated = SessionState::Invalidated {
            session_id: sample_session_id(),
        };

        assert!(!no_session.is_terminated());
        assert!(!active.is_terminated());
        assert!(expired.is_terminated());
        assert!(invalidated.is_terminated());
    }

    #[test]
    fn session_id_extraction() {
        let no_session = SessionState::NoSession;
        let active = SessionState::Active {
            session_id: sample_session_id(),
            user_id: sample_user_id(),
            expires_at: sample_expires(),
        };

        assert!(no_session.session_id().is_none());
        assert_eq!(active.session_id(), Some(sample_session_id()));
    }

    #[test]
    fn user_id_only_from_active() {
        let no_session = SessionState::NoSession;
        let active = SessionState::Active {
            session_id: sample_session_id(),
            user_id: sample_user_id(),
            expires_at: sample_expires(),
        };
        let expired = SessionState::Expired {
            session_id: sample_session_id(),
        };

        assert!(no_session.user_id().is_none());
        assert_eq!(active.user_id(), Some(sample_user_id()));
        assert!(expired.user_id().is_none());
    }

    #[test]
    fn expires_at_only_from_active() {
        let active = SessionState::Active {
            session_id: sample_session_id(),
            user_id: sample_user_id(),
            expires_at: sample_expires(),
        };
        let expired = SessionState::Expired {
            session_id: sample_session_id(),
        };

        assert_eq!(active.expires_at(), Some(sample_expires()));
        assert!(expired.expires_at().is_none());
    }

    #[test]
    fn session_status_from_state() {
        let no_session = SessionState::NoSession;
        let active = SessionState::Active {
            session_id: sample_session_id(),
            user_id: sample_user_id(),
            expires_at: sample_expires(),
        };
        let expired = SessionState::Expired {
            session_id: sample_session_id(),
        };
        let invalidated = SessionState::Invalidated {
            session_id: sample_session_id(),
        };

        assert_eq!(SessionStatus::from(&no_session), SessionStatus::NoSession);
        assert_eq!(SessionStatus::from(&active), SessionStatus::Active);
        assert_eq!(SessionStatus::from(&expired), SessionStatus::Expired);
        assert_eq!(
            SessionStatus::from(&invalidated),
            SessionStatus::Invalidated
        );
    }
}
