//! Pure Todo Decider implementing fmodel-rust patterns.
//!
//! The Decider is the core decision-making component that embodies the
//! state machine from `spec/Todo/Todo.idr`. It is a pure function with
//! no side effects: all I/O (timestamps, persistence) happens at boundaries.
//!
//! # State Machine
//!
//! ```text
//!                    ┌──────────┐
//!     Create ───────►│  Active  │◄────────┐
//!                    └────┬─────┘         │
//!                         │               │
//!                    Complete        Uncomplete
//!                         │               │
//!                         ▼               │
//!                    ┌──────────┐         │
//!                    │Completed │─────────┘
//!                    └────┬─────┘
//!                         │
//!                      Delete (also from Active)
//!                         │
//!                         ▼
//!                    ┌──────────┐
//!                    │ Deleted  │ (terminal)
//!                    └──────────┘
//! ```
//!
//! # Idempotency
//!
//! Operations that would result in the same state return `Ok(vec![])`:
//! - Complete when already Completed
//! - Uncomplete when already Active
//! - Delete when already Deleted

use fmodel_rust::decider::Decider;

use super::commands::TodoCommand;
use super::errors::TodoError;
use super::events::TodoEvent;
use super::state::{TodoState, TodoStatus};
use super::values::TodoText;

/// Type alias for the Todo Decider.
///
/// The state is `Option<TodoState>` to distinguish non-existent (None)
/// from exists (Some). This makes the NonExistent state explicit in
/// the type system.
pub type TodoDecider<'a> = Decider<'a, TodoCommand, Option<TodoState>, TodoEvent, TodoError>;

/// Factory function creating a pure Todo Decider.
///
/// The decider embodies the state machine from `spec/Todo/Todo.idr`:
/// - NonExistent → Active → Completed ↔ Active → Deleted (terminal)
/// - Idempotent operations return `Ok(vec![])` when already in target state
/// - Precondition violations return `Err(TodoError::X)`
///
/// # Example
///
/// ```rust,ignore
/// use fmodel_rust::decider::EventComputation;
/// use ironstar::domain::todo::{todo_decider, TodoCommand, TodoId};
/// use chrono::Utc;
///
/// let decider = todo_decider();
/// let id = TodoId::new();
/// let now = Utc::now();
///
/// let events = decider.compute_new_events(
///     &[],
///     &TodoCommand::Create { id, text: "Buy milk".into(), created_at: now }
/// );
/// ```
pub fn todo_decider<'a>() -> TodoDecider<'a> {
    Decider {
        decide: Box::new(|command, state| decide(command, state)),
        evolve: Box::new(|state, event| evolve(state, event)),
        initial_state: Box::new(|| None),
    }
}

