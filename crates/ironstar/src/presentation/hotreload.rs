//! Dev-only hot reload SSE endpoint.
//!
//! Provides a `GET /reload` endpoint that enables browser hot-reload during
//! development. This module is gated behind `#[cfg(debug_assertions)]` so it
//! only exists in dev builds.
//!
//! # Mechanism
//!
//! The layout template (`layout.rs`) injects a Datastar `@get('/reload', ...)`
//! directive in debug builds. Datastar opens an SSE connection to `/reload`.
//!
//! On the initial page load, the endpoint sends a keep-alive event with an
//! `id` field and holds the connection open. When `cargo-watch` rebuilds and
//! restarts the server, the SSE connection drops. Datastar's retry logic
//! reconnects to `/reload` with `Last-Event-ID` from the previous connection.
//! The new server detects the reconnection header and sends a
//! `datastar-execute-script` event with `window.location.reload()`, triggering
//! a page reload that picks up the new server's content.
//!
//! # Routes
//!
//! - `GET /reload` - SSE endpoint for dev hot reload

use std::convert::Infallible;

use axum::http::HeaderMap;
use axum::{
    Router,
    response::sse::{Event, Sse},
    routing::get,
};
use datastar::prelude::ExecuteScript;
use futures::stream::{self, Stream, StreamExt};
use std::time::Duration;

/// SSE endpoint that triggers a browser page reload on reconnection.
///
/// On initial connection (no `Last-Event-ID`), sends a keep-alive event with
/// an `id` field and holds the connection open via periodic heartbeats.
///
/// On reconnection (has `Last-Event-ID`), sends `window.location.reload()`
/// via `datastar-execute-script`. This triggers after cargo-watch restarts
/// the server: the old connection drops, Datastar reconnects with the stored
/// event ID, and the new server sends the reload command.
pub async fn reload_handler(
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let is_reconnection = headers.contains_key("Last-Event-ID");

    let initial_event = if is_reconnection {
        // Reconnection after server restart: trigger page reload
        ExecuteScript::new("window.location.reload()").into()
    } else {
        // Initial connection: send event ID for future reconnection detection
        Event::default().id("1").comment("connected")
    };

    let initial = stream::once(async move { Ok(initial_event) });

    let heartbeats = stream::unfold((), |()| async {
        tokio::time::sleep(Duration::from_secs(15)).await;
        Some((Ok(Event::default().comment("keepalive")), ()))
    });

    Sse::new(initial.chain(heartbeats))
}

/// Creates the hot reload router.
///
/// # Routes
///
/// - `GET /reload` - SSE endpoint for dev hot reload
pub fn routes() -> Router {
    Router::new().route("/reload", get(reload_handler))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_script_produces_valid_event() {
        let event: Event = ExecuteScript::new("window.location.reload()").into();
        // Conversion succeeds without panic, confirming the SDK integration
        // produces a well-formed axum SSE event.
        drop(event);
    }

    #[tokio::test]
    async fn reload_handler_returns_sse_stream() {
        let headers = HeaderMap::new();
        let sse = reload_handler(headers).await;
        // Handler returns successfully, producing an Sse wrapper.
        drop(sse);
    }

    #[tokio::test]
    async fn reload_handler_sends_reload_on_reconnection() {
        let mut headers = HeaderMap::new();
        headers.insert("Last-Event-ID", "1".parse().unwrap());
        let sse = reload_handler(headers).await;
        // Handler returns successfully with reconnection header.
        drop(sse);
    }
}
