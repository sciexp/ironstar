# Integration patterns for third-party libraries with Datastar

Datastar's core principle is *server-driven UI*: the server sends HTML fragments and signal updates via SSE, and the browser reflects this state.
This creates a challenge when integrating third-party libraries that manage their own DOM.
Vega-Lite charts, drag-and-drop libraries, rich text editors, and similar components expect to control their own rendering.

This document establishes patterns for preserving Datastar's server-driven philosophy while enabling integration with these DOM-owning libraries.

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

## Pattern 1.5: When Lit is appropriate for wrapper components

Ironstar's general preference is vanilla web components to avoid redundant reactivity layers, but certain integration scenarios benefit architecturally from Lit's lifecycle management and reactive property system.
The key criterion is whether Lit provides structural value beyond convenience.

### Rationale for Lit over vanilla custom elements

Lit becomes appropriate when a web component requires:

1. **Complex lifecycle coordination**: Multiple observers (ResizeObserver, MediaQueryList, IntersectionObserver) with coordinated setup and teardown sequences that must be orchestrated carefully

2. **Imperative library integration**: Third-party libraries with init/dispose patterns (ECharts, D3 force simulations, Three.js scenes) where the library expects ownership of a DOM subtree and has its own internal state machine

3. **Declarative property mapping**: When the component primarily serves as a bridge between Datastar's `data-attr:*` binding pattern and an imperative API, Lit's `@property` decorator provides cleaner attribute-to-API translation than manual `observedAttributes` arrays

### Decision criteria

Use Lit when **all** of the following apply:

- Component wraps an imperative library that manages its own internal state
- Component requires multiple lifecycle observers or listeners that must be cleaned up precisely
- Component properties map directly to library configuration (no business logic)
- Lit's reactivity is isolated to component internals; **Datastar still owns application state**
- Light DOM rendering is acceptable (required for Open Props token access)

If any of these are false, prefer vanilla web components (Pattern 1).

### Canonical example: ds-echarts

The `ds-echarts` component demonstrates when Lit's architectural value justifies the dependency.

**Source**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/libs/lit/src/components/ds-echarts/ds-echarts.ts`

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

- ECharts requires careful init (`echarts.init`) and dispose (`chart.dispose()`) lifecycle
- ResizeObserver must be set up after chart initialization and torn down precisely
- MediaQueryList listener for dark mode requires removal on disconnect to prevent memory leaks
- Lit's `updated(changedProperties)` provides clean property diffing for selective updates

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

All Lit components in this pattern **must** override `createRenderRoot()` to use light DOM:

```typescript
protected createRenderRoot() {
  return this
}
```

This ensures Open Props CSS custom properties are accessible (shadow DOM blocks token inheritance) and allows TailwindCSS utility classes to work correctly if used.

### Build integration

Lit components can be bundled via either Rolldown or esbuild.
See `docs/notes/architecture/frontend-build-pipeline.md` for configuration options including the proven esbuild pattern from the Northstar template.

### Source code references

- **Lit framework**: `~/projects/lakescope-workspace/lit-web-components/` (core library source)
- **Northstar examples**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/libs/lit/src/components/`
- **Datastar integration**: `~/projects/lakescope-workspace/datastar/` (SSE protocol, `data-attr:*` binding semantics)
- **esbuild bundler**: `~/projects/lakescope-workspace/esbuild/` (Go-based bundler used in Northstar)

---

## Pattern 2: Vega-Lite chart integration

Vega-Lite charts via vega-embed exemplify the DOM ownership challenge.
The Vega View manages its own SVG/Canvas rendering, maintains internal state for selections and tooltips, and requires explicit cleanup.

### Why special handling is required

When you call `embed(container, spec)`, vega-embed:

1. Clears the container's innerHTML
2. Creates SVG or Canvas elements
3. Attaches event listeners
4. Maintains a View object with internal state

If Datastar morphs this container, it destroys these elements.
The View object becomes disconnected from the DOM, causing memory leaks and broken interactivity.

### The View class API

From `/Users/crs58/projects/lakescope-workspace/vega-embed/src/embed.ts`, the vega-embed `Result` interface exposes:

```typescript
export interface Result {
  view: View;              // The Vega view for reactive updates
  spec: VisualizationSpec; // Input specification
  vgSpec: VgSpec;          // Compiled Vega specification
  embedOptions: EmbedOptions;
  finalize: () => void;    // Critical: cleanup function
}
```

