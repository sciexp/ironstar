# Integration patterns for third-party libraries with Datastar

Datastar's core principle is *server-driven UI*: the server sends HTML fragments and signal updates via SSE, and the browser reflects this state.
This creates a challenge when integrating third-party libraries that manage their own DOM.
Vega-Lite charts, drag-and-drop libraries, rich text editors, and similar components expect to control their own rendering.

This document establishes core patterns for preserving Datastar's server-driven philosophy while enabling integration with these DOM-owning libraries.
For specific visualization library implementations (Vega-Lite, ECharts, Mosaic), see `integration-patterns-visualizations.md`.
For complete ECharts implementation details, see `ds-echarts-integration-guide.md`.

---

## The fundamental tension

Datastar uses morphing to update the DOM: it compares incoming HTML with the current DOM and applies minimal changes, preserving element state.
This works beautifully for server-rendered content but fails when a third-party library has modified the DOM in ways the server does not know about.

Consider a chart library that creates SVG elements inside a container.
When the server sends a new HTML fragment to morph, Datastar sees that the container's children differ from what it expected and attempts to reconcile them.
This destroys the chart's internal structure.

The solution requires two ingredients:

1. A mechanism to exclude elements from morphing (allowing the library to own its DOM subtree)
2. A communication pattern where the library can receive updates and emit events without violating the server-driven model

---

## Pattern 1: Web component thin wrapper

Web components provide a natural encapsulation boundary for third-party libraries.
The key insight is that the web component should be *thin*: it delegates all state management to Datastar signals while wrapping only the imperative setup and teardown of the library.

### Algebraic perspective

