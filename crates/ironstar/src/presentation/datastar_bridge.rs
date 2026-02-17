//! Datastar bridge for converting domain view state to SSE events.
//!
//! This module implements the transformation F: DomainState -> Vec<PatchEvent>
//! where each PatchEvent is either PatchElements (DOM mutations via rendered
//! HTML) or PatchSignals (reactive signal updates via JSON).
//!
//! The transformation is deterministic: identical state produces identical
//! events. The bridge connects hypertext's lazy rendering with Datastar's
//! HTML fragment morphing and signal patching.
//!
//! # Architecture
//!
//! The bridge sits between the domain view layer (pure state projections)
//! and the SSE transport layer (axum Event streams). Domain views produce
//! typed state; this module converts that state into the wire format
//! Datastar expects.
//!
//! ```text
//! DomainViewState -> ToDatastarEvents::to_datastar_events() -> Vec<Event>
//!                                                                  |
//!                                            SSE stream composition v
//! ```

use axum::response::sse::Event;
use datastar::prelude::PatchElements;
use hypertext::Renderable;
use ironstar_todo::TodoViewState;

use crate::presentation::todo_templates::{todo_footer, todo_list};

/// Trait for converting domain view state to Datastar-formatted SSE events.
///
/// Implements the transformation F: DomainState -> Vec<PatchEvent> where
/// each PatchEvent is either PatchElements (DOM mutations via rendered HTML)
/// or PatchSignals (reactive signal updates via JSON).
///
/// The transformation is deterministic: identical state produces identical
/// events. Empty or no-op state may produce no events.
pub trait ToDatastarEvents {
    /// Render the current state as a sequence of Datastar SSE events.
    fn to_datastar_events(&self) -> Vec<Event>;
}

/// Render a hypertext Renderable as a Datastar PatchElements event.
///
/// This is the primary bridge between hypertext's lazy rendering
/// and Datastar's HTML fragment morphing. The rendered HTML is sent
/// as a `datastar-patch-elements` SSE event that Datastar uses to
/// morph the DOM.
pub fn render_patch_elements(renderable: impl Renderable) -> Event {
    let html = renderable.render().into_inner();
    PatchElements::new(html).into()
}

/// Render a hypertext Renderable as a PatchElements event with
/// a CSS selector target and optional SSE event ID.
///
/// The selector determines which DOM element receives the patched
/// HTML. The optional ID enables `Last-Event-ID` resumption for
/// SSE reconnection.
pub fn render_patch_elements_with_selector(
    renderable: impl Renderable,
    selector: &str,
    id: Option<&str>,
) -> Event {
    let html = renderable.render().into_inner();
    let mut patch = PatchElements::new(html).selector(selector);
    if let Some(id) = id {
        patch = patch.id(id);
    }
    patch.into()
}

impl ToDatastarEvents for TodoViewState {
    fn to_datastar_events(&self) -> Vec<Event> {
        let mut events = Vec::new();

        // Render the todo list as a PatchElements event targeting #todo-list.
        // The todo_list template always renders a <ul id="todo-list"> element,
        // even when empty, so this event is always produced.
        events.push(render_patch_elements_with_selector(
            todo_list(&self.todos),
            "#todo-list",
            None,
        ));

        // Render the footer only when there are todos present.
        // This matches the conditional rendering in todo_app().
        if self.count > 0 {
            let active = self.active_count();
            events.push(render_patch_elements_with_selector(
                todo_footer(active, self.completed_count),
                "#todo-app footer",
                None,
            ));
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ironstar_todo::{TodoItemView, TodoViewState};
    use uuid::Uuid;

    fn sample_todo(text: &str, completed: bool) -> TodoItemView {
        TodoItemView {
            id: Uuid::new_v4(),
            text: text.to_string(),
            completed,
        }
    }

    #[test]
    fn empty_view_state_produces_list_event() {
        let state = TodoViewState::default();
        let events = state.to_datastar_events();

        // Empty state still renders the <ul> container, producing one event.
        // No footer is rendered because count == 0.
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn view_state_with_items_produces_list_and_footer_events() {
        let state = TodoViewState {
            todos: vec![
                sample_todo("Buy groceries", false),
                sample_todo("Walk the dog", false),
            ],
            count: 2,
            completed_count: 0,
        };
        let events = state.to_datastar_events();

        // Two events: one for the list, one for the footer.
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn view_state_with_completed_items_renders_clear_button() {
        let state = TodoViewState {
            todos: vec![
                sample_todo("Buy groceries", false),
                sample_todo("Walk the dog", true),
            ],
            count: 2,
            completed_count: 1,
        };
        let events = state.to_datastar_events();

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn render_patch_elements_produces_event() {
        use hypertext::prelude::*;

        let renderable = maud! {
            div { "Hello, world!" }
        };
        let _event = render_patch_elements(renderable);
        // Conversion succeeds without panic.
    }

    #[test]
    fn render_patch_elements_with_selector_sets_target() {
        use hypertext::prelude::*;

        let renderable = maud! {
            ul id="test-list" {
                li { "Item 1" }
            }
        };
        let _event = render_patch_elements_with_selector(renderable, "#test-list", None);
        // Conversion succeeds without panic.
    }

    #[test]
    fn render_patch_elements_with_selector_and_id() {
        use hypertext::prelude::*;

        let renderable = maud! {
            p { "Content" }
        };
        let _event = render_patch_elements_with_selector(renderable, "#target", Some("evt-42"));
        // Conversion succeeds without panic.
    }
}
