import { expect, test } from "@playwright/test";

test.describe("EventCatalog Homepage", () => {
	test("loads successfully", async ({ page }) => {
		await page.goto("/");

		// Verify the page has loaded (EventCatalog shows "EventCatalog" in title)
		await expect(page).toHaveTitle(/EventCatalog/i);
	});

	test("displays catalog content", async ({ page }) => {
		await page.goto("/");

		// Wait for page to fully load
		await page.waitForLoadState("networkidle");

		// The page should have some visible content
		const body = page.locator("body");
		await expect(body).toBeVisible();
	});

	test("loads without console errors", async ({ page }) => {
		const errors: string[] = [];

		page.on("console", (msg) => {
			if (msg.type() === "error") {
				errors.push(msg.text());
			}
		});

		await page.goto("/");
		await page.waitForLoadState("networkidle");

		// Filter out known benign errors (e.g., favicon 404 in dev mode)
		const significantErrors = errors.filter(
			(e) => !e.includes("favicon") && !e.includes("404"),
		);

		expect(significantErrors).toHaveLength(0);
	});
});
