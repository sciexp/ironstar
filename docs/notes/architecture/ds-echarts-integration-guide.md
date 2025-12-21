# ds-echarts Integration Guide for Ironstar

**Pattern reference**: This guide implements Pattern 1.5 from `integration-patterns.md` — Lit components for TypeScript library integration.

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

### Light DOM Requirement

The component must use Light DOM for Open Props CSS variable inheritance:

```typescript
protected createRenderRoot() {
  return this  // Light DOM, not Shadow DOM
}
```

Shadow DOM would block CSS custom property inheritance from the page, breaking theme support.

## Package Configuration

### package.json

```json
{
  "name": "ironstar-web-components",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "rolldown --config --watch",
    "build": "rolldown --config",
    "test": "vitest",
    "test:coverage": "vitest --coverage",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "echarts": "^5.5.0",
    "lit": "^3.3.1"
  },
  "devDependencies": {
    "@types/node": "^22.0.0",
    "@vitest/coverage-v8": "^2.1.8",
    "autoprefixer": "^10.4.0",
    "cssnano": "^7.0.0",
    "happy-dom": "^15.11.0",
    "postcss": "^8.4.0",
    "postcss-import": "^16.0.0",
    "postcss-preset-env": "^10.0.0",
    "rolldown": "^1.0.0",
    "typescript": "^5.9.0",
    "vitest": "^2.1.8"
  }
}
```

### tsconfig.json

```json
{
  "compilerOptions": {
    "experimentalDecorators": true,
    "useDefineForClassFields": false,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "target": "ES2020",
    "strict": true,
    "skipLibCheck": true,
    "declaration": false,
    "outDir": "dist"
  },
  "include": ["**/*.ts"],
  "exclude": ["node_modules", "dist", "__tests__"]
}
```

## Rolldown Configuration

### rolldown.config.ts

```typescript
import { defineConfig } from 'rolldown';
import postcss from 'rolldown-plugin-postcss';

export default defineConfig({
  input: {
    bundle: 'index.ts',
  },
  output: {
    dir: '../static/dist',
    format: 'esm',
    entryFileNames: '[name].[hash].js',
    chunkFileNames: '[name].[hash].js',
    assetFileNames: '[name].[hash][extname]',
    hashCharacters: 'base36',
    sourcemap: process.env.NODE_ENV === 'production' ? 'hidden' : true,
  },
  plugins: [
    postcss({
      config: './postcss.config.js',
      extract: 'bundle.css',
      minimize: process.env.NODE_ENV === 'production',
    }),
    manifestPlugin(),
  ],
  treeshake: {
    moduleSideEffects: 'no-external',
  },
});

function manifestPlugin() {
  return {
    name: 'manifest-generator',
    generateBundle(options, bundle) {
      const manifest: Record<string, { file: string; css?: string[] }> = {};

      for (const [fileName, asset] of Object.entries(bundle)) {
        if (asset.type === 'chunk' && asset.isEntry) {
          const entry = asset.name || fileName;
          manifest[entry] = { file: fileName };
        }
      }

      // Link CSS to entry
      for (const [fileName, asset] of Object.entries(bundle)) {
        if (asset.type === 'asset' && fileName.endsWith('.css')) {
          const entryName = fileName.replace(/\.[a-z0-9]+\.css$/, '');
          if (manifest[entryName]) {
            manifest[entryName].css = [fileName];
          } else {
            manifest['styles'] = { file: fileName };
          }
        }
      }

      this.emitFile({
        type: 'asset',
        fileName: 'manifest.json',
        source: JSON.stringify(manifest, null, 2),
      });
    },
  };
}
```

### postcss.config.js

See `css-architecture.md` "PostCSS configuration" section for the complete configuration.
The key requirement for ds-echarts is `postcss-preset-env` with stage 0 for modern CSS features.

## CSS Entry Point

See `css-architecture.md` "CSS entry point structure" and "Theme layer" sections for the complete CSS organization.

For ds-echarts specifically, add these chart-specific tokens to your theme:

