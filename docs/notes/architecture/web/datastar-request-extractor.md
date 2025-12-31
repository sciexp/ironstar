---
title: DatastarRequest extractor
---

# DatastarRequest extractor

> **Semantic foundation**: The DatastarRequest extractor implements profunctor dimap.
> It transforms the input (HTTP request → boolean) to determine which output transformation applies (HTML vs SSE).
> This enables the application to be bifunctorial: one functor for initial page loads, another for SSE updates.
> See [semantic-model.md § The complete profunctor](../core/semantic-model.md#the-complete-profunctor).

The DatastarRequest extractor is a critical pattern for implementing progressive enhancement in Datastar applications.
It enables handlers to distinguish between full HTML page loads (browser navigation) and SSE fragment updates (Datastar interactions).

**Important**: The official `datastar-rust` crate does **not** provide this extractor.
Without it, you cannot implement proper progressive enhancement — your application will fail to serve full HTML on initial page loads and break without JavaScript enabled.

## The problem

Datastar applications need to handle two distinct request types with different response requirements:

1. **Initial page load (browser navigation)**: Client expects a complete HTML document with `<!DOCTYPE html>`, `<head>`, `<body>`, and all necessary structure for rendering the page.
2. **Datastar update (SSE fragment)**: Client expects an SSE stream containing only HTML fragments and signal patches for morphing into the existing DOM.

The server must detect which type of request it's handling to respond appropriately.
Serving an SSE stream to a browser navigation request results in broken pages.
Serving full HTML to a Datastar update wastes bandwidth and breaks morphing.

## The pattern

Datastar automatically includes the `datastar-request: true` header on all SSE requests initiated through `data-on-*` attributes or programmatic `sse()` calls.
Initial page loads from browser navigation lack this header.

The DatastarRequest extractor checks for the presence of this header:

```
Has "datastar-request: true" header?
  ├─ Yes → SSE fragment response
  └─ No  → Full HTML page response
```

This pattern enables progressive enhancement: the application works with JavaScript disabled (serving full HTML pages on every navigation) while gaining reactivity benefits when JavaScript is enabled (serving efficient SSE fragment updates).

## Implementation

The DatastarRequest extractor is implemented as an axum extractor using `FromRequestParts`:

```rust
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

/// Extractor that detects Datastar SSE requests via the "datastar-request: true" header.
///
/// Returns `true` when the request is a Datastar SSE update, `false` for initial page loads.
///
/// # Usage
///
/// ```rust
/// async fn handler(DatastarRequest(is_datastar): DatastarRequest) -> Response {
///     if is_datastar {
///         // Return SSE stream with fragments
///     } else {
///         // Return full HTML page
///     }
/// }
/// ```
pub struct DatastarRequest(pub bool);

#[async_trait]
impl<S> FromRequestParts<S> for DatastarRequest
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        const DATASTAR_HEADER: &str = "datastar-request";

        let is_datastar = parts
            .headers
            .get(DATASTAR_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "true")
            .unwrap_or(false);

        Ok(DatastarRequest(is_datastar))
    }
}
```

**Key design choices:**

- **Never fails**: Returns `Ok(DatastarRequest(false))` when the header is missing or malformed, ensuring the handler always receives a value.
- **Boolean value**: Extracts to `bool` for ergonomic destructuring in handler signatures.
- **Header constant**: The header name `"datastar-request"` matches `DATASTAR_REQ_HEADER_STR` from `datastar-rust/src/consts.rs`.

## Usage in handlers

Progressive enhancement pattern with conditional response types:

```rust
use axum::{
    response::{Html, Sse, Response, IntoResponse},
    http::StatusCode,
};
use tokio_stream::StreamExt as _;

async fn todo_list_handler(
    DatastarRequest(is_datastar): DatastarRequest,
    State(app): State<AppState>,
) -> Result<Response, StatusCode> {
    if is_datastar {
        // Datastar SSE update: stream fragments and signal patches
        let stream = app.todo_events()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Sse::new(stream).into_response())
    } else {
        // Initial page load: serve complete HTML document
        let html = render_full_page(app)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Html(html).into_response())
    }
}
```

**Handler responsibilities by request type:**

| Request Type | Response | Content |
|--------------|----------|---------|
| Initial load (`is_datastar = false`) | `Html<String>` | Complete HTML document with `<!DOCTYPE>`, `<head>`, `<body>`, initial state |
| Datastar update (`is_datastar = true`) | `Sse<Stream>` | Fragment-only HTML with signal patches, no document structure |

## Integration with hypertext templates

The DatastarRequest extractor integrates naturally with hypertext's lazy rendering model.
Define separate rendering functions for full pages vs fragments:

```rust
use hypertext::{html_elements, maud_move, Renderable};

