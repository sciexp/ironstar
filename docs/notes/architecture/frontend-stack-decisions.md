# Frontend stack architecture decisions

This document records frontend technology selection decisions for the ironstar stack.
For backend, infrastructure, and CQRS implementation decisions, see the related documentation section below.

## Frontend tooling decisions

**What we use and why:**

| Tool | Role | Why |
|------|------|-----|
| Open Props + Open Props UI | CSS tokens & components | Pure CSS, zero runtime, modern CSS features |
| Rolldown | JS/CSS bundler | Rust-native (over Go-based esbuild), Vite 8 default |
| Vanilla Web Components | Third-party lib wrappers | Thin encapsulation, no reactivity system |
| Lucide | Icons | Build-time SVG inlining, zero runtime |
| TypeScript | Type safety | For minimal JS (web components only) |

**What we avoid and why:**

| Tool | Why Not |
|------|---------|
| Lit | Redundant reactivity: Lit's reactive properties duplicate Datastar signals |
| React / Vue / Svelte | SPA philosophy contradicts hypermedia-driven architecture |
| Leptos / Dioxus | Rust WASM frameworks would duplicate Datastar's role entirely |
| esbuild | Go-based; prefer Rust-native Rolldown for toolchain consistency |
| Shadcn/ui | Requires React runtime; Open Props UI achieves similar aesthetics with pure CSS |
| Tailwind CSS | JIT compiler requires build-time template scanning; Open Props uses runtime CSS variables |

**Web component pattern for Datastar integration:**

When third-party JavaScript libraries require client-side DOM manipulation (charts, drag-and-drop, rich editors), wrap them in vanilla web components that emit custom events:

```typescript
// Thin wrapper: no reactivity system, just encapsulation
class SortableList extends HTMLElement {
  connectedCallback() {
    Sortable.create(this, {
      onEnd: (evt) => {
        // Dispatch event for Datastar to handle
        this.dispatchEvent(new CustomEvent('reorder', {
          detail: { oldIndex: evt.oldIndex, newIndex: evt.newIndex },
          bubbles: true
        }));
      }
    });
  }
}
customElements.define('sortable-list', SortableList);
```

```html
<!-- Datastar handles all state and server communication -->
<sortable-list data-on:reorder="@post('/api/reorder', {body: evt.detail})">
  <!-- Items rendered by server -->
</sortable-list>
```

The web component is a thin adapter.
All state management flows through Datastar signals and server SSE responses.

---

## datastar-rust — frontend as signal algebra

**Algebraic justification:**

Datastar's signals are a *reactive graph*—essentially FRP (Functional Reactive Programming):

```rust
// Signals form an applicative functor
// - pure: lift value into signal
// - ap: combine signals
// - map: transform signal values

// PatchSignals is a morphism: JSON -> Signal State
PatchSignals::new(r#"{"count": 0, "loading": false}"#)

// PatchElements is a morphism: HTML -> DOM
PatchElements::new(render_component(state))
```

**SSE as a stream of patches:**

```rust
use axum::response::sse::{Event, Sse};
use datastar::prelude::*;  // PatchSignals
use futures::stream::{self, Stream};
use std::{convert::Infallible, time::Duration};

// The SSE stream is Free[Patch, DOM] — a program describing UI updates
async fn counter_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::unfold(0, |count| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let patch = PatchSignals::new(format!(r#"{{"count": {}}}"#, count + 1));
        Some((Ok(patch.write_as_axum_sse_event()), count + 1))
    });
    Sse::new(stream)
}
```

**ReadSignals extractor — type-safe signal parsing:**

The `ReadSignals<T>` extractor provides ergonomic, type-safe parsing of incoming Datastar signals.
It handles both GET (query param) and POST (JSON body) transparently:

```rust
use datastar::axum::ReadSignals;
use serde::Deserialize;

// Signal contract as algebraic product type
#[derive(Deserialize)]
struct TodoSignals {
    input: Option<String>,
    filter: TodoFilter,
    editing_id: Option<uuid::Uuid>,
}

#[derive(Deserialize)]
enum TodoFilter {
    All,
    Active,
    Completed,
}

// ReadSignals extracts and deserializes in one step
// Morphism: Request -> Result<T, Rejection>
async fn handle_add_todo(
    State(store): State<EventStore>,
    ReadSignals(signals): ReadSignals<TodoSignals>,
) -> impl IntoResponse {
    // signals.input, signals.filter available as typed values
    if let Some(text) = signals.input {
        store.append(TodoEvent::Added { text }).await?;
    }
    StatusCode::ACCEPTED
}
```

This pattern is preferred over manual JSON parsing.
The alternative (wrapping with `Query<Wrapper>` and `serde_json::from_str`) is verbose and error-prone.