The View class provides methods for reactive updates without re-embedding:

- `view.data(name, values)` — replace a named data source
- `view.signal(name, value)` — update a signal value
- `view.run()` — re-render after updates
- `view.finalize()` — cleanup listeners and prevent memory leaks

### Web component implementation

```typescript
import embed, { Result } from 'vega-embed';
import { View } from 'vega';

class VegaChart extends HTMLElement {
  private result: Result | null = null;
  private view: View | null = null;

  static observedAttributes = ['spec-url', 'data-url', 'signal-values'];

  async connectedCallback() {
    await this.render();
  }

  disconnectedCallback() {
    // Critical: prevent memory leaks from Vega's internal listeners
    this.result?.finalize();
    this.result = null;
    this.view = null;
  }

  async attributeChangedCallback(name: string, oldValue: string, newValue: string) {
    if (!this.view || oldValue === newValue) return;

    switch (name) {
      case 'data-url':
        // Update data without re-embedding
        const data = await fetch(newValue).then(r => r.json());
        this.view.data('source', data).run();
        break;

      case 'signal-values':
        // Update Vega signals from Datastar signals
        const signals = JSON.parse(newValue);
        Object.entries(signals).forEach(([k, v]) => {
          this.view!.signal(k, v);
        });
        this.view.run();
        break;

      case 'spec-url':
        // Spec change requires full re-render
        await this.render();
        break;
    }
  }

  private async render() {
    const specUrl = this.getAttribute('spec-url');
    if (!specUrl) return;

    // Clean up previous instance
    this.result?.finalize();

    // Fetch and render
    const spec = await fetch(specUrl).then(r => r.json());
    this.result = await embed(this, spec, {
      renderer: 'svg',
      actions: false
    });
    this.view = this.result.view;

    // Bridge Vega selections to Datastar custom events
    this.view.addSignalListener('select', (name, value) => {
      this.dispatchEvent(new CustomEvent('vega-select', {
        detail: { name, value },
        bubbles: true
      }));
    });
  }
}

customElements.define('vega-chart', VegaChart);
```

### Datastar usage

```html
<vega-chart
  data-ignore-morph
  data-attr:spec-url="$chartSpec"
  data-attr:data-url="$dataEndpoint"
  data-attr:signal-values="JSON.stringify({highlight: $highlightedItem})"
  data-on:vega-select="$selection = evt.detail.value"
></vega-chart>
```

### Update strategies

The component supports three update patterns with different performance characteristics:

1. *Data updates* (`data-url` change): fetches new data and calls `view.data().run()`. Fast because the visualization structure is preserved.

2. *Signal updates* (`signal-values` change): updates Vega signals directly and calls `view.run()`. Very fast because no data processing occurs.

3. *Spec updates* (`spec-url` change): requires `finalize()` and full re-embed. Slower but necessary when the visualization structure changes.

For dashboards with coordinated views, prefer signal updates for cross-filtering interactions.
Reserve spec updates for user-initiated changes like switching chart types.

---

## Pattern 3: Apache ECharts integration via Lit component

ECharts is a powerful, declarative charting library that manages its own canvas/SVG rendering lifecycle.
Like Vega-Lite, it requires protection from Datastar's morphing to preserve its internal DOM structure and event listeners.
Unlike Vega-Lite, ECharts benefits from a Lit wrapper due to complex lifecycle requirements (see Pattern 1.5).

### Why ECharts requires special handling

When you call `echarts.init(container, theme)`, ECharts:

1. Takes ownership of the container element
2. Creates either canvas or SVG rendering context based on configuration
3. Attaches event listeners for interactions (hover, click, zoom, etc.)
4. Maintains internal state for animations, data series, and visual encodings

If Datastar morphs this container, the ECharts instance becomes disconnected from the DOM.
The chart stops responding to interactions, resize events fail, and the instance leaks memory.

### ECharts instance API

ECharts uses an imperative instance-based API.
From `~/projects/lakescope-workspace/echarts/src/core/echarts.ts`, the core interface provides:

```typescript
interface ECharts {
  setOption(option: EChartsOption, opts?: SetOptionOpts): void;  // Update or replace chart configuration
  resize(opts?: ResizeOpts): void;                                // Manually trigger resize
  dispose(): void;                                                 // Critical: cleanup and remove all listeners
  getWidth(): number;
  getHeight(): number;
  on(eventName: string, handler: Function): void;                  // Event subscriptions
  off(eventName: string, handler?: Function): void;
}
```

