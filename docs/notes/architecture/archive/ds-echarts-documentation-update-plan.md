# ds-echarts Documentation Update Plan

This document synthesizes the findings from a comprehensive audit of ironstar's architecture documentation to identify all integration points that need updates for complete ds-echarts Lit component integration.

## Executive Summary

The ds-echarts integration requires updates to **7 documents** plus CLAUDE.md, touching **4 documentation layers** (foundational, architectural, technical, integration).
The newly created `ds-echarts-integration-guide.md` serves as the primary implementation reference, while existing documents need targeted updates to maintain architectural coherence.

## Priority Classification

| Priority | Document | Impact | Effort |
|----------|----------|--------|--------|
| P0 (Critical) | CLAUDE.md | Conceptual clarity, removes contradiction | Medium |
| P0 (Critical) | integration-patterns.md | Pattern 1.5 canonical example | Low |
| P1 (High) | frontend-build-pipeline.md | Lit bundling section | Medium |
| P1 (High) | signal-contracts.md | ECharts signal types | Low |
| P2 (Medium) | development-workflow.md | Process-compose frontend | Low |
| P2 (Medium) | architecture-decisions.md | Light DOM note | Low |
| P3 (Low) | event-sourcing-sse-pipeline.md | Chart SSE example | Low |

## Detailed Update Specifications

### 1. CLAUDE.md (Root Documentation)

**Location:** `/CLAUDE.md`

#### 1.1 Stack Overview Table (Lines ~167-185)

**Current:** No visualization layer documented

**Update:** Add row between "Icons" and "Web Components":

```markdown
| Visualization | ECharts + ds-echarts | Interactive charts via Lit wrapper |
```

#### 1.2 Frontend Tooling Philosophy - "What we avoid" (Lines ~401-408)

**Current:** Lists "Lit | Redundant reactivity (datastar already provides this)"

**Update:** Replace with nuanced guidance:

```markdown
| Lit (for simple components) | Redundant reactivity (datastar already provides this) |
```

Add clarifying note after table:

```markdown
**Lit exception:** Use Lit only for complex third-party library integration (e.g., ECharts via ds-echarts) where the library manages significant internal state that datastar signals cannot directly control. See Pattern 1.5 in `docs/notes/architecture/integration-patterns.md`.
```

#### 1.3 Frontend Tooling Philosophy - "What we use" (Lines ~380-389)

**Update:** Add Lit row (conditional):

```markdown
| Lit (conditional) | Complex lifecycle management for charts (ECharts) |
```

#### 1.4 Project Structure - web-components/ (Lines ~654-681)

**Update:** Reorganize to show Lit components:

```
├── web-components/
│   ├── components/
│   │   ├── vanilla/              # Simple state (sortable-list)
│   │   │   └── vega-chart.ts
│   │   └── lit/                  # Complex lifecycle (ds-echarts)
│   │       └── ds-echarts.ts
```

#### 1.5 Visualization Libraries Table (Lines ~338-344)

**Update:** Add ds-echarts row:

```markdown
| ds-echarts | `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/libs/lit/src/components/ds-echarts/` | Lit component wrapping ECharts |
```

#### 1.6 Related Documentation Section (Lines ~683-722)

**Update:** Add ds-echarts guide reference:

```markdown
- ECharts integration: `docs/notes/architecture/ds-echarts-integration-guide.md`
  - Lit component lifecycle management
  - Rolldown bundling configuration
  - Hypertext template patterns
  - Axum SSE handlers with DuckDB
```

#### 1.7 Intentional Divergences from Northstar (Lines ~133-147)

**Update:** Add row explaining ECharts adoption:

```markdown
| ds-echarts Lit component | Adopt as-is | Complex chart state requires Lit lifecycle hooks |
```

---

### 2. integration-patterns.md

**Location:** `docs/notes/architecture/integration-patterns.md`

#### 2.1 Pattern 1.5: When Lit is Appropriate

**Current:** Section exists but may lack canonical ECharts example

**Update:** Ensure section includes:

```markdown
## Pattern 1.5: When Lit is appropriate

Use Lit instead of vanilla web components when ALL of these apply:

1. Library manages significant internal state (ECharts scales, animations)
2. Multiple lifecycle observers require coordination (ResizeObserver + MediaQueryList)
3. Light DOM is acceptable (required for Open Props token inheritance)
4. Lit's reactivity is isolated to component internals (not competing with Datastar)

### Canonical example: ds-echarts

The ds-echarts component demonstrates this pattern:

- **ResizeObserver** with debouncing for responsive chart sizing
- **MediaQueryList** listener for dark mode theme switching
- **ECharts lifecycle** (init → setOption → resize → dispose)
- **Custom events** bridging ECharts interactions to Datastar signals

See `ds-echarts-integration-guide.md` for complete implementation details.
```

