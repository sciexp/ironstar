//! Todo View for read-side projections.
//!
//! The View is a pure projection that materializes events into queryable state.
//! Unlike the Decider, it has no command handling — only event folding.
//!
//! # Usage
//!
//! ```rust,ignore
//! use fmodel_rust::view::ViewStateComputation;
//! use ironstar::domain::views::{todo_view, TodoViewState};
//! use ironstar::domain::TodoEvent;
//!
//! let view = todo_view();
//! let state = view.compute_new_state(None, &[event1, event2]);
//! assert_eq!(state.count, 2);
//! ```
//!
//! # State structure
//!
//! `TodoViewState` contains:
//! - `todos`: Vector of `TodoItemView` for rendering the list
//! - `count`: Total non-deleted todos
//! - `completed_count`: Number of completed todos

use fmodel_rust::view::View;

use crate::domain::signals::TodoItemView;
use crate::domain::todo::events::TodoEvent;
use crate::domain::todo::values::TodoId;

/// State materialized by the Todo View.
///
/// This is the read model for the todo list, containing denormalized data
/// optimized for querying and rendering.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TodoViewState {
    /// All non-deleted todos in creation order.
    pub todos: Vec<TodoItemView>,

    /// Total count of non-deleted todos.
    ///
    /// Invariant: `count == todos.len()`
    pub count: usize,

    /// Count of completed todos.
    ///
    /// Invariant: `completed_count == todos.iter().filter(|t| t.completed).count()`
    pub completed_count: usize,
}

impl TodoViewState {
    /// Find a todo by ID, returning its index if present.
    fn find_index(&self, id: TodoId) -> Option<usize> {
        self.todos.iter().position(|t| t.id == id.into_inner())
    }

    /// Count of active (non-completed) todos.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.count.saturating_sub(self.completed_count)
    }
}

/// Type alias for the Todo View.
pub type TodoView<'a> = View<'a, TodoViewState, TodoEvent>;

/// Factory function creating a pure Todo View.
///
/// The view materializes events into a queryable `TodoViewState`:
/// - `Created` adds a new todo to the list
/// - `TextUpdated` updates the todo's text
/// - `Completed` marks the todo as completed
/// - `Uncompleted` marks the todo as active
/// - `Deleted` removes the todo from the list
///
/// # Example
///
/// ```rust,ignore
/// use fmodel_rust::view::ViewStateComputation;
/// use ironstar::domain::views::todo_view;
/// use ironstar::domain::TodoEvent;
///
/// let view = todo_view();
/// let initial = (view.initial_state)();
/// assert_eq!(initial.count, 0);
/// ```
pub fn todo_view<'a>() -> TodoView<'a> {
    View {
        evolve: Box::new(evolve),
        initial_state: Box::new(TodoViewState::default),
    }
}

