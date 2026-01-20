// web-components/datastar.ts
// Datastar entry point for separate bundling with content-hashing.
// This creates datastar.js as its own entry in the manifest,
// allowing proper cache invalidation independent of other bundles.

import "@lufrai/datastar";
