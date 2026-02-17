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
//! This endpoint immediately sends a `datastar-execute-script` event containing
//! `window.location.reload()`.
//!
//! On the initial page load, Datastar connects and receives the reload script,
//! but since the page just loaded this is a no-op in practice. When
//! `cargo-watch` rebuilds and restarts the server, the SSE connection drops.
//! Datastar's retry logic reconnects to `/reload`, receives the execute-script
//! event again, and this time it triggers a page reload that picks up the new
//! server's content.
//!
//! # Routes
//!
//! - `GET /reload` - SSE endpoint sending `window.location.reload()` via
//!   `datastar-execute-script` (PatchElements with inline script)

use std::convert::Infallible;

use axum::{
    Router,
    response::sse::{Event, Sse},
    routing::get,
};
use datastar::prelude::ExecuteScript;
use futures::stream::{self, Stream};

/// SSE endpoint that triggers a browser page reload on reconnection.
///
/// Sends a single `datastar-execute-script` event containing
/// `window.location.reload()`. The stream completes after the single event,
/// causing the SSE connection to close. Datastar's retry configuration
/// (set in the layout template) handles reconnection after server restart.
pub async fn reload_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let event: Event = ExecuteScript::new("window.location.reload()").into();

    Sse::new(stream::once(async move { Ok(event) }))
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
        let sse = reload_handler().await;
        // Handler returns successfully, producing an Sse wrapper.
        drop(sse);
    }
}
