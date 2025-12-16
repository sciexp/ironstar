---
title: Frontend build pipeline component selection (draft)
---

# Frontend build pipeline component selection

Draft sections for integration into `stack-component-selection.md`.

---

## Component justifications

### 11. Rolldown + PostCSS — build as a pure morphism

**Algebraic justification:**

A bundler implements a morphism in the category of module graphs: `Bundle: ModuleGraph -> OutputChunks`.
For this morphism to compose cleanly with other build steps, it must be *referentially transparent*: identical inputs yield identical outputs.

Rolldown's three-stage pipeline makes this explicit:

```
Scan: Sources -> ModuleGraph      (parsing as functor)
Link: ModuleGraph -> SymbolTable  (resolution as join semilattice)
Generate: (ModuleGraph, SymbolTable) -> Chunks  (codegen as fold)
```

Each stage transforms immutable data structures.
The `ModuleTable` and `SymbolRefDb` built during scanning are not mutated during linking; linking produces a new `SymbolTable`.
This mirrors the *free monad* pattern: build a description (module graph), interpret at the boundary (generate chunks).

```typescript
// rolldown.config.ts
import { defineConfig } from 'rolldown';
import postcss from 'rolldown-plugin-postcss';

export default defineConfig({
  input: 'web-components/index.ts',
  output: {
    dir: 'static/dist',
    format: 'esm',
    // Deterministic chunk naming: content hash as identity
    entryFileNames: '[name].[hash].js',
    chunkFileNames: '[name].[hash].js',
  },
  plugins: [
    // PostCSS runs within the bundle pipeline
    // Tailwind's @apply expansion is a macro: ClassName -> CSS properties
    postcss({
      config: './postcss.config.js',
      extract: 'bundle.css',
      minimize: true,
    }),
  ],
});
```

**Why Rust-native matters:**

| Property | esbuild (Go) | Rolldown (Rust) |
|----------|--------------|-----------------|
| Memory model | GC-managed heap | Ownership + borrowing |
| Parallelism | Goroutines + shared heap | Rayon + zero-copy |
| WASM performance | 22s (2.5k modules) | 613ms (2.5k modules) |
| Memory overhead | GC pauses | Deterministic allocation |

The algebraic significance: Rust's ownership system provides *linear types* at the language level.
A `ModuleTask` owns its module data; when processing completes, ownership transfers to `ModuleTable`.
No defensive copies, no GC pauses during the build effect.

**Why Rolldown over esbuild:**

- esbuild lacks Rollup-compatible plugin API (ecosystem fragmentation)
- esbuild's tree-shaking is less sophisticated (larger bundles)
- esbuild's WASM performance degrades significantly in browser environments
- Rolldown unifies dev bundling and production bundling (same tool, same behavior)
- Rust aligns with the stack's language choice (no Go in the dependency tree)

**Effect boundary:**

The build is a single effect executed at deploy time or watch-mode trigger.
All file I/O, network fetches, and process spawning occur within `rolldown build`.
The effect completes atomically: either all outputs are written or none are (via temp files + rename).

**PostCSS integration:**

PostCSS plugins (including `@tailwindcss/postcss`) run as transforms within the bundle pipeline.
Tailwind's class extraction is a *fold* over the source AST: `Fold(Source, ClassSet) -> CSS`.
The `@source` directive scopes this fold to specific paths, making the dependency explicit in the configuration rather than implicit via file watchers.

---

### 12. Lucide — icons as pure data

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
        (PreEscaped(include_str!("../static/icons/camera.svg")))
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
        button class="btn btn-icon" {
            (PreEscaped(icon))
            span { (label) }
        }
    }
}

// Usage: icon_button(icons::CAMERA, "Upload")
```

---

## Component selection matrix (additional rows)

| Component | Role | Algebraic Property | Effect Boundary |
|-----------|------|-------------------|-----------------|
| **Rolldown** | JS/CSS bundler | Pure morphism (deterministic) | `rolldown build` |
| **PostCSS** | CSS transforms | Fold over AST | Within Rolldown |
| **Lucide** | Icons | Constants (Yoneda embedding) | Build time |

---

## Note on TypeScript's scoped role

TypeScript is used *exclusively* for web components in this stack.
This constraint is intentional and algebraically motivated.

**The hypermedia philosophy:**

Datastar's core principle is server-driven UI: the server sends HTML fragments and signal updates via SSE.
Application state lives in datastar signals, not client-side JavaScript.
This inverts the typical SPA architecture where TypeScript manages the entire application state machine.

**TypeScript's limited scope:**

```
TypeScript in ironstar:
├── Web component wrappers (thin DOM encapsulation)
│   ├── sortable-list.ts  -- Wraps Sortable.js
│   └── vega-chart.ts     -- Wraps vega-embed
└── Type definitions for datastar attributes (optional DX improvement)

TypeScript NOT used for:
├── Application logic (lives in Rust)
├── State management (datastar signals)
├── Routing (server-side)
└── Data fetching (server-initiated via SSE)
```

**Algebraic justification:**

Web components in this model are *natural transformations* between the browser's DOM API and datastar's event system.
They translate third-party library events into custom events that datastar can observe:

```typescript
// Natural transformation: SortableEvent -> CustomEvent
class SortableList extends HTMLElement {
  connectedCallback() {
    Sortable.create(this, {
      onEnd: (evt: SortableEvent) => {
        // η: SortableEvent -> CustomEvent (natural transformation)
        this.dispatchEvent(new CustomEvent('reorder', {
          detail: { oldIndex: evt.oldIndex, newIndex: evt.newIndex },
          bubbles: true
        }));
      }
    });
  }
}
```

The component contains no application logic.
It is a *functor* mapping one event category to another.
All state transitions occur on the server via datastar's `@post` or `@put` actions.

**Why this constraint matters:**

1. *Single source of truth*: Application state lives in one place (server + datastar signals)
2. *Testability*: Business logic tested in Rust, not split across languages
3. *Bundle size*: No React/Vue/Svelte framework code
4. *Conceptual integrity*: The server is the application; the browser is a view

---

## Sources

Research sources for this draft:

- [Rolldown official site](https://rolldown.rs/)
- [Rolldown GitHub repository](https://github.com/rolldown/rolldown)
- [Rolldown bundling pipeline architecture (DeepWiki)](https://deepwiki.com/rolldown/rolldown/2.1-bundling-pipeline)
- [VoidZero's Rolldown Library announcement (InfoQ)](https://www.infoq.com/news/2025/11/rolldown-bundler-rust/)
- [Rolldown vs esbuild WASM performance](https://x.com/youyuxi/status/1869608132386922720)
- [Lucide icons official guide](https://lucide.dev/guide/)
- [Lucide static package documentation](https://lucide.dev/guide/packages/lucide-static)
- [Lucide React technical deep dive](https://expertbeacon.com/lucide-react-technical-guide/)
- [Tailwind CSS PostCSS installation](https://tailwindcss.com/docs/installation/using-postcss)
- [leptos-lucide-rs (Rust zero-cost icons)](https://github.com/crabtools-rs/leptos-lucide-rs)