#### 2.2 Decision Matrix: Vega vs. ECharts vs. Mosaic

**Update:** Add section after Pattern 3:

```markdown
## Visualization library decision matrix

| Criterion | Vega-Lite | ECharts | Mosaic |
|-----------|-----------|---------|--------|
| Wrapper pattern | Pattern 2 (vanilla) | Pattern 1.5 (Lit) | TBD |
| Build complexity | Low | Medium | Medium |
| Declarative spec | Yes (JSON) | Yes (JSON) | Yes (grammar) |
| Complex animations | Limited | Excellent | Good |
| Coordinated views | Limited | Limited | Excellent |
| Bundle size | ~200KB | ~800KB (full) | ~400KB |
| Real-time updates | View API | setOption merge | TBD |

**Recommendation:**
- **Vega-Lite:** Simple declarative charts, specification-driven
- **ECharts:** Complex interactions, rich animations, real-time dashboards
- **Mosaic:** Large datasets, coordinated multi-view analysis
```

---

### 3. frontend-build-pipeline.md

**Location:** `docs/notes/architecture/frontend-build-pipeline.md`

#### 3.1 Lit Component Bundling Section

**Update:** Add new section:

```markdown
## Lit component bundling

### Rolldown configuration for Lit

Lit components require TypeScript decorator support:

```typescript
// rolldown.config.ts
export default defineConfig({
  input: 'index.ts',
  output: {
    dir: '../static/dist',
    format: 'esm',
    entryFileNames: '[name].[hash].js',
  },
});
```

```json
// tsconfig.json
{
  "compilerOptions": {
    "experimentalDecorators": true,
    "useDefineForClassFields": false
  }
}
```

### Light DOM requirement

All Lit components using Open Props tokens must render to Light DOM:

```typescript
protected createRenderRoot() {
  return this  // Light DOM, not Shadow DOM
}
```

Shadow DOM would block CSS custom property inheritance from the page.

### Alternative: esbuild (proven pattern)

If Rolldown Lit support is insufficient, use esbuild for components:

```json
{
  "scripts": {
    "build:lit": "esbuild src/index.ts --bundle --format=esm --outdir=dist"
  }
}
```

This is the pattern used in northstar and is battle-tested.
```

---

### 4. signal-contracts.md

**Location:** `docs/notes/architecture/signal-contracts.md`

#### 4.1 ECharts Signal Types Example

**Update:** Add section:

```markdown
## Chart signal contracts

### ECharts configuration signals

```rust
#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../web-components/types/")]
pub struct ChartSignals {
    /// ECharts option object
    #[serde(rename = "chartOption")]
    pub chart_option: serde_json::Value,

    /// Selected data point from chart-click event
    #[ts(optional)]
    pub selected: Option<ChartSelection>,