/// Pure evolve function: (State, Event) -> State
///
/// This function applies an event to produce new state. It is:
/// - **Infallible**: No errors, just state transformation
/// - **Deterministic**: Same inputs always produce the same output
/// - **Total**: Handles all event variants
fn evolve(state: &TodoViewState, event: &TodoEvent) -> TodoViewState {
    match event {
        TodoEvent::Created { id, text, .. } => {
            let mut todos = state.todos.clone();
            todos.push(TodoItemView {
                id: id.into_inner(),
                text: text.to_string(),
                completed: false,
            });
            TodoViewState {
                todos,
                count: state.count + 1,
                completed_count: state.completed_count,
            }
        }

        TodoEvent::TextUpdated { id, text, .. } => {
            let mut todos = state.todos.clone();
            if let Some(idx) = state.find_index(*id) {
                todos[idx].text = text.to_string();
            }
            TodoViewState {
                todos,
                count: state.count,
                completed_count: state.completed_count,
            }
        }

        TodoEvent::Completed { id, .. } => {
            let mut todos = state.todos.clone();
            let mut completed_delta = 0;
            if let Some(idx) = state.find_index(*id) {
                if !todos[idx].completed {
                    todos[idx].completed = true;
                    completed_delta = 1;
                }
            }
            TodoViewState {
                todos,
                count: state.count,
                completed_count: state.completed_count + completed_delta,
            }
        }

        TodoEvent::Uncompleted { id, .. } => {
            let mut todos = state.todos.clone();
            let mut completed_delta = 0;
            if let Some(idx) = state.find_index(*id) {
                if todos[idx].completed {
                    todos[idx].completed = false;
                    completed_delta = 1;
                }
            }
            TodoViewState {
                todos,
                count: state.count,
                completed_count: state.completed_count.saturating_sub(completed_delta),
            }
        }

        TodoEvent::Deleted { id, .. } => {
            if let Some(idx) = state.find_index(*id) {
                let mut todos = state.todos.clone();
                let was_completed = todos[idx].completed;
                todos.remove(idx);
                TodoViewState {
                    todos,
                    count: state.count.saturating_sub(1),
                    completed_count: if was_completed {
                        state.completed_count.saturating_sub(1)
                    } else {
                        state.completed_count
                    },
                }
            } else {
                // Item not found — true no-op, preserve invariants
                state.clone()
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use fmodel_rust::view::ViewStateComputation;
    use uuid::Uuid;

    use crate::domain::todo::values::TodoText;

    fn sample_id() -> TodoId {
        TodoId::from_uuid(Uuid::nil())
    }

    fn sample_id_2() -> TodoId {
        TodoId::from_uuid(Uuid::from_u128(1))
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn sample_text() -> TodoText {
        TodoText::new("Buy groceries").unwrap()
    }

    /// Helper to convert Vec<E> to Vec<&E> for compute_new_state.
    fn as_refs<T>(events: &[T]) -> Vec<&T> {
        events.iter().collect()
    }

    // --- Initial state ---

    #[test]
    fn initial_state_is_empty() {
        let view = todo_view();
        let state = (view.initial_state)();

        assert!(state.todos.is_empty());
        assert_eq!(state.count, 0);
        assert_eq!(state.completed_count, 0);
        assert_eq!(state.active_count(), 0);
    }

    // --- Created event ---

    #[test]
    fn created_adds_todo() {
        let view = todo_view();
        let event = TodoEvent::Created {
            id: sample_id(),
            text: sample_text(),
            created_at: sample_time(),
        };

        let state = view.compute_new_state(None, &[&event]);

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.count, 1);
        assert_eq!(state.completed_count, 0);
        assert_eq!(state.todos[0].id, Uuid::nil());
        assert_eq!(state.todos[0].text, "Buy groceries");
        assert!(!state.todos[0].completed);
    }

    #[test]
    fn multiple_created_accumulates() {
        let view = todo_view();
        let events = vec![
            TodoEvent::Created {
                id: sample_id(),
                text: sample_text(),
                created_at: sample_time(),
            },
            TodoEvent::Created {
                id: sample_id_2(),
                text: TodoText::new("Walk the dog").unwrap(),
                created_at: sample_time(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert_eq!(state.todos.len(), 2);
        assert_eq!(state.count, 2);
        assert_eq!(state.active_count(), 2);
    }

    // --- TextUpdated event ---

    #[test]
    fn text_updated_changes_text() {
        let view = todo_view();
        let events = vec![
            TodoEvent::Created {
                id: sample_id(),
                text: sample_text(),
                created_at: sample_time(),
            },
            TodoEvent::TextUpdated {
                id: sample_id(),
                text: TodoText::new("Buy milk").unwrap(),
                updated_at: sample_time(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert_eq!(state.todos[0].text, "Buy milk");
        assert_eq!(state.count, 1);
    }

    #[test]
    fn text_updated_for_nonexistent_is_noop() {
        let view = todo_view();
        let events = vec![TodoEvent::TextUpdated {
            id: sample_id(),
            text: TodoText::new("Should not crash").unwrap(),
            updated_at: sample_time(),
        }];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(state.todos.is_empty());
        assert_eq!(state.count, 0);
    }

    // --- Completed event ---

    #[test]
    fn completed_marks_todo() {
        let view = todo_view();
        let events = vec![
            TodoEvent::Created {
                id: sample_id(),
                text: sample_text(),
                created_at: sample_time(),
            },
            TodoEvent::Completed {
                id: sample_id(),
                completed_at: sample_time(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(state.todos[0].completed);
        assert_eq!(state.completed_count, 1);
        assert_eq!(state.active_count(), 0);
    }

    #[test]
    fn completed_is_idempotent() {
        let view = todo_view();
        let events = vec![
            TodoEvent::Created {
                id: sample_id(),
                text: sample_text(),
                created_at: sample_time(),
            },
            TodoEvent::Completed {
                id: sample_id(),
                completed_at: sample_time(),
            },
            TodoEvent::Completed {
                id: sample_id(),
                completed_at: sample_time(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert_eq!(state.completed_count, 1);
    }

    // --- Uncompleted event ---

    #[test]
    fn uncompleted_marks_active() {
        let view = todo_view();
        let events = vec![
            TodoEvent::Created {
                id: sample_id(),
                text: sample_text(),
                created_at: sample_time(),
            },
            TodoEvent::Completed {
                id: sample_id(),
                completed_at: sample_time(),
            },
            TodoEvent::Uncompleted {
                id: sample_id(),
                uncompleted_at: sample_time(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(!state.todos[0].completed);
        assert_eq!(state.completed_count, 0);
        assert_eq!(state.active_count(), 1);
    }

    #[test]
    fn uncompleted_is_idempotent() {
        let view = todo_view();
        let events = vec![
            TodoEvent::Created {
                id: sample_id(),
                text: sample_text(),
                created_at: sample_time(),
            },
            TodoEvent::Uncompleted {
                id: sample_id(),
                uncompleted_at: sample_time(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert_eq!(state.completed_count, 0);
    }

    // --- Deleted event ---

    #[test]
    fn deleted_removes_todo() {
        let view = todo_view();
        let events = vec![
            TodoEvent::Created {
                id: sample_id(),
                text: sample_text(),
                created_at: sample_time(),
            },
            TodoEvent::Deleted {
                id: sample_id(),
                deleted_at: sample_time(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(state.todos.is_empty());
        assert_eq!(state.count, 0);
        assert_eq!(state.completed_count, 0);
    }

    #[test]
    fn deleted_completed_decrements_both_counts() {
        let view = todo_view();
        let events = vec![
            TodoEvent::Created {
                id: sample_id(),
                text: sample_text(),
                created_at: sample_time(),
            },
            TodoEvent::Completed {
                id: sample_id(),
                completed_at: sample_time(),
            },
            TodoEvent::Deleted {
                id: sample_id(),
                deleted_at: sample_time(),
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert_eq!(state.count, 0);
        assert_eq!(state.completed_count, 0);
    }

    #[test]
    fn deleted_for_nonexistent_is_noop() {
        let view = todo_view();
        let events = vec![TodoEvent::Deleted {
            id: sample_id(),
            deleted_at: sample_time(),
        }];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert!(state.todos.is_empty());
        assert_eq!(state.count, 0);
    }

    #[test]
    fn deleted_for_nonexistent_preserves_count_invariant() {
        // Regression test: count must equal todos.len() after deleting non-existent ID
        let view = todo_view();
        let existing_id = sample_id();
        let nonexistent_id = sample_id_2();

        // Create one item
        let setup = vec![TodoEvent::Created {
            id: existing_id,
            text: TodoText::new("existing").unwrap(),
            created_at: sample_time(),
        }];
        let state = view.compute_new_state(None, &as_refs(&setup));
        assert_eq!(state.count, 1);
        assert_eq!(state.todos.len(), 1);

        // Delete a different ID that doesn't exist
        let delete_nonexistent = vec![TodoEvent::Deleted {
            id: nonexistent_id,
            deleted_at: sample_time(),
        }];
        let state = view.compute_new_state(Some(state), &as_refs(&delete_nonexistent));

        // Invariant: count == todos.len()
        assert_eq!(state.todos.len(), 1, "item should not be removed");
        assert_eq!(state.count, 1, "count must equal todos.len()");
        assert_eq!(state.count, state.todos.len(), "count invariant broken");
    }

    // --- Full lifecycle ---

    #[test]
    fn full_lifecycle() {
        let view = todo_view();
        let id1 = sample_id();
        let id2 = sample_id_2();
        let ts = sample_time();

        let events = vec![
            // Create two todos
            TodoEvent::Created {
                id: id1,
                text: TodoText::new("First").unwrap(),
                created_at: ts,
            },
            TodoEvent::Created {
                id: id2,
                text: TodoText::new("Second").unwrap(),
                created_at: ts,
            },
            // Complete first
            TodoEvent::Completed {
                id: id1,
                completed_at: ts,
            },
            // Update second's text
            TodoEvent::TextUpdated {
                id: id2,
                text: TodoText::new("Second updated").unwrap(),
                updated_at: ts,
            },
            // Uncomplete first
            TodoEvent::Uncompleted {
                id: id1,
                uncompleted_at: ts,
            },
            // Delete second
            TodoEvent::Deleted {
                id: id2,
                deleted_at: ts,
            },
        ];

        let state = view.compute_new_state(None, &as_refs(&events));

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.count, 1);
        assert_eq!(state.completed_count, 0);
        assert_eq!(state.todos[0].text, "First");
        assert!(!state.todos[0].completed);
    }

    // --- ViewStateComputation trait ---

    #[test]
    fn compute_with_current_state() {
        let view = todo_view();
        let ts = sample_time();

        // Start with existing state
        let initial = TodoViewState {
            todos: vec![TodoItemView {
                id: sample_id().into_inner(),
                text: "Existing".to_string(),
                completed: false,
            }],
            count: 1,
            completed_count: 0,
        };

        // Add new event
        let events = vec![TodoEvent::Created {
            id: sample_id_2(),
            text: TodoText::new("New").unwrap(),
            created_at: ts,
        }];

        let state = view.compute_new_state(Some(initial), &as_refs(&events));

        assert_eq!(state.todos.len(), 2);
        assert_eq!(state.count, 2);
    }
}
