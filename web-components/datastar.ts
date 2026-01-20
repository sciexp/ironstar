// web-components/datastar.ts
// Datastar entry point for separate bundling with content-hashing.
// This creates datastar.js as its own entry in the manifest,
// allowing proper cache invalidation independent of other bundles.
//
// We import from dist/bundles/datastar.js which includes:
// - Full engine with all plugins (GET, POST, Bind, On, Show, etc.)
// - Auto-initialization via load() and apply()

import "@lufrai/datastar/bundles/datastar";
