# Frontend build pipeline

Ironstar uses Rolldown for JavaScript/CSS bundling, with Open Props design tokens and Open Props UI component library for styling.
This document covers the build configuration, development workflow, browser compatibility requirements, and comparison with northstar's Go-based approach.

## Overview

```
web-components/
├── index.ts              # Main entry point
├── styles/
│   ├── main.css          # Entry point: imports Open Props + theme + components
│   ├── theme.css         # Theme layer: app-specific variables from Open Props tokens
│   └── components/       # Copied Open Props UI component CSS (owned by project)
│       ├── button.css
│       ├── card.css
│       ├── dialog.css
│       └── ...
├── components/           # Vanilla web components (TypeScript/JavaScript)
├── types/                # Generated TypeScript (from ts-rs)
├── rolldown.config.ts    # Bundler configuration
├── postcss.config.js     # PostCSS configuration (simpler than Tailwind)
└── package.json

static/dist/              # Build output
├── bundle.[hash].css     # Compiled CSS
├── bundle.[hash].js      # Compiled JS
└── components.[hash].js  # Web component bundle
```

### Local repository references

- **Open Props**: `~/projects/lakescope-workspace/open-props` - CSS design tokens and custom properties
- **Open Props UI**: `~/projects/lakescope-workspace/open-props-ui` - Pure CSS component library (copy-paste ownership model)

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

/* Import Open Props design tokens */
@import "open-props/style";

/* Or selective imports for smaller bundle size */
/* @import "open-props/colors"; */
/* @import "open-props/sizes"; */
/* @import "open-props/animations"; */
/* @import "open-props/easings"; */
/* @import "open-props/shadows"; */
/* @import "open-props/borders"; */

/* Theme layer - application-specific tokens */
@import "./theme.css";

/* Component styles (copied from Open Props UI, owned by project) */
@import "./components/button.css";
@import "./components/card.css";
@import "./components/dialog.css";
@import "./components/input.css";
/* Add other components as needed */

/* View transitions (optional) */
@view-transition {
  navigation: auto;
}
```

### Theme layer

```css
/* web-components/styles/theme.css */

:root {
  /* Primary color derived from Open Props */
  --primary: var(--blue-7);
  --primary-light: var(--blue-5);
  --primary-dark: var(--blue-9);

  /* Accent colors */
  --accent: var(--orange-6);
  --accent-light: var(--orange-4);
  --accent-dark: var(--orange-8);

  /* Surface colors using light-dark() function */
  --surface-default: light-dark(var(--gray-0), var(--gray-9));
  --surface-elevated: light-dark(var(--gray-1), var(--gray-8));
  --surface-overlay: light-dark(var(--gray-2), var(--gray-7));

  /* Text colors */
  --text-primary: light-dark(var(--gray-9), var(--gray-1));
  --text-secondary: light-dark(var(--gray-7), var(--gray-3));
  --text-tertiary: light-dark(var(--gray-6), var(--gray-4));

  /* Border colors */
  --border-default: light-dark(var(--gray-3), var(--gray-7));
  --border-emphasis: light-dark(var(--gray-4), var(--gray-6));

  /* Shadows using Open Props */
  --shadow-card: var(--shadow-2);
  --shadow-elevated: var(--shadow-4);
  --shadow-dialog: var(--shadow-6);

  /* Border radius */
  --radius-default: var(--radius-2);
  --radius-large: var(--radius-3);
  --radius-full: var(--radius-round);

  /* Animation easings */
  --ease-in-out: var(--ease-3);
  --ease-spring: var(--ease-spring-3);
}

/* Dark mode handled automatically by light-dark() function - no JavaScript needed */
/* Browser respects prefers-color-scheme and/or color-scheme meta tag */
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

## Open Props + Open Props UI architecture

Open Props provides design tokens as CSS custom properties, eliminating the need for JIT compilation or class scanning.
Open Props UI provides pure CSS component styles that you copy into your project for full ownership and customization.

### PostCSS configuration

PostCSS configuration is simpler than Tailwind since there is no JIT compilation or class scanning needed.
However, Open Props and Open Props UI use modern CSS features that require `postcss-preset-env` for proper processing.

