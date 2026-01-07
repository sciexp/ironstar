# Todo Application (Generality Canary)

## Workflow Description

This workflow models a minimal todo list application to validate that the ironstar template's event sourcing infrastructure works for any domain. It is intentionally maximally simple with no cross-context dependencies—a "canary in the coal mine" proving the patterns generalize beyond the main Analytics domain.

## Actors

1. **User** (Human) — Creates, manages, and tracks personal todo items
2. **Automation** (Optional) — Future: completion reminders, notifications

## Workflow Narrative (Chronological)

### Todo Lifecycle

1. User creates a new todo with text content (e.g., "Review quarterly report")
2. System records the todo creation with unique identifier and timestamp
3. Todo appears in the TodoList read model as incomplete
4. User may complete the todo when the task is finished
5. System records the completion with timestamp
6. Todo appears in TodoList as completed
7. User may uncomplete a todo if marked complete by mistake
8. System records the uncomplete action
9. Todo returns to incomplete state in TodoList
10. User may delete a todo that is no longer relevant
11. System records the deletion as terminal state
12. Todo is removed from TodoList (or marked as deleted depending on UI choice)

### State Machine

```
NonExistent → [Create] → Active ↔ [Complete/Uncomplete] → CompletedTodo → [Delete] → DeletedTodo (terminal)
                         Active → [Delete] → DeletedTodo (terminal)
```

## Key Events (Past Tense)

- **TodoCreated**: New todo was created with ID and text content
- **TodoCompleted**: Todo was marked as complete
- **TodoUncompleted**: Todo was unmarked (returned to incomplete)
- **TodoDeleted**: Todo was permanently deleted

## Commands (Imperative)

- **Create(text)**: Create new todo with given text content
- **Complete**: Mark todo as done
- **Uncomplete**: Return completed todo to active state
- **Delete**: Remove todo permanently

## Read Models

### TodoList
- Items: List of (TodoId, Text, IsCompleted) tuples
- Projection: Created → add item, Completed → update flag, Uncompleted → update flag, Deleted → remove item

## Idempotent Command Semantics

- Complete on already-completed todo: No-op (returns empty event list, not error)
- Uncomplete on already-active todo: No-op
- Delete on already-deleted todo: No-op
- Create on existing todo: Validation error (TodoAlreadyExists)
- Complete/Uncomplete on deleted todo: Validation error

## Exceptional Flows

- **Duplicate create**: Attempting to create todo with same aggregate ID → TodoAlreadyExists error
- **Invalid state transition**: Complete on NonExistent → CannotCompleteInCurrentState error
- **Terminal state**: Any command on DeletedTodo → appropriate validation error

## Bounded Context

- **Todo** (Generic Example): Single isolated context with no external dependencies
- Validates template infrastructure works independently of Analytics domain concepts

## Design Properties

- Algebraic sum type state machine
- Pure functional decision logic (no I/O in decide function)
- Total event application (evolve never fails)
- Idempotent commands for safe retry semantics
- Event-sourced projections (TodoListView derived from event stream)
