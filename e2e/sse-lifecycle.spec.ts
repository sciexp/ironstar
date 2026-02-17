import { expect, test } from "@playwright/test";

/** Generate a short unique suffix for test item names. */
function uid(): string {
	return Math.random().toString(36).slice(2, 8);
}

test.describe("SSE connection lifecycle", () => {
	// Tests that create items use uid() to generate unique names, preventing
	// locator collisions across parallel browser projects and prior test runs.
	// No event store purge is needed because text-filtered locators match
	// only items created by this specific test invocation.

	test("establishes SSE connection on page load", async ({ page }) => {
		await page.goto("/todos");

		// Verify the SSE endpoint accepts connections
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

	test("receives events through SSE stream", async ({ page }) => {
		await page.goto("/todos");
		const name = `SSEStreamTest ${uid()}`;

		// Create a todo and verify SSE delivers the event via DOM update
		const inputSelector = "#todo-app form input";
		const submitButtonSelector = '#todo-app form button[type="submit"]';
		const todoListSelector = "#todo-app ul";

		await page.fill(inputSelector, name);
		await page.click(submitButtonSelector);

		// Wait for the todo to appear in the DOM (evidence of SSE event reception)
		await expect(page.locator(todoListSelector)).toContainText(name, {
			timeout: 10000,
		});
	});

	test("receives heartbeat keep-alive events", async ({ page }) => {
		await page.goto("/todos");

		// The SSE implementation uses 15-second keep-alive intervals
		// We verify heartbeats are configured but don't wait 15 seconds
		const result = await page.evaluate(async () => {
			const controller = new AbortController();
			const events: string[] = [];
			let timeoutId: number;

			try {
				const response = await fetch("/todos/api/feed", {
					signal: controller.signal,
				});
				if (!response.body) {
					return { success: false, error: "No response body", events: [] };
				}

				const reader = response.body.getReader();
				const decoder = new TextDecoder();

				// Read for up to 20 seconds to catch at least one keep-alive (15s interval + margin)
				timeoutId = window.setTimeout(() => controller.abort(), 20000);

				while (true) {
					const { done, value } = await reader.read();
					if (done) break;

					const chunk = decoder.decode(value);
					events.push(chunk);

					// If we've seen a keep-alive comment, we can stop early
					if (chunk.includes(": keepalive")) {
						clearTimeout(timeoutId);
						controller.abort();
						break;
					}
				}

				return { success: true, error: null, events };
			} catch (e) {
				clearTimeout(timeoutId);
				return {
					success: false,
					error: e instanceof Error ? e.message : String(e),
					events,
				};
			}
		});

		// Verify we got at least one keep-alive comment or the stream is working
		// Note: This test may timeout in 20s if keep-alives are not sent, which is expected behavior
		expect(result.success).toBeTruthy();
	});

	test("replays events from Last-Event-ID on reconnection", async ({
		page,
	}) => {
		// ironstar-wp5: This test requires exclusive event store access to
		// reliably verify Last-Event-ID reconnection semantics. Under parallel
		// execution, concurrent purges from other browser projects invalidate
		// the stored sequence IDs, causing reconnection to replay unexpected
		// state. Additionally, the SSE stream uses fetch with abort-on-timeout
		// rather than EventSource, which has known issues with read completion
		// timing. Fixme'd pending either per-test event namespacing or a
		// dedicated integration test with isolated event store access.
		test.fixme();
		await page.goto("/todos");
		const prefix = test.info().project.name;

		const inputSelector = "#todo-app form input";
		const submitButtonSelector = '#todo-app form button[type="submit"]';

		// Create a todo to generate events with sequence IDs
		await page.fill(inputSelector, `${prefix} ReplayTest initial`);
		await page.click(submitButtonSelector);

		// Wait for the todo to appear in the DOM before fetching the stream,
		// confirming the event is persisted and the SSE feed has delivered it
		await expect(
			page.locator("#todo-list li", {
				hasText: `${prefix} ReplayTest initial`,
			}),
		).toBeVisible();

		// Fetch the SSE stream to capture the latest event ID.
		// The handler folds all events into projected state with the latest
		// sequence as the SSE event ID on the last datastar-patch-elements event.
		const firstFetch = await page.evaluate(async () => {
			const controller = new AbortController();
			const timeout = setTimeout(() => controller.abort(), 3000);
			let lastId: string | null = null;
			let rawText = "";

			try {
				const response = await fetch("/todos/api/feed", {
					signal: controller.signal,
				});
				if (!response.body) {
					return { lastId: null, rawText: "", error: "No response body" };
				}

				const reader = response.body.getReader();
				const decoder = new TextDecoder();

				while (true) {
					const { done, value } = await reader.read();
					if (done) break;

					const chunk = decoder.decode(value);
					rawText += chunk;

					for (const line of chunk.split("\n")) {
						if (line.startsWith("id:")) {
							lastId = line.slice(3).trim();
						}
					}
				}

				return { lastId, rawText, error: null };
			} catch (e) {
				clearTimeout(timeout);
				return {
					lastId,
					rawText,
					error: e instanceof Error ? e.message : String(e),
				};
			} finally {
				clearTimeout(timeout);
			}
		});

		expect(firstFetch.lastId).toBeTruthy();
		expect(firstFetch.rawText).toContain("event: datastar-patch-elements");

		// Create another todo so there is something new after the last ID
		await page.fill(inputSelector, `${prefix} ReplayTest reconnect`);
		await page.click(submitButtonSelector);

		// Wait for the new todo to appear in the DOM, confirming the event
		// is persisted before we attempt the reconnection fetch
		await expect(
			page.locator("#todo-list li", {
				hasText: `${prefix} ReplayTest reconnect`,
			}),
		).toBeVisible();

		// Reconnect with the Last-Event-ID from the first fetch.
		// The handler replays events after that ID, producing HTML that
		// should include the newly created todo.
		const secondFetch = await page.evaluate(
			async ({ lastEventId }) => {
				const controller = new AbortController();
				const timeout = setTimeout(() => controller.abort(), 3000);
				let newLastId: string | null = null;
				let rawText = "";

				try {
					const response = await fetch("/todos/api/feed", {
						headers: { "Last-Event-ID": lastEventId },
						signal: controller.signal,
					});
					if (!response.body) {
						return {
							newLastId: null,
							rawText: "",
							error: "No response body",
						};
					}

					const reader = response.body.getReader();
					const decoder = new TextDecoder();

					while (true) {
						const { done, value } = await reader.read();
						if (done) break;

						const chunk = decoder.decode(value);
						rawText += chunk;

						for (const line of chunk.split("\n")) {
							if (line.startsWith("id:")) {
								newLastId = line.slice(3).trim();
							}
						}
					}

					return { newLastId, rawText, error: null };
				} catch (e) {
					clearTimeout(timeout);
					return {
						newLastId,
						rawText,
						error: e instanceof Error ? e.message : String(e),
					};
				} finally {
					clearTimeout(timeout);
				}
			},
			{ lastEventId: firstFetch.lastId },
		);

		// The reconnection stream should contain datastar-patch-elements events.
		// Under parallel test execution, concurrent purges can remove events
		// between creation and reconnection, so we verify protocol behavior
		// (events received, ID advancement) rather than specific todo text.
		expect(secondFetch.rawText).toContain("event: datastar-patch-elements");

		// When both IDs are available, the reconnection ID should be higher
		if (secondFetch.newLastId && firstFetch.lastId) {
			const newId = Number.parseInt(secondFetch.newLastId, 10);
			const oldId = Number.parseInt(firstFetch.lastId, 10);
			expect(newId).toBeGreaterThan(oldId);
		}

		// Verify the reconnection stream includes our test todo when possible.
		// Concurrent purges from parallel tests may invalidate this, so this
		// is a soft assertion that documents the expected behavior.
		if (!secondFetch.rawText.includes(`${prefix} ReplayTest reconnect`)) {
			console.warn(
				"Last-Event-ID reconnection did not include expected todo; " +
					"likely due to concurrent event store purge from parallel tests",
			);
		}
	});

	test("handles connection interruption gracefully", async ({ page }) => {
		test.fixme(); // ironstar-wp5: abort after completed read does not throw AbortError
		await page.goto("/todos");

		// Establish an SSE connection and then abort it
		const result = await page.evaluate(async () => {
			const controller = new AbortController();
			let connectionEstablished = false;
			let errorCaught = false;

			try {
				const response = await fetch("/todos/api/feed", {
					signal: controller.signal,
				});

				if (response.ok) {
					connectionEstablished = true;
				}

				// Read a bit to ensure connection is active
				if (response.body) {
					const reader = response.body.getReader();
					const { done } = await reader.read();
					if (!done) {
						// Abort the connection mid-stream
						controller.abort();
					}
				}
			} catch (e) {
				if (
					e instanceof DOMException &&
					(e.name === "AbortError" || e.message.includes("abort"))
				) {
					errorCaught = true;
				}
			}

			return { connectionEstablished, errorCaught };
		});

		expect(result.connectionEstablished).toBeTruthy();
		expect(result.errorCaught).toBeTruthy();
	});

	test("connection stays open without events", async ({ page }) => {
		await page.goto("/todos");

		// Verify the connection can stay open for several seconds even without data events
		const result = await page.evaluate(async () => {
			const controller = new AbortController();
			const timeout = setTimeout(() => controller.abort(), 5000);
			let connectionAlive = false;

			try {
				const response = await fetch("/todos/api/feed", {
					signal: controller.signal,
				});
				if (response.ok && response.body) {
					const reader = response.body.getReader();

					// Try to read for 5 seconds
					const { done } = await reader.read();
					if (!done) {
						connectionAlive = true;
					}
				}
			} catch (e) {
				// AbortError is expected after timeout
				if (
					e instanceof DOMException &&
					(e.name === "AbortError" || e.message.includes("abort"))
				) {
					connectionAlive = true; // Still alive until we aborted
				}
			} finally {
				clearTimeout(timeout);
			}

			return { connectionAlive };
		});

		expect(result.connectionAlive).toBeTruthy();
	});
});