/// Full page render: includes document structure, head, scripts
fn render_full_page(todos: Vec<Todo>) -> impl Renderable {
    maud_move! {
        (hypertext::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "Todo App" }
                script src="/static/datastar.js" {}
                link rel="stylesheet" href="/static/main.css";
            }
            body {
                (render_todo_fragment(todos))
            }
        }
    }
}

/// Fragment render: only the dynamic content, no document structure
fn render_todo_fragment(todos: Vec<Todo>) -> impl Renderable {
    maud_move! {
        div id="todo-list" {
            @for todo in todos {
                div class="todo-item" {
                    input type="checkbox" checked[todo.completed];
                    span { (todo.title) }
                }
            }
        }
    }
}

// Handler uses DatastarRequest to choose which template
async fn todos_handler(
    DatastarRequest(is_datastar): DatastarRequest,
    State(todos): State<Vec<Todo>>,
) -> Html<String> {
    let rendered = if is_datastar {
        render_todo_fragment(todos).render()
    } else {
        render_full_page(todos).render()
    };

    Html(rendered)
}
```

**Template composition guidelines:**

- **Extract fragments**: The fragment template should be a self-contained function that renders only the dynamic portion.
- **Compose pages**: The full page template should include the fragment template, not duplicate its content.
- **Lazy evaluation**: hypertext templates are thunks — they only render when `.render()` is called, making conditional rendering efficient.

## Relationship to ReadSignals extractor

DatastarRequest and ReadSignals serve complementary roles:

| Extractor | Purpose | Detects |
|-----------|---------|---------|
| `DatastarRequest` | Response format selection | Whether to serve full HTML vs SSE fragments |
| `ReadSignals<T>` | Signal deserialization | Datastar state sent from client to server |

ReadSignals also checks the `datastar-request` header internally (via `OptionalFromRequest`) but its purpose is extracting signal data, not controlling response format.

**Typical usage pattern:**

```rust
async fn update_todo_handler(
    DatastarRequest(is_datastar): DatastarRequest,
    ReadSignals(signals): ReadSignals<TodoSignals>,
    State(app): State<AppState>,
) -> Result<Response, StatusCode> {
    // ReadSignals provides the client state
    app.update_todo(signals.todo_id, signals.completed).await?;

    // DatastarRequest determines the response format
    if is_datastar {
        Ok(Sse::new(app.todo_fragment_stream()).into_response())
    } else {
        // Fallback for non-JS clients: redirect to show updated state
        Ok(Redirect::to("/todos").into_response())
    }
}
```

**When ReadSignals extraction fails** (missing header or malformed data), the request is not a valid Datastar update.
In this case, you should return an appropriate error response or redirect, not attempt to serve SSE fragments.

## Testing

Test both request types to ensure progressive enhancement works correctly:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_initial_page_load() {
        let app = todo_routes();

        let request = Request::builder()
            .uri("/todos")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/html");

        let body = body_to_string(response).await;
        assert!(body.contains("<!DOCTYPE html>"));
        assert!(body.contains("<head>"));
    }

    #[tokio::test]
    async fn test_datastar_update() {
        let app = todo_routes();

        let request = Request::builder()
            .uri("/todos")
            .header("datastar-request", "true")
            .header("Accept", "text/event-stream")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/event-stream");

        let body = body_to_string(response).await;
        assert!(!body.contains("<!DOCTYPE html>"));
        assert!(body.contains("event: datastar-patch-elements"));
    }
}
```

## Summary

The DatastarRequest extractor is an **essential pattern** for ironstar applications:

- **Not provided by datastar-rust**: You must implement this extractor yourself.
- **Enables progressive enhancement**: Applications work without JavaScript, gain reactivity with JavaScript enabled.
- **Simple implementation**: Single header check with boolean result.
- **Integrates with hypertext**: Compose full pages from fragment templates using lazy rendering.
- **Complements ReadSignals**: DatastarRequest controls response format; ReadSignals extracts client state.

Without this pattern, your application cannot distinguish between browser navigation and Datastar updates, breaking the fundamental hypermedia contract.
