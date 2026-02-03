//! View modules for read-side projections.
//!
//! Views are the read side of CQRS: pure functions that project events into
//! queryable state. Unlike Deciders, Views have no commands or errors — they
//! simply evolve state by folding events.
//!
//! # View vs Decider
//!
//! - **Decider**: `(Command, State) -> Result<Vec<Event>, Error>` + `(State, Event) -> State`
//! - **View**: `(State, Event) -> State` only
//!
//! Views reuse the evolve function pattern but drop command handling entirely.

pub mod catalog;
pub mod todo;

pub use catalog::{CatalogView, CatalogViewState, catalog_view};
pub use todo::{TodoView, TodoViewState, todo_view};