```javascript
// web-components/postcss.config.js
export default {
  plugins: {
    'postcss-import': {},           // Handle @import statements
    'postcss-preset-env': {         // Modern CSS features (required for Open Props)
      stage: 2,
      features: {
        'oklab-function': true,     // OKLab/OKLch color spaces
        'light-dark-function': true, // light-dark() for theme switching
        'custom-media-queries': true // Open Props media queries
      }
    },
    'autoprefixer': {},             // Vendor prefixes (minimal with modern CSS)
    'cssnano': {                    // Minification (production only)
      preset: 'default'
    }
  }
};
```

The `postcss-preset-env` plugin is critical for Open Props integration because it:
- Processes `oklch()` and `oklab()` color functions used throughout Open Props
- Handles `light-dark()` function for automatic dark mode support in Open Props UI
- Supports custom media queries defined by Open Props for responsive design

### Component ownership model

Unlike utility-first frameworks, Open Props UI follows a copy-paste ownership model:

1. **Browse components** at Open Props UI repository (`~/projects/lakescope-workspace/open-props-ui`)
2. **Copy CSS** for needed components into `web-components/styles/components/`
3. **Customize freely** - you own the CSS, modify as needed
4. **No npm dependency** - component CSS is not installed via npm

Example component structure:

```css
/* web-components/styles/components/button.css */
/* Copied from Open Props UI and customized for ironstar */

button,
.button {
  /* Base styles using Open Props tokens */
  padding: var(--size-2) var(--size-4);
  border-radius: var(--radius-default);
  font-size: var(--font-size-1);
  font-weight: var(--font-weight-6);

  /* Colors using theme tokens */
  background-color: var(--primary);
  color: light-dark(var(--gray-0), var(--gray-9));

  /* Interaction states */
  transition: background-color 0.2s var(--ease-in-out);

  &:hover {
    background-color: var(--primary-dark);
  }

  &:active {
    transform: scale(0.98);
  }

  /* Variant: secondary */
  &[data-variant="secondary"] {
    background-color: var(--surface-elevated);
    color: var(--text-primary);
    border: 1px solid var(--border-default);
  }
}
```

### Design tokens

Open Props provides comprehensive design tokens across multiple categories:

- **Colors**: Complete color scales in OKLch color space (better perceptual uniformity)
- **Sizes**: Responsive spacing scale from `--size-000` to `--size-15`
- **Typography**: Font sizes, line heights, weights, and tracking
- **Shadows**: Elevation scales from `--shadow-1` to `--shadow-6`
- **Borders**: Border radius values from `--radius-1` to `--radius-round`
- **Easings**: Animation curves from `--ease-1` to `--ease-spring-5`
- **Gradients**: Pre-defined gradient patterns
- **Media queries**: Responsive breakpoints via custom media

---

## CSS cascade layers organization

CSS cascade layers (`@layer`) provide explicit control over style precedence independent of selector specificity or source order.
This is particularly important when integrating Open Props UI components with custom application styles.

### What are CSS cascade layers

The `@layer` at-rule establishes a layered cascade where later layers override earlier layers, regardless of specificity.
This eliminates specificity wars and makes style precedence predictable and declarative.

```css
/* Layer declaration - establishes order */
@layer base, components, utilities;

/* Styles in 'utilities' will override 'components',
   which will override 'base', regardless of selector specificity */

@layer base {
  h1 { font-size: 2rem; }
}

@layer utilities {
  .text-small { font-size: 1rem !important; }  /* Overrides h1.text-small */
}
```

Without layers, the `.text-small` class would need higher specificity or `!important` to override the `h1` selector.
With layers, the layer order determines precedence.

### Open Props UI layer structure

Open Props UI uses a specific layer structure for organizing styles.
Reference implementation at `~/projects/lakescope-workspace/open-props-ui` uses:

```css
@layer openprops, normalize, theme, components.root, components.extended, utils;
```

**Layer precedence** (later layers override earlier ones):
- `openprops`: Base design tokens and custom properties
- `normalize`: CSS reset and normalization
- `theme`: Theme-specific token overrides
- `components.root`: Core component styles
- `components.extended`: Extended component variants
- `utils`: Utility classes (highest precedence)

This means `utils` overrides `components.extended`, which overrides `components.root`, and so on.

### Why this matters for Ironstar

Understanding the layer structure prevents common pitfalls when customizing components:

**Without layer awareness**:
```css
/* This might not work as expected */
.button {
  background: var(--custom-color);  /* May be overridden by component layer */
}
```

**With layer awareness**:
```css
/* Place in higher-precedence layer to ensure override */
@layer app {
  .button {
    background: var(--custom-color);  /* Guaranteed to override */
  }
}
```

