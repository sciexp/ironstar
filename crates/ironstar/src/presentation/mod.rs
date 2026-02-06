//! Presentation layer - HTTP routes, handlers, and hypertext templates.
//!
//! This layer handles HTTP concerns: routing, request parsing, response
//! formatting, SSE streams, and HTML template rendering. Handlers are `async`
//! because HTTP I/O is inherently asynchronous.
//!
//! # Async handlers calling application layer
//!
//! Presentation handlers are async functions that:
//!
//! 1. Parse and validate HTTP requests (extractors)
//! 2. Call application layer commands/queries (async)
//! 3. Format responses as HTML fragments or SSE events
//!
//! ```rust,ignore
//! async fn create_todo(
//!     State(services): State<Arc<All>>,
//!     Form(input): Form<CreateTodoInput>,
//! ) -> Result<impl IntoResponse, AppError> {
//!     // Parse request into domain command
//!     let command = TodoCommand::Create {
//!         id: TodoId::new(),
//!         text: input.text,
//!     };
//!
//!     // Call application layer (async)
//!     let events = handle_create_todo(&*services, command).await?;
//!
//!     // Render HTML fragment response
//!     Ok(Html(render_todo_item(&events[0])))
//! }
//! ```
//!
//! # Datastar SSE integration
//!
//! Ironstar uses Datastar for reactive UI via Server-Sent Events. The
//! presentation layer is responsible for:
//!
//! - Establishing SSE connections (`text/event-stream`)
//! - Sending signal updates (`datastar-signal`)
//! - Sending HTML fragment updates (`datastar-fragment`)
//! - Handling `Last-Event-ID` for connection resumption
//!
//! ```rust,ignore
//! async fn todo_stream(
//!     State(services): State<Arc<All>>,
//!     headers: HeaderMap,
//! ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
//!     let last_id = headers
//!         .get("Last-Event-ID")
//!         .and_then(|v| v.to_str().ok())
//!         .and_then(|s| s.parse().ok());
//!
//!     let stream = services
//!         .event_bus()
//!         .subscribe_from(last_id)
//!         .map(|event| Ok(render_sse_event(&event)));
//!
//!     Sse::new(stream)
//! }
//! ```
//!
//! # Hypertext templates
//!
//! HTML rendering uses the `hypertext` crate for lazy template composition.
//! Templates are pure functions that produce HTML thunks, keeping rendering
//! logic testable and composable.
//!
//! ```rust,ignore
//! fn render_todo_item(todo: &TodoItemView) -> impl Render {
//!     maud_html! {
//!         li data-id=(todo.id) {
//!             input type="checkbox" checked[todo.completed];
//!             span { (todo.text) }
//!         }
//!     }
//! }
//! ```
//!
//! # Layer boundaries
//!
//! The presentation layer should NOT contain:
//!
//! - Business logic (belongs in [`crate::domain`])
//! - Database queries (belongs in [`crate::infrastructure`])
//! - Service orchestration (belongs in [`crate::application`])
//!
//! If a handler grows complex, extract logic into the application layer.
//!
//! # What belongs here
//!
//! - HTTP route definitions and handler functions
//! - Request extractors and response formatters
//! - SSE stream setup and event rendering
//! - HTML template functions (hypertext/maud)
//! - Static asset serving configuration
//! - Error response formatting
//! - Health check endpoints for infrastructure probes
//!
//! # What does NOT belong here
//!
//! - Domain types or business rules (belongs in [`crate::domain`])
//! - Database or network I/O (belongs in [`crate::infrastructure`])
//! - Multi-step workflows (belongs in [`crate::application`])
//! - Synchronous business logic (domain functions should be sync)

pub mod analytics;
pub mod bar_chart_transformer;
pub mod chart;
pub mod chart_templates;
pub mod chart_transformer;
pub mod components;
pub mod error;
pub mod extractors;
pub mod health;
pub mod layout;
pub mod metrics;
pub mod middleware;
pub mod todo;
pub mod todo_templates;
pub mod workspace;

pub use bar_chart_transformer::BarChartTransformer;
pub use chart::{
    astronauts_chart_page, astronauts_chart_sse, chart_feed_handler, routes as chart_routes,
};
pub use chart_templates::{chart_page, echarts_chart, echarts_chart_with_feedback};
pub use chart_transformer::{
    ChartConfig, ChartTransformer, ChartType, ColumnMetadata, QueryResult, TransformError,
};
pub use components::{button, checkbox, icon, loading_spinner, text_field};
pub use error::{AppError, AppErrorKind, ErrorResponse};
pub use extractors::{
    SESSION_COOKIE_NAME, SessionExtractor, SessionRejection, clear_session_cookie, session_cookie,
};
pub use health::{
    HealthChecks, HealthResponse, HealthState, HealthStatus, health_router, routes as health_routes,
};
pub use metrics::{MetricsState, metrics_handler};
pub use middleware::MakeRequestUuidV7;
pub use todo::{TodoAppState, TodoListResponse, get_todo, list_todos};
pub use todo_templates::{todo_app, todo_item, todo_list, todo_page};

use crate::infrastructure::create_static_router;
use crate::state::AppState;
use axum::Router;
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

/// Compose the application router from all feature routers.
///
/// Router composition follows a clear hierarchy:
/// - Health endpoints at root (/health/*)
/// - Todo feature at /todos
/// - Chart feature at /charts
/// - Static assets at /static
///
/// Each feature router uses `Router<AppState>` and handlers extract
/// domain-specific state via `FromRef`.
///
/// # Middleware stack (outermost first)
///
/// 1. `SetRequestIdLayer` — generates UUID v7 request ID (or preserves existing)
/// 2. `TraceLayer` — creates a tracing span per request with method, URI, and request_id
/// 3. `PropagateRequestIdLayer` — copies request ID to response header
pub fn app_router(state: AppState) -> Router {
    let x_request_id = http::HeaderName::from_static("x-request-id");

    // Compose stateful feature routers and apply state
    let stateful = Router::new()
        .merge(health::routes())
        .merge(metrics::routes())
        .nest("/todos", todo::routes())
        .nest("/analytics", analytics::routes())
        .nest("/charts", chart::routes())
        .nest("/workspace", workspace::routes())
        .with_state(state);

    // Merge stateless static router after state is applied, then add
    // observability middleware. Layer order matters: layers added last
    // execute first on requests.
    stateful
        .merge(create_static_router())
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        .layer(TraceLayer::new_for_http().make_span_with(
            |request: &http::Request<axum::body::Body>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("-");

                tracing::info_span!(
                    "http.request",
                    http.request.method = %request.method(),
                    url.path = %request.uri().path(),
                    request_id = %request_id,
                )
            },
        ))
        .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuidV7))
}
