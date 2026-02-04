import type { ECharts } from "echarts";
import * as echarts from "echarts";
import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("ds-echarts")
export class DsEcharts extends LitElement {
	// Props matching Rocket ECharts interface
	@property({ type: String }) option = "{}";
	@property({ type: String }) theme = "default";
	@property({ type: Number, attribute: "resize-delay" }) resizeDelay = 100;
	@property({ type: String }) events = "lifecycle,mouse";
	@property({ type: Number, attribute: "hover-throttle" }) hoverThrottle = 100;
	@property({ type: String }) renderer: "svg" | "canvas" = "svg";

	// Internal state
	private chart: ECharts | null = null;
	private chartContainer: HTMLDivElement | null = null;
	private resizeObserver: ResizeObserver | null = null;
	private resizeTimeout: number | undefined;
	private currentTheme: string = "default";
	private mediaQueryHandler: ((e: MediaQueryListEvent) => void) | null = null;
	private initialized: boolean = false;
	private boundHandlers: Map<string, (params: any) => void> = new Map();

	// Light DOM for Open Props CSS token inheritance
	protected createRenderRoot() {
		return this;
	}

	render() {
		return html`<div class="chart-container" style="width: 100%; height: 100%;"></div>`;
	}

	firstUpdated() {
		this.chartContainer = this.querySelector(".chart-container");
		if (!this.chartContainer) {
			console.error("[ds-echarts] Chart container not found");
			return;
		}

		this.currentTheme = this.theme;

		// Defer initialization to ensure container has dimensions
		// Use Promise.resolve() for better compatibility with test environments
		Promise.resolve().then(() => this.initChart());
	}

	private initChart() {
		if (!this.chartContainer) return;

		// Ensure container has dimensions
		this.ensureDimensions();

		if (
			this.chartContainer.clientWidth === 0 ||
			this.chartContainer.clientHeight === 0
		) {
			// Retry next microtask if no dimensions yet
			Promise.resolve().then(() => this.initChart());
			return;
		}

		// Dispose existing chart if any
		this.chart?.dispose();

		// Initialize ECharts (null theme = ECharts default)
		const themeArg = this.currentTheme === "default" ? null : this.currentTheme;
		this.chart = echarts.init(this.chartContainer, themeArg, {
			renderer: this.renderer,
		});

		// Mark as initialized BEFORE calling updateOption to prevent recursion
		// (updateOption checks this.initialized and calls initChart if false)
		this.initialized = true;

		// Apply initial option
		this.updateOption();

		// Set up resize observer
		this.setupResizeObserver();

		// Set up dark mode listener
		this.setupMediaQueryListener();

		// Emit chart-ready event (by6.5)
		if (this.isEventEnabled("lifecycle")) {
			this.dispatchEvent(
				this.createChartEvent("chart-ready", {
					width: this.chartContainer.clientWidth,
					height: this.chartContainer.clientHeight,
					theme: this.currentTheme,
				}),
			);
		}

		// Set up event listeners (by6.10)
		this.setupEventListeners();
	}

	private ensureDimensions() {
		if (!this.chartContainer) return;

		// Inherit height from host element if container has none
		const hostHeight = this.clientHeight;
		if (hostHeight > 0 && this.chartContainer.clientHeight === 0) {
			this.chartContainer.style.height = hostHeight + "px";
		}

		// Fallback dimensions
		if (this.chartContainer.clientHeight === 0) {
			this.chartContainer.style.height = "400px";
		}
		if (this.chartContainer.clientWidth === 0) {
			this.chartContainer.style.width = "100%";
		}
	}

	updated(changedProperties: Map<string, unknown>) {
		if (!this.chart) return;

		// Handle option changes
		if (changedProperties.has("option")) {
			this.updateOption();
		}

		// Handle theme changes (requires chart reinit)
		if (changedProperties.has("theme") && this.theme !== this.currentTheme) {
			this.currentTheme = this.theme;
			this.chart.dispose();
			const themeArg =
				this.currentTheme === "default" ? null : this.currentTheme;
			this.chart = echarts.init(this.chartContainer!, themeArg, {
				renderer: this.renderer,
			});
			this.updateOption();
		}

		// Handle renderer changes (requires chart reinit)
		if (changedProperties.has("renderer")) {
			this.chart.dispose();
			const themeArg =
				this.currentTheme === "default" ? null : this.currentTheme;
			this.chart = echarts.init(this.chartContainer!, themeArg, {
				renderer: this.renderer,
			});
			this.updateOption();
		}
	}

	private updateOption() {
		// Ensure chart is initialized before updating
		if (!this.initialized) {
			this.initChart();
		}

		if (!this.chart) return;

		try {
			const parsed = JSON.parse(this.option || "{}");
			if (Object.keys(parsed).length > 0) {
				this.chart.setOption(parsed);

				// Emit chart-updated event (by6.6)
				if (this.isEventEnabled("lifecycle")) {
					this.dispatchEvent(
						this.createChartEvent("chart-updated", {
							timestamp: Date.now(),
						}),
					);
				}
			}
		} catch (e) {
			// Emit chart-error event (by6.9)
			if (this.isEventEnabled("lifecycle")) {
				this.dispatchEvent(
					this.createChartEvent("chart-error", {
						message: String(e),
						error: e instanceof Error ? e.name : "Error",
					}),
				);
			}
			console.error("[ds-echarts] Invalid JSON in option attribute:", e);
		}
	}

	private setupResizeObserver() {
		if (!this.chartContainer) return;

		this.resizeObserver = new ResizeObserver(() => {
			clearTimeout(this.resizeTimeout);
			this.resizeTimeout = window.setTimeout(() => {
				this.chart?.resize();

				// Emit chart-resized event (by6.7)
				if (this.chartContainer && this.isEventEnabled("lifecycle")) {
					this.dispatchEvent(
						this.createChartEvent("chart-resized", {
							width: this.chartContainer.clientWidth,
							height: this.chartContainer.clientHeight,
						}),
					);
				}
			}, this.resizeDelay);
		});

		this.resizeObserver.observe(this.chartContainer);
	}

