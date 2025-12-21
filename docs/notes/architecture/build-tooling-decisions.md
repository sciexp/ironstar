# Build tooling architecture decisions

This document records build tooling technology selection decisions for the ironstar stack, covering JavaScript/CSS bundling and asset processing.
For frontend, backend core, infrastructure, and CQRS decisions, see the related documentation section below.

## Rolldown — build as a pure morphism

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
    // Open Props imports and custom CSS processed as standard CSS
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

PostCSS plugins run as transforms within the bundle pipeline.
For Open Props, this primarily handles CSS imports, autoprefixing, and minification.
No class extraction or template scanning is required—Open Props tokens are referenced directly in CSS via `var()`, making the pipeline a straightforward transform: `Import(CSS) -> Optimize(CSS) -> Output`.

**Local repository:**

- `rolldown` at `~/projects/rust-workspace/rolldown` - Rust-native JavaScript/CSS bundler

---

## PostCSS configuration

PostCSS processes CSS transforms within the Rolldown pipeline.
For Ironstar, the primary use cases are:

1. **Import resolution**: Resolve `@import` statements for Open Props and component CSS
2. **Autoprefixing**: Add vendor prefixes for browser compatibility
3. **Minification**: Compress CSS for production

```javascript
// postcss.config.js
module.exports = {
  plugins: {
    'postcss-import': {},  // Resolve @import statements
    'autoprefixer': {},    // Add vendor prefixes based on browserslist
    'cssnano': {           // Minify CSS (production only)
      preset: ['default', {
        discardComments: { removeAll: true },
      }],
    },
  },
};
```

**Import resolution:**

PostCSS Import resolves `@import` statements at build time, eliminating runtime HTTP requests:

```css
/* web-components/styles/main.css */
@import "open-props/style";          /* Inline all Open Props tokens */
@import "./theme.css";               /* Application theme overrides */
@import "./components/button.css";   /* Component styles */
```

The bundler produces a single `bundle.css` with all imports inlined.

**Autoprefixing:**

Autoprefixer adds vendor prefixes based on the browserslist configuration:

```json
// package.json
{
  "browserslist": [
    "chrome >= 111",
    "firefox >= 119",
    "safari >= 17"
  ]
}
```

This ensures modern CSS features (container queries, `:has()`, `light-dark()`) work across supported browsers.

**Minification:**

cssnano minifies CSS in production builds:
- Removes comments and whitespace
- Shortens color values (`#ffffff` → `#fff`)
- Merges duplicate rules
- Optimizes `calc()` expressions

**Why not Tailwind JIT:**

Tailwind's JIT compiler requires template scanning to detect which utility classes are used.
This introduces complexity for server-rendered HTML:

1. Backend templates (hypertext) would need to be watched by the frontend build tool
2. Dynamic class names (`class={format!("text-{}", color)}`) break static analysis
3. Two-stage build: scan templates, generate CSS, bundle

Open Props avoids this by using CSS custom properties referenced directly:

```rust
// Tailwind: requires JIT to scan this template
maud! { div class="bg-blue-600 p-4 rounded-lg" { ... } }

// Open Props: CSS variables resolve at runtime, no scanning needed
maud! { div style="background: var(--blue-6); padding: var(--size-4); border-radius: var(--radius-2);" { ... } }
```

The Open Props approach integrates cleanly with server-driven HTML generation.

---

## Build pipeline architecture

The build pipeline transforms source files into production assets through deterministic stages:

```
web-components/
├── index.ts              # Entry point
├── components/
│   ├── vanilla/
│   │   └── vega-chart.ts
│   └── lit/
│       └── ds-echarts.ts
└── styles/
    ├── main.css          # Open Props imports
    ├── theme.css         # Custom theme
    └── components/
        ├── button.css
        └── card.css

                ↓  Rolldown + PostCSS

static/dist/
├── bundle.[hash].js      # Hashed bundle
├── bundle.[hash].css     # Hashed styles
└── manifest.json         # Maps entry → hashed files

                ↓  cargo build --release

target/release/ironstar   # Single binary with embedded static/dist/
```

**Stage 1: TypeScript compilation + bundling**

Rolldown compiles TypeScript to JavaScript and bundles modules:

```typescript
// web-components/index.ts
import './styles/main.css';
import './components/vanilla/vega-chart';
import './components/lit/ds-echarts';

// Component registrations happen at import
// No default export needed
```

Output: `static/dist/bundle.[hash].js`

**Stage 2: CSS processing**

PostCSS transforms CSS imports into a single bundle:

```css
/* Input: web-components/styles/main.css */
@import "open-props/style";
@import "./theme.css";
@import "./components/button.css";

/* Output: static/dist/bundle.[hash].css */
/* All imports inlined, minified, autoprefixed */
```

**Stage 3: Manifest generation**

Rolldown generates a manifest mapping logical names to content-hashed filenames:

```json
{
  "index.ts": {
    "file": "bundle.a1b2c3d4.js",
    "css": ["bundle.x9y8z7w6.css"]
  }
}
```

**Stage 4: Embedding (production)**

In release builds, rust-embed includes `static/dist/` in the binary:

```rust
#[cfg(not(debug_assertions))]
#[derive(RustEmbed)]
#[folder = "static/dist"]
struct Assets;
```

Templates resolve asset URLs via the manifest:

```rust
let manifest = AssetManifest::load();
let js_url = format!("/static/{}", manifest.get("index.ts").unwrap());
```

**Development mode differences:**

| Stage | Development | Production |
|-------|-------------|------------|
| **Bundling** | Watch mode, fast rebuild | Single build, optimized |
| **CSS** | Source maps, unminified | Minified, no source maps |
| **Serving** | ServeDir from filesystem | Embedded in binary |
| **Caching** | `Cache-Control: no-store` | `Cache-Control: max-age=31536000, immutable` |

Development mode prioritizes fast iteration; production mode prioritizes performance.

---

## Integration with process-compose

The build pipeline integrates with process-compose for orchestrated development:

```yaml
# process-compose.yaml
processes:
  frontend-build:
    command: cd web-components && pnpm exec rolldown --watch
    readiness_probe:
      exec:
        command: "test -f static/dist/manifest.json"
      initial_delay_seconds: 2

  backend:
    command: cargo run
    depends_on:
      frontend-build:
        condition: process_healthy
    environment:
      DATABASE_URL: "sqlite:./data/ironstar.db"

  frontend-watch:
    command: cd web-components && pnpm exec rolldown --watch
    depends_on:
      frontend-build:
        condition: process_healthy
```

**Process dependency graph:**

```
frontend-build (initial)
    ↓
backend (starts when manifest.json exists)
    ↓
frontend-watch (continuous rebuild)
```

This ensures:
1. Initial assets built before backend starts
2. Backend serves valid assets from the start
3. Watch mode handles incremental rebuilds

---

## Related documentation

- Design principles: `design-principles.md`
- Frontend stack decisions: `frontend-stack-decisions.md`
- Backend core decisions: `backend-core-decisions.md`
- Infrastructure decisions: `infrastructure-decisions.md`
- CQRS implementation: `cqrs-implementation-decisions.md`
- Frontend build pipeline: `frontend-build-pipeline.md`
- Development workflow: `development-workflow.md`
