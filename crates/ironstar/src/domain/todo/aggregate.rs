//! Todo aggregate implementation.
//!
//! The Todo aggregate manages the lifecycle of a single todo item.
//! It demonstrates the core event sourcing patterns:
//!
//! - Pure command handling with validation
//! - State derived from event stream
//! - Type-safe transitions via the Aggregate trait
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
//!                      Delete
//!                         │
//!                         ▼
//!                    ┌──────────┐
//!                    │ Deleted  │ (terminal)
//!                    └──────────┘
//! ```
//!
//! - `Active`: Initial state, can be completed or deleted
//! - `Completed`: Marked done, can be uncompleted or deleted
//! - `Deleted`: Terminal state, no further operations allowed

use chrono::Utc;

use crate::domain::aggregate::Aggregate;

use super::commands::TodoCommand;
use super::errors::TodoError;
#[cfg(test)]
use super::errors::TodoErrorKind;
use super::events::TodoEvent;
use super::state::{TodoState, TodoStatus};
use super::values::{TodoId, TodoText};

/// The Todo aggregate.
///
/// This is the entry point for the Aggregate trait implementation.
/// The actual state lives in [`TodoState`].
#[derive(Debug, Default)]
pub struct TodoAggregate;

impl Aggregate for TodoAggregate {
    const NAME: &'static str = "Todo";

    type State = TodoState;
    type Command = TodoCommand;
    type Event = TodoEvent;
    type Error = TodoError;

    fn handle_command(
        state: &Self::State,
        cmd: Self::Command,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match cmd {
            TodoCommand::Create { id, text } => handle_create(state, id, text),
            TodoCommand::UpdateText { id, text } => handle_update_text(state, id, text),
            TodoCommand::Complete { id } => handle_complete(state, id),
            TodoCommand::Uncomplete { id } => handle_uncomplete(state, id),
            TodoCommand::Delete { id } => handle_delete(state, id),
        }
    }

    fn apply_event(mut state: Self::State, event: Self::Event) -> Self::State {
        match event {
            TodoEvent::Created {
                id,
                text,
                created_at,
            } => {
                state.id = Some(id);
                state.text = Some(text);
                state.created_at = Some(created_at);
                state.status = TodoStatus::Active;
            }

            TodoEvent::TextUpdated { text, .. } => {
                state.text = Some(text);
            }

            TodoEvent::Completed { completed_at, .. } => {
                state.status = TodoStatus::Completed;
                state.completed_at = Some(completed_at);
            }

            TodoEvent::Uncompleted { .. } => {
                state.status = TodoStatus::Active;
                state.completed_at = None;
            }

            TodoEvent::Deleted { deleted_at, .. } => {
                state.status = TodoStatus::Deleted;
                state.deleted_at = Some(deleted_at);
            }
        }

        state
    }
}

// --- Command Handlers (private, pure functions) ---

/// Handle Create command.
///
/// Validates that the aggregate doesn't already exist and that the text
/// is valid. Returns a Created event with the current timestamp.
fn handle_create(state: &TodoState, id: TodoId, text: String) -> Result<Vec<TodoEvent>, TodoError> {
    // Aggregate should not exist yet
    if state.exists() {
        return Err(TodoError::invalid_transition("create", "already exists"));
    }

    // Validate and normalize text (smart constructor)
    let validated_text = TodoText::new(text)?;

    Ok(vec![TodoEvent::Created {
        id,
        text: validated_text,
        created_at: Utc::now(),
    }])
}

/// Handle UpdateText command.
fn handle_update_text(
    state: &TodoState,
    id: TodoId,
    text: String,
) -> Result<Vec<TodoEvent>, TodoError> {
    // Must exist and not be deleted
    if !state.exists() {
        return Err(TodoError::invalid_transition("update", "not created"));
    }

    if state.is_deleted() {
        return Err(TodoError::deleted());
    }

    // Validate new text
    let validated_text = TodoText::new(text)?;

    Ok(vec![TodoEvent::TextUpdated {
        id,
        text: validated_text,
        updated_at: Utc::now(),
    }])
}

