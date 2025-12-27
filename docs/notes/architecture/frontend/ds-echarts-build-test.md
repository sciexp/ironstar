# ds-echarts Build and Testing

This document covers the TypeScript/Rolldown build configuration and Vitest testing setup for ds-echarts integration.

See `ds-echarts-integration-guide.md` for the complete integration overview and `frontend-build-pipeline.md` for common build configuration patterns.

## Package Configuration

### package.json

```json
{
  "name": "ironstar-web-components",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "rolldown --config --watch",
    "build": "rolldown --config",
    "test": "vitest",
    "test:coverage": "vitest --coverage",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "echarts": "^5.5.0",
    "lit": "^3.3.1"
  },
  "devDependencies": {
    "@types/node": "^22.0.0",
    "@vitest/coverage-v8": "^2.1.8",
    "autoprefixer": "^10.4.0",
    "cssnano": "^7.0.0",
    "happy-dom": "^15.11.0",
    "postcss": "^8.4.0",
    "postcss-import": "^16.0.0",
    "postcss-preset-env": "^10.0.0",
    "rolldown": "^1.0.0",
    "typescript": "^5.9.0",
    "vitest": "^2.1.8"
  }
}
```

### tsconfig.json

```json
{
  "compilerOptions": {
    "experimentalDecorators": true,
    "useDefineForClassFields": false,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "target": "ES2020",
    "strict": true,
    "skipLibCheck": true,
    "declaration": false,
    "outDir": "dist"
  },
  "include": ["**/*.ts"],
  "exclude": ["node_modules", "dist", "__tests__"]
}
```

## Rolldown Configuration

See `frontend-build-pipeline.md` for the complete Rolldown configuration pattern used across ironstar.
The ds-echarts-specific configuration follows the common pattern with manifest generation.

### rolldown.config.ts

```typescript
import { defineConfig } from 'rolldown';
import postcss from 'rolldown-plugin-postcss';

export default defineConfig({
  input: {
    bundle: 'index.ts',
  },
  output: {
    dir: '../static/dist',
    format: 'esm',
    entryFileNames: '[name].[hash].js',
    chunkFileNames: '[name].[hash].js',
    assetFileNames: '[name].[hash][extname]',
    hashCharacters: 'base36',
    sourcemap: process.env.NODE_ENV === 'production' ? 'hidden' : true,
  },
  plugins: [
    postcss({
      config: './postcss.config.js',
      extract: 'bundle.css',
      minimize: process.env.NODE_ENV === 'production',
    }),
    manifestPlugin(),
  ],
  treeshake: {
    moduleSideEffects: 'no-external',
  },
});

function manifestPlugin() {
  return {
    name: 'manifest-generator',
    generateBundle(options, bundle) {
      const manifest: Record<string, { file: string; css?: string[] }> = {};

      for (const [fileName, asset] of Object.entries(bundle)) {
        if (asset.type === 'chunk' && asset.isEntry) {
          const entry = asset.name || fileName;
          manifest[entry] = { file: fileName };
        }
      }

      // Link CSS to entry
      for (const [fileName, asset] of Object.entries(bundle)) {
        if (asset.type === 'asset' && fileName.endsWith('.css')) {
          const entryName = fileName.replace(/\.[a-z0-9]+\.css$/, '');
          if (manifest[entryName]) {
            manifest[entryName].css = [fileName];
          } else {
            manifest['styles'] = { file: fileName };
          }
        }
      }

      this.emitFile({
        type: 'asset',
        fileName: 'manifest.json',
        source: JSON.stringify(manifest, null, 2),
      });
    },
  };
}
```

### postcss.config.js

```javascript
export default {
  plugins: {
    'postcss-import': {},
    'postcss-preset-env': {
      stage: 0,
      features: {
        'oklab-function': true,
        'light-dark-function': true,
        'custom-media-queries': true,
        'nesting-rules': true,
      },
    },
    autoprefixer: {},
    ...(process.env.NODE_ENV === 'production' ? { cssnano: { preset: 'default' } } : {}),
  },
};
```

## CSS Entry Point

### styles/main.css

