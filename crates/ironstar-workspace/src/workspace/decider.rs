//! Pure Workspace Decider implementing fmodel-rust patterns.
//!
//! The Decider is the core decision-making component that embodies the
//! state machine from `spec/Workspace/WorkspaceAggregate.idr`. It is a pure function with
//! no side effects: all I/O (timestamps, persistence) happens at boundaries.
//!
//! # State Machine
//!
//! ```text
//!                    ┌──────────────┐
//!     Create ───────►│    Active    │
//!                    └──────┬───────┘
//!                           │
//!            ┌──────────────┼──────────────┐
//!            │              │              │
//!         Rename     SetVisibility    (future: Delete)
//!            │              │              │
//!            └──────────────┴──────────────┘
//!                           │
//!                           ▼
//!                    ┌──────────────┐
//!                    │    Active    │ (same state, updated fields)
//!                    └──────────────┘
//! ```
//!
//! # Idempotency
//!
//! Operations that would result in the same state return `Ok(vec![])`:
//! - Rename with the same name
//! - SetVisibility with the same visibility

use ironstar_core::Decider;

use super::commands::WorkspaceCommand;
use super::errors::WorkspaceError;
use super::events::WorkspaceEvent;
use super::state::{WorkspaceState, WorkspaceStatus};
use super::values::WorkspaceName;

/// Type alias for the Workspace Decider.
///
/// The state is `WorkspaceState` directly, with `WorkspaceStatus::NotCreated`
/// representing the non-existent state.
pub type WorkspaceDecider<'a> =
    Decider<'a, WorkspaceCommand, WorkspaceState, WorkspaceEvent, WorkspaceError>;

/// Factory function creating a pure Workspace Decider.
///
/// The decider embodies the state machine from `spec/Workspace/WorkspaceAggregate.idr`:
/// - NotCreated → Active (Create)
/// - Active → Active (Rename, SetVisibility)
/// - Idempotent operations return `Ok(vec![])` when already in target state
/// - Precondition violations return `Err(WorkspaceError::X)`
///
/// # Example
///
/// ```rust,ignore
/// use fmodel_rust::decider::EventComputation;
/// use ironstar::domain::workspace::{workspace_decider, WorkspaceCommand, WorkspaceId};
/// use chrono::Utc;
///
/// let decider = workspace_decider();
/// let id = WorkspaceId::new();
/// let now = Utc::now();
///
/// let events = decider.compute_new_events(
///     &[],
///     &WorkspaceCommand::Create { workspace_id: id, name: "My Workspace".into(), ... }
/// );
/// ```
pub fn workspace_decider<'a>() -> WorkspaceDecider<'a> {
    Decider {
        decide: Box::new(decide),
        evolve: Box::new(evolve),
        initial_state: Box::new(WorkspaceState::default),
    }
}

/// Pure decide function: (Command, State) -> Result<Vec<Event>, Error>
///
/// This function is the heart of the domain logic. It validates commands
/// against current state and returns events or errors. No side effects.
fn decide(
    command: &WorkspaceCommand,
    state: &WorkspaceState,
) -> Result<Vec<WorkspaceEvent>, WorkspaceError> {
    match (command, state.status) {
        // Create: NotCreated → Active
        (
            WorkspaceCommand::Create {
                workspace_id,
                name,
                owner_id,
                visibility,
                created_at,
            },
            WorkspaceStatus::NotCreated,
        ) => {
            let validated_name = WorkspaceName::new(name.clone())?;
            Ok(vec![WorkspaceEvent::Created {
                workspace_id: *workspace_id,
                name: validated_name,
                owner_id: *owner_id,
                visibility: *visibility,
                created_at: *created_at,
            }])
        }

        // Create when already exists
        (WorkspaceCommand::Create { .. }, WorkspaceStatus::Active) => {
            Err(WorkspaceError::already_exists())
        }

        // Rename: Active → Active (idempotent if same name)
        (
            WorkspaceCommand::Rename {
                workspace_id,
                new_name,
                renamed_at,
            },
            WorkspaceStatus::Active,
        ) => {
            let validated_name = WorkspaceName::new(new_name.clone())?;

            // Idempotent: same name returns empty events
            if let Some(current_name) = &state.name {
                if current_name == &validated_name {
                    return Ok(vec![]);
                }

                Ok(vec![WorkspaceEvent::Renamed {
                    workspace_id: *workspace_id,
                    old_name: current_name.clone(),
                    new_name: validated_name,
                    renamed_at: *renamed_at,
                }])
            } else {
                // Should not happen if state machine is correct
                Err(WorkspaceError::not_found())
            }
        }

        // Rename when not created
        (WorkspaceCommand::Rename { .. }, WorkspaceStatus::NotCreated) => {
            Err(WorkspaceError::not_found())
        }

        // SetVisibility: Active → Active (idempotent if same visibility)
        (
            WorkspaceCommand::SetVisibility {
                workspace_id,
                visibility,
                changed_at,
            },
            WorkspaceStatus::Active,
        ) => {
            // Idempotent: same visibility returns empty events
            if let Some(current_visibility) = &state.visibility {
                if current_visibility == visibility {
                    return Ok(vec![]);
                }

                Ok(vec![WorkspaceEvent::VisibilityChanged {
                    workspace_id: *workspace_id,
                    old_visibility: *current_visibility,
                    new_visibility: *visibility,
                    changed_at: *changed_at,
                }])
            } else {
                // Should not happen if state machine is correct
                Err(WorkspaceError::not_found())
            }
        }

        // SetVisibility when not created
        (WorkspaceCommand::SetVisibility { .. }, WorkspaceStatus::NotCreated) => {
            Err(WorkspaceError::not_found())
        }
    }
}

