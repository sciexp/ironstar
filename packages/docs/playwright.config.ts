import os from "node:os";
import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for E2E testing
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
	// Test directory
	testDir: "./e2e",

	// Run tests in files in parallel
	fullyParallel: true,

	// Fail the build on CI if you accidentally left test.only in the source code
	forbidOnly: Boolean(process.env.CI),

	// Retry on CI only
	retries: process.env.CI ? 2 : 0,

	// Use all available CPU cores locally, 3 workers in CI for faster execution
	workers: process.env.CI ? 3 : os.cpus().length,

	// Reporter configuration
	reporter: process.env.CI
		? [
				["html", { outputFolder: "playwright-report", open: "never" }],
				["json", { outputFile: "playwright-report/results.json" }],
				["github"],
			]
		: [
				["list"],
				["html", { outputFolder: "playwright-report", open: "never" }],
			],

	// Shared settings for all projects
	use: {
		// Base URL for page.goto() calls
		baseURL: process.env.BASE_URL ?? "http://localhost:4321",

		// Collect trace when retrying the failed test
		trace: "on-first-retry",

		// Screenshot on failure
		screenshot: "only-on-failure",

		// Video on failure
		video: "retain-on-failure",
	},

	// Configure projects for major browsers.
	// CI: chromium only for speed.
	// Local: chromium + firefox. Webkit is omitted because the webkit binary
	// shipped by playwright-web-flake/1.59.1 fails the inspector handshake
	// with @playwright/test 1.59.1 ("Protocol error (Console.enable):
	// 'Console' domain was not found"). Re-enable once the flake's webkit
	// revision aligns with playwright's expected protocol.
	projects: process.env.CI
		? [
				{
					name: "chromium",
					use: { ...devices["Desktop Chrome"] },
				},
			]
		: [
				{
					name: "chromium",
					use: { ...devices["Desktop Chrome"] },
				},
				{
					name: "firefox",
					use: { ...devices["Desktop Firefox"] },
				},
			],

	// Run local dev server before starting tests.
	// In CI: bun run preview:ci → astro preview serves the built worker.
	// In dev: invoke astro dev under node (not bun) because bun's incomplete
	// ws shim causes @cloudflare/vite-plugin's configureServer →
	// startOrUpdateMiniflare to silently stall during dev startup, leaving
	// astro's listen socket unbound.
	webServer: {
		command: process.env.CI
			? "bun run preview:ci"
			: "node ../../node_modules/.bin/astro dev",
		url: "http://localhost:4321",
		reuseExistingServer: !process.env.CI,
		timeout: 120000,
	},
});