```css
/* Layer 1: Open Props design tokens */
@import 'open-props/postcss/style';

/* Layer 2: Open Props UI normalize */
@import './normalize.css';

/* Layer 3: Theme overrides */
@import './theme.css';

/* Layer 4: Copied Open Props UI components (owned) */
@import './components/button.css';
@import './components/card.css';
/* Add more as needed */
```

### styles/theme.css

```css
@layer theme {
  :where(html) {
    color-scheme: light dark;

    /* Primary color using Open Props hues */
    --palette-hue: var(--oklch-blue);
    --palette-chroma: 0.2;

    /* Semantic tokens */
    --primary: var(--blue-7);
    --primary-contrast: var(--gray-0);
    --surface-default: light-dark(var(--gray-1), var(--gray-13));
    --surface-elevated: light-dark(var(--gray-0), var(--gray-12));
    --text-color-1: light-dark(var(--gray-15), var(--gray-1));
    --border-color: light-dark(var(--gray-4), var(--gray-11));

    /* Chart-specific tokens */
    --chart-height: 400px;
    --chart-min-height: 200px;
  }
}
```

## Testing Setup

### vitest.config.ts

```typescript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'happy-dom',
    setupFiles: ['./__tests__/setup.ts'],
    include: ['**/__tests__/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['components/**/*.ts'],
    },
  },
});
```

### __tests__/setup.ts

```typescript
import { vi } from 'vitest';

// Mock ECharts to avoid canvas rendering in tests
const mockChartInstance = {
  handlers: new Map<string, Function[]>(),

  setOption: vi.fn(),
  resize: vi.fn(),
  dispose: vi.fn(),

  on(eventName: string, handler: Function) {
    if (!this.handlers.has(eventName)) {
      this.handlers.set(eventName, []);
    }
    this.handlers.get(eventName)!.push(handler);
  },

  off(eventName: string, handler: Function) {
    const handlers = this.handlers.get(eventName);
    if (handlers) {
      const index = handlers.indexOf(handler);
      if (index > -1) handlers.splice(index, 1);
    }
  },

  getHandlers(eventName: string): Function[] {
    return this.handlers.get(eventName) || [];
  },

  reset() {
    this.handlers.clear();
    this.setOption.mockClear();
    this.resize.mockClear();
    this.dispose.mockClear();
  },
};

vi.mock('echarts', () => ({
  init: vi.fn(() => mockChartInstance),
  dispose: vi.fn(),
}));

export { mockChartInstance };
```

### ResizeObserver mock

Browser APIs like ResizeObserver need mocking for chart resize behavior tests.

```typescript
// __tests__/resize-observer-mock.ts
import { vi } from 'vitest';

export class MockResizeObserver {
  private callback: ResizeObserverCallback;
  private observations = new Set<Element>();

  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
  }

  observe(target: Element): void {
    this.observations.add(target);
  }

  unobserve(target: Element): void {
    this.observations.delete(target);
  }

  disconnect(): void {
    this.observations.clear();
  }

  triggerResize(width: number, height: number): void {
    const entries: ResizeObserverEntry[] = Array.from(this.observations).map(target => ({
      target,
      contentRect: { width, height, x: 0, y: 0, top: 0, left: 0, bottom: height, right: width },
      borderBoxSize: [{ inlineSize: width, blockSize: height }],
      contentBoxSize: [{ inlineSize: width, blockSize: height }],
      devicePixelContentBoxSize: [{ inlineSize: width, blockSize: height }],
    })) as ResizeObserverEntry[];

    this.callback(entries, this);
  }
}

export function setupResizeObserverMock(): MockResizeObserver {
  let instance: MockResizeObserver;

  global.ResizeObserver = vi.fn((callback) => {
    instance = new MockResizeObserver(callback);
    return instance;
  }) as any;

  return instance;
}
```

Example test using ResizeObserver mock:

```typescript
// __tests__/ds-echarts-resize.test.ts
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { setupResizeObserverMock } from './resize-observer-mock';
import { render, cleanup } from './test-utils';
import { mockChartInstance } from './setup';
import '../components/ds-echarts';

describe('ds-echarts resize behavior', () => {
  let resizeObserver: ReturnType<typeof setupResizeObserverMock>;

  beforeEach(() => {
    resizeObserver = setupResizeObserverMock();
    mockChartInstance.reset();
  });

  afterEach(() => {
    cleanup();
  });

  it('should resize chart when container size changes', async () => {
    const element = await render<any>('ds-echarts');

    expect(mockChartInstance.resize).not.toHaveBeenCalled();

    // Simulate container resize
    resizeObserver.triggerResize(800, 600);

    expect(mockChartInstance.resize).toHaveBeenCalledOnce();
  });
});
```

