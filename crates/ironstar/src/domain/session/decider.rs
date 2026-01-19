//! Pure Session Decider implementing fmodel-rust patterns.
//!
//! The Decider is the core decision-making component that embodies the
//! state machine from `spec/Session/Session.idr`. It is a pure function with
//! no side effects: all I/O (OAuth callbacks, timestamps) happens at boundaries.
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
//!     │ Expired  │    │ Invalidated │        │ │
//!     └──────────┘    └─────────────┘        │ │
//!         (terminal)      (terminal)         │ │
//!                                            │ │
//!                     (Refresh extends TTL)──┘ │
//!                     (From Active only)───────┘
//! ```
//!
//! # Invariants
//!
//! - Cannot create session when one is already active
//! - Cannot refresh/invalidate a non-existent or terminated session
//! - Session expiration is enforced at boundary, not in decide
//! - All timestamps are boundary-injected (commands carry timestamps)

use fmodel_rust::decider::Decider;

use super::commands::SessionCommand;
use super::errors::SessionError;
use super::events::SessionEvent;
use super::state::SessionState;

/// Type alias for the Session Decider.
///
/// Uses `SessionState` directly as the state type. NoSession variant
/// represents the non-existent state.
pub type SessionDecider<'a> = Decider<'a, SessionCommand, SessionState, SessionEvent, SessionError>;

/// Factory function creating a pure Session Decider.
///
/// The decider embodies the state machine from `spec/Session/Session.idr`:
/// - NoSession → Active (Create)
/// - Active → Active (Refresh - extends TTL)
/// - Active → Invalidated (Invalidate)
/// - Active → Expired (boundary-generated Expired event)
///
/// # Example
///
/// ```rust,ignore
/// use fmodel_rust::decider::EventComputation;
/// use ironstar::domain::session::{session_decider, SessionCommand, SessionId, UserId};
/// use chrono::Utc;
///
/// let decider = session_decider();
/// let events = decider.compute_new_events(
///     &[],
///     &SessionCommand::Create { ... }
/// );
/// ```
pub fn session_decider<'a>() -> SessionDecider<'a> {
    Decider {
        decide: Box::new(|command, state| decide(command, state)),
        evolve: Box::new(|state, event| evolve(state, event)),
        initial_state: Box::new(|| SessionState::NoSession),
    }
}

/// Pure decide function: (Command, State) -> Result<Vec<Event>, Error>
///
/// This function is the heart of the domain logic. It validates commands
/// against current state and returns events or errors. No side effects.
fn decide(
    command: &SessionCommand,
    state: &SessionState,
) -> Result<Vec<SessionEvent>, SessionError> {
    match (command, state) {
        // Create: NoSession → Active
        (
            SessionCommand::Create {
                session_id,
                user_id,
                provider,
                created_at,
                expires_at,
                metadata,
            },
            SessionState::NoSession,
        ) => Ok(vec![SessionEvent::Created {
            session_id: *session_id,
            user_id: *user_id,
            provider: *provider,
            created_at: *created_at,
            expires_at: *expires_at,
            metadata: metadata.clone(),
        }]),

        // Create when already active
        (SessionCommand::Create { .. }, SessionState::Active { .. }) => {
            Err(SessionError::already_active())
        }

        // Create when expired
        (SessionCommand::Create { .. }, SessionState::Expired { .. }) => {
            Err(SessionError::session_expired())
        }

        // Create when invalidated
        (SessionCommand::Create { .. }, SessionState::Invalidated { .. }) => {
            Err(SessionError::session_invalidated())
        }

        // Refresh: Active → Active (with new expires_at)
        (
            SessionCommand::Refresh {
                session_id,
                refreshed_at,
                new_expires_at,
            },
            SessionState::Active {
                session_id: active_sid,
                ..
            },
        ) => {
            if session_id == active_sid {
                Ok(vec![SessionEvent::Refreshed {
                    session_id: *session_id,
                    refreshed_at: *refreshed_at,
                    new_expires_at: *new_expires_at,
                }])
            } else {
                // Session ID mismatch - treat as no active session
                Err(SessionError::no_active_session())
            }
        }

        // Refresh when no session
        (SessionCommand::Refresh { .. }, SessionState::NoSession) => {
            Err(SessionError::no_active_session())
        }

        // Refresh when expired
        (SessionCommand::Refresh { .. }, SessionState::Expired { .. }) => {
            Err(SessionError::session_expired())
        }

        // Refresh when invalidated
        (SessionCommand::Refresh { .. }, SessionState::Invalidated { .. }) => {
            Err(SessionError::session_invalidated())
        }

        // Invalidate: Active → Invalidated
        (
            SessionCommand::Invalidate {
                session_id,
                invalidated_at,
            },
            SessionState::Active {
                session_id: active_sid,
                ..
            },
        ) => {
            if session_id == active_sid {
                Ok(vec![SessionEvent::Invalidated {
                    session_id: *session_id,
                    invalidated_at: *invalidated_at,
                }])
            } else {
                // Session ID mismatch - treat as no active session
                Err(SessionError::no_active_session())
            }
        }

        // Invalidate when no session
        (SessionCommand::Invalidate { .. }, SessionState::NoSession) => {
            Err(SessionError::no_active_session())
        }

        // Invalidate when expired
        (SessionCommand::Invalidate { .. }, SessionState::Expired { .. }) => {
            Err(SessionError::session_expired())
        }

        // Invalidate when already invalidated
        (SessionCommand::Invalidate { .. }, SessionState::Invalidated { .. }) => {
            Err(SessionError::session_invalidated())
        }
    }
}

