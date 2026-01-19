// web-components/rolldown.config.ts
import { createRequire } from "node:module";
import { defineConfig } from "rolldown";

// CJS interop for rollup plugin
const require = createRequire(import.meta.url);
const outputManifest = require("rollup-plugin-output-manifest").default;

export default defineConfig({
	input: {
		bundle: "./index.ts",
	},
	output: {
		dir: "../static/dist",
		format: "esm",
		// Content-based hashing for cache busting
		entryFileNames: "[name].[hash].js",
		chunkFileNames: "[name].[hash].js",
		// Rolldown uses cssEntryFileNames for CSS output
		cssEntryFileNames: "[name].[hash].css",
		sourcemap: process.env.NODE_ENV !== "production",
	},
	plugins: [
		// Manifest generation for server-side asset lookup
		outputManifest({
			fileName: "manifest.json",
		}),
	],
});