/// Pure evolve function: (State, Event) -> State
///
/// This function applies an event to produce new state. It must be
/// deterministic and total (handle all event variants).
fn evolve(state: &WorkspaceState, event: &WorkspaceEvent) -> WorkspaceState {
    match event {
        // Created: NotCreated → Active
        WorkspaceEvent::Created {
            workspace_id,
            name,
            owner_id,
            visibility,
            created_at,
        } => WorkspaceState {
            id: Some(*workspace_id),
            name: Some(name.clone()),
            owner_id: Some(*owner_id),
            visibility: Some(*visibility),
            created_at: Some(*created_at),
            status: WorkspaceStatus::Active,
        },

        // Renamed: Active → Active (with new name)
        WorkspaceEvent::Renamed { new_name, .. } => WorkspaceState {
            name: Some(new_name.clone()),
            ..state.clone()
        },

        // VisibilityChanged: Active → Active (with new visibility)
        WorkspaceEvent::VisibilityChanged { new_visibility, .. } => WorkspaceState {
            visibility: Some(*new_visibility),
            ..state.clone()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use ironstar_core::DeciderTestSpecification;

    use super::super::values::{Visibility, WorkspaceId};
    use ironstar_shared_kernel::UserId;

    fn sample_workspace_id() -> WorkspaceId {
        WorkspaceId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_user_id() -> UserId {
        UserId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn sample_name() -> WorkspaceName {
        WorkspaceName::new("My Workspace").unwrap()
    }

    // --- Create transitions ---

    #[test]
    fn create_from_not_created_succeeds() {
        let ws_id = sample_workspace_id();
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_decider())
            .given(vec![])
            .when(WorkspaceCommand::Create {
                workspace_id: ws_id,
                name: "My Workspace".to_string(),
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            })
            .then(vec![WorkspaceEvent::Created {
                workspace_id: ws_id,
                name: sample_name(),
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            }]);
    }

    #[test]
    fn create_when_already_exists_fails() {
        let ws_id = sample_workspace_id();
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_decider())
            .given(vec![WorkspaceEvent::Created {
                workspace_id: ws_id,
                name: sample_name(),
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            }])
            .when(WorkspaceCommand::Create {
                workspace_id: ws_id,
                name: "Another Workspace".to_string(),
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            })
            .then_error(WorkspaceError::already_exists());
    }

    #[test]
    fn create_validates_name() {
        let ws_id = sample_workspace_id();
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_decider())
            .given(vec![])
            .when(WorkspaceCommand::Create {
                workspace_id: ws_id,
                name: "   ".to_string(), // Empty after trim
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            })
            .then_error(WorkspaceError::invalid_name(
                "workspace name cannot be empty",
            ));
    }

    // --- Rename transitions ---

    #[test]
    fn rename_active_succeeds() {
        let ws_id = sample_workspace_id();
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_decider())
            .given(vec![WorkspaceEvent::Created {
                workspace_id: ws_id,
                name: sample_name(),
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            }])
            .when(WorkspaceCommand::Rename {
                workspace_id: ws_id,
                new_name: "New Name".to_string(),
                renamed_at: ts,
            })
            .then(vec![WorkspaceEvent::Renamed {
                workspace_id: ws_id,
                old_name: sample_name(),
                new_name: WorkspaceName::new("New Name").unwrap(),
                renamed_at: ts,
            }]);
    }

    #[test]
    fn rename_same_name_is_idempotent() {
        let ws_id = sample_workspace_id();
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_decider())
            .given(vec![WorkspaceEvent::Created {
                workspace_id: ws_id,
                name: sample_name(),
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            }])
            .when(WorkspaceCommand::Rename {
                workspace_id: ws_id,
                new_name: "My Workspace".to_string(), // Same name
                renamed_at: ts,
            })
            .then(vec![]); // Idempotent: no events
    }

    #[test]
    fn rename_not_created_fails() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_decider())
            .given(vec![])
            .when(WorkspaceCommand::Rename {
                workspace_id: ws_id,
                new_name: "New Name".to_string(),
                renamed_at: ts,
            })
            .then_error(WorkspaceError::not_found());
    }

    #[test]
    fn rename_validates_new_name() {
        let ws_id = sample_workspace_id();
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_decider())
            .given(vec![WorkspaceEvent::Created {
                workspace_id: ws_id,
                name: sample_name(),
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            }])
            .when(WorkspaceCommand::Rename {
                workspace_id: ws_id,
                new_name: "".to_string(), // Invalid
                renamed_at: ts,
            })
            .then_error(WorkspaceError::invalid_name(
                "workspace name cannot be empty",
            ));
    }

    // --- SetVisibility transitions ---

    #[test]
    fn set_visibility_active_succeeds() {
        let ws_id = sample_workspace_id();
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_decider())
            .given(vec![WorkspaceEvent::Created {
                workspace_id: ws_id,
                name: sample_name(),
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            }])
            .when(WorkspaceCommand::SetVisibility {
                workspace_id: ws_id,
                visibility: Visibility::Public,
                changed_at: ts,
            })
            .then(vec![WorkspaceEvent::VisibilityChanged {
                workspace_id: ws_id,
                old_visibility: Visibility::Private,
                new_visibility: Visibility::Public,
                changed_at: ts,
            }]);
    }

    #[test]
    fn set_visibility_same_is_idempotent() {
        let ws_id = sample_workspace_id();
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_decider())
            .given(vec![WorkspaceEvent::Created {
                workspace_id: ws_id,
                name: sample_name(),
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            }])
            .when(WorkspaceCommand::SetVisibility {
                workspace_id: ws_id,
                visibility: Visibility::Private, // Same visibility
                changed_at: ts,
            })
            .then(vec![]); // Idempotent: no events
    }

    #[test]
    fn set_visibility_not_created_fails() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_decider())
            .given(vec![])
            .when(WorkspaceCommand::SetVisibility {
                workspace_id: ws_id,
                visibility: Visibility::Public,
                changed_at: ts,
            })
            .then_error(WorkspaceError::not_found());
    }

    // --- Full lifecycle ---

    #[test]
    fn full_lifecycle_create_rename_set_visibility() {
        let ws_id = sample_workspace_id();
        let user_id = sample_user_id();
        let ts = sample_time();

        // Create
        let events = decide(
            &WorkspaceCommand::Create {
                workspace_id: ws_id,
                name: "Initial Name".to_string(),
                owner_id: user_id,
                visibility: Visibility::Private,
                created_at: ts,
            },
            &WorkspaceState::default(),
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        // Apply create
        let state = evolve(&WorkspaceState::default(), &events[0]);
        assert!(state.is_active());
        assert_eq!(state.name.as_ref().unwrap().as_str(), "Initial Name");

        // Rename
        let events = decide(
            &WorkspaceCommand::Rename {
                workspace_id: ws_id,
                new_name: "New Name".to_string(),
                renamed_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        // Apply rename
        let state = evolve(&state, &events[0]);
        assert_eq!(state.name.as_ref().unwrap().as_str(), "New Name");

        // SetVisibility
        let events = decide(
            &WorkspaceCommand::SetVisibility {
                workspace_id: ws_id,
                visibility: Visibility::Public,
                changed_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        // Apply visibility change
        let state = evolve(&state, &events[0]);
        assert_eq!(state.visibility, Some(Visibility::Public));

        // Idempotent operations
        let events = decide(
            &WorkspaceCommand::Rename {
                workspace_id: ws_id,
                new_name: "New Name".to_string(), // Same name
                renamed_at: ts,
            },
            &state,
        )
        .unwrap();
        assert!(events.is_empty()); // Idempotent

        let events = decide(
            &WorkspaceCommand::SetVisibility {
                workspace_id: ws_id,
                visibility: Visibility::Public, // Same visibility
                changed_at: ts,
            },
            &state,
        )
        .unwrap();
        assert!(events.is_empty()); // Idempotent
    }
}