    /// Loading state for UI feedback
    #[serde(default)]
    pub loading: bool,
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChartSelection {
    #[serde(rename = "seriesName")]
    pub series_name: String,
    #[serde(rename = "dataIndex")]
    pub data_index: i32,
    pub name: String,
    pub value: serde_json::Value,
}
```

### Generated TypeScript

```typescript
export type ChartSignals = {
  chartOption: unknown;  // ECharts option
  selected?: ChartSelection;
  loading: boolean;
};

export type ChartSelection = {
  seriesName: string;
  dataIndex: number;
  name: string;
  value: unknown;
};
```

These types ensure handler `ReadSignals<ChartSelection>` matches browser event payloads.
```

---

### 5. development-workflow.md

**Location:** `docs/notes/architecture/development-workflow.md`

#### 5.1 Process-Compose Frontend Build

**Update:** Add to process-compose section:

```markdown
### Frontend asset build

```yaml
# process-compose.yaml
processes:
  frontend:
    command: pnpm --dir web-components dev
    availability:
      restart: on_failure
    depends_on:
      typegen:
        condition: process_completed_successfully
```

The `pnpm dev` command runs Rolldown in watch mode, rebuilding on TypeScript/CSS changes.

### TypeScript generation for chart signals

Chart signal types are generated alongside other signals:

```bash
cargo test --lib  # Generates web-components/types/ChartSignals.ts
```
```

---

### 6. architecture-decisions.md

**Location:** `docs/notes/architecture/architecture-decisions.md`

#### 6.1 Light DOM Note in Open Props UI Section

**Update:** Add note to section 10 (open-props-ui):

```markdown
**Light DOM requirement:** All web components using Open Props tokens must use Light DOM rendering. This applies to:
- ds-echarts (Lit component)
- vega-chart (vanilla component)
- Any future visualization components

Shadow DOM blocks CSS custom property inheritance, breaking theme token access.
```

---

### 7. event-sourcing-sse-pipeline.md

**Location:** `docs/notes/architecture/event-sourcing-sse-pipeline.md`

#### 7.1 Chart Data SSE Example

**Update:** Add to "Handler Patterns" section:

```markdown
### Chart data streaming

Charts receive data via SSE using the same patterns as other projections:

```rust
async fn chart_data_sse(
    State(state): State<Arc<AppState>>,
    Path(chart_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        // Initial chart data from DuckDB
        let data = state.analytics.query_chart_data(&chart_id).await?;
        let option = build_echarts_option(&data);

        let signals = serde_json::json!({
            "chartOption": option,
            "loading": false,
        });

        yield Ok(PatchSignals::new(signals.to_string())
            .write_as_axum_sse_event());

        // Subscribe to real-time updates via Zenoh
        let mut sub = state.event_bus
            .subscribe(&format!("charts/{}/updates", chart_id))
            .await;

        while let Some(update) = sub.next().await {
            let new_option = build_echarts_option(&update);
            yield Ok(PatchSignals::new(
                serde_json::json!({"chartOption": new_option}).to_string()
            ).write_as_axum_sse_event());
        }
    };

    Sse::new(stream)
}
```

This pattern streams initial chart configuration followed by incremental updates.
```

---

## Documentation Graph After Updates

```
┌─────────────────────────────────────────────────────────────────────┐
│                           CLAUDE.md                                  │
│  (Stack overview, tooling philosophy, project structure)            │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────┐    ┌───────────────────┐    ┌─────────────────────┐
│ Foundational  │    │   Technical       │    │    Integration      │
├───────────────┤    ├───────────────────┤    ├─────────────────────┤
│ design-       │    │ event-sourcing-   │    │ integration-        │
│ principles    │    │ sse-pipeline      │───▶│ patterns            │
│               │    │ ★ Chart SSE ex.   │    │ ★ Pattern 1.5       │
│ architecture- │    │                   │    │ ★ Decision matrix   │
│ decisions     │    │ analytics-cache   │    │                     │
│ ★ Light DOM   │    │                   │    │ signal-contracts    │
│               │    │ frontend-build-   │    │ ★ Chart signals     │
│               │    │ pipeline          │    │                     │
│               │    │ ★ Lit bundling    │    │ ds-echarts-         │
│               │    │                   │    │ integration-guide   │
│               │    │ development-      │    │ (complete)          │
│               │    │ workflow          │    │                     │
│               │    │ ★ Frontend build  │    │                     │
└───────────────┘    └───────────────────┘    └─────────────────────┘
```

★ = Updates required for ds-echarts integration

---

## Implementation Sequence

### Phase 1: Conceptual Clarity (P0)

1. **CLAUDE.md:** Update "What we avoid" to remove blanket Lit rejection
2. **CLAUDE.md:** Add Stack Overview visualization layer
3. **integration-patterns.md:** Ensure Pattern 1.5 has canonical ECharts example

### Phase 2: Technical Details (P1)

4. **frontend-build-pipeline.md:** Add Lit bundling section
5. **signal-contracts.md:** Add ECharts signal type examples
6. **CLAUDE.md:** Update project structure with lit/ directory

### Phase 3: Supporting Documentation (P2-P3)

7. **development-workflow.md:** Add process-compose frontend build
8. **architecture-decisions.md:** Add Light DOM requirement note
9. **event-sourcing-sse-pipeline.md:** Add chart SSE handler example
10. **CLAUDE.md:** Update related documentation links

---

## Validation Checklist

After updates, verify:

- [ ] No contradiction between "What we avoid" and ds-echarts integration
- [ ] Pattern 1.5 clearly explains when Lit is appropriate
- [ ] Lit bundling configuration is documented
- [ ] Chart signal types show ts-rs pattern
- [ ] Project structure reflects lit/ component directory
- [ ] Decision matrix helps choose between Vega/ECharts/Mosaic
- [ ] SSE handler example shows chart data streaming
- [ ] All cross-references resolve to existing documents
