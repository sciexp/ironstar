# Template synthesis sources

Ironstar is instantiated as a deliberate synthesis of patterns from multiple template repositories, each contributing specific architectural elements.

## Pattern sources

| Source Repository | Patterns Extracted |
|-------------------|-------------------|
| `~/projects/nix-workspace/typescript-nix-template` | Flake structure with `import-tree`, deferred module composition, category-based CI with content-addressed caching, om CLI instantiation, custom devShell helper scripts |
| `~/projects/rust-workspace/rust-nix-template` | rust-flake integration (crane + rust-overlay), `rust-toolchain.toml` pattern, per-crate `crane.args` configuration, layered devShell composition |
| `~/projects/rust-workspace/rustlings-workspace` | Cargo workspace organization with `resolver = "2"`, `workspace.dependencies` for DRY, `crates/` subdirectory structure, per-crate `crate.nix` files |
| `~/projects/lakescope-workspace/datastar-go-nats-template-northstar` | Datastar SSE architecture, web component integration, single-binary asset embedding, dev/prod mode separation, three-stage build pipeline |

## Template instantiation

Ironstar uses omnix `om` CLI for parameterized instantiation.
Example from `typescript-nix-template` README adapted for ironstar:

```bash
PROJECT_DIRECTORY=my-ironstar-app && \
PARAMS=$(cat <<EOF
{
  "project-name": "$PROJECT_DIRECTORY",
  "crate-name": "my_ironstar_app",
  "github-ci": true,
  "example-todo": true,
  "nix-template": false
}
EOF
) && \
om init github:user/ironstar/main \
  -o "$PROJECT_DIRECTORY" --non-interactive --params "$PARAMS"
```

The template machinery (omnix params, path-conditional includes) follows the pattern from `typescript-nix-template/modules/template.nix`.

## Single-binary asset embedding

Northstar embeds static assets into the Go binary using `//go:embed` + `hashfs` for content-hashed URLs.
Ironstar replicates this pattern for Rust:

| Go (northstar) | Rust (ironstar) |
|----------------|-----------------|
| `//go:embed static` | `rust-embed` crate with `#[derive(RustEmbed)]` |
| `hashfs.NewFS()` content hashing | Rolldown's `[hash]` in output filenames + `manifest.json` |
| Build tags (`!dev` / `dev`) | Conditional compilation (`#[cfg(debug_assertions)]`) |
| `os.DirFS()` for dev | `tower-http::services::ServeDir` for dev |
| `hashfs.FileServer()` for prod | Custom axum handler serving embedded assets |

The build pipeline:

```
web-components/
├── index.ts                    # Entry point
└── styles/main.css             # Open Props imports
        │
        ▼ (Rolldown build)
static/dist/
├── bundle.[hash].js
├── bundle.[hash].css
└── manifest.json               # Maps entry → hashed filename
        │
        ▼ (cargo build --release)
target/release/ironstar          # Single binary with embedded static/dist/
```

Dev mode serves directly from `static/` via `ServeDir` with no caching.
Prod mode embeds `static/dist/` and serves with `Cache-Control: max-age=31536000, immutable`.

## Workspace scaling path

Ironstar starts as a single crate but the workspace structure supports future decomposition into a multi-crate architecture drawing from patterns in Golem (~25 crates) and Hyperswitch (~40 crates).
Key patterns include HasXxx capability traits, All composition root, three commons crates (enums/types/utils), and configuration-driven adapter selection.
See `docs/notes/architecture/core/crate-architecture.md` for the complete layered decomposition plan and migration path.

## Intentional divergences from Northstar

Ironstar adapts Northstar's patterns for Rust's type system and ecosystem conventions.
These divergences are deliberate architectural choices reflecting Rust-native tooling preferences and single-node deployment targets.
The Northstar Go template and datastar-go SDK remain valuable reference implementations for understanding Datastar's SSE streaming architecture and web component integration patterns.

| Northstar Pattern | Ironstar Adaptation | Rationale |
|-------------------|---------------------|-----------|
| Tailwind + DaisyUI | Open Props + Open Props UI | Design tokens over utility classes; better alignment with server-rendered HTML |
| esbuild (Go) | Rolldown (Rust) | Rust-native toolchain consistency |
| Embedded NATS | Zenoh (embedded mode) | Rust-native pub/sub with key expression filtering; distribution-ready |
| hashfs runtime hashing | Rolldown content hashing | Hash computed at build time via bundler, not at runtime |
| Templ (Go templates) | hypertext (Rust macros) | Compile-time type-checked HTML with lazy evaluation |
| Air hot reload | cargo-watch + process-compose | Rust ecosystem tooling |
| Task runner (Taskfile) | justfile | Rust ecosystem convention |
| ds-echarts Lit component | Adopt as-is | Complex chart state requires Lit lifecycle hooks |
