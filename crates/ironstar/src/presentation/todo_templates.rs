//! Todo hypertext templates for Datastar-driven UI.
//!
//! These templates render HTML fragments for the Todo example application,
//! demonstrating Datastar integration with hypertext's lazy rendering.

use hypertext::prelude::*;

use crate::domain::signals::TodoItemView;
use crate::infrastructure::assets::AssetManifest;
use crate::presentation::components::{button, checkbox, loading_spinner, text_field};
use crate::presentation::layout::base_layout;

/// Complete todo page with base layout.
pub fn todo_page(manifest: &AssetManifest, todos: &[TodoItemView]) -> impl Renderable {
    let content = maud! {
        main class="center stack" {
            h1 { "Todo" }
            p class="text-2" { "A Datastar + Rust demonstration" }

            (todo_app(todos))
        }
    };

    base_layout(manifest, content)
}

/// Todo application container with SSE connection.
pub fn todo_app(todos: &[TodoItemView]) -> impl Renderable {
    let total = todos.len();
    let completed = todos.iter().filter(|t| t.completed).count();
    let active = total - completed;

    maud! {
        div
            id="todo-app"
            class="card stack"
            "data-init"="@get('/todos/api/feed')"
        {
            (add_todo_form())

            (todo_list(todos))

            @if total > 0 {
                (todo_footer(active, completed))
            }
        }
    }
}

/// Form for adding new todos.
fn add_todo_form() -> impl Renderable {
    maud! {
        form
            class="cluster"
            "data-on:submit__prevent"="@post('/todos/api', {body: {text: $input}}); $input = ''"
            "data-indicator"="isFetching"
        {
            (text_field(
                "input",
                "What needs to be done?",
                " ",
                "outlined",
                r#"data-bind:input"#
            ))
            (button(
                "Add",
                "filled",
                None,
                r#"type="submit""#
            ))
            (loading_spinner("isFetching"))
        }
    }
}

/// List of todo items.
pub fn todo_list(todos: &[TodoItemView]) -> impl Renderable {
    maud! {
        ul id="todo-list" class="stack" {
            @for todo in todos {
                (todo_item(todo))
            }
        }
    }
}

/// Single todo item row.
pub fn todo_item(todo: &TodoItemView) -> impl Renderable {
    let id = todo.id.to_string();
    let indicator_signal = format!("fetching_{}", todo.id);

    let checkbox_attrs = format!(
        r#"data-on:change="@post('/todos/api/{}/complete')" data-indicator="{}""#,
        id, indicator_signal
    );

    let delete_attrs = format!(
        r#"data-on:click="@delete('/todos/api/{}')" data-indicator="{}""#,
        id, indicator_signal
    );

    // maud's () interpolation automatically escapes content for XSS safety,
    // so we pass the raw text and let maud handle escaping
    let text = &todo.text;
    let completed = todo.completed;

    maud! {
        li
            id=(format!("todo-{}", id))
            class="cluster"
            "data-id"=(id)
        {
            (checkbox("completed", completed, &checkbox_attrs))

            @if completed {
                span class="completed text-strikethrough" {
                    (text)
                }
            } @else {
                span {
                    (text)
                }
            }

            (button(
                "Delete",
                "outlined",
                Some("small"),
                &delete_attrs
            ))

            (loading_spinner(&indicator_signal))
        }
    }
}

/// Footer with item count and filter buttons.
pub fn todo_footer(active: usize, completed: usize) -> impl Renderable {
    let item_word = if active == 1 { "item" } else { "items" };

    maud! {
        footer class="cluster" {
            span class="text-2" {
                strong { (active) }
                " " (item_word) " left"
            }

            @if completed > 0 {
                (button(
                    "Clear completed",
                    "outlined",
                    Some("small"),
                    r#"data-on:click="@delete('/todos/api/completed')""#
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample_todo(text: &str, completed: bool) -> TodoItemView {
        TodoItemView {
            id: Uuid::new_v4(),
            text: text.to_string(),
            completed,
        }
    }

    #[test]
    fn todo_list_renders_empty() {
        let todos: Vec<TodoItemView> = vec![];
        let html = todo_list(&todos).render();
        let body = html.as_inner();

        assert!(body.contains(r#"<ul id="todo-list""#));
        assert!(body.contains("</ul>"));
    }

    #[test]
    fn todo_list_renders_items() {
        let todos = vec![
            sample_todo("Buy groceries", false),
            sample_todo("Walk the dog", true),
        ];
        let html = todo_list(&todos).render();
        let body = html.as_inner();

        assert!(body.contains("Buy groceries"));
        assert!(body.contains("Walk the dog"));
        assert!(body.contains("<li"));
    }

    #[test]
    fn todo_item_escapes_xss() {
        let todo = sample_todo("<script>alert(1)</script>", false);
        let html = todo_item(&todo).render();
        let body = html.as_inner();

        // maud auto-escapes text content, preventing XSS
        assert!(!body.contains("<script>"));
        assert!(body.contains("&lt;script&gt;"));
    }

    #[test]
    fn add_todo_form_has_datastar_attrs() {
        let html = add_todo_form().render();
        let body = html.as_inner();

        assert!(body.contains("data-on:submit"));
        assert!(body.contains("data-bind:input"));
        assert!(body.contains(r#"data-indicator="isFetching""#));
    }

    #[test]
    fn todo_footer_shows_count() {
        let html = todo_footer(3, 2).render();
        let body = html.as_inner();

        assert!(body.contains("3"));
        assert!(body.contains("items"));
        assert!(body.contains("Clear completed"));
    }

    #[test]
    fn todo_footer_singular_item() {
        let html = todo_footer(1, 0).render();
        let body = html.as_inner();

        assert!(body.contains("1"));
        assert!(body.contains("item left"));
        assert!(!body.contains("Clear completed"));
    }
}