/// Handle Complete command.
fn handle_complete(state: &TodoState, id: TodoId) -> Result<Vec<TodoEvent>, TodoError> {
    if !state.exists() {
        return Err(TodoError::invalid_transition("complete", "not created"));
    }

    if state.is_deleted() {
        return Err(TodoError::deleted());
    }

    if state.is_completed() {
        return Err(TodoError::already_completed());
    }

    Ok(vec![TodoEvent::Completed {
        id,
        completed_at: Utc::now(),
    }])
}

/// Handle Uncomplete command.
fn handle_uncomplete(state: &TodoState, id: TodoId) -> Result<Vec<TodoEvent>, TodoError> {
    if !state.exists() {
        return Err(TodoError::invalid_transition("uncomplete", "not created"));
    }

    if state.is_deleted() {
        return Err(TodoError::deleted());
    }

    if !state.is_completed() {
        return Err(TodoError::not_completed());
    }

    Ok(vec![TodoEvent::Uncompleted {
        id,
        uncompleted_at: Utc::now(),
    }])
}

/// Handle Delete command.
fn handle_delete(state: &TodoState, id: TodoId) -> Result<Vec<TodoEvent>, TodoError> {
    if !state.exists() {
        return Err(TodoError::invalid_transition("delete", "not created"));
    }

    if state.is_deleted() {
        return Err(TodoError::deleted());
    }

    Ok(vec![TodoEvent::Deleted {
        id,
        deleted_at: Utc::now(),
    }])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::aggregate::AggregateRoot;

    fn sample_id() -> TodoId {
        TodoId::from_uuid(uuid::Uuid::nil())
    }

    // --- State Machine Tests ---

    #[test]
    fn create_from_empty_state_succeeds() {
        let state = TodoState::default();
        let cmd = TodoCommand::Create {
            id: sample_id(),
            text: "Buy groceries".to_string(),
        };

        let events = TodoAggregate::handle_command(&state, cmd).unwrap();

        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], TodoEvent::Created { text, .. } if text.as_str() == "Buy groceries")
        );
    }

    #[test]
    fn create_validates_and_trims_text() {
        let state = TodoState::default();
        let cmd = TodoCommand::Create {
            id: sample_id(),
            text: "  Trim me  ".to_string(),
        };

        let events = TodoAggregate::handle_command(&state, cmd).unwrap();

        assert!(
            matches!(&events[0], TodoEvent::Created { text, .. } if text.as_str() == "Trim me")
        );
    }

    #[test]
    fn create_rejects_empty_text() {
        let state = TodoState::default();
        let cmd = TodoCommand::Create {
            id: sample_id(),
            text: "   ".to_string(),
        };

        let result = TodoAggregate::handle_command(&state, cmd);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), &TodoErrorKind::EmptyText);
    }

    #[test]
    fn create_rejects_if_already_exists() {
        let state = TodoState {
            status: TodoStatus::Active,
            ..Default::default()
        };
        let cmd = TodoCommand::Create {
            id: sample_id(),
            text: "Test".to_string(),
        };

        let result = TodoAggregate::handle_command(&state, cmd);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().kind(),
            TodoErrorKind::InvalidTransition { .. }
        ));
    }

    #[test]
    fn complete_active_todo_succeeds() {
        let state = TodoState {
            id: Some(sample_id()),
            status: TodoStatus::Active,
            ..Default::default()
        };
        let cmd = TodoCommand::Complete { id: sample_id() };

        let events = TodoAggregate::handle_command(&state, cmd).unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], TodoEvent::Completed { .. }));
    }

    #[test]
    fn complete_already_completed_fails() {
        let state = TodoState {
            id: Some(sample_id()),
            status: TodoStatus::Completed,
            ..Default::default()
        };
        let cmd = TodoCommand::Complete { id: sample_id() };

        let result = TodoAggregate::handle_command(&state, cmd);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), &TodoErrorKind::AlreadyCompleted);
    }

    #[test]
    fn uncomplete_completed_todo_succeeds() {
        let state = TodoState {
            id: Some(sample_id()),
            status: TodoStatus::Completed,
            ..Default::default()
        };
        let cmd = TodoCommand::Uncomplete { id: sample_id() };

        let events = TodoAggregate::handle_command(&state, cmd).unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], TodoEvent::Uncompleted { .. }));
    }

    #[test]
    fn uncomplete_active_todo_fails() {
        let state = TodoState {
            id: Some(sample_id()),
            status: TodoStatus::Active,
            ..Default::default()
        };
        let cmd = TodoCommand::Uncomplete { id: sample_id() };

        let result = TodoAggregate::handle_command(&state, cmd);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), &TodoErrorKind::NotCompleted);
    }

    #[test]
    fn delete_active_todo_succeeds() {
        let state = TodoState {
            id: Some(sample_id()),
            status: TodoStatus::Active,
            ..Default::default()
        };
        let cmd = TodoCommand::Delete { id: sample_id() };

        let events = TodoAggregate::handle_command(&state, cmd).unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], TodoEvent::Deleted { .. }));
    }

    #[test]
    fn operations_on_deleted_todo_fail() {
        let state = TodoState {
            id: Some(sample_id()),
            status: TodoStatus::Deleted,
            ..Default::default()
        };

        let commands = vec![
            TodoCommand::UpdateText {
                id: sample_id(),
                text: "Test".to_string(),
            },
            TodoCommand::Complete { id: sample_id() },
            TodoCommand::Uncomplete { id: sample_id() },
            TodoCommand::Delete { id: sample_id() },
        ];

        for cmd in commands {
            let result = TodoAggregate::handle_command(&state, cmd);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind(), &TodoErrorKind::Deleted);
        }
    }

    // --- State Reconstruction Tests ---

    #[test]
    fn apply_created_sets_initial_state() {
        let event = TodoEvent::Created {
            id: sample_id(),
            text: TodoText::new("Test").unwrap(),
            created_at: Utc::now(),
        };

        let state = TodoAggregate::apply_event(TodoState::default(), event);

        assert_eq!(state.id, Some(sample_id()));
        assert_eq!(state.text.as_ref().map(|t| t.as_str()), Some("Test"));
        assert_eq!(state.status, TodoStatus::Active);
    }

    #[test]
    fn apply_completed_changes_status() {
        let initial = TodoState {
            id: Some(sample_id()),
            status: TodoStatus::Active,
            ..Default::default()
        };
        let event = TodoEvent::Completed {
            id: sample_id(),
            completed_at: Utc::now(),
        };

        let state = TodoAggregate::apply_event(initial, event);

        assert_eq!(state.status, TodoStatus::Completed);
        assert!(state.completed_at.is_some());
    }

    #[test]
    fn apply_uncompleted_reverts_to_active() {
        let initial = TodoState {
            id: Some(sample_id()),
            status: TodoStatus::Completed,
            completed_at: Some(Utc::now()),
            ..Default::default()
        };
        let event = TodoEvent::Uncompleted {
            id: sample_id(),
            uncompleted_at: Utc::now(),
        };

        let state = TodoAggregate::apply_event(initial, event);

        assert_eq!(state.status, TodoStatus::Active);
        assert!(state.completed_at.is_none());
    }

    // --- Integration with AggregateRoot ---

    #[test]
    fn aggregate_root_full_lifecycle() {
        let mut root = AggregateRoot::<TodoAggregate>::new();
        let id = TodoId::new();

        // Create
        let events = root
            .handle(TodoCommand::Create {
                id,
                text: "Buy groceries".to_string(),
            })
            .unwrap();
        root.apply_all(events);
        assert_eq!(root.version(), 1);
        assert!(root.state().is_active());

        // Complete
        let events = root.handle(TodoCommand::Complete { id }).unwrap();
        root.apply_all(events);
        assert_eq!(root.version(), 2);
        assert!(root.state().is_completed());

        // Uncomplete
        let events = root.handle(TodoCommand::Uncomplete { id }).unwrap();
        root.apply_all(events);
        assert_eq!(root.version(), 3);
        assert!(root.state().is_active());

        // Delete
        let events = root.handle(TodoCommand::Delete { id }).unwrap();
        root.apply_all(events);
        assert_eq!(root.version(), 4);
        assert!(root.state().is_deleted());

        // Further operations fail
        assert!(root.handle(TodoCommand::Complete { id }).is_err());
    }
}
