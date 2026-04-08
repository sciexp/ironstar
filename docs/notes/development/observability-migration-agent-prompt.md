# Observability migration agent prompt

Agent prompt for an independent Claude Code session working inside
`~/projects/rust-workspace/ironstar/` to research, map blast radius, and plan
a beads epic for migrating to an OpenTelemetry-forward observability stack
integrating SigNoz, Sentry, and PostHog.

## Prompt

```
You are a subagent Task. Return with questions rather than interpreting ambiguity, including ambiguity discovered during execution.

# Objective

Research, map, and plan a beads epic for migrating ironstar's observability stack to a fully OpenTelemetry-forward architecture integrating three systems:

1. **SigNoz** — OTel-native traces, metrics, and logs (replacing the current metrics-rs/Prometheus-exposition approach)
2. **Sentry** — error tracking and performance monitoring (new integration)
3. **PostHog** — product analytics, feature flags, event tracking (new integration)

This includes Rust crate changes, OTel SDK initialization, deploying SigNoz as a local dev service via the Nix services flake and process-compose, and wiring Sentry and PostHog into the telemetry pipeline.

Do NOT implement anything. Produce a blast radius map of current state, an analysis of how the three systems compose without duplication, and a proposed beads epic with issue decomposition.

# Working directory

~/projects/rust-workspace/ironstar/

Confirm your working directory as your first action.

# Source repositories available for reference

All paths are absolute. Read source code, examples, READMEs, and Cargo.toml files from these repos as needed for complete context.

## Ironstar (the target application)
- ~/projects/rust-workspace/ironstar/

## OpenTelemetry Rust SDK (monorepo: opentelemetry, opentelemetry_sdk, opentelemetry-otlp)
- ~/projects/rust-workspace/opentelemetry-rust/

## gRPC transport for OTLP export
- ~/projects/rust-workspace/tonic/

## Tracing-to-OpenTelemetry bridge
- ~/projects/rust-workspace/tracing-opentelemetry/

## Axum OTel instrumentation (monorepo: axum-tracing-opentelemetry, init-tracing-opentelemetry, fake-opentelemetry-collector)
- ~/projects/rust-workspace/tracing-opentelemetry-instrumentation-sdk/

## SigNoz application source
- ~/projects/lakescope-workspace/signoz/

## SigNoz documentation site (contains Rust integration guides)
- ~/projects/lakescope-workspace/signoz.io/

Key docs within:
- ~/projects/lakescope-workspace/signoz.io/data/docs/instrumentation/opentelemetry-rust.mdx
  (tracer provider setup, axum/actix/reqwest/tonic framework integrations, tracing bridge)
- ~/projects/lakescope-workspace/signoz.io/data/docs/instrumentation/rust/manual-instrumentation.mdx
  (manual span creation, attributes, events, error recording)
- ~/projects/lakescope-workspace/signoz.io/data/docs/metrics-management/send-metrics/applications/opentelemetry-rust.mdx
  (OTel metrics: MeterProvider setup, counters, histograms, gauges, observable instruments)

## Sentry platform source
- ~/projects/lakescope-workspace/sentry/

## Sentry Rust SDK (monorepo: sentry, sentry-core, sentry-tracing, sentry-tower, sentry-contexts, etc.)
- ~/projects/rust-workspace/sentry-rust/

## PostHog platform source (SDKs, API docs, feature flag evaluation)
- ~/projects/lakescope-workspace/posthog/

# Known context from prior analysis

## Crates to ADD for OTel/SigNoz (workspace.dependencies)
- opentelemetry (features: trace, metrics)
- opentelemetry_sdk (features: trace, metrics, rt-tokio)
- opentelemetry-otlp (features: grpc-tonic, trace, metrics, tls-roots)
- tonic (features: tls-native-roots)
- tracing-opentelemetry
- axum-tracing-opentelemetry

## Crates to REMOVE
- metrics (v0.24) — Prometheus-oriented metrics facade
- metrics-exporter-prometheus (v0.18) — Prometheus pull-based exporter

## Feature to evaluate for removal
- tower-http "trace" feature — may duplicate axum-tracing-opentelemetry's OtelAxumLayer

## Known metrics-rs usage sites (non-exhaustive, verify and complete)
- crates/ironstar/src/infrastructure/metrics.rs — PrometheusBuilder, init_prometheus_recorder, describe_* macros, test_prometheus_handle
- crates/ironstar/src/presentation/metrics.rs — /metrics endpoint, MetricsState
- crates/ironstar/src/state.rs — PrometheusHandle threaded through AppState
- crates/ironstar/src/main.rs — init_prometheus_recorder call
- crates/ironstar-event-store/src/event_store.rs — counter! macro usage
- crates/ironstar/tests/todo_feed.rs — test_prometheus_handle in test fixtures
- crates/ironstar/src/presentation/extractors.rs — test_prometheus_handle in tests

# Research tasks

## 1. Blast radius map

Produce a complete inventory of every file, function, type, and test in ironstar that touches the metrics-rs ecosystem (metrics crate, metrics-exporter-prometheus, PrometheusHandle, counter!/histogram!/gauge! macros, describe_* macros). Include transitive references (e.g., AppState fields that carry PrometheusHandle, test helpers that construct it).

Also inventory the current tracing setup: how tracing-subscriber is initialized, whether tower-http TraceLayer is used, and how traces currently flow (stdout? structured JSON? nowhere?). This determines the delta to bridge tracing → OTel → SigNoz.

## 2. Sentry integration analysis

Examine ~/projects/rust-workspace/sentry-rust/ to understand:
- Which subcrates are relevant for an axum + tracing app (likely: sentry, sentry-tracing, sentry-tower)
- How sentry-tracing interacts with the tracing ecosystem — does it conflict with or complement tracing-opentelemetry?
- Whether Sentry can ingest via OTel (Sentry has experimental OTel support) or requires its own SDK transport
- What the recommended layering looks like: can tracing-subscriber host both a tracing-opentelemetry layer AND a sentry-tracing layer simultaneously?
- Performance and error event routing: which events go to SigNoz (traces, metrics) vs Sentry (errors, crashes, performance) vs both

## 3. PostHog integration analysis

Examine ~/projects/lakescope-workspace/posthog/ to understand:
- Whether a Rust SDK exists or if integration is HTTP API-based
- What PostHog needs from the app: product analytics events, feature flag evaluation, user identification
- Whether PostHog events should flow through the OTel pipeline (via OTel events/logs) or through a separate PostHog client
- How PostHog feature flags would be consumed in axum handlers
- What the minimal viable integration looks like for a server-side Rust app

## 4. Composability analysis

The three systems serve different purposes but overlap in some areas. Produce a clear routing matrix:

| Signal type | SigNoz | Sentry | PostHog |
|---|---|---|---|
| Request traces | ? | ? | ? |
| Custom spans | ? | ? | ? |
| Application metrics | ? | ? | ? |
| Errors/panics | ? | ? | ? |
| Product analytics events | ? | ? | ? |
| Feature flags | ? | ? | ? |

Identify where duplication is acceptable (e.g., errors going to both SigNoz traces and Sentry), where it should be avoided, and how the tracing-subscriber layer stack should be ordered.

## 5. Nix services flake and process-compose

Examine the current Nix flake (flake.nix, any services-flake or process-compose-flake usage) to understand:
- What dev services already run under process-compose (databases, event buses, etc.)
- How the services flake output is structured
- What SigNoz deployment would look like as an additional service

SigNoz can run as a Docker Compose stack or individual containers. Research what the minimal self-hosted SigNoz deployment requires (ClickHouse, query-service, frontend, otel-collector) and how to express that in the services flake / process-compose configuration. If the flake uses nix-services or process-compose-flake, note which.

Also assess whether Sentry requires a local dev instance (likely not — cloud Sentry with a DSN is typical for dev) and whether PostHog needs local deployment or can use cloud with an API key.

Check whether there's an existing OpenTelemetry Collector in the dev setup, or whether the app currently exports directly (it likely doesn't export at all yet).

## 6. Epic decomposition

Propose a beads epic with issues decomposed along these dimensions:

**Infrastructure:**
- SigNoz dev service in process-compose/services flake
- OTel Collector configuration (if needed as intermediary)
- Sentry and PostHog configuration (DSN/API keys, secrets management)

**Foundation:**
- OTel SDK initialization (TracerProvider, MeterProvider, OTLP exporter config) as a new ironstar-telemetry crate or module
- tracing-subscriber layer stack design: ordering of fmt layer, tracing-opentelemetry layer, sentry-tracing layer

**Traces migration:**
- tracing-opentelemetry bridge
- axum-tracing-opentelemetry layer (replacing tower-http trace if appropriate)
- Sentry performance monitoring integration

**Metrics migration:**
- Replacing every metrics-rs call site with OTel meter API equivalents
- Removing PrometheusHandle from AppState and /metrics endpoint
- Defining equivalent OTel metric instruments

**Error tracking:**
- Sentry SDK initialization and axum integration
- Panic handler and error capture configuration
- Sentry-specific context enrichment (user, tags, breadcrumbs)

**Product analytics:**
- PostHog client initialization
- Event capture patterns for axum handlers
- Feature flag evaluation integration

**Test migration:**
- Replacing test_prometheus_handle fixtures
- Verifying metric assertions still work
- Test strategies for OTel, Sentry, and PostHog (fake-opentelemetry-collector from the instrumentation-sdk monorepo may be useful)

**Validation:**
- End-to-end verification that traces and metrics appear in local SigNoz
- Sentry error capture verification
- PostHog event delivery verification

For each proposed issue, state:
- What it depends on (other issues in the epic)
- What files it primarily touches
- Whether it's independently testable
- Estimated complexity (small/medium/large)

## 7. Open questions

Surface any ambiguities you find, including but not limited to:
- Should the OTel initialization live in a new ironstar-telemetry crate or in the existing ironstar crate's infrastructure module?
- Are there metrics currently emitted via metrics-rs that have no direct OTel equivalent (e.g., describe_* metadata)?
- Does the process-compose setup support Docker containers, or only native processes? This affects SigNoz deployment strategy.
- Should the /metrics Prometheus endpoint be preserved as a compatibility layer (OTel SDK can expose one), or removed entirely?
- Can tracing-subscriber host tracing-opentelemetry and sentry-tracing layers simultaneously without interference? What is the correct layer ordering?
- Does Sentry's OTel integration (experimental) obviate the need for sentry-tracing, or is the dedicated SDK integration more reliable?
- Is there a Rust SDK for PostHog, or is HTTP API integration the only path? If API-only, should this be a thin client in ironstar or a standalone crate?
- Are there any other observability-adjacent crates or patterns (structured logging config, error reporting) that should be addressed in the same epic?
- What is the version compatibility matrix across opentelemetry 0.31, tracing-opentelemetry, sentry-tracing, and axum-tracing-opentelemetry? Are there known conflicts?

# Output format

Return a structured document with seven sections matching the research tasks above. Use markdown. Be concrete — cite file paths, line numbers, function names. For the epic decomposition, use a table or numbered list with dependency arrows.
```
