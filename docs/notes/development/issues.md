# Beads Export

*Generated: Fri, 26 Dec 2025 20:16:25 EST*

## Summary

| Metric | Count |
|--------|-------|
| **Total** | 176 |
| Open | 158 |
| In Progress | 0 |
| Blocked | 0 |
| Closed | 18 |

## Quick Actions

Ready-to-run commands for bulk operations:

```bash
# Close open items (158 total, showing first 10)
bd close ironstar-2nt.13 ironstar-2nt.12 ironstar-2nt.11 ironstar-r62.9 ironstar-r62.8 ironstar-r62.7 ironstar-r62.2 ironstar-r62.1 ironstar-r62 ironstar-nyp.8

# View high-priority items (P0/P1)
bd show ironstar-2nt.13 ironstar-2nt.12 ironstar-2nt.11 ironstar-r62.9 ironstar-r62.8 ironstar-r62.7 ironstar-r62.2 ironstar-r62.1 ironstar-r62 ironstar-nyp.8 ironstar-nyp.6 ironstar-nyp.2 ironstar-nyp.1 ironstar-nyp ironstar-ny3.13 ironstar-ny3.12 ironstar-ny3.11 ironstar-ny3.10 ironstar-ny3.9 ironstar-ny3.8 ironstar-ny3.7 ironstar-ny3.5 ironstar-ny3.4 ironstar-ny3.3 ironstar-ny3.2 ironstar-2nt.7 ironstar-2nt.6 ironstar-2nt.5 ironstar-2nt.4 ironstar-2nt.3 ironstar-2nt.2 ironstar-2nt ironstar-f8b.5 ironstar-f8b.4 ironstar-f8b.3 ironstar-f8b.2 ironstar-f8b.1 ironstar-f8b ironstar-6lq.7 ironstar-6lq ironstar-3gd.3 ironstar-jqv.12 ironstar-nyp.31 ironstar-nyp.30 ironstar-nyp.29 ironstar-nyp.27 ironstar-nyp.26 ironstar-nyp.25 ironstar-nyp.22 ironstar-ny3.17 ironstar-ny3.16 ironstar-753.6 ironstar-nyp.21 ironstar-2nt.10 ironstar-2nt.9 ironstar-nyp.19 ironstar-jqv.7 ironstar-961 ironstar-9b1 ironstar-3gd ironstar-nyp.15 ironstar-753.5 ironstar-753.4 ironstar-jqv ironstar-0tk ironstar-c7z ironstar-amw ironstar-b9h ironstar-09r ironstar-e6k.8 ironstar-e6k.7 ironstar-e6k.6 ironstar-e6k.5 ironstar-e6k.4 ironstar-e6k.3 ironstar-e6k.2 ironstar-e6k.1 ironstar-r62.13 ironstar-r62.10 ironstar-r62.3 ironstar-nyp.12 ironstar-nyp.11 ironstar-nyp.10 ironstar-nyp.9 ironstar-nyp.7 ironstar-nyp.4 ironstar-nyp.3 ironstar-ny3.14 ironstar-ny3.6 ironstar-ny3 ironstar-2nt.8

```

## Table of Contents