Layers eliminate the need for specificity hacks like:
- Nested selectors (`.page .button`)
- ID selectors (`#app .button`)
- `!important` flags
- Inline styles

### Recommended Ironstar layer structure

```css
/* web-components/styles/main.css */

/* Declare layers upfront - establishes precedence */
@layer openprops, normalize, theme, components, utilities, app;

/* Import Open Props design tokens into dedicated layer */
@import "open-props/style" layer(openprops);

/* Normalization layer (if needed) */
@layer normalize {
  *, *::before, *::after {
    box-sizing: border-box;
  }
  body {
    margin: 0;
    line-height: 1.5;
  }
}

/* Theme layer - application-specific token overrides */
@layer theme {
  :root {
    --primary: var(--blue-7);
    --surface-default: light-dark(var(--gray-0), var(--gray-9));
    /* ... theme tokens */
  }
}

/* Components layer - Open Props UI component styles */
@layer components {
  @import "./components/button.css";
  @import "./components/card.css";
  @import "./components/dialog.css";
  /* Each component file can define sublayers if needed */
}

/* Utilities layer - single-purpose utility classes */
@layer utilities {
  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
  }

  .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

/* App layer - application-specific overrides (highest precedence) */
@layer app {
  /* Page-specific customizations */
  .hero-section .button {
    padding: var(--size-3) var(--size-6);
  }
}
```

### Component-specific layer usage

Individual component files can use layers internally:

```css
/* web-components/styles/components/button.css */
@layer components {
  button,
  .button {
    /* Base button styles using Open Props tokens */
    padding: var(--size-2) var(--size-4);
    border-radius: var(--radius-default);
    background-color: var(--primary);
    color: light-dark(var(--gray-0), var(--gray-9));

    /* Variants can be in same layer - specificity still applies within layer */
    &[data-variant="secondary"] {
      background-color: var(--surface-elevated);
      color: var(--text-primary);
      border: 1px solid var(--border-default);
    }

    &[data-size="large"] {
      padding: var(--size-3) var(--size-6);
      font-size: var(--font-size-2);
    }
  }
}
```

### Unlayered styles

Styles not placed in any `@layer` have the highest precedence of all, even higher than the last declared layer.
This is intentional - it allows emergency overrides and inline customizations.

```css
/* These styles override ALL layers, including @layer app */
.emergency-override {
  display: none !important;
}
```

For maintainability, avoid unlayered styles in production code.
All styles should be in explicit layers.

### Layers are optional but recommended

CSS layers are **optional** for ironstar - the stack will work without them.
However, they provide significant benefits:

**Benefits of using layers**:
- Predictable cascade behavior independent of specificity
- Easier component customization without specificity hacks
- Clear separation between base styles, components, and overrides
- Scalable architecture as project grows

**When to skip layers**:
- Very simple projects with minimal CSS
- Need to support older browsers (pre-2022)
- Team unfamiliar with cascade layers

For ironstar, layers are recommended as part of the modern CSS architecture, aligning with the project's preference for modern features over wide compatibility.

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

## Browser compatibility

Open Props and Open Props UI rely on modern CSS features.
Browser support requirements are more stringent than traditional CSS frameworks.

### Required browser versions

- **Chrome/Edge**: 111+ (March 2023)
- **Firefox**: 119+ (October 2023)
- **Safari**: 17+ (September 2023)

### Modern CSS features used

| Feature | Purpose | Fallback strategy |
|---------|---------|-------------------|
| CSS custom properties | Design tokens | Required - no fallback |
| `light-dark()` function | Automatic dark mode | Use `@media (prefers-color-scheme)` |
| OKLch colors | Perceptual uniformity | Use fallback RGB/HSL values |
| `color-mix()` function | Dynamic color blending | Pre-calculate mixed colors |
| Container queries | Responsive components | Use media queries |
| Cascade layers (`@layer`) | Style organization | Not critical - remove if needed |
| CSS nesting | Component styling | PostCSS plugin transforms to flat CSS |

### Graceful degradation patterns

For projects requiring wider browser support, implement fallbacks:

