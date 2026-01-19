// web-components/rolldown.config.ts
import { defineConfig } from "rolldown";
import outputManifest from "rollup-plugin-output-manifest";

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
