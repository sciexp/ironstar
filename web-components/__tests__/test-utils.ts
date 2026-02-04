import type { LitElement } from "lit";
import { resizeObserverInstances } from "./setup";

/**
 * Render helper: creates element, appends to document.body, triggers connectedCallback
 */
export async function render<T extends LitElement>(
	tagName: string,
): Promise<T> {
	const element = document.createElement(tagName) as T;
	document.body.appendChild(element);
	await waitForLitUpdate(element);
	return element;
}

/**
 * Render helper with initial attributes: sets attributes BEFORE adding to DOM
 * This is needed for lifecycle event tests where events fire during initialization
 */
export async function renderWithAttrs<T extends LitElement>(
	tagName: string,
	attrs: Record<string, string>,
): Promise<T> {
	const element = document.createElement(tagName) as T;
	// Set attributes BEFORE adding to DOM
	for (const [key, value] of Object.entries(attrs)) {
		element.setAttribute(key, value);
	}
	document.body.appendChild(element);
	await waitForLitUpdate(element);
	return element;
}

/**
 * Awaits element.updateComplete for Lit element updates
 */
export async function waitForLitUpdate(element: LitElement): Promise<void> {
	await element.updateComplete;
}

/**
 * Wait for initialization to complete (microtask flush + small delay)
 * Use this after render to ensure chart is fully initialized
 */
export async function waitForInit(element?: LitElement): Promise<void> {
	// Flush microtasks (for Promise.resolve().then() in firstUpdated)
	await Promise.resolve();
	await Promise.resolve();
	// Small delay to ensure any async initialization completes
	await new Promise((resolve) => setTimeout(resolve, 10));
}

/**
 * Trigger ResizeObserver callback for an element's chart container
 * Use this to simulate resize events in tests
 */
export function triggerResize(element: LitElement): void {
	const chartContainer = element.querySelector(".chart-container");
	if (!chartContainer) return;

	for (const observer of resizeObserverInstances) {
		if (observer.isObserving(chartContainer)) {
			observer.triggerForElement(chartContainer);
		}
	}
}

/**
 * Cleanup: removes test elements from document.body
 */
export function cleanup(): void {
	document.body.innerHTML = "";
	// Clear any remaining resize observer instances
	resizeObserverInstances.length = 0;
}

/**
 * Access the mock ECharts instance for assertions
 */
export function mockEchartsInstance() {
	// Import the mock from the echarts module
	const echarts = require("echarts");
	return echarts.init();
}
