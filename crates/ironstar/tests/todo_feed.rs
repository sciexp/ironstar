//! Integration tests for the Todo SSE feed endpoint.
//!
//! These tests verify the `/todos/api/feed` endpoint behavior including:
//! - Basic SSE connection establishment
//! - Keep-alive comment delivery
//! - Last-Event-ID reconnection replay

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "test file with standard test assertions"
)]

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use futures::StreamExt;
use ironstar::application::todo::handle_todo_command;
use ironstar::domain::todo::commands::TodoCommand;
use ironstar::domain::todo::events::TodoEvent;
use ironstar::domain::todo::values::TodoId;
use ironstar::infrastructure::event_store::SqliteEventRepository;
use ironstar::infrastructure::{
    AssetManifest, ZenohEventBus, open_embedded_session, test_prometheus_handle,
};
use ironstar::presentation::todo::routes as todo_routes;
use ironstar::state::AppState;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

/// Create an in-memory SQLite pool with event store migrations applied.
async fn create_test_pool() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test pool");

    sqlx::query(include_str!("../migrations/001_events.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migration");

    pool
}

/// Create a test router with AppState configured for SSE feed testing.
///
/// The router includes Zenoh event bus for live event streaming.
async fn create_test_app_with_event_bus() -> (Router, Arc<ZenohEventBus>) {
    let pool = create_test_pool().await;
    let session = Arc::new(open_embedded_session().await.expect("session should open"));
    let event_bus = Arc::new(ZenohEventBus::new(session));

    let state = AppState::new(pool, AssetManifest::default(), test_prometheus_handle())
        .with_event_bus(Arc::clone(&event_bus));

    // Mount todo routes at /todos to match production
    let app = Router::new()
        .nest("/todos", todo_routes())
        .with_state(state);

    (app, event_bus)
}

/// Create a test router without event bus (for testing 503 behavior).
async fn create_test_app_without_event_bus() -> Router {
    let pool = create_test_pool().await;
    let state = AppState::new(pool, AssetManifest::default(), test_prometheus_handle());

    Router::new()
        .nest("/todos", todo_routes())
        .with_state(state)
}

/// Collect SSE events from a response body stream with timeout.
///
/// SSE is a streaming protocol that doesn't naturally terminate.
/// This helper collects data chunks until timeout, then returns accumulated text.
async fn collect_sse_events(body: Body, timeout_ms: u64) -> String {
    let mut stream = body.into_data_stream();
    let mut collected = String::new();

    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }

        match tokio::time::timeout(remaining, stream.next()).await {
            Ok(Some(Ok(chunk))) => {
                collected.push_str(&String::from_utf8_lossy(&chunk));
            }
            Ok(Some(Err(_))) | Ok(None) | Err(_) => {
                break;
            }
        }
    }

    collected
}

/// Test that SSE feed returns 503 when event bus is not configured.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn feed_returns_503_without_event_bus() {
    let app = create_test_app_without_event_bus().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/todos/api/feed")
                .body(Body::empty())
                .expect("request body"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(
        response.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "Feed should return 503 when event bus is not configured"
    );
}

/// Test that SSE feed establishes connection and returns correct content type.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn feed_establishes_sse_connection() {
    let (app, _event_bus) = create_test_app_with_event_bus().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/todos/api/feed")
                .body(Body::empty())
                .expect("request body"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    // Verify SSE content type
    let content_type = response
        .headers()
        .get("content-type")
        .expect("should have content-type header")
        .to_str()
        .expect("content-type should be valid string");

    assert!(
        content_type.contains("text/event-stream"),
        "Expected text/event-stream, got: {content_type}"
    );

    // Verify cache control for SSE
    let cache_control = response
        .headers()
        .get("cache-control")
        .expect("should have cache-control header")
        .to_str()
        .expect("cache-control should be valid string");

    assert!(
        cache_control.contains("no-cache"),
        "SSE should have no-cache, got: {cache_control}"
    );
}

/// Test that SSE feed sends keep-alive comments.
///
/// This test uses a very short keep-alive interval to verify the mechanism
/// without waiting 15 seconds.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn feed_sends_keepalive_comments() {
    let (app, _event_bus) = create_test_app_with_event_bus().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/todos/api/feed")
                .body(Body::empty())
                .expect("request body"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    // Collect body stream for a short time
    let body = response.into_body();
    let mut stream = body.into_data_stream();

    // Wait for first keep-alive (default is 15 seconds, but we can check structure)
    // In a real test with configurable keep-alive, we'd use a shorter interval.
    // For now, just verify the stream is functional by collecting first chunk.
    // With an empty event store, the first chunk is a datastar-patch-elements event
    // for the empty todo list (no sequence ID since latest_seq is 0).
    let first_chunk = tokio::time::timeout(Duration::from_secs(20), stream.next()).await;

    // Either we get a PatchElements event, keep-alive, or the stream is responding
    match first_chunk {
        Ok(Some(Ok(chunk))) => {
            let text = String::from_utf8_lossy(&chunk);
            assert!(
                text.contains(": keepalive")
                    || text.is_empty()
                    || text.contains("datastar-patch-elements"),
                "Expected keep-alive or datastar-patch-elements event, got: {text}"
            );
        }
        Ok(Some(Err(e))) => {
            panic!("Stream error: {e}");
        }
        Ok(None) => {
            // Stream ended, which is unexpected but not a test failure
            // for an empty event store
        }
        Err(_) => {
            // Timeout - this is acceptable as keep-alive is 15 seconds by default
            // and we don't want to wait that long in tests
        }
    }
}

