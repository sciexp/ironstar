# ds-echarts Integration Guide for Ironstar

**Pattern reference**: This guide implements Pattern 1.5 from `integration-patterns.md` — Lit components for TypeScript library integration.

> **Semantic foundation**: ds-echarts is a Moore machine coalgebra.
> The Lit `updated()` lifecycle implements the transition function; `render()` produces output.
> Bisimulation equivalence justifies `data-ignore-morph` safety.
> See [semantic-model.md § Web components as coalgebras](../core/semantic-model.md#web-components-as-coalgebras-moore-machines).

This guide covers the integration of the ds-echarts Lit component from the northstar Go template into ironstar's Rust + hypertext + Datastar architecture.

## Overview

ds-echarts is a Lit-based web component that bridges Apache ECharts with Datastar's hypermedia-driven architecture.
It transforms ECharts from an imperative library into a declarative, signal-driven component where the server remains the source of truth.

The component embodies CQRS principles: reads (chart rendering) project server state via the `option` attribute, while writes (user interactions) emit events that signal the server via POST requests.

## Migration Checklist

### Phase 1: Component setup

1. Create `web-components/` directory structure
2. Copy `ds-echarts.ts` from northstar with minimal adaptations
3. Create `package.json` with required dependencies
4. Configure `tsconfig.json` for Lit decorators
5. Set up `rolldown.config.ts` for bundling

### Phase 2: Build pipeline

1. Configure PostCSS for Open Props imports
2. Create CSS entry point with Open Props + theme layer
3. Generate `manifest.json` for hashed asset URLs
4. Integrate rust-embed for production asset serving

See `ds-echarts-build-test.md` and `frontend-build-pipeline.md` for complete build configuration details.

### Phase 3: Rust integration

1. Define signal types with ts-rs for TypeScript generation
2. Create hypertext templates for ds-echarts usage
3. Implement axum SSE handlers for chart data
4. Set up Zenoh subscription for real-time updates

See `ds-echarts-backend.md` for complete backend integration patterns.

### Phase 4: Testing

1. Configure Vitest with Happy-DOM
2. Set up ECharts mocking strategy
3. Port test patterns from northstar

See `ds-echarts-build-test.md` for testing configuration and data flow diagrams.

## Directory Structure

```
web-components/
├── package.json
├── pnpm-lock.yaml
├── rolldown.config.ts
├── postcss.config.js
├── tsconfig.json
├── index.ts                     # CSS + component entry
│
├── styles/
│   ├── main.css                 # Open Props imports + theme + components
│   ├── theme.css                # App-specific token overrides
│   └── components/              # Copied Open Props UI CSS (owned)
│       ├── button.css
│       ├── card.css
│       └── ...
│
├── components/
│   ├── index.ts                 # Re-export all components
│   └── ds-echarts/
│       ├── ds-echarts.ts        # Lit component (from northstar)
│       └── index.ts             # Re-export
│
├── __tests__/
│   ├── setup.ts                 # ECharts mock
│   ├── test-utils.ts            # Render helpers
│   └── ds-echarts.test.ts       # Component tests
│
└── types/                       # Generated from ts-rs
    ├── ChartSignals.ts
    └── ChartEventDetail.ts
```

## Component Source

The ds-echarts component requires no significant changes from northstar.
The only CSS-related change is cosmetic since the component uses Light DOM and delegates styling to the host element.

### Key Properties (from northstar)

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `option` | string (JSON) | `'{}'` | ECharts configuration as JSON |
| `theme` | string | `'default'` | Theme name or 'default' for auto |
| `resizeDelay` | number | `100` | Debounce ms for resize events |
| `events` | string | `'lifecycle,mouse'` | Event categories to enable |
| `hoverThrottle` | number | `100` | Throttle ms for hover events |
| `renderer` | 'svg' \| 'canvas' | `'svg'` | Rendering engine |

### Custom Events Emitted

| Event | Category | Payload |
|-------|----------|---------|
| `chart-ready` | lifecycle | `{ width, height, theme }` |
| `chart-updated` | lifecycle | `{ timestamp }` |
| `chart-resized` | lifecycle | `{ width, height }` |
| `chart-disposed` | lifecycle | `{}` |
| `chart-error` | lifecycle | `{ message, error }` |
| `chart-click` | mouse | Sanitized ECharts event params |
| `chart-dblclick` | mouse | Sanitized ECharts event params |
| `chart-contextmenu` | mouse | Sanitized ECharts event params |
| `chart-hover-start` | hover | Sanitized event params (opt-in) |
| `chart-hover-end` | hover | Sanitized event params (opt-in) |
| `chart-legend-change` | component | Legend selection state (opt-in) |
| `chart-datazoom` | component | Zoom range (opt-in) |

### Light DOM requirement

The component must use Light DOM for Open Props CSS variable inheritance via `createRenderRoot() { return this }`.
See `css-architecture.md` for detailed rationale on why Shadow DOM blocks CSS custom property inheritance.

## Tailwind to Open Props Mapping

The ds-echarts component uses minimal styling (mainly via inline styles for dimensions).
For any Tailwind classes in surrounding templates:

| Tailwind Class | Open Props Equivalent |
|----------------|----------------------|
| `w-full` | `width: 100%` |
| `h-full` | `height: 100%` |
| `block` | `display: block` |
| `p-3` | `padding: var(--size-3)` |
| `mt-2` | `margin-block-start: var(--size-2)` |
| `gap-4` | `gap: var(--size-4)` |
| `rounded-lg` | `border-radius: var(--radius-2)` |
| `bg-base-200` | `background: var(--surface-tonal)` |
| `text-sm` | `font-size: var(--font-size-sm)` |
| `font-mono` | `font-family: var(--font-mono)` |
| `text-green-500` | `color: var(--green-7)` |

For chart containers specifically:

```css
/* Tailwind: style="height: 400px; width: 100%;" */
/* Open Props: same, or use custom property */
ds-echarts {
  display: block;
  height: var(--chart-height, 400px);
  width: 100%;
}
```

## Hypertext Template Patterns

### Signal Type Definitions (Rust)

```rust
// src/domain/signals.rs
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Chart configuration signals for ds-echarts component
#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../web-components/types/")]
pub struct ChartSignals {
    /// ECharts option object (serialized to JSON for attribute)
    #[serde(rename = "chartOption")]
    pub chart_option: serde_json::Value,

    /// Selected data point (from chart-click events)
    #[ts(optional)]
    pub selected: Option<ChartSelection>,

    /// Loading state for UI feedback
    #[serde(default)]
    pub loading: bool,
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChartSelection {
    #[serde(rename = "seriesName")]
    pub series_name: String,
    #[serde(rename = "dataIndex")]
    pub data_index: i32,
    pub name: String,
    pub value: serde_json::Value,
}

impl Default for ChartSignals {
    fn default() -> Self {
        Self {
            chart_option: serde_json::json!({}),
            selected: None,
            loading: false,
        }
    }
}
```

### Chart Template Component

```rust
// src/presentation/templates/charts.rs
use hypertext::{maud, Raw, Renderable};
use crate::domain::signals::ChartSignals;

/// Render a ds-echarts component with Datastar signal bindings
pub fn echarts_chart(
    id: &str,
    signals: &ChartSignals,
    height: &str,
) -> impl Renderable {
    let signals_json = serde_json::to_string(signals)
        .unwrap_or_else(|_| "{}".to_string());

    maud! {
        div
            id=(id)
            style="width: 100%;"
            "data-signals"=(Raw::dangerously_create(&signals_json))
        {
            ds-echarts
                "data-ignore-morph"="true"
                "data-attr:option"="JSON.stringify($chartOption)"
                "data-on:chart-ready"="console.log('Chart ready:', evt.detail)"
                "data-on:chart-click"="$selected = evt.detail; @post('/api/chart/select', {body: JSON.stringify(evt.detail)})"
                "data-on:chart-error"="console.error('Chart error:', evt.detail)"
                style=(format!("height: {}; width: 100%; display: block;", height))
            {}
        }
    }
}

/// Chart with loading indicator and selection display
pub fn chart_with_feedback(
    id: &str,
    signals: &ChartSignals,
) -> impl Renderable {
    let signals_json = serde_json::to_string(signals)
        .unwrap_or_else(|_| "{}".to_string());

    maud! {
        div
            id=(id)
            class="card elevated"
            style="padding: var(--size-3);"
            "data-signals"=(Raw::dangerously_create(&signals_json))
        {
            // Loading indicator
            div
                class="spinner"
                "data-show"="$loading"
                style="position: absolute; inset: 0; display: grid; place-items: center; background: var(--surface-default); opacity: 0.8;"
            {}

            // Chart element
            ds-echarts
                "data-ignore-morph"="true"
                "data-attr:option"="JSON.stringify($chartOption)"
                "data-on:chart-click"="$selected = evt.detail"
                events="lifecycle,mouse"
                style="height: var(--chart-height, 400px); width: 100%; display: block;"
            {}

            // Selection feedback
            div
                class="rich-text"
                style="margin-block-start: var(--size-2); padding: var(--size-2); background: var(--surface-tonal); border-radius: var(--radius-1);"
                "data-show"="$selected"
            {
                p {
                    "Selected: "
                    strong "data-text"="$selected ? $selected.name + ': ' + $selected.value : ''" {}
                }
            }
        }
    }
}

/// Page layout with chart SSE initialization
pub fn chart_page(title: &str, chart_id: &str) -> impl Renderable {
    maud! {
        main
            style="max-width: var(--size-content-3); margin-inline: auto; padding: var(--size-4);"
            "data-on-load"=(format!("@get('/api/charts/{}/data')", chart_id))
        {
            h1 style="margin-block-end: var(--size-4);" { (title) }

            div id=(format!("{}-container", chart_id)) {
                // SSE will replace this with actual chart
                p { "Loading chart..." }
            }
        }
    }
}
```

## Critical Implementation Notes

### ECharts Lifecycle

1. Theme changes require full dispose and re-init (slow)
2. Option changes use `setOption()` with merge (fast)
3. Always call `chart.dispose()` in `disconnectedCallback()`
4. ResizeObserver must be disconnected on cleanup

### Datastar Integration

1. Use `data-ignore-morph` on ds-echarts to protect ECharts DOM
2. Signals flow via `data-attr:option="JSON.stringify($chartOption)"`
3. Events capture via `data-on:chart-click="$selected = evt.detail"`
4. Server actions via `@post('/path', {body: JSON.stringify(...)})`

### SSE Reliability

1. Use event IDs from global sequence for reconnection replay
2. Subscribe to broadcast before replaying missed events
3. Initial load sends complete state (fat morph philosophy)
4. Incremental updates via PatchSignals (efficient)

### Performance Considerations

1. DuckDB queries run via async-duckdb Pool with dedicated background threads
2. Analytics results cached in moka with TTL
3. Cache invalidation via Zenoh subscription
4. rkyv zero-copy deserialization for cached data

## Related Documentation

- **Backend integration**: `ds-echarts-backend.md` — Axum handlers, DuckDB service, Zenoh event bus
- **Build and testing**: `ds-echarts-build-test.md` — TypeScript/Rolldown config, Vitest setup, data flow diagrams
- **Frontend build pipeline**: `frontend-build-pipeline.md` — Common Rolldown and PostCSS configuration patterns
- **Integration patterns**: `integration-patterns.md` — Web component patterns (vanilla/Lit), Vega-Lite, Mosaic
- **Signal contracts**: `signal-contracts.md` — TypeScript type generation with ts-rs