```css
/* Fallback for light-dark() function */
:root {
  --surface-default: var(--gray-0);
}

@media (prefers-color-scheme: dark) {
  :root {
    --surface-default: var(--gray-9);
  }
}

/* Fallback for OKLch colors */
.button {
  background: rgb(59, 130, 246);  /* RGB fallback */
  background: oklch(0.6 0.15 250deg);  /* Modern syntax */
}

/* Fallback for color-mix() */
.hover-state {
  /* Pre-calculated mixed color */
  background: rgb(49, 108, 207);
  /* Modern dynamic mixing */
  background: color-mix(in oklch, var(--primary) 80%, black);
}
```

For ironstar, we target modern browsers exclusively and do not implement fallbacks.

---

## Comparison with northstar

Northstar uses a Go-native toolchain (gotailwind + esbuild) with Tailwind CSS v4 and DaisyUI.
Ironstar uses Rust-native bundling with Open Props and Open Props UI, representing a different philosophical approach.

| Aspect | Northstar | Ironstar |
|--------|-----------|----------|
| CSS approach | Utility-first (Tailwind) | Design tokens (Open Props) |
| CSS tool | gotailwind (Go CLI) | PostCSS (Node) |
| Component library | DaisyUI (generated classes) | Open Props UI (copy-paste CSS) |
| JS bundler | esbuild (Go) | Rolldown (Rust) |
| JIT compilation | Yes (scans for classes) | No (static tokens) |
| Config style | CSS `@plugin` directives | CSS custom properties |
| Dev workflow | Multi-process (air, gotailwind, esbuild) | Single `rolldown --watch` |
| Asset versioning | hashfs (Go library) | Rolldown built-in `[hash]` |
| Hot reload | HTTP ping from esbuild | Rolldown watch + browser reload |
| Component ownership | Generated by framework | Copy-paste into project |
| Browser requirements | Modern (CSS custom props) | Very modern (OKLch, light-dark()) |

### Philosophical differences

**Utility-first vs Design tokens**: Northstar uses Tailwind's utility-first approach where classes are applied directly in templates.
Ironstar uses design token architecture where CSS custom properties define a system and semantic component classes are built on top.

**Generated vs Owned components**: Northstar's DaisyUI generates component classes via plugin system.
Ironstar copies Open Props UI component CSS directly into the project for full ownership and customization.

**Build complexity**: Northstar requires JIT compilation and class scanning across Rust templates.
Ironstar's Open Props tokens are static constants requiring no compilation step beyond standard PostCSS processing.

**Modern CSS adoption**: Both use modern CSS features, but ironstar pushes further with OKLch color space, `light-dark()` function, and container queries, accepting narrower browser support in exchange for simpler architecture and better developer experience.

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
  "dependencies": {
    "open-props": "^2.0.0-beta.5"
  },
  "devDependencies": {
    "rolldown": "^0.x",
    "rolldown-plugin-postcss": "^0.x",
    "postcss": "^8.x",
    "postcss-import": "^16.x",
    "postcss-custom-media": "^10.x",
    "autoprefixer": "^10.x",
    "cssnano": "^6.x",
    "typescript": "^5.x"
  }
}
```

**Note**: `open-props-ui` is NOT an npm dependency.
Component CSS is copied directly into your project from the local repository at `~/projects/lakescope-workspace/open-props-ui`.
This follows the copy-paste ownership model where you own and customize the component styles.

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

## Lit component bundling options

When using Lit components (see Pattern 1.5 in `integration-patterns.md`), ironstar supports two bundling approaches: extending Rolldown configuration for Lit, or using esbuild specifically for Lit components.

### Option A: Rolldown for all assets (recommended for consistency)

Extend the existing Rolldown configuration to handle Lit components with TypeScript decorators:

```typescript
// web-components/rolldown.config.ts
import { defineConfig } from 'rolldown';
import postcss from 'rolldown-plugin-postcss';
import typescript from '@rollup/plugin-typescript';