```css
/* In web-components/styles/theme.css */
:root {
  /* Chart-specific tokens */
  --chart-height: 400px;
  --chart-min-height: 200px;
}
```

And define component styles for ds-echarts containers:

```css
/* Chart container styling using Open Props tokens */
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

## Axum Handler Patterns

### Chart Data Handler (SSE)

```rust
// src/presentation/handlers/charts.rs
use axum::{
    extract::{Path, State},
    response::{sse::{Event, Sse}, IntoResponse},
};
use datastar::{PatchElements, PatchSignals, ReadSignals};
use futures::stream::{self, Stream};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio_stream::StreamExt;

use crate::{
    domain::signals::{ChartSignals, ChartSelection},
    infrastructure::{analytics::DuckDBService, event_bus::EventBus},
    presentation::templates::charts::echarts_chart,
};

pub struct AppState {
    pub analytics: Arc<DuckDBService>,
    pub event_bus: Arc<EventBus>,
}

/// GET /api/charts/:id/data - Initial chart load via SSE
pub async fn chart_data_sse(
    State(state): State<Arc<AppState>>,
    Path(chart_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        // 1. Query initial data from DuckDB
        let query_result = state.analytics
            .query_chart_data(&chart_id)
            .await;

        match query_result {
            Ok(data) => {
                // 2. Build chart option from data
                let chart_option = build_echarts_option(&data);
                let signals = ChartSignals {
                    chart_option,
                    selected: None,
                    loading: false,
                };

                // 3. Render chart HTML
                let html = echarts_chart(&chart_id, &signals, "400px")
                    .render()
                    .into_inner();

                // 4. Send PatchElements to replace container
                let patch = PatchElements::new(html)
                    .selector(&format!("#{}-container", chart_id));
                yield Ok(patch.write_as_axum_sse_event());

                // 5. Subscribe to real-time updates via Zenoh
                let mut subscription = state.event_bus
                    .subscribe(&format!("charts/{}/updates", chart_id))
                    .await;

                while let Some(update) = subscription.next().await {
                    let updated_option = build_echarts_option(&update);
                    let signals_json = serde_json::json!({
                        "chartOption": updated_option,
                        "loading": false,
                    }).to_string();

                    let patch = PatchSignals::new(signals_json);
                    yield Ok(patch.write_as_axum_sse_event());
                }
            }
            Err(e) => {
                // Send error state
                let error_html = format!(
                    r#"<div id="{}-container" class="alert error">Failed to load chart: {}</div>"#,
                    chart_id, e
                );
                let patch = PatchElements::new(error_html)
                    .selector(&format!("#{}-container", chart_id));
                yield Ok(patch.write_as_axum_sse_event());
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// POST /api/chart/select - Handle chart click interactions
pub async fn chart_select(
    State(state): State<Arc<AppState>>,
    ReadSignals(selection): ReadSignals<ChartSelection>,
) -> impl IntoResponse {
    let stream = async_stream::stream! {
        // 1. Signal loading state
        let loading_patch = PatchSignals::new(r#"{"loading": true}"#);
        yield Ok::<_, Infallible>(loading_patch.write_as_axum_sse_event());

        // 2. Query drill-down data based on selection
        let drill_down = state.analytics
            .query_drill_down(&selection.series_name, selection.data_index)
            .await;

        match drill_down {
            Ok(detail_data) => {
                // 3. Publish to Zenoh for other subscribers
                state.event_bus
                    .publish(&format!("charts/detail/{}", selection.name), &detail_data)
                    .await;

                // 4. Return updated signals
                let signals_json = serde_json::json!({
                    "detailData": detail_data,
                    "loading": false,
                }).to_string();

                let patch = PatchSignals::new(signals_json);
                yield Ok(patch.write_as_axum_sse_event());
            }
            Err(e) => {
                let error_patch = PatchSignals::new(
                    serde_json::json!({
                        "loading": false,
                        "error": e.to_string(),
                    }).to_string()
                );
                yield Ok(error_patch.write_as_axum_sse_event());
            }
        }
    };

    Sse::new(stream)
}

fn build_echarts_option(data: &AnalyticsData) -> serde_json::Value {
    serde_json::json!({
        "title": { "text": &data.title },
        "tooltip": { "trigger": "axis" },
        "xAxis": {
            "type": "category",
            "data": data.labels
        },
        "yAxis": { "type": "value" },
        "series": [{
            "data": data.values,
            "type": "bar",
            "emphasis": { "focus": "series" }
        }]
    })
}
```

### DuckDB Analytics Service

```rust
// src/infrastructure/analytics.rs
use duckdb::{Connection, Result as DuckResult};
use std::sync::Mutex;

pub struct DuckDBService {
    conn: Mutex<Connection>,
}

pub struct AnalyticsData {
    pub title: String,
    pub labels: Vec<String>,
    pub values: Vec<f64>,
}

impl DuckDBService {
    pub fn new() -> DuckResult<Self> {
        let conn = Connection::open_in_memory()?;

        // Install required extensions
        conn.execute_batch(
            r#"
            INSTALL httpfs; LOAD httpfs;
            INSTALL parquet; LOAD parquet;
            "#
        )?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub async fn query_chart_data(&self, chart_id: &str) -> Result<AnalyticsData, Error> {
        // Run on blocking thread pool since DuckDB is sync
        let chart_id = chart_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = self.conn.lock().unwrap();

            // Example: query from HuggingFace dataset
            let mut stmt = conn.prepare(
                r#"
                SELECT category, SUM(value) as total
                FROM read_parquet('hf://datasets/org/data.parquet')
                WHERE chart_id = ?
                GROUP BY category
                ORDER BY total DESC
                "#
            )?;

            let rows = stmt.query_map([&chart_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })?;

            let mut labels = Vec::new();
            let mut values = Vec::new();
            for row in rows {
                let (label, value) = row?;
                labels.push(label);
                values.push(value);
            }

            Ok(AnalyticsData {
                title: format!("Chart {}", chart_id),
                labels,
                values,
            })
        }).await?
    }

    pub async fn query_drill_down(
        &self,
        series: &str,
        index: i32,
    ) -> Result<serde_json::Value, Error> {
        // Similar pattern for drill-down queries
        todo!()
    }
}
```

### Zenoh Event Bus

```rust
// src/infrastructure/event_bus.rs
use std::sync::Arc;
use zenoh::{Config, Session};
use tokio_stream::Stream;

pub struct EventBus {
    session: Arc<Session>,
}

impl EventBus {
    pub async fn new() -> Result<Self, zenoh::Error> {
        // Embedded mode - no external server
        let mut config = Config::default();
        config.set_mode(Some(zenoh::config::WhatAmI::Peer))?;

        let session = Arc::new(zenoh::open(config).await?);
        Ok(Self { session })
    }

    pub async fn publish(&self, key: &str, data: &impl serde::Serialize) -> Result<(), Error> {
        let json = serde_json::to_string(data)?;
        self.session
            .put(key, json)
            .encoding(zenoh::bytes::Encoding::APPLICATION_JSON)
            .await?;
        Ok(())
    }

    pub async fn subscribe(&self, pattern: &str) -> impl Stream<Item = serde_json::Value> {
        let subscriber = self.session
            .declare_subscriber(pattern)
            .await
            .expect("Failed to create subscriber");

        async_stream::stream! {
            loop {
                match subscriber.recv_async().await {
                    Ok(sample) => {
                        if let Ok(json) = sample.payload().try_to_string() {
                            if let Ok(value) = serde_json::from_str(&json) {
                                yield value;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
}
```

## Router Configuration

```rust
// src/presentation/routes.rs
use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use crate::presentation::handlers::charts::{chart_data_sse, chart_select, AppState};

pub fn chart_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/charts/:id/data", get(chart_data_sse))
        .route("/api/chart/select", post(chart_select))
        .with_state(state)
}
```

## Testing Setup

### vitest.config.ts

```typescript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'happy-dom',
    setupFiles: ['./__tests__/setup.ts'],
    include: ['**/__tests__/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['components/**/*.ts'],
    },
  },
});
```

### __tests__/setup.ts

```typescript
import { vi } from 'vitest';

// Mock ECharts to avoid canvas rendering in tests
const mockChartInstance = {
  handlers: new Map<string, Function[]>(),

  setOption: vi.fn(),
  resize: vi.fn(),
  dispose: vi.fn(),

  on(eventName: string, handler: Function) {
    if (!this.handlers.has(eventName)) {
      this.handlers.set(eventName, []);
    }
    this.handlers.get(eventName)!.push(handler);
  },

  off(eventName: string, handler: Function) {
    const handlers = this.handlers.get(eventName);
    if (handlers) {
      const index = handlers.indexOf(handler);
      if (index > -1) handlers.splice(index, 1);
    }
  },

  getHandlers(eventName: string): Function[] {
    return this.handlers.get(eventName) || [];
  },

  reset() {
    this.handlers.clear();
    this.setOption.mockClear();
    this.resize.mockClear();
    this.dispose.mockClear();
  },
};

vi.mock('echarts', () => ({
  init: vi.fn(() => mockChartInstance),
  dispose: vi.fn(),
}));

export { mockChartInstance };
```

### __tests__/test-utils.ts

```typescript
export async function render<T extends HTMLElement>(tagName: string): Promise<T> {
  const element = document.createElement(tagName) as T;
  document.body.appendChild(element);
  await element.updateComplete;
  return element;
}

export async function renderWithAttrs<T extends HTMLElement>(
  tagName: string,
  attrs: Record<string, string>,
): Promise<T> {
  const element = document.createElement(tagName) as T;
  for (const [key, value] of Object.entries(attrs)) {
    element.setAttribute(key, value);
  }
  document.body.appendChild(element);
  await element.updateComplete;
  return element;
}

export async function waitForLitUpdate(element: HTMLElement): Promise<void> {
  await (element as any).updateComplete;
}

export function cleanup(): void {
  document.body.innerHTML = '';
}
```

## Data Flow Summary

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Initial Page Load                                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│ Browser                         │  Server                                   │
│                                 │                                           │
│ 1. Load page                    │                                           │
│    data-on-load="@get(...)"    ─┼──▶ SSE handler                            │
│                                 │     │                                     │
│                                 │     ▼                                     │
│                                 │  2. Query DuckDB for chart data           │
│                                 │     │                                     │
│                                 │     ▼                                     │
│ 3. Receive PatchElements       ◀┼──  3. Render hypertext template           │
│    ds-echarts inserted          │     Send PatchElements                    │
│                                 │     │                                     │
│ 4. ds-echarts initializes       │     ▼                                     │
│    ECharts instance             │  4. Subscribe to Zenoh for updates        │
│                                 │                                           │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ User Interaction                                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│ Browser                         │  Server                                   │
│                                 │                                           │
│ 1. User clicks chart bar        │                                           │
│    chart-click event fired      │                                           │
│    │                            │                                           │
│    ▼                            │                                           │
│ 2. data-on:chart-click=         │                                           │
│    "$selected = evt.detail"     │                                           │
│    "@post('/api/chart/select')"─┼──▶ POST handler                           │
│                                 │     │                                     │
│                                 │     ▼                                     │
│                                 │  3. Query DuckDB for drill-down           │
│                                 │     │                                     │
│                                 │     ▼                                     │
│                                 │  4. Publish to Zenoh                      │
│                                 │     (other SSE handlers receive)          │
│                                 │     │                                     │
│ 5. Receive PatchSignals        ◀┼──   ▼                                     │
│    $detailData updated          │  5. Return PatchSignals                   │
│                                 │                                           │
│ 6. UI updates reactively        │                                           │
│    via Datastar bindings        │                                           │
│                                 │                                           │
└─────────────────────────────────────────────────────────────────────────────┘
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

1. DuckDB queries run on `spawn_blocking` thread pool
2. Analytics results cached in moka with TTL
3. Cache invalidation via Zenoh subscription
4. rkyv zero-copy deserialization for cached data