/// Pure evolve function: (State, Event) -> State
///
/// This function applies an event to produce new state. It must be
/// deterministic and total (handle all event variants).
fn evolve(state: &SessionState, event: &SessionEvent) -> SessionState {
    match event {
        // Created: NoSession → Active
        SessionEvent::Created {
            session_id,
            user_id,
            expires_at,
            ..
        } => SessionState::Active {
            session_id: *session_id,
            user_id: *user_id,
            expires_at: *expires_at,
        },

        // Refreshed: Active → Active (with new expires_at)
        SessionEvent::Refreshed {
            session_id,
            new_expires_at,
            ..
        } => match state {
            SessionState::Active { user_id, .. } => SessionState::Active {
                session_id: *session_id,
                user_id: *user_id,
                expires_at: *new_expires_at,
            },
            // Defensive: shouldn't happen if decide is correct
            other => other.clone(),
        },

        // Invalidated: Active → Invalidated
        SessionEvent::Invalidated { session_id, .. } => SessionState::Invalidated {
            session_id: *session_id,
        },

        // Expired: Active → Expired (boundary-generated)
        SessionEvent::Expired { session_id, .. } => SessionState::Expired {
            session_id: *session_id,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use fmodel_rust::specification::DeciderTestSpecification;

    use super::super::values::{OAuthProvider, SessionId, SessionMetadata, UserId};

    fn sample_session_id() -> SessionId {
        SessionId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_user_id() -> UserId {
        UserId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn sample_expires() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T22:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn sample_new_expires() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-16T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    // --- Create transitions ---

    #[test]
    fn create_from_no_session_succeeds() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![])
            .when(SessionCommand::Create {
                session_id: sid,
                user_id: uid,
                provider: OAuthProvider::GitHub,
                created_at: ts,
                expires_at: exp,
                metadata: SessionMetadata::empty(),
            })
            .then(vec![SessionEvent::Created {
                session_id: sid,
                user_id: uid,
                provider: OAuthProvider::GitHub,
                created_at: ts,
                expires_at: exp,
                metadata: SessionMetadata::empty(),
            }]);
    }

    #[test]
    fn create_when_already_active_fails() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![SessionEvent::Created {
                session_id: sid,
                user_id: uid,
                provider: OAuthProvider::GitHub,
                created_at: ts,
                expires_at: exp,
                metadata: SessionMetadata::empty(),
            }])
            .when(SessionCommand::Create {
                session_id: SessionId::new(),
                user_id: uid,
                provider: OAuthProvider::GitHub,
                created_at: ts,
                expires_at: exp,
                metadata: SessionMetadata::empty(),
            })
            .then_error(SessionError::already_active());
    }

    #[test]
    fn create_when_expired_fails() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![
                SessionEvent::Created {
                    session_id: sid,
                    user_id: uid,
                    provider: OAuthProvider::GitHub,
                    created_at: ts,
                    expires_at: exp,
                    metadata: SessionMetadata::empty(),
                },
                SessionEvent::Expired {
                    session_id: sid,
                    expired_at: exp,
                },
            ])
            .when(SessionCommand::Create {
                session_id: SessionId::new(),
                user_id: uid,
                provider: OAuthProvider::GitHub,
                created_at: ts,
                expires_at: exp,
                metadata: SessionMetadata::empty(),
            })
            .then_error(SessionError::session_expired());
    }

    // --- Refresh transitions ---

    #[test]
    fn refresh_active_succeeds() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();
        let new_exp = sample_new_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![SessionEvent::Created {
                session_id: sid,
                user_id: uid,
                provider: OAuthProvider::GitHub,
                created_at: ts,
                expires_at: exp,
                metadata: SessionMetadata::empty(),
            }])
            .when(SessionCommand::Refresh {
                session_id: sid,
                refreshed_at: ts,
                new_expires_at: new_exp,
            })
            .then(vec![SessionEvent::Refreshed {
                session_id: sid,
                refreshed_at: ts,
                new_expires_at: new_exp,
            }]);
    }

    #[test]
    fn refresh_no_session_fails() {
        let sid = sample_session_id();
        let ts = sample_time();
        let new_exp = sample_new_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![])
            .when(SessionCommand::Refresh {
                session_id: sid,
                refreshed_at: ts,
                new_expires_at: new_exp,
            })
            .then_error(SessionError::no_active_session());
    }

    #[test]
    fn refresh_wrong_session_id_fails() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();
        let new_exp = sample_new_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![SessionEvent::Created {
                session_id: sid,
                user_id: uid,
                provider: OAuthProvider::GitHub,
                created_at: ts,
                expires_at: exp,
                metadata: SessionMetadata::empty(),
            }])
            .when(SessionCommand::Refresh {
                session_id: SessionId::new(), // Different session ID
                refreshed_at: ts,
                new_expires_at: new_exp,
            })
            .then_error(SessionError::no_active_session());
    }

    #[test]
    fn refresh_expired_fails() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();
        let new_exp = sample_new_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![
                SessionEvent::Created {
                    session_id: sid,
                    user_id: uid,
                    provider: OAuthProvider::GitHub,
                    created_at: ts,
                    expires_at: exp,
                    metadata: SessionMetadata::empty(),
                },
                SessionEvent::Expired {
                    session_id: sid,
                    expired_at: exp,
                },
            ])
            .when(SessionCommand::Refresh {
                session_id: sid,
                refreshed_at: ts,
                new_expires_at: new_exp,
            })
            .then_error(SessionError::session_expired());
    }

    #[test]
    fn refresh_invalidated_fails() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();
        let new_exp = sample_new_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![
                SessionEvent::Created {
                    session_id: sid,
                    user_id: uid,
                    provider: OAuthProvider::GitHub,
                    created_at: ts,
                    expires_at: exp,
                    metadata: SessionMetadata::empty(),
                },
                SessionEvent::Invalidated {
                    session_id: sid,
                    invalidated_at: ts,
                },
            ])
            .when(SessionCommand::Refresh {
                session_id: sid,
                refreshed_at: ts,
                new_expires_at: new_exp,
            })
            .then_error(SessionError::session_invalidated());
    }

    // --- Invalidate transitions ---

    #[test]
    fn invalidate_active_succeeds() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![SessionEvent::Created {
                session_id: sid,
                user_id: uid,
                provider: OAuthProvider::GitHub,
                created_at: ts,
                expires_at: exp,
                metadata: SessionMetadata::empty(),
            }])
            .when(SessionCommand::Invalidate {
                session_id: sid,
                invalidated_at: ts,
            })
            .then(vec![SessionEvent::Invalidated {
                session_id: sid,
                invalidated_at: ts,
            }]);
    }

    #[test]
    fn invalidate_no_session_fails() {
        let sid = sample_session_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![])
            .when(SessionCommand::Invalidate {
                session_id: sid,
                invalidated_at: ts,
            })
            .then_error(SessionError::no_active_session());
    }

    #[test]
    fn invalidate_wrong_session_id_fails() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![SessionEvent::Created {
                session_id: sid,
                user_id: uid,
                provider: OAuthProvider::GitHub,
                created_at: ts,
                expires_at: exp,
                metadata: SessionMetadata::empty(),
            }])
            .when(SessionCommand::Invalidate {
                session_id: SessionId::new(), // Different session ID
                invalidated_at: ts,
            })
            .then_error(SessionError::no_active_session());
    }

    #[test]
    fn invalidate_expired_fails() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![
                SessionEvent::Created {
                    session_id: sid,
                    user_id: uid,
                    provider: OAuthProvider::GitHub,
                    created_at: ts,
                    expires_at: exp,
                    metadata: SessionMetadata::empty(),
                },
                SessionEvent::Expired {
                    session_id: sid,
                    expired_at: exp,
                },
            ])
            .when(SessionCommand::Invalidate {
                session_id: sid,
                invalidated_at: ts,
            })
            .then_error(SessionError::session_expired());
    }

    #[test]
    fn invalidate_already_invalidated_fails() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();

        DeciderTestSpecification::default()
            .for_decider(session_decider())
            .given(vec![
                SessionEvent::Created {
                    session_id: sid,
                    user_id: uid,
                    provider: OAuthProvider::GitHub,
                    created_at: ts,
                    expires_at: exp,
                    metadata: SessionMetadata::empty(),
                },
                SessionEvent::Invalidated {
                    session_id: sid,
                    invalidated_at: ts,
                },
            ])
            .when(SessionCommand::Invalidate {
                session_id: sid,
                invalidated_at: ts,
            })
            .then_error(SessionError::session_invalidated());
    }

    // --- Full lifecycle ---

    #[test]
    fn full_lifecycle_create_refresh_invalidate() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let ts = sample_time();
        let exp = sample_expires();
        let new_exp = sample_new_expires();

        // Create
        let events = decide(
            &SessionCommand::Create {
                session_id: sid,
                user_id: uid,
                provider: OAuthProvider::GitHub,
                created_at: ts,
                expires_at: exp,
                metadata: SessionMetadata::empty(),
            },
            &SessionState::NoSession,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        // Apply create
        let state = evolve(&SessionState::NoSession, &events[0]);
        assert!(state.is_active());

        // Refresh
        let events = decide(
            &SessionCommand::Refresh {
                session_id: sid,
                refreshed_at: ts,
                new_expires_at: new_exp,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        // Apply refresh
        let state = evolve(&state, &events[0]);
        assert!(state.is_active());
        assert_eq!(state.expires_at(), Some(new_exp));

        // Invalidate
        let events = decide(
            &SessionCommand::Invalidate {
                session_id: sid,
                invalidated_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        // Apply invalidate
        let state = evolve(&state, &events[0]);
        assert!(state.is_terminated());
        assert!(!state.is_active());

        // Further operations fail
        assert!(
            decide(
                &SessionCommand::Refresh {
                    session_id: sid,
                    refreshed_at: ts,
                    new_expires_at: new_exp,
                },
                &state,
            )
            .is_err()
        );
    }

    #[test]
    fn evolve_expired_event_transitions_to_expired() {
        let sid = sample_session_id();
        let uid = sample_user_id();
        let exp = sample_expires();

        let active_state = SessionState::Active {
            session_id: sid,
            user_id: uid,
            expires_at: exp,
        };

        let expired_event = SessionEvent::Expired {
            session_id: sid,
            expired_at: exp,
        };

        let state = evolve(&active_state, &expired_event);
        assert!(matches!(state, SessionState::Expired { .. }));
    }
}