export default defineConfig({
  input: {
    bundle: 'index.ts',
    components: 'components/index.ts',
    lit: 'lit/index.ts',  // Lit component entry point
  },
  output: {
    dir: '../static/dist',
    format: 'esm',
    entryFileNames: '[name].[hash].js',
    chunkFileNames: '[name].[hash].js',
  },
  plugins: [
    typescript({
      tsconfig: './lit/tsconfig.json',
      compilerOptions: {
        experimentalDecorators: true,     // Required for Lit @customElement
        useDefineForClassFields: false,   // Required for Lit decorator behavior
      },
    }),
    postcss({
      config: './postcss.config.js',
      extract: 'bundle.css',
      minimize: true,
    }),
  ],
});
```

TypeScript configuration for Lit:

```json
// web-components/lit/tsconfig.json
{
  "compilerOptions": {
    "experimentalDecorators": true,
    "useDefineForClassFields": false,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "target": "ES2020",
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["**/*.ts"]
}
```

### Option B: esbuild for Lit components (pragmatic alternative)

esbuild provides the fastest TypeScript compilation with battle-tested Lit support.
This approach uses esbuild for Lit components while maintaining Rolldown for CSS and vanilla web components.

**When to choose esbuild**:
- Fastest possible TypeScript compilation (10-100x faster than tsc)
- Proven pattern from Northstar template
- Zero configuration needed for decorators
- Acceptable to include Go binary in toolchain per project decision

**Reference implementation**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/cmd/web/build/main.go`

```go
// cmd/web/build/main.go (adapted from Northstar pattern)
package main

import (
    "github.com/evanw/esbuild/pkg/api"
)

func build() error {
    opts := api.BuildOptions{
        EntryPointsAdvanced: []api.EntryPoint{
            {
                InputPath:  "web-components/lit/index.ts",
                OutputPath: "libs/lit",
            },
        },
        Bundle:            true,
        Format:            api.FormatESModule,
        MinifyIdentifiers: true,
        MinifySyntax:      true,
        MinifyWhitespace:  true,
        Outdir:            "static/dist",
        Sourcemap:         api.SourceMapLinked,
        Target:            api.ESNext,
        Write:             true,
    }

    result := api.Build(opts)
    return checkBuildErrors(result)
}
```

With process-compose for parallel builds:

```yaml
# process-compose.yaml (extended)
processes:
  lit-components:
    command: go run cmd/web/build/main.go --watch
    availability:
      restart: on_failure

  frontend-assets:
    command: pnpm dev
    working_dir: ./web-components
    availability:
      restart: on_failure

  backend:
    command: cargo watch -x run
    depends_on:
      lit-components:
        condition: process_healthy
      frontend-assets:
        condition: process_healthy
```

### Lit component dependencies

Add to package.json for Lit components:

```json
{
  "dependencies": {
    "lit": "^3.3.1",
    "echarts": "^5.5.0"
  },
  "devDependencies": {
    "typescript": "^5.9.3",
    "@lit/reactive-element": "^2.0.4"
  }
}
```

### Decision matrix

| Criterion | Rolldown | esbuild |
|-----------|----------|---------|
| Build speed | Fast (~1-2s) | Extremely fast (~100-200ms) |
| TypeScript decorators | Plugin required | Built-in |
| Single tool | Yes | No (hybrid with Rolldown for CSS) |
| Rust-native | Yes | No (Go binary) |
| Proven for Lit | Less mature | Battle-tested (Northstar) |
| Development workflow | Single `rolldown --watch` | Requires process coordination |

**Recommendation**: Start with Option B (esbuild) for rapid development, especially if referencing Northstar patterns.
Migrate to Option A (Rolldown) later for tool consolidation if needed.

### Source code references

- **Northstar esbuild config**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/cmd/web/build/main.go`
- **Northstar Lit components**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/libs/lit/`
- **esbuild source**: `~/projects/lakescope-workspace/esbuild/`
- **Lit framework**: `~/projects/lakescope-workspace/lit-web-components/`

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
        button class="button" {
            (Raw::dangerously_create(icons::CAMERA))
            "Upload"
        }
    }
}
```

---

## Key differences from Tailwind pipeline

Understanding the architectural shift from Tailwind v4 + DaisyUI to Open Props + Open Props UI.

| Aspect | Tailwind v4 + DaisyUI | Open Props + Open Props UI |
|--------|----------------------|---------------------------|
| **Class scanning** | Required via `@source` directives | Not needed - tokens are static |
| **JIT compilation** | Yes - generates CSS on-demand | No - imports pre-defined tokens |
| **CSS generation** | Dynamic based on usage | Static imports only |
| **Theme configuration** | `@plugin` directives in CSS | CSS custom properties in `:root` |
| **Component CSS** | Generated by DaisyUI plugin | Copied into project (owned) |
| **PostCSS plugins** | `@tailwindcss/postcss` (complex) | `postcss-import`, `postcss-custom-media` (simple) |
| **npm dependencies** | `tailwindcss`, `@tailwindcss/postcss`, `daisyui` | `open-props`, standard PostCSS plugins |
| **Component updates** | npm package update | Manual copy from reference repo |
| **Customization** | Override via config or CSS layers | Direct CSS editing (full ownership) |
| **Build speed** | Slower (class scanning + JIT) | Faster (no scanning, static imports) |
| **Bundle size** | Smaller (tree-shaken) | Larger (all imported tokens) |
| **Learning curve** | Learn utility classes | Learn CSS custom properties |
| **Browser support** | Modern browsers | Very modern browsers (OKLch, light-dark()) |

### When to use each approach

**Choose Tailwind + DaisyUI if you need**:
- Rapid prototyping with utility classes
- Extensive pre-built component library
- Automatic tree-shaking for minimal bundle size
- Wide ecosystem of plugins and integrations
- Broader browser support

**Choose Open Props + Open Props UI if you need**:
- Full control over component CSS
- Simpler build pipeline without class scanning
- Modern CSS features (OKLch, light-dark(), container queries)
- Design token architecture
- No framework lock-in

Ironstar chooses Open Props because the project values CSS ownership, modern features, and build simplicity over framework convenience and wide browser support.

---

## Troubleshooting

Common frontend build and development issues with diagnostic steps.

### Rolldown build failures

**Missing dependencies**:
- Run `pnpm install` in `web-components/` directory
- Verify `package.json` integrity and lock file consistency
- Check `node_modules/` exists and is not corrupted

**TypeScript errors**:
- Run `pnpm typecheck` to see detailed type errors
- Check `web-components/tsconfig.json` configuration
- Verify generated types in `web-components/types/` are up-to-date (run `cargo test --lib`)

**Invalid PostCSS syntax**:
- Check `postcss.config.js` for correct plugin configuration
- Verify all `@import` statements reference valid paths
- Ensure `postcss-preset-env` is installed for modern CSS features

### Open Props token issues

**Tokens not resolving**:
- Verify `@import "open-props/style"` is present in `web-components/styles/main.css`
- Check `open-props` is installed via `pnpm install`
- Ensure PostCSS processes imports correctly (check build output)

**PostCSS not processing**:
- Verify `postcss-preset-env` plugin is configured in `postcss.config.js`
- Check that `features` option includes `oklab-function`, `light-dark-function`, and `custom-media-queries`
- Run build with `--verbose` flag to see PostCSS processing steps

**OKLch colors not working**:
- Browser version requirements: Chrome 111+, Firefox 119+, Safari 17+ (all from 2023)
- Check browser DevTools console for CSS parsing errors
- Verify `postcss-preset-env` includes `oklab-function: true` feature

### Hot reload not working

**Backend not running**:
- Check `process-compose logs backend` for errors
- Verify backend is listening on expected port
- Ensure `cargo watch` is installed and working

**SSE connection issues**:
- Verify `/hotreload` endpoint responds (check with `curl http://localhost:PORT/hotreload`)
- Check browser DevTools Network tab for SSE connection status
- Ensure no firewall or proxy blocking SSE connections

