import os from "node:os";
import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for EventCatalog E2E testing
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
		// Base URL for page.goto() calls (EventCatalog uses port 3000 for both dev and preview)
		baseURL: process.env.BASE_URL ?? "http://localhost:3000",

		// Collect trace when retrying the failed test
		trace: "on-first-retry",

		// Screenshot on failure
		screenshot: "only-on-failure",

		// Video on failure
		video: "retain-on-failure",
	},

	// Configure projects for major browsers
	// Run only chromium in CI for speed, all browsers locally for comprehensive testing
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
				{
					name: "webkit",
					use: { ...devices["Desktop Safari"] },
				},
			],

	// Serve EventCatalog for e2e tests
	// Uses preview mode (static file server) instead of dev mode because:
	// 1. eventcatalog-build already runs before e2e tests in CI (package-test.yaml)
	// 2. eventcatalog dev has significant startup overhead (file sync, migrations, watcher)
	// 3. preview mode starts in ~1s vs dev mode's 30-60s+ in CI
	webServer: {
		command: "bun run preview",
		url: "http://localhost:3000",
		reuseExistingServer: !process.env.CI,
		timeout: 60000,
	},
});