Key differences from Vega-Lite View API:

- **Initialization**: `echarts.init(container, theme)` is a static factory method that returns an instance
- **Updates**: `setOption()` merges by default (incremental updates), while Vega's `view.data()` replaces data sources entirely
- **Resize**: ECharts requires explicit `resize()` calls
- **Themes**: ECharts theme is set at initialization time only; changing themes requires dispose → reinit → setOption cycle

### Update strategies with performance characteristics

1. **Option updates** (`setOption()`): Fast, incremental updates. ECharts merges new option properties with existing configuration by default.
   ```typescript
   chart.setOption({ series: [{ data: newData }] })  // Merges into existing series
   ```
   This is the primary update mechanism for real-time dashboards.

2. **Theme changes**: Requires full disposal and reinitialization. Slow because ECharts theme is baked into the rendering pipeline at creation time.
   ```typescript
   chart.dispose()
   chart = echarts.init(container, 'dark')
   chart.setOption(previousOption)
   ```
   Reserve this for explicit user theme switches.

3. **Resize**: Very fast because it only recalculates layout without reprocessing data.
   ```typescript
   chart.resize()  // Call after container dimension changes
   ```
   Use ResizeObserver with debouncing to batch resize calls.

### The ds-echarts Lit component

The `ds-echarts` component wraps the ECharts lifecycle in a Lit web component, following the thin wrapper pattern with Lit's lifecycle management benefits.

**Full source**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/libs/lit/src/components/ds-echarts/ds-echarts.ts`

**Design document**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/docs/notes/echarts/lit-echarts-component-plan.md`

**Property interface**:

```typescript
@customElement('ds-echarts')
export class DsEcharts extends LitElement {
  @property({ type: String }) option = '{}'           // JSON string (Datastar stringifies objects)
  @property({ type: String }) theme = 'default'       // 'default' or registered theme name
  @property({ type: Number, attribute: 'resize-delay' }) resizeDelay = 100  // Debounce ms

  private chart: ECharts | null = null
  private chartContainer: HTMLDivElement | null = null
  private resizeObserver: ResizeObserver | null = null
  private currentTheme: string = 'default'
  private mediaQueryHandler: ((e: MediaQueryListEvent) => void) | null = null
}
```

**Key implementation details**:

- `option` is received as a JSON string because Datastar's `data-attr:*` binding sets HTML attributes. The component parses it in `updateOption()`.
- `createRenderRoot() { return this }` enables Open Props token access via light DOM rendering.
- Theme changes trigger dispose → init → setOption cycle.
- ResizeObserver with configurable debounce handles container resize.
- MediaQueryList listener reinitializes chart on dark mode toggle (when `theme="default"`).

**Lifecycle flow**:

```
firstUpdated() → initChart() → setupResizeObserver() → setupMediaQueryListener()
       ↓
updated(changedProperties) → updateOption() or reinit for theme change
       ↓
disconnectedCallback() → dispose chart, disconnect observer, remove listeners
```

**Error handling**: Invalid JSON logs `[ds-echarts]` prefix error to console but does not throw, preserving the rest of the page if chart configuration is temporarily malformed.

### Datastar integration example

```html
<div
  data-signals='{
    chartOption: {
      "title": {"text": "Sales by Region"},
      "tooltip": {"trigger": "axis"},
      "xAxis": {"type": "category", "data": ["Q1", "Q2", "Q3", "Q4"]},
      "yAxis": {"type": "value"},
      "series": [{"name": "Revenue", "type": "bar", "data": [150, 230, 224, 218]}]
    },
    selectedTheme: "default"
  }'
>
  <ds-echarts
    data-ignore-morph
    data-attr:option="JSON.stringify($chartOption)"
    data-attr:theme="$selectedTheme"
    style="height: 400px; width: 100%;"
  ></ds-echarts>
</div>
```

**Critical attributes**:

- `data-ignore-morph` — Prevents Datastar from morphing the component's children
- `data-attr:option="JSON.stringify($chartOption)"` — Converts signal object to JSON string
- `style="height: 400px; width: 100%;"` — Explicit dimensions required; ECharts cannot infer height from content

