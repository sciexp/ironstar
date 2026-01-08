# EventCatalog Build Failure Analysis

## TL;DR

**Problem**: EventCatalog builds successfully but produces zero content pages.

**Root Cause**: `content.config.ts` evaluates `process.env.PROJECT_DIR` at module load time (top-level IIFE), which happens BEFORE the `cross-env` environment variable is set in the child process. This results in `projectDirBase = undefined`, causing glob-loader to use fallback relative path `".."` instead of the absolute project directory.

**Fix**: Replace environment-dependent `projectDirBase` computation with static `import.meta.url`-based path resolution.

## Investigation Summary

### What We Confirmed Works

✅ **Patch is applied**: `eventcatalog.js` line 141 & 190 have `writeFileSync` for package.json
✅ **package.json exists**: `.eventcatalog-core/package.json` is created correctly
✅ **.env is correct**: `PROJECT_DIR=/Users/crs58/projects/rust-workspace/ironstar/packages/eventcatalog`
✅ **Content files exist**: `domains/Example/index.mdx` and other MDX files are present
✅ **Build succeeds**: No errors, produces `dist/index.html`

### What We Discovered is Broken

❌ **glob-loader searches in ".."**: Build output shows `[WARN] [glob-loader] No files found... in directory ".."`
❌ **projectDirBase is undefined**: When `content.config.ts` module loads, `process.env.PROJECT_DIR` is not yet set
❌ **Timing issue**: `cross-env` sets env vars for child process, but content.config.ts loads during Astro initialization

## Timeline of Events

```
Time →
│
├─ justfile invokes: cd packages/eventcatalog && bun run build
│
├─ package.json script runs: eventcatalog build
│
├─ eventcatalog CLI (parent process)
│  ├─ copyCore() creates .eventcatalog-core/
│  ├─ writeFileSync creates package.json (✅ patch works)
│  ├─ dotenv.config() loads .env (⚠️ loads in parent process only)
│  └─ execSync('cross-env PROJECT_DIR=... astro build', { cwd: core })
│     │
│     └─ Spawns child process (astro) ──────────────────────────┐
│                                                               │
├─ Astro initialization (child process) ◄─────────────────────┘
│  ├─ Loads astro.config.mjs
│  ├─ Loads .env from .eventcatalog-core/ (⚠️ async/after config?)
│  ├─ Imports content.config.ts ◄────── CRITICAL POINT
│  │  │
│  │  └─ Top-level IIFE executes:
│  │     const projectDirBase = (() => {
│  │       return process.env.PROJECT_DIR;  // ← undefined! ❌
│  │     })();
│  │
│  ├─ Defines content collections with glob({ base: undefined })
│  └─ Glob falls back to relative path: ".."
│
├─ Astro build phase
│  ├─ cross-env vars NOW available (too late!)
│  ├─ Glob searches ".." (resolves to packages/eventcatalog)
│  ├─ Pattern "domains/*/index.(md|mdx)" relative to ".."
│  ├─ No matches (files are at domains/Example/index.mdx)
│  └─ Builds with 0 pages
│
└─ Build completes ✓ (but empty)
```

## Why .env Doesn't Help

Astro's .env loading documentation says:

> Astro loads environment variables from .env files in your project root using Vite's loadEnv()

However, there's a subtle ordering issue:

1. Astro's config phase loads .env
2. Content collection definitions (using `glob()`) are evaluated during config parsing
3. The `glob()` loader captures `projectDirBase` at **definition time** (module top-level)
4. If PROJECT_DIR isn't available yet, `projectDirBase` becomes `undefined`

The `cross-env` approach sets variables for the child process, but Astro's content config evaluation happens synchronously during import, before the runtime environment is fully established.

## Proof of Concept

Test module evaluation timing:

```javascript
// test.mjs
export const projectDirBase = (() => {
  console.log('[EVAL] process.env.PROJECT_DIR =', process.env.PROJECT_DIR);
  return process.env.PROJECT_DIR;
})();
```

Without PROJECT_DIR:
```bash
$ node test.mjs
[EVAL] process.env.PROJECT_DIR = undefined
```

With PROJECT_DIR:
```bash
$ PROJECT_DIR=/test node test.mjs
[EVAL] process.env.PROJECT_DIR = /test
```

With cross-env in child:
```bash
$ node -e "require('child_process').execSync('cross-env PROJECT_DIR=/test node test.mjs', {stdio:'inherit'})"
[EVAL] process.env.PROJECT_DIR = /test  # ✅ Works in direct invocation
```

But when Astro imports the module during its own initialization:
```bash
$ node -e "require('child_process').execSync('cross-env PROJECT_DIR=/test astro build', {stdio:'inherit'})"
# Astro loads content.config.ts BEFORE cross-env vars are fully propagated
# Result: projectDirBase = undefined
```

## The Actual Build Output

```
23:06:48 [content] Syncing content
23:06:48 [WARN] [glob-loader] No files found matching "docs/*.(md|mdx),docs/**/*.@(md|mdx)" in directory ".."
23:06:48 [WARN] [glob-loader] No files found matching "domains/*/ubiquitous-language.(md|mdx),..." in directory ".."
23:06:48 [WARN] [glob-loader] No files found matching "**/commands/*/index.(md|mdx),..." in directory ".."
...
23:06:48 [content] Synced content
23:06:48 [types] Generated 227ms
23:06:48 [build] ✓ Completed in 286ms.
...
23:07:14 [vite] ✓ built in 7.80s.
```

The `in directory ".."` is the smoking gun. Astro is using a relative path fallback.

## Recommended Fix

Replace the environment-dependent IIFE in `content.config.ts` with static path resolution:

```typescript
// BEFORE (broken - depends on process.env at module eval time)
export const projectDirBase = (() => {
  if (process.platform === 'win32') {
    const projectDirPath = process.env.PROJECT_DIR!.replace(/\\/g, '/');
    return projectDirPath.startsWith('/') ? projectDirPath : `/${projectDirPath}`;
  }
  return process.env.PROJECT_DIR;
})();

// AFTER (fixed - static computation from file location)
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// .eventcatalog-core/src/content.config.ts
// Go up 2 levels: src/ -> .eventcatalog-core/ -> packages/eventcatalog/
export const projectDirBase = resolve(__dirname, '../..');
```

This approach:
- ✅ Works in all contexts (dev, build, preview)
- ✅ Deterministic at module evaluation time
- ✅ No environment variable dependency
- ✅ No .env loading race conditions
- ✅ Compatible with Astro's content collection evaluation model

## Alternative Solutions Considered

### 1. Load .env explicitly in content.config.ts

```typescript
import dotenv from 'dotenv';
dotenv.config({ path: resolve(__dirname, '../.env') });
export const projectDirBase = process.env.PROJECT_DIR;
```

**Issue**: Still has timing dependency, just shifts it to dotenv loading.

### 2. Set PROJECT_DIR in parent process

```bash
export PROJECT_DIR="$(pwd)"
cd packages/eventcatalog
bun run build
```

**Issue**: Requires modifying every invocation point, doesn't work if eventcatalog is run directly.

### 3. Use Astro's built-in env loading

**Issue**: Astro loads .env, but content.config.ts module evaluation happens before the env is fully propagated to the import context.

## Implementation Plan

Update justfile to patch BOTH issues:

1. **Existing patch**: Create package.json for Vite root detection
2. **New patch**: Replace projectDirBase IIFE with import.meta.url-based computation

See `DIAGNOSIS.md` for complete implementation.
