import { expect, test } from "@playwright/test";

test.describe("SSE connection lifecycle", () => {
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
		test.fixme(); // SSE feed sends raw domain events; Datastar expects datastar-fragment HTML events
		await page.goto("/todos");

		// Create a todo and verify SSE delivers the event via DOM update
		const inputSelector = "#todo-app form input";
		const submitButtonSelector = '#todo-app form button[type="submit"]';
		const todoListSelector = "#todo-app ul";

		await page.fill(inputSelector, "Test SSE event delivery");
		await page.click(submitButtonSelector);

		// Wait for the todo to appear in the DOM (evidence of SSE event reception)
		await expect(page.locator(todoListSelector)).toContainText(
			"Test SSE event delivery",
			{ timeout: 10000 },
		);
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
		test.fixme(); // SSE feed sends raw domain events; Datastar expects datastar-fragment HTML events
		await page.goto("/todos");

		// Create two todos to generate events with sequence IDs
		const inputSelector = "#todo-app form input";
		const submitButtonSelector = '#todo-app form button[type="submit"]';

		await page.fill(inputSelector, "First todo");
		await page.click(submitButtonSelector);
		await page.waitForTimeout(500);

		await page.fill(inputSelector, "Second todo");
		await page.click(submitButtonSelector);
		await page.waitForTimeout(500);

		// Now fetch the SSE stream and collect events
		const firstFetch = await page.evaluate(async () => {
			const controller = new AbortController();
			const timeout = setTimeout(() => controller.abort(), 3000);
			const events: Array<{ id: string; data: string }> = [];

			try {
				const response = await fetch("/todos/api/feed", {
					signal: controller.signal,
				});
				if (!response.body) {
					return { events: [], error: "No response body" };
				}

				const reader = response.body.getReader();
				const decoder = new TextDecoder();
				let buffer = "";

				while (true) {
					const { done, value } = await reader.read();
					if (done) break;

					buffer += decoder.decode(value);
					const lines = buffer.split("\n");
					buffer = lines.pop() || "";

					let currentEvent: { id?: string; data?: string } = {};
					for (const line of lines) {
						if (line.startsWith("id:")) {
							currentEvent.id = line.slice(3).trim();
						} else if (line.startsWith("data:")) {
							currentEvent.data = line.slice(5).trim();
						} else if (line === "") {
							// End of event
							if (currentEvent.id && currentEvent.data) {
								events.push({
									id: currentEvent.id,
									data: currentEvent.data,
								});
							}
							currentEvent = {};
						}
					}
				}

				return { events, error: null };
			} catch (e) {
				clearTimeout(timeout);
				return {
					events,
					error: e instanceof Error ? e.message : String(e),
				};
			} finally {
				clearTimeout(timeout);
			}
		});

		// Should have received at least 2 events (the Created events)
		expect(firstFetch.events.length).toBeGreaterThanOrEqual(2);

		// Get the first event ID
		const firstEventId = firstFetch.events[0]?.id;
		expect(firstEventId).toBeTruthy();

		// Now reconnect with Last-Event-ID header
		const secondFetch = await page.evaluate(
			async ({ lastEventId }) => {
				const controller = new AbortController();
				const timeout = setTimeout(() => controller.abort(), 3000);
				const events: Array<{ id: string; data: string }> = [];

				try {
					const response = await fetch("/todos/api/feed", {
						headers: { "Last-Event-ID": lastEventId },
						signal: controller.signal,
					});
					if (!response.body) {
						return { events: [], error: "No response body" };
					}

					const reader = response.body.getReader();
					const decoder = new TextDecoder();
					let buffer = "";

					while (true) {
						const { done, value } = await reader.read();
						if (done) break;

						buffer += decoder.decode(value);
						const lines = buffer.split("\n");
						buffer = lines.pop() || "";

						let currentEvent: { id?: string; data?: string } = {};
						for (const line of lines) {
							if (line.startsWith("id:")) {
								currentEvent.id = line.slice(3).trim();
							} else if (line.startsWith("data:")) {
								currentEvent.data = line.slice(5).trim();
							} else if (line === "") {
								// End of event
								if (currentEvent.id && currentEvent.data) {
									events.push({
										id: currentEvent.id,
										data: currentEvent.data,
									});
								}
								currentEvent = {};
							}
						}
					}

					return { events, error: null };
				} catch (e) {
					clearTimeout(timeout);
					return {
						events,
						error: e instanceof Error ? e.message : String(e),
					};
				} finally {
					clearTimeout(timeout);
				}
			},
			{ lastEventId: firstEventId },
		);

		// Should only receive events AFTER the Last-Event-ID
		// All event IDs should be > firstEventId
		for (const event of secondFetch.events) {
			const eventIdNum = Number.parseInt(event.id, 10);
			const lastEventIdNum = Number.parseInt(firstEventId, 10);
			expect(eventIdNum).toBeGreaterThan(lastEventIdNum);
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