### MediaQueryList mock

Theme changes detected via `window.matchMedia` require mocking for dark/light mode tests.

```typescript
// __tests__/media-query-mock.ts
import { vi } from 'vitest';

export class MockMediaQueryList implements MediaQueryList {
  matches: boolean;
  media: string;
  private listeners = new Set<(event: MediaQueryListEvent) => void>();

  constructor(query: string, matches: boolean = false) {
    this.media = query;
    this.matches = matches;
  }

  addEventListener(_type: 'change', listener: (event: MediaQueryListEvent) => void): void {
    this.listeners.add(listener);
  }

  removeEventListener(_type: 'change', listener: (event: MediaQueryListEvent) => void): void {
    this.listeners.delete(listener);
  }

  addListener(listener: (event: MediaQueryListEvent) => void): void {
    this.addEventListener('change', listener);
  }

  removeListener(listener: (event: MediaQueryListEvent) => void): void {
    this.removeEventListener('change', listener);
  }

  dispatchEvent(_event: Event): boolean {
    return true;
  }

  onchange: ((this: MediaQueryList, ev: MediaQueryListEvent) => any) | null = null;

  setMatches(matches: boolean): void {
    const changed = this.matches !== matches;
    this.matches = matches;

    if (changed) {
      const event = { matches, media: this.media } as MediaQueryListEvent;
      this.listeners.forEach(listener => listener(event));
      if (this.onchange) this.onchange.call(this, event);
    }
  }
}

export function setupMediaQueryMock(): { dark: MockMediaQueryList; light: MockMediaQueryList } {
  const darkQuery = new MockMediaQueryList('(prefers-color-scheme: dark)', false);
  const lightQuery = new MockMediaQueryList('(prefers-color-scheme: light)', true);

  global.window.matchMedia = vi.fn((query: string) => {
    if (query.includes('dark')) return darkQuery;
    if (query.includes('light')) return lightQuery;
    return new MockMediaQueryList(query);
  });

  return { dark: darkQuery, light: lightQuery };
}
```

Example test using MediaQueryList mock:

```typescript
// __tests__/ds-echarts-theme.test.ts
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { setupMediaQueryMock } from './media-query-mock';
import { render, cleanup } from './test-utils';
import { mockChartInstance } from './setup';
import '../components/ds-echarts';

describe('ds-echarts theme changes', () => {
  let mediaQuery: ReturnType<typeof setupMediaQueryMock>;

  beforeEach(() => {
    mediaQuery = setupMediaQueryMock();
    mockChartInstance.reset();
  });

  afterEach(() => {
    cleanup();
  });

  it('should dispose and reinitialize chart when theme changes', async () => {
    const element = await render<any>('ds-echarts');

    expect(mockChartInstance.dispose).not.toHaveBeenCalled();

    // Toggle from light to dark mode
    mediaQuery.light.setMatches(false);
    mediaQuery.dark.setMatches(true);

    // Chart should be disposed and recreated
    expect(mockChartInstance.dispose).toHaveBeenCalledOnce();
  });
});
```

### Datastar signal binding tests

Test pattern for verifying chart option updates when Datastar signals change.