- [🟢 ironstar-2nt.13 Enforce async/sync boundary via module organization](#ironstar-2nt-13)
- [🟢 ironstar-2nt.12 Implement UUID-tracked error type for distributed correlation](#ironstar-2nt-12)
- [🟢 ironstar-2nt.11 Add version(&self) -> u64 to Aggregate trait](#ironstar-2nt-11)
- [🟢 ironstar-r62.9 Create base layout template with Datastar initialization](#ironstar-r62-9)
- [🟢 ironstar-r62.8 Implement RenderableToDatastar conversion trait](#ironstar-r62-8)
- [🟢 ironstar-r62.7 Implement query GET handlers](#ironstar-r62-7)
- [🟢 ironstar-r62.2 Create devShell module with tools and environment](#ironstar-r62-2)
- [🟢 ironstar-r62.1 Add justfile with development and build tasks](#ironstar-r62-1)
- [🟢 ironstar-r62 Presentation layer](#ironstar-r62)
- [🟢 ironstar-nyp.8 Implement SSE 15-second keep-alive comment stream](#ironstar-nyp-8)
- [🟢 ironstar-nyp.6 Create Projection trait for read models](#ironstar-nyp-6)
- [🟢 ironstar-nyp.2 Create EventStore trait abstraction](#ironstar-nyp-2)
- [🟢 ironstar-nyp.1 Create database migrations/ directory with schema.sql](#ironstar-nyp-1)
- [🟢 ironstar-nyp Event sourcing infrastructure](#ironstar-nyp)
- [🟢 ironstar-ny3.13 Implement rust-embed conditional asset serving](#ironstar-ny3-13)
- [🟢 ironstar-ny3.12 Implement manifest.json parser for hashed filename resolution](#ironstar-ny3-12)
- [🟢 ironstar-ny3.11 Create static/dist/ output directory structure](#ironstar-ny3-11)
- [🟢 ironstar-ny3.10 Configure ts-rs export directory and justfile task](#ironstar-ny3-10)
- [🟢 ironstar-ny3.9 Add ts-rs dependency to Cargo.toml](#ironstar-ny3-9)
- [🟢 ironstar-ny3.8 Create web-components/index.ts entry point](#ironstar-ny3-8)
- [🟢 ironstar-ny3.7 Create TypeScript configuration (tsconfig.json)](#ironstar-ny3-7)
- [🟢 ironstar-ny3.5 Configure CSS cascade layers for predictable specificity](#ironstar-ny3-5)
- [🟢 ironstar-ny3.4 Setup Open Props design tokens and theme layer](#ironstar-ny3-4)
- [🟢 ironstar-ny3.3 Setup PostCSS configuration for modern CSS features](#ironstar-ny3-3)
- [🟢 ironstar-ny3.2 Configure Rolldown bundler with content-based hashing](#ironstar-ny3-2)
- [🟢 ironstar-2nt.7 Implement command validation pattern with Result types](#ironstar-2nt-7)
- [🟢 ironstar-2nt.6 Enforce camelCase convention for Datastar signal fields](#ironstar-2nt-6)
- [🟢 ironstar-2nt.5 Create Datastar signal types with ts-rs derives](#ironstar-2nt-5)
- [🟢 ironstar-2nt.4 Design aggregate root state machines](#ironstar-2nt-4)
- [🟢 ironstar-2nt.3 Implement value objects and smart constructors](#ironstar-2nt-3)
- [🟢 ironstar-2nt.2 Define algebraic domain types and aggregate structure](#ironstar-2nt-2)
- [🟢 ironstar-2nt Domain layer](#ironstar-2nt)
- [🟢 ironstar-f8b.5 Verify process-compose up works with all services](#ironstar-f8b-5)
- [🟢 ironstar-f8b.4 Configure cargo-watch to curl hotreload trigger on success](#ironstar-f8b-4)
- [🟢 ironstar-f8b.3 Set up service orchestration (frontend bundler, cargo-watch)](#ironstar-f8b-3)
- [🟢 ironstar-f8b.2 Configure process-compose.yaml for dev services](#ironstar-f8b-2)
- [🟢 ironstar-f8b.1 Integrate process-compose-flake patterns into devShell](#ironstar-f8b-1)
- [🟢 ironstar-f8b Process compose integration](#ironstar-f8b)
- [🟢 ironstar-6lq.7 Add Rust to CI matrix and extend inherited workflows](#ironstar-6lq-7)
- [🟢 ironstar-6lq Rust workspace integration](#ironstar-6lq)
- [🟢 ironstar-3gd.3 Implement CacheDependency struct for Zenoh-based cache invalidation](#ironstar-3gd-3)
- [🟢 ironstar-jqv.12 Implement session regeneration and user binding in OAuth callback](#ironstar-jqv-12)
- [🟢 ironstar-nyp.31 Implement health check endpoints (/health, /health/ready, /health/live)](#ironstar-nyp-31)
- [🟢 ironstar-nyp.30 Implement observability initialization with dev/prod splitting](#ironstar-nyp-30)
- [🟢 ironstar-nyp.29 Implement error propagation pattern through CQRS pipeline](#ironstar-nyp-29)
- [🟢 ironstar-nyp.27 Implement ZenohEventBus struct with publish/subscribe methods](#ironstar-nyp-27)
- [🟢 ironstar-nyp.26 Create Zenoh embedded router configuration](#ironstar-nyp-26)
- [🟢 ironstar-nyp.25 Define Zenoh key expression patterns for event routing](#ironstar-nyp-25)
- [🟢 ironstar-nyp.22 Implement InfrastructureError type with database/network variants](#ironstar-nyp-22)
- [🟢 ironstar-ny3.17 Implement light-dark() theming with prefers-color-scheme](#ironstar-ny3-17)
- [🟢 ironstar-ny3.16 Configure OKLch color system with Open Props syntax](#ironstar-ny3-16)
- [🟢 ironstar-753.6 Implement chart SSE endpoint with signal-driven options](#ironstar-753-6)
- [🟢 ironstar-nyp.21 Implement observability initialization module](#ironstar-nyp-21)
- [🟢 ironstar-2nt.10 Define ErrorCode enum for HTTP error mapping](#ironstar-2nt-10)
- [🟢 ironstar-2nt.9 Define ChartSignals and ChartSelection types with ts-rs](#ironstar-2nt-9)
- [🟢 ironstar-nyp.19 Create EventBus trait abstraction](#ironstar-nyp-19)
- [🟢 ironstar-jqv.7 Implement AuthContext axum extractor](#ironstar-jqv-7)
- [🟢 ironstar-961 Implement DuckDB connection lifecycle management](#ironstar-961)
- [🟢 ironstar-9b1 Implement httpfs extension configuration for DuckDB](#ironstar-9b1)
- [🟢 ironstar-3gd Scientific Data Integration](#ironstar-3gd)
- [🟢 ironstar-nyp.15 Implement moka analytics cache with rkyv serialization](#ironstar-nyp-15)
- [🟢 ironstar-753.5 Implement ds-echarts build and test integration](#ironstar-753-5)
- [🟢 ironstar-753.4 Implement ds-echarts backend support](#ironstar-753-4)
- [🟢 ironstar-jqv Authentication and authorization](#ironstar-jqv)
- [🟢 ironstar-0tk Omicslake presentation slide deck](#ironstar-0tk)
- [🟢 ironstar-c7z Implement DuckDB remote data source integration (DuckLake/HF pattern)](#ironstar-c7z)
- [🟢 ironstar-amw Configure SQLite production PRAGMA settings (WAL, synchronous, cache)](#ironstar-amw)
- [🟢 ironstar-b9h Configure tower-http Brotli compression for SSE responses](#ironstar-b9h)
- [🟢 ironstar-09r Implement ds-echarts Lit web component wrapper](#ironstar-09r)
- [🟢 ironstar-e6k.8 Implement todo example route mounting](#ironstar-e6k-8)
- [🟢 ironstar-e6k.7 Implement todo_list_template rendering function](#ironstar-e6k-7)
- [🟢 ironstar-e6k.6 Implement GET /todos SSE feed endpoint](#ironstar-e6k-6)
- [🟢 ironstar-e6k.5 Implement delete_todo handler (POST /delete-todo)](#ironstar-e6k-5)
- [🟢 ironstar-e6k.4 Implement mark_todo handler (POST /mark-todo)](#ironstar-e6k-4)
- [🟢 ironstar-e6k.3 Implement add_todo handler (POST /add-todo)](#ironstar-e6k-3)
- [🟢 ironstar-e6k.2 Implement TodoListProjection with in-memory rebuild](#ironstar-e6k-2)
- [🟢 ironstar-e6k.1 Define Todo domain model (aggregate, events, commands)](#ironstar-e6k-1)
- [🟢 ironstar-r62.13 Wire all components together in main.rs](#ironstar-r62-13)
- [🟢 ironstar-r62.10 Implement component-level hypertext templates](#ironstar-r62-10)
- [🟢 ironstar-r62.3 Configure pre-commit hooks for code quality](#ironstar-r62-3)
- [🟢 ironstar-nyp.12 Implement DuckDB analytics service](#ironstar-nyp-12)
- [🟢 ironstar-nyp.11 Create Session axum extractor](#ironstar-nyp-11)
- [🟢 ironstar-nyp.10 Add session TTL cleanup background task](#ironstar-nyp-10)
- [🟢 ironstar-nyp.9 Implement SQLite session store with SessionStore trait](#ironstar-nyp-9)
- [🟢 ironstar-nyp.7 Implement ProjectionManager with in-memory state](#ironstar-nyp-7)
- [🟢 ironstar-nyp.4 Implement SQLite connection pooling and configuration](#ironstar-nyp-4)
- [🟢 ironstar-nyp.3 Implement SQLite event store with sqlx](#ironstar-nyp-3)
- [🟢 ironstar-ny3.14 Create web-components/components/ directory for vanilla web components](#ironstar-ny3-14)
- [🟢 ironstar-ny3.6 Copy Open Props UI component CSS files](#ironstar-ny3-6)
- [🟢 ironstar-ny3 Frontend build pipeline](#ironstar-ny3)
- [🟢 ironstar-2nt.8 Define application error types](#ironstar-2nt-8)
- [🟢 ironstar-zuv.4 Implement AggregateTestFramework with given/when/then DSL](#ironstar-zuv-4)
- [🟢 ironstar-nyp.32 Instrument Zenoh event bus with Prometheus metrics](#ironstar-nyp-32)
- [🟢 ironstar-nyp.28 Implement per-session Zenoh subscriptions for SSE streams](#ironstar-nyp-28)
- [🟢 ironstar-nyp.24 Add CQRS pipeline span context propagation](#ironstar-nyp-24)
- [🟢 ironstar-nyp.23 Configure dev vs prod logging subscribers](#ironstar-nyp-23)
- [🟢 ironstar-3gd.2 Implement event-driven cache invalidation](#ironstar-3gd-2)
- [🟢 ironstar-3gd.1 Implement cache-aside pattern for DuckDB analytics](#ironstar-3gd-1)
- [🟢 ironstar-jqv.11 Implement session rate limiting with sliding window](#ironstar-jqv-11)
- [🟢 ironstar-nyp.20 Implement Prometheus metrics endpoint and instrumentation](#ironstar-nyp-20)
- [🟢 ironstar-jqv.10 Implement OAuth CSRF state validation](#ironstar-jqv-10)
- [🟢 ironstar-jqv.9 Implement RequireAuth axum extractor](#ironstar-jqv-9)
- [🟢 ironstar-6lq.9 Add workspace lint configuration to Cargo.toml](#ironstar-6lq-9)
- [🟢 ironstar-jqv.8 Implement session regeneration for fixation prevention](#ironstar-jqv-8)
- [🟢 ironstar-jqv.6 Implement RBAC authorization patterns](#ironstar-jqv-6)
- [🟢 ironstar-ny3.15 Configure Rolldown for Lit web component bundling](#ironstar-ny3-15)
- [🟢 ironstar-nyp.18 Implement SSE ConnectionTracker with atomic counter](#ironstar-nyp-18)
- [🟢 ironstar-nyp.17 Implement EventUpcaster trait and UpcasterChain for schema evolution](#ironstar-nyp-17)
- [🟢 ironstar-89k Integrate analytics cache with dashboard SSE streams](#ironstar-89k)
- [🟢 ironstar-jqv.5 Create user_identities table for multi-provider support](#ironstar-jqv-5)
- [🟢 ironstar-jqv.4 Implement users table schema and UserService](#ironstar-jqv-4)
- [🟢 ironstar-nyp.14 Implement metrics and observability reference](#ironstar-nyp-14)
- [🟢 ironstar-nyp.13 Document error handling decisions](#ironstar-nyp-13)
- [🟢 ironstar-nqq.1 Implement CQRS performance tuning](#ironstar-nqq-1)
- [🟢 ironstar-jqv.3 Implement concrete session patterns](#ironstar-jqv-3)
- [🟢 ironstar-jqv.2 Implement session security hardening](#ironstar-jqv-2)
- [🟢 ironstar-jqv.1 Implement GitHub OAuth provider](#ironstar-jqv-1)
- [🟢 ironstar-nqq Performance optimization](#ironstar-nqq)
- [🟢 ironstar-avp Verify code examples compile and run](#ironstar-avp)
- [🟢 ironstar-ym1 Polish diagrams for visual consistency](#ironstar-ym1)
- [🟢 ironstar-63r Verify technical accuracy of benchmarks](#ironstar-63r)
- [🟢 ironstar-z4s Act 4: Expand vision slides](#ironstar-z4s)
- [🟢 ironstar-b8d Act 3: Expand web interface slides](#ironstar-b8d)
- [🟢 ironstar-a15 Act 2: Expand solution stack slides](#ironstar-a15)
- [🟢 ironstar-ubj Act 1: Expand data problem slides](#ironstar-ubj)
- [🟢 ironstar-6lq.8 Create reusable Rust CI workflow with workflow_call dispatch](#ironstar-6lq-8)
- [🟢 ironstar-rjs Document nixpkgs-unstable Darwin framework migration](#ironstar-rjs)
- [🟢 ironstar-apx.5 Add structured logging with tracing](#ironstar-apx-5)
- [🟢 ironstar-apx.4 Create .env.development template file](#ironstar-apx-4)
- [🟢 ironstar-apx.2 Create template parameters and conditional includes](#ironstar-apx-2)
- [🟢 ironstar-apx.1 Create BOOTSTRAP.md with complete setup instructions](#ironstar-apx-1)
- [🟢 ironstar-zuv.3 Create end-to-end handler tests](#ironstar-zuv-3)
- [🟢 ironstar-zuv.2 Create projection tests](#ironstar-zuv-2)
- [🟢 ironstar-zuv.1 Create event store integration tests](#ironstar-zuv-1)
- [🟢 ironstar-zuv Testing and integration](#ironstar-zuv)
- [🟢 ironstar-753.3 Set up Lucide icon build-time inlining](#ironstar-753-3)
- [🟢 ironstar-753.2 Implement sortable-list web component wrapper](#ironstar-753-2)
- [🟢 ironstar-753.1 Implement VegaChart web component wrapper](#ironstar-753-1)
- [🟢 ironstar-753 Third-party library integration](#ironstar-753)
- [🟢 ironstar-e6k Example application (Todo)](#ironstar-e6k)
- [🟢 ironstar-r62.15 Implement health check endpoint for process-compose](#ironstar-r62-15)
- [🟢 ironstar-r62.14 Implement dev-only hotreload SSE endpoint](#ironstar-r62-14)
- [🟢 ironstar-r62.12 Implement graceful shutdown signal handling](#ironstar-r62-12)
- [🟢 ironstar-r62.11 Implement router composition with feature routes](#ironstar-r62-11)
- [🟢 ironstar-r62.6 Implement command POST handlers](#ironstar-r62-6)
- [🟢 ironstar-r62.5 Implement SSE feed endpoint with event replay](#ironstar-r62-5)
- [🟢 ironstar-r62.4 Define AppState struct with all dependencies](#ironstar-r62-4)
- [🟢 ironstar-ny3.1 Create web-components/ project structure with package.json](#ironstar-ny3-1)
- [🟢 ironstar-nqq.2 Implement advanced performance patterns](#ironstar-nqq-2)
- [🟢 ironstar-k1z Final review and presentation dry-run](#ironstar-k1z)
- [🟢 ironstar-nor Research Mosaic visualization integration (TBD)](#ironstar-nor)
- [🟢 ironstar-apx.3 Define om CLI instantiation tests and metadata](#ironstar-apx-3)
- [🟢 ironstar-apx Documentation and template](#ironstar-apx)
- [🟢 ironstar-nyp.5 Implement tokio broadcast event bus](#ironstar-nyp-5)
- [🟢 ironstar-v4y.3 Define common-utils crate structure](#ironstar-v4y-3)
- [🟢 ironstar-v4y.2 Define common-types crate structure](#ironstar-v4y-2)
- [🟢 ironstar-v4y.1 Define common-enums crate structure](#ironstar-v4y-1)
- [🟢 ironstar-v4y Multi-crate workspace decomposition](#ironstar-v4y)
- [⚫ ironstar-2nt.1 Initialize src/ directory structure with modular organization](#ironstar-2nt-1)
- [⚫ ironstar-6lq.6 Add Rust checks to flake.checks for CI integration](#ironstar-6lq-6)
- [⚫ ironstar-6lq.5 Verify cargo check passes with workspace configuration](#ironstar-6lq-5)
- [⚫ ironstar-6lq.4 Set up per-crate crate.nix pattern for crane args](#ironstar-6lq-4)
- [⚫ ironstar-6lq.3 Configure Cargo.toml with workspace structure (resolver = 2)](#ironstar-6lq-3)
- [⚫ ironstar-6lq.2 Add rust-toolchain.toml with required components](#ironstar-6lq-2)
- [⚫ ironstar-6lq.1 Integrate rust-flake patterns (crane, rust-overlay)](#ironstar-6lq-1)
- [⚫ ironstar-cxe.5 Create .gitignore with comprehensive patterns](#ironstar-cxe-5)
- [⚫ ironstar-cxe.4 Create initial git commit with generated structure](#ironstar-cxe-4)
- [⚫ ironstar-cxe.3 Verify nix develop enters working development shell](#ironstar-cxe-3)
- [⚫ ironstar-cxe.2 Configure secrets management and string replacement](#ironstar-cxe-2)
- [⚫ ironstar-cxe.1 Run om init with typescript-nix-template parameters](#ironstar-cxe-1)
- [⚫ ironstar-cxe Template instantiation](#ironstar-cxe)
- [⚫ ironstar-edx Review narrative arc and timing estimates](#ironstar-edx)
- [⚫ ironstar-nyp.33 Implement session cleanup background task](#ironstar-nyp-33)
- [⚫ ironstar-9oj Implement cache invalidation for analytics queries](#ironstar-9oj)
- [⚫ ironstar-r5f ironstar-6lq](#ironstar-r5f)
- [⚫ ironstar-nyp.16 Implement DualEventBus for tokio::broadcast to Zenoh migration](#ironstar-nyp-16)

---

## Dependency Graph

```mermaid
graph TD
    classDef open fill:#50FA7B,stroke:#333,color:#000
    classDef inprogress fill:#8BE9FD,stroke:#333,color:#000
    classDef blocked fill:#FF5555,stroke:#333,color:#000
    classDef closed fill:#6272A4,stroke:#333,color:#fff

    ironstar-09r["ironstar-09r<br/>Implement ds-echarts Lit web componen..."]
    class ironstar-09r open
    ironstar-0tk["ironstar-0tk<br/>Omicslake presentation slide deck"]
    class ironstar-0tk open
    ironstar-2nt["ironstar-2nt<br/>Domain layer"]
    class ironstar-2nt open
    ironstar-2nt1["ironstar-2nt.1<br/>Initialize src/ directory structure w..."]
    class ironstar-2nt1 closed
    ironstar-2nt10["ironstar-2nt.10<br/>Define ErrorCode enum for HTTP error ..."]
    class ironstar-2nt10 open
    ironstar-2nt11["ironstar-2nt.11<br/>Add version(&self) -&gt; u64 to Aggre..."]
    class ironstar-2nt11 open
    ironstar-2nt12["ironstar-2nt.12<br/>Implement UUID-tracked error type for..."]
    class ironstar-2nt12 open
    ironstar-2nt13["ironstar-2nt.13<br/>Enforce async/sync boundary via modul..."]
    class ironstar-2nt13 open
    ironstar-2nt2["ironstar-2nt.2<br/>Define algebraic domain types and agg..."]
    class ironstar-2nt2 open
    ironstar-2nt3["ironstar-2nt.3<br/>Implement value objects and smart con..."]
    class ironstar-2nt3 open
    ironstar-2nt4["ironstar-2nt.4<br/>Design aggregate root state machines"]
    class ironstar-2nt4 open
    ironstar-2nt5["ironstar-2nt.5<br/>Create Datastar signal types with ts-..."]
    class ironstar-2nt5 open
    ironstar-2nt6["ironstar-2nt.6<br/>Enforce camelCase convention for Data..."]
    class ironstar-2nt6 open
    ironstar-2nt7["ironstar-2nt.7<br/>Implement command validation pattern ..."]
    class ironstar-2nt7 open
    ironstar-2nt8["ironstar-2nt.8<br/>Define application error types"]
    class ironstar-2nt8 open
    ironstar-2nt9["ironstar-2nt.9<br/>Define ChartSignals and ChartSelectio..."]
    class ironstar-2nt9 open
    ironstar-3gd["ironstar-3gd<br/>Scientific Data Integration"]
    class ironstar-3gd open
    ironstar-3gd1["ironstar-3gd.1<br/>Implement cache-aside pattern for Duc..."]
    class ironstar-3gd1 open
    ironstar-3gd2["ironstar-3gd.2<br/>Implement event-driven cache invalida..."]
    class ironstar-3gd2 open
    ironstar-3gd3["ironstar-3gd.3<br/>Implement CacheDependency struct for ..."]
    class ironstar-3gd3 open
    ironstar-63r["ironstar-63r<br/>Verify technical accuracy of benchmarks"]
    class ironstar-63r open
    ironstar-6lq["ironstar-6lq<br/>Rust workspace integration"]
    class ironstar-6lq open
    ironstar-6lq1["ironstar-6lq.1<br/>Integrate rust-flake patterns (crane,..."]
    class ironstar-6lq1 closed
    ironstar-6lq2["ironstar-6lq.2<br/>Add rust-toolchain.toml with required..."]
    class ironstar-6lq2 closed
    ironstar-6lq3["ironstar-6lq.3<br/>Configure Cargo.toml with workspace s..."]
    class ironstar-6lq3 closed
    ironstar-6lq4["ironstar-6lq.4<br/>Set up per-crate crate.nix pattern fo..."]
    class ironstar-6lq4 closed
    ironstar-6lq5["ironstar-6lq.5<br/>Verify cargo check passes with worksp..."]
    class ironstar-6lq5 closed
    ironstar-6lq6["ironstar-6lq.6<br/>Add Rust checks to flake.checks for C..."]
    class ironstar-6lq6 closed
    ironstar-6lq7["ironstar-6lq.7<br/>Add Rust to CI matrix and extend inhe..."]
    class ironstar-6lq7 open
    ironstar-6lq8["ironstar-6lq.8<br/>Create reusable Rust CI workflow with..."]
    class ironstar-6lq8 open
    ironstar-6lq9["ironstar-6lq.9<br/>Add workspace lint configuration to C..."]
    class ironstar-6lq9 open
    ironstar-753["ironstar-753<br/>Third-party library integration"]
    class ironstar-753 open
    ironstar-7531["ironstar-753.1<br/>Implement VegaChart web component wra..."]
    class ironstar-7531 open
    ironstar-7532["ironstar-753.2<br/>Implement sortable-list web component..."]
    class ironstar-7532 open
    ironstar-7533["ironstar-753.3<br/>Set up Lucide icon build-time inlining"]
    class ironstar-7533 open
    ironstar-7534["ironstar-753.4<br/>Implement ds-echarts backend support"]
    class ironstar-7534 open
    ironstar-7535["ironstar-753.5<br/>Implement ds-echarts build and test i..."]
    class ironstar-7535 open
    ironstar-7536["ironstar-753.6<br/>Implement chart SSE endpoint with sig..."]
    class ironstar-7536 open
    ironstar-89k["ironstar-89k<br/>Integrate analytics cache with dashbo..."]
    class ironstar-89k open
    ironstar-961["ironstar-961<br/>Implement DuckDB connection lifecycle..."]
    class ironstar-961 open
    ironstar-9b1["ironstar-9b1<br/>Implement httpfs extension configurat..."]
    class ironstar-9b1 open
    ironstar-9oj["ironstar-9oj<br/>Implement cache invalidation for anal..."]
    class ironstar-9oj closed
    ironstar-a15["ironstar-a15<br/>Act 2: Expand solution stack slides"]
    class ironstar-a15 open
    ironstar-amw["ironstar-amw<br/>Configure SQLite production PRAGMA se..."]
    class ironstar-amw open
    ironstar-apx["ironstar-apx<br/>Documentation and template"]
    class ironstar-apx open
    ironstar-apx1["ironstar-apx.1<br/>Create BOOTSTRAP.md with complete set..."]
    class ironstar-apx1 open
    ironstar-apx2["ironstar-apx.2<br/>Create template parameters and condit..."]
    class ironstar-apx2 open
    ironstar-apx3["ironstar-apx.3<br/>Define om CLI instantiation tests and..."]
    class ironstar-apx3 open
    ironstar-apx4["ironstar-apx.4<br/>Create .env.development template file"]
    class ironstar-apx4 open
    ironstar-apx5["ironstar-apx.5<br/>Add structured logging with tracing"]
    class ironstar-apx5 open
    ironstar-avp["ironstar-avp<br/>Verify code examples compile and run"]
    class ironstar-avp open
    ironstar-b8d["ironstar-b8d<br/>Act 3: Expand web interface slides"]
    class ironstar-b8d open
    ironstar-b9h["ironstar-b9h<br/>Configure tower-http Brotli compressi..."]
    class ironstar-b9h open
    ironstar-c7z["ironstar-c7z<br/>Implement DuckDB remote data source i..."]
    class ironstar-c7z open
    ironstar-cxe["ironstar-cxe<br/>Template instantiation"]
    class ironstar-cxe closed
    ironstar-cxe1["ironstar-cxe.1<br/>Run om init with typescript-nix-templ..."]
    class ironstar-cxe1 closed
    ironstar-cxe2["ironstar-cxe.2<br/>Configure secrets management and stri..."]
    class ironstar-cxe2 closed
    ironstar-cxe3["ironstar-cxe.3<br/>Verify nix develop enters working dev..."]
    class ironstar-cxe3 closed
    ironstar-cxe4["ironstar-cxe.4<br/>Create initial git commit with genera..."]
    class ironstar-cxe4 closed
    ironstar-cxe5["ironstar-cxe.5<br/>Create .gitignore with comprehensive ..."]
    class ironstar-cxe5 closed
    ironstar-e6k["ironstar-e6k<br/>Example application (Todo)"]
    class ironstar-e6k open
    ironstar-e6k1["ironstar-e6k.1<br/>Define Todo domain model (aggregate, ..."]
    class ironstar-e6k1 open
    ironstar-e6k2["ironstar-e6k.2<br/>Implement TodoListProjection with in-..."]
    class ironstar-e6k2 open
    ironstar-e6k3["ironstar-e6k.3<br/>Implement add_todo handler (POST /add..."]
    class ironstar-e6k3 open
    ironstar-e6k4["ironstar-e6k.4<br/>Implement mark_todo handler (POST /ma..."]
    class ironstar-e6k4 open
    ironstar-e6k5["ironstar-e6k.5<br/>Implement delete_todo handler (POST /..."]
    class ironstar-e6k5 open
    ironstar-e6k6["ironstar-e6k.6<br/>Implement GET /todos SSE feed endpoint"]
    class ironstar-e6k6 open
    ironstar-e6k7["ironstar-e6k.7<br/>Implement todo_list_template renderin..."]
    class ironstar-e6k7 open
    ironstar-e6k8["ironstar-e6k.8<br/>Implement todo example route mounting"]
    class ironstar-e6k8 open
    ironstar-edx["ironstar-edx<br/>Review narrative arc and timing estim..."]
    class ironstar-edx closed
    ironstar-f8b["ironstar-f8b<br/>Process compose integration"]
    class ironstar-f8b open
    ironstar-f8b1["ironstar-f8b.1<br/>Integrate process-compose-flake patte..."]
    class ironstar-f8b1 open
    ironstar-f8b2["ironstar-f8b.2<br/>Configure process-compose.yaml for de..."]
    class ironstar-f8b2 open
    ironstar-f8b3["ironstar-f8b.3<br/>Set up service orchestration (fronten..."]
    class ironstar-f8b3 open
    ironstar-f8b4["ironstar-f8b.4<br/>Configure cargo-watch to curl hotrelo..."]
    class ironstar-f8b4 open
    ironstar-f8b5["ironstar-f8b.5<br/>Verify process-compose up works with ..."]
    class ironstar-f8b5 open
    ironstar-jqv["ironstar-jqv<br/>Authentication and authorization"]
    class ironstar-jqv open
    ironstar-jqv1["ironstar-jqv.1<br/>Implement GitHub OAuth provider"]
    class ironstar-jqv1 open
    ironstar-jqv10["ironstar-jqv.10<br/>Implement OAuth CSRF state validation"]
    class ironstar-jqv10 open
    ironstar-jqv11["ironstar-jqv.11<br/>Implement session rate limiting with ..."]
    class ironstar-jqv11 open
    ironstar-jqv12["ironstar-jqv.12<br/>Implement session regeneration and us..."]
    class ironstar-jqv12 open
    ironstar-jqv2["ironstar-jqv.2<br/>Implement session security hardening"]
    class ironstar-jqv2 open
    ironstar-jqv3["ironstar-jqv.3<br/>Implement concrete session patterns"]
    class ironstar-jqv3 open
    ironstar-jqv4["ironstar-jqv.4<br/>Implement users table schema and User..."]
    class ironstar-jqv4 open
    ironstar-jqv5["ironstar-jqv.5<br/>Create user_identities table for mult..."]
    class ironstar-jqv5 open
    ironstar-jqv6["ironstar-jqv.6<br/>Implement RBAC authorization patterns"]
    class ironstar-jqv6 open
    ironstar-jqv7["ironstar-jqv.7<br/>Implement AuthContext axum extractor"]
    class ironstar-jqv7 open
    ironstar-jqv8["ironstar-jqv.8<br/>Implement session regeneration for fi..."]
    class ironstar-jqv8 open
    ironstar-jqv9["ironstar-jqv.9<br/>Implement RequireAuth axum extractor"]
    class ironstar-jqv9 open
    ironstar-k1z["ironstar-k1z<br/>Final review and presentation dry-run"]
    class ironstar-k1z open
    ironstar-nor["ironstar-nor<br/>Research Mosaic visualization integra..."]
    class ironstar-nor open
    ironstar-nqq["ironstar-nqq<br/>Performance optimization"]
    class ironstar-nqq open
    ironstar-nqq1["ironstar-nqq.1<br/>Implement CQRS performance tuning"]
    class ironstar-nqq1 open
    ironstar-nqq2["ironstar-nqq.2<br/>Implement advanced performance patterns"]
    class ironstar-nqq2 open
    ironstar-ny3["ironstar-ny3<br/>Frontend build pipeline"]
    class ironstar-ny3 open
    ironstar-ny31["ironstar-ny3.1<br/>Create web-components/ project struct..."]
    class ironstar-ny31 open
    ironstar-ny310["ironstar-ny3.10<br/>Configure ts-rs export directory and ..."]
    class ironstar-ny310 open
    ironstar-ny311["ironstar-ny3.11<br/>Create static/dist/ output directory ..."]
    class ironstar-ny311 open
    ironstar-ny312["ironstar-ny3.12<br/>Implement manifest.json parser for ha..."]
    class ironstar-ny312 open
    ironstar-ny313["ironstar-ny3.13<br/>Implement rust-embed conditional asse..."]
    class ironstar-ny313 open
    ironstar-ny314["ironstar-ny3.14<br/>Create web-components/components/ dir..."]
    class ironstar-ny314 open
    ironstar-ny315["ironstar-ny3.15<br/>Configure Rolldown for Lit web compon..."]
    class ironstar-ny315 open
    ironstar-ny316["ironstar-ny3.16<br/>Configure OKLch color system with Ope..."]
    class ironstar-ny316 open
    ironstar-ny317["ironstar-ny3.17<br/>Implement light-dark() theming with p..."]
    class ironstar-ny317 open
    ironstar-ny32["ironstar-ny3.2<br/>Configure Rolldown bundler with conte..."]
    class ironstar-ny32 open
    ironstar-ny33["ironstar-ny3.3<br/>Setup PostCSS configuration for moder..."]
    class ironstar-ny33 open
    ironstar-ny34["ironstar-ny3.4<br/>Setup Open Props design tokens and th..."]
    class ironstar-ny34 open
    ironstar-ny35["ironstar-ny3.5<br/>Configure CSS cascade layers for pred..."]
    class ironstar-ny35 open
    ironstar-ny36["ironstar-ny3.6<br/>Copy Open Props UI component CSS files"]
    class ironstar-ny36 open
    ironstar-ny37["ironstar-ny3.7<br/>Create TypeScript configuration (tsco..."]
    class ironstar-ny37 open
    ironstar-ny38["ironstar-ny3.8<br/>Create web-components/index.ts entry ..."]
    class ironstar-ny38 open
    ironstar-ny39["ironstar-ny3.9<br/>Add ts-rs dependency to Cargo.toml"]
    class ironstar-ny39 open
    ironstar-nyp["ironstar-nyp<br/>Event sourcing infrastructure"]
    class ironstar-nyp open
    ironstar-nyp1["ironstar-nyp.1<br/>Create database migrations/ directory..."]
    class ironstar-nyp1 open
    ironstar-nyp10["ironstar-nyp.10<br/>Add session TTL cleanup background task"]
    class ironstar-nyp10 open
    ironstar-nyp11["ironstar-nyp.11<br/>Create Session axum extractor"]
    class ironstar-nyp11 open
    ironstar-nyp12["ironstar-nyp.12<br/>Implement DuckDB analytics service"]
    class ironstar-nyp12 open
    ironstar-nyp13["ironstar-nyp.13<br/>Document error handling decisions"]
    class ironstar-nyp13 open
    ironstar-nyp14["ironstar-nyp.14<br/>Implement metrics and observability r..."]
    class ironstar-nyp14 open
    ironstar-nyp15["ironstar-nyp.15<br/>Implement moka analytics cache with r..."]
    class ironstar-nyp15 open
    ironstar-nyp16["ironstar-nyp.16<br/>Implement DualEventBus for tokio::bro..."]
    class ironstar-nyp16 closed
    ironstar-nyp17["ironstar-nyp.17<br/>Implement EventUpcaster trait and Upc..."]
    class ironstar-nyp17 open
    ironstar-nyp18["ironstar-nyp.18<br/>Implement SSE ConnectionTracker with ..."]
    class ironstar-nyp18 open
    ironstar-nyp19["ironstar-nyp.19<br/>Create EventBus trait abstraction"]
    class ironstar-nyp19 open
    ironstar-nyp2["ironstar-nyp.2<br/>Create EventStore trait abstraction"]
    class ironstar-nyp2 open
    ironstar-nyp20["ironstar-nyp.20<br/>Implement Prometheus metrics endpoint..."]
    class ironstar-nyp20 open
    ironstar-nyp21["ironstar-nyp.21<br/>Implement observability initializatio..."]
    class ironstar-nyp21 open
    ironstar-nyp22["ironstar-nyp.22<br/>Implement InfrastructureError type wi..."]
    class ironstar-nyp22 open
    ironstar-nyp23["ironstar-nyp.23<br/>Configure dev vs prod logging subscri..."]
    class ironstar-nyp23 open
    ironstar-nyp24["ironstar-nyp.24<br/>Add CQRS pipeline span context propag..."]
    class ironstar-nyp24 open
    ironstar-nyp25["ironstar-nyp.25<br/>Define Zenoh key expression patterns ..."]
    class ironstar-nyp25 open
    ironstar-nyp26["ironstar-nyp.26<br/>Create Zenoh embedded router configur..."]
    class ironstar-nyp26 open
    ironstar-nyp27["ironstar-nyp.27<br/>Implement ZenohEventBus struct with p..."]
    class ironstar-nyp27 open
    ironstar-nyp28["ironstar-nyp.28<br/>Implement per-session Zenoh subscript..."]
    class ironstar-nyp28 open
    ironstar-nyp29["ironstar-nyp.29<br/>Implement error propagation pattern t..."]
    class ironstar-nyp29 open
    ironstar-nyp3["ironstar-nyp.3<br/>Implement SQLite event store with sqlx"]
    class ironstar-nyp3 open
    ironstar-nyp30["ironstar-nyp.30<br/>Implement observability initializatio..."]
    class ironstar-nyp30 open
    ironstar-nyp31["ironstar-nyp.31<br/>Implement health check endpoints (/he..."]
    class ironstar-nyp31 open
    ironstar-nyp32["ironstar-nyp.32<br/>Instrument Zenoh event bus with Prome..."]
    class ironstar-nyp32 open
    ironstar-nyp33["ironstar-nyp.33<br/>Implement session cleanup background ..."]
    class ironstar-nyp33 closed
    ironstar-nyp4["ironstar-nyp.4<br/>Implement SQLite connection pooling a..."]
    class ironstar-nyp4 open
    ironstar-nyp5["ironstar-nyp.5<br/>Implement tokio broadcast event bus"]
    class ironstar-nyp5 open
    ironstar-nyp6["ironstar-nyp.6<br/>Create Projection trait for read models"]
    class ironstar-nyp6 open
    ironstar-nyp7["ironstar-nyp.7<br/>Implement ProjectionManager with in-m..."]
    class ironstar-nyp7 open
    ironstar-nyp8["ironstar-nyp.8<br/>Implement SSE 15-second keep-alive co..."]
    class ironstar-nyp8 open
    ironstar-nyp9["ironstar-nyp.9<br/>Implement SQLite session store with S..."]
    class ironstar-nyp9 open
    ironstar-r5f["ironstar-r5f<br/>ironstar-6lq"]
    class ironstar-r5f closed
    ironstar-r62["ironstar-r62<br/>Presentation layer"]
    class ironstar-r62 open
    ironstar-r621["ironstar-r62.1<br/>Add justfile with development and bui..."]
    class ironstar-r621 open
    ironstar-r6210["ironstar-r62.10<br/>Implement component-level hypertext t..."]
    class ironstar-r6210 open
    ironstar-r6211["ironstar-r62.11<br/>Implement router composition with fea..."]
    class ironstar-r6211 open
    ironstar-r6212["ironstar-r62.12<br/>Implement graceful shutdown signal ha..."]
    class ironstar-r6212 open
    ironstar-r6213["ironstar-r62.13<br/>Wire all components together in main.rs"]
    class ironstar-r6213 open
    ironstar-r6214["ironstar-r62.14<br/>Implement dev-only hotreload SSE endp..."]
    class ironstar-r6214 open
    ironstar-r6215["ironstar-r62.15<br/>Implement health check endpoint for p..."]
    class ironstar-r6215 open
    ironstar-r622["ironstar-r62.2<br/>Create devShell module with tools and..."]
    class ironstar-r622 open
    ironstar-r623["ironstar-r62.3<br/>Configure pre-commit hooks for code q..."]
    class ironstar-r623 open
    ironstar-r624["ironstar-r62.4<br/>Define AppState struct with all depen..."]
    class ironstar-r624 open
    ironstar-r625["ironstar-r62.5<br/>Implement SSE feed endpoint with even..."]
    class ironstar-r625 open
    ironstar-r626["ironstar-r62.6<br/>Implement command POST handlers"]
    class ironstar-r626 open
    ironstar-r627["ironstar-r62.7<br/>Implement query GET handlers"]
    class ironstar-r627 open
    ironstar-r628["ironstar-r62.8<br/>Implement RenderableToDatastar conver..."]
    class ironstar-r628 open
    ironstar-r629["ironstar-r62.9<br/>Create base layout template with Data..."]
    class ironstar-r629 open
    ironstar-rjs["ironstar-rjs<br/>Document nixpkgs-unstable Darwin fram..."]
    class ironstar-rjs open
    ironstar-ubj["ironstar-ubj<br/>Act 1: Expand data problem slides"]
    class ironstar-ubj open
    ironstar-v4y["ironstar-v4y<br/>Multi-crate workspace decomposition"]
    class ironstar-v4y open
    ironstar-v4y1["ironstar-v4y.1<br/>Define common-enums crate structure"]
    class ironstar-v4y1 open
    ironstar-v4y2["ironstar-v4y.2<br/>Define common-types crate structure"]
    class ironstar-v4y2 open
    ironstar-v4y3["ironstar-v4y.3<br/>Define common-utils crate structure"]
    class ironstar-v4y3 open
    ironstar-ym1["ironstar-ym1<br/>Polish diagrams for visual consistency"]
    class ironstar-ym1 open
    ironstar-z4s["ironstar-z4s<br/>Act 4: Expand vision slides"]
    class ironstar-z4s open
    ironstar-zuv["ironstar-zuv<br/>Testing and integration"]
    class ironstar-zuv open
    ironstar-zuv1["ironstar-zuv.1<br/>Create event store integration tests"]
    class ironstar-zuv1 open
    ironstar-zuv2["ironstar-zuv.2<br/>Create projection tests"]
    class ironstar-zuv2 open
    ironstar-zuv3["ironstar-zuv.3<br/>Create end-to-end handler tests"]
    class ironstar-zuv3 open
    ironstar-zuv4["ironstar-zuv.4<br/>Implement AggregateTestFramework with..."]
    class ironstar-zuv4 open

    ironstar-09r ==> ironstar-2nt5
    ironstar-09r ==> ironstar-2nt9
    ironstar-09r ==> ironstar-ny3
    ironstar-09r ==> ironstar-r6210
    ironstar-09r ==> ironstar-r625
    ironstar-2nt ==> ironstar-6lq5
    ironstar-2nt1 -.-> ironstar-2nt
    ironstar-2nt10 -.-> ironstar-2nt
    ironstar-2nt10 ==> ironstar-2nt8
    ironstar-2nt11 -.-> ironstar-2nt
    ironstar-2nt12 -.-> ironstar-2nt
    ironstar-2nt13 -.-> ironstar-2nt
    ironstar-2nt2 -.-> ironstar-2nt
    ironstar-2nt2 ==> ironstar-2nt1
    ironstar-2nt3 -.-> ironstar-2nt
    ironstar-2nt3 ==> ironstar-2nt2
    ironstar-2nt4 -.-> ironstar-2nt
    ironstar-2nt4 ==> ironstar-2nt11
    ironstar-2nt4 ==> ironstar-2nt3
    ironstar-2nt5 -.-> ironstar-2nt
    ironstar-2nt5 ==> ironstar-2nt2
    ironstar-2nt6 -.-> ironstar-2nt
    ironstar-2nt6 ==> ironstar-2nt5
    ironstar-2nt7 -.-> ironstar-2nt
    ironstar-2nt7 ==> ironstar-2nt4
    ironstar-2nt8 -.-> ironstar-2nt
    ironstar-2nt8 ==> ironstar-2nt2
    ironstar-2nt9 -.-> ironstar-2nt
    ironstar-2nt9 ==> ironstar-2nt2
    ironstar-3gd ==> ironstar-2nt
    ironstar-3gd ==> ironstar-nyp
    ironstar-3gd1 -.-> ironstar-3gd
    ironstar-3gd2 -.-> ironstar-3gd
    ironstar-3gd2 -.-> ironstar-3gd3
    ironstar-3gd3 -.-> ironstar-3gd
    ironstar-3gd3 -.-> ironstar-nyp25
    ironstar-3gd3 -.-> ironstar-nyp27
    ironstar-63r -.-> ironstar-0tk
    ironstar-6lq ==> ironstar-cxe
    ironstar-6lq1 -.-> ironstar-6lq
    ironstar-6lq2 -.-> ironstar-6lq
    ironstar-6lq2 ==> ironstar-6lq1
    ironstar-6lq3 -.-> ironstar-6lq
    ironstar-6lq3 ==> ironstar-6lq2
    ironstar-6lq4 -.-> ironstar-6lq
    ironstar-6lq4 ==> ironstar-6lq3
    ironstar-6lq5 -.-> ironstar-6lq
    ironstar-6lq5 ==> ironstar-6lq4
    ironstar-6lq6 -.-> ironstar-6lq
    ironstar-6lq6 ==> ironstar-6lq4
    ironstar-6lq7 -.-> ironstar-6lq
    ironstar-6lq7 ==> ironstar-6lq6
    ironstar-6lq7 ==> ironstar-6lq8
    ironstar-6lq8 -.-> ironstar-6lq
    ironstar-6lq9 -.-> ironstar-6lq
    ironstar-753 ==> ironstar-09r
    ironstar-753 ==> ironstar-3gd
    ironstar-753 ==> ironstar-nor
    ironstar-753 ==> ironstar-ny3
    ironstar-7531 -.-> ironstar-753
    ironstar-7531 ==> ironstar-ny314
    ironstar-7532 -.-> ironstar-753
    ironstar-7532 ==> ironstar-ny314
    ironstar-7533 -.-> ironstar-753
    ironstar-7533 ==> ironstar-ny32
    ironstar-7534 -.-> ironstar-753
    ironstar-7534 ==> ironstar-nyp12
    ironstar-7534 ==> ironstar-nyp8
    ironstar-7535 -.-> ironstar-753
    ironstar-7535 ==> ironstar-7534
    ironstar-7536 -.-> ironstar-753
    ironstar-89k -.-> ironstar-3gd
    ironstar-89k ==> ironstar-nyp12
    ironstar-961 -.-> ironstar-3gd
    ironstar-9b1 -.-> ironstar-3gd
    ironstar-9oj -.-> ironstar-3gd
    ironstar-9oj ==> ironstar-nyp12
    ironstar-a15 -.-> ironstar-0tk
    ironstar-a15 ==> ironstar-edx
    ironstar-amw -.-> ironstar-nyp
    ironstar-amw ==> ironstar-nyp3
    ironstar-apx ==> ironstar-rjs
    ironstar-apx ==> ironstar-zuv
    ironstar-apx1 -.-> ironstar-apx
    ironstar-apx1 ==> ironstar-r6213
    ironstar-apx2 ==> ironstar-6lq1
    ironstar-apx2 -.-> ironstar-apx
    ironstar-apx3 -.-> ironstar-apx
    ironstar-apx3 ==> ironstar-apx2
    ironstar-apx4 -.-> ironstar-apx
    ironstar-apx4 ==> ironstar-nyp3
    ironstar-apx5 -.-> ironstar-apx
    ironstar-apx5 ==> ironstar-r6213
    ironstar-avp -.-> ironstar-0tk
    ironstar-b8d -.-> ironstar-0tk
    ironstar-b8d ==> ironstar-edx
    ironstar-b9h -.-> ironstar-r62
    ironstar-b9h ==> ironstar-r625
    ironstar-c7z -.-> ironstar-3gd
    ironstar-c7z ==> ironstar-9b1
    ironstar-c7z ==> ironstar-nyp12
    ironstar-cxe1 -.-> ironstar-cxe
    ironstar-cxe2 -.-> ironstar-cxe
    ironstar-cxe2 ==> ironstar-cxe1
    ironstar-cxe3 -.-> ironstar-cxe
    ironstar-cxe3 ==> ironstar-cxe1
    ironstar-cxe4 -.-> ironstar-cxe
    ironstar-cxe4 ==> ironstar-cxe2
    ironstar-cxe4 ==> ironstar-cxe3
    ironstar-cxe5 -.-> ironstar-cxe
    ironstar-e6k ==> ironstar-r62
    ironstar-e6k1 ==> ironstar-2nt2
    ironstar-e6k1 ==> ironstar-2nt4
    ironstar-e6k1 -.-> ironstar-e6k
    ironstar-e6k2 -.-> ironstar-e6k
    ironstar-e6k2 ==> ironstar-e6k1
    ironstar-e6k2 ==> ironstar-nyp7
    ironstar-e6k3 -.-> ironstar-e6k
    ironstar-e6k3 ==> ironstar-e6k2
    ironstar-e6k3 ==> ironstar-r626
    ironstar-e6k4 -.-> ironstar-e6k
    ironstar-e6k4 ==> ironstar-e6k3
    ironstar-e6k5 -.-> ironstar-e6k
    ironstar-e6k5 ==> ironstar-e6k3
    ironstar-e6k6 -.-> ironstar-e6k
    ironstar-e6k6 ==> ironstar-e6k2
    ironstar-e6k6 ==> ironstar-r625
    ironstar-e6k7 -.-> ironstar-e6k
    ironstar-e6k7 ==> ironstar-r6210
    ironstar-e6k8 -.-> ironstar-e6k
    ironstar-e6k8 ==> ironstar-e6k3
    ironstar-e6k8 ==> ironstar-e6k4
    ironstar-e6k8 ==> ironstar-e6k5
    ironstar-e6k8 ==> ironstar-e6k6
    ironstar-e6k8 ==> ironstar-e6k7
    ironstar-edx -.-> ironstar-0tk
    ironstar-f8b ==> ironstar-6lq
    ironstar-f8b1 -.-> ironstar-f8b
    ironstar-f8b2 -.-> ironstar-f8b
    ironstar-f8b2 ==> ironstar-f8b1
    ironstar-f8b3 -.-> ironstar-f8b
    ironstar-f8b3 ==> ironstar-f8b2
    ironstar-f8b4 -.-> ironstar-f8b
    ironstar-f8b4 ==> ironstar-f8b3
    ironstar-f8b5 -.-> ironstar-f8b
    ironstar-f8b5 ==> ironstar-f8b4
    ironstar-jqv -.-> ironstar-nyp
    ironstar-jqv1 ==> ironstar-2nt2
    ironstar-jqv1 -.-> ironstar-jqv
    ironstar-jqv1 ==> ironstar-jqv10
    ironstar-jqv1 ==> ironstar-jqv4
    ironstar-jqv1 ==> ironstar-nyp11
    ironstar-jqv1 -.-> ironstar-nyp9
    ironstar-jqv10 -.-> ironstar-jqv
    ironstar-jqv11 -.-> ironstar-jqv
    ironstar-jqv12 -.-> ironstar-jqv
    ironstar-jqv2 -.-> ironstar-jqv
    ironstar-jqv2 ==> ironstar-jqv1
    ironstar-jqv2 -.-> ironstar-nyp9
    ironstar-jqv3 -.-> ironstar-jqv
    ironstar-jqv3 ==> ironstar-jqv2
    ironstar-jqv3 -.-> ironstar-nyp9
    ironstar-jqv4 -.-> ironstar-jqv
    ironstar-jqv5 -.-> ironstar-jqv
    ironstar-jqv5 ==> ironstar-jqv4
    ironstar-jqv6 -.-> ironstar-jqv
    ironstar-jqv7 -.-> ironstar-jqv
    ironstar-jqv8 -.-> ironstar-jqv
    ironstar-jqv9 -.-> ironstar-jqv
    ironstar-jqv9 ==> ironstar-jqv7
    ironstar-k1z -.-> ironstar-0tk
    ironstar-k1z ==> ironstar-63r
    ironstar-k1z ==> ironstar-a15
    ironstar-k1z ==> ironstar-avp
    ironstar-k1z ==> ironstar-b8d
    ironstar-k1z ==> ironstar-ubj
    ironstar-k1z ==> ironstar-ym1
    ironstar-k1z ==> ironstar-z4s
    ironstar-nqq1 -.-> ironstar-nqq
    ironstar-nqq2 -.-> ironstar-nqq
    ironstar-nqq2 ==> ironstar-nqq1
    ironstar-ny3 ==> ironstar-6lq7
    ironstar-ny31 -.-> ironstar-ny3
    ironstar-ny310 ==> ironstar-2nt5
    ironstar-ny310 -.-> ironstar-ny3
    ironstar-ny310 ==> ironstar-ny39
    ironstar-ny311 -.-> ironstar-ny3
    ironstar-ny311 ==> ironstar-ny32
    ironstar-ny312 -.-> ironstar-ny3
    ironstar-ny312 ==> ironstar-ny311
    ironstar-ny313 -.-> ironstar-ny3
    ironstar-ny313 ==> ironstar-ny312
    ironstar-ny314 -.-> ironstar-ny3
    ironstar-ny314 ==> ironstar-ny38
    ironstar-ny315 -.-> ironstar-ny3
    ironstar-ny316 -.-> ironstar-ny3
    ironstar-ny317 -.-> ironstar-ny3
    ironstar-ny32 -.-> ironstar-ny3
    ironstar-ny32 ==> ironstar-ny31
    ironstar-ny33 -.-> ironstar-ny3
    ironstar-ny33 ==> ironstar-ny31
    ironstar-ny34 -.-> ironstar-ny3
    ironstar-ny34 ==> ironstar-ny33
    ironstar-ny35 -.-> ironstar-ny3
    ironstar-ny35 ==> ironstar-ny34
    ironstar-ny36 -.-> ironstar-ny3
    ironstar-ny36 ==> ironstar-ny35
    ironstar-ny37 -.-> ironstar-ny3
    ironstar-ny37 ==> ironstar-ny31
    ironstar-ny38 -.-> ironstar-ny3
    ironstar-ny38 ==> ironstar-ny37
    ironstar-ny39 -.-> ironstar-ny3
    ironstar-nyp ==> ironstar-2nt
    ironstar-nyp1 ==> ironstar-2nt11
    ironstar-nyp1 -.-> ironstar-nyp
    ironstar-nyp10 -.-> ironstar-nyp
    ironstar-nyp10 ==> ironstar-nyp9
    ironstar-nyp11 -.-> ironstar-nyp
    ironstar-nyp11 ==> ironstar-nyp10
    ironstar-nyp12 ==> ironstar-2nt2
    ironstar-nyp12 -.-> ironstar-3gd
    ironstar-nyp12 ==> ironstar-961
    ironstar-nyp13 -.-> ironstar-nyp
    ironstar-nyp14 -.-> ironstar-nyp
    ironstar-nyp15 -.-> ironstar-nyp
    ironstar-nyp15 ==> ironstar-nyp12
    ironstar-nyp15 ==> ironstar-nyp27
    ironstar-nyp16 -.-> ironstar-nyp
    ironstar-nyp17 ==> ironstar-2nt2
    ironstar-nyp17 -.-> ironstar-nyp
    ironstar-nyp18 -.-> ironstar-nyp
    ironstar-nyp19 ==> ironstar-2nt2
    ironstar-nyp19 -.-> ironstar-nyp
    ironstar-nyp2 ==> ironstar-2nt11
    ironstar-nyp2 ==> ironstar-2nt2
    ironstar-nyp2 -.-> ironstar-nyp
    ironstar-nyp20 -.-> ironstar-nyp
    ironstar-nyp20 -.-> ironstar-r6215
    ironstar-nyp21 ==> ironstar-2nt8
    ironstar-nyp21 -.-> ironstar-nyp
    ironstar-nyp21 ==> ironstar-r6213
    ironstar-nyp22 -.-> ironstar-nyp
    ironstar-nyp23 -.-> ironstar-nyp
    ironstar-nyp24 -.-> ironstar-nyp
    ironstar-nyp25 -.-> ironstar-nyp
    ironstar-nyp26 -.-> ironstar-nyp
    ironstar-nyp27 ==> ironstar-2nt2
    ironstar-nyp27 -.-> ironstar-nyp
    ironstar-nyp27 ==> ironstar-nyp19
    ironstar-nyp27 ==> ironstar-nyp25
    ironstar-nyp27 ==> ironstar-nyp26
    ironstar-nyp28 -.-> ironstar-nyp
    ironstar-nyp29 -.-> ironstar-nyp
    ironstar-nyp3 -.-> ironstar-nyp
    ironstar-nyp3 ==> ironstar-nyp1
    ironstar-nyp3 ==> ironstar-nyp2
    ironstar-nyp30 -.-> ironstar-nyp
    ironstar-nyp31 -.-> ironstar-nyp
    ironstar-nyp32 -.-> ironstar-nyp
    ironstar-nyp33 -.-> ironstar-nyp
    ironstar-nyp4 -.-> ironstar-nyp
    ironstar-nyp4 ==> ironstar-nyp3
    ironstar-nyp5 ==> ironstar-2nt2
    ironstar-nyp5 -.-> ironstar-nyp
    ironstar-nyp5 ==> ironstar-nyp19
    ironstar-nyp6 ==> ironstar-2nt2
    ironstar-nyp6 -.-> ironstar-nyp
    ironstar-nyp7 -.-> ironstar-nyp
    ironstar-nyp7 ==> ironstar-nyp27
    ironstar-nyp7 ==> ironstar-nyp3
    ironstar-nyp7 ==> ironstar-nyp6
    ironstar-nyp8 -.-> ironstar-nyp
    ironstar-nyp8 ==> ironstar-nyp27
    ironstar-nyp9 ==> ironstar-2nt2
    ironstar-nyp9 -.-> ironstar-nyp
    ironstar-r62 ==> ironstar-ny3
    ironstar-r62 ==> ironstar-nyp
    ironstar-r621 -.-> ironstar-r62
    ironstar-r6210 -.-> ironstar-r62
    ironstar-r6210 ==> ironstar-r629
    ironstar-r6211 -.-> ironstar-r62
    ironstar-r6211 ==> ironstar-r625
    ironstar-r6211 ==> ironstar-r626
    ironstar-r6211 ==> ironstar-r627
    ironstar-r6212 -.-> ironstar-r62
    ironstar-r6212 ==> ironstar-r6211
    ironstar-r6213 -.-> ironstar-r62
    ironstar-r6213 ==> ironstar-r6212
    ironstar-r6214 -.-> ironstar-r62
    ironstar-r6214 ==> ironstar-r625
    ironstar-r6215 -.-> ironstar-r62
    ironstar-r6215 ==> ironstar-r6211
    ironstar-r622 -.-> ironstar-r62
    ironstar-r623 -.-> ironstar-r62
    ironstar-r623 ==> ironstar-r622
    ironstar-r624 ==> ironstar-nyp10
    ironstar-r624 ==> ironstar-nyp27
    ironstar-r624 ==> ironstar-nyp3
    ironstar-r624 ==> ironstar-nyp7
    ironstar-r624 -.-> ironstar-r62
    ironstar-r625 ==> ironstar-nyp8
    ironstar-r625 -.-> ironstar-r62
    ironstar-r625 ==> ironstar-r624
    ironstar-r626 ==> ironstar-2nt4
    ironstar-r626 -.-> ironstar-r62
    ironstar-r626 ==> ironstar-r624
    ironstar-r627 ==> ironstar-nyp7
    ironstar-r627 -.-> ironstar-r62
    ironstar-r627 ==> ironstar-r624
    ironstar-r628 -.-> ironstar-r62
    ironstar-r629 ==> ironstar-ny313
    ironstar-r629 -.-> ironstar-r62
    ironstar-r629 ==> ironstar-r628
    ironstar-ubj -.-> ironstar-0tk
    ironstar-ubj ==> ironstar-edx
    ironstar-v4y1 -.-> ironstar-v4y
    ironstar-v4y2 -.-> ironstar-v4y
    ironstar-v4y3 -.-> ironstar-v4y
    ironstar-ym1 -.-> ironstar-0tk
    ironstar-z4s -.-> ironstar-0tk
    ironstar-z4s ==> ironstar-edx
    ironstar-zuv ==> ironstar-e6k
    ironstar-zuv1 ==> ironstar-nyp3
    ironstar-zuv1 -.-> ironstar-zuv
    ironstar-zuv2 ==> ironstar-nyp7
    ironstar-zuv2 -.-> ironstar-zuv
    ironstar-zuv2 ==> ironstar-zuv1
    ironstar-zuv3 ==> ironstar-r6213
    ironstar-zuv3 -.-> ironstar-zuv
    ironstar-zuv3 ==> ironstar-zuv1
    ironstar-zuv3 ==> ironstar-zuv2
    ironstar-zuv4 -.-> ironstar-2nt2
    ironstar-zuv4 -.-> ironstar-zuv
```

---

## 📋 ironstar-2nt.13 Enforce async/sync boundary via module organization

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2025-12-26 15:53 |

### Description

Establish module organization that enforces effect boundaries.

Core principle from design-principles.md:
- Pure functions (aggregates, value objects) are sync-only
- I/O operations (database, network) are async-only
- Application layer orchestrates async calls around sync domain logic

Implementation:
- Document module naming conventions
- Add clippy configuration if possible
- Create code review checklist

Ref: docs/notes/architecture/core/design-principles.md § Effect boundaries

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.13 -s in_progress

# Add a comment
bd comment ironstar-2nt.13 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.13 -p 1

# View full details
bd show ironstar-2nt.13
```

</details>

---

## 📋 ironstar-2nt.12 Implement UUID-tracked error type for distributed correlation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2025-12-26 15:53 |

### Description

All errors must include id: Uuid field for distributed tracing correlation.

Pattern from error-types.md:
```rust
pub struct AppError {
    id: Uuid,
    kind: AppErrorKind,
    source: Option<Box<dyn std::error::Error>>,
    backtrace: Backtrace,
}
```

Marked as CRITICAL in ironstar-northstar-cqrs-comparison.md § Error Handling.

Ref: docs/notes/architecture/decisions/error-types.md

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.12 -s in_progress

# Add a comment
bd comment ironstar-2nt.12 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.12 -p 1

# View full details
bd show ironstar-2nt.12
```

</details>

---

## 📋 ironstar-2nt.11 Add version(&self) -> u64 to Aggregate trait

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2025-12-26 15:53 |

### Description

Implement version field on aggregates to track event count for optimistic concurrency control. 

Required for optimistic locking pattern documented in event-sourcing-core.md.

Pattern:
- version() returns count of events applied to aggregate
- Used in EventStore append to detect concurrent modifications
- UNIQUE constraint on (aggregate_type, aggregate_id, aggregate_sequence) catches conflicts

Ref: docs/notes/architecture/cqrs/event-sourcing-core.md § Optimistic Locking

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.11 -s in_progress

# Add a comment
bd comment ironstar-2nt.11 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.11 -p 1

# View full details
bd show ironstar-2nt.11
```

</details>

---

## 📋 ironstar-r62.9 Create base layout template with Datastar initialization

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Implement base_layout() function using hypertext::maud! that renders html > head > body with conditional hotreload div (data-init for dev mode), CSS link to bundle.[hash].css, and JS script for datastar.js. Establishes HTML structure for all pages.
Local refs: ~/projects/rust-workspace/hypertext, ~/projects/lakescope-workspace/datastar-go-nats-template-northstar

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.8`
- ⛔ **blocks**: `ironstar-ny3.13`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.9 -s in_progress

# Add a comment
bd comment ironstar-r62.9 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.9 -p 1

# View full details
bd show ironstar-r62.9
```

</details>

---

## 📋 ironstar-r62.8 Implement RenderableToDatastar conversion trait

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:53 |

### Description

Implement RenderableToDatastar conversion trait for hypertext integration.

hypertext renders to a Buffer type, not String. Use into_inner() for conversion:

```rust
pub trait RenderableToDatastar {
    fn to_patch_elements(&self) -> PatchElements;
}

impl<T: Renderable> RenderableToDatastar for T {
    fn to_patch_elements(&self) -> PatchElements {
        let html = self.render().into_inner();  // NOT .to_string()
        PatchElements::new(html)
    }
}
```

The Renderable trait from hypertext produces lazy thunks that evaluate to Buffer.

Ref: docs/notes/architecture/decisions/backend-core-decisions.md

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.8 -s in_progress

# Add a comment
bd comment ironstar-r62.8 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.8 -p 1

# View full details
bd show ironstar-r62.8
```

</details>

---

## 📋 ironstar-r62.7 Implement query GET handlers

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create GET handlers that call query handler (reads from projections), render hypertext template, and return as HTML or JSON. No event persistence, just read path. Handlers use State extractor to access AppState containing projections.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/hypertext

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.4`
- ⛔ **blocks**: `ironstar-nyp.7`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.7 -s in_progress

# Add a comment
bd comment ironstar-r62.7 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.7 -p 1

# View full details
bd show ironstar-r62.7
```

</details>

---

## 📋 ironstar-r62.2 Create devShell module with tools and environment

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Implement nix/modules/devshell.nix defining default devShell with inputsFrom rust devShell and pre-commit hooks, plus packages: just, cargo-watch, pnpm, nodejs, process-compose, sqlite3, nixd, bacon. Complete development environment.
Local refs: ~/projects/rust-workspace/rust-nix-template/nix/modules/devshell.nix, ~/projects/nix-workspace/typescript-nix-template/modules/dev-shell.nix

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.2 -s in_progress

# Add a comment
bd comment ironstar-r62.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.2 -p 1

# View full details
bd show ironstar-r62.2
```

</details>

---

## 📋 ironstar-r62.1 Add justfile with development and build tasks

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create justfile at repository root with recipes: dev, dev-bg, gen-types, build-frontend, build-backend, build (full), test, fmt, lint, check, ci. Centralizes task orchestration following Rust conventions.
Local refs: ~/projects/rust-workspace/rust-nix-template/, ~/projects/nix-workspace/typescript-nix-template/justfile

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.1 -s in_progress

# Add a comment
bd comment ironstar-r62.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.1 -p 1

# View full details
bd show ironstar-r62.1
```

</details>

---

## 🚀 ironstar-r62 Presentation layer

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-24 10:03 |

### Description

Presentation layer with hypertext templates and datastar SSE streaming. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/decisions/backend-core-decisions.md and cqrs/sse-connection-lifecycle.md

### Dependencies

- ⛔ **blocks**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-ny3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62 -s in_progress

# Add a comment
bd comment ironstar-r62 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62 -p 1

# View full details
bd show ironstar-r62
```

</details>

---

## 📋 ironstar-nyp.8 Implement SSE 15-second keep-alive comment stream

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:53 |

### Description

Implement SSE 15-second keep-alive comment stream.

PREREQUISITE: Ensure SSE handler subscribes to event bus BEFORE querying historical events.
This is the Subscribe-Before-Replay Invariant from event-replay-consistency.md - prevents race condition where events arrive between query and subscription.

Pattern:
1. Subscribe to Zenoh/broadcast channel (get receiver)
2. Query historical events since Last-Event-ID
3. Stream historical events
4. Stream live events from receiver
5. Send keep-alive comments every 15 seconds

Keep-alive format: `: keepalive\n\n`

Ref: docs/notes/architecture/cqrs/event-replay-consistency.md
Ref: docs/notes/architecture/cqrs/sse-connection-lifecycle.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.27`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.8 -s in_progress

# Add a comment
bd comment ironstar-nyp.8 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.8 -p 1

# View full details
bd show ironstar-nyp.8
```

</details>

---

## 📋 ironstar-nyp.6 Create Projection trait for read models

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:53 |

### Description

Create Projection trait for read models.

Method signatures:
- fn apply(&mut self, event: &Event)  -- SYNC: non-blocking state update
- async fn rebuild(&mut self) -> Result<(), ProjectionError>  -- ASYNC: loads from disk/network

The apply() method must be sync because it's called in hot path during event processing. No I/O allowed.

The rebuild() method is async because it queries the event store on startup or after reset.

Pattern from projection-patterns.md:
```rust
pub trait Projection {
    fn apply(&mut self, event: &DomainEvent);
    async fn rebuild(&mut self) -> Result<(), ProjectionError>;
    fn name(&self) -> &'static str;
}
```

Ref: docs/notes/architecture/cqrs/projection-patterns.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.6 -s in_progress

# Add a comment
bd comment ironstar-nyp.6 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.6 -p 1

# View full details
bd show ironstar-nyp.6
```

</details>

---

## 📋 ironstar-nyp.2 Create EventStore trait abstraction

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:53 |

### Description

Create EventStore trait abstraction for event persistence.

Trait must include these methods:
- append(events: Vec<Event>) -> Result<(), StoreError>
- load(aggregate_id: &str) -> Result<Vec<Event>, StoreError>
- query_since_sequence(seq: u64) -> Result<Vec<Event>, StoreError>
- earliest_sequence() -> Result<u64, StoreError>  -- For stale Last-Event-ID detection
- latest_sequence() -> Result<u64, StoreError>    -- For gap detection

The earliest/latest sequence methods are required for edge case handling per event-replay-consistency.md:
- Detect stale Last-Event-ID (client requesting seq < earliest)
- Detect sequence gaps in event stream

Ref: docs/notes/architecture/cqrs/event-sourcing-core.md
Ref: docs/notes/architecture/cqrs/event-replay-consistency.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`
- ⛔ **blocks**: `ironstar-2nt.11`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.2 -s in_progress

# Add a comment
bd comment ironstar-nyp.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.2 -p 1

# View full details
bd show ironstar-nyp.2
```

</details>

---

## 📋 ironstar-nyp.1 Create database migrations/ directory with schema.sql

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:53 |

### Description

Create database migrations/ directory with schema.sql for event store.

CRITICAL: Use hybrid dual-sequence schema per ironstar-northstar-cqrs-comparison.md:

```sql
CREATE TABLE events (
    global_sequence INTEGER PRIMARY KEY AUTOINCREMENT,  -- For SSE Last-Event-ID
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    aggregate_sequence INTEGER NOT NULL,  -- For optimistic locking
    event_type TEXT NOT NULL,
    payload JSON NOT NULL,
    event_version INTEGER NOT NULL DEFAULT 1,
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(aggregate_type, aggregate_id, aggregate_sequence)
);

CREATE INDEX idx_events_aggregate ON events(aggregate_type, aggregate_id);
CREATE INDEX idx_events_type ON events(event_type);
```

The dual-sequence approach provides:
- global_sequence: Simple Last-Event-ID for SSE reconnection
- aggregate_sequence: Per-aggregate version for optimistic locking

Ref: docs/notes/architecture/cqrs/event-sourcing-core.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.11`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.1 -s in_progress

# Add a comment
bd comment ironstar-nyp.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.1 -p 1

# View full details
bd show ironstar-nyp.1
```

</details>

---

## 🚀 ironstar-nyp Event sourcing infrastructure

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-24 00:54 |

### Description

Event sourcing infrastructure with SQLite event store, tokio::broadcast event bus (Zenoh migration path for distributed deployment), and CQRS patterns. See docs/notes/architecture/cqrs/ and infrastructure/.

### Dependencies

- ⛔ **blocks**: `ironstar-2nt`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp -s in_progress

# Add a comment
bd comment ironstar-nyp 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp -p 1

# View full details
bd show ironstar-nyp
```

</details>

---

## 📋 ironstar-ny3.13 Implement rust-embed conditional asset serving

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create dual-mode asset serving: dev mode serves from filesystem via tower-http::ServeDir with no-store cache headers; prod mode embeds static/dist/ via rust-embed with immutable cache headers. Include AssetManifest loader for hashed filename resolution.
Local refs: ~/projects/rust-workspace/rust-embed

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.12`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.13 -s in_progress

# Add a comment
bd comment ironstar-ny3.13 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.13 -p 1

# View full details
bd show ironstar-ny3.13
```

</details>

---

## 📋 ironstar-ny3.12 Implement manifest.json parser for hashed filename resolution

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create AssetManifest struct parsing static/dist/manifest.json:
#[derive(Deserialize)]
struct AssetManifest(HashMap<String, String>);
impl AssetManifest {
    fn resolve(&self, entry: &str) -> Option<&str> {
        self.0.get(entry).map(|s| s.as_str())
    }
}
Resolves logical entry names (bundle.js) to content-hashed filenames (bundle.a1b2c3.js) for cache-busting.

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.11`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.12 -s in_progress

# Add a comment
bd comment ironstar-ny3.12 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.12 -p 1

# View full details
bd show ironstar-ny3.12
```

</details>

---

## 📋 ironstar-ny3.11 Create static/dist/ output directory structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Initialize static/dist/ directory placeholder for Rolldown build outputs (bundle.[hash].css, bundle.[hash].js, manifest.json). Create static/datastar/ for runtime datastar.js. Aligns with single-binary asset embedding in production.

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.11 -s in_progress

# Add a comment
bd comment ironstar-ny3.11 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.11 -p 1

# View full details
bd show ironstar-ny3.11
```

</details>

---

## 📋 ironstar-ny3.10 Configure ts-rs export directory and justfile task

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Add [env] section to .cargo/config.toml setting TS_RS_EXPORT_DIR. Create gen-types task in justfile: TS_RS_EXPORT_DIR=web-components/types cargo test --lib. Centralizes type generation configuration.
Local refs: ~/projects/rust-workspace/ts-rs

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.9`
- ⛔ **blocks**: `ironstar-2nt.5`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.10 -s in_progress

# Add a comment
bd comment ironstar-ny3.10 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.10 -p 1

# View full details
bd show ironstar-ny3.10
```

</details>

---

## 📋 ironstar-ny3.9 Add ts-rs dependency to Cargo.toml

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Add ts-rs 11.1+ with features serde-compat and uuid-impl. Enables deriving TS traits on Rust types to generate TypeScript definitions. Ensures frontend and backend signal contracts stay synchronized.
Local refs: ~/projects/rust-workspace/ts-rs

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.9 -s in_progress

# Add a comment
bd comment ironstar-ny3.9 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.9 -p 1

# View full details
bd show ironstar-ny3.9
```

</details>

---

## 📋 ironstar-ny3.8 Create web-components/index.ts entry point

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create index.ts that imports main.css (processed by PostCSS plugin) and auto-registers vanilla web components by importing from components/ subdirectory. Export TypeScript types from web-components/types/ for frontend type safety.
Local refs: ~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/index.ts

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.7`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.8 -s in_progress

# Add a comment
bd comment ironstar-ny3.8 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.8 -p 1

# View full details
bd show ironstar-ny3.8
```

</details>

---

## 📋 ironstar-ny3.7 Create TypeScript configuration (tsconfig.json)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:53 |

### Description

Create tsconfig.json for web-components/ TypeScript.

Configuration per lit-component-bundling.md:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "experimentalDecorators": true,
    "useDefineForClassFields": false,
    "strict": true,
    "paths": {
      "@types/*": ["./types/*"]
    }
  },
  "include": ["**/*.ts"],
  "exclude": ["node_modules"]
}
```

The paths mapping enables importing ts-rs generated types:
```typescript
import type { TodoSignals } from '@types/todo';
```

Ref: docs/notes/architecture/frontend/lit-component-bundling.md
Ref: docs/notes/architecture/frontend/signal-contracts.md

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.7 -s in_progress

# Add a comment
bd comment ironstar-ny3.7 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.7 -p 1

# View full details
bd show ironstar-ny3.7
```

</details>

---

## 📋 ironstar-ny3.5 Configure CSS cascade layers for predictable specificity

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:53 |

### Description

Configure CSS cascade layers for predictable style ordering.

Use 6-layer system per css-architecture.md:

```css
@layer openprops, normalize, theme, components, utilities, app;
```

Layer precedence (later overrides earlier):
1. openprops - Design token definitions
2. normalize - CSS reset/normalize
3. theme - Custom theme overrides
4. components - Open Props UI component styles
5. utilities - Utility classes
6. app - Application-specific overrides

Import order in main.css must match layer order.

Ref: docs/notes/architecture/frontend/css-architecture.md

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.4`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.5 -s in_progress

# Add a comment
bd comment ironstar-ny3.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.5 -p 1

# View full details
bd show ironstar-ny3.5
```

</details>

---

## 📋 ironstar-ny3.4 Setup Open Props design tokens and theme layer

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create web-components/styles/main.css importing Open Props design tokens. Create web-components/styles/theme.css with CSS custom properties using light-dark() function for automatic dark mode. Establish CSS cascade layers: openprops, normalize, theme, components, utilities, app.
Local refs: ~/projects/lakescope-workspace/open-props, ~/projects/lakescope-workspace/open-props-ui

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.4 -s in_progress

# Add a comment
bd comment ironstar-ny3.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.4 -p 1

# View full details
bd show ironstar-ny3.4
```

</details>

---

## 📋 ironstar-ny3.3 Setup PostCSS configuration for modern CSS features

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create web-components/postcss.config.js with plugins: postcss-import, postcss-preset-env (stage 0 for OKLch/light-dark/custom-media), autoprefixer, cssnano. Enables Open Props and modern CSS features.
Local refs: ~/projects/lakescope-workspace/open-props/

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.3 -s in_progress

# Add a comment
bd comment ironstar-ny3.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.3 -p 1

# View full details
bd show ironstar-ny3.3
```

</details>

---

## 📋 ironstar-ny3.2 Configure Rolldown bundler with content-based hashing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:53 |

### Description

Create Rolldown bundler configuration for web-components/.

Configuration per frontend-build-pipeline.md:

```javascript
// rolldown.config.js
import postcss from 'rollup-plugin-postcss';

export default {
  input: 'web-components/index.ts',
  output: {
    dir: 'static/dist',
    format: 'esm',
    entryFileNames: '[name].[hash].js',
    assetFileNames: '[name].[hash][extname]',  // Content-hashed assets
    sourcemap: process.env.NODE_ENV !== 'production'
  },
  plugins: [
    postcss({
      extract: true,
      minimize: true,  // cssnano in production
      plugins: [
        require('postcss-import'),
        require('postcss-preset-env'),
        require('autoprefixer')
      ]
    })
  ],
  treeshake: true
}
```

Output manifest.json for server-side asset lookup.

Ref: docs/notes/architecture/frontend/frontend-build-pipeline.md

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.2 -s in_progress

# Add a comment
bd comment ironstar-ny3.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.2 -p 1

# View full details
bd show ironstar-ny3.2
```

</details>

---

## 📋 ironstar-2nt.7 Implement command validation pattern with Result types

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create validation functions returning Result<ValidatedCommand, ValidationError>:
impl Command {
    fn validate(self) -> Result<ValidatedCommand, ValidationError> {
        match self {
            Command::AddTodo { text } => {
                let text = TodoText::try_from(text)?;
                Ok(ValidatedCommand::AddTodo { text })
            }
            ...
        }
    }
}
Railway-oriented programming pattern ensures invalid commands never reach aggregate apply_command logic.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.4`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.7 -s in_progress

# Add a comment
bd comment ironstar-2nt.7 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.7 -p 1

# View full details
bd show ironstar-2nt.7
```

</details>

---

## 📋 ironstar-2nt.6 Enforce camelCase convention for Datastar signal fields

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Apply #[serde(rename_all = "camelCase")] to all signal struct definitions:
#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
struct AddTodoSignal {
    todo_text: String,  // serializes as todoText
    is_urgent: bool,    // serializes as isUrgent
}
Ensures Rust snake_case fields serialize to JavaScript-idiomatic camelCase for Datastar signals.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.5`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.6 -s in_progress

# Add a comment
bd comment ironstar-2nt.6 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.6 -p 1

# View full details
bd show ironstar-2nt.6
```

</details>

---

## 📋 ironstar-2nt.5 Create Datastar signal types with ts-rs derives

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Define frontend signal contract types using serde::Serialize + ts_rs::TS derives so TypeScript definitions auto-generate. These types specify the shape of signals flowing from browser to server, ensuring type safety across the HTTP boundary.
Local refs: ~/projects/rust-workspace/ts-rs

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.5 -s in_progress

# Add a comment
bd comment ironstar-2nt.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.5 -p 1

# View full details
bd show ironstar-2nt.5
```

</details>

---

## 📋 ironstar-2nt.4 Design aggregate root state machines

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Model aggregate state machines using Rust enums:
enum TodoAggregate {
    Active { id: Uuid, text: TodoText, created_at: DateTime<Utc> },
    Completed { id: Uuid, text: TodoText, created_at: DateTime<Utc>, completed_at: DateTime<Utc> },
    Deleted,
}
impl TodoAggregate {
    fn apply_command(&self, cmd: Command) -> Result<DomainEvent, DomainError> {
        match (self, cmd) {
            (TodoAggregate::Active { .. }, Command::MarkComplete) => Ok(DomainEvent::TodoCompleted { ... }),
            (TodoAggregate::Completed { .. }, Command::MarkComplete) => Err(DomainError::AlreadyCompleted),
            ...
        }
    }
}
Apply commands to aggregates via pure functions that validate state transitions and emit events as output.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.3`
- ⛔ **blocks**: `ironstar-2nt.11`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.4 -s in_progress

# Add a comment
bd comment ironstar-2nt.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.4 -p 1

# View full details
bd show ironstar-2nt.4
```

</details>

---

## 📋 ironstar-2nt.3 Implement value objects and smart constructors

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create validated value objects with smart constructor pattern:
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String")]
struct TodoText(String);
impl TryFrom<String> for TodoText {
    type Error = ValidationError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.is_empty() { return Err(ValidationError::Empty); }
        if s.len() > 500 { return Err(ValidationError::TooLong); }
        Ok(TodoText(s))
    }
}
Product types reject invalid values before they enter the system, preventing bug vectors at the type level.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.3 -s in_progress

# Add a comment
bd comment ironstar-2nt.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.3 -p 1

# View full details
bd show ironstar-2nt.3
```

</details>

---

## 📋 ironstar-2nt.2 Define algebraic domain types and aggregate structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-24 00:42 |

### Description

Implement sum types for DomainEvent, Command, and aggregate states as Rust enums with serde serialization.

Session context: crates/ironstar/src/domain/mod.rs exists and is ready for implementation.
Workspace dependencies already include: serde, uuid, chrono.

Example pattern:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum DomainEvent {
    TodoCreated { id: Uuid, text: String, created_at: DateTime<Utc> },
    TodoCompleted { id: Uuid, completed_at: DateTime<Utc> },
    TodoDeleted { id: Uuid, deleted_at: DateTime<Utc> },
}
```

Establishes the core algebraic vocabulary making invalid states unrepresentable.
Type-level guarantees for all domain logic.

Local refs: crates/ironstar/src/domain/

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.2 -s in_progress

# Add a comment
bd comment ironstar-2nt.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.2 -p 1

# View full details
bd show ironstar-2nt.2
```

</details>

---

## 🚀 ironstar-2nt Domain layer

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-22 00:34 |

### Description

Domain layer with algebraic types, aggregates, and pure functional patterns. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/core/design-principles.md and crate-architecture.md

### Dependencies

- ⛔ **blocks**: `ironstar-6lq.5`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt -s in_progress

# Add a comment
bd comment ironstar-2nt 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt -p 1

# View full details
bd show ironstar-2nt
```

</details>

---

## 📋 ironstar-f8b.5 Verify process-compose up works with all services

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Test that process-compose up successfully starts all services in correct order. Verify readiness probes work, dependencies are respected, and logs are properly separated. Test that services restart appropriately when files change.

### Dependencies

- 🔗 **parent-child**: `ironstar-f8b`
- ⛔ **blocks**: `ironstar-f8b.4`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-f8b.5 -s in_progress

# Add a comment
bd comment ironstar-f8b.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-f8b.5 -p 1

# View full details
bd show ironstar-f8b.5
```

</details>

---

## 📋 ironstar-f8b.4 Configure cargo-watch to curl hotreload trigger on success

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Update backend process command in process-compose.yaml:
  backend:
    command: cargo watch -x run -s 'if cargo check; then curl -X POST http://localhost:3000/hotreload/trigger; fi'
Triggers browser reload via SSE only on successful backend rebuild. Integrates with TASK_HOTRELOAD endpoint (Epic 7) for seamless DX.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-f8b`
- ⛔ **blocks**: `ironstar-f8b.3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-f8b.4 -s in_progress

# Add a comment
bd comment ironstar-f8b.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-f8b.4 -p 1

# View full details
bd show ironstar-f8b.4
```

</details>

---

## 📋 ironstar-f8b.3 Set up service orchestration (frontend bundler, cargo-watch)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Configure service startup order and dependencies in process-compose.yaml:
processes:
  db-init:
    command: sqlite3 dev.db < migrations/schema.sql
    availability: { exit_on_end: true }
  backend:
    command: cargo watch -x run
    depends_on:
      db-init: { condition: process_completed }
    readiness_probe: { http_get: { host: localhost, port: 3000, path: /health } }
  frontend:
    command: cd web-components && pnpm rolldown -w
    depends_on: { backend: { condition: process_healthy } }
  hotreload:
    command: ...
    depends_on: { backend: { condition: process_healthy } }
Ensures db-init completes before backend starts, typegen runs when Rust files change, frontend rebuilds on TypeScript changes, backend restarts on Rust changes, hotreload triggers browser refresh after successful backend build.

### Dependencies

- 🔗 **parent-child**: `ironstar-f8b`
- ⛔ **blocks**: `ironstar-f8b.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-f8b.3 -s in_progress

# Add a comment
bd comment ironstar-f8b.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-f8b.3 -p 1

# View full details
bd show ironstar-f8b.3
```

</details>

---

## 📋 ironstar-f8b.2 Configure process-compose.yaml for dev services

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create process-compose.yaml with processes: db-init (one-shot SQLite schema), frontend (Rolldown watch), typegen (ts-rs watch), backend (cargo watch), hotreload (browser SSE trigger). Define process dependencies, readiness probes, and log_location for each service.
Local refs: ~/projects/nix-workspace/process-compose

### Dependencies

- 🔗 **parent-child**: `ironstar-f8b`
- ⛔ **blocks**: `ironstar-f8b.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-f8b.2 -s in_progress

# Add a comment
bd comment ironstar-f8b.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-f8b.2 -p 1

# View full details
bd show ironstar-f8b.2
```

</details>

---

## 📋 ironstar-f8b.1 Integrate process-compose-flake patterns into devShell

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create nix/modules/process-compose.nix importing process-compose-flake.flakeModule:
{
  imports = [ inputs.process-compose-flake.flakeModule ];
  perSystem = { config, pkgs, ... }: {
    process-compose.dev = {
      settings.processes = { ... };
    };
  };
}
Define perSystem process-compose configurations. Expose as packages.dev runnable via nix run .#dev. Integrates declarative process orchestration into Nix workflow.
Local refs: ~/projects/nix-workspace/process-compose-flake

### Dependencies

- 🔗 **parent-child**: `ironstar-f8b`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-f8b.1 -s in_progress

# Add a comment
bd comment ironstar-f8b.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-f8b.1 -p 1

# View full details
bd show ironstar-f8b.1
```

</details>

---

## 🚀 ironstar-f8b Process compose integration

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Integrate process-compose for orchestrating development services including database initialization, frontend bundler watch mode, TypeScript type generation, backend cargo-watch, and browser hotreload. Uses process-compose-flake for Nix integration and declarative service configuration with dependency ordering and readiness probes.

### Dependencies

- ⛔ **blocks**: `ironstar-6lq`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-f8b -s in_progress

# Add a comment
bd comment ironstar-f8b 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-f8b -p 1

# View full details
bd show ironstar-f8b
```

</details>

---

## 📋 ironstar-6lq.7 Add Rust to CI matrix and extend inherited workflows

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 21:50 |

### Description

Wire Rust checks into ci.yaml matrix after 6lq.6 (flake checks) and 6lq.8 (workflow pattern) are complete.

IMPLEMENTATION OPTIONS (choose one):

Option A: Integrate via nix flake check (simplest)
- Add job to ci.yaml that runs: nix flake check --impure
- Runs ALL checks (Rust + TypeScript + Nix) in one command
- Cachix caching already configured in flake nixConfig
- No separate workflow file needed

Option B: Separate Rust job with matrix
- Add rust-checks job to ci.yaml
- Use list-crates-json for crate discovery (like list-packages-json for TS)
- Dispatch to crate-test.yaml via workflow_call
- More granular caching/reporting but more complex

AVAILABLE FLAKE OUTPUTS:
- checks.*.ironstar-clippy
- checks.*.rust-fmt  
- checks.*.rust-test
- packages.*.ironstar
- packages.*.ironstar-doc

PLATFORM MATRIX:
- x86_64-linux (CI primary)
- aarch64-darwin (dev machines)
- aarch64-linux (optional)

CACHING:
- Cachix binary cache configured in nixConfig
- just cache / cache-all targets for local pushes
- GitHub Actions uses cached-ci-job composite action

NOTE: GitHub workflow_call requires workflow to exist on default branch.
May need to push minimal workflow to main first, or test with workflow_dispatch.

Local refs: typescript-nix-template/.github/workflows/ci.yaml

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.6`
- ⛔ **blocks**: `ironstar-6lq.8`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-6lq.7 -s in_progress

# Add a comment
bd comment ironstar-6lq.7 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-6lq.7 -p 1

# View full details
bd show ironstar-6lq.7
```

</details>

---

## 🚀 ironstar-6lq Rust workspace integration

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Integrate Rust toolchain and workspace patterns into the Nix flake using rust-flake, crane for deterministic builds, and rust-overlay for toolchain management. Establishes Cargo workspace structure with resolver 2, workspace.dependencies for DRY, per-crate crane.args configuration following rustlings-workspace and rust-nix-template patterns. Includes CI integration with flake checks and GitHub Actions matrix builds inherited from template.

### Dependencies

- ⛔ **blocks**: `ironstar-cxe`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-6lq -s in_progress

# Add a comment
bd comment ironstar-6lq 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-6lq -p 1

# View full details
bd show ironstar-6lq
```

</details>

---

## 📋 ironstar-3gd.3 Implement CacheDependency struct for Zenoh-based cache invalidation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 18:29 |
| **Updated** | 2025-12-26 18:29 |

### Description

Implement CacheDependency struct that maps cache keys to Zenoh key expression patterns for precise cache invalidation. Enables declarative cache invalidation by allowing cache entries to declare dependencies on aggregate types or instances.

Core struct: cache_key (String) + depends_on (Vec<String> of Zenoh patterns).

Builder methods:
- depends_on_aggregate(type) -> pattern 'events/{type}/**'
- depends_on_instance(type, id) -> pattern 'events/{type}/{id}/*'

Integration: ZenohCacheInvalidator matches incoming events against depends_on patterns to invalidate corresponding cache entries.

Example: CacheDependency::new("daily_stats:2024-01-15").depends_on_aggregate("Todo")

Include matches_key_expression() helper for pattern matching during invalidation.

Ref: docs/notes/architecture/infrastructure/analytics-cache-patterns.md lines 155-255

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- 🔗 **depends-on**: `ironstar-nyp.27`
- 🔗 **depends-on**: `ironstar-nyp.25`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-3gd.3 -s in_progress

# Add a comment
bd comment ironstar-3gd.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-3gd.3 -p 1

# View full details
bd show ironstar-3gd.3
```

</details>

---

## 📋 ironstar-jqv.12 Implement session regeneration and user binding in OAuth callback

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2025-12-26 15:53 |

### Description

Security-critical session handling after OAuth authentication.

Pattern from oauth-authentication.md:
1. Validate CSRF state parameter
2. Exchange code for tokens
3. Fetch user identity
4. DISCARD tokens (don't store)
5. Regenerate session ID (prevent fixation)
6. Bind user_id to new session

Session regeneration prevents session fixation attacks.

Ref: docs/notes/architecture/decisions/oauth-authentication.md § Session binding after OAuth

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.12 -s in_progress

# Add a comment
bd comment ironstar-jqv.12 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.12 -p 1

# View full details
bd show ironstar-jqv.12
```

</details>

---

## 📋 ironstar-nyp.31 Implement health check endpoints (/health, /health/ready, /health/live)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2025-12-26 15:53 |

### Description

Create Kubernetes-compatible health check endpoints.

Endpoints:
- /health - overall status
- /health/ready - readiness probe (dependencies available)
- /health/live - liveness probe (process healthy)

Check components:
- SQLite connectivity
- Zenoh event bus status
- moka cache status

Return JSON with component statuses and overall health.

Ref: docs/notes/architecture/decisions/observability-decisions.md § Health check implementation

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.31 -s in_progress

# Add a comment
bd comment ironstar-nyp.31 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.31 -p 1

# View full details
bd show ironstar-nyp.31
```

</details>

---

## 📋 ironstar-nyp.30 Implement observability initialization with dev/prod splitting

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2025-12-26 15:53 |

### Description

Set up tracing subscriber with environment-based configuration.

Development mode:
- Pretty printing with colors
- RUST_LOG filtering
- Full backtraces

Production mode:
- JSON structured output
- Span context for correlation
- Error-level minimum by default

Ref: docs/notes/architecture/decisions/observability-decisions.md § Development vs production configuration

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.30 -s in_progress

# Add a comment
bd comment ironstar-nyp.30 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.30 -p 1

# View full details
bd show ironstar-nyp.30
```

</details>

---

## 📋 ironstar-nyp.29 Implement error propagation pattern through CQRS pipeline

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2025-12-26 15:53 |

### Description

Orchestrate error types from domain → application → infrastructure → presentation layers.

Ensure UUID tracking propagates through all layers. Pattern:
- Domain: AggregateError with validation failures
- Application: CommandError wrapping AggregateError  
- Infrastructure: map to InfrastructureError
- Presentation: convert to HTTP response with error_id

Ref: docs/notes/architecture/decisions/error-handling-decisions.md § Error propagation across CQRS layers

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.29 -s in_progress

# Add a comment
bd comment ironstar-nyp.29 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.29 -p 1

# View full details
bd show ironstar-nyp.29
```

</details>

---

## 📋 ironstar-nyp.27 Implement ZenohEventBus struct with publish/subscribe methods

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 15:25 |
| **Updated** | 2025-12-26 15:25 |

### Description

Wrap zenoh::Session in Arc. Implement subscribe(&self, pattern: &str) returning Subscriber. Implement publish(&self, key: &str, payload: Vec<u8>) returning Result. See docs/notes/architecture/infrastructure/zenoh-event-bus.md for implementation patterns. Local refs: ~/projects/rust-workspace/zenoh

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.19`
- ⛔ **blocks**: `ironstar-2nt.2`
- ⛔ **blocks**: `ironstar-nyp.26`
- ⛔ **blocks**: `ironstar-nyp.25`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.27 -s in_progress

# Add a comment
bd comment ironstar-nyp.27 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.27 -p 1

# View full details
bd show ironstar-nyp.27
```

</details>

---

## 📋 ironstar-nyp.26 Create Zenoh embedded router configuration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-26 15:25 |

### Description

Configure zenoh::Config for embedded mode with peer discovery disabled per distributed-event-bus-migration.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.26 -s in_progress

# Add a comment
bd comment ironstar-nyp.26 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.26 -p 1

# View full details
bd show ironstar-nyp.26
```

</details>

---

## 📋 ironstar-nyp.25 Define Zenoh key expression patterns for event routing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-26 15:25 |

### Description

Key expressions: ironstar/{aggregate_type}/{aggregate_id}/events for scoped pub/sub.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.25 -s in_progress

# Add a comment
bd comment ironstar-nyp.25 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.25 -p 1

# View full details
bd show ironstar-nyp.25
```

</details>

---

## 📋 ironstar-nyp.22 Implement InfrastructureError type with database/network variants

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-24 15:37 |

### Description

Create InfrastructureError enum per error-types.md with DatabaseError, NetworkError, SerializationError variants.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.22 -s in_progress

# Add a comment
bd comment ironstar-nyp.22 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.22 -p 1

# View full details
bd show ironstar-nyp.22
```

</details>

---

## 📋 ironstar-ny3.17 Implement light-dark() theming with prefers-color-scheme

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-24 15:37 |

### Description

Configure postcss-preset-env for light-dark() function. Browser requirement: Chrome 123+, Safari 17.4+.

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.17 -s in_progress

# Add a comment
bd comment ironstar-ny3.17 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.17 -p 1

# View full details
bd show ironstar-ny3.17
```

</details>

---

## 📋 ironstar-ny3.16 Configure OKLch color system with Open Props syntax

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-24 15:37 |

### Description

Set up PostCSS color-mix and color() functions for OKLch perceptual uniformity per css-architecture.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.16 -s in_progress

# Add a comment
bd comment ironstar-ny3.16 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.16 -p 1

# View full details
bd show ironstar-ny3.16
```

</details>

---

## 📋 ironstar-753.6 Implement chart SSE endpoint with signal-driven options

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-24 15:37 |

### Description

Create /api/charts/{chart_id}/feed endpoint streaming ECharts options via PatchSignals.

### Dependencies

- 🔗 **parent-child**: `ironstar-753`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-753.6 -s in_progress

# Add a comment
bd comment ironstar-753.6 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-753.6 -p 1

# View full details
bd show ironstar-753.6
```

</details>

---

## 📋 ironstar-nyp.21 Implement observability initialization module

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 10:02 |
| **Updated** | 2025-12-24 10:02 |

### Description

Create infrastructure/observability.rs with init_dev_logging(), init_prod_logging(log_dir), and init_observability_with_config(config) functions. Configure EnvFilter, rolling file appender for prod, JSON output. See docs/notes/architecture/decisions/observability-decisions.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-r62.13`
- ⛔ **blocks**: `ironstar-2nt.8`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.21 -s in_progress

# Add a comment
bd comment ironstar-nyp.21 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.21 -p 1

# View full details
bd show ironstar-nyp.21
```

</details>

---

## 📋 ironstar-2nt.10 Define ErrorCode enum for HTTP error mapping

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 10:02 |
| **Updated** | 2025-12-24 10:02 |

### Description

Implement ErrorCode enum with ValidationFailed, NotFound, Conflict, Unauthorized, etc. and http_status() method. Part of error type hierarchy. See docs/notes/architecture/decisions/error-types.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.8`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.10 -s in_progress

# Add a comment
bd comment ironstar-2nt.10 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.10 -p 1

# View full details
bd show ironstar-2nt.10
```

</details>

---

## 📋 ironstar-2nt.9 Define ChartSignals and ChartSelection types with ts-rs

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:54 |
| **Updated** | 2025-12-24 00:54 |

### Description

Create signal types for ECharts integration per signal-contracts.md. ChartSignals contains chartOption (serde_json::Value), selected (Option ChartSelection), loading (bool). ChartSelection contains seriesName, dataIndex, name, value. Use serde rename camelCase for JSON compatibility.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.9 -s in_progress

# Add a comment
bd comment ironstar-2nt.9 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.9 -p 1

# View full details
bd show ironstar-2nt.9
```

</details>

---

## 📋 ironstar-nyp.19 Create EventBus trait abstraction

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:54 |
| **Updated** | 2025-12-26 15:53 |

### Description

Create EventBus trait abstraction for pub/sub.

PRIMARY: Zenoh embedded mode from day one per infrastructure-decisions.md.

Trait methods:
- publish(key: &str, event: &Event) -> Result<(), BusError>
- subscribe(pattern: &str) -> Result<Receiver<Event>, BusError>

DualEventBus pattern is OPTIONAL - only for legacy integration scenarios where tokio::broadcast coexistence is needed. New ironstar instantiations use Zenoh-only.

Key expression patterns: events/{aggregate_type}/{aggregate_id}/{sequence}

Ref: docs/notes/architecture/infrastructure/zenoh-event-bus.md
Ref: docs/notes/architecture/decisions/infrastructure-decisions.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.19 -s in_progress

# Add a comment
bd comment ironstar-nyp.19 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.19 -p 1

# View full details
bd show ironstar-nyp.19
```

</details>

---

## 📋 ironstar-jqv.7 Implement AuthContext axum extractor

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:44 |
| **Updated** | 2025-12-24 00:44 |

### Description

Implement AuthContext extractor from oauth-authentication.md. FromRequestParts impl that loads user from session. Provides Option<User> and session_id. Used by protected handlers.

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.7 -s in_progress

# Add a comment
bd comment ironstar-jqv.7 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.7 -p 1

# View full details
bd show ironstar-jqv.7
```

</details>

---

## 📋 ironstar-961 Implement DuckDB connection lifecycle management

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:43 |
| **Updated** | 2025-12-26 15:33 |

### Description

Configure async-duckdb::PoolBuilder with database path and connection count (recommend 4 connections). Initialize Pool at application startup in main.rs, store in AppState. Pool manages connection lifecycle automatically - connections are checked out via .conn() and returned when closure completes. Ensure Cargo.toml uses async-duckdb with bundled feature to avoid system DuckDB version mismatches. Do NOT use spawn_blocking - async-duckdb provides native async API.

See docs/notes/architecture/cqrs/projection-patterns.md for Pool initialization patterns.

Local refs: ~/projects/rust-workspace/async-duckdb

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-961 -s in_progress

# Add a comment
bd comment ironstar-961 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-961 -p 1

# View full details
bd show ironstar-961
```

</details>

---

## 📋 ironstar-9b1 Implement httpfs extension configuration for DuckDB

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:43 |
| **Updated** | 2025-12-24 00:43 |

### Description

Configure DuckDB with httpfs extension enabled. Add HuggingFace (hf://) and S3 (s3://) protocol support. Create configuration patterns for HuggingFace authentication tokens. Reference: ~/projects/rust-workspace/rust-duckdb-huggingface-ducklake-query

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-9b1 -s in_progress

# Add a comment
bd comment ironstar-9b1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-9b1 -p 1

# View full details
bd show ironstar-9b1
```

</details>

---

## 🚀 ironstar-3gd Scientific Data Integration

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:42 |
| **Updated** | 2025-12-24 00:42 |

### Description

Implement ironstar's READ-ONLY DuckDB analytics layer for querying external scientific datasets. DuckDB is a dedicated OLAP query interface—completely separate from the SQLite event store—enabling efficient analysis of large scientific data without impacting event sourcing durability. Covers DuckDB integration for analytics queries, remote data source support via httpfs extension (HuggingFace datasets, S3-compatible storage, DuckLake), results caching with moka + rkyv for visualization backends, and SSE/datastar integration for dashboard updates. See docs/notes/architecture/infrastructure/analytics-cache-architecture.md and analytics-cache-patterns.md.

### Dependencies

- ⛔ **blocks**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-3gd -s in_progress

# Add a comment
bd comment ironstar-3gd 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-3gd -p 1

# View full details
bd show ironstar-3gd
```

</details>

---

## 📋 ironstar-nyp.15 Implement moka analytics cache with rkyv serialization

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-23 23:23 |
| **Updated** | 2025-12-26 15:53 |

### Description

Implement moka in-memory cache for analytics query results.

moka is the v1 canonical choice per analytics-cache-architecture.md - critical for CQRS projection performance, not optional.

Configuration:
- Cache::builder() with TTL (5 minutes default)
- rkyv serialization for zero-copy reads
- Zenoh-based invalidation (see ironstar-3gd.2)

Ref: docs/notes/architecture/infrastructure/analytics-cache-architecture.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.12`
- ⛔ **blocks**: `ironstar-nyp.27`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.15 -s in_progress

# Add a comment
bd comment ironstar-nyp.15 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.15 -p 1

# View full details
bd show ironstar-nyp.15
```

</details>

---

## 📋 ironstar-753.5 Implement ds-echarts build and test integration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2025-12-23 23:22 |

### Description

Build pipeline integration, testing strategies. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/frontend/ds-echarts-build-test.md

### Dependencies

- 🔗 **parent-child**: `ironstar-753`
- ⛔ **blocks**: `ironstar-753.4`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-753.5 -s in_progress

# Add a comment
bd comment ironstar-753.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-753.5 -p 1

# View full details
bd show ironstar-753.5
```

</details>

---

## 📋 ironstar-753.4 Implement ds-echarts backend support

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2025-12-23 23:22 |

### Description

Server-side data preparation, SSE streaming for ECharts. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/frontend/ds-echarts-backend.md

### Dependencies

- 🔗 **parent-child**: `ironstar-753`
- ⛔ **blocks**: `ironstar-nyp.8`
- ⛔ **blocks**: `ironstar-nyp.12`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-753.4 -s in_progress

# Add a comment
bd comment ironstar-753.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-753.4 -p 1

# View full details
bd show ironstar-753.4
```

</details>

---

## 🚀 ironstar-jqv Authentication and authorization

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:24 |
| **Updated** | 2025-12-22 00:24 |

### Description

OAuth-only authentication architecture with GitHub as primary provider, Google OIDC planned. Covers session security, CSRF protection, and RBAC patterns. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/decisions/oauth-authentication.md

### Dependencies

- 🔗 **depends-on**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv -s in_progress

# Add a comment
bd comment ironstar-jqv 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv -p 1

# View full details
bd show ironstar-jqv
```

</details>

---

## 🚀 ironstar-0tk Omicslake presentation slide deck

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-19 01:15 |
| **Updated** | 2025-12-19 01:16 |

### Description

Perfect the ~45 minute Omicslake presentation tracing HDF5/AnnData → DuckLake → ironstar/Datastar stack. Located in docs/slides/ironstar-overview/. Target: compelling technical narrative for genomics/data engineering audience.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-0tk -s in_progress

# Add a comment
bd comment ironstar-0tk 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-0tk -p 1

# View full details
bd show ironstar-0tk
```

</details>

---

## 📋 ironstar-c7z Implement DuckDB remote data source integration (DuckLake/HF pattern)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 17:42 |
| **Updated** | 2025-12-23 23:23 |

### Description

Extend DuckDB analytics service to query remote data sources via DuckDB's httpfs extension:
- hf:// protocol for HuggingFace-hosted datasets (parquet files)
- s3:// protocol for S3-compatible storage (R2, MinIO)
- DuckLake catalog integration for versioned data lakes

CORE DEMO: This is the primary data source for demonstrating ironstar's visualization capabilities.

Example: SELECT * FROM 'hf://datasets/org/dataset/data.parquet';

Local refs:
- ~/projects/rust-workspace/rust-duckdb-huggingface-ducklake-query
- ~/projects/omicslake-workspace/marhar-frozen
- ~/projects/omicslake-workspace/marhar-duckdb-tools

### Dependencies

- ⛔ **blocks**: `ironstar-nyp.12`
- 🔗 **parent-child**: `ironstar-3gd`
- ⛔ **blocks**: `ironstar-9b1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-c7z -s in_progress

# Add a comment
bd comment ironstar-c7z 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-c7z -p 1

# View full details
bd show ironstar-c7z
```

</details>

---

## 📋 ironstar-amw Configure SQLite production PRAGMA settings (WAL, synchronous, cache)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 17:31 |
| **Updated** | 2025-12-18 17:31 |

### Description

Optimize SQLite for event sourcing workload after pool creation:
sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
sqlx::query("PRAGMA synchronous=FULL").execute(&pool).await?;
sqlx::query("PRAGMA cache_size=-64000").execute(&pool).await?; // 64MB
sqlx::query("PRAGMA temp_store=MEMORY").execute(&pool).await?;
WAL mode enables concurrent reads during writes. synchronous=FULL ensures durability.
Local refs: ~/projects/rust-workspace/sqlx

### Dependencies

- ⛔ **blocks**: `ironstar-nyp.3`
- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-amw -s in_progress

# Add a comment
bd comment ironstar-amw 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-amw -p 1

# View full details
bd show ironstar-amw
```

</details>

---

## 📋 ironstar-b9h Configure tower-http Brotli compression for SSE responses

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 17:31 |
| **Updated** | 2025-12-18 17:31 |

### Description

Add CompressionLayer from tower-http to SSE routes:
use tower_http::compression::CompressionLayer;
let app = Router::new()
    .route("/feed", get(sse_feed))
    .layer(CompressionLayer::new());
Datastar docs claim 200:1 compression ratios for HTML over SSE with Brotli.
Local refs: ~/projects/rust-workspace/axum, ~/projects/lakescope-workspace/datastar-doc

### Dependencies

- ⛔ **blocks**: `ironstar-r62.5`
- 🔗 **parent-child**: `ironstar-r62`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-b9h -s in_progress

# Add a comment
bd comment ironstar-b9h 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-b9h -p 1

# View full details
bd show ironstar-b9h
```

</details>

---

## 📋 ironstar-09r Implement ds-echarts Lit web component wrapper

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 17:11 |
| **Updated** | 2025-12-26 15:53 |

### Description

Create ds-echarts Lit component wrapper for Apache ECharts.

CRITICAL: Component must include data-ignore-morph attribute to prevent Datastar from morphing ECharts-managed DOM.

Pattern from ds-echarts-integration-guide.md:
- Light DOM via createRenderRoot() { return this }
- ResizeObserver with debounce for responsive charts
- MediaQueryList listener for dark mode
- setOption() for incremental updates
- Custom events for chart interactions

Template:
```html
<ds-echarts data-ignore-morph data-signals-chart-data="...">
</ds-echarts>
```

The data-ignore-morph prevents Datastar from interfering with ECharts' internal DOM management.

Ref: docs/notes/architecture/frontend/ds-echarts-integration-guide.md
Ref: docs/notes/architecture/frontend/integration-patterns.md

### Dependencies

- ⛔ **blocks**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-r62.5`
- ⛔ **blocks**: `ironstar-2nt.5`
- ⛔ **blocks**: `ironstar-r62.10`
- ⛔ **blocks**: `ironstar-2nt.9`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-09r -s in_progress

# Add a comment
bd comment ironstar-09r 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-09r -p 1

# View full details
bd show ironstar-09r
```

</details>

---

## 📋 ironstar-e6k.8 Implement todo example route mounting

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create routes() function that mounts GET /todos, POST /add-todo, POST /mark-todo, POST /delete-todo, and GET /todos-feed endpoints. Wire state with TodoStore, EventStore, Projections, and event_bus. Mount under /api prefix in main Router.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.6`
- ⛔ **blocks**: `ironstar-e6k.3`
- ⛔ **blocks**: `ironstar-e6k.4`
- ⛔ **blocks**: `ironstar-e6k.5`
- ⛔ **blocks**: `ironstar-e6k.7`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-e6k.8 -s in_progress

# Add a comment
bd comment ironstar-e6k.8 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-e6k.8 -p 1

# View full details
bd show ironstar-e6k.8
```

</details>

---

## 📋 ironstar-e6k.7 Implement todo_list_template rendering function

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create hypertext function fn todo_list_template(todos: &[TodoItem]) -> impl Renderable that renders ul#todo-list with li items, checkboxes with data-on:change, delete buttons with data-on:click, and add-todo form with input data-bind. Demonstrates complete Datastar integration for todo app.
Local refs: ~/projects/rust-workspace/hypertext, ~/projects/lakescope-workspace/datastar-go-nats-template-northstar

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-r62.10`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-e6k.7 -s in_progress

# Add a comment
bd comment ironstar-e6k.7 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-e6k.7 -p 1

# View full details
bd show ironstar-e6k.7
```

</details>

---

## 📋 ironstar-e6k.6 Implement GET /todos SSE feed endpoint

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create async handler returning Sse<impl Stream> that on initial connection sends TodoListProjection current state as PatchElements(todo_list_template(todos)), then streams incremental updates from broadcast channel. Implements Tao of Datastar principle 1 (backend is source of truth) with fat morph initial state.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.2`
- ⛔ **blocks**: `ironstar-r62.5`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-e6k.6 -s in_progress

# Add a comment
bd comment ironstar-e6k.6 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-e6k.6 -p 1

# View full details
bd show ironstar-e6k.6
```

</details>

---

## 📋 ironstar-e6k.5 Implement delete_todo handler (POST /delete-todo)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create async handler accepting ReadSignals<{id: Uuid}> that emits TodoDeleted event, appends to event store, broadcasts, returns 202. SSE morphs todo-list to remove deleted item or replaces entire list.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-e6k.5 -s in_progress

# Add a comment
bd comment ironstar-e6k.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-e6k.5 -p 1

# View full details
bd show ironstar-e6k.5
```

</details>

---

## 📋 ironstar-e6k.4 Implement mark_todo handler (POST /mark-todo)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create async handler accepting ReadSignals<{id: Uuid}> that emits TodoCompleted event, appends to event store, broadcasts, returns 202. SSE updates todo item to show completed state via hypertext morphing.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-e6k.4 -s in_progress

# Add a comment
bd comment ironstar-e6k.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-e6k.4 -p 1

# View full details
bd show ironstar-e6k.4
```

</details>

---

## 📋 ironstar-e6k.3 Implement add_todo handler (POST /add-todo)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create async handler accepting ReadSignals<AddTodoCommand> with text field. Validates non-empty, emits TodoCreated event, appends to event store, broadcasts to subscribers, returns 202. Frontend removes loading indicator via SSE update. Demonstrates write path with immediate response.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.2`
- ⛔ **blocks**: `ironstar-r62.6`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-e6k.3 -s in_progress

# Add a comment
bd comment ironstar-e6k.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-e6k.3 -p 1

# View full details
bd show ironstar-e6k.3
```

</details>

---

## 📋 ironstar-e6k.2 Implement TodoListProjection with in-memory rebuild

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create struct TodoListProjection(Vec<TodoItem>) implementing Projection trait. rebuild() method replays all TodoCreated/TodoCompleted/TodoDeleted events to reconstruct current state. apply() method handles incremental event updates. Demonstrates projection pattern.
Local refs: ~/projects/rust-workspace/datastar-rust-lince

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.1`
- ⛔ **blocks**: `ironstar-nyp.7`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-e6k.2 -s in_progress

# Add a comment
bd comment ironstar-e6k.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-e6k.2 -p 1

# View full details
bd show ironstar-e6k.2
```

</details>

---

## 📋 ironstar-e6k.1 Define Todo domain model (aggregate, events, commands)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create src/domain/todo.rs with:
struct TodoAggregate { id: Uuid, text: TodoText, completed: bool, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }
enum TodoEvent { TodoCreated { id: Uuid, text: String, created_at: DateTime<Utc> }, TodoCompleted { id: Uuid, completed_at: DateTime<Utc> }, TodoDeleted { id: Uuid, deleted_at: DateTime<Utc> } }
enum TodoCommand { AddTodo { text: String }, MarkComplete { id: Uuid }, DeleteTodo { id: Uuid } }
Demonstrates algebraic modeling with sum types (events) and product types (aggregates).
Local refs: ~/projects/lakescope-workspace/datastar-go-nats-template-northstar

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-2nt.2`
- ⛔ **blocks**: `ironstar-2nt.4`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-e6k.1 -s in_progress

# Add a comment
bd comment ironstar-e6k.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-e6k.1 -p 1

# View full details
bd show ironstar-e6k.1
```

</details>

---

## 📋 ironstar-r62.13 Wire all components together in main.rs

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create main.rs that initializes EventStore, SessionStore, Projections, EventBus, composes Router, and starts axum server on configured port. Handle graceful shutdown. Orchestration layer tying all services together.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/tokio

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.12`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.13 -s in_progress

# Add a comment
bd comment ironstar-r62.13 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.13 -p 1

# View full details
bd show ironstar-r62.13
```

</details>

---

## 📋 ironstar-r62.10 Implement component-level hypertext templates

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create reusable component functions (e.g., button, form_field, loading_spinner) returning impl Renderable. Components accept data and emit proper Datastar attributes (data-on:, data-show, data-bind). These compose into page templates.
Local refs: ~/projects/rust-workspace/hypertext

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.9`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.10 -s in_progress

# Add a comment
bd comment ironstar-r62.10 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.10 -p 1

# View full details
bd show ironstar-r62.10
```

</details>

---

## 📋 ironstar-r62.3 Configure pre-commit hooks for code quality

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create nix/modules/pre-commit.nix with git hooks for rustfmt, clippy, prettier (frontend), and linters. Set up .pre-commit-config.yaml to integrate with devShell via git-hooks.nix flake module.
Local refs: ~/projects/rust-workspace/rust-nix-template/nix/modules/pre-commit.nix

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.3 -s in_progress

# Add a comment
bd comment ironstar-r62.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.3 -p 1

# View full details
bd show ironstar-r62.3
```

</details>

---

## 📋 ironstar-nyp.12 Implement DuckDB analytics service

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:33 |

### Description

Implement DuckDBService wrapper around async-duckdb::Pool. Use PoolBuilder to configure connection count and database path. Wrap Pool in Arc for sharing across handlers. Expose query methods that use pool.conn(|conn| { ... }).await pattern for non-blocking analytics queries. Do NOT use spawn_blocking or block_in_place - async-duckdb handles threading internally via dedicated background threads.

See docs/notes/architecture/cqrs/projection-patterns.md (DuckDB analytics integration section) and docs/notes/architecture/infrastructure/analytics-cache-patterns.md for implementation patterns.

Local refs: ~/projects/rust-workspace/async-duckdb

### Dependencies

- ⛔ **blocks**: `ironstar-2nt.2`
- ⛔ **blocks**: `ironstar-961`
- 🔗 **parent-child**: `ironstar-3gd`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.12 -s in_progress

# Add a comment
bd comment ironstar-nyp.12 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.12 -p 1

# View full details
bd show ironstar-nyp.12
```

</details>

---

## 📋 ironstar-nyp.11 Create Session axum extractor

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Implement FromRequestParts for Session type extracting session_id from CookieJar. Load or initialize SessionData from SessionStore. Return Session struct with id and data fields for use in handlers.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.10`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.11 -s in_progress

# Add a comment
bd comment ironstar-nyp.11 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.11 -p 1

# View full details
bd show ironstar-nyp.11
```

</details>

---

## 📋 ironstar-nyp.10 Add session TTL cleanup background task

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Spawn tokio background task running every hour to delete expired sessions:
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        interval.tick().await;
        session_store.delete_expired(SystemTime::now() - Duration::from_secs(86400 * 30)).await;
    }
});
Prevents unbounded session store growth in long-running servers.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.9`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.10 -s in_progress

# Add a comment
bd comment ironstar-nyp.10 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.10 -p 1

# View full details
bd show ironstar-nyp.10
```

</details>

---

## 📋 ironstar-nyp.9 Implement SQLite session store with SessionStore trait

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-24 10:03 |

### Description

Implement SessionStore trait with SQLite backend. Schema: id TEXT PRIMARY KEY, user_id TEXT, created_at, last_seen_at, expires_at TIMESTAMP, data JSON. Methods: create, get, update_data, touch, delete, cleanup_expired. Use 24-byte cryptographic session IDs. See docs/notes/architecture/infrastructure/session-management.md and session-implementation.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.9 -s in_progress

# Add a comment
bd comment ironstar-nyp.9 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.9 -p 1

# View full details
bd show ironstar-nyp.9
```

</details>

---

## 📋 ironstar-nyp.7 Implement ProjectionManager with in-memory state

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-24 00:42 |

### Description

Create generic ProjectionManager<P: Projection> wrapping Arc<RwLock<P::State>>:
impl<P: Projection> ProjectionManager<P> {
    async fn init(event_store: Arc<dyn EventStore>, bus: broadcast::Sender<StoredEvent>) -> Self {
        let events = event_store.query_all().await.unwrap();
        let state = projection.rebuild(events).await;
        spawn background task subscribing to bus for incremental updates
    }
    async fn query(&self) -> P::State { self.state.read().await.clone() }
}
Replays all events from event store to build initial state, then applies incremental updates from broadcast channel.
Local refs: ~/projects/rust-workspace/tokio

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.6`
- ⛔ **blocks**: `ironstar-nyp.3`
- ⛔ **blocks**: `ironstar-nyp.27`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.7 -s in_progress

# Add a comment
bd comment ironstar-nyp.7 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.7 -p 1

# View full details
bd show ironstar-nyp.7
```

</details>

---

## 📋 ironstar-nyp.4 Implement SQLite connection pooling and configuration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Configure SqlitePool with PRAGMA settings for event sourcing: journal_mode=WAL, synchronous=FULL, cache_size=-64000 (64MB), temp_store=MEMORY. Optimizes for durability and read throughput on the event store workload.
Local refs: ~/projects/rust-workspace/sqlx

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.4 -s in_progress

# Add a comment
bd comment ironstar-nyp.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.4 -p 1

# View full details
bd show ironstar-nyp.4
```

</details>

---

## 📋 ironstar-nyp.3 Implement SQLite event store with sqlx

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-24 00:42 |

### Description

Create SqliteEventStore struct implementing EventStore trait:
async fn append(&self, event: &DomainEvent) -> Result<u64>;
async fn query_all(&self) -> Result<Vec<StoredEvent>>;
async fn query_since_sequence(&self, seq: u64) -> Result<Vec<StoredEvent>>;
async fn query_aggregate(&self, agg_type: &str, agg_id: Uuid) -> Result<Vec<StoredEvent>>;
Use sqlx compile-time query validation with query!() macro. Create events table with sequence, aggregate_type, aggregate_id, event_type, payload JSON columns. Append-only log foundation for CQRS.
Local refs: ~/projects/rust-workspace/sqlx

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.2`
- ⛔ **blocks**: `ironstar-nyp.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.3 -s in_progress

# Add a comment
bd comment ironstar-nyp.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.3 -p 1

# View full details
bd show ironstar-nyp.3
```

</details>

---

## 📋 ironstar-ny3.14 Create web-components/components/ directory for vanilla web components

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Set up web-components/components/ directory structure for vanilla web components. Create index.ts that exports/registers all components:
class SortableList extends HTMLElement {
  connectedCallback() {
    Sortable.create(this, { onEnd: (evt) => this.dispatchEvent(new CustomEvent('reorder', { detail: evt })) });
  }
  disconnectedCallback() { /* cleanup */ }
}
customElements.define('sortable-list', SortableList);
Contains thin wrapper web components for third-party libraries following the data-ignore-morph pattern with Datastar integration.
Local refs: ~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/components/

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.8`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.14 -s in_progress

# Add a comment
bd comment ironstar-ny3.14 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.14 -p 1

# View full details
bd show ironstar-ny3.14
```

</details>

---

## 📋 ironstar-ny3.6 Copy Open Props UI component CSS files

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Copy component CSS from ~/projects/lakescope-workspace/open-props-ui into web-components/styles/components/ (button.css, card.css, dialog.css, input.css, field.css, etc). Customize for ironstar theming. This follows the copy-paste ownership model where project owns and customizes component CSS.
Local refs: ~/projects/lakescope-workspace/open-props-ui

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.5`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.6 -s in_progress

# Add a comment
bd comment ironstar-ny3.6 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.6 -p 1

# View full details
bd show ironstar-ny3.6
```

</details>

---

## 🚀 ironstar-ny3 Frontend build pipeline

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-22 00:34 |

### Description

Frontend build pipeline with Rolldown, PostCSS, and Open Props. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/frontend/frontend-build-pipeline.md and decisions/frontend-stack-decisions.md

### Dependencies

- ⛔ **blocks**: `ironstar-6lq.7`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3 -s in_progress

# Add a comment
bd comment ironstar-ny3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3 -p 1

# View full details
bd show ironstar-ny3
```

</details>

---

## 📋 ironstar-2nt.8 Define application error types

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-24 10:03 |

### Description

Define layered error type hierarchy: ValidationError (field-level), DomainError (business rules), AggregateError (command handling), InfrastructureError (storage/bus), AppError (HTTP boundary). See docs/notes/architecture/decisions/error-types.md for complete type definitions and From implementations.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.8 -s in_progress

# Add a comment
bd comment ironstar-2nt.8 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.8 -p 1

# View full details
bd show ironstar-2nt.8
```

</details>

---

## 📋 ironstar-zuv.4 Implement AggregateTestFramework with given/when/then DSL

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 18:30 |
| **Updated** | 2025-12-26 18:30 |

### Description

Implement pure aggregate testing framework with declarative given/when/then DSL pattern adapted from cqrs-es TestFramework.

Core types:
- AggregateTestFramework<A: Aggregate>: Entry point with given() and given_no_previous_events()
- AggregateTestExecutor<A>: Executes commands via when()
- AggregateTestResult<A>: Validates via then_expect_events() or then_expect_error()

Benefits: Fast, deterministic testing without persistence or I/O. Zero external dependencies (no SQLite, no event bus, no async runtime). Pure functional aggregate verification.

Example:
AggregateTestFramework::<Todo>::given_no_previous_events()
    .when(TodoCommand::Create { text: "Buy milk".into() })
    .then_expect_events(vec![TodoEvent::Created { text: "Buy milk".into() }]);

Pattern adopted from cqrs-es TestFramework.
Ref: docs/notes/architecture/cqrs/command-write-patterns.md lines 254-346

### Dependencies

- 🔗 **parent-child**: `ironstar-zuv`
- 🔗 **depends-on**: `ironstar-2nt.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-zuv.4 -s in_progress

# Add a comment
bd comment ironstar-zuv.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-zuv.4 -p 1

# View full details
bd show ironstar-zuv.4
```

</details>

---

## 📋 ironstar-nyp.32 Instrument Zenoh event bus with Prometheus metrics

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2025-12-26 15:53 |

### Description

Add observability metrics for Zenoh event bus operations.

Metrics to implement:
- ZENOH_PUBLICATIONS_TOTAL (counter)
- ZENOH_SUBSCRIBER_COUNT (gauge)
- ZENOH_SAMPLE_LATENCY_SECONDS (histogram)

Use prometheus crate with METRICS_REGISTRY pattern.

Ref: docs/notes/architecture/infrastructure/zenoh-event-bus.md § Monitoring and metrics

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.32 -s in_progress

# Add a comment
bd comment ironstar-nyp.32 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.32 -p 1

# View full details
bd show ironstar-nyp.32
```

</details>

---

## 📋 ironstar-nyp.28 Implement per-session Zenoh subscriptions for SSE streams

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 15:25 |
| **Updated** | 2025-12-26 15:25 |

### Description

Subscribe to session-scoped key expressions in SSE handler. Key pattern: ironstar/sessions/{session_id}/**. Enables targeted event delivery per client connection. See docs/notes/architecture/infrastructure/session-management.md and zenoh-event-bus.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.28 -s in_progress

# Add a comment
bd comment ironstar-nyp.28 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.28 -p 1

# View full details
bd show ironstar-nyp.28
```

</details>

---

## 📋 ironstar-nyp.24 Add CQRS pipeline span context propagation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-24 15:37 |

### Description

Instrument command->event->projection flow with tracing spans including command_id, aggregate_id, user_id.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.24 -s in_progress

# Add a comment
bd comment ironstar-nyp.24 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.24 -p 1

# View full details
bd show ironstar-nyp.24
```

</details>

---

## 📋 ironstar-nyp.23 Configure dev vs prod logging subscribers

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-24 15:37 |

### Description

Implement conditional logging: pretty stdout for dev, JSON rolling files for prod via #[cfg(debug_assertions)].

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.23 -s in_progress

# Add a comment
bd comment ironstar-nyp.23 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.23 -p 1

# View full details
bd show ironstar-nyp.23
```

</details>

---

## 📋 ironstar-3gd.2 Implement event-driven cache invalidation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-26 15:25 |

### Description

Subscribe to Zenoh key expressions for aggregate-type events. Invalidate moka cache entries matching the aggregate type when events arrive. Pattern: ironstar/events/{aggregate_type}/**. See docs/notes/architecture/infrastructure/analytics-cache-architecture.md Pattern 4.

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- 🔗 **depends-on**: `ironstar-3gd.3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-3gd.2 -s in_progress

# Add a comment
bd comment ironstar-3gd.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-3gd.2 -p 1

# View full details
bd show ironstar-3gd.2
```

</details>

---

## 📋 ironstar-3gd.1 Implement cache-aside pattern for DuckDB analytics

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-24 15:37 |

### Description

moka get_or_compute wrapper with query hash key, TTL and idle eviction.

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-3gd.1 -s in_progress

# Add a comment
bd comment ironstar-3gd.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-3gd.1 -p 1

# View full details
bd show ironstar-3gd.1
```

</details>

---

## 📋 ironstar-jqv.11 Implement session rate limiting with sliding window

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2025-12-26 15:53 |

### Description

Rate limit session creation to prevent DoS attacks.

Use governor crate (NOT moka) per session-security.md:

```rust
use governor::{Quota, RateLimiter};

let quota = Quota::per_minute(10);
let limiter: RateLimiter<IpAddr, _, _> = RateLimiter::keyed(quota);

// In handler:
if limiter.check_key(&client_ip).is_err() {
    return Err(TooManyRequests);
}
```

Key by IP address for session creation endpoint.

Ref: docs/notes/architecture/infrastructure/session-security.md

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.11 -s in_progress

# Add a comment
bd comment ironstar-jqv.11 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.11 -p 1

# View full details
bd show ironstar-jqv.11
```

</details>

---

## 📋 ironstar-nyp.20 Implement Prometheus metrics endpoint and instrumentation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 10:02 |
| **Updated** | 2025-12-24 10:02 |

### Description

Add /metrics endpoint with prometheus crate. Instrument: events_appended_total (Counter), sse_connections (Gauge), projection_lag (Gauge), broadcast_lag_events (Histogram). See docs/notes/architecture/decisions/metrics-reference.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- 🔗 **discovered-from**: `ironstar-r62.15`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.20 -s in_progress

# Add a comment
bd comment ironstar-nyp.20 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.20 -p 1

# View full details
bd show ironstar-nyp.20
```

</details>

---

## 📋 ironstar-jqv.10 Implement OAuth CSRF state validation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 10:02 |
| **Updated** | 2025-12-24 10:02 |

### Description

Store CsrfToken in session.data during login redirect, validate params.state matches stored_state in callback. See oauth-authentication.md OAuth flow sections.

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.10 -s in_progress

# Add a comment
bd comment ironstar-jqv.10 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.10 -p 1

# View full details
bd show ironstar-jqv.10
```

</details>

---

## 📋 ironstar-jqv.9 Implement RequireAuth axum extractor

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 10:02 |
| **Updated** | 2025-12-24 10:02 |

### Description

Create RequireAuth extractor that wraps AuthContext and returns Unauthorized error if user is None. Enables protected_handler(RequireAuth(user): RequireAuth) pattern. See docs/notes/architecture/decisions/oauth-authentication.md section 'Authorization extractor'.

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`
- ⛔ **blocks**: `ironstar-jqv.7`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.9 -s in_progress

# Add a comment
bd comment ironstar-jqv.9 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.9 -p 1

# View full details
bd show ironstar-jqv.9
```

</details>

---

## 📋 ironstar-6lq.9 Add workspace lint configuration to Cargo.toml

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 10:02 |
| **Updated** | 2025-12-24 10:02 |

### Description

Add [workspace.lints.rust] and [workspace.lints.clippy] sections following Hyperswitch pattern. See docs/notes/architecture/core/crate-services-composition.md section 'Workspace lint configuration'.

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-6lq.9 -s in_progress

# Add a comment
bd comment ironstar-6lq.9 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-6lq.9 -p 1

# View full details
bd show ironstar-6lq.9
```

</details>

---

## 📋 ironstar-jqv.8 Implement session regeneration for fixation prevention

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:44 |
| **Updated** | 2025-12-24 00:44 |

### Description

Implement regenerate_session function from session-security.md. On privilege escalation (OAuth callback), create new session ID, copy data, delete old session. Prevents session fixation attacks.

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.8 -s in_progress

# Add a comment
bd comment ironstar-jqv.8 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.8 -p 1

# View full details
bd show ironstar-jqv.8
```

</details>

---

## 📋 ironstar-jqv.6 Implement RBAC authorization patterns

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:44 |
| **Updated** | 2025-12-24 00:44 |

### Description

Implement Authorizer trait and RbacAuthorizer from oauth-authentication.md. Role enum (Admin/Editor/Viewer). has_role predicate checks. Separate from OAuth authentication flow.

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.6 -s in_progress

# Add a comment
bd comment ironstar-jqv.6 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.6 -p 1

# View full details
bd show ironstar-jqv.6
```

</details>

---

## 📋 ironstar-ny3.15 Configure Rolldown for Lit web component bundling

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:44 |
| **Updated** | 2025-12-26 15:53 |

### Description

Configure Rolldown for Lit web component bundling.

Use Option A from lit-component-bundling.md: Extend Rolldown with TypeScript decorator support.

This aligns with build-tooling-decisions.md philosophy: 'Rust aligns with the stack's language choice (no Go in the dependency tree).'

Configuration:
```javascript
// rolldown.config.js
export default {
  input: 'web-components/index.ts',
  output: {
    dir: 'static/dist',
    format: 'esm',
    entryFileNames: '[name].[hash].js',
    assetFileNames: '[name].[hash][extname]'
  },
  plugins: [
    typescript({ experimentalDecorators: true }),
    postcss({ minimize: true })
  ]
}
```

Do NOT use esbuild as primary bundler.

Ref: docs/notes/architecture/frontend/lit-component-bundling.md
Ref: docs/notes/architecture/decisions/build-tooling-decisions.md

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.15 -s in_progress

# Add a comment
bd comment ironstar-ny3.15 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.15 -p 1

# View full details
bd show ironstar-ny3.15
```

</details>

---

## 📋 ironstar-nyp.18 Implement SSE ConnectionTracker with atomic counter

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:44 |
| **Updated** | 2025-12-24 00:44 |

### Description

Implement ConnectionTracker from sse-connection-lifecycle.md. Atomic counter for active SSE connections. RAII ConnectionGuard for automatic cleanup on disconnect. Metrics endpoint exposure for capacity planning and debugging.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.18 -s in_progress

# Add a comment
bd comment ironstar-nyp.18 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.18 -p 1

# View full details
bd show ironstar-nyp.18
```

</details>

---

## 📋 ironstar-nyp.17 Implement EventUpcaster trait and UpcasterChain for schema evolution

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:44 |
| **Updated** | 2025-12-26 18:31 |

### Description

Implement Upcaster pattern from event-sourcing-core.md for backward-compatible event schema evolution. EventUpcaster trait with can_upcast(&self, event_type, event_version) -> bool and upcast(&self, payload: Value) -> Value methods. UpcasterChain registry for sequential upcaster application. Events store event_version in metadata; upcasters transform old schemas to current during load_events_with_upcasting. Lazy application during event loading preserves event store immutability (events are facts that cannot change). Categorical structure: versions as objects, upcasters as morphisms, composition as sequential application, identity as no-op for current version. Example: TodoCreatedV1ToV2 adds default priority field when v1 event lacks it.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.17 -s in_progress

# Add a comment
bd comment ironstar-nyp.17 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.17 -p 1

# View full details
bd show ironstar-nyp.17
```

</details>

---

## 📋 ironstar-89k Integrate analytics cache with dashboard SSE streams

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:43 |
| **Updated** | 2025-12-24 00:43 |

### Description

Create separate analytics_bus broadcast channel distinct from main event bus. Wire cache refresh to SSE patch updates for dashboards. Implement multi-dashboard concurrent cache access. Reference: analytics-cache-patterns.md SSE/Datastar integration section.

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- ⛔ **blocks**: `ironstar-nyp.12`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-89k -s in_progress

# Add a comment
bd comment ironstar-89k 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-89k -p 1

# View full details
bd show ironstar-89k
```

</details>

---

## 📋 ironstar-jqv.5 Create user_identities table for multi-provider support

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-23 23:23 |
| **Updated** | 2025-12-23 23:23 |

### Description

Create user_identities table linking providers to users. Enables account linking (one user, multiple OAuth providers). Schema per oauth-authentication.md: id, user_id (FK), provider, provider_user_id, provider_email, created_at. Unique constraint on (provider, provider_user_id).

DEFERRED: Future extension after core demo.

Local refs: docs/notes/architecture/decisions/oauth-authentication.md

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`
- ⛔ **blocks**: `ironstar-jqv.4`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.5 -s in_progress

# Add a comment
bd comment ironstar-jqv.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.5 -p 1

# View full details
bd show ironstar-jqv.5
```

</details>

---

## 📋 ironstar-jqv.4 Implement users table schema and UserService

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-23 23:23 |
| **Updated** | 2025-12-23 23:23 |

### Description

Create users table for storing authenticated user profiles. Implement UserService with upsert_from_oauth(), get_by_id() methods. Schema per oauth-authentication.md: id, email, display_name, avatar_url, created_at, updated_at columns.

DEFERRED: Not needed for core demo with anonymous sessions.

Local refs: docs/notes/architecture/decisions/oauth-authentication.md

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.4 -s in_progress

# Add a comment
bd comment ironstar-jqv.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.4 -p 1

# View full details
bd show ironstar-jqv.4
```

</details>

---

## 📋 ironstar-nyp.14 Implement metrics and observability reference

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2025-12-24 00:54 |

### Description

Implement /metrics endpoint for Prometheus scraping. Counters: ironstar_events_appended_total, ironstar_commands_processed_total. Gauges: ironstar_sse_connections_active, ironstar_projection_lag_events. Histogram for command latency. See observability-decisions.md and metrics-reference.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.14 -s in_progress

# Add a comment
bd comment ironstar-nyp.14 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.14 -p 1

# View full details
bd show ironstar-nyp.14
```

</details>

---

## 📋 ironstar-nyp.13 Document error handling decisions

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2025-12-22 00:28 |

### Description

Error type hierarchy, Result propagation patterns. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/decisions/error-handling-decisions.md and error-types.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.13 -s in_progress

# Add a comment
bd comment ironstar-nyp.13 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.13 -p 1

# View full details
bd show ironstar-nyp.13
```

</details>

---

## 📋 ironstar-nqq.1 Implement CQRS performance tuning

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2025-12-22 00:28 |

### Description

Channel sizing, backpressure handling, metrics instrumentation. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/cqrs/performance-tuning.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nqq`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nqq.1 -s in_progress

# Add a comment
bd comment ironstar-nqq.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nqq.1 -p 1

# View full details
bd show ironstar-nqq.1
```

</details>

---

## 📋 ironstar-jqv.3 Implement concrete session patterns

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2025-12-24 00:54 |

### Description

Implement SessionService CRUD and cookie configuration. create_session_cookie() with HttpOnly, Secure (prod), SameSite::Lax, Max-Age 30 days. generate_session_id() with 192-bit entropy (24 bytes URL-safe base64). See session-management.md and session-implementation.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`
- ⛔ **blocks**: `ironstar-jqv.2`
- 🔗 **depends-on**: `ironstar-nyp.9`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.3 -s in_progress

# Add a comment
bd comment ironstar-jqv.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.3 -p 1

# View full details
bd show ironstar-jqv.3
```

</details>

---

## 📋 ironstar-jqv.2 Implement session security hardening

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:27 |
| **Updated** | 2025-12-23 23:22 |

### Description

CSRF protection, secure cookie attributes, session fixation prevention. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/infrastructure/session-security.md

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`
- ⛔ **blocks**: `ironstar-jqv.1`
- 🔗 **depends-on**: `ironstar-nyp.9`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.2 -s in_progress

# Add a comment
bd comment ironstar-jqv.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.2 -p 1

# View full details
bd show ironstar-jqv.2
```

</details>

---

## 📋 ironstar-jqv.1 Implement GitHub OAuth provider

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:27 |
| **Updated** | 2025-12-24 10:03 |

### Description

Implement GitHub OAuth using oauth2 crate. Create BasicClient with auth/token URIs, user:email scope. Handle /login redirect with CsrfToken, /auth/github/callback with code exchange, profile fetch from /user API, user upsert, session binding. Discard tokens after identity verification. See docs/notes/architecture/decisions/oauth-authentication.md for complete implementation patterns.

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`
- ⛔ **blocks**: `ironstar-2nt.2`
- ⛔ **blocks**: `ironstar-jqv.4`
- ⛔ **blocks**: `ironstar-nyp.11`
- 🔗 **depends-on**: `ironstar-nyp.9`
- ⛔ **blocks**: `ironstar-jqv.10`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.1 -s in_progress

# Add a comment
bd comment ironstar-jqv.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.1 -p 1

# View full details
bd show ironstar-jqv.1
```

</details>

---

## 🚀 ironstar-nqq Performance optimization

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:25 |
| **Updated** | 2025-12-22 00:25 |

### Description

Optional performance patterns for CQRS pipeline including channel sizing, backpressure, debouncing, batching, and rate limiting. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/cqrs/performance-tuning.md and performance-advanced-patterns.md

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nqq -s in_progress

# Add a comment
bd comment ironstar-nqq 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nqq -p 1

# View full details
bd show ironstar-nqq
```

</details>

---

## 📋 ironstar-avp Verify code examples compile and run

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-19 01:16 |

### Description

Test all SQL and Rust code snippets. Ensure hf:// queries work with real datasets. Check that example commands are accurate.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-avp -s in_progress

# Add a comment
bd comment ironstar-avp 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-avp -p 1

# View full details
bd show ironstar-avp
```

</details>

---

## 📋 ironstar-ym1 Polish diagrams for visual consistency

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-19 01:16 |

### Description

Ensure all fletcher diagrams use consistent: node sizing, spacing, colors, edge styles. Consider adding subtle animations or build-up for complex diagrams.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ym1 -s in_progress

# Add a comment
bd comment ironstar-ym1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ym1 -p 1

# View full details
bd show ironstar-ym1
```

</details>

---

## 📋 ironstar-63r Verify technical accuracy of benchmarks

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-19 01:16 |

### Description

Cross-check all performance claims against source papers. Verify: AnnSQL 700x speedup context, 4.4M cell benchmark details, tiledbsoma AWS region claims, DuckLake release date.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-63r -s in_progress

# Add a comment
bd comment ironstar-63r 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-63r -p 1

# View full details
bd show ironstar-63r
```

</details>

---

## 📋 ironstar-z4s Act 4: Expand vision slides

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-19 01:16 |

### Description

Flesh out slides 20-24. Add speaker notes. Consider: more compelling architecture diagram, concrete demo scenario, stronger call-to-action.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`
- ⛔ **blocks**: `ironstar-edx`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-z4s -s in_progress

# Add a comment
bd comment ironstar-z4s 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-z4s -p 1

# View full details
bd show ironstar-z4s
```

</details>

---

## 📋 ironstar-b8d Act 3: Expand web interface slides

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-19 01:16 |

### Description

Flesh out slides 15-19. Add speaker notes. Consider: CellXGene screenshot for comparison, Datastar event flow animation concept, ironstar code snippet refinement.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`
- ⛔ **blocks**: `ironstar-edx`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-b8d -s in_progress

# Add a comment
bd comment ironstar-b8d 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-b8d -p 1

# View full details
bd show ironstar-b8d
```

</details>

---

## 📋 ironstar-a15 Act 2: Expand solution stack slides

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-19 01:16 |

### Description

Flesh out slides 9-14. Add speaker notes. Consider: DuckLake metadata schema visualization, httpfs query flow diagram, concrete hf:// query examples with real datasets.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`
- ⛔ **blocks**: `ironstar-edx`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-a15 -s in_progress

# Add a comment
bd comment ironstar-a15 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-a15 -p 1

# View full details
bd show ironstar-a15
```

</details>

---

## 📋 ironstar-ubj Act 1: Expand data problem slides

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-19 01:16 |

### Description

Flesh out slides 2-8. Add speaker notes. Consider: more concrete examples of AnnData failures, visual showing exponential runtime growth, clearer AnnSQL benchmark presentation.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`
- ⛔ **blocks**: `ironstar-edx`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ubj -s in_progress

# Add a comment
bd comment ironstar-ubj 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ubj -p 1

# View full details
bd show ironstar-ubj
```

</details>

---

## 📋 ironstar-6lq.8 Create reusable Rust CI workflow with workflow_call dispatch

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 18:02 |
| **Updated** | 2025-12-18 21:50 |

### Description

Define a reusable GitHub workflow for Rust CI checks using workflow_call dispatch.

IMPLEMENTATION APPROACH (from session research):

Option A (Recommended): Use nix flake check directly
- ci.yaml runs: nix flake check --impure
- All Rust checks (clippy, fmt, test) run via flake.checks
- Simplest approach, leverages existing nix infrastructure
- Content-addressed caching via Cachix (already configured in nixConfig)

Option B: Per-crate matrix with justfile
- Create crate-test.yaml (parallel to package-test.yaml)
- Use list-crates-json for matrix discovery
- Run: nix develop -c just rust-check
- More granular but adds complexity

AVAILABLE CHECKS (implemented in ironstar-6lq.6):
- checks.*.ironstar-clippy (per-crate, autowired)
- checks.*.rust-fmt (workspace-level)
- checks.*.rust-test (workspace-level)

JUSTFILE TARGETS (for Option B):
- just rust-fmt-check
- just rust-clippy  
- just rust-test
- just rust-check (runs all)
- just list-crates-json (matrix data)

WORKFLOW FILE STRUCTURE (if Option B):
.github/workflows/crate-test.yaml:
- workflow_dispatch + workflow_call triggers
- inputs: crate-name, crate-path, debug-enabled, nix-installer, force-run
- secrets: SOPS_AGE_KEY
- Uses cached-ci-job composite action

Local refs: typescript-nix-template/.github/workflows/package-test.yaml

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-6lq.8 -s in_progress

# Add a comment
bd comment ironstar-6lq.8 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-6lq.8 -p 1

# View full details
bd show ironstar-6lq.8
```

</details>

---

## 📋 ironstar-rjs Document nixpkgs-unstable Darwin framework migration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 17:26 |
| **Updated** | 2025-12-18 17:26 |

### Description

Session discovery: pkgs.darwin.apple_sdk.frameworks is deprecated in nixpkgs-unstable.
Error: darwin.apple_sdk_11_0 has been removed as legacy compatibility stub.

For now, ironstar's crate.nix uses minimal deps (just pkg-config + openssl on Linux).
If Darwin frameworks become needed for specific dependencies:
- Check migration guide: https://nixos.org/manual/nixpkgs/stable/#sec-darwin-legacy-frameworks
- Test with pkgs.apple-sdk.frameworks or direct pkgs.darwin.* access
- May need to pin nixpkgs to a version before the breaking change

This is informational - most Rust builds work fine without explicit Darwin frameworks.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-rjs -s in_progress

# Add a comment
bd comment ironstar-rjs 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-rjs -p 1

# View full details
bd show ironstar-rjs
```

</details>

---

## 📋 ironstar-apx.5 Add structured logging with tracing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Integrate tracing and tracing-subscriber crates for structured logging of events appended, handlers executed, projection updates, and errors. Use span context to correlate logs across request lifecycle.

### Dependencies

- 🔗 **parent-child**: `ironstar-apx`
- ⛔ **blocks**: `ironstar-r62.13`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-apx.5 -s in_progress

# Add a comment
bd comment ironstar-apx.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-apx.5 -p 1

# View full details
bd show ironstar-apx.5
```

</details>

---

## 📋 ironstar-apx.4 Create .env.development template file

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create template .env.development with DATABASE_URL=dev.db, LOG_LEVEL=debug, SERVER_PORT=3000, RELOAD_ENABLED=true, STATIC_DIR=static/dist. Document in README that users should copy to .env for local development. Add .env* to .gitignore.

### Dependencies

- 🔗 **parent-child**: `ironstar-apx`
- ⛔ **blocks**: `ironstar-nyp.3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-apx.4 -s in_progress

# Add a comment
bd comment ironstar-apx.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-apx.4 -p 1

# View full details
bd show ironstar-apx.4
```

</details>

---

## 📋 ironstar-apx.2 Create template parameters and conditional includes

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Implement nix/modules/template.nix defining omnix template parameters: project-name, crate-name, github-ci (conditional .github/workflows), example-todo (conditional examples/), nix-template (conditional nix/modules/template.nix). Follow typescript-nix-template pattern.
Local refs: ~/projects/nix-workspace/typescript-nix-template/modules/template.nix

### Dependencies

- 🔗 **parent-child**: `ironstar-apx`
- ⛔ **blocks**: `ironstar-6lq.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-apx.2 -s in_progress

# Add a comment
bd comment ironstar-apx.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-apx.2 -p 1

# View full details
bd show ironstar-apx.2
```

</details>

---

## 📋 ironstar-apx.1 Create BOOTSTRAP.md with complete setup instructions

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Write BOOTSTRAP.md documenting: prerequisites (Nix, direnv), flake.nix structure overview, Nix modules organization, devShell contents, process-compose processes, development workflow, frontend/backend build separation. Include troubleshooting section.

### Dependencies

- 🔗 **parent-child**: `ironstar-apx`
- ⛔ **blocks**: `ironstar-r62.13`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-apx.1 -s in_progress

# Add a comment
bd comment ironstar-apx.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-apx.1 -p 1

# View full details
bd show ironstar-apx.1
```

</details>

---

## 📋 ironstar-zuv.3 Create end-to-end handler tests

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Write integration tests for complete command/query flow: POST command -> event appended -> broadcast sent -> projection updated -> SSE responds with new state. Use test utilities to initialize temporary AppState.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/tokio

### Dependencies

- 🔗 **parent-child**: `ironstar-zuv`
- ⛔ **blocks**: `ironstar-r62.13`
- ⛔ **blocks**: `ironstar-zuv.2`
- ⛔ **blocks**: `ironstar-zuv.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-zuv.3 -s in_progress

# Add a comment
bd comment ironstar-zuv.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-zuv.3 -p 1

# View full details
bd show ironstar-zuv.3
```

</details>

---

## 📋 ironstar-zuv.2 Create projection tests

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Write tests for ProjectionManager: rebuild from events produces correct state, apply increments state correctly, concurrent applies via RwLock don't lose updates. Mock EventStore and Projection trait implementations.
Local refs: ~/projects/rust-workspace/tokio

### Dependencies

- 🔗 **parent-child**: `ironstar-zuv`
- ⛔ **blocks**: `ironstar-nyp.7`
- ⛔ **blocks**: `ironstar-zuv.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-zuv.2 -s in_progress

# Add a comment
bd comment ironstar-zuv.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-zuv.2 -p 1

# View full details
bd show ironstar-zuv.2
```

</details>

---

## 📋 ironstar-zuv.1 Create event store integration tests

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Write tests for SqliteEventStore: append returns monotonic sequences, query_all returns all events, query_since_sequence returns only newer events, index queries work correctly. Use temp SQLite database for isolation.
Local refs: ~/projects/rust-workspace/sqlx

### Dependencies

- 🔗 **parent-child**: `ironstar-zuv`
- ⛔ **blocks**: `ironstar-nyp.3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-zuv.1 -s in_progress

# Add a comment
bd comment ironstar-zuv.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-zuv.1 -p 1

# View full details
bd show ironstar-zuv.1
```

</details>

---

## 🚀 ironstar-zuv Testing and integration

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Comprehensive test coverage including event store integration tests (SQLite append/query semantics), projection tests (rebuild correctness, RwLock concurrency safety), and end-to-end handler tests (command -> event -> broadcast -> projection -> SSE). Uses temporary databases for isolation and mock implementations for unit testing trait boundaries.

### Dependencies

- ⛔ **blocks**: `ironstar-e6k`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-zuv -s in_progress

# Add a comment
bd comment ironstar-zuv 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-zuv -p 1

# View full details
bd show ironstar-zuv
```

</details>

---

## 📋 ironstar-753.3 Set up Lucide icon build-time inlining

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Configure rolldown.config.ts to import lucide icons and inline SVG into bundle. Create icon helper function in hypertext templates for consistent icon usage. Provides zero-runtime icon system.
Local refs: ~/projects/lakescope-workspace/open-props-ui, ~/projects/rust-workspace/hypertext

### Dependencies

- 🔗 **parent-child**: `ironstar-753`
- ⛔ **blocks**: `ironstar-ny3.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-753.3 -s in_progress

# Add a comment
bd comment ironstar-753.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-753.3 -p 1

# View full details
bd show ironstar-753.3
```

</details>

---

## 📋 ironstar-753.2 Implement sortable-list web component wrapper

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create web-components/components/sortable-list.ts implementing Pattern 1 thin wrapper around SortableJS library. Dispatches custom reorder event with detail containing oldIndex/newIndex. Integrates with Datastar via data-on:reorder.
Local refs: ~/projects/lakescope-workspace/datastar-go-nats-template-northstar

### Dependencies

- 🔗 **parent-child**: `ironstar-753`
- ⛔ **blocks**: `ironstar-ny3.14`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-753.2 -s in_progress

# Add a comment
bd comment ironstar-753.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-753.2 -p 1

# View full details
bd show ironstar-753.2
```

</details>

---

## 📋 ironstar-753.1 Implement VegaChart web component wrapper

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:53 |

### Description

Create VegaChart web component wrapper for Vega-Lite.

Use vanilla web component (Pattern 2 from integration-patterns-visualizations.md).

Configuration:
- renderer: 'svg' (recommended over canvas for accessibility)
- Store Result and View instances
- Call result?.finalize() on disconnectedCallback to prevent memory leaks
- Support view.data() and view.signal() for incremental updates

Template:
```html
<vega-chart data-ignore-morph data-signals-spec="...">
</vega-chart>
```

Ref: docs/notes/architecture/frontend/integration-patterns-visualizations.md

### Dependencies

- 🔗 **parent-child**: `ironstar-753`
- ⛔ **blocks**: `ironstar-ny3.14`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-753.1 -s in_progress

# Add a comment
bd comment ironstar-753.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-753.1 -p 1

# View full details
bd show ironstar-753.1
```

</details>

---

## 🚀 ironstar-753 Third-party library integration

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-22 00:34 |

### Description

Third-party library integration including ECharts and Vega-Lite visualization components. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/frontend/integration-patterns.md and ds-echarts-integration-guide.md

### Dependencies

- ⛔ **blocks**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-09r`
- ⛔ **blocks**: `ironstar-nor`
- ⛔ **blocks**: `ironstar-3gd`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-753 -s in_progress

# Add a comment
bd comment ironstar-753 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-753 -p 1

# View full details
bd show ironstar-753
```

</details>

---

## 🚀 ironstar-e6k Example application (Todo)

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Complete TodoMVC demonstration integrating all architectural layers: Todo domain model with algebraic types, event-sourced state management, SSE-driven UI updates via Datastar, hypertext template rendering, and CQRS command/query separation. Demonstrates the full stack in action with a familiar reference application following northstar patterns adapted for Rust.

### Dependencies

- ⛔ **blocks**: `ironstar-r62`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-e6k -s in_progress

# Add a comment
bd comment ironstar-e6k 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-e6k -p 1

# View full details
bd show ironstar-e6k
```

</details>

---

## 📋 ironstar-r62.15 Implement health check endpoint for process-compose

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create GET /health endpoint that returns 200 OK when server is ready. Used by process-compose readiness_probe to coordinate startup dependency ordering between db-init, backend, frontend, and hotreload processes.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.11`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.15 -s in_progress

# Add a comment
bd comment ironstar-r62.15 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.15 -p 1

# View full details
bd show ironstar-r62.15
```

</details>

---

## 📋 ironstar-r62.14 Implement dev-only hotreload SSE endpoint

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create conditional compilation block (#[cfg(debug_assertions)]) with GET /hotreload SSE endpoint that broadcasts ExecuteScript(window.location.reload()) when triggered, plus POST /hotreload/trigger endpoint. Coordinates with cargo-watch for browser reload on build completion.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.5`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.14 -s in_progress

# Add a comment
bd comment ironstar-r62.14 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.14 -p 1

# View full details
bd show ironstar-r62.14
```

</details>

---

## 📋 ironstar-r62.12 Implement graceful shutdown signal handling

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 18:02 |

### Description

Add tokio signal handling for SIGTERM/SIGINT:
async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(SignalKind::terminate()).unwrap().recv().await;
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
axum::Server::bind(&addr).serve(app.into_make_service()).with_graceful_shutdown(shutdown_signal()).await
Ensures clean shutdown of EventStore, SessionStore, and SSE connections.

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.11`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.12 -s in_progress

# Add a comment
bd comment ironstar-r62.12 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.12 -p 1

# View full details
bd show ironstar-r62.12
```

</details>

---

## 📋 ironstar-r62.11 Implement router composition with feature routes

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 18:02 |

### Description

Create main Router that merges feature modules. Each feature provides route() -> Router<AppState> composing GET/POST/SSE handlers. Use Router::merge to combine features and apply State layer to inject AppState.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.5`
- ⛔ **blocks**: `ironstar-r62.6`
- ⛔ **blocks**: `ironstar-r62.7`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.11 -s in_progress

# Add a comment
bd comment ironstar-r62.11 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.11 -p 1

# View full details
bd show ironstar-r62.11
```

</details>

---

## 📋 ironstar-r62.6 Implement command POST handlers

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 18:02 |

### Description

Create POST handlers using ReadSignals extractor:
async fn add_todo(ReadSignals(cmd): ReadSignals<AddTodoCommand>, State(state): State<AppState>) -> impl IntoResponse {
    let validated = cmd.validate()?;
    let event = aggregate.apply_command(validated)?;
    let seq = state.event_store.append(&event).await?;
    state.event_bus.send(StoredEvent { sequence: seq, event })?;
    StatusCode::ACCEPTED
}
Extracts Command from ReadSignals<T> extractor (requires #[derive(Deserialize)] on signal types), calls command handler (pure logic), appends events to event store, broadcasts to subscribers, and returns 202 Accepted immediately WITHOUT waiting for SSE update. Implements CQRS write path.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.4`
- ⛔ **blocks**: `ironstar-2nt.4`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.6 -s in_progress

# Add a comment
bd comment ironstar-r62.6 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.6 -p 1

# View full details
bd show ironstar-r62.6
```

</details>

---

## 📋 ironstar-r62.5 Implement SSE feed endpoint with event replay

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 18:02 |

### Description

Create async sse_feed handler:
async fn sse_feed(headers: HeaderMap, State(state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let last_event_id = headers.get("Last-Event-ID").and_then(|h| h.to_str().ok()).and_then(|s| s.parse().ok()).unwrap_or(0);
    let replay = state.event_store.query_since_sequence(last_event_id).await.unwrap().into_iter().map(|e| Event::default().id(e.sequence).data(e.payload));
    let live = BroadcastStream::new(state.event_bus.subscribe()).filter_map(|e| e.ok()).map(|e| Event::default().id(e.sequence).data(e.payload));
    Sse::new(stream::iter(replay).chain(live))
}
Extracts Last-Event-ID, subscribes to broadcast channel, replays events since that ID from event store, chains with live stream, and emits SSE events with id field set to sequence number. Implements reconnection recovery.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust, ~/projects/lakescope-workspace/datastar/sdk/ADR.md

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.4`
- ⛔ **blocks**: `ironstar-nyp.8`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.5 -s in_progress

# Add a comment
bd comment ironstar-r62.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.5 -p 1

# View full details
bd show ironstar-r62.5
```

</details>

---

## 📋 ironstar-r62.4 Define AppState struct with all dependencies

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 18:02 |

### Description

Create AppState struct holding Arc<EventStore>, Arc<SessionStore>, Arc<Projections>, broadcast::Sender<StoredEvent>, and optional debug-only reload channel. Implement AppState::new() to initialize all services and replay events to rebuild projections at startup.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/tokio

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-nyp.3`
- ⛔ **blocks**: `ironstar-nyp.10`
- ⛔ **blocks**: `ironstar-nyp.7`
- ⛔ **blocks**: `ironstar-nyp.27`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.4 -s in_progress

# Add a comment
bd comment ironstar-r62.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.4 -p 1

# View full details
bd show ironstar-r62.4
```

</details>

---

## 📋 ironstar-ny3.1 Create web-components/ project structure with package.json

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 18:02 |

### Description

Initialize web-components/ subdirectory with package.json (type: module, scripts: dev/build for Rolldown), tsconfig.json (target ES2020, experimentalDecorators, strict mode), and PostCSS configuration. Establishes the frontend asset build pipeline.
Local refs: ~/projects/lakescope-workspace/open-props, ~/projects/lakescope-workspace/open-props-ui

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.1 -s in_progress

# Add a comment
bd comment ironstar-ny3.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.1 -p 1

# View full details
bd show ironstar-ny3.1
```

</details>

---

## 📋 ironstar-nqq.2 Implement advanced performance patterns

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ☕ Low (P3) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2025-12-22 00:28 |

### Description

Debouncing, batching, rate limiting (optional optimizations). See ~/projects/rust-workspace/ironstar/docs/notes/architecture/cqrs/performance-advanced-patterns.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nqq`
- ⛔ **blocks**: `ironstar-nqq.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nqq.2 -s in_progress

# Add a comment
bd comment ironstar-nqq.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nqq.2 -p 1

# View full details
bd show ironstar-nqq.2
```

</details>

---

## 📋 ironstar-k1z Final review and presentation dry-run

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ☕ Low (P3) |
| **Status** | 🟢 open |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-19 01:16 |

### Description

Complete read-through for flow. Time each section. Identify any remaining gaps. Prepare for Q&A topics.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`
- ⛔ **blocks**: `ironstar-ubj`
- ⛔ **blocks**: `ironstar-a15`
- ⛔ **blocks**: `ironstar-b8d`
- ⛔ **blocks**: `ironstar-z4s`
- ⛔ **blocks**: `ironstar-63r`
- ⛔ **blocks**: `ironstar-ym1`
- ⛔ **blocks**: `ironstar-avp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-k1z -s in_progress

# Add a comment
bd comment ironstar-k1z 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-k1z -p 1

# View full details
bd show ironstar-k1z
```

</details>

---

## 📋 ironstar-nor Research Mosaic visualization integration (TBD)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ☕ Low (P3) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 17:11 |
| **Updated** | 2025-12-18 17:11 |

### Description

Mosaic provides grammar of graphics for large datasets with coordinated views. Integration pattern TBD - requires bridging Mosaic coordinator (push-based query updates) with Datastar signals (SSE-driven state). More complex than Vega-Lite due to competing reactive systems. See integration-patterns.md Mosaic section. Ref: ~/projects/lakescope-workspace/mosaic/

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nor -s in_progress

# Add a comment
bd comment ironstar-nor 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nor -p 1

# View full details
bd show ironstar-nor
```

</details>

---

## 📋 ironstar-apx.3 Define om CLI instantiation tests and metadata

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ☕ Low (P3) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Add om.templates.ironstar definition with template description, parameters array, and integration tests validating: Cargo.toml generation, flake.nix presence, .github/workflows/ci.yml conditionally present, packages.default builds successfully.
Local refs: ~/projects/rust-workspace/rust-nix-template/nix/modules/template.nix

### Dependencies

- 🔗 **parent-child**: `ironstar-apx`
- ⛔ **blocks**: `ironstar-apx.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-apx.3 -s in_progress

# Add a comment
bd comment ironstar-apx.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-apx.3 -p 1

# View full details
bd show ironstar-apx.3
```

</details>

---

## 🚀 ironstar-apx Documentation and template

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ☕ Low (P3) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Template finalization with omnix integration (om CLI parameters, conditional file inclusion), comprehensive BOOTSTRAP.md documentation, environment configuration templates, and structured logging via tracing. Enables users to instantiate ironstar as a template project with parameterized customization following typescript-nix-template and rust-nix-template patterns.

### Dependencies

- ⛔ **blocks**: `ironstar-zuv`
- ⛔ **blocks**: `ironstar-rjs`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-apx -s in_progress

# Add a comment
bd comment ironstar-apx 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-apx -p 1

# View full details
bd show ironstar-apx
```

</details>

---

## 📋 ironstar-nyp.5 Implement tokio broadcast event bus

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ☕ Low (P3) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 15:25 |

### Description

Implement tokio::broadcast fallback for environments with less than 10MB memory constraints. See docs/notes/architecture/infrastructure/distributed-event-bus-migration.md for usage criteria. Primary implementation uses ZenohEventBus. Local refs: ~/projects/rust-workspace/tokio

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`
- ⛔ **blocks**: `ironstar-nyp.19`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.5 -s in_progress

# Add a comment
bd comment ironstar-nyp.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.5 -p 1

# View full details
bd show ironstar-nyp.5
```

</details>

---

## 📋 ironstar-v4y.3 Define common-utils crate structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 💤 Backlog (P4) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:54 |
| **Updated** | 2025-12-24 00:54 |

### Description

Layer 0 foundation crate. Contains: crypto helpers, validation utilities, serialization helpers, extension traits. See crate-architecture.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-v4y`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-v4y.3 -s in_progress

# Add a comment
bd comment ironstar-v4y.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-v4y.3 -p 1

# View full details
bd show ironstar-v4y.3
```

</details>

---

## 📋 ironstar-v4y.2 Define common-types crate structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 💤 Backlog (P4) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:54 |
| **Updated** | 2025-12-24 00:54 |

### Description

Layer 0 foundation crate. Contains: MinorUnit, Timestamp, Sequence newtypes. TodoId, TodoText smart constructors with validation. See crate-architecture.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-v4y`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-v4y.2 -s in_progress

# Add a comment
bd comment ironstar-v4y.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-v4y.2 -p 1

# View full details
bd show ironstar-v4y.2
```

</details>

---

## 📋 ironstar-v4y.1 Define common-enums crate structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 💤 Backlog (P4) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:54 |
| **Updated** | 2025-12-24 00:54 |

### Description

Layer 0 foundation crate. Contains: AggregateType, EventType, ErrorCode, FilterType enums. No ironstar dependencies. See crate-architecture.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-v4y`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-v4y.1 -s in_progress

# Add a comment
bd comment ironstar-v4y.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-v4y.1 -p 1

# View full details
bd show ironstar-v4y.1
```

</details>

---

## 🚀 ironstar-v4y Multi-crate workspace decomposition

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 💤 Backlog (P4) |
| **Status** | 🟢 open |
| **Created** | 2025-12-24 00:44 |
| **Updated** | 2025-12-24 00:44 |

### Description

Implement 8-layer crate decomposition from crate-architecture.md. Includes common-enums/types/utils (Layer 0), ironstar-domain/commands/events (Layer 1), ironstar-app (Layer 2), ironstar-interfaces (Layer 3), ironstar-adapters/analytics/projections/config (Layer 4), ironstar-services (Layer 5), ironstar-web (Layer 6), ironstar binary (Layer 7). Deferred until single-crate grows beyond 800 lines or coupling becomes problematic.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-v4y -s in_progress

# Add a comment
bd comment ironstar-v4y 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-v4y -p 1

# View full details
bd show ironstar-v4y
```

</details>

---

## 📋 ironstar-2nt.1 Initialize src/ directory structure with modular organization

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 17:26 |
| **Closed** | 2025-12-18 17:26 |

### Description

Create src/ subdirectories for domain/ (aggregates, events, commands, values, signals.rs), application/ (command_handlers, query_handlers, projections), infrastructure/ (event_store, session_store, analytics, event_bus), and presentation/ (routes, handlers, templates). Create placeholder mod.rs files.
Local refs: CLAUDE.md Project structure section

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`

---

## 📋 ironstar-6lq.6 Add Rust checks to flake.checks for CI integration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 21:50 |
| **Closed** | 2025-12-18 21:50 |

### Description

Create Rust checks for CI integration. Options to evaluate:

1. rust-flake provides built-in checks via rust-project configuration - may already be available
2. If custom checks needed, create nix/modules/checks.nix defining perSystem.checks:
   - cargo-test: cargo test --workspace --all-features
   - cargo-clippy: cargo clippy --workspace --all-targets -- -D warnings
   - cargo-fmt: cargo fmt --all -- --check
   - cargo-doc: cargo doc --workspace --no-deps --document-private-items

Session learning: ironstar uses import-tree (not nix/modules/ path) - any new module goes in modules/ directly.

Verify what rust-flake provides first: nix flake show | grep checks

Local refs: ~/projects/rust-workspace/rust-nix-template

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.4`

---

## 📋 ironstar-6lq.5 Verify cargo check passes with workspace configuration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 17:23 |
| **Closed** | 2025-12-18 17:23 |

### Description

Run cargo check to validate workspace configuration, dependency resolution, and that all crates compile. Fix any issues with feature flags or dependency versions. Ensures Rust workspace is properly configured before proceeding to process orchestration.

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.4`

---

## 📋 ironstar-6lq.4 Set up per-crate crate.nix pattern for crane args

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 17:23 |
| **Closed** | 2025-12-18 17:23 |

### Description

Create crate.nix files for each workspace crate defining crane-specific build arguments:
{ lib, pkgs, rustPlatform, ... }:
{
  pname = "ironstar";
  buildInputs = [ ... ];
  nativeBuildInputs = [ ... ];
  preBuild = ''export DATABASE_URL=file:test.db'';
}
Implements pattern from rustlings-workspace for granular build customization. Set DATABASE_URL env var for sqlx compile-time query validation.
Local refs: ~/projects/rust-workspace/rustlings-workspace/

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.3`

---

## 📋 ironstar-6lq.3 Configure Cargo.toml with workspace structure (resolver = 2)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 17:23 |
| **Closed** | 2025-12-18 17:23 |

### Description

Create Cargo.toml at repository root with [workspace], resolver = "2", members array, and workspace.dependencies section for DRY dependency management. Include all core dependencies: axum, tokio, sqlx, duckdb, ts-rs, datastar, hypertext, redb, rust-embed, thiserror. Add release profile optimizations.
Local refs: ~/projects/rust-workspace/rustlings-workspace/Cargo.toml

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.2`

---

## 📋 ironstar-6lq.2 Add rust-toolchain.toml with required components

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 17:23 |
| **Closed** | 2025-12-18 17:23 |

### Description

Create rust-toolchain.toml at repository root specifying stable channel with components: rustfmt, clippy, rust-analyzer, rust-src. Ensures consistent Rust version across development environments and CI.
Local refs: ~/projects/rust-workspace/rust-nix-template/rust-toolchain.toml

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.1`

---

## 📋 ironstar-6lq.1 Integrate rust-flake patterns (crane, rust-overlay)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 17:23 |
| **Closed** | 2025-12-18 17:23 |

### Description

Create nix/modules/rust.nix importing rust-flake module. Configure:
- crane with per-crate crane.args via callPackage pattern
- rust-overlay for toolchain management with rust-toolchain.toml
- Platform-specific buildInputs: darwin.apple_sdk.frameworks (Security, SystemConfiguration), pkgs.openssl
- Syntax: perSystem = { config, pkgs, lib, ... }: { rust-flake.crateOverrides = ...; }
Establishes deterministic Rust build infrastructure with native dependency handling.
Local refs: ~/projects/rust-workspace/rust-nix-template/nix/modules/rust.nix, ~/projects/rust-workspace/rustlings-workspace/nix/modules/rust.nix

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`

---

## 📋 ironstar-cxe.5 Create .gitignore with comprehensive patterns

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 13:42 |
| **Closed** | 2025-12-18 13:42 |

### Description

Create .gitignore at repository root with patterns: /target/, Cargo.lock, /static/dist/, web-components/dist, node_modules, .env*, dev.db*, .DS_Store, .direnv, result, .beads/. Protects against accidental secret commits and build artifacts.

### Dependencies

- 🔗 **parent-child**: `ironstar-cxe`

---

## 📋 ironstar-cxe.4 Create initial git commit with generated structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 13:43 |
| **Closed** | 2025-12-18 13:43 |

### Description

Stage all generated files from om init and create initial commit with message: 'feat: initialize ironstar from typescript-nix-template'. Establishes baseline for tracking subsequent changes.

### Dependencies

- 🔗 **parent-child**: `ironstar-cxe`
- ⛔ **blocks**: `ironstar-cxe.3`
- ⛔ **blocks**: `ironstar-cxe.2`

---

## 📋 ironstar-cxe.3 Verify nix develop enters working development shell

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 13:42 |
| **Closed** | 2025-12-18 13:42 |

### Description

Test that nix develop successfully enters the devShell with basic tooling available. Verify nixd, direnv, and foundational utilities are present. This validates the template instantiation before proceeding to Rust integration.

### Dependencies

- 🔗 **parent-child**: `ironstar-cxe`
- ⛔ **blocks**: `ironstar-cxe.1`

---

## 📋 ironstar-cxe.2 Configure secrets management and string replacement

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 13:42 |
| **Closed** | 2025-12-18 13:42 |

### Description

Secrets are managed via sops-encrypted vars/*.yaml files following the typescript-nix-template pattern, rather than .env files. The .sops.yaml configuration and vars/ directory structure were added during template instantiation. Runtime secrets are decrypted by sops and injected via the devShell or CI workflows. The .gitignore already includes patterns to prevent accidental secret commits (.env*, vars/* with exceptions for encrypted yaml/json).

### Dependencies

- 🔗 **parent-child**: `ironstar-cxe`
- ⛔ **blocks**: `ironstar-cxe.1`

---

## 📋 ironstar-cxe.1 Run om init with typescript-nix-template parameters

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 11:14 |
| **Closed** | 2025-12-18 11:14 |

### Description

Execute om init with typescript-nix-template to generate the initial flake structure with flake-parts, import-tree module composition, treefmt-nix formatting, nix-unit testing, git-hooks, and GitHub Actions workflows.

Template source: github:sciexp/typescript-nix-template/main

Parameters:
- project-name: ironstar
- npm-scope: @ironstar
- git-org: sciexp
- author: Ironstar Developers
- author-email: ironstar@scientistexperience.net
- project-description: Rust + Datastar template for reactive, event-sourced web applications with hypermedia-driven architecture
- cloudflare-worker-name: ironstar-docs
- production-url: ironstar.scientistexperience.net
- github-ci: true
- vscode: true
- docs: true
- nix-template: false

Execution plan:
1. Run om init to ~/projects/rust-workspace/ironstar-init (temp directory)
2. Selectively merge files into existing ironstar repo with atomic commits
3. Preserve existing docs/notes/ content
4. Set up docs/ symlinks following typescript-nix-template pattern
5. Merge .gitignore and .gitattributes (preserve beads merge driver)

Local refs: ~/projects/nix-workspace/typescript-nix-template

### Dependencies

- 🔗 **parent-child**: `ironstar-cxe`

---

## 🚀 ironstar-cxe Template instantiation

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 13:43 |
| **Closed** | 2025-12-18 13:43 |

### Description

Bootstrap the ironstar project from typescript-nix-template using omnix om CLI. This epic establishes the foundational Nix flake structure with deterministic development environments, secrets management patterns, and git repository initialization. Validates that the template instantiation succeeds before proceeding to Rust-specific integration.

---

## 📋 ironstar-edx Review narrative arc and timing estimates

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-19 01:28 |
| **Closed** | 2025-12-19 01:28 |

### Description

Validate the 4-act structure (15-15-10-5 min). Ensure logical flow from problem to solution to interface to vision. Check that each slide has one clear concept.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`

---

## 📋 ironstar-nyp.33 Implement session cleanup background task

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2025-12-26 18:29 |
| **Closed** | 2025-12-26 18:29 |

### Description

Spawn background task to clean up expired sessions.

Pattern from session-implementation.md:
```rust
async fn spawn_session_cleanup(pool: SqlitePool, interval: Duration) {
    let mut ticker = tokio::time::interval(interval);
    loop {
        ticker.tick().await;
        let _ = sqlx::query!("DELETE FROM sessions WHERE expires_at < ?", Utc::now())
            .execute(&pool)
            .await;
    }
}
```

Run on configurable interval (default: 1 hour).

Ref: docs/notes/architecture/infrastructure/session-implementation.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

---

## 📋 ironstar-9oj Implement cache invalidation for analytics queries

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 00:43 |
| **Updated** | 2025-12-26 15:52 |
| **Closed** | 2025-12-26 15:52 |

### Description

Implement cache invalidation via Zenoh subscription. Subscribe to aggregate-type key expressions and invalidate corresponding cache entries. See docs/notes/architecture/infrastructure/analytics-cache-architecture.md Pattern 4.

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- ⛔ **blocks**: `ironstar-nyp.12`

---

## 📋 ironstar-r5f ironstar-6lq

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 18:03 |
| **Updated** | 2025-12-24 00:45 |
| **Closed** | 2025-12-24 00:45 |

---

## 📋 ironstar-nyp.16 Implement DualEventBus for tokio::broadcast to Zenoh migration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ☕ Low (P3) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 00:44 |
| **Updated** | 2025-12-26 15:26 |
| **Closed** | 2025-12-26 15:26 |

### Description

Implement DualEventBus coexistence pattern from distributed-event-bus-migration.md. Phase 1: add Zenoh alongside broadcast. Phase 2: incremental subscriber migration. Phase 3: remove broadcast. Deferred until scaling beyond ~256 SSE subscribers.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

---

