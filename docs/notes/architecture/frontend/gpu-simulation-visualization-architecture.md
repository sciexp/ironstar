# GPU-accelerated simulation visualization architecture

This document establishes the architecture for integrating GPU-accelerated scientific simulation visualization into ironstar's hypermedia framework.
It addresses Gillespie stochastic simulation algorithm (SSA) trajectory visualization as the primary use case, with patterns generalizable to other dynamical systems.

> **Prerequisite reading**: [integration-patterns.md](integration-patterns.md) for the Lit web component bridge pattern, [signal-contracts.md](signal-contracts.md) for Datastar signal semantics, and [ds-echarts-integration-guide.md](ds-echarts-integration-guide.md) for the canonical Pattern 1.5 example.

---

## Two distinct uses of WebGPU

A critical architectural distinction: WebGPU serves two independent roles in this system, and conflating them leads to incorrect architectural decisions.

**WebGPU for rendering** is what three.js uses internally.
When you create a `MeshStandardMaterial` and call `renderer.render(scene, camera)`, three.js builds a TSL (Three Shading Language) node graph from JavaScript objects, compiles it to WGSL via `WGSLNodeBuilder`, creates a `GPURenderPipeline`, and issues draw commands.
You never write WGSL for this — three.js generates it as an implementation detail, just as it previously generated GLSL for its WebGL renderer.

**WebGPU for compute** is a separate capability for general-purpose GPU computation.
This is where you write (or generate) WGSL compute shaders that perform numerical simulation — for example, running thousands of Gillespie SSA trajectories in parallel.
Three.js has nothing to do with this; it uses the browser's WebGPU API directly.

These two uses share a single connection point: a `GPUBuffer`.
A compute shader writes simulation output (trajectory positions, species counts) into a `StorageBuffer`.
Three.js reads that same buffer as vertex data for rendering.
The data stays on the GPU — no CPU round-trip.

```
WebGPU compute (independent of three.js):
  WGSL kernel  -->  writes trajectory data  -->  StorageBuffer

Three.js rendering (generates its own WGSL internally):
  StorageBuffer  -->  read as vertex positions  -->  render as lines/points/mesh
```

This separation means:

- A `ds-threejs` web component that only renders server-provided data needs no WGSL authoring whatsoever.
- WGSL authoring becomes relevant only when client-side GPU simulation is added.
- The two capabilities can be adopted independently and incrementally.

---

## Computation placement taxonomy

Three categories of computation exist in this architecture, each with a natural placement.

### Authoritative computation (server-side)

Computation that produces persisted, shared, or decision-driving results.
The server is the source of truth per the Tao of Datastar.

- Bayesian parameter inference (Hodosome.jl posteriors)
- Pre-computed posterior predictive check ensembles
- DuckDB OLAP queries over stored simulation results
- Model selection and comparison statistics

These results are served via PatchElements (HTML fragments) or PatchSignals (signal updates) following the existing ChartTransformer pattern.

### Presentation computation (client-side)

Computation that transforms authoritative results into visual output.
This is analogous to the browser's CSS layout engine performing complex computation from server-sent HTML.

- Three.js scene graph traversal, frustum culling, GPU draw commands
- ECharts internal animation and transition computation
- Forward simulation from a specific parameter set (ephemeral, not persisted)
- Distribution kernel density estimation for real-time plots

Presentation computation is consistent with HATEOAS: the server determines *which parameters to visualize* (application state), the client determines *how to simulate and render them* (presentation).
Results are ephemeral — they exist only for the current visualization session.

### Reference computation (offline, data lakehouse)

Large-scale pre-computed results stored in Parquet, queryable via DuckDB.

- PPC tier 3 ensembles (5M trajectories) from Hodosome.jl
- Summary statistics across parameter sweeps
- Stored in omicslakehouse Parquet tables
- Served via DuckDB httpfs or DuckDB-WASM client-side querying

---

## The generators-over-data principle

When posterior parameter values are available and the goal is forward simulation visualization, sending parameters (generators) rather than trajectories (generated data) is categorically more efficient.

