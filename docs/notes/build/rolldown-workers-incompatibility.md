---
title: Rolldown-Vite Integration (Currently Disabled)
---

## Status

**Disabled**: Rolldown-vite is currently disabled due to runtime incompatibility with Cloudflare Workers.

**Last Tested**: October 5, 2025
**Rolldown-Vite Version**: 7.1.15
**Astro Version**: 5.14.1

## Issue Details

### Problem

Rolldown's bundler generates runtime code that uses Node.js's `createRequire(import.meta.url)`, which fails in Cloudflare Workers because `import.meta.url` is undefined in that execution context.

### Error

```
✘ [ERROR] service core:user:ironstar: Uncaught TypeError:
The argument 'path' must be a file URL object, a file URL string, or an absolute path string.
Received 'undefined'

at null.<anonymous> (node:module:34:15) in createRequire
at null.<anonymous> (index.js:1203:33) in dist/_worker.js/chunks/rolldown-runtime_wKfdzHzU.mjs
```

### Investigation Results

**GitHub Issues & PRs**:

- Issue: <https://github.com/cloudflare/workers-sdk/issues/9415> (Closed)
- Fix Merged: <https://github.com/cloudflare/workers-sdk/pull/9891> (July 18, 2025)

**Root Cause**:

- The fix in PR #9891 sets `rollupOptions.platform: "neutral"` to prevent rolldown's Node.js polyfills
- **Critical limitation**: The fix only applies to `@cloudflare/vite-plugin`, not `@astrojs/cloudflare`
- We use `@astrojs/cloudflare` adapter which internally manages Vite configuration
- No direct way to apply the platform: "neutral" fix through Astro's adapter layer

**Attempted Workarounds** (all unsuccessful):

1. Setting `vite.build.rollupOptions.platform: "neutral"` in astro.config.mjs
2. Setting `vite.ssr.build.rollupOptions.platform: "neutral"`
3. Setting `vite.optimizeDeps.esbuildOptions.platform: "neutral"`

None of these approaches prevented the `createRequire` code from appearing in the worker bundle.

## Re-enabling Instructions

When rolldown compatibility is resolved with Astro + Cloudflare, follow these steps:

### Step 1: Update package.json

Add to `devDependencies`:

```json
"vite": "npm:rolldown-vite@latest"
```

Add new `overrides` section at root level:

```json
"overrides": {
  "vite": "npm:rolldown-vite@latest"
}
```

### Step 2: Update astro.config.mjs

1. Uncomment the vite import at the top:

   ```javascript
   import * as vite from "vite";
   ```

2. Uncomment the vite configuration block (see inline comments in file)

### Step 3: Install Dependencies

```bash
bun install
```

### Step 4: Test

```bash
# Test build
bun run build

# Test Cloudflare Workers preview
bun run preview
```

**Success Criteria**:

- Build completes without errors
- No `createRequire` in `dist/_worker.js/chunks/rolldown-runtime_*.mjs`
- Wrangler dev starts successfully
- Site loads at <http://localhost:8787>

## Alternative Paths Forward

### Option A: Wait for Official Support

- Monitor Astro + Rolldown roadmap: <https://github.com/rolldown/rolldown/discussions/153>
- Currently "on hold" for Astro support
- Subscribe to: <https://github.com/withastro/adapters/issues>

### Option B: Migrate to @cloudflare/vite-plugin

If Astro adds support for using `@cloudflare/vite-plugin` directly:

**Pros**:

- Direct access to rolldown compatibility fixes
- Native Workers runtime in dev server
- Official Cloudflare support

**Cons**:

- Would lose Astro-specific adapter features
- Significant configuration changes required
- Unknown SSR parity with `@astrojs/cloudflare`

**Resources**:

- <https://developers.cloudflare.com/workers/vite-plugin/>
- <https://blog.cloudflare.com/introducing-the-cloudflare-vite-plugin/>

## Performance Comparison

**Standard Vite** (current):

- Server build: ~900ms
- Client build: ~34ms
- Total: ~1.6s

**Rolldown-Vite** (when tested):

- Server build: ~890ms (similar)
- Client build: ~33ms (similar)
- Total: ~1.6s

**Note**: Performance gains with Rolldown are more significant on larger projects.
Reference projects report 2-16x improvements.

## Dependencies

The `@astrojs/cloudflare` adapter currently depends on:

```json
{
  "@cloudflare/workers-types": "^4.20250109.0",
  "esbuild": "^0.24.0",
  "miniflare": "^3.20241230.1",
  "vite": "^6.0.7",
  "wrangler": "^3.101.0"
}
```

Any rolldown integration must maintain compatibility with this stack.

## See Also

- Biome migration: Successfully completed, all linting/formatting now uses Biome 2.2.4
- Cloudflare deployment: Working with standard Vite
- Pre-commit hooks: Configured with Biome + nixfmt-rfc-style
