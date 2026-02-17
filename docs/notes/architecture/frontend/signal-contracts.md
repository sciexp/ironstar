# Type-safe signal contracts with ts-rs

> **Semantic foundation**: Datastar signals form a comonad, dual to server-side monads.
> The `extract` operation (accessing signal value) and `extend` operation (computed signals) satisfy comonad laws.
> See [semantic-model.md § Comonad](../core/semantic-model.md#client-signals-as-comonad).

Ironstar uses ts-rs to generate TypeScript type definitions from Rust signal structs, ensuring type safety across the full Datastar request/response cycle.

## Comonad operations in practice

The `extract` operation gets the current signal value:

```typescript
const count = $count.value;  // extract: Signal Int → Int
```

The `extend` operation derives computed signals:

```typescript
// extend: (Signal Int → Bool) → Signal Int → Signal Bool
const isPositive = computed(() => $count.value > 0);
```

### Comonad laws and their guarantees

Three laws govern signal composition.
Together they guarantee that derived signals behave predictably regardless of how the derivation chain is structured.

**Law 1: `extend extract = id`.** Wrapping a signal in a computed that simply reads its value produces the original signal.
In Datastar terms, `computed(() => $count.value)` is functionally identical to `$count`.
This means trivial computed wrappers introduce no semantic difference, so refactoring can freely introduce or remove them.

**Law 2: `extract . extend f = f`.** Reading the value of a computed signal is the same as applying the derivation function directly to the source.
In Datastar terms, if `isPositive = computed(() => $count.value > 0)`, then `$isPositive.value` always equals `$count.value > 0`.
This guarantees there is no hidden state or timing artifact in computed signal evaluation.

**Law 3: `extend f . extend g = extend (f . extend g)`.** Chaining two computed derivations is equivalent to a single computed that inlines both steps.
In Datastar terms, building `doubled = computed(() => $count.value * 2)` and then `isLarge = computed(() => $doubled.value > 100)` is the same as writing `isLarge = computed(() => ($count.value * 2) > 100)` directly.
This means signal derivation chains can be refactored between nested and flattened forms without changing behavior.

These laws are verified as executable Rust tests in `crates/ironstar/src/domain/signals.rs` using a minimal `Signal<A>` model that mirrors the comonad interface.
The model is test-only (`#[cfg(test)]`) since production signals are managed by Datastar's JavaScript runtime.
Tests exercise the laws with concrete types including `i32`, `String`, `TodoFilter`, and `TodoSignals` to confirm the algebraic properties hold across the signal types used in the application.

## The contract problem

Datastar signals flow through multiple boundaries:

```
HTML template (data-signals="{...}")
    ↓ JSON string
Browser (Datastar signals)
    ↓ JSON body/query param
Rust handler (ReadSignals<T>)
    ↓ serde deserialization
Rust struct
```

Without type synchronization, mismatches can occur:
- Frontend sends `filter: "all"`, backend expects `filter: "All"` (case mismatch)
- Frontend sends `count: "5"`, backend expects `count: 5` (type mismatch)
- Frontend sends extra fields backend doesn't expect (silent failures)

ts-rs solves this by generating TypeScript types from the authoritative Rust definitions.

---

## When to create signal types

Signals represent client-side reactive state for Datastar's FRP model.
They exist to enable two-way form binding, persist UI state across DOM morphs, and track loading indicators.

**Signals derive from UI requirements, not aggregate structure.**
Not every aggregate needs signal types.
Create signals when the UI design requires them, not preemptively for each domain aggregate.

| UI Requirement | Needs Signals? | Example |
|----------------|----------------|---------|
| Two-way form input binding | Yes | `data-bind:inputField` needs signal storage |
| Filter/selection state | Yes | Client maintains which filter is active |
| Loading indicators | Yes | Client tracks pending request state |
| Editing mode (which item) | Yes | Client tracks which item is being edited |
| Server-pushed HTML fragments | No | SSE sends HTML, client morphs DOM |
| Display-only projections | No | Server renders, client displays |
| Redirect-based flows (OAuth) | No | No client state to maintain |

**Timing guidance:**
- Create signal types when UI wireframes or component designs exist
- Do not create signal issues for aggregates whose UI is not yet designed
- Chart signals (ChartSignals, ChartSelection) exist because ECharts integration requires explicit state contracts
- Simple bounded contexts (Session, QuerySession) may not need dedicated signal types if their UI is server-rendered

This prevents premature abstraction and ensures signals match actual UI needs rather than hypothetical requirements.

---

## Basic usage

### Defining signal types in Rust

```rust
// src/domain/signals.rs
use ts_rs::TS;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Todo application signals
#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "web-components/types/")]
pub struct TodoSignals {
    /// Current input field value
    #[ts(optional)]
    pub input: Option<String>,

    /// Active filter mode
    pub filter: TodoFilter,

    /// ID of todo being edited (if any)
    #[ts(optional)]
    pub editing_id: Option<Uuid>,

    /// Loading indicator state
    #[serde(default)]
    pub loading: bool,
}

/// Filter options for todo list
#[derive(Clone, Copy, Serialize, Deserialize, TS)]
pub enum TodoFilter {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "completed")]
    Completed,
}

impl Default for TodoFilter {
    fn default() -> Self {
        Self::All
    }
}
```

### Generating TypeScript types

ts-rs generates types during test execution:

```bash
# Generate all exported types
cargo test --lib

# Or with explicit environment variable
TS_RS_EXPORT_DIR=web-components/types cargo test --lib
```

This produces `web-components/types/TodoSignals.ts`:

```typescript
// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
export type TodoSignals = {
  input?: string;
  filter: TodoFilter;
  editing_id?: string;
  loading: boolean;
};

export type TodoFilter = "all" | "active" | "completed";
```

---

## Type mappings

ts-rs converts Rust types to TypeScript according to these rules:

| Rust Type | TypeScript Type | Notes |
|-----------|-----------------|-------|
| `String` | `string` | |
| `i8`, `u8`, `i16`, `u16`, `i32`, `u32` | `number` | |
| `f32`, `f64` | `number` | |
| `i64`, `u64`, `i128`, `u128` | `bigint` | Configurable via `TS_RS_LARGE_INT` env var |
| `bool` | `boolean` | |
| `Option<T>` | `T \| null` | Or `T?` with `#[ts(optional)]` |
| `Vec<T>` | `Array<T>` | |
| `HashMap<K, V>` | `{ [key in K]?: V }` | |
| `Uuid` | `string` | Requires `uuid-impl` feature |
| Unit variants | String literal union | `"All" \| "Active"` |
| Struct variants | Discriminated union | With `#[serde(tag = "type")]` |

### Option handling

By default, `Option<T>` becomes `T | null`. Use `#[ts(optional)]` for optional fields:

```rust
#[derive(TS)]
struct Signals {
    // Required field: must be present, can be null
    name: Option<String>,  // -> name: string | null

    // Optional field: can be omitted entirely
    #[ts(optional)]
    nickname: Option<String>,  // -> nickname?: string
}
```

For struct-wide optional fields:

```rust
#[derive(TS)]
#[ts(optional_fields)]
struct Signals {
    name: Option<String>,     // -> name?: string
    nickname: Option<String>, // -> nickname?: string
}
```

### Enum serialization

ts-rs respects serde's enum representations:

```rust
// External tagging (default): {"Active": null} or {"WithData": {...}}
#[derive(Serialize, TS)]
enum Status {
    Active,
    WithData { count: i32 },
}

// Internal tagging: {"type": "Active"} or {"type": "WithData", "count": 5}
#[derive(Serialize, TS)]
#[serde(tag = "type")]
enum Action {
    Add { text: String },
    Delete { id: String },
}

// Untagged: just the variant value
#[derive(Serialize, TS)]
#[serde(untagged)]
enum StringOrNumber {
    String(String),
    Number(i32),
}
```

---

## Project structure

Recommended layout for ironstar:

```
ironstar/
├── .cargo/
│   └── config.toml              # TS_RS_EXPORT_DIR setting
├── Cargo.toml
├── src/
│   └── domain/
│       ├── mod.rs
│       └── signals.rs           # Signal type definitions
├── web-components/
│   ├── types/                   # Generated TypeScript (git-ignored)
│   │   ├── TodoSignals.ts
│   │   └── TodoFilter.ts
│   ├── tsconfig.json            # Include types/ in compilation
│   └── index.ts
└── justfile                     # Build tasks
```

### Cargo configuration

```toml
# .cargo/config.toml
[env]
TS_RS_EXPORT_DIR = { value = "web-components/types", relative = true }
```

### Cargo.toml dependencies

The `#[ts(export, export_to = "...")]` attribute syntax requires ts-rs version 10.0 or later.
Earlier versions used a different export mechanism.

```toml
[dependencies]
ts-rs = { version = "11.1", features = ["serde-compat", "uuid-impl"] }
serde = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
# None needed for ts-rs
```

### TypeScript configuration

```json
// web-components/tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "module": "ESNext",
    "target": "ESNext",
    "moduleResolution": "bundler",
    "paths": {
      "@types/*": ["./types/*"]
    }
  },
  "include": ["**/*.ts", "types/**/*.ts"]
}
```

### Build integration

```justfile
# justfile

# Generate TypeScript types from Rust signal definitions
gen-types:
    TS_RS_EXPORT_DIR=web-components/types cargo test --lib 2>&1 | grep -E "^test|export_bindings"
    @echo "Generated types in web-components/types/"

# Run all builds
build: gen-types
    cd web-components && pnpm build
    cargo build --release

# Development with type generation
dev:
    @just gen-types
    # Start watchers...
```

---

## Usage in Datastar templates

### Server-rendered initial state

```rust
use hypertext::prelude::*;
use crate::domain::signals::{TodoSignals, TodoFilter};

fn todo_page(signals: &TodoSignals) -> impl Renderable {
    maud! {
        div
            #main
            "data-signals"=(serde_json::to_string(signals).unwrap())
            "data-init"="@get('/todos/updates')"
        {
            // Page content
        }
    }
}

// Handler
async fn page() -> impl IntoResponse {
    let signals = TodoSignals {
        input: None,
        filter: TodoFilter::All,
        editing_id: None,
        loading: false,
    };
    todo_page(&signals).render()
}
```

### Frontend type safety

```typescript
// web-components/components/todo-form.ts
import type { TodoSignals, TodoFilter } from '@types/TodoSignals';

// TypeScript enforces correct signal structure
const validateSignals = (signals: TodoSignals): boolean => {
  // Type-safe access to signal properties
  const validFilters: TodoFilter[] = ['all', 'active', 'completed'];
  return validFilters.includes(signals.filter);
};
```

### Type-safe API calls

```typescript
// The TodoSignals type ensures correct JSON structure
const signals: TodoSignals = {
  input: 'New todo',
  filter: 'all',
  loading: true,
};

// Datastar sends this as the request body
// ReadSignals<TodoSignals> on the Rust side will parse it correctly
```

---

## Best practices

### Use serde rename consistently

serde attributes affect both JSON serialization and TypeScript output:

```rust
#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
struct UserSignals {
    first_name: String,  // -> firstName in both JSON and TS
    last_name: String,   // -> lastName in both JSON and TS
}
```

### Define signals in domain layer

Keep signal types with your domain model, not scattered across handlers:

```rust
// src/domain/signals.rs - centralized signal definitions
pub mod todo;
pub mod user;
pub mod settings;

// Re-export for convenience
pub use todo::*;
pub use user::*;
```

### Generate types in CI

Add type generation to your CI pipeline to catch mismatches:

```yaml
# .github/workflows/ci.yml
- name: Generate TypeScript types
  run: cargo test --lib
- name: Check TypeScript compiles
  run: cd web-components && pnpm tsc --noEmit
```

### Version control strategy

Option A: Commit generated types (simpler for frontend developers)

```gitignore
# Don't ignore types
# web-components/types/
```

Option B: Generate on build (ensures freshness)

```gitignore
# Ignore generated types
web-components/types/
```

Ironstar recommends Option A for development experience, with CI verification.

---

## Limitations and workarounds

### No build.rs integration

ts-rs generates types via `cargo test`, not `cargo build`. This is intentional: type generation is a development-time concern, not a runtime concern.

Workaround: Run `just gen-types` before `pnpm build` in your development workflow.

### Generic types require concrete specifications

```rust
// Won't work: generic T is not exported
#[derive(TS)]
struct Response<T> { data: T }

// Works: concrete type specification
#[derive(TS)]
#[ts(concrete(T = String))]
struct Response<T> { data: T }
```

### Recursive types need explicit handling

```rust
// May cause issues with deeply recursive types
#[derive(TS)]
struct TreeNode {
    value: String,
    children: Vec<TreeNode>,  // Recursive
}

// Workaround: use #[ts(type = "...")] for custom handling
```

---

## Chart signal contracts

### ECharts configuration signals

Chart components require signal types for configuration and event handling.
The ds-echarts component receives chart options via signals and emits events when users interact with the chart.

```rust
// src/domain/signals.rs
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Signals for ds-echarts component state
#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../web-components/types/")]
pub struct ChartSignals {
    /// ECharts option object (passed to setOption)
    #[serde(rename = "chartOption")]
    pub chart_option: serde_json::Value,

    /// Currently selected data point from chart-click or chart-dblclick
    #[ts(optional)]
    pub selected: Option<ChartSelection>,

    /// Loading state for UI feedback during data refresh
    #[serde(default)]
    pub loading: bool,
}

/// Payload from chart-click, chart-dblclick, and mouseover events
#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChartSelection {
    /// Series name (bar series, line series, etc.)
    #[serde(rename = "seriesName")]
    pub series_name: String,

    /// Index of the selected data point in the series
    #[serde(rename = "dataIndex")]
    pub data_index: i32,

    /// Category name or x-axis label
    pub name: String,

    /// The data value (y-axis or other dimension)
    pub value: serde_json::Value,
}
```

### Generated TypeScript

```typescript
// web-components/types/ChartSignals.ts (auto-generated by ts-rs)
export type ChartSignals = {
  chartOption: unknown;
  selected?: ChartSelection;
  loading: boolean;
};

export type ChartSelection = {
  seriesName: string;
  dataIndex: number;
  name: string;
  value: unknown;
};
```

### Usage in handlers

Type-safe access to chart event payloads via `ReadSignals<ChartSelection>`:

```rust
use axum::{response::IntoResponse, Json};
use datastar::ReadSignals;
use crate::domain::signals::ChartSelection;

/// Handle chart click events
async fn handle_chart_click(
    ReadSignals(selection): ReadSignals<ChartSelection>,
) -> impl IntoResponse {
    // Type-safe access to event properties
    let series = &selection.series_name;
    let index = selection.data_index;
    let label = &selection.name;

    // Perform query or update based on selection
    Json(serde_json::json!({
        "series": series,
        "point": label,
        "details": selection.value
    }))
}
```

### HTML integration

```html
<!-- In Rust hypertext template -->
<div
    id="chart-container"
    data-signals="{chartOption: {}, selected: null, loading: false}"
    data-on:chart-click="@post('/api/chart-selection', {body: evt.detail})"
>
    <ds-echarts
        data-attr:option="$chartOption"
        data-attr:loading="$loading"
    ></ds-echarts>
</div>
```

The component receives option updates via `$chartOption` and reports selections back through the `chart-click` event, which triggers the POST handler with the typed `ChartSelection` payload.

---

## Related documentation

- datastar-rust ReadSignals: `~/projects/rust-workspace/datastar-rust/src/axum.rs`
- ts-rs documentation: `~/projects/rust-workspace/ts-rs/`
- Signal extraction patterns: see the "datastar-rust — frontend as signal algebra" section in `../core/architecture-decisions.md`