/// Pure decide function: (Command, State) -> Result<Vec<Event>, Error>
///
/// This function is the heart of the domain logic. It validates commands
/// against current state and returns events or errors. No side effects.
fn decide(command: &TodoCommand, state: &Option<TodoState>) -> Result<Vec<TodoEvent>, TodoError> {
    match (command, state) {
        // Create: NonExistent → Active
        (
            TodoCommand::Create {
                id,
                text,
                created_at,
            },
            None,
        ) => {
            let validated_text = TodoText::new(text.clone())?;
            Ok(vec![TodoEvent::Created {
                id: *id,
                text: validated_text,
                created_at: *created_at,
            }])
        }
        (TodoCommand::Create { .. }, Some(_)) => Err(TodoError::already_exists()),

        // UpdateText: Active/Completed → same state with new text
        (
            TodoCommand::UpdateText {
                id,
                text,
                updated_at,
            },
            Some(s),
        ) if !s.is_deleted() => {
            let validated_text = TodoText::new(text.clone())?;
            Ok(vec![TodoEvent::TextUpdated {
                id: *id,
                text: validated_text,
                updated_at: *updated_at,
            }])
        }
        (TodoCommand::UpdateText { .. }, None) => Err(TodoError::not_found()),
        (TodoCommand::UpdateText { .. }, Some(_)) => {
            // Deleted state
            Err(TodoError::not_found())
        }

        // Complete: Active → Completed (idempotent if already Completed)
        (TodoCommand::Complete { id, completed_at }, Some(s)) if s.is_active() => {
            Ok(vec![TodoEvent::Completed {
                id: *id,
                completed_at: *completed_at,
            }])
        }
        (TodoCommand::Complete { .. }, Some(s)) if s.is_completed() => {
            Ok(vec![]) // Idempotent: already completed
        }
        (TodoCommand::Complete { .. }, _) => Err(TodoError::cannot_complete()),

        // Uncomplete: Completed → Active (idempotent if already Active)
        (TodoCommand::Uncomplete { id, uncompleted_at }, Some(s)) if s.is_completed() => {
            Ok(vec![TodoEvent::Uncompleted {
                id: *id,
                uncompleted_at: *uncompleted_at,
            }])
        }
        (TodoCommand::Uncomplete { .. }, Some(s)) if s.is_active() => {
            Ok(vec![]) // Idempotent: already active
        }
        (TodoCommand::Uncomplete { .. }, _) => Err(TodoError::cannot_uncomplete()),

        // Delete: Active/Completed → Deleted (idempotent if already Deleted)
        (TodoCommand::Delete { id, deleted_at }, Some(s)) if s.is_active() || s.is_completed() => {
            Ok(vec![TodoEvent::Deleted {
                id: *id,
                deleted_at: *deleted_at,
            }])
        }
        (TodoCommand::Delete { .. }, Some(s)) if s.is_deleted() => {
            Ok(vec![]) // Idempotent: already deleted
        }
        (TodoCommand::Delete { .. }, None) => Err(TodoError::cannot_delete()),
        // Fallback for Any Some state not covered above (e.g., NotCreated status)
        // This should never happen in normal operation but provides exhaustiveness
        (TodoCommand::Delete { .. }, Some(_)) => Err(TodoError::cannot_delete()),
    }
}

