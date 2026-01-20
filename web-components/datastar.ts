// web-components/datastar.ts
// Datastar entry point for separate bundling with content-hashing.
// This creates datastar.js as its own entry in the manifest,
// allowing proper cache invalidation independent of other bundles.
//
// Vendored from official jsDelivr CDN distribution.
// Update via: just download-datastar
// Version tracked in justfile variable: datastar-version
//
// Includes full engine with all plugins (GET, POST, Bind, On, Show, etc.)
// and auto-initialization via load() and apply().

import "./vendor/datastar.js";
