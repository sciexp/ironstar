import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for ironstar E2E testing.
 *
 * The webServer directive compiles and runs the Rust server via `just rust-serve`.
 * The readiness probe at /health/ready gates test execution until the server
 * is fully initialized (database migrations complete, event bus ready).
 *
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
	testDir: "./e2e",

	fullyParallel: true,

	// Fail the build on CI if test.only was left in source
	forbidOnly: Boolean(process.env.CI),

	// Retry on CI only
	retries: process.env.CI ? 2 : 0,

	// Limit workers to avoid overwhelming the debug-mode Rust server with
	// concurrent SSE connections. The server's event delivery latency increases
	// significantly under high parallel load, causing SSE-dependent assertions
	// to timeout. CI uses 2 workers (chromium only); local uses 4.
	workers: process.env.CI ? 2 : 4,

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

	use: {
		baseURL: "http://localhost:3000",

		// Collect trace when retrying the failed test
		trace: "on-first-retry",

		// Screenshot on failure
		screenshot: "only-on-failure",

		// Video on failure
		video: "retain-on-failure",
	},

	// Chromium-only in CI for speed, all browsers locally
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

	// Start the server before tests. In CI, IRONSTAR_BINARY points to the
	// nix-built binary (pre-cached via cachix). Locally, falls back to cargo run.
	// The readiness probe gates test execution until migrations and subsystems are ready.
	webServer: {
		command: process.env.IRONSTAR_BINARY || "just rust-serve",
		url: "http://localhost:3000/health/ready",
		reuseExistingServer: !process.env.CI,
		timeout: 180_000,
	},
});