/// Pure evolve function: (State, Event) -> State
///
/// This function applies an event to produce new state. It must be
/// deterministic and total (handle all event variants).
fn evolve(state: &Option<TodoState>, event: &TodoEvent) -> Option<TodoState> {
    match event {
        // Created: None → Some(Active)
        TodoEvent::Created {
            id,
            text,
            created_at,
        } => Some(TodoState {
            id: Some(*id),
            text: Some(text.clone()),
            created_at: Some(*created_at),
            status: TodoStatus::Active,
            completed_at: None,
            deleted_at: None,
        }),

        // TextUpdated: update text field
        TodoEvent::TextUpdated { text, .. } => state.as_ref().map(|s| TodoState {
            text: Some(text.clone()),
            ..s.clone()
        }),

        // Completed: Active → Completed
        TodoEvent::Completed { completed_at, .. } => state.as_ref().map(|s| TodoState {
            status: TodoStatus::Completed,
            completed_at: Some(*completed_at),
            ..s.clone()
        }),

        // Uncompleted: Completed → Active
        TodoEvent::Uncompleted { .. } => state.as_ref().map(|s| TodoState {
            status: TodoStatus::Active,
            completed_at: None,
            ..s.clone()
        }),

        // Deleted: Any → Deleted
        TodoEvent::Deleted { deleted_at, .. } => state.as_ref().map(|s| TodoState {
            status: TodoStatus::Deleted,
            deleted_at: Some(*deleted_at),
            ..s.clone()
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use fmodel_rust::specification::DeciderTestSpecification;

    use super::super::values::TodoId;

    fn sample_id() -> TodoId {
        TodoId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn sample_text() -> TodoText {
        TodoText::new("Buy groceries").unwrap()
    }

    // --- Create transitions ---

    #[test]
    fn create_from_nonexistent_succeeds() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![])
            .when(TodoCommand::Create {
                id,
                text: "Buy groceries".to_string(),
                created_at: ts,
            })
            .then(vec![TodoEvent::Created {
                id,
                text: sample_text(),
                created_at: ts,
            }]);
    }

    #[test]
    fn create_when_already_exists_fails() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![TodoEvent::Created {
                id,
                text: sample_text(),
                created_at: ts,
            }])
            .when(TodoCommand::Create {
                id,
                text: "Another todo".to_string(),
                created_at: ts,
            })
            .then_error(TodoError::already_exists());
    }

    #[test]
    fn create_validates_text() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![])
            .when(TodoCommand::Create {
                id,
                text: "   ".to_string(), // Empty after trim
                created_at: ts,
            })
            .then_error(TodoError::empty_text());
    }

    // --- Complete transitions ---

    #[test]
    fn complete_active_succeeds() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![TodoEvent::Created {
                id,
                text: sample_text(),
                created_at: ts,
            }])
            .when(TodoCommand::Complete {
                id,
                completed_at: ts,
            })
            .then(vec![TodoEvent::Completed {
                id,
                completed_at: ts,
            }]);
    }

    #[test]
    fn complete_already_completed_is_idempotent() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created {
                    id,
                    text: sample_text(),
                    created_at: ts,
                },
                TodoEvent::Completed {
                    id,
                    completed_at: ts,
                },
            ])
            .when(TodoCommand::Complete {
                id,
                completed_at: ts,
            })
            .then(vec![]); // Idempotent: no events
    }

    #[test]
    fn complete_deleted_fails() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created {
                    id,
                    text: sample_text(),
                    created_at: ts,
                },
                TodoEvent::Deleted { id, deleted_at: ts },
            ])
            .when(TodoCommand::Complete {
                id,
                completed_at: ts,
            })
            .then_error(TodoError::cannot_complete());
    }

    #[test]
    fn complete_nonexistent_fails() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![])
            .when(TodoCommand::Complete {
                id,
                completed_at: ts,
            })
            .then_error(TodoError::cannot_complete());
    }

    // --- Uncomplete transitions ---

    #[test]
    fn uncomplete_completed_succeeds() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created {
                    id,
                    text: sample_text(),
                    created_at: ts,
                },
                TodoEvent::Completed {
                    id,
                    completed_at: ts,
                },
            ])
            .when(TodoCommand::Uncomplete {
                id,
                uncompleted_at: ts,
            })
            .then(vec![TodoEvent::Uncompleted {
                id,
                uncompleted_at: ts,
            }]);
    }

    #[test]
    fn uncomplete_active_is_idempotent() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![TodoEvent::Created {
                id,
                text: sample_text(),
                created_at: ts,
            }])
            .when(TodoCommand::Uncomplete {
                id,
                uncompleted_at: ts,
            })
            .then(vec![]); // Idempotent: no events
    }

    #[test]
    fn uncomplete_deleted_fails() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created {
                    id,
                    text: sample_text(),
                    created_at: ts,
                },
                TodoEvent::Deleted { id, deleted_at: ts },
            ])
            .when(TodoCommand::Uncomplete {
                id,
                uncompleted_at: ts,
            })
            .then_error(TodoError::cannot_uncomplete());
    }

    #[test]
    fn uncomplete_nonexistent_fails() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![])
            .when(TodoCommand::Uncomplete {
                id,
                uncompleted_at: ts,
            })
            .then_error(TodoError::cannot_uncomplete());
    }

    // --- Delete transitions ---

    #[test]
    fn delete_active_succeeds() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![TodoEvent::Created {
                id,
                text: sample_text(),
                created_at: ts,
            }])
            .when(TodoCommand::Delete { id, deleted_at: ts })
            .then(vec![TodoEvent::Deleted { id, deleted_at: ts }]);
    }

    #[test]
    fn delete_completed_succeeds() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created {
                    id,
                    text: sample_text(),
                    created_at: ts,
                },
                TodoEvent::Completed {
                    id,
                    completed_at: ts,
                },
            ])
            .when(TodoCommand::Delete { id, deleted_at: ts })
            .then(vec![TodoEvent::Deleted { id, deleted_at: ts }]);
    }

    #[test]
    fn delete_already_deleted_is_idempotent() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created {
                    id,
                    text: sample_text(),
                    created_at: ts,
                },
                TodoEvent::Deleted { id, deleted_at: ts },
            ])
            .when(TodoCommand::Delete { id, deleted_at: ts })
            .then(vec![]); // Idempotent: no events
    }

    #[test]
    fn delete_nonexistent_fails() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![])
            .when(TodoCommand::Delete { id, deleted_at: ts })
            .then_error(TodoError::cannot_delete());
    }

    // --- UpdateText transitions ---

    #[test]
    fn update_text_active_succeeds() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![TodoEvent::Created {
                id,
                text: sample_text(),
                created_at: ts,
            }])
            .when(TodoCommand::UpdateText {
                id,
                text: "New text".to_string(),
                updated_at: ts,
            })
            .then(vec![TodoEvent::TextUpdated {
                id,
                text: TodoText::new("New text").unwrap(),
                updated_at: ts,
            }]);
    }

    #[test]
    fn update_text_completed_succeeds() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created {
                    id,
                    text: sample_text(),
                    created_at: ts,
                },
                TodoEvent::Completed {
                    id,
                    completed_at: ts,
                },
            ])
            .when(TodoCommand::UpdateText {
                id,
                text: "Updated completed".to_string(),
                updated_at: ts,
            })
            .then(vec![TodoEvent::TextUpdated {
                id,
                text: TodoText::new("Updated completed").unwrap(),
                updated_at: ts,
            }]);
    }

    #[test]
    fn update_text_deleted_fails() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![
                TodoEvent::Created {
                    id,
                    text: sample_text(),
                    created_at: ts,
                },
                TodoEvent::Deleted { id, deleted_at: ts },
            ])
            .when(TodoCommand::UpdateText {
                id,
                text: "Should fail".to_string(),
                updated_at: ts,
            })
            .then_error(TodoError::not_found());
    }

    #[test]
    fn update_text_nonexistent_fails() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![])
            .when(TodoCommand::UpdateText {
                id,
                text: "Should fail".to_string(),
                updated_at: ts,
            })
            .then_error(TodoError::not_found());
    }

    #[test]
    fn update_text_validates() {
        let id = sample_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![TodoEvent::Created {
                id,
                text: sample_text(),
                created_at: ts,
            }])
            .when(TodoCommand::UpdateText {
                id,
                text: "   ".to_string(), // Empty after trim
                updated_at: ts,
            })
            .then_error(TodoError::empty_text());
    }

    // --- Full lifecycle ---

    #[test]
    fn full_lifecycle_create_complete_uncomplete_delete() {
        let id = sample_id();
        let ts = sample_time();
        let decider = todo_decider();

        // Create
        let events = decide(
            &TodoCommand::Create {
                id,
                text: "Test".to_string(),
                created_at: ts,
            },
            &None,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        // Apply create
        let state = evolve(&None, &events[0]);
        assert!(state.as_ref().unwrap().is_active());

        // Complete
        let events = decide(
            &TodoCommand::Complete {
                id,
                completed_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        // Apply complete
        let state = evolve(&state, &events[0]);
        assert!(state.as_ref().unwrap().is_completed());

        // Uncomplete
        let events = decide(
            &TodoCommand::Uncomplete {
                id,
                uncompleted_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        // Apply uncomplete
        let state = evolve(&state, &events[0]);
        assert!(state.as_ref().unwrap().is_active());

        // Delete
        let events = decide(&TodoCommand::Delete { id, deleted_at: ts }, &state).unwrap();
        assert_eq!(events.len(), 1);

        // Apply delete
        let state = evolve(&state, &events[0]);
        assert!(state.as_ref().unwrap().is_deleted());

        // Verify terminal state: further operations fail or are idempotent
        assert!(
            decide(
                &TodoCommand::Complete {
                    id,
                    completed_at: ts
                },
                &state
            )
            .is_err()
        );
        assert!(
            decide(
                &TodoCommand::Uncomplete {
                    id,
                    uncompleted_at: ts
                },
                &state
            )
            .is_err()
        );

        // Delete is idempotent
        let events = decide(&TodoCommand::Delete { id, deleted_at: ts }, &state).unwrap();
        assert!(events.is_empty());
    }
}