**ts-rs integration for type-safe signal contracts:**

Signal types defined in Rust can generate corresponding TypeScript definitions via ts-rs, ensuring frontend and backend contracts stay synchronized:

```rust
use ts_rs::TS;
use serde::{Serialize, Deserialize};

// Derive TS alongside Serialize/Deserialize
#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "web-components/types/")]
pub struct TodoSignals {
    #[ts(optional)]
    pub input: Option<String>,
    pub filter: TodoFilter,
    #[ts(optional)]
    pub editing_id: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize, TS)]
pub enum TodoFilter {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "completed")]
    Completed,
}
```

Running `cargo test --lib` generates TypeScript in `web-components/types/`:

```typescript
// TodoSignals.ts (auto-generated)
export type TodoSignals = {
  input?: string;
  filter: TodoFilter;
  editing_id?: string;
};

export type TodoFilter = "all" | "active" | "completed";
```

This ensures the JSON structure in `data-signals` attributes matches what `ReadSignals<T>` expects.
See `signal-contracts.md` for detailed integration patterns.

---

## open-props — design tokens as constants

**Algebraic justification:**

Open Props provides design tokens as CSS custom properties, representing a fundamentally different abstraction than utility-class systems.
Rather than a combinator language that generates styles at build time, Open Props is a *vocabulary* of constants that exist at runtime as CSS variables:

```rust
// Open Props tokens are constant lookups: TokenName -> Value
// This is dictionary access, not morphism application
maud! {
    div style="
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: var(--size-4);
        background: var(--gray-2);
        border-radius: var(--radius-2);
    " {
        // --size-4: constant value from sizing scale
        // --gray-2: constant value from color palette
        // var() performs lookup, not generation
    }
}
```

**Algebraic model: constants vs. morphisms**

Where Tailwind implements `ClassName -> CSSProperty` (a morphism from class to generated style), Open Props implements `TokenName -> Value` (a constant lookup).
The key difference is that Open Props tokens are *pre-defined values* referenced directly in CSS, while Tailwind classes are *instructions* for generating CSS at build time.

This makes Open Props closer to a **map data structure** than a combinator algebra.
The composition happens in your own CSS rules, not in generated utilities.

**Token categories:**

Open Props provides "sub-atomic styles" organized into semantic categories:

- **Colors**: 19 color palettes (gray, red, pink, purple, violet, indigo, blue, cyan, teal, green, lime, yellow, orange, choco, brown, sand, camo, jungle, stone) with OKLCH variants for modern color spaces
- **Sizing**: `--size-*` (rem-based), `--size-fluid-*` (clamp-based responsive), `--size-content-*`, `--size-header-*`, `--size-*-px` (pixel-based)
- **Typography**: `--font-sans`, `--font-serif`, `--font-mono`, `--font-weight-*`, `--font-size-*`, `--font-lineheight-*`
- **Spacing**: `--space-*` for margins/padding
- **Animations**: 15+ keyframe animations (`@keyframes` defined, ready to use)
- **Easings**: 60+ easing functions including spring, elastic, bounce variations
- **Shadows**: `--shadow-*` (elevation-based), `--inner-shadow-*`
- **Borders**: `--border-size-*`, `--radius-*` (border radius), `--radius-*-px`
- **Gradients**: predefined gradient values
- **Z-index**: `--layer-*` (semantic layering)
- **Aspect ratios**: `--ratio-*` (square, landscape, portrait, widescreen, ultrawide, golden)

**Framework-agnostic abstraction:**

CSS custom properties are browser-native.
This means:

- No build-time class scanning or JIT compilation required
- Works with any templating system (hypertext, JSX, plain HTML)
- Tokens are defined once, available everywhere
- No purging step needed (tokens are referenced, not generated)

**Why chosen:**

Open Props aligns better with Datastar's hypermedia philosophy.
When the server generates complete HTML fragments, it needs to reference *known class names* or *CSS properties*.
Open Props provides stable token names that can be referenced in inline styles or custom CSS classes without requiring a build-time scanning step to detect which utilities are used.

The backend-driven model means: server knows exact styling -> emits HTML with tokens -> browser applies via native CSS variables.
There's no need for a JIT compiler to watch template files and generate utility classes.

**Local repository:**

- `open-props` at `~/projects/lakescope-workspace/open-props` - CSS design tokens library

**Integration pattern:**

```rust
// Import Open Props tokens in your CSS
// @import "open-props/style";
// Or selective imports:
// @import "open-props/colors";
// @import "open-props/sizes";

// Then reference in templates via inline styles or custom classes
maud! {
    button style="
        padding: var(--size-2) var(--size-4);
        background: var(--blue-6);
        color: var(--gray-0);
        border-radius: var(--radius-2);
        font-weight: var(--font-weight-6);
    " {
        "Submit"
    }
}
```

