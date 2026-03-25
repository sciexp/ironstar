# ECharts vs ChartGPU evaluation

Architectural comparison and selection guide for the two charting engines available in `~/projects/lakescope-workspace/`.
For ECharts integration patterns within ironstar, see `ds-echarts-integration-guide.md` and `integration-patterns-visualizations.md`.
For the high-performance visualization tooling evaluation covering Deck.gl, Three.js, and Lyr.jl, see `~/projects/sciexp/planning/docs/notes/research/high-performance-visualization-tooling.md`.

## Source repositories

| Library | Path | Version | License |
|---------|------|---------|---------|
| Apache ECharts | `~/projects/lakescope-workspace/echarts` | 6.0.0 | Apache 2.0 |
| ChartGPU | `~/projects/lakescope-workspace/ChartGPU` | 0.3.2 | MIT |

## Rendering architecture

ECharts is a *declarative option to pipeline to scene graph* system.
A JSON option object passes through an 8-stage priority-ordered pipeline (data restore, series tasks, coordinate system creation, data processing, stream mode selection, coordinate system update, visual encoding, render) that constructs a zrender scene graph of `Element` objects.
ECharts never touches pixels directly; zrender owns all Canvas2D and SVG rendering as an interchangeable backend.

ChartGPU is a *functional factory to GPU buffer to multi-pass render* system.
Factory functions (`createDataStore`, `createLineRenderer`, `createRenderCoordinator`) return interface objects with closure-encapsulated state.
Data flows through typed-array packing, GPU buffer upload, and a 3-pass render pipeline: main scene at 4x MSAA, annotation overlay at 4x MSAA, and UI overlay at 1x sample.
There is no scene graph or intermediate representation; each series type has a dedicated WGSL shader that reads storage buffers and emits vertices directly.

The scene graph vs. direct GPU buffer distinction is the deepest difference.
ECharts builds an intermediate representation (zrender elements) that a Canvas2D or SVG painter re-traverses each frame.
ChartGPU eliminates the IR entirely, sending data from `Float32Array` to GPU storage buffer to vertex shader output.
This is why ChartGPU scales to 50M points: no per-element object allocation, no scene graph traversal, no CPU-side per-point draw calls.

## Data model

| Aspect | ECharts | ChartGPU |
|--------|---------|----------|
| Internal storage | Column-oriented typed arrays (`Float64Array`) in `DataStore` | Interleaved `Float32Array` packed for GPU upload |
| Numeric precision | Float64 | Float32 with `xOffset` subtraction for large magnitudes |
| Input formats | 5 (original, arrayRows, objectRows, keyedColumns, typedArray) | 3 (`DataPoint[]`, columnar `XYArraysData`, interleaved `ArrayBufferView`) |
| Change detection | None; always re-processes on `setOption` | FNV-1a hash of IEEE-754 bit patterns; skips upload when unchanged |
| Streaming | Re-provide full dataset via `setOption` | `appendData()` writes only new bytes to existing GPU buffer |
| Filtering | Index indirection array (no data copy) | Binary search visible x-range, upload and render only visible slice |

## Data processing

ECharts has a rich CPU-side processing pipeline with pluggable data transforms, stacking, sampling, and negative filtering, executed as prioritized stage handlers in the Scheduler.
ChartGPU performs minimal CPU processing (LTTB downsampling with dual fast paths for typed arrays vs. objects, plus optional average/max/min/OHLC sampling) and delegates heavy computation to GPU compute shaders (scatter density binning via `atomicAdd` in workgroup_size=256 compute shaders).

## Anti-aliasing

ECharts inherits whatever Canvas2D or SVG provides natively.
ChartGPU implements analytical anti-aliasing via signed distance fields in every series shader (`fwidth()` + `smoothstep()` for edge feathering) plus 4x MSAA at the framebuffer level.

## Chart type breadth and extensibility

ECharts ships 22 built-in chart types plus a `custom` type with `renderItem` callbacks, 7 coordinate systems (cartesian, polar, geo, singleAxis, parallel, calendar, matrix), and a comprehensive `use()` / `install` extension API enabling tree-shaking.
Every chart type, component, coordinate system, data transform, and rendering backend is pluggable.