### SSE server-side pattern with hypertext templates

In Ironstar, hypertext templates render the component from Rust handlers:

```rust
use hypertext::{html_elements, maud_move, Renderable};
use serde_json::json;

fn echarts_dashboard(chart_data: &ChartData) -> impl Renderable {
    let chart_option = json!({
        "title": {"text": &chart_data.title},
        "xAxis": {"type": "category", "data": &chart_data.x_labels},
        "yAxis": {"type": "value"},
        "series": [{"data": &chart_data.values, "type": "bar"}]
    });

    maud_move! {
        div "data-signals"=(format!(r#"{{"chartOption": {}}}"#, chart_option)) {
            ds-echarts
                "data-ignore-morph"=""
                "data-attr:option"="JSON.stringify($chartOption)"
                style="height: 400px; width: 100%;"
            {}
        }
    }
}
```

**SSE streaming for real-time updates**:

```rust
async fn stream_chart_updates(
    State(analytics): State<AnalyticsService>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = interval(Duration::from_secs(2))
        .then(move |_| async {
            let new_data = analytics.fetch_latest().await;
            let patch = json!({"chartOption": {"series": [{"data": new_data.values}]}});
            Ok::<_, Infallible>(PatchSignals::new(patch.to_string()).into())
        });

    Sse::new(stream)
}
```

The server patches only the changed signal properties.
ECharts' `setOption()` merges incremental updates efficiently.

### Source code references

- **ECharts source**: `~/projects/lakescope-workspace/echarts/`
- **ds-echarts component**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/libs/lit/src/components/ds-echarts/ds-echarts.ts`
- **Component design doc**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/docs/notes/echarts/lit-echarts-component-plan.md`
- **Northstar Lit build**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/libs/lit/`
- **Datastar-rust SDK**: `~/projects/rust-workspace/datastar-rust/`

---

## Considerations for Mosaic integration (TBD)

Mosaic (`/Users/crs58/projects/lakescope-workspace/mosaic`) provides a higher-level grammar of graphics built on DuckDB and Observable Plot.
It manages coordinated views through params and selections, with query optimization for large datasets.

Mosaic is architecturally different from Vega-Lite in ways that affect Datastar integration:

From `/Users/crs58/projects/lakescope-workspace/mosaic/docs/what-is-mosaic/index.md`:

> A key idea is that interface components — Mosaic *clients* — publish their data needs as queries that are managed by a central *coordinator*. The coordinator may further optimize queries before issuing them to a backing *data source* such as DuckDB.

Mosaic clients already have their own lifecycle and state management.
The `makeClient` API documented in `/Users/crs58/projects/lakescope-workspace/mosaic/docs/web-clients/index.md` shows how Mosaic expects to manage component updates:

```typescript
let client = makeClient({
  coordinator,
  selection,
  query: (predicate) => Query.from(table).select(...).where(predicate),
  queryResult: (data) => { /* update visualization */ },
});
return () => { client.destroy(); };
```

A Datastar integration would need to bridge two reactive systems:

1. Mosaic's coordinator (push-based query updates)
2. Datastar's signals (SSE-driven state updates)

Potential approach: create a Mosaic client that synchronizes its selections with Datastar signals, allowing server-driven filter updates to propagate through the Mosaic coordinator.
This is more complex than the Vega-Lite pattern because both systems want to own reactivity.

Full Mosaic integration patterns require further research.
For now, the Vega-Lite pattern covers most visualization needs in Ironstar.

---

## Summary: when to use which pattern

| Scenario | Pattern | Key Attributes |
|----------|---------|----------------|
| Simple library wrapper | Pattern 1: Thin web component | `data-ignore-morph`, `data-attr:*`, `data-on:*` |
| Complex lifecycle (multiple observers) | Pattern 1.5: Lit wrapper | Same, plus Lit `@property` and lifecycle hooks |
| Vega-Lite charts | Pattern 2: VegaChart component | Same, plus `finalize()` cleanup |
| ECharts charts | Pattern 3: ds-echarts Lit component | Lit wrapper with ResizeObserver, theme handling |
| Drag-and-drop (SortableJS) | Pattern 1: Thin wrapper | Dispatch custom events on reorder |
| Rich text editors | Pattern 1 or 1.5 | Two-way sync via `data-bind` and custom events |
| Mosaic visualizations | TBD | Requires bridging two reactive systems |

The unifying principle: *Datastar owns state, the library owns DOM*.
Web components provide the encapsulation boundary, `data-ignore-morph` prevents morphing conflicts, and custom events enable communication back to the signal world.

---

## Styling web components with Open Props

Ironstar uses Open Props for design tokens and Open Props UI for component styling, following an ownership model where component CSS is copied into the project rather than imported as a dependency.

### CSS framework architecture

The styling system has three layers:

1. **Open Props** (`~/projects/lakescope-workspace/open-props`): provides design tokens as CSS custom properties (size scales, color palettes, easing functions, shadows, etc.)

2. **Open Props UI** (`~/projects/lakescope-workspace/open-props-ui`): provides semantic component classes (`.button`, `.card`, etc.) which are copied into the project's `web/resources/static/css/` directory

3. **Theme layer** (`theme.css`): derives app-specific tokens from Open Props primitives, enabling theming via CSS custom properties without JavaScript

### Dark mode support

Open Props includes the `light-dark()` CSS function for automatic dark mode support without JavaScript:

```css
.component {
  background: light-dark(var(--gray-1), var(--gray-9));
  color: light-dark(var(--gray-9), var(--gray-1));
}
```

The browser automatically switches between light and dark values based on the user's `prefers-color-scheme` preference.

### Styling patterns for web components

When creating web components that need styling, use Open Props tokens and semantic classes:

```typescript
class StyledComponent extends HTMLElement {
  connectedCallback() {
    // Container with Open Props UI semantic class
    const container = document.createElement('div');
    container.className = 'card';

    // Custom sizing with Open Props tokens
    container.style.width = 'var(--size-content-3)';
    container.style.padding = 'var(--size-4)';

    // Button with semantic class
    const button = document.createElement('button');
    button.className = 'btn btn-primary';
    button.textContent = 'Submit';

    container.appendChild(button);
    this.appendChild(container);
  }
}
```

### Integration with hypertext templates

When web components are embedded in hypertext templates, apply Open Props classes and inline styles:

```rust
use hypertext::{html_elements, maud_move, Renderable};