For a telegraph model (4 parameters, 100-trajectory ensemble, 2 species x 1000 timepoints):

| What | Size |
|---|---|
| Parameters (4 reaction rates) | 16 bytes |
| One SSA trajectory (2 species x 1000 timepoints) | 8 KB |
| Ensemble (100 trajectories) | 800 KB |
| PPC tier 2 (100 posterior samples x 5000 cells) | ~400 MB |

The parameter-to-trajectory compression ratio is roughly 50,000x.

This principle applies when the simulation is deterministic given parameters and PRNG seed, the client has sufficient compute (CPU or GPU), and the results are ephemeral.
It does not apply when results must be reproducible across sessions, results are shared between users, or the simulation exceeds client compute capacity.

---

## Technology evaluation

### Production-ready (suitable for prototypes and production)

| Technology | Role | Evidence |
|---|---|---|
| Three.js WebGPURenderer (r171+) | 3D rendering with automatic WebGL2 fallback | Production since Sep 2025, 2.7M weekly npm downloads |
| Lit 3.x | Web component bridge (Pattern 1.5) | Proven in ironstar ds-echarts |
| Datastar + SSE | Signal transport, DOM morphing | Proven in ironstar |
| rust_embed | Static asset delivery (JS, CSS, WGSL) | Proven in ironstar |
| DuckDB (server-side) | Pre-computed data queries | Proven in ironstar analytics layer |
| rebop (Rust crate) | Server-side Gillespie SSA | Fastest SSA implementation benchmarked across all languages |

### Emerging (suitable for experimentation, approaching production)

| Technology | Role | Status | Risk |
|---|---|---|---|
| TypeGPU v0.7 | Type-safe WebGPU compute shaders | Active development, Software Mansion | API evolving; `@typegpu/three` interop is young |
| ChartGPU | 2D WebGPU streaming charts | ~2.9K stars, TypeScript-first, `appendData()` streaming API | Young project, no WebGL fallback |
| Plain WGSL compute | Client-side SSA kernels | Standard WebGPU, no library dependency | Manual buffer management, no type safety layer |

### Watch list (promising but not prototype-ready)

| Technology | Role | Status | Why watch |
|---|---|---|---|
| wgsl-rs 0.1.0 | Rust to WGSL transpilation | NLnet-funded, active (112 commits since Jan 2025) | "Two worlds" — same code as Rust (server CPU) and WGSL (client GPU), compile-time naga validation |
| rust-gpu | Rust to SPIR-V (then WGSL via naga) | Experimental, broader Rust subset than wgsl-rs | Write shaders in full Rust with standard toolchain |
| rebop to WASM | Client-side SSA on CPU | rebop is mature; WASM compilation target untested | Isomorphic simulation code without GPU dependency |

---

## Recommended phased architecture

### Phase 1: server-computed, client-rendered (extend proven patterns)

Extend the ds-echarts Pattern 1.5 to three.js.
Server computes trajectory data; client renders it.
No WGSL authoring required.

**Server side:**

- rebop or DuckDB queries on pre-computed Parquet produce trajectory data
- ChartTransformer pattern adapts simulation output to visualization signals
- SSE PatchSignals delivers trajectory arrays as typed signals

**Client side:**

- `ds-threejs` Lit web component (Pattern 1.5 with animation loop adaptation)
- Three.js WebGPURenderer with automatic WebGL2 fallback
- Server-delivered trajectory data fed to `BufferAttribute` and rendered as instanced geometry

**Signal contract:**

```json
{
  "trajectoryData": "Float64Array (flattened: species x timepoints x trajectories)",
  "speciesNames": ["mRNA", "protein"],
  "timepoints": "Float64Array",
  "ensembleSize": 100,
  "modelName": "telegraph",
  "loading": false
}
```

**Three adaptations from ds-echarts Pattern 1.5:**

First, the animation loop.
ECharts is event-driven: `setOption()` triggers a render.
Three.js requires a persistent `requestAnimationFrame` loop managed in the Lit component's lifecycle (start in `firstUpdated`, cancel in `disconnectedCallback`).

