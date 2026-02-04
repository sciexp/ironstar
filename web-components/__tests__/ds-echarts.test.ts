import { afterEach, describe, expect, it, vi } from "vitest";
import type { DsEcharts } from "../components/ds-echarts";
import { mockChartInstance } from "./setup";
import {
	cleanup,
	render,
	renderWithAttrs,
	triggerResize,
	waitForInit,
	waitForLitUpdate,
} from "./test-utils";
import "../components/ds-echarts";

describe("ds-echarts", () => {
	afterEach(() => {
		cleanup();
	});

	it("element can be created and has correct tag name", async () => {
		const element = await render<DsEcharts>("ds-echarts");
		expect(element.tagName.toLowerCase()).toBe("ds-echarts");
	});

	it("element renders chart-container div after update", async () => {
		const element = await render<DsEcharts>("ds-echarts");
		await waitForLitUpdate(element);

		const container = element.querySelector(".chart-container");
		expect(container).toBeTruthy();
		expect(container?.tagName.toLowerCase()).toBe("div");
	});

	it("element accepts option attribute", async () => {
		const element = await render<DsEcharts>("ds-echarts");
		const testOption = JSON.stringify({ title: { text: "Test Chart" } });

		element.setAttribute("option", testOption);
		await waitForLitUpdate(element);

		expect(element.option).toBe(testOption);
	});

	it("disconnectedCallback cleans up properly", async () => {
		const element = await render<DsEcharts>("ds-echarts");
		await waitForLitUpdate(element);

		// Trigger disconnection
		element.remove();

		// Verify element is no longer in document
		expect(document.body.contains(element)).toBe(false);
	});

	describe("events property", () => {
		it('has default value of "lifecycle,mouse"', async () => {
			const element = await render<DsEcharts>("ds-echarts");
			expect(element.events).toBe("lifecycle,mouse");
		});

		it("parses single event category", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.events = "click";
			await waitForLitUpdate(element);
			expect(element.events).toBe("click");
		});

		it("parses multiple event categories", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.events = "lifecycle,mouse";
			await waitForLitUpdate(element);
			expect(element.events).toBe("lifecycle,mouse");
		});

		it("handles empty string", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.events = "";
			await waitForLitUpdate(element);
			expect(element.events).toBe("");
		});

		it("can be set via attribute", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.setAttribute("events", "component");
			await waitForLitUpdate(element);
			expect(element.events).toBe("component");
		});
	});

	describe("hoverThrottle property", () => {
		it("has default value of 100", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			expect(element.hoverThrottle).toBe(100);
		});

		it("accepts custom value via attribute", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.setAttribute("hover-throttle", "250");
			await waitForLitUpdate(element);
			expect(element.hoverThrottle).toBe(250);
		});

		it("can be updated dynamically", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.setAttribute("hover-throttle", "500");
			await waitForLitUpdate(element);
			expect(element.hoverThrottle).toBe(500);
		});

		it("attribute name uses kebab-case", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.setAttribute("hover-throttle", "300");
			await waitForLitUpdate(element);
			expect(element.hoverThrottle).toBe(300);
		});
	});

	describe("renderer property", () => {
		it('has default value of "svg"', async () => {
			const element = await render<DsEcharts>("ds-echarts");
			expect(element.renderer).toBe("svg");
		});

		it('can be set to "canvas" via attribute', async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.setAttribute("renderer", "canvas");
			await waitForLitUpdate(element);
			expect(element.renderer).toBe("canvas");
		});

		it("passes renderer option to echarts.init", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.setAttribute("renderer", "canvas");
			await waitForInit(element);

			// Check that echarts.init was called with renderer option
			const echarts = await import("echarts");
			const initCalls = vi.mocked(echarts.init).mock.calls;
			const lastCall = initCalls[initCalls.length - 1];
			expect(lastCall[2]).toEqual({ renderer: "canvas" });
		});

		it("reinitializes chart when renderer changes", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			await waitForInit(element);

			const echarts = await import("echarts");
			const initCallsBefore = vi.mocked(echarts.init).mock.calls.length;

			// Change renderer
			element.renderer = "canvas";
			await waitForLitUpdate(element);

			const initCallsAfter = vi.mocked(echarts.init).mock.calls.length;
			expect(initCallsAfter).toBeGreaterThan(initCallsBefore);
		});
	});

	describe("createChartEvent helper", () => {
		it("creates CustomEvent with bubbles:true and composed:true", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			await waitForLitUpdate(element);

			let capturedEvent: CustomEvent<unknown> | null = null;
			const wrapper = document.createElement("div");
			wrapper.appendChild(element);
			document.body.appendChild(wrapper);

			wrapper.addEventListener("test-event", (e) => {
				capturedEvent = e as CustomEvent<unknown>;
			});

			// Use reflection to access private method for testing
			const createEvent = (element as any).createChartEvent.bind(element);
			const testEvent = createEvent("test-event", { foo: "bar" });
			element.dispatchEvent(testEvent);

			expect(capturedEvent).toBeTruthy();
			expect(capturedEvent!.bubbles).toBe(true);
			expect(capturedEvent!.composed).toBe(true);
			expect(capturedEvent!.detail).toEqual({ foo: "bar" });

			wrapper.remove();
		});

		it("accepts type string and detail object", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			await waitForLitUpdate(element);

			const createEvent = (element as any).createChartEvent.bind(element);
			const event1 = createEvent("event-type-1", { value: 123 });
			const event2 = createEvent("event-type-2", { name: "test", count: 456 });

			expect(event1.type).toBe("event-type-1");
			expect(event1.detail).toEqual({ value: 123 });
			expect(event2.type).toBe("event-type-2");
			expect(event2.detail).toEqual({ name: "test", count: 456 });
		});

		it("event can be dispatched and received by parent element", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			await waitForLitUpdate(element);

			const wrapper = document.createElement("div");
			wrapper.appendChild(element);
			document.body.appendChild(wrapper);

			let receivedDetail: any = null;
			wrapper.addEventListener("custom-test", (e) => {
				receivedDetail = (e as CustomEvent).detail;
			});

			const createEvent = (element as any).createChartEvent.bind(element);
			const testEvent = createEvent("custom-test", {
				data: "test-data",
				id: 42,
			});
			element.dispatchEvent(testEvent);

			expect(receivedDetail).toEqual({ data: "test-data", id: 42 });

			wrapper.remove();
		});
	});

	describe("sanitizePayload helper", () => {
		it("removes non-serializable properties", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			await waitForLitUpdate(element);

			const sanitize = (element as any).sanitizePayload.bind(element);

			const payload = {
				validString: "test",
				validNumber: 42,
				validBoolean: true,
				validArray: [1, 2, 3],
				validObject: { nested: "value" },
				invalidFunction: () => {},
				invalidDomElement: document.createElement("div"),
				invalidEvent: new Event("click"),
			};

			const sanitized = sanitize(payload);

			expect(sanitized.validString).toBe("test");
			expect(sanitized.validNumber).toBe(42);
			expect(sanitized.validBoolean).toBe(true);
			expect(sanitized.validArray).toEqual([1, 2, 3]);
			expect(sanitized.validObject).toEqual({ nested: "value" });
			expect(sanitized.invalidFunction).toBeUndefined();
			expect(sanitized.invalidDomElement).toBeUndefined();
			expect(sanitized.invalidEvent).toBeUndefined();
		});

		it("sanitized payload can be JSON.stringify'd without error", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			await waitForLitUpdate(element);

			const sanitize = (element as any).sanitizePayload.bind(element);

			const payload = {
				name: "test",
				value: 123,
				metadata: { type: "chart", version: 1 },
				badFunction: () => {},
				badNode: document.createElement("span"),
			};

			const sanitized = sanitize(payload);

			expect(() => {
				JSON.stringify(sanitized);
			}).not.toThrow();

			const jsonString = JSON.stringify(sanitized);
			const parsed = JSON.parse(jsonString);

			expect(parsed.name).toBe("test");
			expect(parsed.value).toBe(123);
			expect(parsed.metadata).toEqual({ type: "chart", version: 1 });
			expect(parsed.badFunction).toBeUndefined();
			expect(parsed.badNode).toBeUndefined();
		});

		it("preserves essential ECharts properties", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			await waitForLitUpdate(element);

			const sanitize = (element as any).sanitizePayload.bind(element);

			const echartsPayload = {
				seriesName: "Sales Data",
				dataIndex: 3,
				value: 1234.56,
				name: "Q3",
				componentType: "series",
				seriesType: "bar",
				color: "#5470c6",
				data: { category: "Revenue", amount: 1234.56 },
			};

			const sanitized = sanitize(echartsPayload);

			expect(sanitized.seriesName).toBe("Sales Data");
			expect(sanitized.dataIndex).toBe(3);
			expect(sanitized.value).toBe(1234.56);
			expect(sanitized.name).toBe("Q3");
			expect(sanitized.componentType).toBe("series");
			expect(sanitized.seriesType).toBe("bar");
			expect(sanitized.color).toBe("#5470c6");
			expect(sanitized.data).toEqual({ category: "Revenue", amount: 1234.56 });
		});
	});

	describe("isEventEnabled helper", () => {
		it("returns true when category is in events list", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.events = "lifecycle,mouse";
			await waitForLitUpdate(element);

			const isEnabled = (element as any).isEventEnabled.bind(element);

			expect(isEnabled("lifecycle")).toBe(true);
			expect(isEnabled("mouse")).toBe(true);
			expect(isEnabled("component")).toBe(false);
		});

		it("returns false when events is empty string", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.events = "";
			await waitForLitUpdate(element);

			const isEnabled = (element as any).isEventEnabled.bind(element);

			expect(isEnabled("lifecycle")).toBe(false);
			expect(isEnabled("mouse")).toBe(false);
			expect(isEnabled("component")).toBe(false);
		});

		it("handles single category", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.events = "mouse";
			await waitForLitUpdate(element);

			const isEnabled = (element as any).isEventEnabled.bind(element);

			expect(isEnabled("lifecycle")).toBe(false);
			expect(isEnabled("mouse")).toBe(true);
			expect(isEnabled("component")).toBe(false);
		});

		it("handles whitespace in events list", async () => {
			const element = await render<DsEcharts>("ds-echarts");
			element.events = "lifecycle, mouse, component";
			await waitForLitUpdate(element);

			const isEnabled = (element as any).isEventEnabled.bind(element);

			expect(isEnabled("lifecycle")).toBe(true);
			expect(isEnabled("mouse")).toBe(true);
			expect(isEnabled("component")).toBe(true);
		});
	});

	describe("lifecycle events", () => {
		describe("chart-ready event (by6.5)", () => {
			it("emits chart-ready after initialization", async () => {
				const eventPromise = new Promise<CustomEvent>((resolve) => {
					document.body.addEventListener(
						"chart-ready",
						(e) => resolve(e as CustomEvent),
						{ once: true },
					);
				});

				const element = await render<DsEcharts>("ds-echarts");
				element.option = JSON.stringify({ title: { text: "Test" } });
				element.style.width = "600px";
				element.style.height = "400px";
				await waitForLitUpdate(element);

				const event = await eventPromise;
				expect(event.detail).toHaveProperty("width");
				expect(event.detail).toHaveProperty("height");
				expect(event.detail).toHaveProperty("theme");
			});

			it("includes correct width and height in payload", async () => {
				const eventPromise = new Promise<CustomEvent>((resolve) => {
					document.body.addEventListener(
						"chart-ready",
						(e) => resolve(e as CustomEvent),
						{ once: true },
					);
				});

				const element = await render<DsEcharts>("ds-echarts");
				element.style.width = "800px";
				element.style.height = "600px";
				element.option = JSON.stringify({ title: { text: "Test" } });
				await waitForLitUpdate(element);

				const event = await eventPromise;
				expect(event.detail.width).toBeGreaterThan(0);
				expect(event.detail.height).toBeGreaterThan(0);
			});

			it("includes theme in payload", async () => {
				const eventPromise = new Promise<CustomEvent>((resolve) => {
					document.body.addEventListener(
						"chart-ready",
						(e) => resolve(e as CustomEvent),
						{ once: true },
					);
				});

				// Set theme attribute BEFORE element is added to DOM so it's used during initialization
				await renderWithAttrs<DsEcharts>("ds-echarts", {
					theme: "dark",
					option: JSON.stringify({ title: { text: "Test" } }),
				});
				await waitForInit();

				const event = await eventPromise;
				expect(event.detail.theme).toBe("dark");
			});

			it("respects isEventEnabled check for lifecycle", async () => {
				let eventFired = false;
				const listener = () => {
					eventFired = true;
				};
				document.body.addEventListener("chart-ready", listener);

				// Set events='mouse' BEFORE element is added to DOM to disable lifecycle during init
				await renderWithAttrs<DsEcharts>("ds-echarts", {
					events: "mouse", // disable lifecycle
					option: JSON.stringify({ title: { text: "Test" } }),
				});
				await waitForInit();

				// Wait a bit to ensure event would have fired
				await new Promise((resolve) => setTimeout(resolve, 50));

				expect(eventFired).toBe(false);
				document.body.removeEventListener("chart-ready", listener);
			});
		});

		describe("chart-updated event (by6.6)", () => {
			it("emits chart-updated after setOption succeeds", async () => {
				const element = await render<DsEcharts>("ds-echarts");
				element.style.width = "600px";
				element.style.height = "400px";
				element.option = JSON.stringify({ title: { text: "Initial" } });
				await waitForLitUpdate(element);

				// Wait for initialization
				await new Promise((resolve) => setTimeout(resolve, 100));

				const eventPromise = new Promise<CustomEvent>((resolve) => {
					document.body.addEventListener(
						"chart-updated",
						(e) => resolve(e as CustomEvent),
						{ once: true },
					);
				});

				element.option = JSON.stringify({ title: { text: "Updated" } });
				await waitForLitUpdate(element);

				const event = await eventPromise;
				expect(event.detail).toHaveProperty("timestamp");
			});

			it("includes timestamp in payload", async () => {
				const element = await render<DsEcharts>("ds-echarts");
				element.style.width = "600px";
				element.style.height = "400px";
				element.option = JSON.stringify({ title: { text: "Initial" } });
				await waitForLitUpdate(element);

				await new Promise((resolve) => setTimeout(resolve, 100));

				const eventPromise = new Promise<CustomEvent>((resolve) => {
					document.body.addEventListener(
						"chart-updated",
						(e) => resolve(e as CustomEvent),
						{ once: true },
					);
				});

				const beforeTime = Date.now();
				element.option = JSON.stringify({ title: { text: "Updated" } });
				await waitForLitUpdate(element);

				const event = await eventPromise;
				const afterTime = Date.now();

				expect(event.detail.timestamp).toBeGreaterThanOrEqual(beforeTime);
				expect(event.detail.timestamp).toBeLessThanOrEqual(afterTime);
			});

			it("does not fire when option is empty", async () => {
				const element = await render<DsEcharts>("ds-echarts");
				element.style.width = "600px";
				element.style.height = "400px";
				await waitForLitUpdate(element);

				let eventFired = false;
				const listener = () => {
					eventFired = true;
				};
				document.body.addEventListener("chart-updated", listener);

				element.option = "{}";
				await waitForLitUpdate(element);

				await new Promise((resolve) => setTimeout(resolve, 100));

				expect(eventFired).toBe(false);
				document.body.removeEventListener("chart-updated", listener);
			});
		});

		describe("chart-resized event (by6.7)", () => {
			it("emits chart-resized after resize", async () => {
				const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
					"resize-delay": "10",
					option: JSON.stringify({ title: { text: "Test" } }),
				});
				await waitForInit();

				const eventPromise = new Promise<CustomEvent>((resolve) => {
					document.body.addEventListener(
						"chart-resized",
						(e) => resolve(e as CustomEvent),
						{ once: true },
					);
				});

				// Change dimensions and manually trigger ResizeObserver
				const container = element.querySelector(
					".chart-container",
				) as HTMLElement;
				container.style.width = "800px";
				container.style.height = "600px";
				triggerResize(element);

				// Wait for debounce
				await new Promise((resolve) => setTimeout(resolve, 50));

				const event = await eventPromise;
				expect(event.detail).toHaveProperty("width");
				expect(event.detail).toHaveProperty("height");
			});

			it("includes new dimensions in payload", async () => {
				const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
					"resize-delay": "10",
					option: JSON.stringify({ title: { text: "Test" } }),
				});
				await waitForInit();

				const eventPromise = new Promise<CustomEvent>((resolve) => {
					document.body.addEventListener(
						"chart-resized",
						(e) => resolve(e as CustomEvent),
						{ once: true },
					);
				});

				// Change dimensions and manually trigger ResizeObserver
				const container = element.querySelector(
					".chart-container",
				) as HTMLElement;
				container.style.width = "1000px";
				container.style.height = "800px";
				triggerResize(element);

				// Wait for debounce
				await new Promise((resolve) => setTimeout(resolve, 50));

				const event = await eventPromise;
				expect(event.detail.width).toBeGreaterThan(0);
				expect(event.detail.height).toBeGreaterThan(0);
			});

			it("respects resizeDelay debounce", async () => {
				const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
					"resize-delay": "100",
					option: JSON.stringify({ title: { text: "Test" } }),
				});
				await waitForInit();

				let eventCount = 0;
				const listener = () => {
					eventCount++;
				};
				document.body.addEventListener("chart-resized", listener);

				// Trigger multiple resizes rapidly
				const container = element.querySelector(
					".chart-container",
				) as HTMLElement;
				container.style.width = "700px";
				triggerResize(element);
				await new Promise((resolve) => setTimeout(resolve, 30));

				container.style.width = "800px";
				triggerResize(element);
				await new Promise((resolve) => setTimeout(resolve, 30));

				container.style.width = "900px";
				triggerResize(element);

				// Wait for debounce to complete
				await new Promise((resolve) => setTimeout(resolve, 150));

				// Should only fire once due to debounce
				expect(eventCount).toBe(1);
				document.body.removeEventListener("chart-resized", listener);
			});
		});

		describe("chart-disposed event (by6.8)", () => {
			it("emits chart-disposed before chart.dispose()", async () => {
				const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
					option: JSON.stringify({ title: { text: "Test" } }),
				});
				await waitForInit();

				// Listen on element directly since bubbling during disconnection is unreliable in happy-dom
				const eventPromise = new Promise<CustomEvent>((resolve) => {
					element.addEventListener(
						"chart-disposed",
						(e) => resolve(e as CustomEvent),
						{ once: true },
					);
				});

				element.remove();

				const event = await eventPromise;
				expect(event.detail).toEqual({});
			});

			it("emits when element is removed from DOM", async () => {
				const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
					option: JSON.stringify({ title: { text: "Test" } }),
				});
				await waitForInit();

				let eventFired = false;
				// Listen on element directly since bubbling during disconnection is unreliable in happy-dom
				element.addEventListener("chart-disposed", () => {
					eventFired = true;
				});

				element.remove();

				await new Promise((resolve) => setTimeout(resolve, 10));

				expect(eventFired).toBe(true);
			});
		});

		describe("chart-error event (by6.9)", () => {
			it("emits chart-error on invalid JSON", async () => {
				const element = await render<DsEcharts>("ds-echarts");
				element.style.width = "600px";
				element.style.height = "400px";
				await waitForLitUpdate(element);

				const eventPromise = new Promise<CustomEvent>((resolve) => {
					document.body.addEventListener(
						"chart-error",
						(e) => resolve(e as CustomEvent),
						{ once: true },
					);
				});

				element.option = "invalid json{{";
				await waitForLitUpdate(element);

				const event = await eventPromise;
				expect(event.detail).toHaveProperty("message");
				expect(event.detail).toHaveProperty("error");
			});

			it("includes error message in payload", async () => {
				const element = await render<DsEcharts>("ds-echarts");
				element.style.width = "600px";
				element.style.height = "400px";
				await waitForLitUpdate(element);

				const eventPromise = new Promise<CustomEvent>((resolve) => {
					document.body.addEventListener(
						"chart-error",
						(e) => resolve(e as CustomEvent),
						{ once: true },
					);
				});

				element.option = "not valid json";
				await waitForLitUpdate(element);

				const event = await eventPromise;
				expect(event.detail.message).toBeTruthy();
				expect(typeof event.detail.message).toBe("string");
			});

			it("component remains stable after error", async () => {
				const element = await render<DsEcharts>("ds-echarts");
				element.style.width = "600px";
				element.style.height = "400px";
				element.option = JSON.stringify({ title: { text: "Valid" } });
				await waitForLitUpdate(element);

				// Trigger error
				element.option = "invalid";
				await waitForLitUpdate(element);

				// Should still be able to set valid option
				element.option = JSON.stringify({ title: { text: "Valid Again" } });
				await waitForLitUpdate(element);

				expect(element.option).toBe(
					JSON.stringify({ title: { text: "Valid Again" } }),
				);
			});
		});
	});

	describe("setupEventListeners (by6.10)", () => {
		afterEach(() => {
			mockChartInstance.clearHandlers();
			vi.clearAllMocks();
		});

		it("is called during initChart and registers handlers", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			// Verify chart.on was called to register handlers
			expect(mockChartInstance.on).toHaveBeenCalled();
			expect(mockChartInstance.on.mock.calls.length).toBeGreaterThan(0);
		});

		it("registers mouse event handlers when mouse category enabled", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,mouse",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			// Verify mouse events were registered (mouseover/mouseout removed in by6.14)
			const mouseEvents = ["click", "dblclick", "contextmenu"];
			for (const eventName of mouseEvents) {
				const handlers = mockChartInstance.getHandlers(eventName);
				expect(handlers.length).toBe(1);
			}
		});

		it("registers component event handlers when component category enabled", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,component",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			// Verify component events were registered
			const componentEvents = [
				"legendselectchanged",
				"datazoom",
				"selectchanged",
				"brush",
			];
			for (const eventName of componentEvents) {
				const handlers = mockChartInstance.getHandlers(eventName);
				expect(handlers.length).toBe(1);
			}
		});

		it("does not register handlers when category disabled", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			// Verify mouse and component events were NOT registered (by6.14: mouseover/mouseout moved to hover)
			const mouseEvents = ["click", "dblclick", "contextmenu"];
			const hoverEvents = ["mouseover", "mouseout"];
			const componentEvents = [
				"legendselectchanged",
				"datazoom",
				"selectchanged",
				"brush",
			];

			for (const eventName of [
				...mouseEvents,
				...hoverEvents,
				...componentEvents,
			]) {
				const handlers = mockChartInstance.getHandlers(eventName);
				expect(handlers.length).toBe(0);
			}
		});

		it("cleans up handlers in disconnectedCallback", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,mouse",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			// Verify handlers were registered
			expect(mockChartInstance.getHandlers("click").length).toBe(1);

			// Remove element and verify chart.off was called
			element.remove();

			expect(mockChartInstance.off).toHaveBeenCalled();
			// Verify all mouse event handlers were removed (by6.14: mouseover/mouseout moved to hover)
			const mouseEvents = ["click", "dblclick", "contextmenu"];
			for (const eventName of mouseEvents) {
				expect(mockChartInstance.off).toHaveBeenCalledWith(
					eventName,
					expect.any(Function),
				);
			}
		});

		it("forwards ECharts events as CustomEvents", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,mouse",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			// Capture the dispatched event
			const eventPromise = new Promise<CustomEvent>((resolve) => {
				document.body.addEventListener(
					"chart-click",
					(e) => resolve(e as CustomEvent),
					{ once: true },
				);
			});

			// Get the registered handler and call it with mock params
			const clickHandlers = mockChartInstance.getHandlers("click");
			expect(clickHandlers.length).toBe(1);

			const mockParams = {
				type: "click",
				seriesName: "Test Series",
				dataIndex: 5,
				value: 100,
				name: "Data Point",
				// Include a non-serializable property that should be sanitized out
				eventTarget: document.createElement("div"),
			};

			clickHandlers[0](mockParams);

			const event = await eventPromise;
			expect(event.type).toBe("chart-click");
			expect(event.detail).toHaveProperty("type", "click");
			expect(event.detail).toHaveProperty("seriesName", "Test Series");
			expect(event.detail).toHaveProperty("dataIndex", 5);
			expect(event.detail).toHaveProperty("value", 100);
			expect(event.detail).toHaveProperty("name", "Data Point");
			// Verify non-serializable property was removed
			expect(event.detail).not.toHaveProperty("eventTarget");
		});
	});

	describe("hover events (by6.14)", () => {
		it("forwards mouseover as chart-hover-start when hover enabled", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,hover",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			const eventPromise = new Promise<CustomEvent>((resolve) => {
				document.body.addEventListener(
					"chart-hover-start",
					(e) => resolve(e as CustomEvent),
					{ once: true },
				);
			});

			const handlers = mockChartInstance.getHandlers("mouseover");
			expect(handlers.length).toBe(1);
			handlers[0]({ type: "mouseover", seriesName: "Test" });

			const event = await eventPromise;
			expect(event.type).toBe("chart-hover-start");
			expect(event.detail).toHaveProperty("seriesName", "Test");
		});

		it("forwards mouseout as chart-hover-end when hover enabled", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,hover",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			const eventPromise = new Promise<CustomEvent>((resolve) => {
				document.body.addEventListener(
					"chart-hover-end",
					(e) => resolve(e as CustomEvent),
					{ once: true },
				);
			});

			const handlers = mockChartInstance.getHandlers("mouseout");
			expect(handlers.length).toBe(1);
			handlers[0]({ type: "mouseout", seriesName: "Test" });

			const event = await eventPromise;
			expect(event.type).toBe("chart-hover-end");
			expect(event.detail).toHaveProperty("seriesName", "Test");
		});

		it("hover events disabled by default", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			// With default events='lifecycle,mouse', hover should NOT be registered
			const mouseoverHandlers = mockChartInstance.getHandlers("mouseover");
			const mouseoutHandlers = mockChartInstance.getHandlers("mouseout");

			// mouseover/mouseout should not be registered
			// (they're no longer in mouse category either)
			expect(mouseoverHandlers.length).toBe(0);
			expect(mouseoutHandlers.length).toBe(0);
		});

		it("throttles hover events by hoverThrottle prop", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,hover",
				"hover-throttle": "200",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			let eventCount = 0;
			document.body.addEventListener("chart-hover-start", () => eventCount++);

			const handlers = mockChartInstance.getHandlers("mouseover");

			// Rapid fire events
			handlers[0]({ type: "mouseover" });
			handlers[0]({ type: "mouseover" });
			handlers[0]({ type: "mouseover" });

			// Should only fire once due to throttle
			expect(eventCount).toBe(1);
		});

		it("throttles hover-end events by hoverThrottle prop", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,hover",
				"hover-throttle": "200",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			let eventCount = 0;
			document.body.addEventListener("chart-hover-end", () => eventCount++);

			const handlers = mockChartInstance.getHandlers("mouseout");

			// Rapid fire events
			handlers[0]({ type: "mouseout" });
			handlers[0]({ type: "mouseout" });
			handlers[0]({ type: "mouseout" });

			// Should only fire once due to throttle
			expect(eventCount).toBe(1);
		});

		it("allows hover events after throttle period elapses", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,hover",
				"hover-throttle": "100",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			let eventCount = 0;
			document.body.addEventListener("chart-hover-start", () => eventCount++);

			const handlers = mockChartInstance.getHandlers("mouseover");

			// First event
			handlers[0]({ type: "mouseover" });
			expect(eventCount).toBe(1);

			// Second event immediately (should be throttled)
			handlers[0]({ type: "mouseover" });
			expect(eventCount).toBe(1);

			// Wait for throttle period to elapse
			await new Promise((resolve) => setTimeout(resolve, 101));

			// Third event (should fire)
			handlers[0]({ type: "mouseover" });
			expect(eventCount).toBe(2);
		});

		it("sanitizes hover event payloads", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,hover",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			let capturedEvent: CustomEvent | null = null;
			document.body.addEventListener(
				"chart-hover-start",
				(e) => {
					capturedEvent = e as CustomEvent;
				},
				{ once: true },
			);

			const handlers = mockChartInstance.getHandlers("mouseover");

			const mockParams = {
				type: "mouseover",
				seriesName: "Sales",
				dataIndex: 3,
				value: 100,
				// Non-serializable property that should be removed
				target: document.createElement("div"),
			};

			handlers[0](mockParams);

			// Give event time to propagate
			await new Promise((resolve) => setTimeout(resolve, 10));

			expect(capturedEvent).toBeTruthy();
			expect(capturedEvent!.detail).toHaveProperty("seriesName", "Sales");
			expect(capturedEvent!.detail).toHaveProperty("dataIndex", 3);
			expect(capturedEvent!.detail).toHaveProperty("value", 100);
			// Non-serializable property should be removed
			expect(capturedEvent!.detail).not.toHaveProperty("target");
		});

		it("cleans up hover handlers in disconnectedCallback", async () => {
			const element = await renderWithAttrs<DsEcharts>("ds-echarts", {
				events: "lifecycle,hover",
				option: JSON.stringify({ title: { text: "Test" } }),
			});
			await waitForInit();

			// Verify handlers were registered
			expect(mockChartInstance.getHandlers("mouseover").length).toBe(1);
			expect(mockChartInstance.getHandlers("mouseout").length).toBe(1);

			// Clear the mock to get fresh call counts
			vi.clearAllMocks();

			// Remove element
			element.remove();

			// Give cleanup time to complete
			await new Promise((resolve) => setTimeout(resolve, 10));

			// Verify chart.off was called for both hover events
			expect(mockChartInstance.off).toHaveBeenCalledWith(
				"mouseover",
				expect.any(Function),
			);
			expect(mockChartInstance.off).toHaveBeenCalledWith(
				"mouseout",
				expect.any(Function),
			);
		});
	});
});