	private setupMediaQueryListener() {
		const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

		this.mediaQueryHandler = () => {
			if (!this.chart || !this.chartContainer) return;

			// Only reinit if using default theme (which should respond to system preference)
			if (this.theme === "default") {
				this.chart.dispose();
				this.chart = echarts.init(this.chartContainer, null, {
					renderer: this.renderer,
				});
				this.updateOption();
			}
		};

		mediaQuery.addEventListener("change", this.mediaQueryHandler);
	}

	private setupEventListeners() {
		if (!this.chart) return;

		// Mouse events
		if (this.isEventEnabled("mouse")) {
			const mouseEvents = ["click", "dblclick", "contextmenu"];
			for (const eventName of mouseEvents) {
				const handler = (params: any) => {
					const sanitized = this.sanitizePayload(params);
					this.dispatchEvent(
						this.createChartEvent(`chart-${eventName}`, sanitized),
					);
				};
				this.chart.on(eventName, handler);
				this.boundHandlers.set(eventName, handler);
			}
		}

		// Hover events (separate category, throttled)
		if (this.isEventEnabled("hover")) {
			let lastHoverTime = 0;

			const hoverStartHandler = (params: any) => {
				const now = Date.now();
				if (now - lastHoverTime < this.hoverThrottle) return;
				lastHoverTime = now;
				const sanitized = this.sanitizePayload(params);
				this.dispatchEvent(
					this.createChartEvent("chart-hover-start", sanitized),
				);
			};
			this.chart.on("mouseover", hoverStartHandler);
			this.boundHandlers.set("mouseover", hoverStartHandler);

			const hoverEndHandler = (params: any) => {
				const now = Date.now();
				if (now - lastHoverTime < this.hoverThrottle) return;
				lastHoverTime = now;
				const sanitized = this.sanitizePayload(params);
				this.dispatchEvent(this.createChartEvent("chart-hover-end", sanitized));
			};
			this.chart.on("mouseout", hoverEndHandler);
			this.boundHandlers.set("mouseout", hoverEndHandler);
		}

		// Component events
		if (this.isEventEnabled("component")) {
			// legendselectchanged -> chart-legend-change
			const legendHandler = (params: any) => {
				const sanitized = this.sanitizePayload(params);
				this.dispatchEvent(
					this.createChartEvent("chart-legend-change", sanitized),
				);
			};
			this.chart.on("legendselectchanged", legendHandler);
			this.boundHandlers.set("legendselectchanged", legendHandler);

			// datazoom -> chart-datazoom
			const datazoomHandler = (params: any) => {
				const sanitized = this.sanitizePayload(params);
				this.dispatchEvent(this.createChartEvent("chart-datazoom", sanitized));
			};
			this.chart.on("datazoom", datazoomHandler);
			this.boundHandlers.set("datazoom", datazoomHandler);

			// selectchanged -> chart-select-change
			const selectHandler = (params: any) => {
				const sanitized = this.sanitizePayload(params);
				this.dispatchEvent(
					this.createChartEvent("chart-select-change", sanitized),
				);
			};
			this.chart.on("selectchanged", selectHandler);
			this.boundHandlers.set("selectchanged", selectHandler);

			// brush -> chart-brush
			const brushHandler = (params: any) => {
				const sanitized = this.sanitizePayload(params);
				this.dispatchEvent(this.createChartEvent("chart-brush", sanitized));
			};
			this.chart.on("brush", brushHandler);
			this.boundHandlers.set("brush", brushHandler);
		}
	}

	// Event infrastructure helpers

	private createChartEvent<T>(type: string, detail: T): CustomEvent<T> {
		return new CustomEvent(type, {
			bubbles: true,
			composed: true,
			detail,
		});
	}

	private sanitizePayload<T extends Record<string, unknown>>(
		payload: T,
	): Partial<T> {
		const result: Partial<T> = {};
		for (const [key, value] of Object.entries(payload)) {
			// Skip functions, DOM elements, and circular references
			if (typeof value === "function") continue;
			if (value instanceof Node) continue;
			if (value instanceof Event) continue;
			try {
				JSON.stringify(value);
				result[key as keyof T] = value as T[keyof T];
			} catch {
				// Skip non-serializable values
			}
		}
		return result;
	}

	private isEventEnabled(
		category: "lifecycle" | "mouse" | "component" | "hover",
	): boolean {
		return this.events
			.split(",")
			.map((s) => s.trim())
			.includes(category);
	}

	disconnectedCallback() {
		// Emit chart-disposed event (by6.8)
		if (this.chart && this.isEventEnabled("lifecycle")) {
			this.dispatchEvent(this.createChartEvent("chart-disposed", {}));
		}

		super.disconnectedCallback();

		// Disconnect resize observer
		this.resizeObserver?.disconnect();
		this.resizeObserver = null;

		// Clear resize timeout
		clearTimeout(this.resizeTimeout);

		// Remove media query listener
		if (this.mediaQueryHandler) {
			window
				.matchMedia("(prefers-color-scheme: dark)")
				.removeEventListener("change", this.mediaQueryHandler);
			this.mediaQueryHandler = null;
		}

		// Clean up bound event handlers
		if (this.chart) {
			for (const [eventName, handler] of this.boundHandlers) {
				this.chart.off(eventName, handler);
			}
			this.boundHandlers.clear();
		}

		// Dispose ECharts instance
		this.chart?.dispose();
		this.chart = null;
		this.initialized = false;
	}
}
