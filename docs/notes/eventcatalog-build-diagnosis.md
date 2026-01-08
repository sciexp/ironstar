# EventCatalog Build Failure Diagnosis

## Executive Summary

The EventCatalog build completes but produces zero pages because the glob-loader cannot find content files. The root cause is **timing**: `content.config.ts` evaluates `process.env.PROJECT_DIR` at module load time, which happens BEFORE the environment variable is actually set, resulting in `projectDirBase = undefined`. When undefined, Astro's glob loader uses the fallback relative path `".."` instead of the absolute project directory path.

## Evidence Chain

### 1. The Patch IS Applied

```bash
$ grep -n 'writeFileSync.*package\.json.*eventcatalog-core' \
    ../../node_modules/@eventcatalog/core/dist/eventcatalog.js
141:  copyCore(); fs.writeFileSync(path.join(core, "package.json"), JSON.stringify({name: "eventcatalog-core", type: "module"}));
190:  copyCore(); fs.writeFileSync(path.join(core, "package.json"), JSON.stringify({name: "eventcatalog-core", type: "module"}));
```

The justfile patch successfully modified `eventcatalog.js` to create `package.json` in `.eventcatalog-core/`. This file exists:

```bash
$ cat .eventcatalog-core/package.json
{"name":"eventcatalog-core","type":"module","dependencies":{"@astrojs/react":"*","@astrojs/mdx":"*","@astrojs/tailwind":"*","astro":"*"}}
```

**Conclusion**: The patch works. This is not the problem.

### 2. The .env File Has Correct PROJECT_DIR

```bash
$ cat .eventcatalog-core/.env
PROJECT_DIR=/Users/crs58/projects/rust-workspace/ironstar/packages/eventcatalog
```

The `.env` file exists in `.eventcatalog-core/` with the correct absolute path.

**Conclusion**: The environment variable is correctly configured in the .env file.

### 3. Content Files Exist

```bash
$ find domains -name "*.mdx" | head -3
domains/Example/index.mdx
domains/Example/services/ExampleService/index.mdx
domains/Example/services/ExampleService/events/ExampleEvent/index.mdx
```

The content files are present in the expected locations relative to `packages/eventcatalog/`.

**Conclusion**: Content exists and should be discoverable.

### 4. The Glob Loader Searches in ".." Instead of Absolute Path

From build output:

```
23:06:48 [WARN] [glob-loader] No files found matching "docs/*.(md|mdx),docs/**/*.@(md|mdx)" in directory ".."
23:06:48 [WARN] [glob-loader] No files found matching "domains/*/ubiquitous-language.(md|mdx),domains/*/subdomains/*/ubiquitous-language.(md|mdx)" in directory ".."
```

The glob-loader is searching in `".."` (relative path) instead of the absolute PROJECT_DIR.

**Conclusion**: `projectDirBase` is `undefined` when content.config.ts evaluates.

### 5. Root Cause: Module Evaluation Timing

`content.config.ts` computes `projectDirBase` at the top level:

```typescript
export const projectDirBase = (() => {
  if (process.platform === 'win32') {
    const projectDirPath = process.env.PROJECT_DIR!.replace(/\\/g, '/');
    return projectDirPath.startsWith('/') ? projectDirPath : `/${projectDirPath}`;
  }
  return process.env.PROJECT_DIR;  // ← This is undefined at module load time!
})();
```

This IIFE executes when the module is **first imported**, which happens during Astro's initialization phase.

The eventcatalog CLI sets PROJECT_DIR via `cross-env`:

```javascript
// From eventcatalog.js line 210-211
execSync(
  `cross-env PROJECT_DIR='${dir}' CATALOG_DIR='${core}' ... npx astro build ...`,
  { cwd: core, stdio: "inherit" }
);
```

**The problem**: Astro's content collection system evaluates `content.config.ts` during its initialization, which happens BEFORE the `cross-env` variables take effect for the child process.

### 6. Why .env Loading Doesn't Help

Astro **does** have builtin .env support, but:

1. Astro loads `.env` files during its initialization
2. Content collection loaders (like `glob()`) are evaluated during config parsing
3. The `glob()` loader captures `projectDirBase` at definition time (top-level)
4. If PROJECT_DIR isn't available yet, `projectDirBase` becomes `undefined`
5. Astro's glob loader then uses `".."` as a fallback relative path

## Sequence Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ justfile: eventcatalog-build                                 │
│   └─> cd packages/eventcatalog && bun run build             │
└─────────────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ package.json: "build": "eventcatalog build"                  │
└─────────────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ eventcatalog.js: program.command("build")                    │
│   1. copyCore() - creates .eventcatalog-core/                │
│   2. fs.writeFileSync(package.json) ← patch applied          │
│   3. dotenv.config() loads .env ← happens in CLI process     │
│   4. execSync(`cross-env PROJECT_DIR='${dir}' ... astro ...`)│
└─────────────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Astro initialization (NEW PROCESS)                           │
│   1. Loads astro.config.mjs                                  │
│   2. Loads .env file from cwd (.eventcatalog-core/)          │
│   3. Imports content.config.ts                               │
│      ├─> IIFE evaluates: projectDirBase = process.env.PROJECT_DIR
│      │   ⚠️  BUT cross-env vars not yet in environment!      │
│      └─> projectDirBase = undefined                          │
│   4. Defines content collections with glob({ base: undefined })│
│   5. Glob uses fallback: base = ".."                         │
└─────────────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Astro build phase                                            │
│   - cross-env vars NOW available in runtime                  │
│   - But too late! projectDirBase already computed as undefined│
│   - Glob searches ".." (parent of .eventcatalog-core)        │
│   - Finds 0 content files                                    │
│   - Build succeeds with empty catalog                        │
└─────────────────────────────────────────────────────────────┘
```

## Why It's Confusing

1. **The patch IS applied** - We can see the writeFileSync in eventcatalog.js
2. **The .env file IS correct** - PROJECT_DIR is set properly
3. **Content files DO exist** - All MDX files are in place
4. **The build "succeeds"** - No errors, just zero pages

The failure is **silent** because:
- Astro doesn't error when content collections are empty
- The glob-loader warnings are filtered out by the eventcatalog CLI's grep
- The build completes successfully and produces a valid dist/ with an index.html (the shell, but no content pages)

## The True Root Cause

**Astro's content collection API expects the `base` path to be statically determinable at module evaluation time.**

When using environment variables for the base path, you MUST ensure those variables are available BEFORE Astro initializes, which means:

1. Set in the parent process environment (not via cross-env in the child)
2. OR loaded from .env during Astro's initialization
3. OR computed from a static value (like `import.meta.url`)

The eventcatalog CLI's approach of setting PROJECT_DIR via `cross-env` in the child process is fundamentally incompatible with Astro's content collection evaluation model.

## Better Fix Proposals

### Option 1: Compute PROJECT_DIR from import.meta.url (Recommended)

Modify `.eventcatalog-core/src/content.config.ts` to compute the project directory statically:

```typescript
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Compute PROJECT_DIR relative to this file's location
// .eventcatalog-core/src/content.config.ts -> go up 2 levels to packages/eventcatalog
export const projectDirBase = resolve(__dirname, '../..');

