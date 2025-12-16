# Frontend build pipeline

Ironstar uses Rolldown for JavaScript/CSS bundling, with Tailwind CSS v4 and DaisyUI for styling.
This document covers the build configuration, development workflow, and comparison with northstar's Go-based approach.

## Overview

```
web-components/
├── index.ts              # Main entry point
├── styles/
│   └── main.css          # Tailwind entry (@import "tailwindcss")
├── components/           # Vanilla web components
├── types/                # Generated TypeScript (from ts-rs)
├── rolldown.config.ts    # Bundler configuration
├── postcss.config.js     # PostCSS/Tailwind configuration
├── tailwind.config.js    # Tailwind customization
└── package.json

static/dist/              # Build output
├── bundle.[hash].css     # Compiled CSS
├── bundle.[hash].js      # Compiled JS
└── components.[hash].js  # Web component bundle
```

---

## Rolldown configuration

Rolldown is a Rust-native bundler with Rollup-compatible API.
It provides deterministic builds with content-based hashing.

### Basic configuration

```typescript
// web-components/rolldown.config.ts
import { defineConfig } from 'rolldown';
import postcss from 'rolldown-plugin-postcss';

export default defineConfig({
  input: {
    bundle: 'index.ts',
    components: 'components/index.ts',
  },
  output: {
    dir: '../static/dist',
    format: 'esm',
    // Content-based hashing for cache busting
    entryFileNames: '[name].[hash].js',
    chunkFileNames: '[name].[hash].js',
    assetFileNames: '[name].[hash][extname]',
  },
  plugins: [
    postcss({
      config: './postcss.config.js',
      extract: 'bundle.css',
      minimize: true,
    }),
  ],
});
```

### CSS entry point

```css
/* web-components/styles/main.css */
@import "tailwindcss";

/* DaisyUI theme customization */
@plugin "daisyui" {
  themes: light --default, dark --prefersdark;
}

/* Custom theme (optional) */
@plugin "daisyui/theme" {
  name: "ironstar";
  --color-base-100: "#0b1325";
  --color-primary: "#c9a75f";
  /* ... other colors */
}

/* View transitions (optional) */
@view-transition {
  navigation: auto;
}

/* Source paths for Tailwind class scanning */
@source "../**/*.ts";
@source "../../src/**/*.rs";
```

### TypeScript entry

```typescript
// web-components/index.ts

// Import CSS (processed by PostCSS plugin)
import './styles/main.css';

// Web components auto-register on import
import './components/vega-chart';
import './components/sortable-list';

// Signal type exports for type checking
export type * from './types/TodoSignals';
```

---

## Tailwind CSS v4 with DaisyUI

Tailwind v4 uses CSS-native configuration via `@import` and `@plugin` directives.

### PostCSS configuration

```javascript
// web-components/postcss.config.js
export default {
  plugins: {
    '@tailwindcss/postcss': {},
  },
};
```

### Tailwind configuration (optional overrides)

```javascript
// web-components/tailwind.config.js
export default {
  content: [
    './index.ts',
    './components/**/*.ts',
    '../src/**/*.rs',  // Scan Rust templates for classes
  ],
  theme: {
    extend: {
      // Custom extensions
    },
  },
};
```

### DaisyUI themes

DaisyUI provides 35+ themes.
Configure via the `@plugin` directive in CSS:

```css
/* Use built-in themes */
@plugin "daisyui" {
  themes: light --default, dark --prefersdark, cyberpunk, dracula;
}

/* Or define custom theme */
@plugin "daisyui/theme" {
  name: "ironstar";
  --color-base-100: "#ffffff";
  --color-base-200: "#f5f5f5";
  --color-primary: "#3b82f6";
  --color-secondary: "#6366f1";
  --color-accent: "#f59e0b";
  --color-neutral: "#374151";
  --color-info: "#0ea5e9";
  --color-success: "#22c55e";
  --color-warning: "#eab308";
  --color-error: "#ef4444";
}
```

---

## Development vs production

### Development mode

```bash
# Watch mode with hot reload
cd web-components && pnpm dev
```

Development features:
- Source maps enabled
- No minification
- Fast incremental rebuilds
- Watch mode for file changes

### Production mode

```bash
# Optimized production build
cd web-components && pnpm build
```

Production features:
- Minification enabled
- Tree shaking (unused code removed)
- Content-based hashing for cache busting
- Brotli-compatible output (serve with compression)

---

## Asset serving patterns

### Development (direct file serving)

```rust
// In development, serve files directly with no-cache headers
use tower_http::services::ServeDir;

let static_service = ServeDir::new("static/dist")
    .append_index_html_on_directories(false);

let app = Router::new()
    .nest_service("/static", static_service)
    .layer(SetResponseHeaderLayer::if_not_present(
        CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    ));
```

### Production (hash-based versioning)

In production, use content-based hashing for long-lived caching:

```rust
// Asset manifest for hash lookups
use std::collections::HashMap;

pub struct AssetManifest {
    entries: HashMap<String, String>,  // "bundle.js" -> "bundle.a1b2c3.js"
}

impl AssetManifest {
    pub fn load() -> Self {
        // Load from rolldown's manifest.json
        let manifest = include_str!("../../static/dist/manifest.json");
        // Parse and build lookup table
        // ...
    }

    pub fn path(&self, name: &str) -> &str {
        self.entries.get(name).map(|s| s.as_str()).unwrap_or(name)
    }
}

// In templates
fn page(manifest: &AssetManifest) -> impl Renderable {
    maud! {
        link rel="stylesheet" href=(format!("/static/{}", manifest.path("bundle.css")));
        script type="module" src=(format!("/static/{}", manifest.path("bundle.js")));
    }
}
```

With hash-based versioning, serve with long-lived cache headers:

```rust
let app = Router::new()
    .nest_service("/static", ServeDir::new("static/dist"))
    .layer(SetResponseHeaderLayer::overriding(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    ));
```

---

## Comparison with northstar

Northstar uses a Go-native toolchain (gotailwind + esbuild).
Ironstar prefers Rust-native tools where possible.

| Aspect | Northstar | Ironstar |
|--------|-----------|----------|
| CSS tool | gotailwind (Go CLI) | PostCSS + Tailwind (Node) |
| JS bundler | esbuild (Go) | Rolldown (Rust) |
| DaisyUI | Prebuilt plugins (committed) | npm package |
| Config style | CSS `@plugin` directives | Same |
| Dev workflow | Multi-process (air, gotailwind, esbuild) | Single `rolldown --watch` |
| Asset versioning | hashfs (Go library) | Rolldown built-in `[hash]` |
| Hot reload | HTTP ping from esbuild | Rolldown watch + browser reload |

### Why the differences?

**Rolldown over esbuild**: Rust-native aligns with the stack philosophy, and Rolldown provides Rollup-compatible plugin API for better ecosystem support.

**PostCSS over gotailwind**: PostCSS is more widely supported and allows standard Tailwind plugins.
gotailwind is a Go port that may lag behind upstream Tailwind features.

**npm DaisyUI over prebuilt plugins**: npm package receives updates automatically and allows standard configuration patterns.
Northstar's prebuilt plugins are a workaround for avoiding Node.js in Go projects.

---

## Build integration

### Justfile tasks

```justfile
# Frontend development
dev-frontend:
    cd web-components && pnpm dev

# Frontend production build
build-frontend:
    cd web-components && pnpm build

# Generate TypeScript types from Rust
gen-types:
    TS_RS_EXPORT_DIR=web-components/types cargo test --lib

# Full development workflow
dev: gen-types
    process-compose up

# Full production build
build: gen-types build-frontend
    cargo build --release
```

### Package.json scripts

```json
{
  "name": "ironstar-web",
  "type": "module",
  "scripts": {
    "dev": "rolldown --watch",
    "build": "rolldown",
    "typecheck": "tsc --noEmit"
  },
  "devDependencies": {
    "rolldown": "^0.x",
    "rolldown-plugin-postcss": "^0.x",
    "@tailwindcss/postcss": "^4.x",
    "daisyui": "^5.x",
    "typescript": "^5.x"
  }
}
```

### Process-compose integration

```yaml
# process-compose.yaml
version: "0.6"

processes:
  frontend:
    command: pnpm dev
    working_dir: ./web-components
    availability:
      restart: on_failure

  backend:
    command: cargo watch -x run
    depends_on:
      frontend:
        condition: process_healthy
    environment:
      RUST_LOG: debug

  types:
    command: cargo watch -s "cargo test --lib"
    availability:
      restart: on_failure
```

---

## Lucide icons integration

Lucide icons are inlined at build time, not loaded at runtime.

### Icon extraction script

```typescript
// web-components/scripts/extract-icons.ts
import { writeFileSync } from 'fs';
import * as icons from 'lucide-static';

const usedIcons = ['Camera', 'Settings', 'User', 'Trash2', 'Edit'];

const output = usedIcons.map(name => {
  const svg = icons[name as keyof typeof icons];
  return `pub const ${name.toUpperCase()}: &str = r#"${svg}"#;`;
}).join('\n');

writeFileSync('../src/presentation/icons.rs', `// Auto-generated\n${output}`);
```

### Usage in Rust templates

```rust
// src/presentation/icons.rs (generated)
pub const CAMERA: &str = r#"<svg>...</svg>"#;
pub const SETTINGS: &str = r#"<svg>...</svg>"#;

// In templates
use crate::presentation::icons;

fn button_with_icon() -> impl Renderable {
    maud! {
        button class="btn" {
            (PreEscaped(icons::CAMERA))
            "Upload"
        }
    }
}
```

---

## Related documentation

- Rolldown component selection: `docs/notes/architecture/stack-component-selection.md` (section 12)
- DaisyUI patterns: `docs/notes/architecture/stack-component-selection.md` (section 10)
- Lucide integration: `docs/notes/architecture/stack-component-selection.md` (section 13)
- Northstar reference: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/`