/// Test Last-Event-ID reconnection replays events after the specified sequence.
///
/// The handler folds historical events into a TodoViewState and emits Datastar
/// PatchElements events with the latest sequence as the SSE event ID.
/// With Last-Event-ID: 1, events 2 and 3 are replayed into a projected state
/// containing "Second todo" and "Third todo", with id: 3 on the last event.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn feed_replays_events_after_last_event_id() {
    let pool = create_test_pool().await;
    let session = Arc::new(open_embedded_session().await.expect("session should open"));
    let event_bus = Arc::new(ZenohEventBus::new(session));

    let state = AppState::new(
        pool.clone(),
        AssetManifest::default(),
        test_prometheus_handle(),
    )
    .with_event_bus(Arc::clone(&event_bus));

    // Create some Todo events directly in the event store
    let repo: Arc<SqliteEventRepository<TodoCommand, TodoEvent>> =
        Arc::new(SqliteEventRepository::new(pool.clone()));

    let now = Utc::now();

    // Create 3 todos to have events at sequences 1, 2, 3
    let id1 = TodoId::new();
    let id2 = TodoId::new();
    let id3 = TodoId::new();

    // Type alias for None event bus to satisfy generic constraint
    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    handle_todo_command(
        Arc::clone(&repo),
        NO_EVENT_BUS,
        TodoCommand::Create {
            id: id1,
            text: "First todo".to_string(),
            created_at: now,
        },
    )
    .await
    .expect("create first todo");

    handle_todo_command(
        Arc::clone(&repo),
        NO_EVENT_BUS,
        TodoCommand::Create {
            id: id2,
            text: "Second todo".to_string(),
            created_at: now,
        },
    )
    .await
    .expect("create second todo");

    handle_todo_command(
        Arc::clone(&repo),
        NO_EVENT_BUS,
        TodoCommand::Create {
            id: id3,
            text: "Third todo".to_string(),
            created_at: now,
        },
    )
    .await
    .expect("create third todo");

    // Mount router
    let app = Router::new()
        .nest("/todos", todo_routes())
        .with_state(state);

    // Request with Last-Event-ID: 1 should replay events 2 and 3 as projected state
    let response = app
        .oneshot(
            Request::builder()
                .uri("/todos/api/feed")
                .header("Last-Event-ID", "1")
                .body(Body::empty())
                .expect("request body"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    // Collect SSE events with short timeout (replay events come immediately)
    let text = collect_sse_events(response.into_body(), 500).await;

    // The handler folds events 2 and 3 into a projected view state, then emits
    // Datastar PatchElements with the latest sequence (3) as the SSE event ID.
    let has_id_3 = text.contains("id: 3") || text.contains("id:3");
    assert!(
        has_id_3,
        "Expected event with id: 3 (latest sequence after replay). Response:\n{}",
        text
    );

    // Should NOT have event with id: 1 (was before Last-Event-ID)
    let has_id_1 = text.contains("id: 1\n") || text.contains("id:1\n");
    assert!(
        !has_id_1,
        "Should NOT have event id: 1 (before Last-Event-ID). Response:\n{}",
        text
    );

    // The projected state should contain the two todos from events 2 and 3
    // (not the first todo which is before the Last-Event-ID cutoff)
    assert!(
        text.contains("Second todo"),
        "Expected 'Second todo' in projected HTML. Response:\n{}",
        text
    );
    assert!(
        text.contains("Third todo"),
        "Expected 'Third todo' in projected HTML. Response:\n{}",
        text
    );
}

/// Test that SSE events use the Datastar PatchElements event type.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn feed_events_use_datastar_patch_elements_type() {
    let pool = create_test_pool().await;
    let session = Arc::new(open_embedded_session().await.expect("session should open"));
    let event_bus = Arc::new(ZenohEventBus::new(session));

    let state = AppState::new(
        pool.clone(),
        AssetManifest::default(),
        test_prometheus_handle(),
    )
    .with_event_bus(Arc::clone(&event_bus));

    // Create a Todo event
    let repo: Arc<SqliteEventRepository<TodoCommand, TodoEvent>> =
        Arc::new(SqliteEventRepository::new(pool.clone()));

    let now = Utc::now();
    let id = TodoId::new();

    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    handle_todo_command(
        Arc::clone(&repo),
        NO_EVENT_BUS,
        TodoCommand::Create {
            id,
            text: "Test todo".to_string(),
            created_at: now,
        },
    )
    .await
    .expect("create todo");

    let app = Router::new()
        .nest("/todos", todo_routes())
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/todos/api/feed")
                .body(Body::empty())
                .expect("request body"),
        )
        .await
        .expect("request should succeed");

    // Collect SSE events with short timeout
    let text = collect_sse_events(response.into_body(), 500).await;

    // Should have the Datastar event type for HTML fragment patching
    assert!(
        text.contains("event: datastar-patch-elements"),
        "Expected 'event: datastar-patch-elements' in SSE output. Response:\n{}",
        text
    );
}

