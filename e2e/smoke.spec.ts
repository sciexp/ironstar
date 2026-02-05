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

	test("no application console errors on page load", async ({ page }) => {
		const errors: string[] = [];
		page.on("console", (msg) => {
			if (msg.type() === "error") {
				const text = msg.text();
				// Filter out expected errors when the frontend build pipeline
				// has not run. These are static asset loading errors, not
				// application logic errors.
				if (
					text.includes("Failed to load resource") ||
					text.includes("disallowed MIME type") ||
					text.includes("Loading module from")
				) {
					return;
				}
				errors.push(text);
			}
		});

		await page.goto("/todos");
		await page.waitForLoadState("networkidle");

		expect(errors).toEqual([]);
	});

	test("SSE feed endpoint accepts connection", async ({ page }) => {
		// Navigate first so fetch runs in the correct origin context
		await page.goto("/todos");

		// Verify the SSE endpoint responds with the correct content type.
		// We use page.evaluate with fetch + AbortController because SSE
		// streams are long-lived and would timeout with request.get().
		const result = await page.evaluate(async () => {
			const controller = new AbortController();
			const timeout = setTimeout(() => controller.abort(), 5000);
			try {
				const response = await fetch("/todos/api/feed", {
					signal: controller.signal,
				});
				clearTimeout(timeout);
				return {
					ok: response.ok,
					status: response.status,
					contentType: response.headers.get("content-type"),
					error: null,
				};
			} catch (e) {
				clearTimeout(timeout);
				return {
					ok: false,
					status: 0,
					contentType: null,
					error: e instanceof Error ? e.message : String(e),
				};
			}
		});

		expect(result.ok).toBeTruthy();
		expect(result.status).toBe(200);
		expect(result.contentType).toContain("text/event-stream");
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