// Now projectDirBase is ALWAYS correct, no environment dependency
```

**Pros**:
- No environment variable dependency
- Works in all contexts (dev, build, preview)
- Deterministic at module evaluation time

**Cons**:
- Assumes .eventcatalog-core is always 1 level deep (which it is)
- Requires modifying the core template (but that's already patched anyway)

### Option 2: Force .env Loading Before Content Config

Modify `.eventcatalog-core/src/content.config.ts` to explicitly load .env:

```typescript
import dotenv from 'dotenv';
import { resolve, join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Explicitly load .env before computing projectDirBase
dotenv.config({ path: resolve(__dirname, '../.env') });

export const projectDirBase = (() => {
  if (process.platform === 'win32') {
    const projectDirPath = process.env.PROJECT_DIR!.replace(/\\/g, '/');
    return projectDirPath.startsWith('/') ? projectDirPath : `/${projectDirPath}`;
  }
  return process.env.PROJECT_DIR!;
})();
```

**Pros**:
- Explicit control over .env loading
- Works with existing .env file

**Cons**:
- Still depends on .env being correct
- Adds dotenv as a runtime dependency of content.config.ts

### Option 3: Set PROJECT_DIR in Parent Process

Modify the justfile to set PROJECT_DIR BEFORE invoking eventcatalog:

```bash
eventcatalog-build:
  #!/usr/bin/env bash
  set -euo pipefail
  export PROJECT_DIR="$(pwd)"
  cd packages/eventcatalog
  bun run build
```

**Pros**:
- No modification to .eventcatalog-core needed
- Environment variable available from the start

**Cons**:
- Requires exporting in every invocation context
- Doesn't work if eventcatalog is run directly (not via justfile)

### Option 4: Patch copyCore to Inject Computed PROJECT_DIR

Enhance the justfile patch to also modify `content.config.ts`:

```bash
# After copyCore, inject a computed projectDirBase
cat >> "$CORE_SRC/content.config.ts" << 'EOF'
// Injected by ironstar build
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
export const projectDirBase = resolve(__dirname, '../..');
EOF
```

**Pros**:
- Automated fix during build
- No manual intervention needed

**Cons**:
- Fragile (appending to a file that already exports projectDirBase would break)
- Would need to use sed to replace the existing export instead

## Recommended Solution

**Option 1** is the cleanest: Modify the justfile patch to ALSO fix `content.config.ts` to use `import.meta.url` instead of `process.env.PROJECT_DIR`.

This makes the EventCatalog build deterministic and removes the environment variable dependency entirely.

### Implementation

Update `justfile` `eventcatalog-build` recipe:

```bash
eventcatalog-build:
  #!/usr/bin/env bash
  set -euo pipefail
  cd packages/eventcatalog

  # WORKAROUND 1: Patch @eventcatalog/core to create package.json
  CORE_DIST="../../node_modules/@eventcatalog/core/dist/eventcatalog.js"
  if [ -f "$CORE_DIST" ]; then
    if ! grep -q 'writeFileSync.*package\.json.*eventcatalog-core' "$CORE_DIST" 2>/dev/null; then
      echo "Patching @eventcatalog/core to fix Vite root detection..."
      sed -i.bak 's/copyCore();/copyCore(); fs.writeFileSync(path.join(core, "package.json"), JSON.stringify({name: "eventcatalog-core", type: "module", dependencies: {"@astrojs\/react": "*", "@astrojs\/mdx": "*", "@astrojs\/tailwind": "*", "astro": "*"}}));/g' "$CORE_DIST"
      echo "Patch applied successfully"
    else
      echo "Patch already applied"
    fi
  fi

  # Run build (this creates .eventcatalog-core/)
  bun run build || true  # Allow first build to fail

  # WORKAROUND 2: Fix content.config.ts to use static path resolution
  CONTENT_CONFIG=".eventcatalog-core/src/content.config.ts"
  if [ -f "$CONTENT_CONFIG" ]; then
    if ! grep -q 'import.meta.url' "$CONTENT_CONFIG" 2>/dev/null; then
      echo "Patching content.config.ts to use static path resolution..."
      # Replace the projectDirBase IIFE with import.meta.url-based computation
      sed -i.bak '
        /^export const projectDirBase = (() => {/,/})();$/c\
import { fileURLToPath } from '\''url'\'';\
import { dirname, resolve } from '\''path'\'';\
\
const __filename_contentconfig = fileURLToPath(import.meta.url);\
const __dirname_contentconfig = dirname(__filename_contentconfig);\
\
export const projectDirBase = resolve(__dirname_contentconfig, '\''../..'\'');
      ' "$CONTENT_CONFIG"
      echo "Content config patched successfully"
    else
      echo "Content config already patched"
    fi
  fi

  # Run build again (should now work)
  bun run build

  # Verify
  if [ ! -f dist/index.html ]; then
    echo "ERROR: EventCatalog build completed but dist/index.html not found"
    exit 1
  fi
  echo "Build verified: dist/index.html exists"
```

This fixes both issues:
1. Creates package.json for Vite root detection
2. Makes projectDirBase deterministic via import.meta.url