**Effect boundary:**

Open Props CSS is imported at load time as standard CSS.
No build-time generation step required beyond standard CSS concatenation/minification.
The browser's CSS engine resolves `var()` lookups during style computation.

---

## open-props-ui — semantic components via modern CSS

**Algebraic justification:**

Open Props UI extends Open Props tokens into a component layer using pure CSS and modern browser features.
Rather than generating utility classes or requiring a JavaScript framework, it provides semantic component classes that compose via a three-layer architecture:

```rust
// Three-layer composition:
// 1. Open Props tokens (constants)
// 2. Theme variables (application-specific derivations)
// 3. Component classes (semantic composition)

// Layer 1: Token constants
// --blue-6, --size-2, --radius-2 (from Open Props)

// Layer 2: Theme variables (your app-level CSS)
// --brand-primary: var(--blue-6);
// --spacing-md: var(--size-2);
// --corner-md: var(--radius-2);

// Layer 3: Component classes
maud! {
    button class="button filled" {
        // .button uses theme variables
        // .filled uses --brand-primary
        "Submit"
    }

    div class="card" {
        // .card uses theme variables for padding, radius, shadow
        h2 { "Card Title" }
        p { "Card content goes here." }
    }
}
```

**Modern CSS features:**

Open Props UI leverages CSS capabilities from 2023+ browsers:

- **OKLCH colors**: Perceptually uniform color space via `oklch()` function
- **Container queries**: `@container` for component-scoped responsive behavior
- **CSS layers**: `@layer` for cascade management
- **color-mix()**: Dynamic color blending without preprocessor
- **light-dark()**: Native light/dark theme switching without JavaScript
- **Nesting**: Native CSS nesting without Sass/PostCSS
- **:has()**: Parent selector for relational styling

Browser requirements: Chrome 111+, Firefox 119+, Safari 17+ (mid-2023 forward).

**Component catalog:**

Open Props UI provides 31 component types as semantic CSS classes.
Note: naming conventions differ from common patterns: dialog (not modal), snackbar (not toast), range (not slider), chip (not tag), tab-buttons (not tabs).

Available components:
- **Forms**: button, button-group, checkbox-radio, field, field-group, icon-button, link, range, select, switch, text-field, textarea, toggle-button-group
- **Layout**: card, dialog, divider
- **Navigation**: tab-buttons
- **Feedback**: alert, badge, progress, snackbar, spinner, tooltip
- **Data**: avatar, chip, definition-list, list, table, typography
- **Utilities**: accordion, rich-text

**Copy-paste ownership model:**

Open Props UI differs from traditional component libraries.
Components are *copied into your project*, not imported from node_modules:

```bash
# Typical workflow
cp node_modules/open-props-ui/components/button.css src/components/
# Edit button.css to match your needs
```

This gives you **full ownership**:

- Modify component styles directly (no overrides needed)
- No breaking changes from library updates
- Complete transparency (you own the CSS)
- Tree-shake automatically (only include what you use)

**Why chosen over DaisyUI:**

| Consideration | DaisyUI | Open Props UI |
|--------------|---------|---------------|
| Build dependencies | Tailwind JIT compiler, PostCSS | None (pure CSS) |
| Class generation | Build-time utility scanning | No generation step |
| Modern CSS | Limited (Tailwind constraints) | Full (OKLch, container queries, :has()) |
| Ownership model | NPM dependency, version updates | Copy-paste, full control |
| Server-side HTML | Requires JIT to scan templates | Direct token/class reference |
| Theme system | CSS variables via Tailwind config | Native CSS custom properties |

For Ironstar's hypermedia-driven architecture, Open Props UI provides:

1. **Stable class names**: The server can emit `class="button filled"` without build-time coordination
2. **No JIT scanning**: Backend templates don't need to be watched for class extraction
3. **Modern CSS alignment**: Embraces browser-native features over build-time abstractions
4. **Copy-paste ownership**: Full control over component CSS without override complexity

**Why not Shadcn/ui:**

Shadcn/ui provides similar aesthetics but requires React:

```tsx
// Shadcn: React component with JS runtime
import { Button } from "@/components/ui/button"
<Button variant="primary" size="lg">Submit</Button>
```

```rust
// Open Props UI: Pure CSS class, no runtime
maud! { button class="button filled large" { "Submit" } }
```

When Datastar already provides reactivity, adding React for UI components introduces redundant complexity.

**Local repository:**

- `open-props-ui` at `~/projects/lakescope-workspace/open-props-ui` - Pure CSS component library