From a categorical viewpoint, a thin web component wrapper is a *morphism* in the category of DOM updates.
It transforms signal updates (objects in the Datastar signal category) into DOM mutations (objects in the library's internal category), while dispatching custom events that transform back into signal updates.

This forms a bidirectional functor between the signal world and the imperative library world:

```
Signals ──data-attr:*──▶ Web Component ──library API──▶ Library DOM
                              │
                              ◀──custom events──
                              │
Signals ◀──data-on:*─── Event Dispatch ◀──callbacks──
```

The web component is pure in the functional sense: given the same attribute values, it produces the same library configuration.
Side effects (the actual DOM manipulation) are isolated within the component's lifecycle methods.

### Implementation pattern

A thin wrapper follows this structure:

```typescript
class ThirdPartyWrapper extends HTMLElement {
  private instance: LibraryInstance | null = null;

  // Observe attributes that Datastar will set via data-attr:*
  static observedAttributes = ['config', 'data-url'];

  connectedCallback() {
    // Initialize the library when element enters the DOM
    this.instance = ThirdPartyLib.create(this, this.getConfig());

    // Bridge library callbacks to custom events
    this.instance.on('selection', (value) => {
      this.dispatchEvent(new CustomEvent('select', {
        detail: value,
        bubbles: true
      }));
    });
  }

  disconnectedCallback() {
    // Critical: clean up to prevent memory leaks
    this.instance?.destroy();
    this.instance = null;
  }

  attributeChangedCallback(name: string, oldValue: string, newValue: string) {
    if (!this.instance || oldValue === newValue) return;

    // Update library via its API, not by re-creating
    switch (name) {
      case 'config':
        this.instance.updateConfig(JSON.parse(newValue));
        break;
      case 'data-url':
        this.loadData(newValue);
        break;
    }
  }

  private getConfig(): object {
    const config = this.getAttribute('config');
    return config ? JSON.parse(config) : {};
  }

  private async loadData(url: string) {
    const data = await fetch(url).then(r => r.json());
    this.instance?.setData(data);
  }
}

customElements.define('third-party-wrapper', ThirdPartyWrapper);
```

### Datastar integration

```html
<third-party-wrapper
  data-ignore-morph
  data-attr:config="JSON.stringify($chartConfig)"
  data-attr:data-url="$dataEndpoint"
  data-on:select="$selection = evt.detail"
></third-party-wrapper>
```

Key attributes:

- `data-ignore-morph` prevents Datastar from morphing the component's children, allowing the library to own its DOM subtree
- `data-attr:*` binds signal values to HTML attributes, triggering `attributeChangedCallback`
- `data-on:*` listens for custom events and updates signals accordingly

The component itself holds no state visible to Datastar.
All state flows through signals, and the component merely translates between the signal world and the imperative library world.

---

## Pattern 1.5: When Lit is appropriate

**Note:** This is the standard pattern for ANY TypeScript library integration (not just ECharts).
Use Lit instead of vanilla web components when you need lifecycle management, reactive properties, or observer coordination for libraries like ECharts, D3, Plotly, Three.js, Mapbox GL, etc.

Use Lit instead of vanilla web components when ALL of these conditions apply:

1. **Complex internal state**: Library manages significant internal state (ECharts scales, animations, selections)
2. **Multiple lifecycle observers**: Coordination needed for ResizeObserver, MediaQueryList, IntersectionObserver
3. **Light DOM acceptable**: Required for Open Props CSS token inheritance
4. **Isolated reactivity**: Lit's reactivity is internal to component, not competing with Datastar signals

### Canonical example: ds-echarts

The ds-echarts component from northstar demonstrates this pattern perfectly:

**Source**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/libs/lit/src/components/ds-echarts/ds-echarts.ts`

**Ironstar implementation**: See `ds-echarts-integration-guide.md` for the complete ironstar adaptation.

```typescript
@customElement('ds-echarts')
export class DsEcharts extends LitElement {
  @property({ type: String }) option = '{}'
  @property({ type: String }) theme = 'default'
  @property({ type: Number, attribute: 'resize-delay' }) resizeDelay = 100

  private chart: ECharts | null = null
  private resizeObserver: ResizeObserver | null = null
  private mediaQueryHandler: ((e: MediaQueryListEvent) => void) | null = null

  // Light DOM for Open Props token access
  protected createRenderRoot() {
    return this
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this.resizeObserver?.disconnect()
    clearTimeout(this.resizeTimeout)
    if (this.mediaQueryHandler) {
      window.matchMedia('(prefers-color-scheme: dark)')
        .removeEventListener('change', this.mediaQueryHandler)
    }
    this.chart?.dispose()
  }
}
```

**Why Lit here**:

- **ResizeObserver** with debouncing (configurable via `resize-delay` prop)
- **MediaQueryList** listener for automatic dark mode theme switching
- **ECharts lifecycle** coordination (init → setOption → resize → dispose)
- **Custom events** bridging ECharts interactions to Datastar signals (chart-click, chart-ready, etc.)
- **Light DOM** via `createRenderRoot() { return this }` for Open Props token access

A vanilla implementation would require manually tracking previous property values and managing all observer cleanup in `disconnectedCallback`.
Lit's lifecycle hooks eliminate this boilerplate while preserving the thin wrapper pattern.

### Key architectural principle

Lit's reactivity in this pattern is *not* redundant with Datastar because:

- Datastar owns application state via signals (chart configuration, data sources, user selections)
- Lit handles only *component-internal* reactivity (attribute changes → library API calls, observer setup/teardown)
- The component remains pure: given the same attributes, it produces the same library configuration
- Custom events bridge library callbacks back to Datastar signals via `data-on:*`

This is a functor between categories, not competing reactive systems:

```
Datastar Signals ──data-attr:option──▶ Lit @property ──setOption()──▶ ECharts Instance
                                            │
                                            ◀──chart events──
                                            │
Datastar Signals ◀──data-on:*─────── CustomEvent ◀──ECharts callbacks──
```

### Light DOM requirement

All Lit components in this pattern must use Light DOM via `createRenderRoot() { return this }` to inherit Open Props CSS custom properties.
See `css-architecture.md` for detailed rationale and implementation patterns.

### Build integration

Lit components can be bundled via either Rolldown or esbuild.
See `frontend-build-pipeline.md` for configuration options including the proven esbuild pattern from the Northstar template.

### Complete implementation reference

See `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/docs/notes/echarts/datastar-lit-echarts-component-guide.md` for complete implementation details including:
- Component property reference
- Event system design
- Hypertext template patterns
- Axum SSE handler integration

### Source code references

- **Lit framework**: `~/projects/lakescope-workspace/lit-web-components/` (core library source)
- **Northstar examples**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/libs/lit/src/components/`
- **Datastar integration**: `~/projects/lakescope-workspace/datastar/` (SSE protocol, `data-attr:*` binding semantics)
- **esbuild bundler**: `~/projects/lakescope-workspace/esbuild/` (Go-based bundler used in Northstar)

---

## Algebraic perspective on web components

From a categorical viewpoint, a thin web component wrapper is a *morphism* in the category of DOM updates.
It transforms signal updates (objects in the Datastar signal category) into DOM mutations (objects in the library's internal category), while dispatching custom events that transform back into signal updates.

This forms a bidirectional functor between the signal world and the imperative library world:

```
Signals ──data-attr:*──▶ Web Component ──library API──▶ Library DOM
                              │
                              ◀──custom events──
                              │
Signals ◀──data-on:*─── Event Dispatch ◀──callbacks──
```

The web component is pure in the functional sense: given the same attribute values, it produces the same library configuration.
Side effects (the actual DOM manipulation) are isolated within the component's lifecycle methods.

This principle holds regardless of whether you use vanilla web components (Pattern 1) or Lit wrappers (Pattern 1.5).
The choice between patterns affects implementation ergonomics, not the underlying categorical structure.

---

## Pattern 2: Progressive enhancement with DatastarRequest guard

Datastar enables progressive enhancement: applications work without JavaScript while providing enhanced interactivity when JavaScript is available.
The DatastarRequest guard pattern detects whether a request originated from Datastar (via SSE update) or from a regular browser navigation, allowing handlers to return appropriate responses for each case.

### The pattern

axum's `Option<DatastarRequest>` extractor enables handlers to detect Datastar-initiated requests:

```rust
use axum::{extract::State, response::IntoResponse};
use datastar::axum::DatastarRequest;

async fn handler(
    State(state): State<AppState>,
    datastar_request: Option<DatastarRequest>,
    // other extractors...
) -> impl IntoResponse {
    if datastar_request.is_some() {
        // Datastar-initiated request: return fragment via SSE
        // Uses PatchElements to morph partial content
        serve_fragment(&state).await
    } else {
        // Initial page load or non-JS browser: return full HTML
        // Progressive enhancement: works without JavaScript
        serve_full_page(&state).await
    }
}
```

**Why this works:**

Datastar includes specific headers when making requests via `data-on:*` event handlers or `data-get` directives.
The `DatastarRequest` extractor detects these headers and extracts Some(DatastarRequest) when present, None otherwise.

### Canonical implementation

This pattern is demonstrated in the northstar Go template using the `ssr.IsDatastar(r)` helper:

**Source**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/internal/ssr/`

The Rust equivalent uses axum's type-safe extractors:

```rust
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response, Sse},
};
use datastar::prelude::*;
use hypertext::{html_elements, maud_move, Renderable};

