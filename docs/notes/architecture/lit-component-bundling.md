# Lit component bundling

Lit components require special bundling considerations due to TypeScript decorator support, Light DOM rendering for Open Props token inheritance, and tree-shaking strategies for large dependencies like ECharts.
This document covers bundler options, configuration requirements, and build integration patterns.
For general frontend build configuration, see `frontend-build-pipeline.md`.

## Bundling options

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

**Recommendation**: Rolldown (Rust-native) aligns with ironstar's tooling philosophy and should be preferred for greenfield development.
esbuild is pragmatic only when porting directly from Northstar patterns or when TypeScript decorator compilation speed becomes a critical bottleneck.
For new ironstar projects, prefer Rolldown to maintain a unified Rust-native toolchain.

### Source code references

- **Northstar esbuild config**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/cmd/web/build/main.go`
- **Northstar Lit components**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/libs/lit/`
- **esbuild source**: `~/projects/lakescope-workspace/esbuild/`
- **Lit framework**: `~/projects/lakescope-workspace/lit-web-components/`

---

## TypeScript configuration for Lit

Lit components require decorator support with specific compiler options:

```json
// tsconfig.json
{
  "compilerOptions": {
    "experimentalDecorators": true,
    "useDefineForClassFields": false,
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler"
  }
}
```

The `useDefineForClassFields: false` setting is critical for Lit's `@property` decorators to work correctly.
Without this, class fields are initialized before decorators run, breaking Lit's reactive property system.

---

## Light DOM requirement

All Lit components using Open Props tokens must render to Light DOM instead of Shadow DOM.
See `css-architecture.md` for complete explanation of why this is required and the architectural implications.

```typescript
// In your Lit component
protected createRenderRoot() {
  return this  // Light DOM, not Shadow DOM
}
```

**Why Light DOM is required**:

Shadow DOM encapsulation prevents CSS custom properties from inheriting from the document's `:root`.
Open Props design tokens are defined globally as CSS custom properties, and Light DOM is the only rendering mode that allows components to inherit these tokens.

This architectural constraint has implications:
- **No style encapsulation**: Component styles are scoped via BEM-style class names, not Shadow DOM boundaries
- **Global token inheritance**: Components automatically inherit `:root` level Open Props tokens
- **Simpler debugging**: Component DOM is visible in DevTools without shadow root barriers

For components that require style encapsulation (rare in server-first hypermedia architecture), consider using vanilla web components with manual CSS variable forwarding or accepting reduced Open Props integration.

---

## Tree-shaking ECharts

When bundling ds-echarts or custom ECharts components, tree-shake by importing only needed chart types:

```typescript
// Instead of: import * as echarts from 'echarts'
import * as echarts from 'echarts/core';
import { BarChart, LineChart } from 'echarts/charts';
import { GridComponent, TooltipComponent } from 'echarts/components';
import { SVGRenderer } from 'echarts/renderers';

echarts.use([BarChart, LineChart, GridComponent, TooltipComponent, SVGRenderer]);
```

This reduces bundle size from ~800KB to ~200-300KB depending on chart types used.
The SVGRenderer is recommended over CanvasRenderer for better accessibility and DOM integration.

---

## Related documentation

- Frontend build pipeline: `frontend-build-pipeline.md` (Rolldown configuration, PostCSS, development workflow)
- CSS architecture: `css-architecture.md` (Light DOM requirement details, Open Props token inheritance)
- Integration patterns: `integration-patterns.md` (Pattern 1.5: Lit components for complex state)
- ds-echarts integration guide: `ds-echarts-integration-guide.md` (Complete ECharts Lit component implementation)
- Northstar reference: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/` (esbuild + Lit pattern)
