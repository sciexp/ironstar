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

All Lit components in this pattern **must** override `createRenderRoot()` to use light DOM:

```typescript
protected createRenderRoot() {
  return this
}
```

This ensures Open Props CSS custom properties are accessible (shadow DOM blocks token inheritance).

### Build integration

Lit components can be bundled via either Rolldown or esbuild.
See `docs/notes/architecture/frontend-build-pipeline.md` for configuration options including the proven esbuild pattern from the Northstar template.

### Complete implementation reference

See `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/docs/notes/echarts/ds-echarts-integration-guide.md` for complete implementation details including:
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

## Summary: when to use which pattern

| Scenario | Pattern | Key Attributes |
|----------|---------|----------------|
| Simple library wrapper | Pattern 1: Thin web component | `data-ignore-morph`, `data-attr:*`, `data-on:*` |
| Complex lifecycle (multiple observers) | Pattern 1.5: Lit wrapper | Same, plus Lit `@property` and lifecycle hooks |
| Drag-and-drop (SortableJS) | Pattern 1: Thin wrapper | Dispatch custom events on reorder |
| Rich text editors | Pattern 1 or 1.5 | Two-way sync via `data-bind` and custom events |
| Visualization libraries | See `integration-patterns-visualizations.md` | Vega-Lite, ECharts, Mosaic patterns |

The unifying principle: *Datastar owns state, the library owns DOM*.
Web components provide the encapsulation boundary, `data-ignore-morph` prevents morphing conflicts, and custom events enable communication back to the signal world.

For specific visualization library implementations (Vega-Lite, ECharts, Mosaic), styling with Open Props, and hypertext to SSE integration patterns, see `integration-patterns-visualizations.md`.

---

## Related documentation

### Ironstar architecture

- Visualization library implementations: `docs/notes/architecture/integration-patterns-visualizations.md`
- Complete ECharts implementation: `docs/notes/architecture/ds-echarts-integration-guide.md`
- Hypertext + Datastar syntax: see the hypertext section in `architecture-decisions.md`
- Event sourcing core concepts: `docs/notes/architecture/event-sourcing-core.md`
- SSE connection lifecycle: `docs/notes/architecture/sse-connection-lifecycle.md`
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