fn component_container() -> impl Renderable {
    maud_move! {
        // Semantic class from Open Props UI
        div class="card" style="width: var(--size-content-3);" {
            button class="btn btn-primary" { "Submit" }
        }
    }
}
```

For components that need custom styling beyond Open Props UI's semantic classes, use inline styles with Open Props tokens rather than utility classes.
This approach maintains the design system while preserving component ownership.

---

## Hypertext to SSE integration

When integrating hypertext templates with Datastar SSE, use the `RenderableToDatastar` helper trait defined in the hypertext section of `architecture-decisions.md`:

```rust
use hypertext::Renderable;
use datastar::prelude::*;

pub trait RenderableToDatastar: Renderable {
    fn to_patch_elements(&self) -> PatchElements {
        PatchElements::new(self.render().into_inner())
    }

    fn append_to(&self, selector: &str) -> PatchElements {
        PatchElements::new(self.render().into_inner())
            .selector(selector)
            .mode(ElementPatchMode::Append)
    }

    fn replace_inner(&self, selector: &str) -> PatchElements {
        PatchElements::new(self.render().into_inner())
            .selector(selector)
            .mode(ElementPatchMode::Inner)
    }
}

impl<T: Renderable> RenderableToDatastar for T {}
```

This enables ergonomic conversion from hypertext components to Datastar SSE events:

```rust
async fn get_todo_list(State(store): State<TodoStore>) -> impl IntoResponse {
    let todos = store.list().await;
    let html = todo_list_component(&todos);

    // Single SSE event with the rendered HTML
    Sse::new(stream::once(async move {
        Ok::<_, Infallible>(html.to_patch_elements().into())
    }))
}
```

For complete SSE streaming patterns including event replay and projection updates, see `docs/notes/architecture/event-sourcing-sse-pipeline.md`.

---

## Related documentation

### Ironstar architecture

- Hypertext + Datastar syntax: see the hypertext section in `architecture-decisions.md`
- Event sourcing and SSE: `docs/notes/architecture/event-sourcing-sse-pipeline.md`
- Signal type contracts: `docs/notes/architecture/signal-contracts.md`
- Frontend build pipeline: `docs/notes/architecture/frontend-build-pipeline.md` (includes Lit bundling options)

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