async fn todo_page(
    State(state): State<AppState>,
    datastar_request: Option<DatastarRequest>,
) -> Response {
    if datastar_request.is_some() {
        // Return SSE stream with fragment for Datastar morph
        let stream = async_stream::stream! {
            let fragment = render_todo_list(&state).await;
            yield Ok(sse::Event::default().data(
                ServerSentEvent::new_patch_elements(fragment.render(), "#todo-list")
            ));
        };
        Sse::new(stream).into_response()
    } else {
        // Return full HTML page for initial load
        let page = html_elements::html((
            html_elements::head((
                html_elements::title("Todos"),
                html_elements::script().src("/static/datastar.js"),
            )),
            html_elements::body((
                html_elements::h1("Todo List"),
                html_elements::div()
                    .id("todo-list")
                    .child(render_todo_list(&state).await),
            )),
        ));
        Html(page.render()).into_response()
    }
}

async fn render_todo_list(state: &AppState) -> impl Renderable {
    let todos = state.projections.get_all_todos().await;
    maud_move! {
        ul {
            @for todo in todos {
                li { (todo.title) }
            }
        }
    }
}
```

### Key architectural benefits

1. **Progressive enhancement**: Application works without JavaScript (full page loads), enhanced with JavaScript (partial updates)
2. **Type-safe detection**: `Option<DatastarRequest>` eliminates conditional header parsing
3. **Single handler**: One endpoint serves both full pages and fragments
4. **SEO-friendly**: Initial page loads return full HTML for crawlers
5. **Reduced duplication**: Fragment rendering logic shared between both response modes

### When to use this pattern

Use the DatastarRequest guard when:

- Supporting browsers with JavaScript disabled
- Implementing SEO-critical pages (e.g., public content, marketing pages)
- Building forms that should work with and without JavaScript
- Creating dashboard pages where initial load returns full layout, updates return fragments

Avoid this pattern when:

- Building admin interfaces where JavaScript is guaranteed
- Implementing SPA-like experiences with no server-rendered fallback
- Creating API-only endpoints (use dedicated API routes instead)

### Integration with hypertext

The pattern works seamlessly with hypertext's lazy rendering:

```rust
fn full_page_layout(content: impl Renderable) -> impl Renderable {
    html_elements::html((
        html_elements::head((
            html_elements::title("App"),
            html_elements::script().src("/static/datastar.js"),
        )),
        html_elements::body(content),
    ))
}

async fn handler(
    State(state): State<AppState>,
    datastar_request: Option<DatastarRequest>,
) -> Response {
    let content = render_content(&state).await;

    if datastar_request.is_some() {
        // Return fragment via SSE
        let stream = async_stream::stream! {
            yield Ok(sse::Event::default().data(
                ServerSentEvent::new_patch_elements(content.render(), "#main")
            ));
        };
        Sse::new(stream).into_response()
    } else {
        // Wrap content in full page layout
        Html(full_page_layout(content).render()).into_response()
    }
}
```

**Note**: The content rendering function (`render_content`) is called once, and the result is reused for both response modes.
hypertext's lazy evaluation ensures no work is wasted.

### Related patterns

- **Northstar reference**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/internal/ssr/datastar.go`
- **datastar-rust SDK**: `~/projects/rust-workspace/datastar-rust/src/axum.rs` (DatastarRequest extractor implementation)
- **Canonical SDK spec**: `~/projects/lakescope-workspace/datastar/sdk/ADR.md` (Datastar request detection semantics)