Second, the signal contract.
ECharts receives a single declarative JSON blob (`$chartOption`).
Three.js has no `setOption()` equivalent, so signals must represent structured scene data (trajectory arrays, camera state, visualization parameters) rather than a monolithic configuration.

Third, the two-clock model.
With ds-echarts, the server pushes a new option and the client renders once (one clock).
With three.js, the server pushes trajectory data at event-driven intervals while the client renders continuously at display refresh rate (two clocks).
The Lit component must buffer incoming signal updates and apply them on the next animation frame, not synchronously in `updated()`.

### Phase 2: client-side GPU compute (when interactivity demands it)

Add client-side SSA execution for interactive parameter exploration.
Server sends parameters; client runs SSA on GPU; results are ephemeral.

**The boundary shift:**

- SSE PatchSignals carries *parameters* (16-100 bytes), not *trajectories* (hundreds of KB)
- A WGSL compute shader runs SSA in parallel across ensemble members
- StorageBuffer output feeds directly into three.js render (zero-copy GPU path)
- Server retains control of *which* parameters to visualize via hypermedia controls

**Technology choice for compute shader authoring:**

At this phase, evaluate whichever of the following has matured sufficiently:

1. **Plain WGSL** (handwritten) — zero dependency risk, full control, no type safety beyond the WGSL validator
2. **TypeGPU** — if v1.0+ is stable, provides type-safe TypeScript-to-WGSL compute with three.js interop via `@typegpu/three`
3. **wgsl-rs** — if the Rust subset covers SSA needs, provides isomorphic "two worlds" execution: same code runs as Rust on the server (for validation, testing, pre-computation) and transpiles to WGSL for client GPU execution, with compile-time naga validation

Do not commit to a compute shader authoring tool prematurely.
The choice depends on which has matured sufficiently by the time Phase 2 begins.

**WGSL delivery:**

Regardless of authoring tool, the generated WGSL can be served through the existing rust_embed asset pipeline by adding `#[include = "*.wgsl"]` to the `StaticAssets` struct.
Three delivery options exist:

- Static asset at `/static/ssa.[hash].wgsl` (cached, separate HTTP request)
- Inlined in PatchElements HTML as `<script type="wgsl">` (zero-fetch, server-controlled — most HATEOAS-aligned)
- Bundled in the JS web component via Rolldown raw import (zero-fetch, but shader changes require JS rebuild)

### Phase 3: isomorphic compute (when wgsl-rs matures)

If wgsl-rs reaches stability, the same `#[wgsl] mod` code can:

- Run as Rust on the server for validation, pre-computation, and reference results
- Transpile to WGSL served to the browser for GPU execution
- Be tested with standard Rust test infrastructure including property-based testing

This eliminates the divergence risk between server-side and client-side simulation implementations.
The `WGSL_MODULE.wgsl_source()` output is `&'static str` data baked into the binary at compile time, compatible with both rust_embed delivery and direct inclusion in hypertext templates.

---

## Pattern 2: GPU compute web component

A new web component pattern extending Pattern 1.5 for components that perform GPU computation in addition to rendering.

| Aspect | Pattern 1.5 (ds-echarts) | Pattern 2 (ds-gillespie) |
|---|---|---|
| Signal contract | Complete visualization spec (`$chartOption`) | Parameters + model definition (`$simParams`) |
| Computation | None (stateless transform of server data) | GPU compute (SSA simulation) |
| Render trigger | `setOption()` on signal change | `requestAnimationFrame` loop (continuous) |
| Clock model | One-clock (server push, then render) | Two-clock (server pushes parameters, client renders continuously) |
| Data flow | Server signal to library API | Server signal to GPU compute to GPU render |
| GPU buffers | None (SVG/canvas managed by library) | StorageBuffers shared between compute and render |

Pattern 2 shares with Pattern 1.5:

- Light DOM (`createRenderRoot() { return this; }`) for CSS token inheritance
- `data-ignore-morph` on the element to protect library-managed DOM
- ResizeObserver with debounce for responsive sizing
- Custom events for user interactions (`bubbles: true, composed: true`)
- `disconnectedCallback()` full cleanup (dispose renderer, cancel animation frame, disconnect observers)

Pattern 2 is only needed for Phase 2 (client-side compute).
Phase 1 uses Pattern 1.5 directly, with the three adaptations described above.

---

## HATEOAS compatibility

Client-side GPU simulation is compatible with the Tao of Datastar's "server as source of truth" because:

The server controls *application state*: which parameter set to visualize, which model to use, what the posterior distribution is.
The client performs *presentation computation*: running the forward model as part of rendering, same as ECharts computing internal animations from `setOption()` data.
SSE carries *hypermedia controls*: parameter values, model definitions, UI configuration, and potentially the WGSL compute shader itself.
Simulation results are *ephemeral*: not persisted, not shared, not authoritative beyond the current session.

The categorical distinction is: the server decides *what to show*; the client decides *how to show it*.
Forward simulation from server-provided parameters is "how to show it" — a presentation concern.

---

## Data flow architecture

```
Hodosome.jl (offline)
  -> Bayesian inference -> posterior samples
  -> Store in omicslakehouse Parquet via DuckDB

Ironstar server (request time)
  -> DuckDB queries posterior samples
  -> Phase 1: run rebop SSA, send trajectories via PatchSignals
  -> Phase 2: send parameters only via PatchSignals, client computes

Browser
  -> ds-threejs Lit web component receives signals
  -> Phase 1: trajectory data -> three.js BufferAttribute -> render
  -> Phase 2: parameters -> WGSL compute shader -> StorageBuffer -> render
  -> User interactions -> custom events -> SSE back to server
```

---

## Related documentation

Internal ironstar architecture:

- [integration-patterns.md](integration-patterns.md) — Pattern 1 and 1.5 decision framework
- [integration-patterns-visualizations.md](integration-patterns-visualizations.md) — ECharts, Vega-Lite, Mosaic patterns
- [ds-echarts-integration-guide.md](ds-echarts-integration-guide.md) — canonical Pattern 1.5 implementation
- [ds-echarts-backend.md](ds-echarts-backend.md) — server-side SSE handler patterns
- [signal-contracts.md](signal-contracts.md) — Datastar signal type contracts
- [echarts-vs-chartgpu-evaluation.md](echarts-vs-chartgpu-evaluation.md) — 2D charting technology evaluation
- [../core/design-principles.md](../core/design-principles.md) — foundational design principles and HATEOAS commitment
- [../core/semantic-model.md](../core/semantic-model.md) — monadic server and comonadic client boundary semantics
- [../cqrs/sse-connection-lifecycle.md](../cqrs/sse-connection-lifecycle.md) — SSE streaming and backpressure patterns
- [../cqrs/performance-advanced-patterns.md](../cqrs/performance-advanced-patterns.md) — high-frequency update batching
- [../application/chart-transformer-pattern.md](../application/chart-transformer-pattern.md) — DuckDB to chart configuration pipeline

External source code references:

- `~/projects/lakescope-workspace/three.js/` — three.js source (WebGPURenderer, TSL, StorageBufferAttribute)
- `~/projects/lakescope-workspace/TypeGPU/` — type-safe WebGPU toolkit (`@typegpu/three` interop)
- `~/projects/lakescope-workspace/ChartGPU/` — WebGPU-native 2D streaming charts
- `~/projects/rust-workspace/wgsl-rs/` — Rust to WGSL transpiler (watch list)
- `~/projects/hodosome-workspace/Hodosome.jl/` — Gillespie SSA and FSP simulation (Catalyst.jl, JumpProcesses.jl)
- `~/projects/hodosome-workspace/hodosome/` — methodology specification and data pipeline architecture
- `~/projects/omicslake-workspace/omicslakehouse/` — pre-computed simulation data in Parquet (SCOO star schema)
