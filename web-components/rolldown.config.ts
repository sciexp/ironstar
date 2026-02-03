// web-components/rolldown.config.ts
import { createRequire } from "node:module";
import { defineConfig } from "rolldown";

// CJS interop for rollup plugin
const require = createRequire(import.meta.url);
const outputManifest = require("rollup-plugin-output-manifest").default;

// Custom manifest generator that includes CSS assets.
// The default generator skips assets without a `name` property,
// but CSS extracted from JS entry points only have `fileName`.
// We derive the logical name from the filename pattern: [name].[hash].[ext]
interface Bundle {
	name?: string;
	fileName: string;
	type?: string;
}

type KeyValueDecorator = (
	k: string,
	v: string,
	opt: unknown,
) => Record<string, string>;

function generateWithCss(
	keyValueDecorator: KeyValueDecorator,
	seed: object,
	opt: unknown,
) {
	return (chunks: Bundle[]) =>
		chunks.reduce(
			(manifest, { name, fileName }) => {
				// Use explicit name if available (JS entries)
				if (name) {
					return {
						...manifest,
						...keyValueDecorator(name, fileName, opt),
					};
				}
				// For CSS assets, derive logical name from filename pattern [name].[hash].css
				// Pass base name only; keyValueDecorator adds extension via nameWithExt option
				if (fileName.endsWith(".css")) {
					const parts = fileName.split(".");
					if (parts.length >= 3) {
						const baseName = parts[0];
						return {
							...manifest,
							...keyValueDecorator(baseName, fileName, opt),
						};
					}
				}
				return manifest;
			},
			seed as Record<string, string>,
		);
}

export default defineConfig({
	input: {
		bundle: "./index.ts",
		datastar: "./datastar.ts",
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
	// Lit web components require legacy (experimental) TypeScript decorators
	// for @customElement and @property. Rolldown auto-detects this from
	// tsconfig.json experimentalDecorators, but we declare it explicitly
	// to make the Lit bundling requirement visible in the build config.
	transform: {
		decorator: {
			legacy: true,
		},
	},
	// Suppress expected warning about transform.decorator.legacy overriding
	// tsconfig.json experimentalDecorators (both are intentionally set).
	checks: {
		configurationFieldConflict: false,
	},
	plugins: [
		// Manifest generation for server-side asset lookup
		outputManifest({
			fileName: "manifest.json",
			generate: generateWithCss,
		}),
	],
});
