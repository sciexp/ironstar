import { test, expect } from "@playwright/test";

test.describe("Health endpoints", () => {
	test("GET /health returns JSON with status", async ({ request }) => {
		const response = await request.get("/health");
		expect(response.ok()).toBeTruthy();
		expect(response.headers()["content-type"]).toContain("application/json");
		const body = await response.json();
		expect(body).toHaveProperty("status");
		expect(body).toHaveProperty("checks");
	});

	test("GET /health/ready returns 200", async ({ request }) => {
		const response = await request.get("/health/ready");
		expect(response.ok()).toBeTruthy();
	});

	test("GET /health/live returns 200", async ({ request }) => {
		const response = await request.get("/health/live");
		expect(response.ok()).toBeTruthy();
	});
});

test.describe("Todo page", () => {
	test("homepage loads at /todos with correct structure", async ({
		page,
	}) => {
		const response = await page.goto("/todos");
		expect(response?.status()).toBe(200);

		// Verify the page has the expected HTML structure
		await expect(page.locator("h1")).toHaveText("Todo");
		await expect(page.locator("#todo-app")).toBeVisible();
	});

	test("no console errors on page load", async ({ page }) => {
		const errors: string[] = [];
		page.on("console", (msg) => {
			if (msg.type() === "error") {
				errors.push(msg.text());
			}
		});

		await page.goto("/todos");
		// Wait for network to settle so Datastar initializes
		await page.waitForLoadState("networkidle");

		expect(errors).toEqual([]);
	});

	test("SSE connection establishes via Datastar", async ({ page }) => {
		// Track SSE requests initiated by Datastar's data-on-load directive
		const sseRequests: string[] = [];
		page.on("request", (request) => {
			const url = request.url();
			if (url.includes("/todos/api/feed")) {
				sseRequests.push(url);
			}
		});

		await page.goto("/todos");

		// Datastar's data-on-load="@get('/todos/api/feed')" fires on page load.
		// Wait for the SSE request to be initiated.
		await page.waitForTimeout(2000);

		expect(sseRequests.length).toBeGreaterThan(0);
	});

	test("todo form is present and interactive", async ({ page }) => {
		await page.goto("/todos");

		// Verify the add-todo form elements exist
		const form = page.locator("#todo-app form");
		await expect(form).toBeVisible();

		// Verify the input field exists
		const input = form.locator("input");
		await expect(input).toBeVisible();

		// Verify the submit button exists
		const button = form.locator('button[type="submit"]');
		await expect(button).toBeVisible();
		await expect(button).toHaveText("Add");
	});
});

test.describe("Navigation", () => {
	test("charts page loads at /charts/astronauts", async ({ page }) => {
		const response = await page.goto("/charts/astronauts");
		expect(response?.status()).toBe(200);
	});
});