**Integration pattern:**

```rust
// 1. Import Open Props and component CSS in your stylesheet
// @import "open-props/style";
// @import "./components/button.css";  // Copied from open-props-ui
// @import "./components/card.css";

// 2. Define theme variables (optional customization layer)
// :root {
//   --brand-primary: var(--blue-6);
//   --brand-secondary: var(--purple-6);
// }

// 3. Use semantic classes in templates
maud! {
    button class="button filled large" {
        "Submit"
    }

    div class="card" {
        div class="card-header" {
            h2 { "Title" }
        }
        div class="card-body" {
            p { "Content goes here." }
        }
    }
}
```

**Light DOM requirement for web components:**

All web components using Open Props tokens must use Light DOM rendering.
This applies to visualization components like ds-echarts (Lit component for ECharts) and vega-chart (vanilla component for Vega-Lite).

Shadow DOM blocks CSS custom property inheritance, preventing theme token access.
By keeping web components in Light DOM, they inherit the parent document's CSS variables and can reference design tokens directly.

**Effect boundary:**

Open Props UI CSS is loaded as standard stylesheets.
Theme switching via `light-dark()` is handled by the browser's native color scheme support (no JavaScript).
Component styles apply immediately via the browser's CSS engine with no runtime compilation.

---

## Lucide — icons as pure data

**Algebraic justification:**

Icons are *constants*: they do not vary with application state.
This places them in a different category than reactive UI elements.
The optimal representation is as *compile-time constants*, not runtime-fetched resources.

Lucide provides icons as pure functions from props to SVG:

```typescript
// Each icon is a pure function: IconProps -> SVGElement
// No side effects, no state, no network requests
import { Camera, Settings, User } from 'lucide-static';

// Icon as string constant (build-time resolved)
const cameraIcon: string = Camera;  // Pure SVG markup

// Or in hypertext templates (Rust side):
// The SVG string is embedded at compile time
maud! {
    button class="btn" {
        (Raw::dangerously_create(include_str!("../static/icons/camera.svg")))
        "Upload"
    }
}
```

**Build-time inlining pattern:**

Rather than loading icons at runtime (HTTP requests, bundle bloat, FOUC), Lucide icons are inlined during the build:

```
Build step: Icon name -> SVG file -> Embedded constant
Runtime: Zero icon-related network requests, zero JS execution
```

This is the *Yoneda embedding* for icons: instead of asking "what icon should I render?" at runtime, we pre-compute the answer and embed it.
The icon's identity is its content, making caching trivial (content-addressed).

**Why build-time over runtime:**

| Approach | HTTP Requests | JS Runtime | Bundle Size | FOUC Risk |
|----------|--------------|------------|-------------|-----------|
| Runtime fetch | N per icon | Required | Small initial | High |
| Icon font | 1 (all icons) | CSS only | Large fixed | Medium |
| SVG sprite | 1 (all icons) | None | Large fixed | Low |
| Build-time inline | 0 | None | Minimal (used only) | None |

Build-time inlining achieves the optimal trade-off: zero runtime overhead, minimal bundle size (tree-shaking applies), and no flash of unstyled content.

**Why Lucide specifically:**

- Consistent 24x24 grid with 2px stroke (compositional uniformity)
- 1600+ icons covering common UI patterns
- MIT licensed, actively maintained fork of Feather Icons
- Multiple output formats: individual SVGs, static strings, framework components
- `lucide-static` package provides SVG strings for server-side embedding

**Integration with hypertext:**

```rust
// icons.rs - Generated at build time by a simple script
pub mod icons {
    pub const CAMERA: &str = include_str!("../../static/icons/camera.svg");
    pub const SETTINGS: &str = include_str!("../../static/icons/settings.svg");
    pub const USER: &str = include_str!("../../static/icons/user.svg");
}

// In templates
use crate::icons;

fn icon_button(icon: &str, label: &str) -> impl Renderable {
    maud! {
        button class="button icon-only" {
            (Raw::dangerously_create(icon))
            span { (label) }
        }
    }
}

// Usage: icon_button(icons::CAMERA, "Upload")
```

---

## Related documentation

- Design principles: `design-principles.md`
- Backend core decisions: `backend-core-decisions.md`
- Infrastructure decisions: `infrastructure-decisions.md`
- CQRS implementation: `cqrs-implementation-decisions.md`
- Build tooling decisions: `build-tooling-decisions.md`
- Event sourcing core concepts: `event-sourcing-core.md`
- SSE connection lifecycle: `sse-connection-lifecycle.md`
- Signal contracts: `signal-contracts.md`
- Build pipeline: `frontend-build-pipeline.md`
- Third-party integration: `integration-patterns.md`