```typescript
// __tests__/ds-echarts-signals.test.ts
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { renderWithAttrs, waitForLitUpdate, cleanup } from './test-utils';
import { mockChartInstance } from './setup';
import '../components/ds-echarts';

describe('ds-echarts Datastar signal binding', () => {
  beforeEach(() => {
    mockChartInstance.reset();
  });

  afterEach(() => {
    cleanup();
  });

  it('should call setOption when data-attr:option changes', async () => {
    const initialOptions = JSON.stringify({
      xAxis: { type: 'category', data: ['A', 'B', 'C'] },
      yAxis: { type: 'value' },
      series: [{ type: 'bar', data: [10, 20, 30] }],
    });

    const element = await renderWithAttrs<any>('ds-echarts', {
      'data-attr:option': initialOptions,
    });

    expect(mockChartInstance.setOption).toHaveBeenCalledOnce();
    const firstCall = mockChartInstance.setOption.mock.calls[0][0];
    expect(firstCall.series[0].data).toEqual([10, 20, 30]);

    // Simulate Datastar signal update
    const updatedOptions = JSON.stringify({
      xAxis: { type: 'category', data: ['A', 'B', 'C'] },
      yAxis: { type: 'value' },
      series: [{ type: 'bar', data: [15, 25, 35] }],
    });

    element.setAttribute('data-attr:option', updatedOptions);
    await waitForLitUpdate(element);

    expect(mockChartInstance.setOption).toHaveBeenCalledTimes(2);
    const secondCall = mockChartInstance.setOption.mock.calls[1][0];
    expect(secondCall.series[0].data).toEqual([15, 25, 35]);
  });

  it('should handle malformed JSON gracefully', async () => {
    const element = await renderWithAttrs<any>('ds-echarts', {
      'data-attr:option': 'not valid json',
    });

    // Should not throw, chart remains uninitialized
    expect(mockChartInstance.setOption).not.toHaveBeenCalled();
  });
});
```

### __tests__/test-utils.ts

```typescript
export async function render<T extends HTMLElement>(tagName: string): Promise<T> {
  const element = document.createElement(tagName) as T;
  document.body.appendChild(element);
  await element.updateComplete;
  return element;
}

export async function renderWithAttrs<T extends HTMLElement>(
  tagName: string,
  attrs: Record<string, string>,
): Promise<T> {
  const element = document.createElement(tagName) as T;
  for (const [key, value] of Object.entries(attrs)) {
    element.setAttribute(key, value);
  }
  document.body.appendChild(element);
  await element.updateComplete;
  return element;
}

export async function waitForLitUpdate(element: HTMLElement): Promise<void> {
  await (element as any).updateComplete;
}

export function cleanup(): void {
  document.body.innerHTML = '';
}
```

## Data Flow Summary

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Initial Page Load                                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│ Browser                         │  Server                                   │
│                                 │                                           │
│ 1. Load page                    │                                           │
│    data-on-load="@get(...)"    ─┼──▶ SSE handler                            │
│                                 │     │                                     │
│                                 │     ▼                                     │
│                                 │  2. Query DuckDB for chart data           │
│                                 │     │                                     │
│                                 │     ▼                                     │
│ 3. Receive PatchElements       ◀┼──  3. Render hypertext template           │
│    ds-echarts inserted          │     Send PatchElements                    │
│                                 │     │                                     │
│ 4. ds-echarts initializes       │     ▼                                     │
│    ECharts instance             │  4. Subscribe to Zenoh for updates        │
│                                 │                                           │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ User Interaction                                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│ Browser                         │  Server                                   │
│                                 │                                           │
│ 1. User clicks chart bar        │                                           │
│    chart-click event fired      │                                           │
│    │                            │                                           │
│    ▼                            │                                           │
│ 2. data-on:chart-click=         │                                           │
│    "$selected = evt.detail"     │                                           │
│    "@post('/api/chart/select')"─┼──▶ POST handler                           │
│                                 │     │                                     │
│                                 │     ▼                                     │
│                                 │  3. Query DuckDB for drill-down           │
│                                 │     │                                     │
│                                 │     ▼                                     │
│                                 │  4. Publish to Zenoh                      │
│                                 │     (other SSE handlers receive)          │
│                                 │     │                                     │
│ 5. Receive PatchSignals        ◀┼──   ▼                                     │
│    $detailData updated          │  5. Return PatchSignals                   │
│                                 │                                           │
│ 6. UI updates reactively        │                                           │
│    via Datastar bindings        │                                           │
│                                 │                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Related Documentation

- **Integration overview**: `ds-echarts-integration-guide.md` — Component properties, hypertext templates, critical notes
- **Backend integration**: `ds-echarts-backend.md` — Axum handlers, DuckDB service, Zenoh event bus
- **Frontend build pipeline**: `frontend-build-pipeline.md` — Common Rolldown and PostCSS configuration patterns
- **Development workflow**: `../infrastructure/development-workflow.md` — process-compose orchestration, hot reload, asset serving modes