ChartGPU has 6 series types (line, area, bar, scatter, pie, candlestick) with no user extension API.
Each type is a hard-coded renderer+shader pair.
Adding a chart type requires writing both a TypeScript renderer factory and a WGSL shader.

## Type system design

| Pattern | ECharts | ChartGPU |
|---------|---------|----------|
| Language target | ES3 with tslib | Modern ESM |
| Paradigm | Deep class hierarchies with `enableClassExtend` | Factory functions returning interfaces |
| Union modeling | String-discriminated `type` field for component lookup | Branded numeric types (`ExactFPS`, `Milliseconds`, `Bytes`) and discriminated unions |
| Option typing | Generic models (`ComponentModel<Opt>`) with `ComposeOption<T>` conditional type | `readonly` interfaces resolved by `OptionResolver` |
| Encapsulation | `makeInner<T, Host>()` for type-safe private slots | Closure encapsulation in factory functions |

ChartGPU's branded numeric types create compile-time dimensional analysis, analogous to Rust newtypes.
ECharts, targeting ES3, uses string-based discrimination instead.

## Performance strategies

| Strategy | ECharts | ChartGPU |
|----------|---------|----------|
| Large dataset mode | `large: true` batches all symbols into single Canvas2D path | All modes GPU-native; no mode switch |
| Progressive rendering | Frame-budgeted pipeline (1ms per frame) with linked-list task chains | Not needed; GPU parallelism renders all data per frame |
| Buffer management | CPU-side only | Geometric growth, double-buffered `StreamBuffer` with diff-based partial writes |
| Resource sharing | `echarts.connect()` for interaction sync | Shared `GPUDevice` + `PipelineCache` with FNV-1a shader module dedup |
| Dirty optimization | zrender `useDirtyRect` repaints only invalidated canvas regions | Dirty-flag coalescing + external render mode for dashboard-level rAF control |

## DOM boundary

ECharts renders everything through the scene graph: text, axes, legends, and tooltips are all zrender elements drawn on canvas or SVG.
ChartGPU draws only series data and grid lines on the GPU; axis labels, tooltips, legends, and annotation labels are HTML DOM overlays.
This is a pragmatic choice since text rendering on the GPU is hard, and HTML text is already resolution-independent and accessible.

## Gap handling

ECharts handles data gaps through its data model (null/undefined values in `SeriesData`).
ChartGPU detects gaps in WGSL shaders via the NaN self-inequality pattern (`pA_data.x != pA_data.x`) since WGSL lacks `isnan()`.
Gaps are encoded as NaN in the Float32 buffer and handled at the GPU level without CPU-side special casing.

## ECharts interop

There is no compatibility or interop layer between the two libraries.
ChartGPU's API shows ECharts influence (OHLC tuple ordering matches ECharts convention, option structure follows ECharts naming patterns) but it is not a drop-in replacement.
There is no option translation layer, adapter pattern, or shared type system.

## Selection guide for ironstar and lakescope

Use ECharts when the visualization needs breadth: diverse chart types (treemap, sankey, radar, gauge, geo), rich interaction patterns (brush selection, visual map, data zoom), progressive rendering for graceful degradation on constrained devices, or SVG output for print/export.
ECharts is already integrated via ds-echarts Lit web components and is the right choice for general dashboards and exploratory analytics.

Use ChartGPU when the visualization needs throughput: millions of points in scatter plots (UMAP/t-SNE cell embeddings), high-frequency time series with real-time streaming (`appendData()`), or density visualization via GPU compute binning.
ChartGPU's 6 chart types cover the core needs of scientific data at scale.

The two are complementary, not competing.
An ironstar instance like lakescope would use ECharts for general dashboard panels and ChartGPU for the high-density scatter and time series views where Canvas2D hits its ceiling.
Both can coexist as Lit or vanilla web components within the same Datastar-driven page; the integration pattern (Light DOM for Open Props token inheritance, `data-ignore-morph` for DOM ownership) is identical regardless of which rendering engine backs the component.

For rendering needs beyond 2D charts (3D vector fields, volumetric rendering, large-scale WebGPU point clouds), see the high-performance visualization tooling evaluation referenced at the top of this document.