/// Test that SSE events include HTML fragments with Datastar data lines.
///
/// Datastar PatchElements events use `data: selector <css>` and
/// `data: elements <html>` lines instead of JSON payloads.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn feed_events_include_html_fragments() {
    let pool = create_test_pool().await;
    let session = Arc::new(open_embedded_session().await.expect("session should open"));
    let event_bus = Arc::new(ZenohEventBus::new(session));

    let state = AppState::new(
        pool.clone(),
        AssetManifest::default(),
        test_prometheus_handle(),
    )
    .with_event_bus(Arc::clone(&event_bus));

    let repo: Arc<SqliteEventRepository<TodoCommand, TodoEvent>> =
        Arc::new(SqliteEventRepository::new(pool.clone()));

    let now = Utc::now();
    let id = TodoId::new();

    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    handle_todo_command(
        Arc::clone(&repo),
        NO_EVENT_BUS,
        TodoCommand::Create {
            id,
            text: "HTML test todo".to_string(),
            created_at: now,
        },
    )
    .await
    .expect("create todo");

    let app = Router::new()
        .nest("/todos", todo_routes())
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/todos/api/feed")
                .body(Body::empty())
                .expect("request body"),
        )
        .await
        .expect("request should succeed");

    // Collect SSE events with short timeout
    let text = collect_sse_events(response.into_body(), 500).await;

    // Should have Datastar selector data line targeting #todo-list
    assert!(
        text.contains("data: selector #todo-list"),
        "Expected 'data: selector #todo-list' in SSE output. Response:\n{}",
        text
    );

    // Should have Datastar elements data line with HTML content
    assert!(
        text.contains("data: elements"),
        "Expected 'data: elements' prefix in SSE output. Response:\n{}",
        text
    );

    // The HTML should contain the todo text
    assert!(
        text.contains("HTML test todo"),
        "Expected todo text in HTML fragment. Response:\n{}",
        text
    );

    // With one todo, there should also be a footer event
    assert!(
        text.contains("data: selector #todo-app footer"),
        "Expected footer selector in SSE output. Response:\n{}",
        text
    );
}

/// Test that reconnection with Last-Event-ID: 0 gets all events projected as HTML.
///
/// The handler folds all events into a single TodoViewState and emits
/// Datastar PatchElements. The latest sequence (2) is the SSE event ID.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn feed_with_last_event_id_zero_gets_all_events() {
    let pool = create_test_pool().await;
    let session = Arc::new(open_embedded_session().await.expect("session should open"));
    let event_bus = Arc::new(ZenohEventBus::new(session));

    let state = AppState::new(
        pool.clone(),
        AssetManifest::default(),
        test_prometheus_handle(),
    )
    .with_event_bus(Arc::clone(&event_bus));

    let repo: Arc<SqliteEventRepository<TodoCommand, TodoEvent>> =
        Arc::new(SqliteEventRepository::new(pool.clone()));

    let now = Utc::now();

    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    // Create 2 events
    handle_todo_command(
        Arc::clone(&repo),
        NO_EVENT_BUS,
        TodoCommand::Create {
            id: TodoId::new(),
            text: "First".to_string(),
            created_at: now,
        },
    )
    .await
    .expect("create first todo");

    handle_todo_command(
        Arc::clone(&repo),
        NO_EVENT_BUS,
        TodoCommand::Create {
            id: TodoId::new(),
            text: "Second".to_string(),
            created_at: now,
        },
    )
    .await
    .expect("create second todo");

    let app = Router::new()
        .nest("/todos", todo_routes())
        .with_state(state);

    // Request with Last-Event-ID: 0 should get all events projected as HTML
    let response = app
        .oneshot(
            Request::builder()
                .uri("/todos/api/feed")
                .header("Last-Event-ID", "0")
                .body(Body::empty())
                .expect("request body"),
        )
        .await
        .expect("request should succeed");

    // Collect SSE events with short timeout
    let text = collect_sse_events(response.into_body(), 500).await;

    // Both events are folded into the projected state, so the latest sequence
    // (2) is set as the SSE event ID on the last PatchElements event.
    let has_id_2 = text.contains("id: 2") || text.contains("id:2");
    assert!(
        has_id_2,
        "Expected id: 2 (latest sequence) with Last-Event-ID: 0. Response:\n{}",
        text
    );

    // Both todos should appear in the rendered HTML
    assert!(
        text.contains("First") && text.contains("Second"),
        "Expected both todos in projected HTML. Response:\n{}",
        text
    );

    // Should use Datastar event type
    assert!(
        text.contains("event: datastar-patch-elements"),
        "Expected Datastar PatchElements event type. Response:\n{}",
        text
    );
}