---

## DatastarRequest extractor for progressive enhancement

The `datastar-rust` crate generates SSE payloads but provides no mechanism to detect whether a request came from a Datastar client.
This detection is essential for progressive enhancement: the same handler must respond differently to browser navigation (full HTML) versus Datastar signal updates (SSE fragments).

### Implementation

The DatastarRequest extractor checks for the `datastar-request: true` header that Datastar clients automatically inject:

```rust
use std::convert::Infallible;
use axum::async_trait;
use axum::extract::FromRequestParts;
use http::request::Parts;

#[derive(Debug, Clone, Copy)]
pub struct DatastarRequest(pub bool);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for DatastarRequest {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let is_datastar = parts
            .headers
            .get("datastar-request")
            .is_some_and(|value| value.as_bytes() == b"true");

        Ok(Self(is_datastar))
    }
}
```

**Design decisions:**
- Returns `Infallible` (never fails) — always succeeds with a boolean value
- Owned by application code, not the SDK — datastar-rust focuses on SSE generation, not request detection
- Zero runtime cost — single header lookup, no parsing

### Usage in handlers

Branch handler responses based on request origin:

```rust
async fn page_handler(
    State(state): State<AppState>,
    DatastarRequest(is_datastar): DatastarRequest,
) -> Response {
    if is_datastar {
        // Datastar client: return SSE stream with fragments
        sse_handler(State(state)).await.into_response()
    } else {
        // Browser navigation: return full HTML page
        let html = render_full_page(&state);
        Html(html).into_response()
    }
}
```

**Benefits:**
- **Backward compatible**: Application works without JavaScript (full-page responses are baseline)
- **Composable**: Integrates seamlessly with other axum extractors (State, Session, etc.)
- **Type-safe**: Compiler enforces two-path handling via pattern matching
- **Testable**: Write tests for both branches independently

---

## Summary: when to use which pattern

| Scenario | Pattern | Key Attributes |
|----------|---------|----------------|
| Simple library wrapper | Pattern 1: Thin web component | `data-ignore-morph`, `data-attr:*`, `data-on:*` |
| Complex lifecycle (multiple observers) | Pattern 1.5: Lit wrapper | Same, plus Lit `@property` and lifecycle hooks |
| Progressive enhancement | Pattern 2: DatastarRequest guard | `Option<DatastarRequest>` extractor, dual response modes |
| Drag-and-drop (SortableJS) | Pattern 1: Thin wrapper | Dispatch custom events on reorder |
| Rich text editors | Pattern 1 or 1.5 | Two-way sync via `data-bind` and custom events |
| Visualization libraries | See `integration-patterns-visualizations.md` | Vega-Lite, ECharts, Mosaic patterns |

The unifying principle: *Datastar owns state, the library owns DOM*.
Web components provide the encapsulation boundary, `data-ignore-morph` prevents morphing conflicts, and custom events enable communication back to the signal world.

For specific visualization library implementations (Vega-Lite, ECharts, Mosaic), styling with Open Props, and hypertext to SSE integration patterns, see `integration-patterns-visualizations.md`.

---

## Related documentation

### Ironstar architecture

- Visualization library implementations: `integration-patterns-visualizations.md`
- Complete ECharts implementation: `ds-echarts-integration-guide.md`
- Hypertext + Datastar syntax: see the hypertext section in `../core/architecture-decisions.md`
- Event sourcing core concepts: `../cqrs/event-sourcing-core.md`
- SSE connection lifecycle: `../cqrs/sse-connection-lifecycle.md`
- Signal type contracts: `signal-contracts.md`
- Frontend build pipeline: `frontend-build-pipeline.md` (includes Lit bundling options)

### External source code references

- **Datastar framework**: `~/projects/lakescope-workspace/datastar/`
- **Datastar documentation**: `~/projects/lakescope-workspace/datastar-doc/`
- **Datastar Go SDK**: `~/projects/lakescope-workspace/datastar-go/`
- **Datastar Rust SDK**: `~/projects/rust-workspace/datastar-rust/`
- **Northstar template** (Go + Datastar): `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/`
- **Lit framework**: `~/projects/lakescope-workspace/lit-web-components/`
- **ECharts**: `~/projects/lakescope-workspace/echarts/`
- **Vega-embed**: `~/projects/lakescope-workspace/vega-embed/`
- **Mosaic**: `~/projects/lakescope-workspace/mosaic/`
- **esbuild**: `~/projects/lakescope-workspace/esbuild/`
- **hypertext**: `~/projects/rust-workspace/hypertext/`
