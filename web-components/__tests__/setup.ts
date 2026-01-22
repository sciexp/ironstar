import { vi } from 'vitest'

// Mock ECharts module with event handler tracking
const handlers = new Map<string, Set<Function>>()

export const mockChartInstance = {
  setOption: vi.fn(),
  resize: vi.fn(),
  dispose: vi.fn(),
  getDom: vi.fn(() => document.createElement('div')),

  // Event handler registration
  on: vi.fn((eventName: string, handler: Function) => {
    if (!handlers.has(eventName)) {
      handlers.set(eventName, new Set())
    }
    handlers.get(eventName)!.add(handler)
  }),

  // Event handler removal
  off: vi.fn((eventName: string, handler?: Function) => {
    if (!handler) {
      // Remove all handlers for this event
      handlers.delete(eventName)
    } else {
      // Remove specific handler
      const eventHandlers = handlers.get(eventName)
      if (eventHandlers) {
        eventHandlers.delete(handler)
        if (eventHandlers.size === 0) {
          handlers.delete(eventName)
        }
      }
    }
  }),

  // Test helper: get handlers for an event
  getHandlers: (eventName: string): Function[] => {
    const eventHandlers = handlers.get(eventName)
    return eventHandlers ? Array.from(eventHandlers) : []
  },

  // Test helper: clear all handlers
  clearHandlers: () => {
    handlers.clear()
  },
}

vi.mock('echarts', () => ({
  init: vi.fn(() => mockChartInstance),
  use: vi.fn(),
}))

// Mock clientWidth and clientHeight for happy-dom
// These are always 0 in happy-dom because it doesn't perform layout.
// The ds-echarts component requires non-zero dimensions to initialize,
// otherwise it retries indefinitely causing tests to hang.
Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
  configurable: true,
  get() {
    // Parse width from inline style if set, otherwise return default
    const style = this.style?.width
    if (style && style.endsWith('px')) {
      return parseInt(style, 10)
    }
    return 600 // Default test dimension
  },
})

Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
  configurable: true,
  get() {
    // Parse height from inline style if set, otherwise return default
    const style = this.style?.height
    if (style && style.endsWith('px')) {
      return parseInt(style, 10)
    }
    return 400 // Default test dimension
  },
})

// Store all ResizeObserver instances for manual triggering in tests
export const resizeObserverInstances: MockResizeObserver[] = []

// Mock ResizeObserver with working callback and manual trigger support
class MockResizeObserver implements ResizeObserver {
  private callback: ResizeObserverCallback
  private observedElements: Set<Element> = new Set()

  constructor(callback: ResizeObserverCallback) {
    this.callback = callback
    resizeObserverInstances.push(this)
  }

  observe(target: Element) {
    this.observedElements.add(target)
    // Simulate async resize observation by calling callback after a tick
    Promise.resolve().then(() => {
      this.triggerForElement(target)
    })
  }

  unobserve(target: Element) {
    this.observedElements.delete(target)
  }

  disconnect() {
    this.observedElements.clear()
    const index = resizeObserverInstances.indexOf(this)
    if (index > -1) {
      resizeObserverInstances.splice(index, 1)
    }
  }

  // Manual trigger for testing
  triggerForElement(target: Element) {
    if (!this.observedElements.has(target)) return

    const entries = [{
      target,
      contentRect: {
        width: (target as any).clientWidth || 100,
        height: (target as any).clientHeight || 100,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        x: 0,
        y: 0,
        toJSON: () => ({}),
      },
      borderBoxSize: [],
      contentBoxSize: [],
      devicePixelContentBoxSize: [],
    }] as ResizeObserverEntry[]
    this.callback(entries, this)
  }

  // Trigger resize for all observed elements
  triggerAll() {
    this.observedElements.forEach(target => {
      this.triggerForElement(target)
    })
  }

  // Check if an element is being observed
  isObserving(target: Element): boolean {
    return this.observedElements.has(target)
  }
}

global.ResizeObserver = MockResizeObserver

// Mock matchMedia for theme detection tests
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})
