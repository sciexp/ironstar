# Documentation map

## Getting oriented

Start here when new to the project or reviewing architectural principles:

- **Design principles**: `docs/notes/architecture/core/design-principles.md` — Tao of Datastar principles, functional programming foundations, algebraic types
- **Architecture decisions**: `docs/notes/architecture/core/architecture-decisions.md` — Technology choices (Open Props, hypertext, Zenoh), project structure tree, rationale for all major decisions
- **Crate architecture**: `docs/notes/architecture/core/crate-architecture.md` — Multi-crate decomposition plan, HasXxx capability traits, workspace scaling path
- **Crate services composition**: `docs/notes/architecture/core/crate-services-composition.md` — Service layer patterns, All composition root, HasXxx trait implementation

## Architecture decision records

Consult these when questioning "why this technology?" for specific subsystems:

- **Frontend stack**: `docs/notes/architecture/decisions/frontend-stack-decisions.md` — Open Props vs Tailwind, Rolldown vs esbuild, when to use Lit
- **Backend core**: `docs/notes/architecture/decisions/backend-core-decisions.md` — axum + hypertext integration, lazy rendering strategy
- **Infrastructure**: `docs/notes/architecture/decisions/infrastructure-decisions.md` — SQLite vs PostgreSQL, Zenoh vs NATS, embedded vs external services
- **CQRS implementation**: `docs/notes/architecture/decisions/cqrs-implementation-decisions.md` — Custom CQRS vs cqrs-es/esrs frameworks, pure sync aggregates
- **fmodel-rust adoption**: `docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md` — Decider pattern adoption, SQLite EventRepository, migration path
- **Build tooling**: `docs/notes/architecture/decisions/build-tooling-decisions.md` — Rolldown configuration, asset embedding, dev/prod modes
- **Authentication**: `docs/notes/architecture/decisions/oauth-authentication.md` — OAuth-only auth (GitHub first, Google planned), provider strategy, RBAC patterns
- **Error handling**: `docs/notes/architecture/decisions/error-handling-decisions.md` — Error types, Result propagation, user-facing messages
- **Error types**: `docs/notes/architecture/decisions/error-types.md` — Error type hierarchy definitions, domain vs infrastructure errors
- **Observability**: `docs/notes/architecture/decisions/observability-decisions.md` — Structured logging, Prometheus metrics, health checks
- **Metrics reference**: `docs/notes/architecture/decisions/metrics-reference.md` — Prometheus metrics reference, naming conventions, cardinality

## Implementing features

Read these when implementing specific subsystems or integrating libraries:

- **Event sourcing core**: `docs/notes/architecture/cqrs/event-sourcing-core.md` — Decider patterns, command handling, event schema, event store
- **Session management**: `docs/notes/architecture/infrastructure/session-management.md` — Session cookies, SQLite schema, axum extractors, per-session Zenoh keys
- **Session implementation**: `docs/notes/architecture/infrastructure/session-implementation.md` — Concrete implementation patterns, middleware integration, session lifecycle
- **Session security**: `docs/notes/architecture/infrastructure/session-security.md` — CSRF protection, secure cookie attributes, session fixation prevention
- **Integration patterns**: `docs/notes/architecture/frontend/integration-patterns.md` — Web components (vanilla/Lit), Vega-Lite, ECharts
- **Integration patterns visualizations**: `docs/notes/architecture/frontend/integration-patterns-visualizations.md` — Visualization-specific patterns, data binding, reactivity
- **ECharts integration guide**: `docs/notes/architecture/frontend/ds-echarts-integration-guide.md` — Complete ds-echarts Lit component implementation
- **ECharts backend**: `docs/notes/architecture/frontend/ds-echarts-backend.md` — Server-side data preparation, SSE streaming, signal contracts
- **ECharts build test**: `docs/notes/architecture/frontend/ds-echarts-build-test.md` — Build pipeline integration, testing strategies, deployment verification
- **Signal contracts**: `docs/notes/architecture/frontend/signal-contracts.md` — TypeScript type generation with ts-rs, datastar signal patterns
- **Development workflow**: `docs/notes/architecture/infrastructure/development-workflow.md` — process-compose orchestration, hot reload, asset serving modes

## CQRS pipeline deep dives

Read these when implementing or debugging the event sourcing + SSE integration:

- **SSE connection lifecycle**: `docs/notes/architecture/cqrs/sse-connection-lifecycle.md` — Client subscription, reconnection resilience, Last-Event-ID
- **Event replay consistency**: `docs/notes/architecture/cqrs/event-replay-consistency.md` — Snapshot + delta patterns, cache-aside with Zenoh invalidation
- **Projection patterns**: `docs/notes/architecture/cqrs/projection-patterns.md` — Materialized views, denormalization, DuckDB analytics
- **Performance tuning**: `docs/notes/architecture/cqrs/performance-tuning.md` — Channel sizing, backpressure, metrics instrumentation
- **Performance advanced patterns**: `docs/notes/architecture/cqrs/performance-advanced-patterns.md` — Debouncing, batching, rate limiting
- **Command write patterns**: `docs/notes/architecture/cqrs/command-write-patterns.md` — Validation, optimistic locking, idempotency

## Caching

Read these when implementing caching strategies for analytics and projections:

- **Analytics cache architecture**: `docs/notes/architecture/infrastructure/analytics-cache-architecture.md` — Moka cache design, TTL-based eviction, Zenoh invalidation (Pattern 4)
- **Analytics cache patterns**: `docs/notes/architecture/infrastructure/analytics-cache-patterns.md` — Cache-aside, write-through, invalidation strategies, DuckDB query caching

## Frontend implementation

Read these when working on CSS, bundling, or web components:

- **Frontend build pipeline**: `docs/notes/architecture/frontend/frontend-build-pipeline.md` — Rolldown config, PostCSS, asset serving modes
- **Lit component bundling**: `docs/notes/architecture/frontend/lit-component-bundling.md` — Lit-specific bundling, Rolldown vs esbuild, TypeScript decorators
- **CSS architecture**: `docs/notes/architecture/frontend/css-architecture.md` — Open Props tokens, theme customization, component styles

## Event bus and distribution

Read these when working with pub/sub, event distribution, or scaling beyond single-node:

- **Zenoh event bus**: `docs/notes/architecture/infrastructure/zenoh-event-bus.md` — Key expression patterns, embedded config, Zenoh architecture
- **Event bus compatibility patterns**: `docs/notes/architecture/infrastructure/event-bus-compatibility-patterns.md` — DualEventBus pattern for legacy Zenoh integration, compatibility strategies, testing approaches

## External references

- Datastar SDK specification: `~/projects/lakescope-workspace/datastar/sdk/ADR.md`
- Datastar documentation: `~/projects/lakescope-workspace/datastar-doc/`
- Tao of Datastar: `~/projects/lakescope-workspace/datastar-doc/guide_the_tao_of_datastar.md`
- Northstar (Go template): `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/`
- Open Props design tokens: `~/projects/lakescope-workspace/open-props/`
- Open Props UI components: `~/projects/lakescope-workspace/open-props-ui/`