**Static assets stale**:
- Check that `static/dist/` directory is being watched by Rolldown
- Verify `pnpm dev` is running and outputting rebuild messages
- Clear browser cache or hard refresh (Cmd+Shift+R / Ctrl+Shift+F5)

### TypeScript type generation issues

**Types not updating**:
- Run `cargo test --lib` manually to trigger ts-rs generation
- Check that tests pass (type generation only happens on successful test runs)
- Verify `TS_RS_EXPORT_DIR=web-components/types` environment variable is set

**Path issues**:
- Check `TS_RS_EXPORT_DIR` environment variable in shell or process-compose config
- Verify path is relative to workspace root or absolute path
- Ensure path separators match OS conventions

**Types directory not created**:
- Ensure `web-components/types/` directory exists before running type generation
- Create manually if needed: `mkdir -p web-components/types`
- Check file permissions on `web-components/` directory

---

## Related documentation

- Open Props tokens: see the Open Props section in `architecture-decisions.md`
- Open Props UI components: see the Open Props UI section in `architecture-decisions.md`
- Rolldown bundler: see the Rolldown section in `architecture-decisions.md`
- Lucide icons: see the Lucide section in `architecture-decisions.md`
- Northstar reference (Tailwind approach): `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/`
- Open Props repository: `~/projects/lakescope-workspace/open-props`
- Open Props UI repository: `~/projects/lakescope-workspace/open-props-ui`
