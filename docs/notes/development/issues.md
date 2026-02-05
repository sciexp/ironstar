# Beads Export

*Generated: Thu, 05 Feb 2026 09:38:26 EST*

## Summary

| Metric | Count |
|--------|-------|
| **Total** | 304 |
| Open | 92 |
| In Progress | 0 |
| Blocked | 0 |
| Closed | 212 |

## Quick Actions

Ready-to-run commands for bulk operations:

```bash
# Close open items (92 total, showing first 10)
bd close ironstar-r62.16 ironstar-753 ironstar-r62.8 ironstar-r62.2 ironstar-r62.1 ironstar-r62 ironstar-nyp ironstar-ny3 ironstar-f8b.5 ironstar-f8b.4

# View high-priority items (P0/P1)
bd show ironstar-r62.16 ironstar-753 ironstar-r62.8 ironstar-r62.2 ironstar-r62.1 ironstar-r62 ironstar-nyp ironstar-ny3 ironstar-f8b.5 ironstar-f8b.4 ironstar-f8b.3 ironstar-f8b.2 ironstar-f8b.1 ironstar-f8b ironstar-sgc ironstar-jqv.13 ironstar-b2l ironstar-58f ironstar-nyp.36 ironstar-jqv.12 ironstar-nyp.30 ironstar-nyp.22 ironstar-ny3.17 ironstar-ny3.16 ironstar-nyp.21 ironstar-jqv.7 ironstar-jqv ironstar-amw ironstar-b9h ironstar-r62.3 ironstar-nyp.4 ironstar-ny3.6

```

## Table of Contents

- [🟢 ironstar-r62.16 Implement DatastarRequest extractor for HTML/SSE routing](#ironstar-r62-16-implement-datastarrequest-extractor-for-html-sse-routing)
- [🟢 ironstar-753 Third-party library integration](#ironstar-753-third-party-library-integration)
- [🟢 ironstar-r62.8 Implement RenderableToDatastar conversion trait](#ironstar-r62-8-implement-renderabletodatastar-conversion-trait)
- [🟢 ironstar-r62.2 Create devShell module with tools and environment](#ironstar-r62-2-create-devshell-module-with-tools-and-environment)
- [🟢 ironstar-r62.1 Add justfile with development and build tasks](#ironstar-r62-1-add-justfile-with-development-and-build-tasks)
- [🟢 ironstar-r62 Presentation layer](#ironstar-r62-presentation-layer)
- [🟢 ironstar-nyp Event sourcing infrastructure](#ironstar-nyp-event-sourcing-infrastructure)
- [🟢 ironstar-ny3 Frontend build pipeline](#ironstar-ny3-frontend-build-pipeline)
- [🟢 ironstar-f8b.5 Verify process-compose up works with all services](#ironstar-f8b-5-verify-process-compose-up-works-with-all-services)
- [🟢 ironstar-f8b.4 Configure cargo-watch to curl hotreload trigger on success](#ironstar-f8b-4-configure-cargo-watch-to-curl-hotreload-trigger-on-success)
- [🟢 ironstar-f8b.3 Set up service orchestration (frontend bundler, cargo-watch)](#ironstar-f8b-3-set-up-service-orchestration-frontend-bundler-cargo-watch)
- [🟢 ironstar-f8b.2 Configure process-compose.yaml for dev services](#ironstar-f8b-2-configure-process-compose-yaml-for-dev-services)
- [🟢 ironstar-f8b.1 Integrate process-compose-flake patterns into devShell](#ironstar-f8b-1-integrate-process-compose-flake-patterns-into-devshell)
- [🟢 ironstar-f8b Process compose integration](#ironstar-f8b-process-compose-integration)
- [🟢 ironstar-sgc Implement CSS manifest handling for production builds](#ironstar-sgc-implement-css-manifest-handling-for-production-builds)
- [🟢 ironstar-jqv.13 Add SessionMetadata to Session aggregate](#ironstar-jqv-13-add-sessionmetadata-to-session-aggregate)
- [🟢 ironstar-b2l Design Zenoh key expression schema for bounded context routing](#ironstar-b2l-design-zenoh-key-expression-schema-for-bounded-context-routing)
- [🟢 ironstar-58f Implement ViewStateRepository for SQLite](#ironstar-58f-implement-viewstaterepository-for-sqlite)
- [🟢 ironstar-nyp.36 Document and enforce subscribe-before-replay invariant](#ironstar-nyp-36-document-and-enforce-subscribe-before-replay-invariant)
- [🟢 ironstar-jqv.12 Implement session regeneration and user binding in OAuth callback](#ironstar-jqv-12-implement-session-regeneration-and-user-binding-in-oauth-callback)
- [🟢 ironstar-nyp.30 Implement observability initialization with dev/prod splitting](#ironstar-nyp-30-implement-observability-initialization-with-dev-prod-splitting)
- [🟢 ironstar-nyp.22 Implement InfrastructureError type with database/network variants](#ironstar-nyp-22-implement-infrastructureerror-type-with-database-network-variants)
- [🟢 ironstar-ny3.17 Implement light-dark() theming with prefers-color-scheme](#ironstar-ny3-17-implement-light-dark-theming-with-prefers-color-scheme)
- [🟢 ironstar-ny3.16 Configure OKLch color system with Open Props syntax](#ironstar-ny3-16-configure-oklch-color-system-with-open-props-syntax)
- [🟢 ironstar-nyp.21 Implement observability initialization module](#ironstar-nyp-21-implement-observability-initialization-module)
- [🟢 ironstar-jqv.7 Implement AuthContext axum extractor](#ironstar-jqv-7-implement-authcontext-axum-extractor)
- [🟢 ironstar-jqv Authentication and authorization](#ironstar-jqv-authentication-and-authorization)
- [🟢 ironstar-amw Configure SQLite production PRAGMA settings (WAL, synchronous, cache)](#ironstar-amw-configure-sqlite-production-pragma-settings-wal-synchronous-cache)
- [🟢 ironstar-b9h Configure tower-http Brotli compression for SSE responses](#ironstar-b9h-configure-tower-http-brotli-compression-for-sse-responses)
- [🟢 ironstar-r62.3 Configure pre-commit hooks for code quality](#ironstar-r62-3-configure-pre-commit-hooks-for-code-quality)
- [🟢 ironstar-nyp.4 Implement SQLite connection pooling and configuration](#ironstar-nyp-4-implement-sqlite-connection-pooling-and-configuration)
- [🟢 ironstar-ny3.6 Copy Open Props UI component CSS files](#ironstar-ny3-6-copy-open-props-ui-component-css-files)
- [🟢 ironstar-wis Optimize CI: replace nix develop -c with direct nix run](#ironstar-wis-optimize-ci-replace-nix-develop-c-with-direct-nix-run)
- [🟢 ironstar-o66.5 Verify cachix effectiveness for workspace cargoArtifacts](#ironstar-o66-5-verify-cachix-effectiveness-for-workspace-cargoartifacts)
- [🟢 ironstar-ny3.22 Investigate unifying web-components under bun monorepo with nix-compatible builds](#ironstar-ny3-22-investigate-unifying-web-components-under-bun-monorepo-with-nix-compatible-builds)
- [🟢 ironstar-a9b.14 Document CommandPipelineError in architecture docs](#ironstar-a9b-14-document-commandpipelineerror-in-architecture-docs)
- [🟢 ironstar-507.4 Integrate Zenoh event publishing for Session](#ironstar-507-4-integrate-zenoh-event-publishing-for-session)
- [🟢 ironstar-507.2 Wire ActiveSessionView projection](#ironstar-507-2-wire-activesessionview-projection)
- [🟢 ironstar-507.1 Wire Session EventSourcedAggregate to SQLite](#ironstar-507-1-wire-session-eventsourcedaggregate-to-sqlite)
- [🟢 ironstar-jqv.14 Document auth evolution strategy](#ironstar-jqv-14-document-auth-evolution-strategy)
- [🟢 ironstar-9dh Reference: Bounded context patterns](#ironstar-9dh-reference-bounded-context-patterns)
- [🟢 ironstar-k94 Reference: Strategic domain classification](#ironstar-k94-reference-strategic-domain-classification)
- [🟢 ironstar-53t Reference: Hoffman's Laws compliance mapping](#ironstar-53t-reference-hoffman-s-laws-compliance-mapping)
- [🟢 ironstar-sj6 Reference: DDD Starter Modelling Process integration](#ironstar-sj6-reference-ddd-starter-modelling-process-integration)
- [🟢 ironstar-0ha Document SSE projection function semantics](#ironstar-0ha-document-sse-projection-function-semantics)
- [🟢 ironstar-2vp Test bitemporal semantics and SSE reconnection edge cases](#ironstar-2vp-test-bitemporal-semantics-and-sse-reconnection-edge-cases)
- [🟢 ironstar-72q Verify Zenoh key filtering preserves free monoid structure](#ironstar-72q-verify-zenoh-key-filtering-preserves-free-monoid-structure)
- [🟢 ironstar-a1s Verify catamorphism uniqueness with property-based tests](#ironstar-a1s-verify-catamorphism-uniqueness-with-property-based-tests)
- [🟢 ironstar-753.7 Document web components as coalgebras with bisimulation testing](#ironstar-753-7-document-web-components-as-coalgebras-with-bisimulation-testing)
- [🟢 ironstar-r62.17 Implement comonadic signal composition laws verification](#ironstar-r62-17-implement-comonadic-signal-composition-laws-verification)
- [🟢 ironstar-nyp.40 Document CQRS as profunctor P: Command^op × View → Set](#ironstar-nyp-40-document-cqrs-as-profunctor-p-command-op-view-set)
- [🟢 ironstar-nyp.39 Coordinate event time, processing time, and table version time](#ironstar-nyp-39-coordinate-event-time-processing-time-and-table-version-time)
- [🟢 ironstar-nyp.38 Implement quotient equivalence testing for projections](#ironstar-nyp-38-implement-quotient-equivalence-testing-for-projections)
- [🟢 ironstar-nyp.37 Document Galois connection properties in Projection trait](#ironstar-nyp-37-document-galois-connection-properties-in-projection-trait)
- [🟢 ironstar-zuv.4 Implement DeciderTestSpecification given/when/then DSL](#ironstar-zuv-4-implement-decidertestspecification-given-when-then-dsl)
- [🟢 ironstar-nyp.32 Instrument Zenoh event bus with Prometheus metrics](#ironstar-nyp-32-instrument-zenoh-event-bus-with-prometheus-metrics)
- [🟢 ironstar-nyp.28 Implement per-session Zenoh subscriptions for SSE streams](#ironstar-nyp-28-implement-per-session-zenoh-subscriptions-for-sse-streams)
- [🟢 ironstar-nyp.24 Add CQRS pipeline span context propagation](#ironstar-nyp-24-add-cqrs-pipeline-span-context-propagation)
- [🟢 ironstar-nyp.23 Configure dev vs prod logging subscribers](#ironstar-nyp-23-configure-dev-vs-prod-logging-subscribers)
- [🟢 ironstar-jqv.11 Implement session rate limiting with sliding window](#ironstar-jqv-11-implement-session-rate-limiting-with-sliding-window)
- [🟢 ironstar-nyp.20 Implement Prometheus metrics endpoint and instrumentation](#ironstar-nyp-20-implement-prometheus-metrics-endpoint-and-instrumentation)
- [🟢 ironstar-jqv.10 Implement OAuth CSRF state validation](#ironstar-jqv-10-implement-oauth-csrf-state-validation)
- [🟢 ironstar-jqv.9 Implement RequireAuth axum extractor](#ironstar-jqv-9-implement-requireauth-axum-extractor)
- [🟢 ironstar-jqv.8 Implement session regeneration for fixation prevention](#ironstar-jqv-8-implement-session-regeneration-for-fixation-prevention)
- [🟢 ironstar-jqv.6 Implement RBAC authorization patterns](#ironstar-jqv-6-implement-rbac-authorization-patterns)
- [🟢 ironstar-nyp.18 Implement SSE ConnectionTracker with atomic counter](#ironstar-nyp-18-implement-sse-connectiontracker-with-atomic-counter)
- [🟢 ironstar-nyp.17 Implement EventUpcaster trait and UpcasterChain for schema evolution](#ironstar-nyp-17-implement-eventupcaster-trait-and-upcasterchain-for-schema-evolution)
- [🟢 ironstar-jqv.5 Create user_identities table for multi-provider support](#ironstar-jqv-5-create-user-identities-table-for-multi-provider-support)
- [🟢 ironstar-jqv.4 Implement users table schema and UserService](#ironstar-jqv-4-implement-users-table-schema-and-userservice)
- [🟢 ironstar-nyp.14 Implement metrics and observability reference](#ironstar-nyp-14-implement-metrics-and-observability-reference)
- [🟢 ironstar-nyp.13 Document error handling decisions](#ironstar-nyp-13-document-error-handling-decisions)
- [🟢 ironstar-jqv.3 Implement concrete session patterns](#ironstar-jqv-3-implement-concrete-session-patterns)
- [🟢 ironstar-jqv.2 Implement session security hardening](#ironstar-jqv-2-implement-session-security-hardening)
- [🟢 ironstar-jqv.1 Implement GitHub OAuth provider](#ironstar-jqv-1-implement-github-oauth-provider)
- [🟢 ironstar-rjs Document nixpkgs-unstable Darwin framework migration](#ironstar-rjs-document-nixpkgs-unstable-darwin-framework-migration)
- [🟢 ironstar-apx.5 Add structured logging with tracing](#ironstar-apx-5-add-structured-logging-with-tracing)
- [🟢 ironstar-apx.4 Create .env.development template file](#ironstar-apx-4-create-env-development-template-file)
- [🟢 ironstar-apx.2 Create template parameters and conditional includes](#ironstar-apx-2-create-template-parameters-and-conditional-includes)
- [🟢 ironstar-apx.1 Create BOOTSTRAP.md with complete setup instructions](#ironstar-apx-1-create-bootstrap-md-with-complete-setup-instructions)
- [🟢 ironstar-zuv.3 Create end-to-end handler tests](#ironstar-zuv-3-create-end-to-end-handler-tests)
- [🟢 ironstar-zuv.2 Create projection tests with mock EventRepository](#ironstar-zuv-2-create-projection-tests-with-mock-eventrepository)
- [🟢 ironstar-zuv.1 Create EventRepository integration tests](#ironstar-zuv-1-create-eventrepository-integration-tests)
- [🟢 ironstar-zuv Testing and integration](#ironstar-zuv-testing-and-integration)
- [🟢 ironstar-753.3 Set up Lucide icon build-time inlining](#ironstar-753-3-set-up-lucide-icon-build-time-inlining)
- [🟢 ironstar-753.2 Implement sortable-list web component wrapper](#ironstar-753-2-implement-sortable-list-web-component-wrapper)
- [🟢 ironstar-753.1 Implement VegaChart web component wrapper](#ironstar-753-1-implement-vegachart-web-component-wrapper)
- [🟢 ironstar-r62.15 Implement health check endpoint for process-compose](#ironstar-r62-15-implement-health-check-endpoint-for-process-compose)
- [🟢 ironstar-r62.14 Implement dev-only hotreload SSE endpoint](#ironstar-r62-14-implement-dev-only-hotreload-sse-endpoint)
- [🟢 ironstar-nor Research Mosaic visualization integration (TBD)](#ironstar-nor-research-mosaic-visualization-integration-tbd)
- [🟢 ironstar-apx.3 Define om CLI instantiation tests and metadata](#ironstar-apx-3-define-om-cli-instantiation-tests-and-metadata)
- [🟢 ironstar-apx Documentation and template](#ironstar-apx-documentation-and-template)
- [🟢 ironstar-nyp.5 Implement tokio broadcast event bus](#ironstar-nyp-5-implement-tokio-broadcast-event-bus)
- [⚫ ironstar-nyp.48 Implement Analytics HTTP routes and SSE feed](#ironstar-nyp-48-implement-analytics-http-routes-and-sse-feed)
- [⚫ ironstar-nyp.45 Implement QuerySession command handler with Zenoh publishing](#ironstar-nyp-45-implement-querysession-command-handler-with-zenoh-publishing)
- [⚫ ironstar-nyp.44 Implement Catalog event repository adapter](#ironstar-nyp-44-implement-catalog-event-repository-adapter)
- [⚫ ironstar-nyp.43 Implement QuerySession event repository adapter](#ironstar-nyp-43-implement-querysession-event-repository-adapter)
- [⚫ ironstar-nyp.41 Implement Catalog aggregate Decider (decide/evolve)](#ironstar-nyp-41-implement-catalog-aggregate-decider-decide-evolve)
- [⚫ ironstar-ny3.19 Add datastar.js to Rolldown build pipeline](#ironstar-ny3-19-add-datastar-js-to-rolldown-build-pipeline)
- [⚫ ironstar-2it.23 Resolve UserId identity model: composite key pattern](#ironstar-2it-23-resolve-userid-identity-model-composite-key-pattern)
- [⚫ ironstar-b43 Implement error type hierarchy](#ironstar-b43-implement-error-type-hierarchy)
- [⚫ ironstar-a9b.12 Implement Decider specification tests](#ironstar-a9b-12-implement-decider-specification-tests)
- [⚫ ironstar-a9b.9 Integrate Zenoh event publishing](#ironstar-a9b-9-integrate-zenoh-event-publishing)
- [⚫ ironstar-a9b.7 Wire Todo EventSourcedAggregate](#ironstar-a9b-7-wire-todo-eventsourcedaggregate)
- [⚫ ironstar-a9b.4 Implement Todo Decider](#ironstar-a9b-4-implement-todo-decider)
- [⚫ ironstar-a9b.3 Create event store SQLite schema](#ironstar-a9b-3-create-event-store-sqlite-schema)
- [⚫ ironstar-a9b.2 Implement fmodel-rust identifier traits](#ironstar-a9b-2-implement-fmodel-rust-identifier-traits)
- [⚫ ironstar-a9b.1 Implement SQLite EventRepository](#ironstar-a9b-1-implement-sqlite-eventrepository)
- [⚫ ironstar-nyp.35 Implement hybrid event store schema with dual sequence columns](#ironstar-nyp-35-implement-hybrid-event-store-schema-with-dual-sequence-columns)
- [⚫ ironstar-nyp.34 Implement spawn-after-persist for DuckDB query execution](#ironstar-nyp-34-implement-spawn-after-persist-for-duckdb-query-execution)
- [⚫ ironstar-2nt.17 Implement AnalyticsError with UUID correlation](#ironstar-2nt-17-implement-analyticserror-with-uuid-correlation)
- [⚫ ironstar-2nt.16 Define analytics workflow as pure function pipeline](#ironstar-2nt-16-define-analytics-workflow-as-pure-function-pipeline)
- [⚫ ironstar-2nt.15 Define analytics value objects (DatasetRef, SqlQuery, ChartConfig)](#ironstar-2nt-15-define-analytics-value-objects-datasetref-sqlquery-chartconfig)
- [⚫ ironstar-2nt.14 Define QuerySession aggregate with typed holes](#ironstar-2nt-14-define-querysession-aggregate-with-typed-holes)
- [⚫ ironstar-jdk Migrate from cargoTest to cargoNextest with dual devshell/CI support](#ironstar-jdk-migrate-from-cargotest-to-cargonextest-with-dual-devshell-ci-support)
- [⚫ ironstar-2nt.13 Enforce async/sync boundary via module organization](#ironstar-2nt-13-enforce-async-sync-boundary-via-module-organization)
- [⚫ ironstar-nyp.29 Implement error propagation pattern through CQRS pipeline](#ironstar-nyp-29-implement-error-propagation-pattern-through-cqrs-pipeline)
- [⚫ ironstar-2nt.12 Implement UUID-tracked error type for distributed correlation](#ironstar-2nt-12-implement-uuid-tracked-error-type-for-distributed-correlation)
- [⚫ ironstar-2nt.11 Add version(&self) -> u64 to Aggregate trait](#ironstar-2nt-11-add-version-self-u64-to-aggregate-trait)
- [⚫ ironstar-nyp.26 Create Zenoh embedded router configuration](#ironstar-nyp-26-create-zenoh-embedded-router-configuration)
- [⚫ ironstar-753.6 Implement chart SSE endpoint with signal-driven options](#ironstar-753-6-implement-chart-sse-endpoint-with-signal-driven-options)
- [⚫ ironstar-2nt.9 Define ChartSignals and ChartSelection types with ts-rs](#ironstar-2nt-9-define-chartsignals-and-chartselection-types-with-ts-rs)
- [⚫ ironstar-961 Implement DuckDB connection lifecycle management](#ironstar-961-implement-duckdb-connection-lifecycle-management)
- [⚫ ironstar-9b1 Implement httpfs extension configuration for DuckDB](#ironstar-9b1-implement-httpfs-extension-configuration-for-duckdb)
- [⚫ ironstar-3gd Scientific Data Integration](#ironstar-3gd-scientific-data-integration)
- [⚫ ironstar-753.5 Implement ds-echarts build and test integration](#ironstar-753-5-implement-ds-echarts-build-and-test-integration)
- [⚫ ironstar-753.4 Implement ds-echarts backend support](#ironstar-753-4-implement-ds-echarts-backend-support)
- [⚫ ironstar-c7z Implement DuckDB remote data source integration (DuckLake/HF pattern)](#ironstar-c7z-implement-duckdb-remote-data-source-integration-ducklake-hf-pattern)
- [⚫ ironstar-09r Implement ds-echarts Lit web component wrapper](#ironstar-09r-implement-ds-echarts-lit-web-component-wrapper)
- [⚫ ironstar-r62.9 Create base layout template with Datastar initialization](#ironstar-r62-9-create-base-layout-template-with-datastar-initialization)
- [⚫ ironstar-r62.7 Implement query GET handlers](#ironstar-r62-7-implement-query-get-handlers)
- [⚫ ironstar-nyp.12 Implement DuckDB analytics service](#ironstar-nyp-12-implement-duckdb-analytics-service)
- [⚫ ironstar-nyp.8 Implement SSE 15-second keep-alive comment stream](#ironstar-nyp-8-implement-sse-15-second-keep-alive-comment-stream)
- [⚫ ironstar-nyp.6 Create Projection trait for read models](#ironstar-nyp-6-create-projection-trait-for-read-models)
- [⚫ ironstar-nyp.2 Create EventStore trait abstraction](#ironstar-nyp-2-create-eventstore-trait-abstraction)
- [⚫ ironstar-nyp.1 Create database migrations/ directory with schema.sql](#ironstar-nyp-1-create-database-migrations-directory-with-schema-sql)
- [⚫ ironstar-ny3.13 Implement rust-embed conditional asset serving](#ironstar-ny3-13-implement-rust-embed-conditional-asset-serving)
- [⚫ ironstar-ny3.12 Implement manifest.json parser for hashed filename resolution](#ironstar-ny3-12-implement-manifest-json-parser-for-hashed-filename-resolution)
- [⚫ ironstar-ny3.11 Create static/dist/ output directory structure](#ironstar-ny3-11-create-static-dist-output-directory-structure)
- [⚫ ironstar-ny3.10 Configure ts-rs export directory and justfile task](#ironstar-ny3-10-configure-ts-rs-export-directory-and-justfile-task)
- [⚫ ironstar-ny3.9 Add ts-rs dependency to Cargo.toml](#ironstar-ny3-9-add-ts-rs-dependency-to-cargo-toml)
- [⚫ ironstar-ny3.8 Create web-components/index.ts entry point](#ironstar-ny3-8-create-web-components-index-ts-entry-point)
- [⚫ ironstar-ny3.7 Create TypeScript configuration (tsconfig.json)](#ironstar-ny3-7-create-typescript-configuration-tsconfig-json)
- [⚫ ironstar-ny3.5 Configure CSS cascade layers for predictable specificity](#ironstar-ny3-5-configure-css-cascade-layers-for-predictable-specificity)
- [⚫ ironstar-ny3.4 Setup Open Props design tokens and theme layer](#ironstar-ny3-4-setup-open-props-design-tokens-and-theme-layer)
- [⚫ ironstar-ny3.3 Setup PostCSS configuration for modern CSS features](#ironstar-ny3-3-setup-postcss-configuration-for-modern-css-features)
- [⚫ ironstar-ny3.2 Configure Rolldown bundler with content-based hashing](#ironstar-ny3-2-configure-rolldown-bundler-with-content-based-hashing)
- [⚫ ironstar-ny3.1 Create web-components/ project structure with package.json](#ironstar-ny3-1-create-web-components-project-structure-with-package-json)
- [⚫ ironstar-2nt.7 Implement command validation pattern with Result types](#ironstar-2nt-7-implement-command-validation-pattern-with-result-types)
- [⚫ ironstar-2nt.6 Enforce camelCase convention for Datastar signal fields](#ironstar-2nt-6-enforce-camelcase-convention-for-datastar-signal-fields)
- [⚫ ironstar-2nt.5 Create Datastar signal types with ts-rs derives](#ironstar-2nt-5-create-datastar-signal-types-with-ts-rs-derives)
- [⚫ ironstar-2nt.4 Design aggregate root state machines](#ironstar-2nt-4-design-aggregate-root-state-machines)
- [⚫ ironstar-2nt.3 Implement value objects and smart constructors](#ironstar-2nt-3-implement-value-objects-and-smart-constructors)
- [⚫ ironstar-2nt.2 Define algebraic domain types and aggregate structure](#ironstar-2nt-2-define-algebraic-domain-types-and-aggregate-structure)
- [⚫ ironstar-2nt.1 Initialize src/ directory structure with modular organization](#ironstar-2nt-1-initialize-src-directory-structure-with-modular-organization)
- [⚫ ironstar-2nt Domain layer](#ironstar-2nt-domain-layer)
- [⚫ ironstar-6lq.7 Add Rust to CI matrix and extend inherited workflows](#ironstar-6lq-7-add-rust-to-ci-matrix-and-extend-inherited-workflows)
- [⚫ ironstar-6lq.6 Add Rust checks to flake.checks for CI integration](#ironstar-6lq-6-add-rust-checks-to-flake-checks-for-ci-integration)
- [⚫ ironstar-6lq.5 Verify cargo check passes with workspace configuration](#ironstar-6lq-5-verify-cargo-check-passes-with-workspace-configuration)
- [⚫ ironstar-6lq.4 Set up per-crate crate.nix pattern for crane args](#ironstar-6lq-4-set-up-per-crate-crate-nix-pattern-for-crane-args)
- [⚫ ironstar-6lq.3 Configure Cargo.toml with workspace structure (resolver = 2)](#ironstar-6lq-3-configure-cargo-toml-with-workspace-structure-resolver-2)
- [⚫ ironstar-6lq.2 Add rust-toolchain.toml with required components](#ironstar-6lq-2-add-rust-toolchain-toml-with-required-components)
- [⚫ ironstar-6lq.1 Integrate rust-flake patterns (crane, rust-overlay)](#ironstar-6lq-1-integrate-rust-flake-patterns-crane-rust-overlay)
- [⚫ ironstar-6lq Rust workspace integration](#ironstar-6lq-rust-workspace-integration)
- [⚫ ironstar-cxe.5 Create .gitignore with comprehensive patterns](#ironstar-cxe-5-create-gitignore-with-comprehensive-patterns)
- [⚫ ironstar-cxe.4 Create initial git commit with generated structure](#ironstar-cxe-4-create-initial-git-commit-with-generated-structure)
- [⚫ ironstar-cxe.3 Verify nix develop enters working development shell](#ironstar-cxe-3-verify-nix-develop-enters-working-development-shell)
- [⚫ ironstar-cxe.2 Configure secrets management and string replacement](#ironstar-cxe-2-configure-secrets-management-and-string-replacement)
- [⚫ ironstar-cxe.1 Run om init with typescript-nix-template parameters](#ironstar-cxe-1-run-om-init-with-typescript-nix-template-parameters)
- [⚫ ironstar-cxe Template instantiation](#ironstar-cxe-template-instantiation)
- [⚫ ironstar-o66.6 Remove rust-flake and add per-crate crane derivations](#ironstar-o66-6-remove-rust-flake-and-add-per-crate-crane-derivations)
- [⚫ ironstar-o66.3 Skip per-crate doc derivation builds in CI](#ironstar-o66-3-skip-per-crate-doc-derivation-builds-in-ci)
- [⚫ ironstar-o66.2 Remove redundant per-crate clippy checks](#ironstar-o66-2-remove-redundant-per-crate-clippy-checks)
- [⚫ ironstar-o66.1 Audit rust-flake autoWire settings per library crate](#ironstar-o66-1-audit-rust-flake-autowire-settings-per-library-crate)
- [⚫ ironstar-o66 Optimize CI build derivation set for multi-crate workspace](#ironstar-o66-optimize-ci-build-derivation-set-for-multi-crate-workspace)
- [⚫ ironstar-29b.12 Configure workspace and Nix integration](#ironstar-29b-12-configure-workspace-and-nix-integration)
- [⚫ ironstar-29b.11 Collapse monolith to binary crate](#ironstar-29b-11-collapse-monolith-to-binary-crate)
- [⚫ ironstar-29b.10 Extract ironstar-session-store crate](#ironstar-29b-10-extract-ironstar-session-store-crate)
- [⚫ ironstar-29b.9 Extract ironstar-analytics-infra crate](#ironstar-29b-9-extract-ironstar-analytics-infra-crate)
- [⚫ ironstar-29b.8 Extract ironstar-event-bus crate](#ironstar-29b-8-extract-ironstar-event-bus-crate)
- [⚫ ironstar-29b.7 Extract ironstar-event-store crate](#ironstar-29b-7-extract-ironstar-event-store-crate)
- [⚫ ironstar-29b.6 Extract ironstar-workspace crate](#ironstar-29b-6-extract-ironstar-workspace-crate)
- [⚫ ironstar-29b.5 Extract ironstar-analytics crate](#ironstar-29b-5-extract-ironstar-analytics-crate)
- [⚫ ironstar-29b.4 Extract ironstar-session crate](#ironstar-29b-4-extract-ironstar-session-crate)
- [⚫ ironstar-29b.3 Extract ironstar-todo crate](#ironstar-29b-3-extract-ironstar-todo-crate)
- [⚫ ironstar-29b.2 Extract ironstar-shared-kernel crate](#ironstar-29b-2-extract-ironstar-shared-kernel-crate)
- [⚫ ironstar-29b.1 Extract ironstar-core crate](#ironstar-29b-1-extract-ironstar-core-crate)
- [⚫ ironstar-29b Spec-aligned crate decomposition](#ironstar-29b-spec-aligned-crate-decomposition)
- [⚫ ironstar-nyp.49 Implement combined analyticsDecider composition](#ironstar-nyp-49-implement-combined-analyticsdecider-composition)
- [⚫ ironstar-nyp.47 Implement Analytics query handlers (read-side)](#ironstar-nyp-47-implement-analytics-query-handlers-read-side)
- [⚫ ironstar-nyp.46 Implement Catalog command handler with Zenoh publishing](#ironstar-nyp-46-implement-catalog-command-handler-with-zenoh-publishing)
- [⚫ ironstar-nyp.42 Implement Catalog View read model projection](#ironstar-nyp-42-implement-catalog-view-read-model-projection)
- [⚫ ironstar-ny3.20 Fix CSS bundle not appearing in manifest.json](#ironstar-ny3-20-fix-css-bundle-not-appearing-in-manifest-json)
- [⚫ ironstar-507 Session aggregate implementation](#ironstar-507-session-aggregate-implementation)
- [⚫ ironstar-7a2.2 Implement Workspace aggregate with visibility control](#ironstar-7a2-2-implement-workspace-aggregate-with-visibility-control)
- [⚫ ironstar-7a2.1 Implement User and UserId types with composite key pattern](#ironstar-7a2-1-implement-user-and-userid-types-with-composite-key-pattern)
- [⚫ ironstar-7a2 Implement Workspace bounded context aggregates in Rust](#ironstar-7a2-implement-workspace-bounded-context-aggregates-in-rust)
- [⚫ ironstar-2it.21 Integrate Idris2 spec modules and verify type-checking](#ironstar-2it-21-integrate-idris2-spec-modules-and-verify-type-checking)
- [⚫ ironstar-2it.18 Formalize Analytics bounded context in Idris2 (Core domain)](#ironstar-2it-18-formalize-analytics-bounded-context-in-idris2-core-domain)
- [⚫ ironstar-2it.17 Create Idris2 spec infrastructure and core abstractions](#ironstar-2it-17-create-idris2-spec-infrastructure-and-core-abstractions)
- [⚫ ironstar-2it.16 Co-refine D2 diagrams and Idris2 spec (iterative)](#ironstar-2it-16-co-refine-d2-diagrams-and-idris2-spec-iterative)
- [⚫ ironstar-2it.15 Cross-reference fmodel-rust patterns for Decider implementation](#ironstar-2it-15-cross-reference-fmodel-rust-patterns-for-decider-implementation)
- [⚫ ironstar-2it.14 Cross-reference northstar patterns for Datastar SSE architecture](#ironstar-2it-14-cross-reference-northstar-patterns-for-datastar-sse-architecture)
- [⚫ ironstar-2it.13 Review bounded context definitions against architecture docs](#ironstar-2it-13-review-bounded-context-definitions-against-architecture-docs)
- [⚫ ironstar-2it.11 Cross-reference and validate EventCatalog artifacts (Phase 9)](#ironstar-2it-11-cross-reference-and-validate-eventcatalog-artifacts-phase-9)
- [⚫ ironstar-2it.10 Transform Qlerify JSON to EventCatalog MDX (Phase 3: Entities and Flows)](#ironstar-2it-10-transform-qlerify-json-to-eventcatalog-mdx-phase-3-entities-and-flows)
- [⚫ ironstar-2it.9 Transform Qlerify JSON to EventCatalog MDX (Phase 2: Events, Commands, Queries)](#ironstar-2it-9-transform-qlerify-json-to-eventcatalog-mdx-phase-2-events-commands-queries)
- [⚫ ironstar-2it.8 Transform Qlerify JSON to EventCatalog MDX (Phase 1: Domain and Services)](#ironstar-2it-8-transform-qlerify-json-to-eventcatalog-mdx-phase-1-domain-and-services)
- [⚫ ironstar-2it.7 Instantiate EventCatalog infrastructure](#ironstar-2it-7-instantiate-eventcatalog-infrastructure)
- [⚫ ironstar-2it.6 Refine D2 diagrams to match Qlerify exports](#ironstar-2it-6-refine-d2-diagrams-to-match-qlerify-exports)
- [⚫ ironstar-2it.5 Export Qlerify JSON and validate structure](#ironstar-2it-5-export-qlerify-json-and-validate-structure)
- [⚫ ironstar-2it.4 Elaborate GWT scenarios and prioritize releases (Step 7)](#ironstar-2it-4-elaborate-gwt-scenarios-and-prioritize-releases-step-7)
- [⚫ ironstar-2it.3 Apply Conway's Law and assign bounded contexts (Step 6)](#ironstar-2it-3-apply-conway-s-law-and-assign-bounded-contexts-step-6)
- [⚫ ironstar-2it.2 Execute Qlerify sessions and refine generated models](#ironstar-2it-2-execute-qlerify-sessions-and-refine-generated-models)
- [⚫ ironstar-2it.1 Generate Qlerify prompts for all bounded contexts](#ironstar-2it-1-generate-qlerify-prompts-for-all-bounded-contexts)
- [⚫ ironstar-2it Complete Event Modeling and EventCatalog documentation](#ironstar-2it-complete-event-modeling-and-eventcatalog-documentation)
- [⚫ ironstar-ny3.18 Add CUBE CSS composition layer](#ironstar-ny3-18-add-cube-css-composition-layer)
- [⚫ ironstar-a9b.13 Implement View specification tests](#ironstar-a9b-13-implement-view-specification-tests)
- [⚫ ironstar-a9b.11 Implement Todo query handler](#ironstar-a9b-11-implement-todo-query-handler)
- [⚫ ironstar-a9b.10 Implement Todo command handler](#ironstar-a9b-10-implement-todo-command-handler)
- [⚫ ironstar-a9b.8 Implement Todo query service with View replay](#ironstar-a9b-8-implement-todo-query-service-with-view-replay)
- [⚫ ironstar-a9b.6 Implement QuerySession Decider](#ironstar-a9b-6-implement-querysession-decider)
- [⚫ ironstar-a9b.5 Implement Todo View](#ironstar-a9b-5-implement-todo-view)
- [⚫ ironstar-a9b Implement fmodel-rust event sourcing foundation](#ironstar-a9b-implement-fmodel-rust-event-sourcing-foundation)
- [⚫ ironstar-3gd.4 Implement embedded DuckLake catalog pattern with rust_embed](#ironstar-3gd-4-implement-embedded-ducklake-catalog-pattern-with-rust-embed)
- [⚫ ironstar-3gd.3 Implement CacheDependency struct for Zenoh-based cache invalidation](#ironstar-3gd-3-implement-cachedependency-struct-for-zenoh-based-cache-invalidation)
- [⚫ ironstar-nyp.31 Implement health check endpoints (/health, /health/ready, /health/live)](#ironstar-nyp-31-implement-health-check-endpoints-health-health-ready-health-live)
- [⚫ ironstar-nyp.27 Implement ZenohEventBus struct with publish/subscribe methods](#ironstar-nyp-27-implement-zenoheventbus-struct-with-publish-subscribe-methods)
- [⚫ ironstar-nyp.25 Define Zenoh key expression patterns for event routing](#ironstar-nyp-25-define-zenoh-key-expression-patterns-for-event-routing)
- [⚫ ironstar-2nt.10 Define ErrorCode enum for HTTP error mapping](#ironstar-2nt-10-define-errorcode-enum-for-http-error-mapping)
- [⚫ ironstar-nyp.19 Create EventBus trait abstraction](#ironstar-nyp-19-create-eventbus-trait-abstraction)
- [⚫ ironstar-nyp.15 Implement moka analytics cache with rkyv serialization](#ironstar-nyp-15-implement-moka-analytics-cache-with-rkyv-serialization)
- [⚫ ironstar-edx Review narrative arc and timing estimates](#ironstar-edx-review-narrative-arc-and-timing-estimates)
- [⚫ ironstar-0tk Omicslake presentation slide deck](#ironstar-0tk-omicslake-presentation-slide-deck)
- [⚫ ironstar-e6k.8 Implement todo example route mounting](#ironstar-e6k-8-implement-todo-example-route-mounting)
- [⚫ ironstar-e6k.7 Implement todo_list_template rendering function](#ironstar-e6k-7-implement-todo-list-template-rendering-function)
- [⚫ ironstar-e6k.6 Implement GET /todos SSE feed endpoint](#ironstar-e6k-6-implement-get-todos-sse-feed-endpoint)
- [⚫ ironstar-e6k.5 Implement delete_todo handler (POST /delete-todo)](#ironstar-e6k-5-implement-delete-todo-handler-post-delete-todo)
- [⚫ ironstar-e6k.4 Implement mark_todo handler (POST /mark-todo)](#ironstar-e6k-4-implement-mark-todo-handler-post-mark-todo)
- [⚫ ironstar-e6k.3 Implement add_todo handler (POST /add-todo)](#ironstar-e6k-3-implement-add-todo-handler-post-add-todo)
- [⚫ ironstar-e6k.2 Implement TodoListProjection with in-memory rebuild](#ironstar-e6k-2-implement-todolistprojection-with-in-memory-rebuild)
- [⚫ ironstar-e6k.1 Define Todo domain model (aggregate, events, commands)](#ironstar-e6k-1-define-todo-domain-model-aggregate-events-commands)
- [⚫ ironstar-r62.13 Wire all components together in main.rs](#ironstar-r62-13-wire-all-components-together-in-main-rs)
- [⚫ ironstar-r62.10 Implement component-level hypertext templates](#ironstar-r62-10-implement-component-level-hypertext-templates)
- [⚫ ironstar-nyp.11 Create Session axum extractor](#ironstar-nyp-11-create-session-axum-extractor)
- [⚫ ironstar-nyp.10 Add session TTL cleanup background task](#ironstar-nyp-10-add-session-ttl-cleanup-background-task)
- [⚫ ironstar-nyp.9 Implement SQLite session store with SessionStore trait](#ironstar-nyp-9-implement-sqlite-session-store-with-sessionstore-trait)
- [⚫ ironstar-nyp.7 Implement ProjectionManager with in-memory state](#ironstar-nyp-7-implement-projectionmanager-with-in-memory-state)
- [⚫ ironstar-nyp.3 Implement SQLite EventRepository with sqlx](#ironstar-nyp-3-implement-sqlite-eventrepository-with-sqlx)
- [⚫ ironstar-ny3.14 Create web-components/components/ directory for vanilla web components](#ironstar-ny3-14-create-web-components-components-directory-for-vanilla-web-components)
- [⚫ ironstar-2nt.8 Define application error types](#ironstar-2nt-8-define-application-error-types)
- [⚫ ironstar-o66.4 Remove ironstar-release from CI packages build](#ironstar-o66-4-remove-ironstar-release-from-ci-packages-build)
- [⚫ ironstar-507.3 Implement Session Decider specification tests](#ironstar-507-3-implement-session-decider-specification-tests)
- [⚫ ironstar-7a2.14 Implement Workspace View specification tests](#ironstar-7a2-14-implement-workspace-view-specification-tests)
- [⚫ ironstar-7a2.13 Implement Workspace Decider specification tests](#ironstar-7a2-13-implement-workspace-decider-specification-tests)
- [⚫ ironstar-7a2.12 Implement Workspace query handlers](#ironstar-7a2-12-implement-workspace-query-handlers)
- [⚫ ironstar-7a2.11 Implement Workspace command handlers](#ironstar-7a2-11-implement-workspace-command-handlers)
- [⚫ ironstar-7a2.10 Integrate Zenoh event publishing for Workspace](#ironstar-7a2-10-integrate-zenoh-event-publishing-for-workspace)
- [⚫ ironstar-7a2.9 Wire Workspace MaterializedView projections](#ironstar-7a2-9-wire-workspace-materializedview-projections)
- [⚫ ironstar-7a2.8 Wire Workspace EventSourcedAggregate to SQLite](#ironstar-7a2-8-wire-workspace-eventsourcedaggregate-to-sqlite)
- [⚫ ironstar-1ks Wire Workspace EventSourcedAggregate to SQLite](#ironstar-1ks-wire-workspace-eventsourcedaggregate-to-sqlite)
- [⚫ ironstar-7a2.7 Implement workspaceContextDecider composition](#ironstar-7a2-7-implement-workspacecontextdecider-composition)
- [⚫ ironstar-7a2.6 Implement UserPreferences aggregate (user-scoped only)](#ironstar-7a2-6-implement-userpreferences-aggregate-user-scoped-only)
- [⚫ ironstar-7a2.5 Implement SavedQuery aggregate with workspaceId scope](#ironstar-7a2-5-implement-savedquery-aggregate-with-workspaceid-scope)
- [⚫ ironstar-7a2.4 Implement Dashboard aggregate with workspaceId scope](#ironstar-7a2-4-implement-dashboard-aggregate-with-workspaceid-scope)
- [⚫ ironstar-7a2.3 Implement WorkspacePreferences aggregate](#ironstar-7a2-3-implement-workspacepreferences-aggregate)
- [⚫ ironstar-2nt.19 Add smart constructors for refined types](#ironstar-2nt-19-add-smart-constructors-for-refined-types)
- [⚫ ironstar-2nt.18 Add missing enums to Idris specifications](#ironstar-2nt-18-add-missing-enums-to-idris-specifications)
- [⚫ ironstar-2it.24 Generate EventCatalog entity documentation for value objects](#ironstar-2it-24-generate-eventcatalog-entity-documentation-for-value-objects)
- [⚫ ironstar-2it.22 Formalize Workspace bounded context in Idris2 (Supporting domain)](#ironstar-2it-22-formalize-workspace-bounded-context-in-idris2-supporting-domain)
- [⚫ ironstar-2it.19 Formalize Session bounded context in Idris2 (Supporting domain)](#ironstar-2it-19-formalize-session-bounded-context-in-idris2-supporting-domain)
- [⚫ ironstar-2it.12 Document fmodel-rust Decider mapping in EventCatalog](#ironstar-2it-12-document-fmodel-rust-decider-mapping-in-eventcatalog)
- [⚫ ironstar-e8d Refactor domain module into aggregate-based subdirectories](#ironstar-e8d-refactor-domain-module-into-aggregate-based-subdirectories)
- [⚫ ironstar-nyp.33 Implement session cleanup background task](#ironstar-nyp-33-implement-session-cleanup-background-task)
- [⚫ ironstar-3gd.2 Implement event-driven cache invalidation](#ironstar-3gd-2-implement-event-driven-cache-invalidation)
- [⚫ ironstar-3gd.1 Implement cache-aside pattern for DuckDB analytics](#ironstar-3gd-1-implement-cache-aside-pattern-for-duckdb-analytics)
- [⚫ ironstar-6lq.9 Add workspace lint configuration to Cargo.toml](#ironstar-6lq-9-add-workspace-lint-configuration-to-cargo-toml)
- [⚫ ironstar-ny3.15 Configure Rolldown for Lit web component bundling](#ironstar-ny3-15-configure-rolldown-for-lit-web-component-bundling)
- [⚫ ironstar-89k Integrate analytics cache with dashboard SSE streams](#ironstar-89k-integrate-analytics-cache-with-dashboard-sse-streams)
- [⚫ ironstar-9oj Implement cache invalidation for analytics queries](#ironstar-9oj-implement-cache-invalidation-for-analytics-queries)
- [⚫ ironstar-nqq.1 Implement CQRS performance tuning](#ironstar-nqq-1-implement-cqrs-performance-tuning)
- [⚫ ironstar-nqq Performance optimization](#ironstar-nqq-performance-optimization)
- [⚫ ironstar-avp Verify code examples compile and run](#ironstar-avp-verify-code-examples-compile-and-run)
- [⚫ ironstar-ym1 Polish diagrams for visual consistency](#ironstar-ym1-polish-diagrams-for-visual-consistency)
- [⚫ ironstar-63r Verify technical accuracy of benchmarks](#ironstar-63r-verify-technical-accuracy-of-benchmarks)
- [⚫ ironstar-z4s Act 4: Expand vision slides](#ironstar-z4s-act-4-expand-vision-slides)
- [⚫ ironstar-b8d Act 3: Expand web interface slides](#ironstar-b8d-act-3-expand-web-interface-slides)
- [⚫ ironstar-a15 Act 2: Expand solution stack slides](#ironstar-a15-act-2-expand-solution-stack-slides)
- [⚫ ironstar-ubj Act 1: Expand data problem slides](#ironstar-ubj-act-1-expand-data-problem-slides)
- [⚫ ironstar-r5f ironstar-6lq](#ironstar-r5f-ironstar-6lq)
- [⚫ ironstar-6lq.8 Create reusable Rust CI workflow with workflow_call dispatch](#ironstar-6lq-8-create-reusable-rust-ci-workflow-with-workflow-call-dispatch)
- [⚫ ironstar-r62.12 Implement graceful shutdown signal handling](#ironstar-r62-12-implement-graceful-shutdown-signal-handling)
- [⚫ ironstar-r62.11 Implement router composition with feature routes](#ironstar-r62-11-implement-router-composition-with-feature-routes)
- [⚫ ironstar-r62.6 Implement command POST handlers](#ironstar-r62-6-implement-command-post-handlers)
- [⚫ ironstar-r62.5 Implement SSE feed endpoint with event replay](#ironstar-r62-5-implement-sse-feed-endpoint-with-event-replay)
- [⚫ ironstar-r62.4 Define AppState struct with all dependencies](#ironstar-r62-4-define-appstate-struct-with-all-dependencies)
- [⚫ ironstar-2it.20 Formalize Todo bounded context in Idris2 (Generic Example)](#ironstar-2it-20-formalize-todo-bounded-context-in-idris2-generic-example)
- [⚫ ironstar-nyp.16 Implement DualEventBus for tokio::broadcast to Zenoh migration](#ironstar-nyp-16-implement-dualeventbus-for-tokio-broadcast-to-zenoh-migration)
- [⚫ ironstar-nqq.2 Implement advanced performance patterns](#ironstar-nqq-2-implement-advanced-performance-patterns)
- [⚫ ironstar-k1z Final review and presentation dry-run](#ironstar-k1z-final-review-and-presentation-dry-run)
- [⚫ ironstar-e6k Example application (Todo)](#ironstar-e6k-example-application-todo)
- [⚫ ironstar-v4y.3 Define common-utils crate structure](#ironstar-v4y-3-define-common-utils-crate-structure)
- [⚫ ironstar-v4y.2 Define common-types crate structure](#ironstar-v4y-2-define-common-types-crate-structure)
- [⚫ ironstar-v4y.1 Define common-enums crate structure](#ironstar-v4y-1-define-common-enums-crate-structure)
- [⚫ ironstar-v4y Multi-crate workspace decomposition](#ironstar-v4y-multi-crate-workspace-decomposition)

---

## Dependency Graph

```mermaid
graph TD
    classDef open fill:#50FA7B,stroke:#333,color:#000
    classDef inprogress fill:#8BE9FD,stroke:#333,color:#000
    classDef blocked fill:#FF5555,stroke:#333,color:#000
    classDef closed fill:#6272A4,stroke:#333,color:#fff

    ironstar-09r["ironstar-09r<br/>Implement ds-echarts Lit web componen..."]
    class ironstar-09r closed
    ironstar-0ha["ironstar-0ha<br/>Document SSE projection function sema..."]
    class ironstar-0ha open
    ironstar-0tk["ironstar-0tk<br/>Omicslake presentation slide deck"]
    class ironstar-0tk closed
    ironstar-1ks["ironstar-1ks<br/>Wire Workspace EventSourcedAggregate ..."]
    class ironstar-1ks closed
    ironstar-29b["ironstar-29b<br/>Spec-aligned crate decomposition"]
    class ironstar-29b closed
    ironstar-29b1["ironstar-29b.1<br/>Extract ironstar-core crate"]
    class ironstar-29b1 closed
    ironstar-29b10["ironstar-29b.10<br/>Extract ironstar-session-store crate"]
    class ironstar-29b10 closed
    ironstar-29b11["ironstar-29b.11<br/>Collapse monolith to binary crate"]
    class ironstar-29b11 closed
    ironstar-29b12["ironstar-29b.12<br/>Configure workspace and Nix integration"]
    class ironstar-29b12 closed
    ironstar-29b2["ironstar-29b.2<br/>Extract ironstar-shared-kernel crate"]
    class ironstar-29b2 closed
    ironstar-29b3["ironstar-29b.3<br/>Extract ironstar-todo crate"]
    class ironstar-29b3 closed
    ironstar-29b4["ironstar-29b.4<br/>Extract ironstar-session crate"]
    class ironstar-29b4 closed
    ironstar-29b5["ironstar-29b.5<br/>Extract ironstar-analytics crate"]
    class ironstar-29b5 closed
    ironstar-29b6["ironstar-29b.6<br/>Extract ironstar-workspace crate"]
    class ironstar-29b6 closed
    ironstar-29b7["ironstar-29b.7<br/>Extract ironstar-event-store crate"]
    class ironstar-29b7 closed
    ironstar-29b8["ironstar-29b.8<br/>Extract ironstar-event-bus crate"]
    class ironstar-29b8 closed
    ironstar-29b9["ironstar-29b.9<br/>Extract ironstar-analytics-infra crate"]
    class ironstar-29b9 closed
    ironstar-2it["ironstar-2it<br/>Complete Event Modeling and EventCata..."]
    class ironstar-2it closed
    ironstar-2it1["ironstar-2it.1<br/>Generate Qlerify prompts for all boun..."]
    class ironstar-2it1 closed
    ironstar-2it10["ironstar-2it.10<br/>Transform Qlerify JSON to EventCatalo..."]
    class ironstar-2it10 closed
    ironstar-2it11["ironstar-2it.11<br/>Cross-reference and validate EventCat..."]
    class ironstar-2it11 closed
    ironstar-2it12["ironstar-2it.12<br/>Document fmodel-rust Decider mapping ..."]
    class ironstar-2it12 closed
    ironstar-2it13["ironstar-2it.13<br/>Review bounded context definitions ag..."]
    class ironstar-2it13 closed
    ironstar-2it14["ironstar-2it.14<br/>Cross-reference northstar patterns fo..."]
    class ironstar-2it14 closed
    ironstar-2it15["ironstar-2it.15<br/>Cross-reference fmodel-rust patterns ..."]
    class ironstar-2it15 closed
    ironstar-2it16["ironstar-2it.16<br/>Co-refine D2 diagrams and Idris2 spec..."]
    class ironstar-2it16 closed
    ironstar-2it17["ironstar-2it.17<br/>Create Idris2 spec infrastructure and..."]
    class ironstar-2it17 closed
    ironstar-2it18["ironstar-2it.18<br/>Formalize Analytics bounded context i..."]
    class ironstar-2it18 closed
    ironstar-2it19["ironstar-2it.19<br/>Formalize Session bounded context in ..."]
    class ironstar-2it19 closed
    ironstar-2it2["ironstar-2it.2<br/>Execute Qlerify sessions and refine g..."]
    class ironstar-2it2 closed
    ironstar-2it20["ironstar-2it.20<br/>Formalize Todo bounded context in Idr..."]
    class ironstar-2it20 closed
    ironstar-2it21["ironstar-2it.21<br/>Integrate Idris2 spec modules and ver..."]
    class ironstar-2it21 closed
    ironstar-2it22["ironstar-2it.22<br/>Formalize Workspace bounded context i..."]
    class ironstar-2it22 closed
    ironstar-2it23["ironstar-2it.23<br/>Resolve UserId identity model: compos..."]
    class ironstar-2it23 closed
    ironstar-2it24["ironstar-2it.24<br/>Generate EventCatalog entity document..."]
    class ironstar-2it24 closed
    ironstar-2it3["ironstar-2it.3<br/>Apply Conway's Law and assign bounded..."]
    class ironstar-2it3 closed
    ironstar-2it4["ironstar-2it.4<br/>Elaborate GWT scenarios and prioritiz..."]
    class ironstar-2it4 closed
    ironstar-2it5["ironstar-2it.5<br/>Export Qlerify JSON and validate stru..."]
    class ironstar-2it5 closed
    ironstar-2it6["ironstar-2it.6<br/>Refine D2 diagrams to match Qlerify e..."]
    class ironstar-2it6 closed
    ironstar-2it7["ironstar-2it.7<br/>Instantiate EventCatalog infrastructure"]
    class ironstar-2it7 closed
    ironstar-2it8["ironstar-2it.8<br/>Transform Qlerify JSON to EventCatalo..."]
    class ironstar-2it8 closed
    ironstar-2it9["ironstar-2it.9<br/>Transform Qlerify JSON to EventCatalo..."]
    class ironstar-2it9 closed
    ironstar-2nt["ironstar-2nt<br/>Domain layer"]
    class ironstar-2nt closed
    ironstar-2nt1["ironstar-2nt.1<br/>Initialize src/ directory structure w..."]
    class ironstar-2nt1 closed
    ironstar-2nt10["ironstar-2nt.10<br/>Define ErrorCode enum for HTTP error ..."]
    class ironstar-2nt10 closed
    ironstar-2nt11["ironstar-2nt.11<br/>Add version(&self) -&gt; u64 to Aggre..."]
    class ironstar-2nt11 closed
    ironstar-2nt12["ironstar-2nt.12<br/>Implement UUID-tracked error type for..."]
    class ironstar-2nt12 closed
    ironstar-2nt13["ironstar-2nt.13<br/>Enforce async/sync boundary via modul..."]
    class ironstar-2nt13 closed
    ironstar-2nt14["ironstar-2nt.14<br/>Define QuerySession aggregate with ty..."]
    class ironstar-2nt14 closed
    ironstar-2nt15["ironstar-2nt.15<br/>Define analytics value objects (Datas..."]
    class ironstar-2nt15 closed
    ironstar-2nt16["ironstar-2nt.16<br/>Define analytics workflow as pure fun..."]
    class ironstar-2nt16 closed
    ironstar-2nt17["ironstar-2nt.17<br/>Implement AnalyticsError with UUID co..."]
    class ironstar-2nt17 closed
    ironstar-2nt18["ironstar-2nt.18<br/>Add missing enums to Idris specificat..."]
    class ironstar-2nt18 closed
    ironstar-2nt19["ironstar-2nt.19<br/>Add smart constructors for refined types"]
    class ironstar-2nt19 closed
    ironstar-2nt2["ironstar-2nt.2<br/>Define algebraic domain types and agg..."]
    class ironstar-2nt2 closed
    ironstar-2nt3["ironstar-2nt.3<br/>Implement value objects and smart con..."]
    class ironstar-2nt3 closed
    ironstar-2nt4["ironstar-2nt.4<br/>Design aggregate root state machines"]
    class ironstar-2nt4 closed
    ironstar-2nt5["ironstar-2nt.5<br/>Create Datastar signal types with ts-..."]
    class ironstar-2nt5 closed
    ironstar-2nt6["ironstar-2nt.6<br/>Enforce camelCase convention for Data..."]
    class ironstar-2nt6 closed
    ironstar-2nt7["ironstar-2nt.7<br/>Implement command validation pattern ..."]
    class ironstar-2nt7 closed
    ironstar-2nt8["ironstar-2nt.8<br/>Define application error types"]
    class ironstar-2nt8 closed
    ironstar-2nt9["ironstar-2nt.9<br/>Define ChartSignals and ChartSelectio..."]
    class ironstar-2nt9 closed
    ironstar-2vp["ironstar-2vp<br/>Test bitemporal semantics and SSE rec..."]
    class ironstar-2vp open
    ironstar-3gd["ironstar-3gd<br/>Scientific Data Integration"]
    class ironstar-3gd closed
    ironstar-3gd1["ironstar-3gd.1<br/>Implement cache-aside pattern for Duc..."]
    class ironstar-3gd1 closed
    ironstar-3gd2["ironstar-3gd.2<br/>Implement event-driven cache invalida..."]
    class ironstar-3gd2 closed
    ironstar-3gd3["ironstar-3gd.3<br/>Implement CacheDependency struct for ..."]
    class ironstar-3gd3 closed
    ironstar-3gd4["ironstar-3gd.4<br/>Implement embedded DuckLake catalog p..."]
    class ironstar-3gd4 closed
    ironstar-507["ironstar-507<br/>Session aggregate implementation"]
    class ironstar-507 closed
    ironstar-5071["ironstar-507.1<br/>Wire Session EventSourcedAggregate to..."]
    class ironstar-5071 open
    ironstar-5072["ironstar-507.2<br/>Wire ActiveSessionView projection"]
    class ironstar-5072 open
    ironstar-5073["ironstar-507.3<br/>Implement Session Decider specificati..."]
    class ironstar-5073 closed
    ironstar-5074["ironstar-507.4<br/>Integrate Zenoh event publishing for ..."]
    class ironstar-5074 open
    ironstar-53t["ironstar-53t<br/>Reference: Hoffman's Laws compliance ..."]
    class ironstar-53t open
    ironstar-58f["ironstar-58f<br/>Implement ViewStateRepository for SQLite"]
    class ironstar-58f open
    ironstar-63r["ironstar-63r<br/>Verify technical accuracy of benchmarks"]
    class ironstar-63r closed
    ironstar-6lq["ironstar-6lq<br/>Rust workspace integration"]
    class ironstar-6lq closed
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
    class ironstar-6lq7 closed
    ironstar-6lq8["ironstar-6lq.8<br/>Create reusable Rust CI workflow with..."]
    class ironstar-6lq8 closed
    ironstar-6lq9["ironstar-6lq.9<br/>Add workspace lint configuration to C..."]
    class ironstar-6lq9 closed
    ironstar-72q["ironstar-72q<br/>Verify Zenoh key filtering preserves ..."]
    class ironstar-72q open
    ironstar-753["ironstar-753<br/>Third-party library integration"]
    class ironstar-753 open
    ironstar-7531["ironstar-753.1<br/>Implement VegaChart web component wra..."]
    class ironstar-7531 open
    ironstar-7532["ironstar-753.2<br/>Implement sortable-list web component..."]
    class ironstar-7532 open
    ironstar-7533["ironstar-753.3<br/>Set up Lucide icon build-time inlining"]
    class ironstar-7533 open
    ironstar-7534["ironstar-753.4<br/>Implement ds-echarts backend support"]
    class ironstar-7534 closed
    ironstar-7535["ironstar-753.5<br/>Implement ds-echarts build and test i..."]
    class ironstar-7535 closed
    ironstar-7536["ironstar-753.6<br/>Implement chart SSE endpoint with sig..."]
    class ironstar-7536 closed
    ironstar-7537["ironstar-753.7<br/>Document web components as coalgebras..."]
    class ironstar-7537 open
    ironstar-7a2["ironstar-7a2<br/>Implement Workspace bounded context a..."]
    class ironstar-7a2 closed
    ironstar-7a21["ironstar-7a2.1<br/>Implement User and UserId types with ..."]
    class ironstar-7a21 closed
    ironstar-7a210["ironstar-7a2.10<br/>Integrate Zenoh event publishing for ..."]
    class ironstar-7a210 closed
    ironstar-7a211["ironstar-7a2.11<br/>Implement Workspace command handlers"]
    class ironstar-7a211 closed
    ironstar-7a212["ironstar-7a2.12<br/>Implement Workspace query handlers"]
    class ironstar-7a212 closed
    ironstar-7a213["ironstar-7a2.13<br/>Implement Workspace Decider specifica..."]
    class ironstar-7a213 closed
    ironstar-7a214["ironstar-7a2.14<br/>Implement Workspace View specificatio..."]
    class ironstar-7a214 closed
    ironstar-7a22["ironstar-7a2.2<br/>Implement Workspace aggregate with vi..."]
    class ironstar-7a22 closed
    ironstar-7a23["ironstar-7a2.3<br/>Implement WorkspacePreferences aggregate"]
    class ironstar-7a23 closed
    ironstar-7a24["ironstar-7a2.4<br/>Implement Dashboard aggregate with wo..."]
    class ironstar-7a24 closed
    ironstar-7a25["ironstar-7a2.5<br/>Implement SavedQuery aggregate with w..."]
    class ironstar-7a25 closed
    ironstar-7a26["ironstar-7a2.6<br/>Implement UserPreferences aggregate (..."]
    class ironstar-7a26 closed
    ironstar-7a27["ironstar-7a2.7<br/>Implement workspaceContextDecider com..."]
    class ironstar-7a27 closed
    ironstar-7a28["ironstar-7a2.8<br/>Wire Workspace EventSourcedAggregate ..."]
    class ironstar-7a28 closed
    ironstar-7a29["ironstar-7a2.9<br/>Wire Workspace MaterializedView proje..."]
    class ironstar-7a29 closed
    ironstar-89k["ironstar-89k<br/>Integrate analytics cache with dashbo..."]
    class ironstar-89k closed
    ironstar-961["ironstar-961<br/>Implement DuckDB connection lifecycle..."]
    class ironstar-961 closed
    ironstar-9b1["ironstar-9b1<br/>Implement httpfs extension configurat..."]
    class ironstar-9b1 closed
    ironstar-9dh["ironstar-9dh<br/>Reference: Bounded context patterns"]
    class ironstar-9dh open
    ironstar-9oj["ironstar-9oj<br/>Implement cache invalidation for anal..."]
    class ironstar-9oj closed
    ironstar-a15["ironstar-a15<br/>Act 2: Expand solution stack slides"]
    class ironstar-a15 closed
    ironstar-a1s["ironstar-a1s<br/>Verify catamorphism uniqueness with p..."]
    class ironstar-a1s open
    ironstar-a9b["ironstar-a9b<br/>Implement fmodel-rust event sourcing ..."]
    class ironstar-a9b closed
    ironstar-a9b1["ironstar-a9b.1<br/>Implement SQLite EventRepository"]
    class ironstar-a9b1 closed
    ironstar-a9b10["ironstar-a9b.10<br/>Implement Todo command handler"]
    class ironstar-a9b10 closed
    ironstar-a9b11["ironstar-a9b.11<br/>Implement Todo query handler"]
    class ironstar-a9b11 closed
    ironstar-a9b12["ironstar-a9b.12<br/>Implement Decider specification tests"]
    class ironstar-a9b12 closed
    ironstar-a9b13["ironstar-a9b.13<br/>Implement View specification tests"]
    class ironstar-a9b13 closed
    ironstar-a9b14["ironstar-a9b.14<br/>Document CommandPipelineError in arch..."]
    class ironstar-a9b14 open
    ironstar-a9b2["ironstar-a9b.2<br/>Implement fmodel-rust identifier traits"]
    class ironstar-a9b2 closed
    ironstar-a9b3["ironstar-a9b.3<br/>Create event store SQLite schema"]
    class ironstar-a9b3 closed
    ironstar-a9b4["ironstar-a9b.4<br/>Implement Todo Decider"]
    class ironstar-a9b4 closed
    ironstar-a9b5["ironstar-a9b.5<br/>Implement Todo View"]
    class ironstar-a9b5 closed
    ironstar-a9b6["ironstar-a9b.6<br/>Implement QuerySession Decider"]
    class ironstar-a9b6 closed
    ironstar-a9b7["ironstar-a9b.7<br/>Wire Todo EventSourcedAggregate"]
    class ironstar-a9b7 closed
    ironstar-a9b8["ironstar-a9b.8<br/>Implement Todo query service with Vie..."]
    class ironstar-a9b8 closed
    ironstar-a9b9["ironstar-a9b.9<br/>Integrate Zenoh event publishing"]
    class ironstar-a9b9 closed
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
    class ironstar-avp closed
    ironstar-b2l["ironstar-b2l<br/>Design Zenoh key expression schema fo..."]
    class ironstar-b2l open
    ironstar-b43["ironstar-b43<br/>Implement error type hierarchy"]
    class ironstar-b43 closed
    ironstar-b8d["ironstar-b8d<br/>Act 3: Expand web interface slides"]
    class ironstar-b8d closed
    ironstar-b9h["ironstar-b9h<br/>Configure tower-http Brotli compressi..."]
    class ironstar-b9h open
    ironstar-c7z["ironstar-c7z<br/>Implement DuckDB remote data source i..."]
    class ironstar-c7z closed
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
    class ironstar-e6k closed
    ironstar-e6k1["ironstar-e6k.1<br/>Define Todo domain model (aggregate, ..."]
    class ironstar-e6k1 closed
    ironstar-e6k2["ironstar-e6k.2<br/>Implement TodoListProjection with in-..."]
    class ironstar-e6k2 closed
    ironstar-e6k3["ironstar-e6k.3<br/>Implement add_todo handler (POST /add..."]
    class ironstar-e6k3 closed
    ironstar-e6k4["ironstar-e6k.4<br/>Implement mark_todo handler (POST /ma..."]
    class ironstar-e6k4 closed
    ironstar-e6k5["ironstar-e6k.5<br/>Implement delete_todo handler (POST /..."]
    class ironstar-e6k5 closed
    ironstar-e6k6["ironstar-e6k.6<br/>Implement GET /todos SSE feed endpoint"]
    class ironstar-e6k6 closed
    ironstar-e6k7["ironstar-e6k.7<br/>Implement todo_list_template renderin..."]
    class ironstar-e6k7 closed
    ironstar-e6k8["ironstar-e6k.8<br/>Implement todo example route mounting"]
    class ironstar-e6k8 closed
    ironstar-e8d["ironstar-e8d<br/>Refactor domain module into aggregate..."]
    class ironstar-e8d closed
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
    ironstar-jdk["ironstar-jdk<br/>Migrate from cargoTest to cargoNextes..."]
    class ironstar-jdk closed
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
    ironstar-jqv13["ironstar-jqv.13<br/>Add SessionMetadata to Session aggregate"]
    class ironstar-jqv13 open
    ironstar-jqv14["ironstar-jqv.14<br/>Document auth evolution strategy"]
    class ironstar-jqv14 open
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
    class ironstar-k1z closed
    ironstar-k94["ironstar-k94<br/>Reference: Strategic domain classific..."]
    class ironstar-k94 open
    ironstar-nor["ironstar-nor<br/>Research Mosaic visualization integra..."]
    class ironstar-nor open
    ironstar-nqq["ironstar-nqq<br/>Performance optimization"]
    class ironstar-nqq closed
    ironstar-nqq1["ironstar-nqq.1<br/>Implement CQRS performance tuning"]
    class ironstar-nqq1 closed
    ironstar-nqq2["ironstar-nqq.2<br/>Implement advanced performance patterns"]
    class ironstar-nqq2 closed
    ironstar-ny3["ironstar-ny3<br/>Frontend build pipeline"]
    class ironstar-ny3 open
    ironstar-ny31["ironstar-ny3.1<br/>Create web-components/ project struct..."]
    class ironstar-ny31 closed
    ironstar-ny310["ironstar-ny3.10<br/>Configure ts-rs export directory and ..."]
    class ironstar-ny310 closed
    ironstar-ny311["ironstar-ny3.11<br/>Create static/dist/ output directory ..."]
    class ironstar-ny311 closed
    ironstar-ny312["ironstar-ny3.12<br/>Implement manifest.json parser for ha..."]
    class ironstar-ny312 closed
    ironstar-ny313["ironstar-ny3.13<br/>Implement rust-embed conditional asse..."]
    class ironstar-ny313 closed
    ironstar-ny314["ironstar-ny3.14<br/>Create web-components/components/ dir..."]
    class ironstar-ny314 closed
    ironstar-ny315["ironstar-ny3.15<br/>Configure Rolldown for Lit web compon..."]
    class ironstar-ny315 closed
    ironstar-ny316["ironstar-ny3.16<br/>Configure OKLch color system with Ope..."]
    class ironstar-ny316 open
    ironstar-ny317["ironstar-ny3.17<br/>Implement light-dark() theming with p..."]
    class ironstar-ny317 open
    ironstar-ny318["ironstar-ny3.18<br/>Add CUBE CSS composition layer"]
    class ironstar-ny318 closed
    ironstar-ny319["ironstar-ny3.19<br/>Add datastar.js to Rolldown build pip..."]
    class ironstar-ny319 closed
    ironstar-ny32["ironstar-ny3.2<br/>Configure Rolldown bundler with conte..."]
    class ironstar-ny32 closed
    ironstar-ny320["ironstar-ny3.20<br/>Fix CSS bundle not appearing in manif..."]
    class ironstar-ny320 closed
    ironstar-ny322["ironstar-ny3.22<br/>Investigate unifying web-components u..."]
    class ironstar-ny322 open
    ironstar-ny33["ironstar-ny3.3<br/>Setup PostCSS configuration for moder..."]
    class ironstar-ny33 closed
    ironstar-ny34["ironstar-ny3.4<br/>Setup Open Props design tokens and th..."]
    class ironstar-ny34 closed
    ironstar-ny35["ironstar-ny3.5<br/>Configure CSS cascade layers for pred..."]
    class ironstar-ny35 closed
    ironstar-ny36["ironstar-ny3.6<br/>Copy Open Props UI component CSS files"]
    class ironstar-ny36 open
    ironstar-ny37["ironstar-ny3.7<br/>Create TypeScript configuration (tsco..."]
    class ironstar-ny37 closed
    ironstar-ny38["ironstar-ny3.8<br/>Create web-components/index.ts entry ..."]
    class ironstar-ny38 closed
    ironstar-ny39["ironstar-ny3.9<br/>Add ts-rs dependency to Cargo.toml"]
    class ironstar-ny39 closed
    ironstar-nyp["ironstar-nyp<br/>Event sourcing infrastructure"]
    class ironstar-nyp open
    ironstar-nyp1["ironstar-nyp.1<br/>Create database migrations/ directory..."]
    class ironstar-nyp1 closed
    ironstar-nyp10["ironstar-nyp.10<br/>Add session TTL cleanup background task"]
    class ironstar-nyp10 closed
    ironstar-nyp11["ironstar-nyp.11<br/>Create Session axum extractor"]
    class ironstar-nyp11 closed
    ironstar-nyp12["ironstar-nyp.12<br/>Implement DuckDB analytics service"]
    class ironstar-nyp12 closed
    ironstar-nyp13["ironstar-nyp.13<br/>Document error handling decisions"]
    class ironstar-nyp13 open
    ironstar-nyp14["ironstar-nyp.14<br/>Implement metrics and observability r..."]
    class ironstar-nyp14 open
    ironstar-nyp15["ironstar-nyp.15<br/>Implement moka analytics cache with r..."]
    class ironstar-nyp15 closed
    ironstar-nyp16["ironstar-nyp.16<br/>Implement DualEventBus for tokio::bro..."]
    class ironstar-nyp16 closed
    ironstar-nyp17["ironstar-nyp.17<br/>Implement EventUpcaster trait and Upc..."]
    class ironstar-nyp17 open
    ironstar-nyp18["ironstar-nyp.18<br/>Implement SSE ConnectionTracker with ..."]
    class ironstar-nyp18 open
    ironstar-nyp19["ironstar-nyp.19<br/>Create EventBus trait abstraction"]
    class ironstar-nyp19 closed
    ironstar-nyp2["ironstar-nyp.2<br/>Create EventStore trait abstraction"]
    class ironstar-nyp2 closed
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
    class ironstar-nyp25 closed
    ironstar-nyp26["ironstar-nyp.26<br/>Create Zenoh embedded router configur..."]
    class ironstar-nyp26 closed
    ironstar-nyp27["ironstar-nyp.27<br/>Implement ZenohEventBus struct with p..."]
    class ironstar-nyp27 closed
    ironstar-nyp28["ironstar-nyp.28<br/>Implement per-session Zenoh subscript..."]
    class ironstar-nyp28 open
    ironstar-nyp29["ironstar-nyp.29<br/>Implement error propagation pattern t..."]
    class ironstar-nyp29 closed
    ironstar-nyp3["ironstar-nyp.3<br/>Implement SQLite EventRepository with..."]
    class ironstar-nyp3 closed
    ironstar-nyp30["ironstar-nyp.30<br/>Implement observability initializatio..."]
    class ironstar-nyp30 open
    ironstar-nyp31["ironstar-nyp.31<br/>Implement health check endpoints (/he..."]
    class ironstar-nyp31 closed
    ironstar-nyp32["ironstar-nyp.32<br/>Instrument Zenoh event bus with Prome..."]
    class ironstar-nyp32 open
    ironstar-nyp33["ironstar-nyp.33<br/>Implement session cleanup background ..."]
    class ironstar-nyp33 closed
    ironstar-nyp34["ironstar-nyp.34<br/>Implement spawn-after-persist for Duc..."]
    class ironstar-nyp34 closed
    ironstar-nyp35["ironstar-nyp.35<br/>Implement hybrid event store schema w..."]
    class ironstar-nyp35 closed
    ironstar-nyp36["ironstar-nyp.36<br/>Document and enforce subscribe-before..."]
    class ironstar-nyp36 open
    ironstar-nyp37["ironstar-nyp.37<br/>Document Galois connection properties..."]
    class ironstar-nyp37 open
    ironstar-nyp38["ironstar-nyp.38<br/>Implement quotient equivalence testin..."]
    class ironstar-nyp38 open
    ironstar-nyp39["ironstar-nyp.39<br/>Coordinate event time, processing tim..."]
    class ironstar-nyp39 open
    ironstar-nyp4["ironstar-nyp.4<br/>Implement SQLite connection pooling a..."]
    class ironstar-nyp4 open
    ironstar-nyp40["ironstar-nyp.40<br/>Document CQRS as profunctor P: Comman..."]
    class ironstar-nyp40 open
    ironstar-nyp41["ironstar-nyp.41<br/>Implement Catalog aggregate Decider (..."]
    class ironstar-nyp41 closed
    ironstar-nyp42["ironstar-nyp.42<br/>Implement Catalog View read model pro..."]
    class ironstar-nyp42 closed
    ironstar-nyp43["ironstar-nyp.43<br/>Implement QuerySession event reposito..."]
    class ironstar-nyp43 closed
    ironstar-nyp44["ironstar-nyp.44<br/>Implement Catalog event repository ad..."]
    class ironstar-nyp44 closed
    ironstar-nyp45["ironstar-nyp.45<br/>Implement QuerySession command handle..."]
    class ironstar-nyp45 closed
    ironstar-nyp46["ironstar-nyp.46<br/>Implement Catalog command handler wit..."]
    class ironstar-nyp46 closed
    ironstar-nyp47["ironstar-nyp.47<br/>Implement Analytics query handlers (r..."]
    class ironstar-nyp47 closed
    ironstar-nyp48["ironstar-nyp.48<br/>Implement Analytics HTTP routes and S..."]
    class ironstar-nyp48 closed
    ironstar-nyp49["ironstar-nyp.49<br/>Implement combined analyticsDecider c..."]
    class ironstar-nyp49 closed
    ironstar-nyp5["ironstar-nyp.5<br/>Implement tokio broadcast event bus"]
    class ironstar-nyp5 open
    ironstar-nyp6["ironstar-nyp.6<br/>Create Projection trait for read models"]
    class ironstar-nyp6 closed
    ironstar-nyp7["ironstar-nyp.7<br/>Implement ProjectionManager with in-m..."]
    class ironstar-nyp7 closed
    ironstar-nyp8["ironstar-nyp.8<br/>Implement SSE 15-second keep-alive co..."]
    class ironstar-nyp8 closed
    ironstar-nyp9["ironstar-nyp.9<br/>Implement SQLite session store with S..."]
    class ironstar-nyp9 closed
    ironstar-o66["ironstar-o66<br/>Optimize CI build derivation set for ..."]
    class ironstar-o66 closed
    ironstar-o661["ironstar-o66.1<br/>Audit rust-flake autoWire settings pe..."]
    class ironstar-o661 closed
    ironstar-o662["ironstar-o66.2<br/>Remove redundant per-crate clippy checks"]
    class ironstar-o662 closed
    ironstar-o663["ironstar-o66.3<br/>Skip per-crate doc derivation builds ..."]
    class ironstar-o663 closed
    ironstar-o664["ironstar-o66.4<br/>Remove ironstar-release from CI packa..."]
    class ironstar-o664 closed
    ironstar-o665["ironstar-o66.5<br/>Verify cachix effectiveness for works..."]
    class ironstar-o665 open
    ironstar-o666["ironstar-o66.6<br/>Remove rust-flake and add per-crate c..."]
    class ironstar-o666 closed
    ironstar-r5f["ironstar-r5f<br/>ironstar-6lq"]
    class ironstar-r5f closed
    ironstar-r62["ironstar-r62<br/>Presentation layer"]
    class ironstar-r62 open
    ironstar-r621["ironstar-r62.1<br/>Add justfile with development and bui..."]
    class ironstar-r621 open
    ironstar-r6210["ironstar-r62.10<br/>Implement component-level hypertext t..."]
    class ironstar-r6210 closed
    ironstar-r6211["ironstar-r62.11<br/>Implement router composition with fea..."]
    class ironstar-r6211 closed
    ironstar-r6212["ironstar-r62.12<br/>Implement graceful shutdown signal ha..."]
    class ironstar-r6212 closed
    ironstar-r6213["ironstar-r62.13<br/>Wire all components together in main.rs"]
    class ironstar-r6213 closed
    ironstar-r6214["ironstar-r62.14<br/>Implement dev-only hotreload SSE endp..."]
    class ironstar-r6214 open
    ironstar-r6215["ironstar-r62.15<br/>Implement health check endpoint for p..."]
    class ironstar-r6215 open
    ironstar-r6216["ironstar-r62.16<br/>Implement DatastarRequest extractor f..."]
    class ironstar-r6216 open
    ironstar-r6217["ironstar-r62.17<br/>Implement comonadic signal compositio..."]
    class ironstar-r6217 open
    ironstar-r622["ironstar-r62.2<br/>Create devShell module with tools and..."]
    class ironstar-r622 open
    ironstar-r623["ironstar-r62.3<br/>Configure pre-commit hooks for code q..."]
    class ironstar-r623 open
    ironstar-r624["ironstar-r62.4<br/>Define AppState struct with all depen..."]
    class ironstar-r624 closed
    ironstar-r625["ironstar-r62.5<br/>Implement SSE feed endpoint with even..."]
    class ironstar-r625 closed
    ironstar-r626["ironstar-r62.6<br/>Implement command POST handlers"]
    class ironstar-r626 closed
    ironstar-r627["ironstar-r62.7<br/>Implement query GET handlers"]
    class ironstar-r627 closed
    ironstar-r628["ironstar-r62.8<br/>Implement RenderableToDatastar conver..."]
    class ironstar-r628 open
    ironstar-r629["ironstar-r62.9<br/>Create base layout template with Data..."]
    class ironstar-r629 closed
    ironstar-rjs["ironstar-rjs<br/>Document nixpkgs-unstable Darwin fram..."]
    class ironstar-rjs open
    ironstar-sgc["ironstar-sgc<br/>Implement CSS manifest handling for p..."]
    class ironstar-sgc open
    ironstar-sj6["ironstar-sj6<br/>Reference: DDD Starter Modelling Proc..."]
    class ironstar-sj6 open
    ironstar-ubj["ironstar-ubj<br/>Act 1: Expand data problem slides"]
    class ironstar-ubj closed
    ironstar-v4y["ironstar-v4y<br/>Multi-crate workspace decomposition"]
    class ironstar-v4y closed
    ironstar-v4y1["ironstar-v4y.1<br/>Define common-enums crate structure"]
    class ironstar-v4y1 closed
    ironstar-v4y2["ironstar-v4y.2<br/>Define common-types crate structure"]
    class ironstar-v4y2 closed
    ironstar-v4y3["ironstar-v4y.3<br/>Define common-utils crate structure"]
    class ironstar-v4y3 closed
    ironstar-wis["ironstar-wis<br/>Optimize CI: replace nix develop -c w..."]
    class ironstar-wis open
    ironstar-ym1["ironstar-ym1<br/>Polish diagrams for visual consistency"]
    class ironstar-ym1 closed
    ironstar-z4s["ironstar-z4s<br/>Act 4: Expand vision slides"]
    class ironstar-z4s closed
    ironstar-zuv["ironstar-zuv<br/>Testing and integration"]
    class ironstar-zuv open
    ironstar-zuv1["ironstar-zuv.1<br/>Create EventRepository integration tests"]
    class ironstar-zuv1 open
    ironstar-zuv2["ironstar-zuv.2<br/>Create projection tests with mock Eve..."]
    class ironstar-zuv2 open
    ironstar-zuv3["ironstar-zuv.3<br/>Create end-to-end handler tests"]
    class ironstar-zuv3 open
    ironstar-zuv4["ironstar-zuv.4<br/>Implement DeciderTestSpecification gi..."]
    class ironstar-zuv4 open

    ironstar-09r ==> ironstar-2nt5
    ironstar-09r ==> ironstar-2nt9
    ironstar-09r ==> ironstar-e6k7
    ironstar-09r ==> ironstar-ny3
    ironstar-09r ==> ironstar-r6210
    ironstar-09r ==> ironstar-r625
    ironstar-0ha -.-> ironstar-r62
    ironstar-29b -.-> ironstar-9dh
    ironstar-29b1 -.-> ironstar-29b
    ironstar-29b10 -.-> ironstar-29b
    ironstar-29b10 ==> ironstar-29b1
    ironstar-29b10 ==> ironstar-29b4
    ironstar-29b11 -.-> ironstar-29b
    ironstar-29b11 ==> ironstar-29b10
    ironstar-29b11 ==> ironstar-29b3
    ironstar-29b11 ==> ironstar-29b4
    ironstar-29b11 ==> ironstar-29b5
    ironstar-29b11 ==> ironstar-29b6
    ironstar-29b11 ==> ironstar-29b7
    ironstar-29b11 ==> ironstar-29b8
    ironstar-29b11 ==> ironstar-29b9
    ironstar-29b12 -.-> ironstar-29b
    ironstar-29b12 ==> ironstar-29b11
    ironstar-29b2 -.-> ironstar-29b
    ironstar-29b2 ==> ironstar-29b1
    ironstar-29b3 -.-> ironstar-29b
    ironstar-29b3 ==> ironstar-29b1
    ironstar-29b4 -.-> ironstar-29b
    ironstar-29b4 ==> ironstar-29b1
    ironstar-29b4 ==> ironstar-29b2
    ironstar-29b5 -.-> ironstar-29b
    ironstar-29b5 ==> ironstar-29b1
    ironstar-29b6 -.-> ironstar-29b
    ironstar-29b6 ==> ironstar-29b1
    ironstar-29b6 ==> ironstar-29b2
    ironstar-29b6 ==> ironstar-29b5
    ironstar-29b7 -.-> ironstar-29b
    ironstar-29b7 ==> ironstar-29b1
    ironstar-29b8 -.-> ironstar-29b
    ironstar-29b8 ==> ironstar-29b1
    ironstar-29b9 -.-> ironstar-29b
    ironstar-29b9 ==> ironstar-29b1
    ironstar-29b9 ==> ironstar-29b5
    ironstar-2it1 -.-> ironstar-2it
    ironstar-2it1 ==> ironstar-2it16
    ironstar-2it10 -.-> ironstar-2it
    ironstar-2it10 ==> ironstar-2it9
    ironstar-2it11 -.-> ironstar-2it
    ironstar-2it11 ==> ironstar-2it10
    ironstar-2it12 -.-> ironstar-2it
    ironstar-2it12 ==> ironstar-2it11
    ironstar-2it13 -.-> ironstar-2it
    ironstar-2it14 -.-> ironstar-2it
    ironstar-2it15 -.-> ironstar-2it
    ironstar-2it16 -.-> ironstar-2it
    ironstar-2it16 ==> ironstar-2it21
    ironstar-2it17 -.-> ironstar-2it
    ironstar-2it17 ==> ironstar-2it13
    ironstar-2it17 ==> ironstar-2it14
    ironstar-2it17 ==> ironstar-2it15
    ironstar-2it18 -.-> ironstar-2it
    ironstar-2it18 ==> ironstar-2it17
    ironstar-2it19 -.-> ironstar-2it
    ironstar-2it19 ==> ironstar-2it17
    ironstar-2it2 -.-> ironstar-2it
    ironstar-2it2 ==> ironstar-2it1
    ironstar-2it20 -.-> ironstar-2it
    ironstar-2it20 ==> ironstar-2it17
    ironstar-2it21 -.-> ironstar-2it
    ironstar-2it21 ==> ironstar-2it18
    ironstar-2it21 ==> ironstar-2it19
    ironstar-2it21 ==> ironstar-2it20
    ironstar-2it21 ==> ironstar-2it22
    ironstar-2it22 -.-> ironstar-2it
    ironstar-2it22 ==> ironstar-2it17
    ironstar-2it23 -.-> ironstar-2it
    ironstar-2it24 -.-> ironstar-2it
    ironstar-2it3 -.-> ironstar-2it
    ironstar-2it3 ==> ironstar-2it2
    ironstar-2it4 -.-> ironstar-2it
    ironstar-2it4 ==> ironstar-2it3
    ironstar-2it5 -.-> ironstar-2it
    ironstar-2it5 ==> ironstar-2it4
    ironstar-2it6 -.-> ironstar-2it
    ironstar-2it6 ==> ironstar-2it5
    ironstar-2it7 -.-> ironstar-2it
    ironstar-2it7 ==> ironstar-2it5
    ironstar-2it8 -.-> ironstar-2it
    ironstar-2it8 ==> ironstar-2it7
    ironstar-2it9 -.-> ironstar-2it
    ironstar-2it9 ==> ironstar-2it8
    ironstar-2nt ==> ironstar-6lq5
    ironstar-2nt1 -.-> ironstar-2nt
    ironstar-2nt10 -.-> ironstar-2nt
    ironstar-2nt10 ==> ironstar-2nt8
    ironstar-2nt11 -.-> ironstar-2nt
    ironstar-2nt12 -.-> ironstar-2nt
    ironstar-2nt12 ==> ironstar-a9b
    ironstar-2nt13 -.-> ironstar-2nt
    ironstar-2nt14 -.-> ironstar-2nt
    ironstar-2nt14 -.-> ironstar-2nt2
    ironstar-2nt15 -.-> ironstar-2nt
    ironstar-2nt15 -.-> ironstar-2nt3
    ironstar-2nt16 -.-> ironstar-2nt
    ironstar-2nt16 -.-> ironstar-2nt14
    ironstar-2nt16 ==> ironstar-2nt17
    ironstar-2nt16 ==> ironstar-a9b6
    ironstar-2nt17 -.-> ironstar-2nt
    ironstar-2nt17 -.-> ironstar-2nt12
    ironstar-2nt18 -.-> ironstar-2nt
    ironstar-2nt19 -.-> ironstar-2nt
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
    ironstar-2vp -.-> ironstar-zuv
    ironstar-3gd ==> ironstar-2nt
    ironstar-3gd ==> ironstar-nyp
    ironstar-3gd1 -.-> ironstar-3gd
    ironstar-3gd1 ==> ironstar-nyp15
    ironstar-3gd2 -.-> ironstar-3gd
    ironstar-3gd2 -.-> ironstar-3gd3
    ironstar-3gd3 -.-> ironstar-3gd
    ironstar-3gd3 -.-> ironstar-nyp25
    ironstar-3gd3 -.-> ironstar-nyp27
    ironstar-3gd4 -.-> ironstar-3gd
    ironstar-3gd4 -.-> ironstar-c7z
    ironstar-507 -.-> ironstar-jqv
    ironstar-5071 -.-> ironstar-507
    ironstar-5072 -.-> ironstar-507
    ironstar-5073 -.-> ironstar-507
    ironstar-5074 -.-> ironstar-507
    ironstar-5074 ==> ironstar-5071
    ironstar-58f ==> ironstar-a9b2
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
    ironstar-72q -.-> ironstar-zuv
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
    ironstar-7536 ==> ironstar-2nt14
    ironstar-7536 -.-> ironstar-753
    ironstar-7536 ==> ironstar-nyp45
    ironstar-7537 -.-> ironstar-09r
    ironstar-7537 -.-> ironstar-753
    ironstar-7a2 -.-> ironstar-2it
    ironstar-7a21 ==> ironstar-507
    ironstar-7a21 -.-> ironstar-7a2
    ironstar-7a210 -.-> ironstar-7a2
    ironstar-7a210 ==> ironstar-7a28
    ironstar-7a211 -.-> ironstar-7a2
    ironstar-7a211 ==> ironstar-7a210
    ironstar-7a211 ==> ironstar-7a28
    ironstar-7a212 -.-> ironstar-7a2
    ironstar-7a212 ==> ironstar-7a29
    ironstar-7a213 -.-> ironstar-7a2
    ironstar-7a213 ==> ironstar-7a22
    ironstar-7a213 ==> ironstar-7a23
    ironstar-7a213 ==> ironstar-7a24
    ironstar-7a213 ==> ironstar-7a25
    ironstar-7a213 ==> ironstar-7a26
    ironstar-7a214 -.-> ironstar-7a2
    ironstar-7a214 ==> ironstar-7a29
    ironstar-7a22 -.-> ironstar-7a2
    ironstar-7a22 ==> ironstar-7a21
    ironstar-7a23 -.-> ironstar-7a2
    ironstar-7a23 ==> ironstar-7a22
    ironstar-7a24 -.-> ironstar-7a2
    ironstar-7a24 ==> ironstar-7a22
    ironstar-7a25 -.-> ironstar-7a2
    ironstar-7a25 ==> ironstar-7a22
    ironstar-7a26 -.-> ironstar-7a2
    ironstar-7a26 ==> ironstar-7a21
    ironstar-7a27 -.-> ironstar-7a2
    ironstar-7a27 ==> ironstar-7a22
    ironstar-7a27 ==> ironstar-7a23
    ironstar-7a27 ==> ironstar-7a24
    ironstar-7a27 ==> ironstar-7a25
    ironstar-7a27 ==> ironstar-7a26
    ironstar-7a28 -.-> ironstar-7a2
    ironstar-7a28 ==> ironstar-7a22
    ironstar-7a28 ==> ironstar-7a23
    ironstar-7a28 ==> ironstar-7a24
    ironstar-7a28 ==> ironstar-7a25
    ironstar-7a28 ==> ironstar-7a26
    ironstar-7a29 -.-> ironstar-7a2
    ironstar-7a29 ==> ironstar-7a22
    ironstar-7a29 ==> ironstar-7a23
    ironstar-7a29 ==> ironstar-7a24
    ironstar-7a29 ==> ironstar-7a25
    ironstar-7a29 ==> ironstar-7a26
    ironstar-89k -.-> ironstar-3gd
    ironstar-89k ==> ironstar-3gd1
    ironstar-89k ==> ironstar-nyp12
    ironstar-961 -.-> ironstar-3gd
    ironstar-961 -.-> ironstar-3gd
    ironstar-9b1 -.-> ironstar-3gd
    ironstar-9b1 -.-> ironstar-3gd
    ironstar-9oj -.-> ironstar-3gd
    ironstar-9oj ==> ironstar-nyp12
    ironstar-a15 -.-> ironstar-0tk
    ironstar-a15 ==> ironstar-edx
    ironstar-a1s -.-> ironstar-zuv
    ironstar-a9b -.-> ironstar-2it
    ironstar-a9b1 -.-> ironstar-a9b
    ironstar-a9b1 ==> ironstar-a9b2
    ironstar-a9b1 ==> ironstar-a9b3
    ironstar-a9b10 -.-> ironstar-a9b
    ironstar-a9b10 ==> ironstar-a9b7
    ironstar-a9b10 ==> ironstar-a9b9
    ironstar-a9b11 -.-> ironstar-a9b
    ironstar-a9b11 ==> ironstar-a9b8
    ironstar-a9b12 -.-> ironstar-a9b
    ironstar-a9b12 ==> ironstar-a9b4
    ironstar-a9b13 -.-> ironstar-a9b
    ironstar-a9b13 ==> ironstar-a9b5
    ironstar-a9b14 ==> ironstar-a9b7
    ironstar-a9b2 -.-> ironstar-a9b
    ironstar-a9b3 -.-> ironstar-a9b
    ironstar-a9b4 -.-> ironstar-a9b
    ironstar-a9b4 ==> ironstar-a9b2
    ironstar-a9b4 -.-> ironstar-b43
    ironstar-a9b5 -.-> ironstar-a9b
    ironstar-a9b5 ==> ironstar-a9b4
    ironstar-a9b6 -.-> ironstar-a9b
    ironstar-a9b6 -.-> ironstar-a9b2
    ironstar-a9b6 -.-> ironstar-b43
    ironstar-a9b7 -.-> ironstar-a9b
    ironstar-a9b7 ==> ironstar-a9b1
    ironstar-a9b7 ==> ironstar-a9b4
    ironstar-a9b8 -.-> ironstar-58f
    ironstar-a9b8 -.-> ironstar-a9b
    ironstar-a9b8 ==> ironstar-a9b5
    ironstar-a9b9 -.-> ironstar-a9b
    ironstar-a9b9 ==> ironstar-a9b7
    ironstar-amw ==> ironstar-a9b1
    ironstar-amw -.-> ironstar-nyp
    ironstar-amw -.-> ironstar-nyp
    ironstar-apx ==> ironstar-rjs
    ironstar-apx ==> ironstar-zuv
    ironstar-apx1 -.-> ironstar-apx
    ironstar-apx1 ==> ironstar-r6213
    ironstar-apx2 ==> ironstar-6lq1
    ironstar-apx2 -.-> ironstar-apx
    ironstar-apx3 -.-> ironstar-apx
    ironstar-apx3 ==> ironstar-apx2
    ironstar-apx4 ==> ironstar-a9b1
    ironstar-apx4 -.-> ironstar-apx
    ironstar-apx5 -.-> ironstar-apx
    ironstar-apx5 ==> ironstar-r6213
    ironstar-avp -.-> ironstar-0tk
    ironstar-b2l ==> ironstar-nyp26
    ironstar-b8d -.-> ironstar-0tk
    ironstar-b8d ==> ironstar-edx
    ironstar-b9h -.-> ironstar-r62
    ironstar-b9h -.-> ironstar-r62
    ironstar-b9h ==> ironstar-r625
    ironstar-c7z -.-> ironstar-3gd
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
    ironstar-e6k2 ==> ironstar-a9b8
    ironstar-e6k2 -.-> ironstar-e6k
    ironstar-e6k2 ==> ironstar-e6k1
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
    ironstar-e8d -.-> ironstar-2nt14
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
    ironstar-jdk -.-> ironstar-6lq
    ironstar-jqv -.-> ironstar-nyp
    ironstar-jqv1 ==> ironstar-2nt2
    ironstar-jqv1 -.-> ironstar-jqv
    ironstar-jqv1 ==> ironstar-jqv10
    ironstar-jqv1 ==> ironstar-jqv4
    ironstar-jqv1 ==> ironstar-nyp11
    ironstar-jqv1 -.-> ironstar-nyp9
    ironstar-jqv10 -.-> ironstar-jqv
    ironstar-jqv11 -.-> ironstar-jqv
    ironstar-jqv12 ==> ironstar-507
    ironstar-jqv12 -.-> ironstar-jqv
    ironstar-jqv13 ==> ironstar-507
    ironstar-jqv13 -.-> ironstar-jqv
    ironstar-jqv14 -.-> ironstar-jqv
    ironstar-jqv2 -.-> ironstar-jqv
    ironstar-jqv2 ==> ironstar-jqv1
    ironstar-jqv2 -.-> ironstar-nyp9
    ironstar-jqv3 ==> ironstar-507
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
    ironstar-ny318 -.-> ironstar-ny3
    ironstar-ny318 ==> ironstar-ny35
    ironstar-ny319 -.-> ironstar-e6k
    ironstar-ny319 -.-> ironstar-ny3
    ironstar-ny32 -.-> ironstar-ny3
    ironstar-ny32 ==> ironstar-ny31
    ironstar-ny320 -.-> ironstar-e6k
    ironstar-ny320 -.-> ironstar-ny3
    ironstar-ny322 -.-> ironstar-ny3
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
    ironstar-nyp1 ==> ironstar-nyp35
    ironstar-nyp10 -.-> ironstar-nyp
    ironstar-nyp10 ==> ironstar-nyp9
    ironstar-nyp11 -.-> ironstar-nyp
    ironstar-nyp11 ==> ironstar-nyp10
    ironstar-nyp12 ==> ironstar-2nt2
    ironstar-nyp12 -.-> ironstar-3gd
    ironstar-nyp12 ==> ironstar-961
    ironstar-nyp12 -.-> ironstar-nyp
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
    ironstar-nyp29 ==> ironstar-a9b
    ironstar-nyp29 ==> ironstar-b43
    ironstar-nyp29 -.-> ironstar-nyp
    ironstar-nyp3 ==> ironstar-a9b1
    ironstar-nyp3 -.-> ironstar-nyp
    ironstar-nyp3 ==> ironstar-nyp1
    ironstar-nyp30 -.-> ironstar-nyp
    ironstar-nyp31 -.-> ironstar-nyp
    ironstar-nyp32 -.-> ironstar-nyp
    ironstar-nyp33 -.-> ironstar-nyp
    ironstar-nyp34 -.-> ironstar-nyp
    ironstar-nyp34 -.-> ironstar-nyp3
    ironstar-nyp35 -.-> ironstar-nyp
    ironstar-nyp36 -.-> ironstar-nyp
    ironstar-nyp37 ==> ironstar-a9b5
    ironstar-nyp37 -.-> ironstar-nyp
    ironstar-nyp38 ==> ironstar-a9b5
    ironstar-nyp38 -.-> ironstar-nyp
    ironstar-nyp39 -.-> ironstar-nyp
    ironstar-nyp39 -.-> ironstar-nyp35
    ironstar-nyp4 ==> ironstar-a9b1
    ironstar-nyp4 -.-> ironstar-nyp
    ironstar-nyp40 -.-> ironstar-nyp
    ironstar-nyp40 -.-> ironstar-nyp2
    ironstar-nyp41 -.-> ironstar-nyp
    ironstar-nyp42 -.-> ironstar-nyp
    ironstar-nyp42 ==> ironstar-nyp41
    ironstar-nyp43 -.-> ironstar-nyp
    ironstar-nyp44 -.-> ironstar-nyp
    ironstar-nyp44 ==> ironstar-nyp41
    ironstar-nyp45 -.-> ironstar-nyp
    ironstar-nyp45 ==> ironstar-nyp34
    ironstar-nyp45 ==> ironstar-nyp43
    ironstar-nyp46 -.-> ironstar-nyp
    ironstar-nyp46 ==> ironstar-nyp44
    ironstar-nyp47 -.-> ironstar-nyp
    ironstar-nyp47 ==> ironstar-nyp42
    ironstar-nyp47 ==> ironstar-nyp43
    ironstar-nyp48 -.-> ironstar-nyp
    ironstar-nyp48 ==> ironstar-nyp45
    ironstar-nyp48 ==> ironstar-nyp46
    ironstar-nyp48 ==> ironstar-nyp47
    ironstar-nyp49 -.-> ironstar-nyp
    ironstar-nyp49 ==> ironstar-nyp41
    ironstar-nyp5 ==> ironstar-2nt2
    ironstar-nyp5 -.-> ironstar-nyp
    ironstar-nyp5 ==> ironstar-nyp19
    ironstar-nyp6 ==> ironstar-2nt2
    ironstar-nyp6 -.-> ironstar-nyp
    ironstar-nyp7 -.-> ironstar-nyp
    ironstar-nyp7 ==> ironstar-nyp27
    ironstar-nyp7 ==> ironstar-nyp3
    ironstar-nyp7 ==> ironstar-nyp5
    ironstar-nyp7 ==> ironstar-nyp6
    ironstar-nyp8 -.-> ironstar-nyp
    ironstar-nyp8 ==> ironstar-nyp27
    ironstar-nyp8 ==> ironstar-nyp5
    ironstar-nyp9 ==> ironstar-2nt2
    ironstar-nyp9 -.-> ironstar-nyp
    ironstar-o661 -.-> ironstar-o66
    ironstar-o662 -.-> ironstar-o66
    ironstar-o662 ==> ironstar-o661
    ironstar-o663 -.-> ironstar-o66
    ironstar-o663 ==> ironstar-o661
    ironstar-o664 -.-> ironstar-o66
    ironstar-o664 ==> ironstar-o661
    ironstar-o665 -.-> ironstar-o66
    ironstar-o666 -.-> ironstar-o66
    ironstar-r62 ==> ironstar-ny3
    ironstar-r62 ==> ironstar-nyp
    ironstar-r621 -.-> ironstar-r62
    ironstar-r6210 ==> ironstar-ny318
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
    ironstar-r6216 -.-> ironstar-r62
    ironstar-r6217 -.-> ironstar-2nt5
    ironstar-r6217 -.-> ironstar-r62
    ironstar-r622 -.-> ironstar-r62
    ironstar-r623 -.-> ironstar-r62
    ironstar-r623 ==> ironstar-r622
    ironstar-r624 ==> ironstar-a9b7
    ironstar-r624 ==> ironstar-a9b8
    ironstar-r624 ==> ironstar-nyp10
    ironstar-r624 ==> ironstar-nyp27
    ironstar-r624 ==> ironstar-nyp3
    ironstar-r624 ==> ironstar-nyp5
    ironstar-r624 ==> ironstar-nyp7
    ironstar-r624 -.-> ironstar-r62
    ironstar-r625 ==> ironstar-nyp8
    ironstar-r625 -.-> ironstar-r62
    ironstar-r625 ==> ironstar-r624
    ironstar-r626 ==> ironstar-2nt4
    ironstar-r626 -.-> ironstar-r62
    ironstar-r626 ==> ironstar-r624
    ironstar-r627 ==> ironstar-a9b8
    ironstar-r627 -.-> ironstar-r62
    ironstar-r627 ==> ironstar-r624
    ironstar-r628 -.-> ironstar-r62
    ironstar-r629 ==> ironstar-ny313
    ironstar-r629 -.-> ironstar-r62
    ironstar-r629 ==> ironstar-r628
    ironstar-sgc -.-> ironstar-ny33
    ironstar-ubj -.-> ironstar-0tk
    ironstar-ubj ==> ironstar-edx
    ironstar-v4y1 -.-> ironstar-v4y
    ironstar-v4y2 -.-> ironstar-v4y
    ironstar-v4y3 -.-> ironstar-v4y
    ironstar-ym1 -.-> ironstar-0tk
    ironstar-z4s -.-> ironstar-0tk
    ironstar-z4s ==> ironstar-edx
    ironstar-zuv ==> ironstar-e6k
    ironstar-zuv1 ==> ironstar-a9b1
    ironstar-zuv1 -.-> ironstar-zuv
    ironstar-zuv2 ==> ironstar-a9b8
    ironstar-zuv2 -.-> ironstar-zuv
    ironstar-zuv2 ==> ironstar-zuv1
    ironstar-zuv3 ==> ironstar-r6213
    ironstar-zuv3 -.-> ironstar-zuv
    ironstar-zuv3 ==> ironstar-zuv1
    ironstar-zuv3 ==> ironstar-zuv2
    ironstar-zuv4 -.-> ironstar-2nt2
    ironstar-zuv4 ==> ironstar-a9b12
    ironstar-zuv4 -.-> ironstar-zuv
```

---

<a id="ironstar-r62-16-implement-datastarrequest-extractor-for-html-sse-routing"></a>

## 📋 ironstar-r62.16 Implement DatastarRequest extractor for HTML/SSE routing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 22:08 |
| **Updated** | 2025-12-26 22:08 |

### Description

Create axum extractor that detects 'datastar-request: true' header to distinguish full HTML page requests from SSE fragment requests. Required for progressive enhancement pattern where initial page load returns full HTML and subsequent Datastar interactions return SSE fragments only. Reference: northstar essential-patterns.md DatastarRequest guard pattern.

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.16 -s in_progress

# Add a comment
bd comment ironstar-r62.16 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.16 -p 1

# View full details
bd show ironstar-r62.16
```

</details>

---

<a id="ironstar-753-third-party-library-integration"></a>

## 🚀 ironstar-753 Third-party library integration

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-27 12:50 |

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

<a id="ironstar-r62-8-implement-renderabletodatastar-conversion-trait"></a>

## 📋 ironstar-r62.8 Implement RenderableToDatastar conversion trait

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-29 17:56 |

### Description

Implement RenderableToDatastar conversion trait preserving structure for SSE transport composition.

This trait implements the transformation F: DomainState → PatchEvent:
- Converts hypertext Buffer to PatchElements (DOM mutations)
- Converts signal state to PatchSignals (reactive updates)

The transformation is a deterministic function (not a functor on individual events) that preserves event identity: no-op state produces no patches.

Reference: denotational-semantics.md 'SSE streaming' section.

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

<a id="ironstar-r62-2-create-devshell-module-with-tools-and-environment"></a>

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

<a id="ironstar-r62-1-add-justfile-with-development-and-build-tasks"></a>

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

<a id="ironstar-r62-presentation-layer"></a>

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

<a id="ironstar-nyp-event-sourcing-infrastructure"></a>

## 🚀 ironstar-nyp Event sourcing infrastructure

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-03 18:44 |

### Description

Event sourcing infrastructure: SQLite event store, Zenoh event bus, projections, SSE streaming

**Strategic classification:** Generic infrastructure (reusable across all domains)

**Hoffman Laws coverage:**
- Law 1: Events immutable ✓ (append-only SQLite)
- Law 2: Event schemas immutable ✓ (upcaster pattern)
- Law 3: All projection data from events ✓ (rebuild from stream)
- Law 5: Projections stem from events ✓ (disposable, rebuildable)
- Law 6: Failure events preserve state ✓ (QueryFailed pattern)
- Law 7: Work is side effect ✓ (pure aggregates)
- Law 8: One flow per process manager — DEFERRED to v2
- Law 9: Process managers consume events, emit commands — DEFERRED to v2
- Law 10: Aggregates own streams ✓ (compound key constraint)

**Algebraic foundation:**
- Events as free monoid (append-only, no inverse)
- Projections as Galois connections (lossy abstraction)
- State reconstruction as catamorphism (fold)

**Reference:** Kevin Hoffman, Real World Event Sourcing (2024)

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

<a id="ironstar-ny3-frontend-build-pipeline"></a>

## 🚀 ironstar-ny3 Frontend build pipeline

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-26 22:08 |

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

<a id="ironstar-f8b-5-verify-process-compose-up-works-with-all-services"></a>

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

<a id="ironstar-f8b-4-configure-cargo-watch-to-curl-hotreload-trigger-on-success"></a>

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

<a id="ironstar-f8b-3-set-up-service-orchestration-frontend-bundler-cargo-watch"></a>

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

<a id="ironstar-f8b-2-configure-process-compose-yaml-for-dev-services"></a>

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

<a id="ironstar-f8b-1-integrate-process-compose-flake-patterns-into-devshell"></a>

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

<a id="ironstar-f8b-process-compose-integration"></a>

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

<a id="ironstar-sgc-implement-css-manifest-handling-for-production-builds"></a>

## 📋 ironstar-sgc Implement CSS manifest handling for production builds

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2026-01-19 17:27 |
| **Updated** | 2026-01-19 17:27 |

### Description

manifest.json from rollup-plugin-output-manifest only includes JS bundles, not CSS. AssetManifest::resolve("bundle.css") returns unhashed fallback, which breaks in production (rust-embed serves hashed filenames). Dev mode unaffected. Discovered during ny3.3. Related: r62.9 base_layout.

### Dependencies

- 🔗 **discovered-from**: `ironstar-ny3.3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-sgc -s in_progress

# Add a comment
bd comment ironstar-sgc 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-sgc -p 1

# View full details
bd show ironstar-sgc
```

</details>

---

<a id="ironstar-jqv-13-add-sessionmetadata-to-session-aggregate"></a>

## 📋 ironstar-jqv.13 Add SessionMetadata to Session aggregate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2026-01-16 19:14 |
| **Updated** | 2026-01-16 19:14 |

### Description

Add SessionMetadata (ip_address, user_agent, geo_location) to SessionCreated event for security audit trail.

Scope:
- Update Session Rust types to include SessionMetadata
- Capture metadata from axum request context at boundary
- Store in SessionCreated event payload
- Update EventCatalog Session events with metadata schema

Spec reference: spec/Session/Session.idr SessionMetadata record
Architecture: docs/notes/architecture/decisions/auth-evolution-strategy.md

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`
- ⛔ **blocks**: `ironstar-507`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.13 -s in_progress

# Add a comment
bd comment ironstar-jqv.13 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.13 -p 1

# View full details
bd show ironstar-jqv.13
```

</details>

---

<a id="ironstar-b2l-design-zenoh-key-expression-schema-for-bounded-context-routing"></a>

## 📋 ironstar-b2l Design Zenoh key expression schema for bounded context routing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2026-01-05 22:44 |
| **Updated** | 2026-01-05 22:44 |

### Description

Define complete key expression hierarchy for CQRS event routing across bounded contexts.

**Context from architecture review:**
- bounded-contexts.md establishes 3 contexts: Session, Todo, Analytics
- event-sourcing-core.md references events/Todo/** filtering pattern
- Session context is Customer-Supplier upstream to other contexts

**Schema requirements:**
- Per-context key prefixes (events/session/**, events/todo/**, events/analytics/**)
- Per-aggregate routing within contexts
- Session-scoped delivery for SSE (events/session/{session_id}/**)
- Version namespace consideration (events/v1/** vs events/**/v1/**)

**Deliverable:**
Document in docs/notes/architecture/infrastructure/zenoh-key-expressions.md

### Dependencies

- ⛔ **blocks**: `ironstar-nyp.26`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-b2l -s in_progress

# Add a comment
bd comment ironstar-b2l 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-b2l -p 1

# View full details
bd show ironstar-b2l
```

</details>

---

<a id="ironstar-58f-implement-viewstaterepository-for-sqlite"></a>

## 📋 ironstar-58f Implement ViewStateRepository for SQLite

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2026-01-05 10:42 |
| **Updated** | 2026-01-05 10:42 |

### Description

Implement fmodel-rust's ViewStateRepository trait for SQLite persistence of View projections.

## Trait Definition (from fmodel-rust)

```rust
pub trait ViewStateRepository<E, S, Error> {
    fn fetch_state(&self, event: &E) -> impl Future<Output = Result<Option<S>, Error>>;
    fn save(&self, state: &S) -> impl Future<Output = Result<S, Error>>;
}
```

## Implementation

```rust
pub struct SqliteViewRepository {
    pool: SqlitePool,
}

impl<E, S> ViewStateRepository<E, S, InfrastructureError> for SqliteViewRepository
where
    E: Identifier,
    S: Serialize + DeserializeOwned + Clone + Send + Sync,
{
    // Store view state keyed by aggregate identifier
}
```

## Schema

```sql
CREATE TABLE view_state (
    view_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    state TEXT NOT NULL CHECK(json_valid(state)),
    updated_at TEXT NOT NULL DEFAULT(datetime('now', 'utc')),
    PRIMARY KEY (view_type, aggregate_id)
) STRICT;
```

## Acceptance Criteria

- [ ] ViewStateRepository trait impl for SQLite
- [ ] JSON serialization of view state
- [ ] Keyed by (view_type, aggregate_id)
- [ ] Used by MaterializedView in application layer

### Dependencies

- ⛔ **blocks**: `ironstar-a9b.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-58f -s in_progress

# Add a comment
bd comment ironstar-58f 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-58f -p 1

# View full details
bd show ironstar-58f
```

</details>

---

<a id="ironstar-nyp-36-document-and-enforce-subscribe-before-replay-invariant"></a>

## 📋 ironstar-nyp.36 Document and enforce subscribe-before-replay invariant

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-27 14:20 |
| **Updated** | 2025-12-27 14:20 |

### Description

Elevate the subscribe-before-replay pattern to a Critical Invariant section in SSE lifecycle documentation. Add assertions/tests.

Critical invariant: SSE handlers must subscribe to the event bus BEFORE loading historical events to prevent race condition where events are missed during replay-to-subscription gap.

See sse-connection-lifecycle.md for full pattern description.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.36 -s in_progress

# Add a comment
bd comment ironstar-nyp.36 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.36 -p 1

# View full details
bd show ironstar-nyp.36
```

</details>

---

<a id="ironstar-jqv-12-implement-session-regeneration-and-user-binding-in-oauth-callback"></a>

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
- ⛔ **blocks**: `ironstar-507`

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

<a id="ironstar-nyp-30-implement-observability-initialization-with-dev-prod-splitting"></a>

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

<a id="ironstar-nyp-22-implement-infrastructureerror-type-with-database-network-variants"></a>

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

<a id="ironstar-ny3-17-implement-light-dark-theming-with-prefers-color-scheme"></a>

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

<a id="ironstar-ny3-16-configure-oklch-color-system-with-open-props-syntax"></a>

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

<a id="ironstar-nyp-21-implement-observability-initialization-module"></a>

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

<a id="ironstar-jqv-7-implement-authcontext-axum-extractor"></a>

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

<a id="ironstar-jqv-authentication-and-authorization"></a>

## 🚀 ironstar-jqv Authentication and authorization

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-22 00:24 |
| **Updated** | 2025-12-27 12:50 |

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

<a id="ironstar-amw-configure-sqlite-production-pragma-settings-wal-synchronous-cache"></a>

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

- 🔗 **parent-child**: `ironstar-nyp`
- 🔗 **child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-a9b.1`

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

<a id="ironstar-b9h-configure-tower-http-brotli-compression-for-sse-responses"></a>

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
- 🔗 **child**: `ironstar-r62`

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

<a id="ironstar-r62-3-configure-pre-commit-hooks-for-code-quality"></a>

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

<a id="ironstar-nyp-4-implement-sqlite-connection-pooling-and-configuration"></a>

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
- ⛔ **blocks**: `ironstar-a9b.1`

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

<a id="ironstar-ny3-6-copy-open-props-ui-component-css-files"></a>

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

<a id="ironstar-wis-optimize-ci-replace-nix-develop-c-with-direct-nix-run"></a>

## 📋 ironstar-wis Optimize CI: replace nix develop -c with direct nix run

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-02-05 00:59 |
| **Updated** | 2026-02-05 00:59 |

### Description

CI audit found 13+ invocations of 'nix develop -c tool' which build the full devshell when only a single tool is needed. Replace with 'nix run .#tool' or 'nix shell .#tool -c command'. Locations: ci.yaml (3×), package-test.yaml (4×), deploy-site.yaml (4×), package-release.yaml (2×). Estimated savings: 75-225 seconds per workflow run.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-wis -s in_progress

# Add a comment
bd comment ironstar-wis 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-wis -p 1

# View full details
bd show ironstar-wis
```

</details>

---

<a id="ironstar-o66-5-verify-cachix-effectiveness-for-workspace-cargoartifacts"></a>

## 📋 ironstar-o66.5 Verify cachix effectiveness for workspace cargoArtifacts

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-02-04 21:51 |
| **Updated** | 2026-02-04 21:51 |

### Description

With the workspace using a single cargoArtifacts derivation (modules/rust.nix:92), verify that cachix properly caches and serves dependency artifacts across CI runs. Check: (1) whether cargoArtifacts hash changes on every Cargo.lock update, (2) whether per-crate autoWire derivations create separate cargoArtifacts or share the workspace one, (3) cachix push/pull success rates from CI logs. Reference: .github/actions/setup-nix/action.yml:111-118 (cachix config).

### Dependencies

- 🔗 **parent-child**: `ironstar-o66`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-o66.5 -s in_progress

# Add a comment
bd comment ironstar-o66.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-o66.5 -p 1

# View full details
bd show ironstar-o66.5
```

</details>

---

<a id="ironstar-ny3-22-investigate-unifying-web-components-under-bun-monorepo-with-nix-compatible-builds"></a>

## 📋 ironstar-ny3.22 Investigate unifying web-components under bun monorepo with nix-compatible builds

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-01-21 22:53 |
| **Updated** | 2026-01-21 22:53 |

### Description

Investigation to determine if web-components/ can become a bun workspace package (packages/web-components) while maintaining nix-compatible builds for rust-embed + crane + cargo.

  Key questions:
  1. Does nixpkgs.bun support workspace package builds for static asset generation?
  2. Can bun-built assets integrate with nix build via crane without breaking reproducibility?
  3. Are there lockfile format issues between bun.lock and crane's Cargo.lock handling?
  4. What are the tradeoffs vs keeping pnpm for web-components?

  Deliverable: Decision document in docs/notes/decisions/ with recommendation.

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-ny3.22 -s in_progress

# Add a comment
bd comment ironstar-ny3.22 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-ny3.22 -p 1

# View full details
bd show ironstar-ny3.22
```

</details>

---

<a id="ironstar-a9b-14-document-commandpipelineerror-in-architecture-docs"></a>

## 📋 ironstar-a9b.14 Document CommandPipelineError in architecture docs

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-01-18 14:56 |
| **Updated** | 2026-01-18 22:00 |

### Description

Document the CommandPipelineError type introduced for fmodel-rust EventSourcedAggregate type unification.

## Context

fmodel-rust's EventSourcedAggregate requires Repository and Decider to share the same Error type parameter. This necessitates a unified error type at the application layer that can represent both domain errors (from Decider) and infrastructure errors (from Repository).

CommandPipelineError is the solution — a new application-layer type that:
- Wraps DomainError for Decider failures
- Wraps InfrastructureError for Repository failures
- Enables fmodel-rust's `Decider::map_error()` pattern

## Files to update

### 1. error-handling-decisions.md

Update the error flow diagram (lines 68-74):
```
ValidationError ──┐
                  ├──> AggregateError ──┐
DomainError ──────┘                     │
                                        ├──> AppError ──> HTTP Response
TodoError ──┐                           │
            ├──> CommandPipelineError ──┘
InfraErr ───┘
```

Add new section explaining:
- Why CommandPipelineError exists (fmodel-rust type constraint)
- How it differs from AggregateError (I/O boundary vs pure aggregation)
- The `map_error` pattern for Decider error transformation

### 2. error-types.md

Add CommandPipelineError definition:
```rust
pub enum CommandPipelineError {
    Domain(DomainError),
    Infrastructure(InfrastructureError),
}
```

Document From implementations and conversion patterns.

### 3. command-write-patterns.md

Update Decider pattern integration (lines 230-353) to show:
- `Decider::map_error()` usage
- Repository adapter pattern for error unification
- Complete EventSourcedAggregate wiring example

## Acceptance criteria

- [ ] error-handling-decisions.md updated with CommandPipelineError flow
- [ ] error-types.md updated with type definition
- [ ] command-write-patterns.md updated with map_error pattern
- [ ] All diagrams consistent with implementation

### Dependencies

- ⛔ **blocks**: `ironstar-a9b.7`

### Comments

> **Cameron Smith** (2026-01-19)
>
> Deferred: core a9b patterns complete. Document when 507/7a2 reveal whether CommandPipelineError pattern generalizes or needs revision.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-a9b.14 -s in_progress

# Add a comment
bd comment ironstar-a9b.14 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-a9b.14 -p 1

# View full details
bd show ironstar-a9b.14
```

</details>

---

<a id="ironstar-507-4-integrate-zenoh-event-publishing-for-session"></a>

## 📋 ironstar-507.4 Integrate Zenoh event publishing for Session

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-01-18 12:56 |
| **Updated** | 2026-01-18 12:56 |

### Description

Integrate Zenoh pub/sub for Session event distribution.

## Key expression pattern

```
events/Session/{session_id}
```

## Implementation

**Location**: `crates/ironstar/src/infrastructure/event_bus/session.rs`

```rust
pub struct SessionEventPublisher {
    session: Arc<zenoh::Session>,
}

impl SessionEventPublisher {
    pub async fn publish(&self, event: &SessionEvent) -> Result<(), PublishError> {
        let key = format!("events/Session/{}", event.identifier());
        let payload = serde_json::to_vec(event)?;
        self.session.put(&key, payload).await?;
        Ok(())
    }
}
```

## Acceptance criteria

- [ ] SessionEventPublisher implementation
- [ ] Key expression routing by session ID
- [ ] Integration tests with embedded Zenoh

## References

- ironstar-a9b.9 task description
- ironstar-nyp.26 (Zenoh embedded router configuration)

### Dependencies

- 🔗 **parent-child**: `ironstar-507`
- ⛔ **blocks**: `ironstar-507.1`

### Comments

> **Cameron Smith** (2026-01-19)
>
> Deferred: Core Session domain complete in 507. Zenoh integration follows pattern from a9b.9 when needed.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-507.4 -s in_progress

# Add a comment
bd comment ironstar-507.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-507.4 -p 1

# View full details
bd show ironstar-507.4
```

</details>

---

<a id="ironstar-507-2-wire-activesessionview-projection"></a>

## 📋 ironstar-507.2 Wire ActiveSessionView projection

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-01-18 12:56 |
| **Updated** | 2026-01-18 12:56 |

### Description

Implement ActiveSessionView projection for quick session lookup.

## Context

The Idris2 spec defines ActiveSessionView for authentication checks. This is a read-side projection optimized for session validation.

## Implementation

**Location**: `crates/ironstar/src/domain/views/session.rs`

```rust
pub struct ActiveSessionViewState {
    pub active_session: Option<(SessionId, UserId, ExpiresAt)>,
}

pub fn active_session_view<'a>() -> ActiveSessionView<'a> {
    View {
        evolve: Box::new(|state, event| evolve_active_session(state, event)),
        initial_state: Box::new(ActiveSessionViewState::default),
    }
}
```

**Event handling**:
- SessionCreated → set active_session to Some(...)
- SessionRefreshed → update expires_at
- SessionInvalidated/SessionExpired → set active_session to None

## Acceptance criteria

- [ ] ActiveSessionView with correct event folding
- [ ] Invariant preservation on edge cases
- [ ] ViewStateComputation tests

## References

- spec/Session/Session.idr (activeSessionView definition)
- ironstar-a9b.8 task description

### Dependencies

- 🔗 **parent-child**: `ironstar-507`

### Comments

> **Cameron Smith** (2026-01-19)
>
> Deferred: Core Session domain complete in 507. Wire ActiveSessionView when auth handlers need session lookup.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-507.2 -s in_progress

# Add a comment
bd comment ironstar-507.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-507.2 -p 1

# View full details
bd show ironstar-507.2
```

</details>

---

<a id="ironstar-507-1-wire-session-eventsourcedaggregate-to-sqlite"></a>

## 📋 ironstar-507.1 Wire Session EventSourcedAggregate to SQLite

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-01-18 12:56 |
| **Updated** | 2026-01-18 12:56 |

### Description

Wire Session Decider to SQLite event store using fmodel-rust EventSourcedAggregate pattern.

## Context

Following ironstar-a9b.7 pattern, connect the Session Decider to SQLite persistence.

## Implementation

**Location**: `crates/ironstar/src/infrastructure/event_store/session.rs`

```rust
pub struct SessionEventStore {
    pool: SqlitePool,
}

impl EventRepository<SessionCommand, SessionEvent, SessionError> for SessionEventStore {
    async fn fetch_events(&self, command: &SessionCommand) -> Result<Vec<SessionEvent>, SessionError>;
    async fn save(&self, events: &[SessionEvent]) -> Result<Vec<SessionEvent>, SessionError>;
}
```

**Aggregate ID routing**: `session_{session_id}`

## Acceptance criteria

- [ ] EventRepository trait implementation for Session
- [ ] Aggregate ID parsing and routing
- [ ] Optimistic locking via previous_id
- [ ] Integration tests with in-memory SQLite

## References

- ironstar-a9b.7 task description
- crates/ironstar/src/domain/session/ (Session Decider)

### Dependencies

- 🔗 **parent-child**: `ironstar-507`

### Comments

> **Cameron Smith** (2026-01-19)
>
> Deferred: Core Session domain complete in 507. Wire EventSourcedAggregate when presentation layer (r62) integration begins.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-507.1 -s in_progress

# Add a comment
bd comment ironstar-507.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-507.1 -p 1

# View full details
bd show ironstar-507.1
```

</details>

---

<a id="ironstar-jqv-14-document-auth-evolution-strategy"></a>

## 📋 ironstar-jqv.14 Document auth evolution strategy

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-01-16 19:14 |
| **Updated** | 2026-01-16 19:14 |

### Description

Create ADR documenting authentication/authorization evolution path from MVP OAuth to future WebAuthn/Cedar/multi-provider.

Deliverable: docs/notes/architecture/decisions/auth-evolution-strategy.md
Covers: UserId evolution, OAuthProvider→AuthMethod, Session boundaries, RBAC→ABAC migration

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-jqv.14 -s in_progress

# Add a comment
bd comment ironstar-jqv.14 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-jqv.14 -p 1

# View full details
bd show ironstar-jqv.14
```

</details>

---

<a id="ironstar-9dh-reference-bounded-context-patterns"></a>

## 📋 ironstar-9dh Reference: Bounded context patterns

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-01-03 18:44 |
| **Updated** | 2026-01-03 18:44 |

### Description

Ironstar v1 operates as single bounded context with implicit internal boundaries (Session, Todo, Analytics). ACL patterns documented for future decomposition. See docs/notes/architecture/core/bounded-contexts.md.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-9dh -s in_progress

# Add a comment
bd comment ironstar-9dh 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-9dh -p 1

# View full details
bd show ironstar-9dh
```

</details>

---

<a id="ironstar-k94-reference-strategic-domain-classification"></a>

## 📋 ironstar-k94 Reference: Strategic domain classification

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-01-03 18:43 |
| **Updated** | 2026-01-03 18:43 |

### Description

Core: QuerySession (analytics). Supporting: Session, Auth. Generic: Todo (example), ES infrastructure. See docs/notes/architecture/core/architecture-decisions.md § Strategic domain classification.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-k94 -s in_progress

# Add a comment
bd comment ironstar-k94 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-k94 -p 1

# View full details
bd show ironstar-k94
```

</details>

---

<a id="ironstar-53t-reference-hoffman-s-laws-compliance-mapping"></a>

## 📋 ironstar-53t Reference: Hoffman's Laws compliance mapping

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-01-03 18:43 |
| **Updated** | 2026-01-03 18:43 |

### Description

Ironstar implements Kevin Hoffman's Ten Laws of Event Sourcing. Laws 1-7, 10 are explicit in implementation. Laws 8-9 (process managers) are deferred to v2. See docs/notes/architecture/cqrs/event-sourcing-core.md.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-53t -s in_progress

# Add a comment
bd comment ironstar-53t 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-53t -p 1

# View full details
bd show ironstar-53t
```

</details>

---

<a id="ironstar-sj6-reference-ddd-starter-modelling-process-integration"></a>

## 📋 ironstar-sj6 Reference: DDD Starter Modelling Process integration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2026-01-03 18:43 |
| **Updated** | 2026-01-03 18:43 |

### Description

Ironstar follows the 8-step DDD Starter Modelling Process adapted for algebraic FDM. See docs/notes/architecture/core/discovery-and-specification.md for mapping of EventStorming artifacts to type system.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-sj6 -s in_progress

# Add a comment
bd comment ironstar-sj6 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-sj6 -p 1

# View full details
bd show ironstar-sj6
```

</details>

---

<a id="ironstar-0ha-document-sse-projection-function-semantics"></a>

## 📋 ironstar-0ha Document SSE projection function semantics

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-30 21:59 |
| **Updated** | 2026-01-05 12:22 |

### Description

Add documentation explaining SSE projection function semantics.

Document:
1. Totality: every domain event maps to at least one patch type (or Option::None for no-ops)
2. Determinism: same event always produces same patch
3. Identity preservation: event sequence order = patch sequence order
4. Conformance to datastar SDK specification

Reference: semantic-model.md § SSE streaming as projection function

### Dependencies

- 🔗 **subtask**: `ironstar-r62`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-0ha -s in_progress

# Add a comment
bd comment ironstar-0ha 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-0ha -p 1

# View full details
bd show ironstar-0ha
```

</details>

---

<a id="ironstar-2vp-test-bitemporal-semantics-and-sse-reconnection-edge-cases"></a>

## 📋 ironstar-2vp Test bitemporal semantics and SSE reconnection edge cases

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-30 21:59 |
| **Updated** | 2026-01-05 12:22 |

### Description

Tests for bitemporal semantics and SSE reconnection edge cases.

Verify:
1. Monotonicity: global_sequence strictly increases
2. Gap detection: server detects when client requests seq N but next available is N+k (k > 1)
3. Stale ID handling: client requests seq < earliest triggers appropriate fallback
4. Correct replay: client receives all events from Last-Event-ID + 1 onwards

Reference: semantic-model.md § Temporal structure

### Dependencies

- 🔗 **subtask**: `ironstar-zuv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2vp -s in_progress

# Add a comment
bd comment ironstar-2vp 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2vp -p 1

# View full details
bd show ironstar-2vp
```

</details>

---

<a id="ironstar-72q-verify-zenoh-key-filtering-preserves-free-monoid-structure"></a>

## 📋 ironstar-72q Verify Zenoh key filtering preserves free monoid structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-30 21:59 |
| **Updated** | 2026-01-05 12:22 |

### Description

Verify Zenoh key expression filtering preserves free monoid structure.

Tests should verify:
1. Filter commutes with concatenation: filter_s([e1, e2]) = filter_s([e1]) ++ filter_s([e2])
2. No duplication: each event appears at most once per session stream
3. No reordering: global_sequence order preserved within session
4. Quotient preservation: session-filtered projection is quotient of global projection

Reference: semantic-model.md § Sessions as indexed profunctor

### Dependencies

- 🔗 **subtask**: `ironstar-zuv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-72q -s in_progress

# Add a comment
bd comment ironstar-72q 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-72q -p 1

# View full details
bd show ironstar-72q
```

</details>

---

<a id="ironstar-a1s-verify-catamorphism-uniqueness-with-property-based-tests"></a>

## 📋 ironstar-a1s Verify catamorphism uniqueness with property-based tests

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-30 21:59 |
| **Updated** | 2026-01-05 12:22 |

### Description

Verify catamorphism uniqueness from initiality using property-based tests (proptest crate).

Properties to verify:
1. Determinism: fold_events([e1, e2, ...]) always produces identical state
2. Batching invariance: fold_events([e1, e2]) = apply(apply(S0, e1), e2)
3. Replay idempotence: replaying same events produces same state

Reference: semantic-model.md § State reconstruction as catamorphism

### Dependencies

- 🔗 **subtask**: `ironstar-zuv`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-a1s -s in_progress

# Add a comment
bd comment ironstar-a1s 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-a1s -p 1

# View full details
bd show ironstar-a1s
```

</details>

---

<a id="ironstar-753-7-document-web-components-as-coalgebras-with-bisimulation-testing"></a>

## 📋 ironstar-753.7 Document web components as coalgebras with bisimulation testing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-29 17:51 |
| **Updated** | 2026-01-03 18:45 |

### Description

Document web components as coalgebras with bisimulation testing

**Algebraic foundation:** semantic-model.md § Coalgebra and web components

**Moore machine interpretation:**
- State: Component internal state
- Output: Rendered DOM
- Transition: Event → new state

**Bisimulation testing:**
- Two states bisimilar iff same output AND equivalent transitions
- data-ignore-morph preserves bisimulation on protected subtrees
- Test: morphing equivalent states produces equivalent DOM

**Verification:** Property tests comparing pre/post-morph DOM structure

### Dependencies

- 🔗 **parent-child**: `ironstar-753`
- 🔗 **documents**: `ironstar-09r`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-753.7 -s in_progress

# Add a comment
bd comment ironstar-753.7 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-753.7 -p 1

# View full details
bd show ironstar-753.7
```

</details>

---

<a id="ironstar-r62-17-implement-comonadic-signal-composition-laws-verification"></a>

## 📋 ironstar-r62.17 Implement comonadic signal composition laws verification

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-29 17:51 |
| **Updated** | 2025-12-29 17:51 |

### Description

Add documentation and tests for Datastar signal comonad laws:
- extract: Signal a → a (getting current value via $signal)
- extend: (Signal a → b) → Signal a → Signal b (computed signals)

Verify laws:
- extend extract = id
- extract ∘ extend f = f
- extend f ∘ extend g = extend (f ∘ extend g)

Update signal-contracts.md with comonad explanation.

Reference: semantic-model.md 'Client signals as comonad' section

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- 🔗 **implements**: `ironstar-2nt.5`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-r62.17 -s in_progress

# Add a comment
bd comment ironstar-r62.17 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-r62.17 -p 1

# View full details
bd show ironstar-r62.17
```

</details>

---

<a id="ironstar-nyp-40-document-cqrs-as-profunctor-p-command-op-view-set"></a>

## 📋 ironstar-nyp.40 Document CQRS as profunctor P: Command^op × View → Set

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-29 17:51 |
| **Updated** | 2026-01-03 18:45 |

### Description

Document CQRS as profunctor P: Command^op × View → Set

**Algebraic foundation:** semantic-model.md § Profunctor architecture

**Properties to document:**
- Contravariance in commands (input transformations compose backwards)
- Covariance in views (output transformations compose forwards)
- Event log as pivot point (mediating data structure)
- Independent scaling follows from profunctor factorization

**Practical implication:** Commands and queries can be modified independently as long as event schema is stable.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- 🔗 **documents**: `ironstar-nyp.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.40 -s in_progress

# Add a comment
bd comment ironstar-nyp.40 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.40 -p 1

# View full details
bd show ironstar-nyp.40
```

</details>

---

<a id="ironstar-nyp-39-coordinate-event-time-processing-time-and-table-version-time"></a>

## 📋 ironstar-nyp.39 Coordinate event time, processing time, and table version time

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-29 17:51 |
| **Updated** | 2025-12-29 17:51 |

### Description

Ensure consistent handling of three temporal axes:
- Event time: created_at in domain events (when event occurred)
- Processing time: global_sequence (when event persisted)
- Table version time: DuckLake snapshots (when analytics snapshot taken)

Implement:
- Clear separation in event schema
- SSE Last-Event-ID uses processing time
- DuckDB queries can specify version time
- Documentation of temporal query semantics

Reference: semantic-model.md 'Bitemporal semantics' section

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- 🔗 **implements**: `ironstar-nyp.35`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.39 -s in_progress

# Add a comment
bd comment ironstar-nyp.39 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.39 -p 1

# View full details
bd show ironstar-nyp.39
```

</details>

---

<a id="ironstar-nyp-38-implement-quotient-equivalence-testing-for-projections"></a>

## 📋 ironstar-nyp.38 Implement quotient equivalence testing for projections

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-29 17:51 |
| **Updated** | 2025-12-29 17:51 |

### Description

Add test utilities verifying quotient monoid properties:
- Equivalence relation is reflexive, symmetric, transitive
- Congruence holds: e₁ ≡ e₂ implies (e₁ ++ e₃) ≡ (e₂ ++ e₃)
- Log compaction produces equivalent projection
- Snapshot + replay equals full replay

Reference: semantic-model.md 'Read models as quotients' section

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-a9b.5`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.38 -s in_progress

# Add a comment
bd comment ironstar-nyp.38 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.38 -p 1

# View full details
bd show ironstar-nyp.38
```

</details>

---

<a id="ironstar-nyp-37-document-galois-connection-properties-in-projection-trait"></a>

## 📋 ironstar-nyp.37 Document Galois connection properties in Projection trait

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-29 17:51 |
| **Updated** | 2026-01-03 18:45 |

### Description

Document Galois connection properties in Projection trait

**Algebraic foundation:** semantic-model.md § Projections as Galois connections

**Properties to document:**
- Prefix order definition on EventLog
- Adjunction properties (abstract ∘ concrete = id, concrete ∘ abstract ≤ id)
- Why projections are lossy (many events → same view)
- Canonical representative selection

**Verification strategy:**
- Property tests for adjunction laws in tests/galois_connection.rs
- QuickCheck: abstract(concrete(v)) == v for all views
- QuickCheck: length(concrete(abstract(events))) <= length(events)

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-a9b.5`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-nyp.37 -s in_progress

# Add a comment
bd comment ironstar-nyp.37 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-nyp.37 -p 1

# View full details
bd show ironstar-nyp.37
```

</details>

---

<a id="ironstar-zuv-4-implement-decidertestspecification-given-when-then-dsl"></a>

## 📋 ironstar-zuv.4 Implement DeciderTestSpecification given/when/then DSL

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-26 18:30 |
| **Updated** | 2026-01-05 11:12 |

### Description

Implement testing infrastructure using fmodel-rust's DeciderTestSpecification DSL.

## Given/When/Then Pattern

```rust
use fmodel_rust::decider::DeciderTestSpecification;

#[test]
fn test_create_todo() {
    DeciderTestSpecification::default()
        .for_decider(todo_decider())
        .given(vec![])  // No prior events
        .when(TodoCommand::Create { id, text })
        .then(vec![TodoEvent::Created { id, text, created_at }]);
}

#[test]
fn test_complete_active_todo() {
    DeciderTestSpecification::default()
        .for_decider(todo_decider())
        .given(vec![TodoEvent::Created { id, text, created_at }])
        .when(TodoCommand::Complete)
        .then_state(Some(TodoState { status: TodoStatus::Completed, .. }));
}
```

## Acceptance Criteria

- [ ] Tests for Todo Decider (create, complete, delete)
- [ ] Tests for QuerySession Decider
- [ ] Failure event tests (NotCreated, NotCompleted)
- [ ] State assertion tests using then_state()

Replaces custom AggregateTestFramework with fmodel-rust's built-in testing DSL.

Local refs: ~/projects/rust-workspace/fmodel-rust/tests/


### Dependencies

- 🔗 **parent-child**: `ironstar-zuv`
- 🔗 **depends-on**: `ironstar-2nt.2`
- ⛔ **blocks**: `ironstar-a9b.12`

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

<a id="ironstar-nyp-32-instrument-zenoh-event-bus-with-prometheus-metrics"></a>

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

<a id="ironstar-nyp-28-implement-per-session-zenoh-subscriptions-for-sse-streams"></a>

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

<a id="ironstar-nyp-24-add-cqrs-pipeline-span-context-propagation"></a>

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

<a id="ironstar-nyp-23-configure-dev-vs-prod-logging-subscribers"></a>

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

<a id="ironstar-jqv-11-implement-session-rate-limiting-with-sliding-window"></a>

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

<a id="ironstar-nyp-20-implement-prometheus-metrics-endpoint-and-instrumentation"></a>

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

<a id="ironstar-jqv-10-implement-oauth-csrf-state-validation"></a>

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

<a id="ironstar-jqv-9-implement-requireauth-axum-extractor"></a>

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

<a id="ironstar-jqv-8-implement-session-regeneration-for-fixation-prevention"></a>

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

<a id="ironstar-jqv-6-implement-rbac-authorization-patterns"></a>

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

<a id="ironstar-nyp-18-implement-sse-connectiontracker-with-atomic-counter"></a>

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

<a id="ironstar-nyp-17-implement-eventupcaster-trait-and-upcasterchain-for-schema-evolution"></a>

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

<a id="ironstar-jqv-5-create-user-identities-table-for-multi-provider-support"></a>

## 📋 ironstar-jqv.5 Create user_identities table for multi-provider support

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-23 23:23 |
| **Updated** | 2026-01-16 12:05 |

### Description

Create user_identities table linking OAuth providers to users. Schema: id (UUID PK), user_id (FK), provider (TEXT: github/google), provider_user_id (TEXT NOT NULL), provider_email (TEXT nullable), created_at. UNIQUE(provider, provider_user_id). Ref: docs/notes/architecture/decisions/oauth-authentication.md

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

<a id="ironstar-jqv-4-implement-users-table-schema-and-userservice"></a>

## 📋 ironstar-jqv.4 Implement users table schema and UserService

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-23 23:23 |
| **Updated** | 2026-01-16 12:05 |

### Description

Create users table for storing authenticated user profiles. Schema (aligned with loom pattern): id (UUID PK), email (TEXT nullable), display_name (TEXT NOT NULL), avatar_url (TEXT nullable), is_admin (BOOLEAN DEFAULT FALSE for first-user promotion), created_at, updated_at. Ref: docs/notes/architecture/decisions/oauth-authentication.md

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

<a id="ironstar-nyp-14-implement-metrics-and-observability-reference"></a>

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

<a id="ironstar-nyp-13-document-error-handling-decisions"></a>

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

<a id="ironstar-jqv-3-implement-concrete-session-patterns"></a>

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
- ⛔ **blocks**: `ironstar-507`

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

<a id="ironstar-jqv-2-implement-session-security-hardening"></a>

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

<a id="ironstar-jqv-1-implement-github-oauth-provider"></a>

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

<a id="ironstar-rjs-document-nixpkgs-unstable-darwin-framework-migration"></a>

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

<a id="ironstar-apx-5-add-structured-logging-with-tracing"></a>

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

<a id="ironstar-apx-4-create-env-development-template-file"></a>

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
- ⛔ **blocks**: `ironstar-a9b.1`

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

<a id="ironstar-apx-2-create-template-parameters-and-conditional-includes"></a>

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

<a id="ironstar-apx-1-create-bootstrap-md-with-complete-setup-instructions"></a>

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

<a id="ironstar-zuv-3-create-end-to-end-handler-tests"></a>

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

<a id="ironstar-zuv-2-create-projection-tests-with-mock-eventrepository"></a>

## 📋 ironstar-zuv.2 Create projection tests with mock EventRepository

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-05 11:12 |

### Description

Unit tests for View projections using mock EventRepository.

## Test Approach

Use fmodel-rust's View pattern with test events:

```rust
#[test]
fn test_todo_view_projection() {
    let view = todo_view();
    let state = (view.initial_state)();
    
    let state = (view.evolve)(&state, &TodoEvent::Created { id, text, created_at });
    assert!(state.items.contains_key(&id));
    
    let state = (view.evolve)(&state, &TodoEvent::Completed { completed_at });
    assert_eq!(state.items[&id].status, TodoStatus::Completed);
}
```

## Mock EventRepository

For MaterializedView integration tests:

```rust
struct MockEventRepository {
    events: Vec<(TestEvent, Uuid)>,
}

impl EventRepository<_, _, _, _> for MockEventRepository { ... }
```

No database required - pure function testing.


### Dependencies

- 🔗 **parent-child**: `ironstar-zuv`
- ⛔ **blocks**: `ironstar-zuv.1`
- ⛔ **blocks**: `ironstar-a9b.8`

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

<a id="ironstar-zuv-1-create-eventrepository-integration-tests"></a>

## 📋 ironstar-zuv.1 Create EventRepository integration tests

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-05 11:12 |

### Description

Integration tests for SqliteEventRepository implementation.

## Test Cases

1. **Basic persistence**: append events, fetch_events returns them in order
2. **Optimistic locking**: previous_id chain validation, concurrent write detection
3. **Version provider**: version_provider returns latest event_id
4. **Query extension methods**: query_since_sequence, earliest/latest_sequence
5. **Stream isolation**: events from different aggregates don't interfere

## Test Setup

```rust
#[sqlx::test]
async fn test_append_and_fetch(pool: SqlitePool) {
    let repo = SqliteEventRepository::new(pool);
    let events = repo.save(&[test_event]).await?;
    let fetched = repo.fetch_events(&command).await?;
    assert_eq!(fetched.len(), 1);
}
```

Uses sqlx::test for automatic database setup/teardown.

Local refs: ~/projects/rust-workspace/fmodel-rust-demo/tests/


### Dependencies

- 🔗 **parent-child**: `ironstar-zuv`
- ⛔ **blocks**: `ironstar-a9b.1`

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

<a id="ironstar-zuv-testing-and-integration"></a>

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

<a id="ironstar-753-3-set-up-lucide-icon-build-time-inlining"></a>

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

<a id="ironstar-753-2-implement-sortable-list-web-component-wrapper"></a>

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

<a id="ironstar-753-1-implement-vegachart-web-component-wrapper"></a>

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

<a id="ironstar-r62-15-implement-health-check-endpoint-for-process-compose"></a>

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

<a id="ironstar-r62-14-implement-dev-only-hotreload-sse-endpoint"></a>

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

<a id="ironstar-nor-research-mosaic-visualization-integration-tbd"></a>

## 📋 ironstar-nor Research Mosaic visualization integration (TBD)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ☕ Low (P3) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 17:11 |
| **Updated** | 2025-12-27 14:20 |

### Description

Evaluate Mosaic grammar of graphics for coordinated multi-chart views.

Research questions:
- DuckDB integration (Mosaic uses DuckDB under the hood)
- Web component wrapper pattern (like ds-echarts)
- Signal binding for selection coordination

Defer to post-MVP unless specific use case emerges.

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

<a id="ironstar-apx-3-define-om-cli-instantiation-tests-and-metadata"></a>

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

<a id="ironstar-apx-documentation-and-template"></a>

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

<a id="ironstar-nyp-5-implement-tokio-broadcast-event-bus"></a>

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

<a id="ironstar-nyp-48-implement-analytics-http-routes-and-sse-feed"></a>

## 📋 ironstar-nyp.48 Implement Analytics HTTP routes and SSE feed

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-02 20:34 |
| **Updated** | 2026-02-02 23:31 |
| **Closed** | 2026-02-02 23:31 |

### Description

Create presentation/analytics.rs with HTTP routes and SSE feed handler following the Todo presentation pattern.

Routes:
- POST /api/analytics/catalog/select - SelectCatalog command
- POST /api/analytics/catalog/refresh - RefreshCatalogMetadata command
- GET /api/analytics/catalog - Query current catalog state
- POST /api/analytics/queries - StartQuery command (returns 202 Accepted)
- DELETE /api/analytics/queries/{id} - CancelQuery command
- GET /api/analytics/queries/{id} - Query specific query status
- GET /api/analytics/queries - Query history
- GET /api/analytics/feed - SSE feed for analytics events

Create AnalyticsAppState (extracted via FromRef<AppState>) containing:
- catalog_repo: Arc<SqliteEventRepository<CatalogCommand, CatalogEvent>>
- query_session_repo: Arc<SqliteEventRepository<QuerySessionCommand, QuerySessionEvent>>
- event_bus: Option<Arc<ZenohEventBus>>
- cached_analytics: Option<CachedAnalyticsService>

SSE feed handler must subscribe to Zenoh BEFORE replaying historical events (critical invariant from Todo pattern). Subscribe to 'events/Analytics/**' for combined Catalog + QuerySession events.

Register routes in presentation/mod.rs app_router under /analytics prefix.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.45`
- ⛔ **blocks**: `ironstar-nyp.46`
- ⛔ **blocks**: `ironstar-nyp.47`

---

<a id="ironstar-nyp-45-implement-querysession-command-handler-with-zenoh-publishing"></a>

## 📋 ironstar-nyp.45 Implement QuerySession command handler with Zenoh publishing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-02 20:33 |
| **Updated** | 2026-02-02 22:41 |
| **Closed** | 2026-02-02 22:41 |

### Description

Create handle_query_session_command following the handle_todo_command pattern.

In application/query_session/handlers.rs add:
- handle_query_session_command<B: EventBus>: generic version for testing
- handle_query_session_command_zenoh: concrete version for axum handlers (Send bounds)

Pattern:
1. Wrap QuerySessionEventRepositoryAdapter
2. Map query_session_decider errors to CommandPipelineError (preserving UUID)
3. Create EventSourcedAggregate from decider + repository
4. Handle command via aggregate
5. Publish saved events to Zenoh (fire-and-forget) on key 'events/Analytics/QuerySession/{id}'

Depends on nyp.43 (repo adapter) and nyp.34 (spawn-after-persist) because the StartQuery command handler must spawn the DuckDB background task after persisting QueryStarted event.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.43`
- ⛔ **blocks**: `ironstar-nyp.34`

---

<a id="ironstar-nyp-44-implement-catalog-event-repository-adapter"></a>

## 📋 ironstar-nyp.44 Implement Catalog event repository adapter

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-02 20:33 |
| **Updated** | 2026-02-02 22:43 |
| **Closed** | 2026-02-02 22:43 |

### Description

Create CatalogEventRepositoryAdapter following the same pattern as TodoEventRepositoryAdapter and QuerySessionEventRepositoryAdapter.

Create application/catalog/mod.rs and application/catalog/handlers.rs with:
- CatalogEventRepositoryAdapter wrapping Arc<SqliteEventRepository<CatalogCommand, CatalogEvent>>
- Implement fmodel-rust EventRepository trait mapping InfrastructureError to CommandPipelineError
- Aggregate type string: 'Catalog'

Depends on nyp.41 (Catalog Decider) because the adapter is parameterized over CatalogCommand and CatalogEvent types.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.41`

---

<a id="ironstar-nyp-43-implement-querysession-event-repository-adapter"></a>

## 📋 ironstar-nyp.43 Implement QuerySession event repository adapter

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-02 20:33 |
| **Updated** | 2026-02-02 21:31 |
| **Closed** | 2026-02-02 21:31 |

### Description

Create QuerySessionEventRepositoryAdapter following the TodoEventRepositoryAdapter pattern from application/todo/handlers.rs.

Create application/query_session/mod.rs and application/query_session/handlers.rs with:
- QuerySessionEventRepositoryAdapter wrapping Arc<SqliteEventRepository<QuerySessionCommand, QuerySessionEvent>>
- Implement fmodel-rust EventRepository trait mapping InfrastructureError to CommandPipelineError
- fetch_events: load events by aggregate type 'QuerySession' and aggregate ID
- save: persist events with sequence, event_type, schema_version, is_final
- version_provider: extract version from latest event for optimistic locking

The existing SqliteEventRepository is generic over C,E and already handles the SQL operations. This adapter maps error types and provides the concrete type bindings.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

---

<a id="ironstar-nyp-41-implement-catalog-aggregate-decider-decide-evolve"></a>

## 📋 ironstar-nyp.41 Implement Catalog aggregate Decider (decide/evolve)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-02 20:32 |
| **Updated** | 2026-02-02 21:38 |
| **Closed** | 2026-02-02 21:38 |

### Description

Implement the Catalog aggregate as a pure Decider following the Todo reference pattern (a9b). The Idris spec (spec/Analytics/Catalog.idr) defines:

Commands: SelectCatalog(CatalogRef), RefreshCatalogMetadata
Events: CatalogSelected(CatalogRef, Timestamp), CatalogMetadataRefreshed(CatalogMetadata, Timestamp)
State machine: NoCatalogSelected -> CatalogActive(CatalogRef, CatalogMetadata)

Create files mirroring Todo structure:
- domain/catalog/mod.rs (re-exports)
- domain/catalog/state.rs (CatalogState, CatalogStatus)
- domain/catalog/commands.rs (CatalogCommand enum with Identifier, DeciderType traits)
- domain/catalog/events.rs (CatalogEvent enum with Identifier, EventType, DeciderType, IsFinal traits)
- domain/catalog/values.rs (CatalogRef, DatasetInfo, CatalogMetadata value objects with smart constructors)
- domain/catalog/errors.rs (CatalogError, CatalogErrorKind with UUID tracking)
- domain/catalog/decider.rs (pure decide/evolve functions, catalog_decider() factory)

Key invariant: Only one catalog active at a time. SelectCatalog on CatalogActive with different ref returns error.
Typed holes from Idris spec (timestamp, metadata) resolve at effect boundary, not in Decider.

Include DeciderTestSpecification tests covering all state transitions and edge cases.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

---

<a id="ironstar-ny3-19-add-datastar-js-to-rolldown-build-pipeline"></a>

## 📋 ironstar-ny3.19 Add datastar.js to Rolldown build pipeline

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-19 19:58 |
| **Updated** | 2026-01-19 20:32 |
| **Closed** | 2026-01-19 20:32 |

### Description

Add @starfederation/datastar npm dependency and configure Rolldown to bundle datastar.js as a separate entry point with content-hashing via manifest.

**Implementation:**
1. npm install @starfederation/datastar (pin version compatible with datastar-rust 0.3.1)
2. Create packages/web-components/src/datastar.ts that imports datastar
3. Add datastar entry to rolldown.config.ts input configuration
4. Update layout.rs to use manifest.resolve("datastar.js") instead of hardcoded path
5. Verify manifest.json includes datastar.js → datastar.[hash].js mapping

**Rationale:** Maintains architectural consistency with ironstar's manifest-based asset resolution pattern rather than introducing a second pattern (copy script).

**Discovered from:** Browser validation revealed datastar.js missing from build pipeline.

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- 🔗 **discovered-from**: `ironstar-e6k`

### Comments

> **Cameron Smith** (2026-01-20)
>
> Implementation complete. Key learnings:
> 
> 1. Package name changed: @starfederation/datastar is deprecated, used @lufrai/datastar instead
> 2. Import path: Must import from @lufrai/datastar/bundles/datastar (full browser bundle with plugins), not the package root (engine only)
> 3. Bundle size: 70KB full bundle vs 11KB engine-only
> 
> Fixed bugs discovered during validation:
> - todo.rs: AssetManifest must be extracted via FromRef, not default
> - main.rs: Todo routes must use AppState (not TodoAppState) for FromRef extractors
> - todo_templates.rs: API paths changed from /api/todos to /todos/api
> 
> CSS still not in manifest - separate issue (Rolldown manifest plugin warning about CSS output).

---

<a id="ironstar-2it-23-resolve-userid-identity-model-composite-key-pattern"></a>

## 📋 ironstar-2it.23 Resolve UserId identity model: composite key pattern

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-15 12:35 |
| **Updated** | 2026-01-15 12:38 |
| **Closed** | 2026-01-15 12:38 |

### Description

Update EventCatalog User entity to match Idris Session.idr composite key pattern (provider, externalId). Remove surrogate UUID and email (email captured in SessionStarted events, projected for UI). Rationale: event immutability (Hoffman Law 1), Shared Kernel self-sufficiency, FRP signal stability, algebraic correctness. Evolution path: Option 3 (VerifiedContact entity) for future account linking.

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`

---

<a id="ironstar-b43-implement-error-type-hierarchy"></a>

## 📋 ironstar-b43 Implement error type hierarchy

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:42 |
| **Updated** | 2026-01-17 15:18 |
| **Closed** | 2026-01-17 15:18 |

### Description

Implement ironstar's layered error type hierarchy as specified in error-types.md.

## Error Types Required

### Foundation Layer (common-enums)
```rust
pub enum ErrorCode {
    ValidationFailed, InvalidInput, NotFound, Conflict,
    Unauthorized, Forbidden, InternalError, DatabaseError, ServiceUnavailable,
}
```

### Domain Layer (ironstar-domain)
```rust
pub struct ValidationError { id: Uuid, kind: ValidationErrorKind, backtrace: Backtrace }
pub struct DomainError { id: Uuid, kind: DomainErrorKind, backtrace: Backtrace }
```

### Application Layer (ironstar-app)
```rust
pub enum AggregateError {
    Validation(Vec<ValidationError>),  // Applicative (collect all)
    Domain(DomainError),                // Monadic (fail fast)
}
```

### Infrastructure Layer (ironstar-interfaces)
```rust
pub struct InfrastructureError { id: Uuid, kind: InfrastructureErrorKind, backtrace: Backtrace }
```

### Presentation Layer (ironstar-web)
```rust
pub struct AppError { id: Uuid, kind: AppErrorKind, backtrace: Backtrace }
```

## Key Design Points

- All errors include UUID for distributed tracing correlation
- From impls enable ? propagation across layers
- AggregateError::Validation holds Vec for applicative validation
- ErrorCode maps to HTTP status codes

## Acceptance Criteria

- [ ] All 5 error types implemented in correct crates
- [ ] From impls for ? propagation chain
- [ ] ErrorCode::http_status() returns correct HTTP status codes
- [ ] UUID tracking on all error types
- [ ] AppError implements axum::IntoResponse

---

<a id="ironstar-a9b-12-implement-decider-specification-tests"></a>

## 📋 ironstar-a9b.12 Implement Decider specification tests

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:19 |
| **Updated** | 2026-01-18 21:29 |
| **Closed** | 2026-01-18 21:29 |

### Description

Implement tests for Todo and QuerySession Deciders using DeciderTestSpecification.

## Implementation

```rust
#[test]
fn test_create_todo() {
    DeciderTestSpecification::default()
        .for_decider(todo_decider())
        .given(vec![])
        .when(TodoCommand::Create { id: id.clone(), text: text.clone() })
        .then(vec![TodoEvent::Created { id, text, created_at: /* ... */ }]);
}

#[test]
fn test_create_existing_todo_emits_failure() {
    DeciderTestSpecification::default()
        .for_decider(todo_decider())
        .given(vec![TodoEvent::Created { ... }])
        .when(TodoCommand::Create { ... })
        .then(vec![TodoEvent::NotCreated { reason: "..." }]);
}
```

## Test Scenarios

### Todo Decider
- Create new todo
- Create existing todo (failure event)
- Complete active todo
- Complete non-existent todo (failure event)
- Delete todo

### QuerySession Decider
- Start query with valid SQL
- Start query with invalid SQL (validation error)
- Cancel running query

## Acceptance Criteria

- [ ] All happy paths tested
- [ ] All failure event paths tested
- [ ] No I/O in tests (pure Decider testing)
- [ ] Tests in crates/ironstar-domain/tests/

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-a9b.4`

---

<a id="ironstar-a9b-9-integrate-zenoh-event-publishing"></a>

## 📋 ironstar-a9b.9 Integrate Zenoh event publishing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:18 |
| **Updated** | 2026-01-18 21:43 |
| **Closed** | 2026-01-18 21:43 |

### Description

Integrate Zenoh event bus for post-persist event broadcast.

## Implementation

```rust
// After EventSourcedAggregate::handle() succeeds
for (event, version) in &events {
    let key = format!("events/{}/{}", E::decider_type(), event.identifier());
    zenoh.put(&key, serde_json::to_vec(event)?).await?;
}
```

## Key Pattern

- Publish AFTER successful persist (not before)
- Use key expression for routing: events/Todo/{aggregate_id}
- Fire-and-forget (don't fail command on publish error)
- SSE handlers subscribe with wildcard: events/Todo/**

## Acceptance Criteria

- [ ] EventBus trait with publish method
- [ ] Zenoh implementation using key expressions
- [ ] Integration in command handler after persist
- [ ] Error logging but no command failure on publish error

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-a9b.7`

---

<a id="ironstar-a9b-7-wire-todo-eventsourcedaggregate"></a>

## 📋 ironstar-a9b.7 Wire Todo EventSourcedAggregate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:18 |
| **Updated** | 2026-01-18 15:29 |
| **Closed** | 2026-01-18 15:29 |

### Description

Wire Todo Decider with EventRepository using fmodel-rust's EventSourcedAggregate.

## Type unification constraint

fmodel-rust's EventSourcedAggregate requires Repository and Decider to share the same Error type parameter:

```rust
pub struct EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
    Decider: EventComputation<C, S, E, Error>,  // Same Error
```

This requires a unified error type: `CommandPipelineError`.

## Implementation

```rust
// crates/ironstar/src/application/error.rs

/// Error type for EventSourcedAggregate pipeline.
/// Unifies domain and infrastructure failures for fmodel-rust type alignment.
#[derive(Debug)]
pub enum CommandPipelineError {
    /// Domain logic failure (from Decider via map_error).
    Domain(DomainError),
    /// Infrastructure failure (from Repository).
    Infrastructure(InfrastructureError),
}

impl From<CommandPipelineError> for AppError {
    fn from(e: CommandPipelineError) -> Self {
        match e {
            CommandPipelineError::Domain(d) => AppError::from(d),
            CommandPipelineError::Infrastructure(i) => AppError::from(i),
        }
    }
}
```

```rust
// crates/ironstar/src/application/todo/handlers.rs

use crate::application::error::CommandPipelineError;
use crate::domain::error::DomainError;
use crate::domain::todo::{TodoCommand, TodoEvent, todo_decider};
use crate::infrastructure::event_store::SqliteEventRepository;
use fmodel_rust::aggregate::EventSourcedAggregate;
use std::sync::Arc;

/// Repository adapter that maps InfrastructureError to CommandPipelineError.
struct TodoEventRepository {
    inner: Arc<SqliteEventRepository<TodoCommand, TodoEvent>>,
}

impl EventRepository<TodoCommand, TodoEvent, String, CommandPipelineError> for TodoEventRepository {
    // Delegate to inner, mapping errors
}

pub async fn handle_todo_command(
    event_repository: Arc<SqliteEventRepository<TodoCommand, TodoEvent>>,
    command: TodoCommand,
) -> Result<Vec<(TodoEvent, String)>, CommandPipelineError> {
    let repo_adapter = TodoEventRepository { inner: event_repository };
    
    let mapped_decider = todo_decider()
        .map_error(|e| CommandPipelineError::Domain(DomainError::from(e)));

    let aggregate = EventSourcedAggregate::new(
        repo_adapter,
        mapped_decider,
    );
    
    aggregate.handle(&command).await
}
```

## Key patterns

- **Unified error type**: `CommandPipelineError` satisfies fmodel-rust's type constraint
- **Decider::map_error()**: Transforms `TodoError` → `CommandPipelineError::Domain`
- **Repository adapter**: Wraps SqliteEventRepository to map `InfrastructureError` → `CommandPipelineError::Infrastructure`
- **Version type**: `String` (matching SqliteEventRepository implementation)

## Design decisions

**Why CommandPipelineError (not extend AggregateError)?**
- AggregateError is for applicative/monadic error aggregation (validation collection)
- CommandPipelineError is for fmodel-rust type unification (I/O boundary)
- Mixing infrastructure errors into AggregateError would violate layering
- Reference: research in architecture docs, `application/error.rs` comments

**File location**: `crates/ironstar/src/application/todo/handlers.rs`
- Mirrors domain organization pattern (`domain/todo/*`)
- Scales to additional bounded contexts

## References

- fmodel-rust map_error: `/Users/crs58/projects/rust-workspace/fmodel-rust/src/decider.rs:338-376`
- fmodel-rust test pattern: `/Users/crs58/projects/rust-workspace/fmodel-rust/tests/aggregate_test.rs:188-189`
- SqliteEventRepository: `crates/ironstar/src/infrastructure/event_store.rs:297`

## Acceptance criteria

- [ ] Create `CommandPipelineError` in `application/error.rs`
- [ ] Create `crates/ironstar/src/application/todo/` directory with `mod.rs`
- [ ] Implement `TodoEventRepository` adapter in `handlers.rs`
- [ ] Implement `handle_todo_command` with `map_error` pattern
- [ ] Add `From<CommandPipelineError> for AppError` implementation
- [ ] Re-export from `application/mod.rs`

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-a9b.1`
- ⛔ **blocks**: `ironstar-a9b.4`

---

<a id="ironstar-a9b-4-implement-todo-decider"></a>

## 📋 ironstar-a9b.4 Implement Todo Decider

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:18 |
| **Updated** | 2026-01-17 21:46 |
| **Closed** | 2026-01-17 21:46 |

### Description

REWRITE Todo domain using fmodel-rust Decider pattern. Source of truth: `spec/Todo/Todo.idr`

## Files to DELETE (no preservation, no backward compatibility)

- `crates/ironstar/src/domain/todo/aggregate.rs`

## Files to CREATE

- `crates/ironstar/src/domain/todo/decider.rs` — todo_decider() factory function

## Principled decisions

1. **Error handling**: Err(AggregateError) for preconditions; Ok(vec![]) for idempotent no-ops
2. **Timestamp injection**: CommandContext parameter, not Utc::now()
3. **No backward compatibility**: Delete aggregate.rs entirely

## State model (from Idris spec)

- NonExistent → Active ↔ Completed → Deleted (terminal)

## Acceptance criteria

- [ ] DELETE crates/ironstar/src/domain/todo/aggregate.rs
- [ ] CREATE crates/ironstar/src/domain/todo/decider.rs
- [ ] Pure decide/evolve (no async, no I/O, no Utc::now())
- [ ] Timestamps via CommandContext
- [ ] Idempotent ops return Ok(vec![])
- [ ] DeciderTestSpecification tests for all transitions


### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-a9b.2`
- 🔗 **blocked-by**: `ironstar-b43`

---

<a id="ironstar-a9b-3-create-event-store-sqlite-schema"></a>

## 📋 ironstar-a9b.3 Create event store SQLite schema

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:18 |
| **Updated** | 2026-01-15 00:33 |
| **Closed** | 2026-01-15 00:33 |

### Description

Create SQLite schema adapted from fmodel-rust-postgres for event store.

## Schema

```sql
CREATE TABLE deciders (
    decider TEXT NOT NULL,
    event TEXT NOT NULL,
    PRIMARY KEY (decider, event)
) STRICT;

CREATE TABLE events (
    event TEXT NOT NULL,
    event_id TEXT NOT NULL UNIQUE CHECK(length(event_id) = 36),
    decider TEXT NOT NULL,
    decider_id TEXT NOT NULL,
    data TEXT NOT NULL CHECK(json_valid(data)),
    command_id TEXT,
    previous_id TEXT UNIQUE REFERENCES events(event_id),
    final INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT(datetime('now', 'utc')),
    offset INTEGER PRIMARY KEY AUTOINCREMENT,
    FOREIGN KEY (decider, event) REFERENCES deciders (decider, event)
) STRICT;

CREATE INDEX idx_events_decider ON events (decider_id, offset);
```

Plus triggers for immutability and stream integrity.

## Acceptance Criteria

- [ ] Schema in migrations/001_events.sql
- [ ] Triggers prevent DELETE/UPDATE on events
- [ ] Triggers validate previous_id chain integrity
- [ ] sqlx migration runs successfully

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`

---

<a id="ironstar-a9b-2-implement-fmodel-rust-identifier-traits"></a>

## 📋 ironstar-a9b.2 Implement fmodel-rust identifier traits

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:18 |
| **Updated** | 2026-01-17 21:09 |
| **Closed** | 2026-01-17 21:09 |

### Description

Implement the marker traits required by fmodel-rust's EventRepository bounds.

## Strategic context

This is the entry point for the fmodel-rust adoption. Completing this enables:
- a9b.4 (Todo Decider) + a9b.6 (QuerySession Decider) — REWRITE existing aggregates
- a9b.6 unblocks 2nt.16 (analytics workflow) — closes Domain layer epic at 100%

**See `bd show ironstar-a9b` for principled decisions** (error handling, timestamps, deprecation strategy).

## Traits required

```rust
pub trait Identifier { fn identifier(&self) -> String; }
pub trait EventType { fn event_type(&self) -> String; }
pub trait DeciderType { fn decider_type(&self) -> String; }
pub trait IsFinal { fn is_final(&self) -> bool; }
```

## Location

`crates/ironstar/src/domain/common/traits.rs` — re-exported via domain module

## Acceptance criteria

- [ ] All 4 traits defined in domain layer
- [ ] TodoCommand implements Identifier + DeciderType
- [ ] TodoEvent implements Identifier + EventType + DeciderType + IsFinal
- [ ] QueryCommand implements Identifier + DeciderType
- [ ] QueryEvent implements Identifier + EventType + DeciderType + IsFinal
- [ ] Traits re-exported from domain module

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`

---

<a id="ironstar-a9b-1-implement-sqlite-eventrepository"></a>

## 📋 ironstar-a9b.1 Implement SQLite EventRepository

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:18 |
| **Updated** | 2026-01-17 23:38 |
| **Closed** | 2026-01-17 23:38 |

### Description

Implement fmodel-rust's EventRepository<C, E, Version, Error> trait for SQLite.

## Implementation

Reference: fmodel-rust-postgres (~/projects/rust-workspace/fmodel-rust-postgres/)

```rust
impl<C, E> EventRepository<C, E, Uuid, InfrastructureError> for SqliteEventRepository
where
    C: Identifier + DeciderType + Sync,
    E: Identifier + EventType + DeciderType + IsFinal + Serialize + DeserializeOwned + Clone + Sync,
{
    async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Uuid)>, InfrastructureError>;
    async fn save(&self, events: &[E]) -> Result<Vec<(E, Uuid)>, InfrastructureError>;
    async fn version_provider(&self, event: &E) -> Result<Option<Uuid>, InfrastructureError>;
}
```

## Extension Methods (beyond fmodel-rust trait)

```rust
impl SqliteEventRepository {
    /// Query all events (for projection rebuild on startup)
    pub async fn query_all(&self) -> Result<Vec<StoredEvent>, InfrastructureError>;
    
    /// Query events since a global sequence (for SSE Last-Event-ID reconnection)
    pub async fn query_since_sequence(&self, since: i64) -> Result<Vec<StoredEvent>, InfrastructureError>;
    
    /// Get earliest global sequence (for stream bounds)
    pub async fn earliest_sequence(&self) -> Result<Option<i64>, InfrastructureError>;
    
    /// Get latest global sequence (for SSE initial state)
    pub async fn latest_sequence(&self) -> Result<Option<i64>, InfrastructureError>;
}
```

## Schema

Use schema from evaluation doc with previous_id for optimistic locking.

## Acceptance Criteria

- [ ] Implements all 3 EventRepository trait methods
- [ ] Uses previous_id UNIQUE constraint for optimistic locking
- [ ] Global offset column for SSE Last-Event-ID support
- [ ] Triggers prevent delete/update on events table
- [ ] Extension methods for projection rebuild and SSE reconnection

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-a9b.3`
- ⛔ **blocks**: `ironstar-a9b.2`

---

<a id="ironstar-nyp-35-implement-hybrid-event-store-schema-with-dual-sequence-columns"></a>

## 📋 ironstar-nyp.35 Implement hybrid event store schema with dual sequence columns

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-27 14:20 |
| **Updated** | 2026-01-05 11:10 |
| **Closed** | 2026-01-05 11:10 |

### Description

Implement hybrid event store schema with dual sequence columns establishing the free monoid identity/composition structure and bitemporal semantics.

Free monoid structure:
- Identity: empty event sequence []
- Composition: event concatenation ++
- global_sequence: monotonic total order (processing time)
- aggregate_sequence: per-aggregate versioning (optimistic locking)

Bitemporal axes:
- Event time: created_at (when domain event occurred)
- Processing time: global_sequence (when event persisted)

Reference: denotational-semantics.md 'Event log as free monoid' section.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

---

<a id="ironstar-nyp-34-implement-spawn-after-persist-for-duckdb-query-execution"></a>

## 📋 ironstar-nyp.34 Implement spawn-after-persist for DuckDB query execution

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-27 12:50 |
| **Updated** | 2026-02-02 21:33 |
| **Closed** | 2026-02-02 21:33 |

### Description

After QueryStarted event, tokio::spawn background task for DuckDB execution. Emits QueryCompleted/Failed via Zenoh. Returns 202 immediately.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- 🔗 **depends-on**: `ironstar-nyp.3`

---

<a id="ironstar-2nt-17-implement-analyticserror-with-uuid-correlation"></a>

## 📋 ironstar-2nt.17 Implement AnalyticsError with UUID correlation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-27 12:50 |
| **Updated** | 2026-01-17 00:42 |
| **Closed** | 2026-01-17 00:42 |

### Description

Error wrapper: struct AnalyticsError { id: Uuid, kind: ErrorKind, backtrace }. Enables distributed tracing across async boundaries.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- 🔗 **depends-on**: `ironstar-2nt.12`

---

<a id="ironstar-2nt-16-define-analytics-workflow-as-pure-function-pipeline"></a>

## 📋 ironstar-2nt.16 Define analytics workflow as pure function pipeline

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-27 12:50 |
| **Updated** | 2026-01-17 22:17 |
| **Closed** | 2026-01-17 22:17 |

### Description

Wlaschin pipeline: validate_dataset_url -> load_schema -> validate_query -> execute -> transform_for_chart. Pure functions, Result types, effects at boundaries.

## Pattern reference (from session 2024-12-27)

The value objects and aggregate are now implemented:
- DatasetRef, SqlQuery, QueryId, ChartConfig in domain/analytics/values.rs
- QuerySessionAggregate with spawn-after-persist pattern in domain/query_session/

The workflow should compose these types into a railway-oriented pipeline where:
1. Each step returns Result<T, AnalyticsError>
2. Async I/O (DuckDB execution) happens at boundaries
3. Pure validation and transformation in the middle

## Implementation location

Create in domain/analytics/workflow.rs (new file in analytics subdirectory).

## Dependencies

Requires ironstar-2nt.17 (AnalyticsError) for unified error handling across pipeline stages.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- 🔗 **depends-on**: `ironstar-2nt.14`
- ⛔ **blocks**: `ironstar-2nt.17`
- ⛔ **blocks**: `ironstar-a9b.6`

---

<a id="ironstar-2nt-15-define-analytics-value-objects-datasetref-sqlquery-chartconfig"></a>

## 📋 ironstar-2nt.15 Define analytics value objects (DatasetRef, SqlQuery, ChartConfig)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-27 12:50 |
| **Updated** | 2025-12-27 22:50 |
| **Closed** | 2025-12-27 22:50 |

### Description

Create validated value objects for analytics domain following TodoId/TodoText patterns.

## Pattern reference (from Todo implementation)

Follow crates/ironstar/src/domain/values.rs:
- Newtype wrappers with private inner field
- Smart constructor: new() -> Result<Self, Error>
- #[serde(transparent)] for ID types
- #[serde(try_from, into)] for validated strings
- #[ts(export, type = "string")] for TypeScript generation

## Value objects to implement

### DatasetRef
Reference to a dataset (HuggingFace path, S3 URI, or local path).

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(try_from = "String", into = "String")]
pub struct DatasetRef(String);

impl DatasetRef {
    pub fn new(s: impl Into<String>) -> Result<Self, AnalyticsError> {
        let s = s.into();
        // Validate: non-empty, valid URI format or path
        Ok(Self(s))
    }
}
```

### SqlQuery
Validated SQL query string.

```rust
pub struct SqlQuery(String);
// Validation: non-empty, basic SQL sanity checks
```

### QueryId
Unique query identifier (UUID wrapper like TodoId).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct QueryId(Uuid);
```

### ChartConfig
Configuration for ECharts visualization (may be more complex - product type).

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub struct ChartConfig {
    pub chart_type: ChartType,
    pub x_axis: Option<String>,
    pub y_axis: Option<String>,
    // ...
}
```

Local refs:
- Pattern: crates/ironstar/src/domain/values.rs (TodoId, TodoText)
- Error type: will need AnalyticsError (see ironstar-2nt.17)

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- 🔗 **depends-on**: `ironstar-2nt.3`

---

<a id="ironstar-2nt-14-define-querysession-aggregate-with-typed-holes"></a>

## 📋 ironstar-2nt.14 Define QuerySession aggregate with typed holes

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-27 12:50 |
| **Updated** | 2025-12-27 22:54 |
| **Closed** | 2025-12-27 22:54 |

### Description

Implement QuerySession aggregate following patterns established by TodoAggregate.

## Pattern reference (from Todo implementation)

Follow the patterns in crates/ironstar/src/domain/:
- **Aggregate trait**: aggregate.rs - handle_command returns Result<Vec<Event>, Error>
- **State machine**: todo.rs - TodoStatus enum, pure apply_event
- **Value objects**: values.rs - smart constructors with validation

## QuerySession-specific concerns

Unlike Todo, QuerySession has async infrastructure needs:

1. **Spawn-after-persist pattern**: DuckDB query execution happens AFTER event persistence
   - Command handler emits QueryStarted event
   - Application layer persists event, THEN spawns async query task
   - Query completion emits QueryCompleted/QueryFailed via separate command

2. **Session-scoped state machine**:
   ```
   Idle → Pending → Executing → Completed
                  ↘ Failed
   ```

3. **Typed holes pattern**: Define signatures with todo!() implementations first

## Implementation sketch

```rust
pub struct QuerySessionAggregate;

pub enum QuerySessionStatus {
    Idle,
    Pending { query_id: QueryId },
    Executing { query_id: QueryId, started_at: DateTime<Utc> },
    Completed { query_id: QueryId, row_count: usize },
    Failed { query_id: QueryId, error: String },
}

impl Aggregate for QuerySessionAggregate {
    const NAME: &'static str = "QuerySession";
    type State = QuerySessionState;
    type Command = QuerySessionCommand;
    type Event = QuerySessionEvent;
    type Error = QuerySessionError;
    // ... follow TodoAggregate pattern
}
```

Local refs: 
- Pattern: crates/ironstar/src/domain/todo.rs
- Architecture: docs/notes/architecture/cqrs/event-sourcing-core.md (Reference implementation section)

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- 🔗 **depends-on**: `ironstar-2nt.2`

---

<a id="ironstar-jdk-migrate-from-cargotest-to-cargonextest-with-dual-devshell-ci-support"></a>

## 📋 ironstar-jdk Migrate from cargoTest to cargoNextest with dual devshell/CI support

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-27 10:16 |
| **Updated** | 2025-12-27 10:21 |
| **Closed** | 2025-12-27 10:21 |

### Description

Replace crane cargoTest with cargoNextest in flake checks. Add cargoDocTest for doctest coverage. Include cargo-nextest in devshell for local CLI usage. Create .config/nextest.toml with default and ci profiles. Update justfile rust-test recipe to use nextest. Partitions=1 initially, ready to scale.

### Dependencies

- 🔗 **child-of**: `ironstar-6lq`

---

<a id="ironstar-2nt-13-enforce-async-sync-boundary-via-module-organization"></a>

## 📋 ironstar-2nt.13 Enforce async/sync boundary via module organization

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2026-01-17 00:33 |
| **Closed** | 2026-01-17 00:33 |

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

---

<a id="ironstar-nyp-29-implement-error-propagation-pattern-through-cqrs-pipeline"></a>

## 📋 ironstar-nyp.29 Implement error propagation pattern through CQRS pipeline

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2026-01-19 13:29 |
| **Closed** | 2026-01-19 13:29 |

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
- ⛔ **blocks**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-b43`

---

<a id="ironstar-2nt-12-implement-uuid-tracked-error-type-for-distributed-correlation"></a>

## 📋 ironstar-2nt.12 Implement UUID-tracked error type for distributed correlation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2026-01-17 00:33 |
| **Closed** | 2026-01-17 00:33 |

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
- ⛔ **blocks**: `ironstar-a9b`

---

<a id="ironstar-2nt-11-add-version-self-u64-to-aggregate-trait"></a>

## 📋 ironstar-2nt.11 Add version(&self) -> u64 to Aggregate trait

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2025-12-27 22:26 |
| **Closed** | 2025-12-27 22:26 |

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

---

<a id="ironstar-nyp-26-create-zenoh-embedded-router-configuration"></a>

## 📋 ironstar-nyp.26 Create Zenoh embedded router configuration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2026-01-19 13:29 |
| **Closed** | 2026-01-19 13:29 |

### Description

Configure zenoh::Config for embedded mode with peer discovery disabled per distributed-event-bus-migration.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

---

<a id="ironstar-753-6-implement-chart-sse-endpoint-with-signal-driven-options"></a>

## 📋 ironstar-753.6 Implement chart SSE endpoint with signal-driven options

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2026-02-03 00:02 |
| **Closed** | 2026-02-03 00:02 |

### Description

Create /api/charts/{chart_id}/feed endpoint streaming ECharts options via PatchSignals.

### Dependencies

- 🔗 **parent-child**: `ironstar-753`
- ⛔ **blocks**: `ironstar-2nt.14`
- ⛔ **blocks**: `ironstar-nyp.45`

---

<a id="ironstar-2nt-9-define-chartsignals-and-chartselection-types-with-ts-rs"></a>

## 📋 ironstar-2nt.9 Define ChartSignals and ChartSelection types with ts-rs

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 00:54 |
| **Updated** | 2026-01-17 00:33 |
| **Closed** | 2026-01-17 00:33 |

### Description

Create signal types for ECharts integration per signal-contracts.md. ChartSignals contains chartOption (serde_json::Value), selected (Option ChartSelection), loading (bool). ChartSelection contains seriesName, dataIndex, name, value. Use serde rename camelCase for JSON compatibility.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.2`

---

<a id="ironstar-961-implement-duckdb-connection-lifecycle-management"></a>

## 📋 ironstar-961 Implement DuckDB connection lifecycle management

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 00:43 |
| **Updated** | 2026-01-20 23:32 |
| **Closed** | 2026-01-20 23:32 |

### Description

Configure async-duckdb::PoolBuilder with database path and connection count (recommend 4 connections). Initialize Pool at application startup in main.rs, store in AppState. Pool manages connection lifecycle automatically - connections are checked out via .conn() and returned when closure completes. Ensure Cargo.toml uses async-duckdb with bundled feature to avoid system DuckDB version mismatches. Do NOT use spawn_blocking - async-duckdb provides native async API.

See docs/notes/architecture/cqrs/projection-patterns.md for Pool initialization patterns.

Local refs: ~/projects/rust-workspace/async-duckdb

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- 🔗 **child**: `ironstar-3gd`

---

<a id="ironstar-9b1-implement-httpfs-extension-configuration-for-duckdb"></a>

## 📋 ironstar-9b1 Implement httpfs extension configuration for DuckDB

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 00:43 |
| **Updated** | 2026-01-21 11:56 |
| **Closed** | 2026-01-21 11:56 |

### Description

Configure DuckDB with httpfs extension enabled. Add HuggingFace (hf://) and S3 (s3://) protocol support. Create configuration patterns for HuggingFace authentication tokens. Reference: ~/projects/rust-workspace/rust-duckdb-huggingface-ducklake-query

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- 🔗 **child**: `ironstar-3gd`

---

<a id="ironstar-3gd-scientific-data-integration"></a>

## 🚀 ironstar-3gd Scientific Data Integration

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 00:42 |
| **Updated** | 2026-02-02 20:24 |
| **Closed** | 2026-02-02 20:24 |

### Description

Implement ironstar's READ-ONLY DuckDB analytics layer for querying external scientific datasets. DuckDB is a dedicated OLAP query interface—completely separate from the SQLite event store—enabling efficient analysis of large scientific data without impacting event sourcing durability. Covers DuckDB integration for analytics queries, remote data source support via httpfs extension (HuggingFace datasets, S3-compatible storage, DuckLake), results caching with moka + rkyv for visualization backends, and SSE/datastar integration for dashboard updates. See docs/notes/architecture/infrastructure/analytics-cache-architecture.md and analytics-cache-patterns.md.

### Dependencies

- ⛔ **blocks**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt`

---

<a id="ironstar-753-5-implement-ds-echarts-build-and-test-integration"></a>

## 📋 ironstar-753.5 Implement ds-echarts build and test integration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2026-01-22 00:28 |
| **Closed** | 2026-01-22 00:28 |

### Description

Build pipeline integration, testing strategies. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/frontend/ds-echarts-build-test.md

### Dependencies

- 🔗 **parent-child**: `ironstar-753`
- ⛔ **blocks**: `ironstar-753.4`

### Comments

> **Cameron Smith** (2026-01-22)
>
> Implementation notes: Adapted ds-echarts Lit component from northstar. Key changes: (1) Light DOM comment updated for Open Props CSS token inheritance, (2) Uses pnpm not npm for nix compatibility. Build produces ~2.5MB bundle with ECharts. 56 Vitest tests cover lifecycle events, resize handling, theme changes, and error cases.

---

<a id="ironstar-753-4-implement-ds-echarts-backend-support"></a>

## 📋 ironstar-753.4 Implement ds-echarts backend support

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2026-01-21 18:48 |
| **Closed** | 2026-01-21 18:48 |

### Description

Server-side data preparation, SSE streaming for ECharts. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/frontend/ds-echarts-backend.md

### Dependencies

- 🔗 **parent-child**: `ironstar-753`
- ⛔ **blocks**: `ironstar-nyp.8`
- ⛔ **blocks**: `ironstar-nyp.12`

---

<a id="ironstar-c7z-implement-duckdb-remote-data-source-integration-ducklake-hf-pattern"></a>

## 📋 ironstar-c7z Implement DuckDB remote data source integration (DuckLake/HF pattern)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 17:42 |
| **Updated** | 2026-01-21 11:56 |
| **Closed** | 2026-01-21 11:56 |

### Description

Extend DuckDB analytics service to query remote data sources via DuckLake catalogs.

## Query pattern

Once attached, query tables directly:
```sql
ATTACH 'ducklake:...' AS space;
SELECT * FROM space.main.astronauts ORDER BY total_space_days DESC;
```

DuckLake handles catalog→parquet mapping transparently via httpfs.

## Embedded catalogs (recommended for demos)

Embed DuckLake catalogs (~5MB SQLite) in binary via rust_embed to eliminate ~2s ATTACH latency.
Extract to temp file at startup, ATTACH locally.
Cache keys use build-time versioning (CARGO_PKG_VERSION).

## Runtime catalogs (user-provided)

ATTACH from hf:// at runtime for user-specified datasets.
Cache keys use ducklake_current_snapshot() for versioning.

## Protocols supported

- ducklake: for DuckLake catalog attachment
- hf:// for HuggingFace-hosted parquet data
- s3:// for S3-compatible storage

## Canonical test dataset

sciexp-fixtures: hf://datasets/sciexp/fixtures/lakes/frozen/space.db

## Local refs

- ~/projects/omicslake-workspace/sciexp-fixtures
- ~/projects/lakescope-workspace/ducklake
- docs/notes/architecture/infrastructure/analytics-cache-patterns.md

### Dependencies

- ⛔ **blocks**: `ironstar-nyp.12`
- 🔗 **parent-child**: `ironstar-3gd`
- ⛔ **blocks**: `ironstar-9b1`
- 🔗 **child**: `ironstar-3gd`

### Comments

> **crs58** (2025-12-29)
>
> Session 2025-12-28: Validated DuckLake API pattern. Key discovery: use ducklake_current_snapshot() function, not __ducklake_metadata_* tables. Documentation added to analytics-cache-patterns.md. sciexp-fixtures and ducklake refs added to CLAUDE.md.

> **crs58** (2025-12-29)
>
> Session 2025-12-28 (cont): Refined understanding — embed DuckLake catalogs via rust_embed to eliminate ATTACH latency. Cache keys use build-time versioning for embedded catalogs. Created ironstar-3gd.4 for implementation.

---

<a id="ironstar-09r-implement-ds-echarts-lit-web-component-wrapper"></a>

## 📋 ironstar-09r Implement ds-echarts Lit web component wrapper

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 17:11 |
| **Updated** | 2026-02-03 00:07 |
| **Closed** | 2026-02-03 00:07 |

### Description

Implement ds-echarts Lit web component wrapper as a Moore machine coalgebra with event-driven state transitions.

Coalgebra structure:
- State: { option: ChartConfig, theme: string, chart: ECharts | null }
- Output: render() → canvas/SVG DOM
- Transition: updated() lifecycle + event handlers

Bisimulation equivalence ensures morphing safety: two states are behaviorally equivalent if they produce the same output and transition to equivalent states on all inputs.

Reference: denotational-semantics.md 'Web components as coalgebras' section.

### Dependencies

- ⛔ **blocks**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-r62.5`
- ⛔ **blocks**: `ironstar-2nt.5`
- ⛔ **blocks**: `ironstar-e6k.7`
- ⛔ **blocks**: `ironstar-r62.10`
- ⛔ **blocks**: `ironstar-2nt.9`

### Comments

> **Cameron Smith** (2026-02-03)
>
> Dependency correction: this issue depended on ironstar-ny3 (epic) but actual prerequisites were ny3.8, ny3.14, ny3.15 (all now closed). The remaining ny3 children (CSS theming: ny3.6, ny3.16, ny3.17; build investigation: ny3.22) are not prerequisites for ds-echarts.

---

<a id="ironstar-r62-9-create-base-layout-template-with-datastar-initialization"></a>

## 📋 ironstar-r62.9 Create base layout template with Datastar initialization

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 17:41 |
| **Closed** | 2026-01-19 17:41 |

### Description

Implement base_layout() function using hypertext::maud! that renders html > head > body with conditional hotreload div (data-init for dev mode), CSS link to bundle.[hash].css, and JS script for datastar.js. Establishes HTML structure for all pages.
Local refs: ~/projects/rust-workspace/hypertext, ~/projects/lakescope-workspace/datastar-go-nats-template-northstar

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.8`
- ⛔ **blocks**: `ironstar-ny3.13`

---

<a id="ironstar-r62-7-implement-query-get-handlers"></a>

## 📋 ironstar-r62.7 Implement query GET handlers

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-20 17:21 |
| **Closed** | 2026-01-20 17:21 |

### Description

Create GET handlers that call query handler (reads from projections), render hypertext template, and return as HTML or JSON. No event persistence, just read path. Handlers use State extractor to access AppState containing projections.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/hypertext

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.4`
- ⛔ **blocks**: `ironstar-a9b.8`

### Comments

> **Cameron Smith** (2026-01-20)
>
> Verified: list_todos and get_todo handlers in presentation/todo.rs with full test coverage.

---

<a id="ironstar-nyp-12-implement-duckdb-analytics-service"></a>

## 📋 ironstar-nyp.12 Implement DuckDB analytics service

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-20 23:51 |
| **Closed** | 2026-01-20 23:51 |

### Description

Implement DuckDBService wrapper around async-duckdb::Pool. Use PoolBuilder to configure connection count and database path. Wrap Pool in Arc for sharing across handlers. Expose query methods that use pool.conn(|conn| { ... }).await pattern for non-blocking analytics queries. Do NOT use spawn_blocking or block_in_place - async-duckdb handles threading internally via dedicated background threads.

See docs/notes/architecture/cqrs/projection-patterns.md (DuckDB analytics integration section) and docs/notes/architecture/infrastructure/analytics-cache-patterns.md for implementation patterns.

Local refs: ~/projects/rust-workspace/async-duckdb

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`
- ⛔ **blocks**: `ironstar-961`
- 🔗 **parent-child**: `ironstar-3gd`

---

<a id="ironstar-nyp-8-implement-sse-15-second-keep-alive-comment-stream"></a>

## 📋 ironstar-nyp.8 Implement SSE 15-second keep-alive comment stream

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 13:50 |
| **Closed** | 2026-01-19 13:50 |

### Description

Implement SSE 15-second keep-alive comment stream

Critical: Enforce subscribe-before-replay invariant - subscribe to broadcast BEFORE loading historical events.
See sse-connection-lifecycle.md Critical Invariant section.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.5`
- ⛔ **blocks**: `ironstar-nyp.27`

---

<a id="ironstar-nyp-6-create-projection-trait-for-read-models"></a>

## 📋 ironstar-nyp.6 Create Projection trait for read models

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-06 13:46 |
| **Closed** | 2026-01-06 13:46 |

### Description

Create Projection trait for read models implementing the Galois connection abstract/concrete pair and quotient monoid structure.

The trait defines the catamorphism algebra for state reconstruction:
- apply(&mut self, event: &Event): the fold step (catamorphism algebra)
- rebuild(): reconstruct from events (unique fold from initiality)

Projections form a Galois connection with the event log:
- abstract: EventLog → ReadModel (lossy, many-to-one)
- concrete: ReadModel → EventLog (partial reconstruction)

Reference: denotational-semantics.md sections on Galois connection and quotients.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`

---

<a id="ironstar-nyp-2-create-eventstore-trait-abstraction"></a>

## 📋 ironstar-nyp.2 Create EventStore trait abstraction

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-05 11:10 |
| **Closed** | 2026-01-05 11:10 |

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

---

<a id="ironstar-nyp-1-create-database-migrations-directory-with-schema-sql"></a>

## 📋 ironstar-nyp.1 Create database migrations/ directory with schema.sql

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-06 13:46 |
| **Closed** | 2026-01-06 13:46 |

### Description

Create database migrations/ directory with schema.sql

Include both:
- global_sequence (PRIMARY KEY AUTOINCREMENT) for SSE Last-Event-ID
- aggregate_sequence (UNIQUE per aggregate_type, aggregate_id) for optimistic locking

See event-sourcing-core.md hybrid schema section.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.11`
- ⛔ **blocks**: `ironstar-nyp.35`

---

<a id="ironstar-ny3-13-implement-rust-embed-conditional-asset-serving"></a>

## 📋 ironstar-ny3.13 Implement rust-embed conditional asset serving

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 14:56 |
| **Closed** | 2026-01-19 14:56 |

### Description

Create dual-mode asset serving: dev mode serves from filesystem via tower-http::ServeDir with no-store cache headers; prod mode embeds static/dist/ via rust-embed with immutable cache headers. Include AssetManifest loader for hashed filename resolution.
Local refs: ~/projects/rust-workspace/rust-embed

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.12`

---

<a id="ironstar-ny3-12-implement-manifest-json-parser-for-hashed-filename-resolution"></a>

## 📋 ironstar-ny3.12 Implement manifest.json parser for hashed filename resolution

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 14:47 |
| **Closed** | 2026-01-19 14:47 |

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

---

<a id="ironstar-ny3-11-create-static-dist-output-directory-structure"></a>

## 📋 ironstar-ny3.11 Create static/dist/ output directory structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 14:40 |
| **Closed** | 2026-01-19 14:40 |

### Description

Initialize static/dist/ directory placeholder for Rolldown build outputs (bundle.[hash].css, bundle.[hash].js, manifest.json). Create static/datastar/ for runtime datastar.js. Aligns with single-binary asset embedding in production.

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.2`

---

<a id="ironstar-ny3-10-configure-ts-rs-export-directory-and-justfile-task"></a>

## 📋 ironstar-ny3.10 Configure ts-rs export directory and justfile task

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-02-03 00:05 |
| **Closed** | 2026-02-03 00:05 |

### Description

Set TS_RS_EXPORT_DIR to place generated TypeScript bindings in the frontend source tree.

Configuration:
- Add to .cargo/config.toml:
  ```toml
  [env]
  TS_RS_EXPORT_DIR = { value = "web-components/bindings", relative = true }
  ```
- Create justfile task:
  ```just
  gen-types:
    cargo test --workspace  # ts-rs exports on test run
  ```

The `bindings/` name aligns with:
- ts-rs default output directory name
- Existing gitignore pattern `**/bindings/`
- Clear semantic meaning (generated bindings, not handwritten types)

Output structure:
```
web-components/bindings/
├── commands/
│   └── TodoCommand.ts
├── domain/
│   ├── TodoId.ts
│   └── TodoText.ts
└── events/
    └── TodoEvent.ts
```

Frontend TypeScript can import via:
```typescript
import type { TodoEvent } from './bindings/events/TodoEvent';
import type { TodoCommand } from './bindings/commands/TodoCommand';
```

Local refs: ~/projects/rust-workspace/ts-rs

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.9`
- ⛔ **blocks**: `ironstar-2nt.5`

---

<a id="ironstar-ny3-9-add-ts-rs-dependency-to-cargo-toml"></a>

## 📋 ironstar-ny3.9 Add ts-rs dependency to Cargo.toml

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-02-03 00:02 |
| **Closed** | 2026-02-03 00:02 |

### Description

Add ts-rs 11.1+ with features serde-compat and uuid-impl. Enables deriving TS traits on Rust types to generate TypeScript definitions. Ensures frontend and backend signal contracts stay synchronized.
Local refs: ~/projects/rust-workspace/ts-rs

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`

---

<a id="ironstar-ny3-8-create-web-components-index-ts-entry-point"></a>

## 📋 ironstar-ny3.8 Create web-components/index.ts entry point

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-02-03 00:02 |
| **Closed** | 2026-02-03 00:02 |

### Description

Create index.ts that imports main.css (processed by PostCSS plugin) and auto-registers vanilla web components by importing from components/ subdirectory. Export TypeScript types from web-components/types/ for frontend type safety.
Local refs: ~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/index.ts

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.7`

---

<a id="ironstar-ny3-7-create-typescript-configuration-tsconfig-json"></a>

## 📋 ironstar-ny3.7 Create TypeScript configuration (tsconfig.json)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-22 00:22 |
| **Closed** | 2026-01-22 00:22 |

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

### Comments

> **Cameron Smith** (2026-01-22)
>
> Implementation notes: Standalone tsconfig.json (not extending root) with Lit-specific settings: experimentalDecorators=true, useDefineForClassFields=false. Include pattern is explicit: *.ts, components/**/*.ts, __tests__/**/*.ts. Added @types/node for rolldown.config.ts Node.js imports.

---

<a id="ironstar-ny3-5-configure-css-cascade-layers-for-predictable-specificity"></a>

## 📋 ironstar-ny3.5 Configure CSS cascade layers for predictable specificity

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 18:08 |
| **Closed** | 2026-01-19 18:08 |

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

### Comments

> **crs58** (2026-01-06)
>
> Note: Layer structure will be updated from 6 to 7 layers to include compositions layer. See ironstar-ny3.18 for composition layer implementation.

---

<a id="ironstar-ny3-4-setup-open-props-design-tokens-and-theme-layer"></a>

## 📋 ironstar-ny3.4 Setup Open Props design tokens and theme layer

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 17:52 |
| **Closed** | 2026-01-19 17:52 |

### Description

Create web-components/styles/main.css importing Open Props design tokens. Create web-components/styles/theme.css with CSS custom properties using light-dark() function for automatic dark mode. Establish CSS cascade layers: openprops, normalize, theme, components, utilities, app.
Local refs: ~/projects/lakescope-workspace/open-props, ~/projects/lakescope-workspace/open-props-ui

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.3`

---

<a id="ironstar-ny3-3-setup-postcss-configuration-for-modern-css-features"></a>

## 📋 ironstar-ny3.3 Setup PostCSS configuration for modern CSS features

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 17:27 |
| **Closed** | 2026-01-19 17:27 |

### Description

Create web-components/postcss.config.js with plugins: postcss-import, postcss-preset-env (stage 0 for OKLch/light-dark/custom-media), autoprefixer, cssnano. Enables Open Props and modern CSS features.
Local refs: ~/projects/lakescope-workspace/open-props/

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.1`

---

<a id="ironstar-ny3-2-configure-rolldown-bundler-with-content-based-hashing"></a>

## 📋 ironstar-ny3.2 Configure Rolldown bundler with content-based hashing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 14:40 |
| **Closed** | 2026-01-19 14:40 |

### Description

Create rolldown.config.ts with content-based hashing ([name].[hash].js). Use Rolldown native CSS bundling (experimental). Add rollup-plugin-output-manifest for manifest.json generation. Output to ../static/dist/. Does NOT include PostCSS plugin config (ny3.3 adds that). Update package.json scripts to use config.

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.1`

---

<a id="ironstar-ny3-1-create-web-components-project-structure-with-package-json"></a>

## 📋 ironstar-ny3.1 Create web-components/ project structure with package.json

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 14:35 |
| **Closed** | 2026-01-19 14:35 |

### Description

Create web-components/ directory structure with package.json (type:module, open-props deps, rolldown devDeps, dev/build script stubs). Create stub index.ts entry point and styles/ directory. Does NOT include tsconfig.json (ny3.7), PostCSS config (ny3.3), or Rolldown config (ny3.2).

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`

---

<a id="ironstar-2nt-7-implement-command-validation-pattern-with-result-types"></a>

## 📋 ironstar-2nt.7 Implement command validation pattern with Result types

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-27 22:26 |
| **Closed** | 2025-12-27 22:26 |

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

---

<a id="ironstar-2nt-6-enforce-camelcase-convention-for-datastar-signal-fields"></a>

## 📋 ironstar-2nt.6 Enforce camelCase convention for Datastar signal fields

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-16 23:57 |
| **Closed** | 2026-01-16 23:57 |

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

---

<a id="ironstar-2nt-5-create-datastar-signal-types-with-ts-rs-derives"></a>

## 📋 ironstar-2nt.5 Create Datastar signal types with ts-rs derives

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-16 23:56 |
| **Closed** | 2026-01-16 23:56 |

### Description

Create Datastar signal types with ts-rs derives supporting comonadic extend/extract operations for computed signal derivation.

Signals form a comonad (dual to server-side monads):
- extract: Signal a → a (get current value via $signal.value)
- extend: (Signal a → b) → Signal a → Signal b (computed(() => ...))

The comonad laws ensure signal composition is well-behaved:
- extend extract = id
- extract ∘ extend f = f

Reference: denotational-semantics.md 'Client signals as comonad' section.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.2`

---

<a id="ironstar-2nt-4-design-aggregate-root-state-machines"></a>

## 📋 ironstar-2nt.4 Design aggregate root state machines

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-27 22:26 |
| **Closed** | 2025-12-27 22:26 |

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

---

<a id="ironstar-2nt-3-implement-value-objects-and-smart-constructors"></a>

## 📋 ironstar-2nt.3 Implement value objects and smart constructors

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-27 22:26 |
| **Closed** | 2025-12-27 22:26 |

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

---

<a id="ironstar-2nt-2-define-algebraic-domain-types-and-aggregate-structure"></a>

## 📋 ironstar-2nt.2 Define algebraic domain types and aggregate structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-27 21:37 |
| **Closed** | 2025-12-27 21:37 |

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

---

<a id="ironstar-2nt-1-initialize-src-directory-structure-with-modular-organization"></a>

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

<a id="ironstar-2nt-domain-layer"></a>

## 🚀 ironstar-2nt Domain layer

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-17 22:51 |
| **Closed** | 2026-01-17 22:51 |

### Description

Domain layer: Algebraic types, aggregates, value objects, state machines

**Strategic classification:**
- QuerySession: Core domain (differentiating capability)
- Todo: Generic example domain (demonstration pattern)

**Discovery grounding:**
- EventStorming: Yellow stickies → Aggregate modules
- EventStorming: Orange stickies → Event enum variants
- EventStorming: Blue stickies → Command enum variants

**Algebraic foundation:**
- semantic-model.md § Aggregate interpretation as coalgebra
- Catamorphism: fold_events reconstructs state from event stream
- Commands as Kleisli arrows in Result monad

**Hoffman Laws implemented:**
- Law 1: Events immutable (past-tense, append-only)
- Law 7: Work is side effect (pure aggregates, I/O at boundaries)
- Law 10: Aggregates own event streams

### Dependencies

- ⛔ **blocks**: `ironstar-6lq.5`

---

<a id="ironstar-6lq-7-add-rust-to-ci-matrix-and-extend-inherited-workflows"></a>

## 📋 ironstar-6lq.7 Add Rust to CI matrix and extend inherited workflows

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-27 18:19 |
| **Closed** | 2025-12-27 18:19 |

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

---

<a id="ironstar-6lq-6-add-rust-checks-to-flake-checks-for-ci-integration"></a>

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

<a id="ironstar-6lq-5-verify-cargo-check-passes-with-workspace-configuration"></a>

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

<a id="ironstar-6lq-4-set-up-per-crate-crate-nix-pattern-for-crane-args"></a>

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

<a id="ironstar-6lq-3-configure-cargo-toml-with-workspace-structure-resolver-2"></a>

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

<a id="ironstar-6lq-2-add-rust-toolchain-toml-with-required-components"></a>

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

<a id="ironstar-6lq-1-integrate-rust-flake-patterns-crane-rust-overlay"></a>

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

<a id="ironstar-6lq-rust-workspace-integration"></a>

## 🚀 ironstar-6lq Rust workspace integration

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-27 18:20 |
| **Closed** | 2025-12-27 18:20 |

### Description

Integrate Rust toolchain and workspace patterns into the Nix flake using rust-flake, crane for deterministic builds, and rust-overlay for toolchain management. Establishes Cargo workspace structure with resolver 2, workspace.dependencies for DRY, per-crate crane.args configuration following rustlings-workspace and rust-nix-template patterns. Includes CI integration with flake checks and GitHub Actions matrix builds inherited from template.

### Dependencies

- ⛔ **blocks**: `ironstar-cxe`

---

<a id="ironstar-cxe-5-create-gitignore-with-comprehensive-patterns"></a>

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

<a id="ironstar-cxe-4-create-initial-git-commit-with-generated-structure"></a>

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

<a id="ironstar-cxe-3-verify-nix-develop-enters-working-development-shell"></a>

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

<a id="ironstar-cxe-2-configure-secrets-management-and-string-replacement"></a>

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

<a id="ironstar-cxe-1-run-om-init-with-typescript-nix-template-parameters"></a>

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

<a id="ironstar-cxe-template-instantiation"></a>

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

<a id="ironstar-o66-6-remove-rust-flake-and-add-per-crate-crane-derivations"></a>

## 📋 ironstar-o66.6 Remove rust-flake and add per-crate crane derivations

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 22:37 |
| **Updated** | 2026-02-04 22:44 |
| **Closed** | 2026-02-04 22:44 |

### Description

Remove rust-flake dependency from flake.nix. Replace with direct crane input and inline definitions for crane-lib, rustToolchain, src filtering, and nixpkgs overlay in modules/rust.nix. Add programmatic per-crate test derivations (cargoNextest with -p flag) sharing the workspace cargoArtifacts. Update toolchain references in dev-shell.nix, formatting.nix, nix-unit.nix. Delete crates/ironstar/crate.nix. Run nix flake lock to update lockfile. Verify with nix flake show and nix develop -c just rust-check-full.

### Dependencies

- 🔗 **parent-child**: `ironstar-o66`

---

<a id="ironstar-o66-3-skip-per-crate-doc-derivation-builds-in-ci"></a>

## 📋 ironstar-o66.3 Skip per-crate doc derivation builds in CI

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 21:51 |
| **Updated** | 2026-02-04 22:37 |
| **Closed** | 2026-02-04 22:37 |

### Description

10 per-crate -doc derivations are built by CI packages job but serve no purpose for PR validation. Either disable doc from per-crate autoWire or filter -doc derivations in ci-build-category.sh. Prefer autoWire configuration. Reference: nix flake show output showing ironstar-*-doc packages.

### Dependencies

- 🔗 **parent-child**: `ironstar-o66`
- ⛔ **blocks**: `ironstar-o66.1`

---

<a id="ironstar-o66-2-remove-redundant-per-crate-clippy-checks"></a>

## 📋 ironstar-o66.2 Remove redundant per-crate clippy checks

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 21:51 |
| **Updated** | 2026-02-04 22:37 |
| **Closed** | 2026-02-04 22:37 |

### Description

nix flake show reveals 10 per-crate clippy checks (ironstar-*-clippy) alongside workspace-clippy in the checks output. These are redundant — workspace-clippy already runs clippy on all workspace crates. Either disable clippy from per-crate autoWire or filter them out in ci-build-category.sh. Prefer disabling at the autoWire level so the derivation set stays clean. Reference: modules/rust.nix:152-158 (workspace-clippy), ci-build-category.sh:135-174 (checks iteration).

### Dependencies

- 🔗 **parent-child**: `ironstar-o66`
- ⛔ **blocks**: `ironstar-o66.1`

---

<a id="ironstar-o66-1-audit-rust-flake-autowire-settings-per-library-crate"></a>

## 📋 ironstar-o66.1 Audit rust-flake autoWire settings per library crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 21:51 |
| **Updated** | 2026-02-04 22:37 |
| **Closed** | 2026-02-04 22:37 |

### Description

Each of the 10 library crates under crates/ gets autoWire defaults from rust-flake, producing build + doc + clippy derivations per crate. Audit what crate.nix files exist (or are auto-discovered) and configure autoWire to only produce what CI actually needs. The binary crate (crates/ironstar/crate.nix) already has autoWire = []. Reference: modules/rust.nix for crane configuration, nix flake show for current derivation tree.

### Dependencies

- 🔗 **parent-child**: `ironstar-o66`

---

<a id="ironstar-o66-optimize-ci-build-derivation-set-for-multi-crate-workspace"></a>

## 🚀 ironstar-o66 Optimize CI build derivation set for multi-crate workspace

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 21:51 |
| **Updated** | 2026-02-05 00:59 |
| **Closed** | 2026-02-05 00:59 |

### Description

Remove rust-flake dependency and replace with direct crane usage. The 29b crate decomposition caused rust-flake autoWire to produce 30 unintended per-crate derivations (10 builds + 10 docs + 10 clippy), each with separate cargoArtifacts. Solution: remove rust-flake entirely, inline the 4 things it provided (crane-lib, toolchain, src filtering, nixpkgs overlay), add programmatic per-crate test derivations sharing a single workspace cargoArtifacts, and delete crate.nix. Target: clean derivation set, shared dep caching, per-crate test isolation.

---

<a id="ironstar-29b-12-configure-workspace-and-nix-integration"></a>

## 📋 ironstar-29b.12 Configure workspace and Nix integration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 18:56 |
| **Closed** | 2026-02-04 18:56 |

### Description

Update root Cargo.toml workspace members and add [workspace.lints]. Add [lints] workspace = true to each crate. Update flake.nix with per-crate derivations using fileSetForCrate pattern from nix-cargo-crane quick-start-workspace. Add crate.nix files where needed. Evaluate cargo-hakari workspace hack crate for dependency divergence.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.11`

---

<a id="ironstar-29b-11-collapse-monolith-to-binary-crate"></a>

## 📋 ironstar-29b.11 Collapse monolith to binary crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 18:50 |
| **Closed** | 2026-02-04 18:50 |

### Description

Reduce crates/ironstar/ to binary crate containing: main.rs, AppState, config, axum routes, SSE endpoints, extractors, hypertext templates, datastar-rust integration, application layer command/query handlers, All composition root, HasXxx trait implementations. Remove empty domain/ and infrastructure/ directories. Verify cargo check --workspace and cargo test --workspace pass.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.3`
- ⛔ **blocks**: `ironstar-29b.4`
- ⛔ **blocks**: `ironstar-29b.5`
- ⛔ **blocks**: `ironstar-29b.6`
- ⛔ **blocks**: `ironstar-29b.7`
- ⛔ **blocks**: `ironstar-29b.8`
- ⛔ **blocks**: `ironstar-29b.9`
- ⛔ **blocks**: `ironstar-29b.10`

---

<a id="ironstar-29b-10-extract-ironstar-session-store-crate"></a>

## 📋 ironstar-29b.10 Extract ironstar-session-store crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 18:03 |
| **Closed** | 2026-02-04 18:03 |

### Description

Create crates/ironstar-session-store/ with SqliteSessionStore, TTL cleanup. Moves infrastructure/session_store.rs. Depends on ironstar-core, ironstar-session, sqlx.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.1`
- ⛔ **blocks**: `ironstar-29b.4`

---

<a id="ironstar-29b-9-extract-ironstar-analytics-infra-crate"></a>

## 📋 ironstar-29b.9 Extract ironstar-analytics-infra crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 18:42 |
| **Closed** | 2026-02-04 18:42 |

### Description

Create crates/ironstar-analytics-infra/ with DuckDBService, AnalyticsCache (moka), CachedAnalyticsService, CacheInvalidationRegistry, CacheDependency, EmbeddedCatalogs. Moves infrastructure/analytics.rs, analytics_cache.rs, cached_analytics.rs, cache_invalidation.rs, cache_dependency.rs, embedded_catalogs.rs. Depends on ironstar-core, ironstar-analytics, async-duckdb, moka, rkyv, zenoh.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.1`
- ⛔ **blocks**: `ironstar-29b.5`

---

<a id="ironstar-29b-8-extract-ironstar-event-bus-crate"></a>

## 📋 ironstar-29b.8 Extract ironstar-event-bus crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 18:28 |
| **Closed** | 2026-02-04 18:28 |

### Description

Create crates/ironstar-event-bus/ with ZenohEventBus, EventBus trait implementation, key expression utilities, workspace subscriber factory. Moves infrastructure/event_bus/ and infrastructure/key_expr.rs. Depends on ironstar-core and zenoh.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.1`

### Comments

> **Cameron Smith** (2026-02-04)
>
> Checkpoint: 2026-02-04 session
> 
> Done:
> - Issue explored and read (bd show), marked in_progress
> - Subagent explored all event_bus/ files, key_expr.rs, and consumers
> - Identified that port traits (EventBus, EventNotifier, EventSubscriber) land here per user's note about InfrastructureError/sqlx dependency
> 
> Remaining:
> - Create crates/ironstar-event-bus/ crate
> - Move infrastructure/event_bus/ module (ZenohEventBus, workspace subscriber factory)
> - Move infrastructure/key_expr.rs (key expression utilities)
> - Move port traits (EventBus, EventNotifier, EventSubscriber) from monolith
> - Define EventBusError as crate-specific error type
> - Update monolith re-exports and From conversions
> - Verify with cargo test (baseline 891 passing)
> 
> Learnings:
> - Per-crate error types (SessionStoreError, EventStoreError) work well as extraction pattern
> - From conversions needed in both InfrastructureError and CommandPipelineError
> - treefmt hook reformats import ordering on new crate files, stage after first attempt
> 
> Suggested next steps:
> - Read event_bus/ module files and key_expr.rs
> - Follow same extraction pattern as 29b.10 and 29b.7

---

<a id="ironstar-29b-7-extract-ironstar-event-store-crate"></a>

## 📋 ironstar-29b.7 Extract ironstar-event-store crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 18:13 |
| **Closed** | 2026-02-04 18:13 |

### Description

Create crates/ironstar-event-store/ with SqliteEventRepository implementing EventRepository from core, SSE stream composition. Moves infrastructure/event_store.rs and infrastructure/sse_stream.rs. Depends on ironstar-core and sqlx.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.1`

---

<a id="ironstar-29b-6-extract-ironstar-workspace-crate"></a>

## 📋 ironstar-29b.6 Extract ironstar-workspace crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 16:19 |
| **Closed** | 2026-02-04 16:19 |

### Description

Create crates/ironstar-workspace/ with 5 aggregates (WorkspaceAggregate, WorkspacePreferences, Dashboard, SavedQuery, UserPreferences), combined workspaceContextDecider. Moves domain/workspace/, domain/dashboard/, domain/saved_query/, domain/user_preferences/, domain/workspace_preferences/ and related views. Maps to spec/Workspace/*. Depends on ironstar-core, ironstar-shared-kernel, and ironstar-analytics (Customer-Supplier: Dashboard references ChartType, SavedQuery references DatasetRef).

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.1`
- ⛔ **blocks**: `ironstar-29b.2`
- ⛔ **blocks**: `ironstar-29b.5`

---

<a id="ironstar-29b-5-extract-ironstar-analytics-crate"></a>

## 📋 ironstar-29b.5 Extract ironstar-analytics crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 16:04 |
| **Closed** | 2026-02-04 16:04 |

### Description

Create crates/ironstar-analytics/ with Catalog aggregate, QuerySession aggregate, Chart value objects, combined analyticsDecider. Moves domain/analytics/, domain/catalog/, domain/query_session/ and related views. Maps to spec/Analytics/*. Depends on ironstar-core.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.1`

---

<a id="ironstar-29b-4-extract-ironstar-session-crate"></a>

## 📋 ironstar-29b.4 Extract ironstar-session crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 15:32 |
| **Closed** | 2026-02-04 15:32 |

### Description

Create crates/ironstar-session/ with Session aggregate (commands, events, state, decider, values, errors) and SessionView. Moves domain/session/. Maps to spec/Session/*. Depends on ironstar-core and ironstar-shared-kernel.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.1`
- ⛔ **blocks**: `ironstar-29b.2`

---

<a id="ironstar-29b-3-extract-ironstar-todo-crate"></a>

## 📋 ironstar-29b.3 Extract ironstar-todo crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 15:42 |
| **Closed** | 2026-02-04 15:42 |

### Description

Create crates/ironstar-todo/ with Todo aggregate (commands, events, state, decider, values, errors) and TodoListView. Moves domain/todo/ and domain/views/todo.rs. Maps to spec/Todo/*. Depends only on ironstar-core.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.1`

---

<a id="ironstar-29b-2-extract-ironstar-shared-kernel-crate"></a>

## 📋 ironstar-29b.2 Extract ironstar-shared-kernel crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 15:24 |
| **Closed** | 2026-02-04 15:24 |

### Description

Create crates/ironstar-shared-kernel/ with UserId, OAuthProvider. Maps to spec/SharedKernel/*. Depends on ironstar-core if UserId uses BoundedString.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`
- ⛔ **blocks**: `ironstar-29b.1`

---

<a id="ironstar-29b-1-extract-ironstar-core-crate"></a>

## 📋 ironstar-29b.1 Extract ironstar-core crate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:34 |
| **Updated** | 2026-02-04 15:13 |
| **Closed** | 2026-02-04 15:13 |

### Description

Create crates/ironstar-core/ with port traits (EventRepository, EventNotifier, EventSubscriber, EventBus), event infrastructure (EventEnvelope, EventId, Timestamp), domain traits (DeciderType, EventType, IsFinal), common value objects (BoundedString, validated newtypes), and domain error types (DomainError, ValidationError). Maps to spec/Core/*. Monolithic crate replaces internal definitions with use ironstar_core::* imports. Resolve open question: re-export fmodel-rust types or require direct dependency.

### Dependencies

- 🔗 **parent-child**: `ironstar-29b`

---

<a id="ironstar-29b-spec-aligned-crate-decomposition"></a>

## 🚀 ironstar-29b Spec-aligned crate decomposition

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 13:33 |
| **Updated** | 2026-02-04 18:56 |
| **Closed** | 2026-02-04 18:56 |

### Description

Decompose the monolithic crates/ironstar/ into 11 spec-aligned crates. Bounded-context-based decomposition where crate boundaries mirror the Idris2 spec/ module boundaries. See docs/notes/architecture/core/crate-decomposition-plan.md for full plan. Supersedes deleted epic ironstar-v4y (layer-based decomposition).

### Dependencies

- 🔗 **related**: `ironstar-9dh`

---

<a id="ironstar-nyp-49-implement-combined-analyticsdecider-composition"></a>

## 📋 ironstar-nyp.49 Implement combined analyticsDecider composition

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-02 20:34 |
| **Updated** | 2026-02-02 22:47 |
| **Closed** | 2026-02-02 22:47 |

### Description

Implement the combined Analytics Decider as specified in the Idris spec (Analytics/Analytics.idr).

The Idris spec defines:
  analyticsDecider = combine catalogDecider queryDecider

This composes the two independent Deciders into a single Decider with:
- Commands: Sum<CatalogCommand, QueryCommand> (via fmodel-rust Either or custom Sum type)
- State: (CatalogState, QuerySessionState) (product type)
- Events: Sum<CatalogEvent, QuerySessionEvent>
- Error: String (or unified AnalyticsError)

Also implement the combined AnalyticsReadModel View:
  analyticsView combining catalog metadata projection and query history projection

Create domain/analytics/combined.rs (or similar) with:
- analytics_decider() factory
- analytics_view() factory
- AnalyticsReadModel struct (catalog_metadata + query_history)

This is needed for coordinated command routing where a single endpoint can dispatch to either aggregate. May not be needed immediately if Catalog and QuerySession have separate endpoints, but matches the spec and enables future composition.

Evaluate whether fmodel-rust's combine function supports this or if a manual composition is needed. Reference the Decider combine pattern from fmodel-rust source.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.41`

---

<a id="ironstar-nyp-47-implement-analytics-query-handlers-read-side"></a>

## 📋 ironstar-nyp.47 Implement Analytics query handlers (read-side)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-02 20:34 |
| **Updated** | 2026-02-02 23:22 |
| **Closed** | 2026-02-02 23:22 |

### Description

Create analytics query handlers following the query_all_todos / query_todo_state pattern.

Create application/query_session/queries.rs and application/catalog/queries.rs with:

QuerySession queries:
- query_session_state: replay QuerySession events through View to get current state
- query_query_history: replay through queryHistoryView to get completed/failed query lists

Catalog queries:
- query_catalog_state: replay Catalog events to get current CatalogState (NoCatalogSelected or CatalogActive)
- query_catalog_metadata: extract CatalogMetadata from CatalogViewState

All queries use compute-on-demand pattern: fetch events from repository, fold through View, return projected state. No persistent projections yet.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.42`
- ⛔ **blocks**: `ironstar-nyp.43`

---

<a id="ironstar-nyp-46-implement-catalog-command-handler-with-zenoh-publishing"></a>

## 📋 ironstar-nyp.46 Implement Catalog command handler with Zenoh publishing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-02 20:33 |
| **Updated** | 2026-02-02 23:11 |
| **Closed** | 2026-02-02 23:11 |

### Description

Create handle_catalog_command following the handle_todo_command pattern.

In application/catalog/handlers.rs add:
- handle_catalog_command<B: EventBus>: generic version for testing
- handle_catalog_command_zenoh: concrete version for axum handlers

Pattern mirrors QuerySession handler but for Catalog aggregate. Publishes on Zenoh key 'events/Analytics/Catalog/{id}'.

RefreshCatalogMetadata requires an effect boundary call to DuckDB (via CachedAnalyticsService) to fetch actual metadata. The handler orchestrates: persist CatalogMetadataRefreshed event, then invalidate relevant cache entries via Zenoh.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.44`

---

<a id="ironstar-nyp-42-implement-catalog-view-read-model-projection"></a>

## 📋 ironstar-nyp.42 Implement Catalog View read model projection

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-02 20:32 |
| **Updated** | 2026-02-02 23:15 |
| **Closed** | 2026-02-02 23:15 |

### Description

Implement CatalogView following the TodoView pattern. The Idris spec defines analyticsView combining CatalogMetadata and QueryHistory projections.

Create domain/views/catalog.rs with:
- CatalogViewState: current catalog ref, metadata (datasets list, last refreshed)
- catalog_view() factory returning fmodel-rust View type
- Pure infallible evolve: (CatalogViewState, CatalogEvent) -> CatalogViewState

The View tracks the current active catalog and its metadata. CatalogSelected sets the ref with empty metadata. CatalogMetadataRefreshed updates the dataset list.

Tests should verify evolve is total and produces correct projections.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.41`

---

<a id="ironstar-ny3-20-fix-css-bundle-not-appearing-in-manifest-json"></a>

## 🐛 ironstar-ny3.20 Fix CSS bundle not appearing in manifest.json

| Property | Value |
|----------|-------|
| **Type** | 🐛 bug |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-19 20:40 |
| **Updated** | 2026-01-19 20:51 |
| **Closed** | 2026-01-19 20:51 |

### Description

Rolldown build produces hashed CSS (bundle.BtE3nzmj.css) but manifest.json only contains JS entries.

**Symptom:** bundle.css returns 503/404 because manifest.resolve falls back to unhashed name.

**Root cause:** Rolldown manifest plugin warns 'output file bundle.xxx.css has no related origin name, so omit it'

**Expected:** manifest.json should contain both:
- bundle.js → bundle.[hash].js
- bundle.css → bundle.[hash].css

**Discovered from:** Browser validation of e6k Todo example.

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- 🔗 **discovered-from**: `ironstar-e6k`

---

<a id="ironstar-507-session-aggregate-implementation"></a>

## 📋 ironstar-507 Session aggregate implementation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-16 12:39 |
| **Updated** | 2026-01-18 22:16 |
| **Closed** | 2026-01-18 22:16 |

### Description

Implement Session aggregate from spec/Session/Session.idr following fmodel-rust patterns established in ironstar-a9b.

## Context

Session is a Supporting domain that manages OAuth-based authentication lifecycle. It provides authenticated User identity to other bounded contexts (Workspace, Analytics) via the Shared Kernel pattern (UserId).

**Important**: This is the *Authentication* Session aggregate, distinct from QuerySession (Analytics query execution lifecycle) which was implemented in a9b.

## Implementation requirements

**Location**: `crates/ironstar/src/domain/session/`

**Module structure**:
```
session/
├── mod.rs              # Re-exports SessionCommand, SessionEvent, etc.
├── decider.rs          # session_decider() factory
├── commands.rs         # SessionCommand enum
├── events.rs           # SessionEvent enum
├── state.rs            # SessionState + SessionStatus
├── errors.rs           # SessionError + SessionErrorKind
└── values.rs           # SessionId, ExpiresAt, SessionMetadata, RevocationReason
```

**Commands** (from spec/Session/Session.idr):
- `CreateSession { user_id: UserId, provider: OAuthProvider, created_at: Timestamp, expires_at: ExpiresAt, metadata: SessionMetadata }`
- `RefreshSession { session_id: SessionId, refreshed_at: Timestamp, new_expires_at: ExpiresAt }`
- `InvalidateSession { session_id: SessionId, invalidated_at: Timestamp }`

**Events**:
- `SessionCreated { session_id: SessionId, user_id: UserId, provider: OAuthProvider, created_at: Timestamp, expires_at: ExpiresAt, metadata: SessionMetadata }`
- `SessionRefreshed { session_id: SessionId, new_expires_at: ExpiresAt, refreshed_at: Timestamp }`
- `SessionInvalidated { session_id: SessionId, invalidated_at: Timestamp }`
- `SessionExpired { session_id: SessionId, expired_at: Timestamp }` (generated by boundary layer TTL check)

**State**:
```rust
pub enum SessionState {
    NoSession,
    Active { session_id: SessionId, user_id: UserId, expires_at: ExpiresAt },
    Expired { session_id: SessionId },
    Invalidated { session_id: SessionId },
}
```

**Value objects**:
- `SessionId` - UUID wrapper with smart constructor
- `ExpiresAt` - Timestamp wrapper for TTL
- `SessionMetadata` - { ip_address, user_agent, geo_location } captured at boundary
- `RevocationReason` - enum { UserLogout, AdminAction, SecurityConcern }

**Boundary injection**: Commands carry timestamps and metadata injected at HTTP handler layer. The pure `decide()` function does not call `Utc::now()`. The Idris2 spec uses holes (`?newSid`, `?now`, `?expires`, `?metadata`) to mark these injection points.

**Error handling**: Return `Err(SessionError::already_active())` when creating duplicate session, `Err(SessionError::no_active_session())` when refreshing/invalidating non-existent session, etc.

## View

The spec defines `ActiveSessionView` for quick session lookup:
```rust
pub struct ActiveSessionViewState {
    pub active_session: Option<(SessionId, UserId, ExpiresAt)>,
}
```

This is a read-side projection for authentication checks.

## Acceptance criteria

- [ ] SessionCommand enum with Identifier + DeciderType traits
- [ ] SessionEvent enum with Identifier + EventType + DeciderType + IsFinal traits
- [ ] SessionState enum with helper methods (is_active, is_terminated, get_session_id, get_user_id)
- [ ] SessionError with factory constructors
- [ ] Value objects: SessionId, ExpiresAt, SessionMetadata, RevocationReason
- [ ] session_decider() factory function
- [ ] Pure decide() and evolve() functions
- [ ] ActiveSessionView projection
- [ ] All types export to TypeScript via ts-rs
- [ ] Decider specification tests (given/when/then)

## Aggregate ID pattern

`session_{session_id}` (e.g., `session_a1b2c3d4`)

## References

- spec/Session/Session.idr (formal specification)
- crates/ironstar/src/domain/todo/decider.rs (reference pattern)
- crates/ironstar/src/domain/query_session/ (sibling aggregate, different domain)
- docs/notes/architecture/decisions/oauth-authentication.md

### Dependencies

- 🔗 **parent-child**: `ironstar-jqv`

---

<a id="ironstar-7a2-2-implement-workspace-aggregate-with-visibility-control"></a>

## 📋 ironstar-7a2.2 Implement Workspace aggregate with visibility control

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-16 12:06 |
| **Updated** | 2026-01-18 22:26 |
| **Closed** | 2026-01-18 22:26 |

### Description

Implement WorkspaceAggregate Decider following fmodel-rust patterns from ironstar-a9b.

## Context

WorkspaceAggregate manages workspace lifecycle: creation, renaming, and visibility control. It is the root aggregate for the Workspace bounded context.

## Implementation requirements

**Location**: `crates/ironstar/src/domain/workspace/`

**Module structure**:
```
workspace/
├── mod.rs              # Re-exports WorkspaceCommand, WorkspaceEvent, etc.
├── decider.rs          # workspace_decider() factory
├── commands.rs         # WorkspaceCommand enum
├── events.rs           # WorkspaceEvent enum
├── state.rs            # WorkspaceState + WorkspaceStatus
├── errors.rs           # WorkspaceError + WorkspaceErrorKind
└── values.rs           # WorkspaceId, WorkspaceName, Visibility
```

**Commands** (from spec/Workspace/WorkspaceAggregate.idr):
- `CreateWorkspace { name: String, owner_id: UserId, visibility: Visibility, created_at: DateTime<Utc> }`
- `RenameWorkspace { workspace_id: WorkspaceId, new_name: String, renamed_at: DateTime<Utc> }`
- `SetVisibility { workspace_id: WorkspaceId, visibility: Visibility, changed_at: DateTime<Utc> }`

**Events**:
- `WorkspaceCreated { id: WorkspaceId, name: WorkspaceName, owner_id: UserId, visibility: Visibility, created_at: DateTime<Utc> }`
- `WorkspaceRenamed { id: WorkspaceId, old_name: WorkspaceName, new_name: WorkspaceName, renamed_at: DateTime<Utc> }`
- `VisibilityChanged { id: WorkspaceId, old_visibility: Visibility, new_visibility: Visibility, changed_at: DateTime<Utc> }`

**State**:
```rust
pub struct WorkspaceState {
    pub id: Option<WorkspaceId>,
    pub name: Option<WorkspaceName>,
    pub owner_id: Option<UserId>,
    pub visibility: Option<Visibility>,
    pub created_at: Option<DateTime<Utc>>,
    pub status: WorkspaceStatus,
}

pub enum WorkspaceStatus {
    NotCreated,
    Active,
}
```

**Boundary injection**: Commands carry `created_at`, `renamed_at`, `changed_at` timestamps injected at HTTP handler layer. The pure `decide()` function does not call `Utc::now()`.

**Error handling**: Return `Err(WorkspaceError::already_exists())` when creating duplicate, `Err(WorkspaceError::not_found())` when operating on non-existent workspace.

**Idempotency**: Renaming to same name or setting same visibility returns `Ok(vec![])`.

## Acceptance criteria

- [ ] WorkspaceCommand enum with Identifier + DeciderType traits
- [ ] WorkspaceEvent enum with Identifier + EventType + DeciderType + IsFinal traits
- [ ] WorkspaceState with status enum and helper methods
- [ ] WorkspaceError with factory constructors
- [ ] Value objects: WorkspaceId, WorkspaceName, Visibility
- [ ] workspace_decider() factory function
- [ ] Pure decide() and evolve() functions
- [ ] All types export to TypeScript via ts-rs

## References

- spec/Workspace/WorkspaceAggregate.idr (formal specification)
- crates/ironstar/src/domain/todo/decider.rs (reference pattern)

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.1`

---

<a id="ironstar-7a2-1-implement-user-and-userid-types-with-composite-key-pattern"></a>

## 📋 ironstar-7a2.1 Implement User and UserId types with composite key pattern

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-16 12:06 |
| **Updated** | 2026-01-18 22:19 |
| **Closed** | 2026-01-18 22:19 |

### Description

Import UserId from session module into workspace bounded context, verify shared kernel pattern works.

## Context

UserId was implemented in ironstar-507 as a UUID wrapper in `domain/session/values.rs`. This task verifies the shared kernel pattern: Workspace aggregates can import and use UserId from Session context.

## Scope change

Original description requested composite key implementation. Architecture docs (oauth-authentication.md) specify dual representation:
- Domain layer: UserId = UUID (canonical) — **already implemented in 507**
- Infrastructure layer: user_identities table handles (provider, external_id) → UUID lookup

The composite OAuthIdentity is infrastructure concern under jqv (auth epic), not a domain type.

## Implementation requirements

1. Verify `UserId` is re-exported from `domain/mod.rs`
2. Create `domain/workspace/` module structure
3. Import `UserId` in workspace aggregates
4. Ensure no circular dependencies between session and workspace

## Acceptance criteria

- [ ] Workspace module can import UserId from domain re-exports
- [ ] No compilation errors with shared kernel pattern
- [ ] UserId usage documented in workspace aggregate commands/events

## References

- ironstar-507: Session aggregate (UserId source)
- docs/notes/architecture/decisions/oauth-authentication.md

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-507`

### Comments

> **Cameron Smith** (2026-01-19)
>
> Scope revised: UserId (UUID) already exists from 507. This task now verifies shared kernel import pattern for Workspace context.

---

<a id="ironstar-7a2-implement-workspace-bounded-context-aggregates-in-rust"></a>

## 🚀 ironstar-7a2 Implement Workspace bounded context aggregates in Rust

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-16 12:06 |
| **Updated** | 2026-02-03 18:09 |
| **Closed** | 2026-02-03 18:09 |

### Description

Implement Rust types and fmodel-rust Deciders for Workspace bounded context following Idris2 spec and the canonical patterns established in ironstar-a9b.

## Context

The Workspace bounded context contains 5 aggregates composed via the Decider pattern:
- WorkspaceAggregate (identity, ownership, visibility)
- WorkspacePreferences (workspace-scoped settings)
- Dashboard (layout, tabs, chart placements)
- SavedQuery (named queries with dataset refs)
- UserPreferences (user-scoped settings)

This epic follows the patterns proven in ironstar-a9b (Todo domain) and documented in:
- `spec/Workspace/*.idr` (Idris2 formal specifications)
- `docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md`
- `crates/ironstar/src/domain/todo/` (reference implementation)

## Canonical patterns (from a9b)

**Module organization** (per aggregate):
```
domain/workspace/
├── mod.rs              # Re-exports
├── decider.rs          # Pure Decider factory + decide/evolve
├── commands.rs         # Command enum + Identifier + DeciderType
├── events.rs           # Event enum + Identifier + EventType + IsFinal
├── state.rs            # State type + status enum
├── errors.rs           # Error + ErrorKind + factory constructors
└── values.rs           # Value objects with smart constructors
```

**Decider factory pattern**:
```rust
pub fn workspace_decider<'a>() -> WorkspaceDecider<'a> {
    Decider {
        decide: Box::new(|cmd, state| decide(cmd, state)),
        evolve: Box::new(|state, event| evolve(state, event)),
        initial_state: Box::new(WorkspaceState::default),
    }
}
```

**Boundary injection**: Timestamps and UUIDs are injected at the HTTP handler layer, not generated inside pure `decide()` functions. The Idris2 spec uses holes (`?newWsId`, `?now`) to mark these boundary concerns.

**Error handling**: Use `Err(WorkspaceError::validation_failed())` for precondition violations. Do NOT use `Ok(vec![FailureEvent])` — keep errors and events separate.

**Required traits**:
- Commands: `Identifier`, `DeciderType`, `Serialize`, `Deserialize`, `TS`
- Events: `Identifier`, `EventType`, `DeciderType`, `IsFinal`, `Serialize`, `Deserialize`, `TS`
- State: `Debug`, `Clone`, `Default`, `PartialEq`

## Deliverables

Phase 1 (domain types):
- User and UserId types with composite key pattern (SharedKernel)
- All 5 aggregate Deciders with commands, events, state, errors, values

Phase 2 (infrastructure wiring):
- EventSourcedAggregate wiring to SQLite
- MaterializedView for Workspace projections
- Zenoh event publishing integration

Phase 3 (application layer):
- Command handlers (POST routes)
- Query handlers (GET routes)

Phase 4 (testing):
- Decider specification tests (given/when/then)
- View specification tests

## References

- spec/Workspace/WorkspaceAggregate.idr
- spec/Workspace/Dashboard.idr
- spec/Workspace/SavedQuery.idr
- spec/Workspace/UserPreferences.idr
- spec/Workspace/WorkspacePreferences.idr
- spec/SharedKernel/UserId.idr
- crates/ironstar/src/domain/todo/ (reference implementation)

### Dependencies

- 🔗 **implements**: `ironstar-2it`

### Comments

> **Cameron Smith** (2026-01-19)
>
> Checkpoint: Session 2026-01-18
> 
> Done:
> - 7a2.1: UserId shared kernel pattern verified
> - 7a2.2: Workspace aggregate implemented (commands, events, state, errors, decider)
> 
> Remaining P2 aggregates follow established pattern:
> - 7a2.3: WorkspacePreferences
> - 7a2.4: Dashboard
> - 7a2.5: SavedQuery
> - 7a2.6: UserPreferences
> - 7a2.7: workspaceContextDecider composition
> 
> Suggested next: Continue with 7a2.3-7a2.6 following Workspace pattern, or pivot to infrastructure wiring (nyp) if integration testing needed.

---

<a id="ironstar-2it-21-integrate-idris2-spec-modules-and-verify-type-checking"></a>

## 📋 ironstar-2it.21 Integrate Idris2 spec modules and verify type-checking

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 15:01 |
| **Updated** | 2026-01-06 20:44 |
| **Closed** | 2026-01-06 20:44 |

### Description

Create top-level IronstarSpec.idr module, verify all modules type-check, and perform initial D2 cross-reference.

## Module: spec/src/IronstarSpec.idr

```idris
module IronstarSpec

import public Decider
import public Repository
import public Domain.Analytics.Catalog
import public Domain.Analytics.QuerySession
import public Domain.Analytics.Chart
import public Domain.Session
import public Domain.Todo
```

## Type-checking verification

```bash
cd spec && idris2 --check ironstar.ipkg
```

All modules must compile without errors.

## Initial D2 cross-reference

Before detailed co-refinement (.16 modified), perform quick consistency check:

### D2 diagrams to compare
- docs/notes/event-modeling/d2/bounded-contexts.d2
- docs/notes/event-modeling/d2/analytics-context-timeline.d2
- docs/notes/event-modeling/d2/session-context-timeline.d2
- docs/notes/event-modeling/d2/todo-context-timeline.d2

### Consistency checks
- [ ] Every aggregate in D2 has corresponding Idris2 module
- [ ] Every event (orange) in D2 has corresponding Idris2 constructor
- [ ] Every command (blue) in D2 has corresponding Idris2 constructor
- [ ] Every read model (green) in D2 has corresponding Idris2 type

### Document gaps
List any mismatches found for resolution in .16 co-refinement task.

## Deliverables

- [ ] spec/src/IronstarSpec.idr created
- [ ] `idris2 --check ironstar.ipkg` passes
- [ ] Initial D2 cross-reference documented
- [ ] Gap list for .16 co-refinement

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.18`
- ⛔ **blocks**: `ironstar-2it.19`
- ⛔ **blocks**: `ironstar-2it.20`
- ⛔ **blocks**: `ironstar-2it.22`

---

<a id="ironstar-2it-18-formalize-analytics-bounded-context-in-idris2-core-domain"></a>

## 📋 ironstar-2it.18 Formalize Analytics bounded context in Idris2 (Core domain)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 15:00 |
| **Updated** | 2026-01-06 19:48 |
| **Closed** | 2026-01-06 19:48 |

### Description

Define the Analytics bounded context as Idris2 modules with dependent types. This is the Core domain (primary differentiator) warranting highest specification rigor.

## Business context

Scientific data analysis dashboard using DuckDB/DuckLake + ECharts. The Analytics context handles:
1. **Catalog selection** - Choose which DuckLake catalog to query (outside QuerySession)
2. **Query execution** - Run SQL queries within selected catalog context
3. **Chart rendering** - Transform query results to ECharts visualizations

## Module structure

```
spec/src/Domain/Analytics/
├── Catalog.idr        # DuckLake catalog selection (aggregate)
├── QuerySession.idr   # Query execution within catalog (aggregate)
└── Chart.idr          # Visualization types (value objects)
```

## Catalog.idr (Aggregate)

Manages DuckLake catalog selection. A catalog contains multiple datasets.

```idris
-- Commands
data CatalogCommand
  = SelectCatalog CatalogRef
  | RefreshCatalogMetadata

-- Events  
data CatalogEvent
  = CatalogSelected CatalogRef Timestamp
  | CatalogMetadataRefreshed CatalogMetadata Timestamp

-- State
data CatalogState
  = NoCatalogSelected
  | CatalogActive CatalogRef CatalogMetadata

-- Value objects
record CatalogRef where
  constructor MkCatalogRef
  uri : String  -- e.g., "ducklake://hf/sciexp/fixtures"
  
record CatalogMetadata where
  constructor MkCatalogMetadata
  datasets : List DatasetInfo
  lastRefreshed : Timestamp
```

## QuerySession.idr (Aggregate)

Query execution within a selected catalog context.

```idris
-- Precondition: CatalogState must be CatalogActive
-- This is where dependent types shine

-- Commands
data QueryCommand
  = StartQuery DatasetRef SqlQuery (Maybe ChartConfig)
  | CancelQuery QueryId

-- Events
data QueryEvent
  = QueryStarted QueryId DatasetRef SqlQuery (Maybe ChartConfig) Timestamp
  | QueryCompleted QueryId QueryResults Duration Timestamp
  | QueryFailed QueryId ErrorMessage Timestamp

-- State (state machine)
data QueryState
  = Idle
  | Running QueryId DatasetRef SqlQuery
  | Completed QueryId QueryResults
  | Failed QueryId ErrorMessage

-- Invariant: Only one active query per session
-- Dependent type can encode this
```

## Chart.idr (Value Objects)

ECharts configuration and data types.

```idris
record ChartConfig where
  constructor MkChartConfig
  chartType : ChartType
  options : ChartOptions

data ChartType = Line | Bar | Scatter | Pie | Heatmap

record ChartData where
  constructor MkChartData
  series : List Series
  xAxis : Maybe AxisData
  yAxis : Maybe AxisData
```

## Reference materials

### D2 diagram
- docs/notes/event-modeling/d2/analytics-context-timeline.d2

### Architecture docs
- docs/notes/architecture/frontend/ds-echarts-integration-guide.md
- docs/notes/architecture/application/chart-transformer-pattern.md
- docs/notes/architecture/infrastructure/analytics-cache-architecture.md

### Domain modeling methodology
- Wlaschin: ~/projects/functional-programming-workspace/domain-modeling-made-functional
- Ghosh: ~/projects/functional-programming-workspace/functional-and-reactive-domain-modeling

### Idris2 reference
- ~/projects/functional-programming-workspace/Idris2/docs/

## Dependent type opportunities

1. **Catalog precondition**: QuerySession commands require CatalogActive state
2. **Query state machine**: Transitions only valid from certain states
3. **Dataset existence**: DatasetRef must exist in CatalogMetadata.datasets

## Deliverables

- [ ] spec/src/Domain/Analytics/Catalog.idr
- [ ] spec/src/Domain/Analytics/QuerySession.idr
- [ ] spec/src/Domain/Analytics/Chart.idr
- [ ] All modules type-check
- [ ] Cross-reference with D2 diagram for consistency

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.17`

---

<a id="ironstar-2it-17-create-idris2-spec-infrastructure-and-core-abstractions"></a>

## 📋 ironstar-2it.17 Create Idris2 spec infrastructure and core abstractions

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 15:00 |
| **Updated** | 2026-01-06 16:50 |
| **Closed** | 2026-01-06 16:50 |

### Description

Set up the spec/ directory structure and implement core Decider + Repository abstractions in Idris2.

## Business context

Idris2 formal specification serves as machine-checkable source of truth for ironstar's event-sourced domain model. The spec enables type-driven discovery during Event Modeling, catching inconsistencies early through dependent types.

## Directory structure

```
spec/
├── ironstar.ipkg              # Idris2 package definition
├── README.md                  # Purpose, type-checking instructions, methodology
└── src/
    ├── Decider.idr            # Core Decider pattern with dependent types
    ├── Repository.idr         # EventRepository interface (fstore-sql pattern)
    ├── Domain/                # Bounded context modules (separate tasks)
    └── IronstarSpec.idr       # Top-level re-export (separate task)
```

## Decider.idr specification

Translate fmodel-rust's Decider pattern to Idris2 with dependent types:

```idris
record Decider (command : Type) (state : Type) (event : Type) where
  constructor MkDecider
  decide : command -> state -> List event
  evolve : state -> event -> state
  initialState : state
  -- Dependent type: evolve preserves state validity
  -- evolvePreservesValidity : ValidState s -> (e : event) -> ValidState (evolve s e)
```

## Repository.idr specification

Translate fstore-sql EventRepository pattern:

```idris
interface EventRepository (m : Type -> Type) (event : Type) where
  append : event -> m ()
  loadEvents : AggregateId -> m (List event)
  -- Dependent type: events are monotonically ordered by sequence
```

## Reference materials

### fmodel-rust (Decider pattern)
- Source: ~/projects/rust-workspace/fmodel-rust
- Key files: src/decider.rs, src/aggregate.rs

### fstore-sql (EventRepository pattern)
- Source: ~/projects/rust-workspace/fstore-sql
- Key file: schema.sql (events table with previous_id chain)

### Idris2 reference
- Source: ~/projects/functional-programming-workspace/Idris2
- Docs: ~/projects/functional-programming-workspace/Idris2/docs/

### Domain modeling methodology
- Wlaschin: ~/projects/functional-programming-workspace/domain-modeling-made-functional
- Ghosh: ~/projects/functional-programming-workspace/functional-and-reactive-domain-modeling

## Deliverables

- [ ] spec/ directory created
- [ ] spec/ironstar.ipkg with package definition
- [ ] spec/README.md with methodology explanation
- [ ] spec/src/Decider.idr with dependent-typed Decider record
- [ ] spec/src/Repository.idr with EventRepository interface
- [ ] Type-checking passes: `cd spec && idris2 --check ironstar.ipkg`

## Acceptance criteria

- Decider pattern matches fmodel-rust semantics
- Repository interface matches fstore-sql pattern
- Dependent types encode key invariants (state validity, event ordering)
- README explains the spec's role in type-driven Event Modeling workflow

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.13`
- ⛔ **blocks**: `ironstar-2it.14`
- ⛔ **blocks**: `ironstar-2it.15`

### Comments

> **crs58** (2026-01-06)
>
> ## Implementation Complete
> 
> Created Idris2 formal specification infrastructure at spec/.
> 
> ### Deliverables
> 
> **Package:** spec/ironstar.ipkg
> - Version 0.1.0 with base dependency
> - All 9 modules typecheck with idris2 --typecheck
> 
> **Core Modules:**
> 
> 1. **Core/Decider.idr** - Minimal algebraic interface for event-sourced aggregates
>    - `Decider c s e err` record with decide, evolve, initialState
>    - `Sum` and `Sum3` types for combining independent command/event types
>    - `combine`, `combine3` composition operators
>    - Derived computations: `reconstruct`, `computeNewEvents`, `computeNewState`
>    - Laws as reflexivity proofs (purity, determinism, replay)
>    - `foldlAssociative` lemma for snapshot support
> 
> 2. **Core/View.idr** - Read-side projections
>    - `View s e` record for event projection to read models
>    - `merge` composition for same-event-type projections
>    - `viewFromDecider` extraction function
>    - `mapState`, `contramapEvent` transformations
> 
> 3. **Core/Saga.idr** - Process managers
>    - `Saga ar a` record (action result → list of actions)
>    - `merge` composition for multi-workflow coordination
>    - `chain` (>=>) for sequential pipelines
>    - `identity`, `empty` as composition units
>    - `filter`, `fromMaybe` conditional constructors
> 
> 4. **Core/Effect.idr** - IO boundaries
>    - Identification types: EventId, AggregateId, Version, SessionId
>    - `EventRepository` interface with fetch, append, fetchSince
>    - `EventSourcedAggregate` record with `handleWith` function
>    - `ViewRepository`, `EventNotifier`, `EventSubscriber` interfaces
>    - `sessionKeyExpr` for Zenoh key pattern generation
> 
> 5. **Core/Event.idr** - Event algebra
>    - `Timestamp` for bitemporal events
>    - `EventEnvelope` wrapper with full metadata
>    - `EventIdLT`, `MonotonicIds` for ordering proofs
>    - Replay operations: `replayAfter`, `asOfEventTime`, `asOfRecordedTime`
>    - Free monoid laws (identity, associativity)
>    - `FailureEventPreservesState` interface (Hoffman's Law 6)
> 
> **Stub Modules:**
> - Analytics.Stub, Session.Stub, Workspace.Stub, Todo.Stub
> - Each documents intended abstractions for .18/.19/.20/.22
> 
> ### Key Design Decisions
> 
> 1. **Effect boundaries:** Pure Decider/View/Saga, effectful EventRepository via IO monad
> 2. **Laws as reflexivity:** Started with trivial proofs (x = x), upgrade to proper proofs incrementally
> 3. **DPair over records:** For dependent fields like MonotonicSequence
> 4. **Explicit function parameters:** handleWith takes fetch/append functions instead of interface constraints for simpler types
> 
> ### Commits
> - d65962f feat(spec): add Idris2 formal specification package definition
> - eacef1d feat(spec): add Core.Decider with fmodel-rust pattern
> - 5479262 feat(spec): add Core.View for read-side projections
> - 0b5759e feat(spec): add Core.Saga for process managers
> - a9db2ce feat(spec): add Core.Effect for IO boundaries
> - 07c7f53 feat(spec): add Core.Event for event algebra
> - 0510c7e feat(spec): add stub modules for bounded contexts
> 
> ### Typecheck verification
> ```bash
> cd spec && idris2 --typecheck ironstar.ipkg
> # All 9 modules build successfully
> ```
> 
> Unblocks: .18 (Analytics), .19 (Session), .20 (Todo), .22 (Workspace)

---

<a id="ironstar-2it-16-co-refine-d2-diagrams-and-idris2-spec-iterative"></a>

## 📋 ironstar-2it.16 Co-refine D2 diagrams and Idris2 spec (iterative)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:41 |
| **Updated** | 2026-01-06 20:52 |
| **Closed** | 2026-01-06 20:52 |

### Description

Iterate between D2 visual representation and Idris2 algebraic specification until both are consistent. This is the synthesis step that produces implementation-ready artifacts.

## Methodology

Type-driven Event Modeling uses two complementary views:
- **D2 diagrams**: Visual topology (what connects to what, temporal flow)
- **Idris2 spec**: Algebraic structure (what operations are valid, type invariants)

Iterate until both views represent the same domain model:

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│   Idris2 spec ←────────────→ D2 diagrams               │
│        ↑                           ↑                    │
│        │     iterate until         │                    │
│        │     consistent            │                    │
│        ↓                           ↓                    │
│   Type errors reveal      Visual gaps reveal            │
│   missing concepts        missing transitions           │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Input artifacts

### Idris2 spec (from .17-.21)
- spec/src/Decider.idr
- spec/src/Repository.idr
- spec/src/Domain/Analytics/*.idr
- spec/src/Domain/Session.idr
- spec/src/Domain/Todo.idr

### D2 diagrams
- docs/notes/event-modeling/d2/bounded-contexts.d2
- docs/notes/event-modeling/d2/analytics-context-timeline.d2
- docs/notes/event-modeling/d2/session-context-timeline.d2
- docs/notes/event-modeling/d2/todo-context-timeline.d2

## Refinement checklist

### Structural alignment
- [ ] Every Idris2 aggregate has a D2 yellow box
- [ ] Every Idris2 event constructor has a D2 orange node
- [ ] Every Idris2 command constructor has a D2 blue node
- [ ] Every Idris2 state has representation in D2 swimlanes

### Semantic alignment
- [ ] D2 transition arrows match valid Idris2 state machine transitions
- [ ] D2 read models (green) match Idris2 query return types
- [ ] D2 external systems (purple) match Idris2 Repository/effect interfaces

### Dependent type validation
- [ ] Idris2 preconditions (e.g., CatalogActive required for QuerySession) reflected in D2 flow
- [ ] Idris2 state machine constraints match D2 allowed transitions
- [ ] No D2 path that violates Idris2 type constraints

## Resolution process

When mismatch found:
1. Determine which artifact is "correct" (consult architecture docs)
2. Update the other artifact
3. Re-verify type-checking (Idris2) or render (D2)
4. Document the decision

## Output

- [ ] Consistent D2 diagrams matching Idris2 spec
- [ ] Consistent Idris2 spec matching D2 diagrams
- [ ] Resolution log documenting decisions made
- [ ] Ready for Qlerify prompt generation (.1)

## Reference materials

### Domain modeling methodology
- Wlaschin: ~/projects/functional-programming-workspace/domain-modeling-made-functional
- Ghosh: ~/projects/functional-programming-workspace/functional-and-reactive-domain-modeling

### Idris2 reference
- ~/projects/functional-programming-workspace/Idris2/docs/

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.21`

---

<a id="ironstar-2it-15-cross-reference-fmodel-rust-patterns-for-decider-implementation"></a>

## 📋 ironstar-2it.15 Cross-reference fmodel-rust patterns for Decider implementation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:41 |
| **Updated** | 2026-01-06 22:18 |
| **Closed** | 2026-01-06 22:18 |

### Description

Review fmodel-rust and fmodel-rust-demo patterns to ensure Event Modeling artifacts will map correctly to Decider implementations.

## Source repositories

- `~/projects/rust-workspace/fmodel-rust` — Core library
- `~/projects/rust-workspace/fmodel-rust-demo` — Working example (Order + Restaurant domains)
- `~/projects/rust-workspace/fmodel-rust-postgres` — PostgreSQL EventRepository
- `~/projects/rust-workspace/fstore-sql` — Canonical event store schema

## Patterns to validate

### Decider trait mapping
```rust
pub trait Decider<C, S, E> {
    fn decide(&self, command: &C, state: &S) -> Vec<E>;
    fn evolve(&self, state: &S, event: &E) -> S;
    fn initial_state(&self) -> S;
}
```

Ensure D2 diagrams capture:
- Commands (blue) → C type parameter
- Events (orange) → E type parameter
- Aggregate state → S type parameter

### Identifier traits
From fmodel-rust:
- `Identifier` — aggregate identity
- `EventType` — event type discriminator
- `DeciderType` — decider type discriminator
- `IsFinal` — terminal event marker

Ensure Event Modeling artifacts include these concepts.

### EventRepository pattern
From fmodel-rust-demo and fstore-sql:
- `previous_id` UUID chain for optimistic locking
- `offset BIGSERIAL` for global monotonic ordering
- Partition locking for concurrent consumers

### View/Projection pattern
```rust
pub trait View<S, E> {
    fn evolve(&self, state: &S, event: &E) -> S;
    fn initial_state(&self) -> S;
}
```

Ensure read models in D2 map to View implementations.

## Validation against ironstar docs

- `docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md`
- `docs/notes/architecture/cqrs/event-sourcing-core.md`
- `docs/notes/architecture/core/discovery-and-specification.md`

## Output

- Validation that D2 artifacts align with fmodel-rust expectations
- Document any gaps in current Event Modeling artifacts
- Update D2 diagrams if fmodel-rust patterns reveal missing elements

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`

### Comments

> **crs58** (2026-01-06)
>
> ## Task .15 Complete
> 
> Extracted fmodel-rust Decider patterns to: `docs/notes/architecture/decisions/fmodel-rust-decider-signatures.md`
> 
> **Key findings:**
> - Exact Decider<C, S, E, Error> signatures with Rust→Idris2 type mapping
> - Laws identified: decide purity, evolve totality, state reconstruction via fold
> - Effect boundary: Decider (pure) vs EventSourcedAggregate (effectful handle())
> - Composition: combine() for independent Deciders, merge() for shared event streams
> 
> **Idris2 questions for .17:**
> - Trait bound encoding (implicit auto vs explicit constraints)
> - Effect type (IO vs Eff vs Free monad)
> - Proof obligations (postulate vs proof-carrying Decider)
> 
> Commit: 9aa10e9

---

<a id="ironstar-2it-14-cross-reference-northstar-patterns-for-datastar-sse-architecture"></a>

## 📋 ironstar-2it.14 Cross-reference northstar patterns for Datastar SSE architecture

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:41 |
| **Updated** | 2026-01-06 22:18 |
| **Closed** | 2026-01-06 22:18 |

### Description

Review northstar (Go template) patterns to ensure ironstar correctly adapts Datastar SSE streaming architecture.

## Source repository

`~/projects/lakescope-workspace/datastar-go-nats-template-northstar`

## Patterns to validate

### SSE streaming architecture
- How northstar structures SSE endpoints
- Signal merging patterns (MergeFragments, MergeSignals)
- Last-Event-ID reconnection handling
- Keep-alive comment streams

### Web component integration
- ds-echarts Lit component pattern
- Light DOM requirement for Open Props token inheritance
- Build pipeline (esbuild in northstar → Rolldown in ironstar)

### Event bus patterns
- NATS in northstar → Zenoh in ironstar
- Key expression filtering equivalents
- Per-session subscription patterns

### Asset embedding
- hashfs pattern in Go → rust-embed in Rust
- Content-hashed URLs
- Dev/prod mode separation

## Validation against ironstar docs

Cross-reference findings with:
- `docs/notes/architecture/frontend/ds-echarts-integration-guide.md`
- `docs/notes/architecture/infrastructure/zenoh-event-bus.md`
- `docs/notes/architecture/cqrs/sse-connection-lifecycle.md`
- `docs/notes/architecture/frontend/frontend-build-pipeline.md`

## Output

- Document any pattern gaps or adaptations needed
- Update architecture docs if northstar reveals missing patterns
- Note any patterns that don't translate directly to Rust

## Key files to review in northstar

- `internal/sse/` — SSE streaming implementation
- `internal/web/` — Web component integration
- `web/components/ds-echarts/` — ECharts Lit component
- `cmd/server/main.go` — Application wiring

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`

### Comments

> **crs58** (2026-01-06)
>
> ## Task .14 Complete
> 
> Extracted northstar SSE patterns to: `docs/notes/architecture/decisions/northstar-sse-patterns-for-idris2.md`
> 
> **Key findings:**
> - Effect boundaries mapped: pure core (Decider) vs effects at edges (Subscribe, Append, SSE Write)
> - Invariants identified: monotonicity, replay idempotence, session isolation
> - 6 ironstar doc gaps found requiring updates
> - 3 northstar anti-patterns to avoid
> 
> **Idris2 questions for .17:**
> - Dependent pairs for replay consistency proofs
> - Effect placement for subscribe-then-replay ordering
> - Algebraic properties: SSE projection totality, replay idempotence
> 
> Commit: def6a6d

---

<a id="ironstar-2it-13-review-bounded-context-definitions-against-architecture-docs"></a>

## 📋 ironstar-2it.13 Review bounded context definitions against architecture docs

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:40 |
| **Updated** | 2026-01-06 22:18 |
| **Closed** | 2026-01-06 22:18 |

### Description

Validate that D2 bounded-contexts.d2 accurately reflects the architecture documented in docs/notes/architecture/, with special attention to the scientific data analysis business context.

## Business context

Ironstar's primary purpose is a **DuckDB/DuckLake query and Lit ECharts web component data analysis dashboard for scientific data analysis**. This context must drive the Event Modeling artifacts:

- Analytics Context is Core (primary differentiator)
- Todo Context is Generic Example (demonstration only)
- Session Context is Supporting (authentication infrastructure)

## Artifacts to cross-reference

### D2 diagram
`docs/notes/event-modeling/d2/bounded-contexts.d2`

Current contexts:
- Session Context (Supporting) — Session aggregate
- Todo Context (Generic Example) — Todo aggregate
- Analytics Context (Core) — QuerySession aggregate

### Architecture docs to validate against

- `docs/notes/architecture/core/bounded-contexts.md` (if exists)
- `docs/notes/architecture/core/architecture-decisions.md` § bounded context decisions
- `docs/notes/architecture/core/design-principles.md` § strategic classification
- `docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md` § aggregate mapping
- `docs/notes/architecture/frontend/ds-echarts-integration-guide.md`
- `docs/notes/architecture/application/chart-transformer-pattern.md`

### Scientific data integration references

- `ironstar-3gd` epic children (Scientific Data Integration)
- HuggingFace Hub integration patterns (`hf://` protocol)
- DuckLake catalog versioning
- `~/projects/rust-workspace/rust-duckdb-huggingface-ducklake-query`

## Validation checklist

- [ ] All aggregates in D2 are documented in architecture docs
- [ ] Strategic classification (Core/Supporting/Generic) is consistent
- [ ] Integration patterns (Customer-Supplier, Partnership) match architecture intent
- [ ] Event bus (Zenoh) key expression schema matches both artifacts
- [ ] No orphan concepts (in D2 but not in docs, or vice versa)
- [ ] Analytics Context adequately captures scientific data analysis workflows
- [ ] Dataset management (DuckLake, HuggingFace) is represented or planned

## Questions to resolve

- Is QuerySession the right aggregate name? (vs AnalyticsSession, DatasetQuery)
- Should there be a separate Dataset aggregate for dataset catalog management?
- Should Dashboard composition (multiple charts) be a separate aggregate or part of QuerySession?
- Are chart configuration persistence and query history adequately modeled?

## Output

- Updated D2 if discrepancies found
- Updated architecture docs if D2 reveals gaps
- Validation notes documenting any decisions made
- Clear articulation of scientific data analysis event flows

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`

### Comments

> **crs58** (2026-01-06)
>
> ## Validation Complete
> 
> **Discrepancies found and resolved:**
> 
> 1. **Missing Catalog aggregate** in D2 → Added with SelectCatalog/RefreshCatalogMetadata commands and CatalogSelected/CatalogMetadataRefreshed events
> 
> 2. **Todo-Analytics Partnership** in D2 conflicted with "Standalone" documentation → Removed Partnership edge; Todo is intentionally isolated
> 
> 3. **Todo framing** updated from "Generic Example" to "Generality Canary" — emphasizes its role as template validation rather than demonstration
> 
> **Commits:**
> - fcdd92b: docs(event-modeling): add Catalog aggregate, remove Todo-Analytics partnership
> - e83c3a9: docs(architecture): reframe Todo as generality canary
> 
> **Alignment verified:**
> - 4 bounded contexts: Analytics (Core), Session (Supporting), Workspace (Supporting), Todo (Generality Canary)
> - Analytics now has 3 aggregates: QuerySession, ChartDefinition, Catalog
> - Integration patterns: Customer-Supplier (Session→*, Analytics→Workspace), Shared Kernel (Session→Workspace)
> - Zenoh key expressions correctly namespace all 4 contexts
> 
> Ready for .14/.15 upstream pattern validation.

---

<a id="ironstar-2it-11-cross-reference-and-validate-eventcatalog-artifacts-phase-9"></a>

## 📋 ironstar-2it.11 Cross-reference and validate EventCatalog artifacts (Phase 9)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:19 |
| **Updated** | 2026-01-11 01:07 |
| **Closed** | 2026-01-11 01:07 |

### Description

Execute Phase 9 of the transformation workflow: link all artifacts through cross-references and validate output quality.

## Prerequisites

- All MDX artifacts generated (ironstar-2it.10)

## Cross-referencing

### Update service sends/receives

For each service:
- `sends`: events where domainEvent.laneId matches service's lane
- `receives`: commands/events consumed by this service
- `entities`: aggregates owned by this service

### Update event consumers

For each event:
- `consumers`: services whose lanes contain domainEvents listing this event in parents[]

### Update command producers

For each command:
- `producers`: services/actors that trigger this command

## Validation checklist

Per event-catalog-qlerify.md:
- [ ] All schema.json files are valid JSON Schema draft-07
- [ ] All service cross-references point to existing artifacts
- [ ] All flow steps reference valid messages and services
- [ ] All inferred types have confidence notes
- [ ] All enum fields have populated enum arrays
- [ ] All aggregate roots have aggregateRoot: true and identifier

## Structural validation commands

```bash
# Verify schema files
fd -e json -x jaq '.""' {} \;

# Count artifacts by type
echo "Services: $(fd -t d . domains/*/services | wc -l)"
echo "Events: $(fd index.mdx domains/*/services/*/events | wc -l)"
echo "Commands: $(fd index.mdx domains/*/services/*/commands | wc -l)"
echo "Entities: $(fd index.mdx domains/*/entities | wc -l)"
echo "Flows: $(fd index.mdx domains/*/flows | wc -l)"
```

## Output

- Fully cross-referenced EventCatalog
- Validation report
- No orphan artifacts (all linked bidirectionally)

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.10`

---

<a id="ironstar-2it-10-transform-qlerify-json-to-eventcatalog-mdx-phase-3-entities-and-flows"></a>

## 📋 ironstar-2it.10 Transform Qlerify JSON to EventCatalog MDX (Phase 3: Entities and Flows)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:19 |
| **Updated** | 2026-01-11 01:07 |
| **Closed** | 2026-01-11 01:07 |

### Description

Execute Phase 7-8 of the transformation workflow from event-catalog-qlerify.md.

## Prerequisites

- Events, commands, queries created (ironstar-2it.9)

## Phase 7: Extract entities from AggregateRoot cards

For each Entity schema (type=Entity), generate entity MDX:

`domains/ironstar/entities/Session/index.mdx`:
```yaml
---
id: Session
name: Session
version: 0.0.1
summary: User session aggregate root
owners: []
aggregateRoot: true
identifier: sessionId
properties:
  - name: sessionId
    type: string
    required: true
    summary: Unique session identifier (UUID)
  - name: userId
    type: string
    required: true
    summary: Associated user identity
  - name: expiresAt
    type: string
    required: true
    summary: Session expiration timestamp (ISO 8601)
---
```

### Decider pattern documentation

Entity MDX should document:
- `decide` function: command → state → events
- `evolve` function: state → event → state
- Link to fmodel-rust implementation

## Phase 8: Generate flow from parent-child chains

Reconstruct temporal flow from domainEvent.parents[]:

`domains/ironstar/flows/session-lifecycle/index.mdx`:
```yaml
---
id: session-lifecycle
name: Session Lifecycle Flow
version: 0.0.1
summary: OAuth login through session expiration
steps:
  - id: step-1
    title: OAuth Callback Received
    actor:
      name: Guest
    message:
      id: OAuthCallback
      version: "0.0.1"
    service:
      id: SessionService
    next_step:
      id: step-2
      label: "on success"
  - id: step-2
    title: Session Created
    message:
      id: SessionCreated
    service:
      id: SessionService
---
```

## Output

- Entity MDX for all aggregates (Session, Todo, QuerySession)
- Flow MDX for primary workflows:
  - session-lifecycle
  - todo-crud
  - analytics-query-execution

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.9`

---

<a id="ironstar-2it-9-transform-qlerify-json-to-eventcatalog-mdx-phase-2-events-commands-queries"></a>

## 📋 ironstar-2it.9 Transform Qlerify JSON to EventCatalog MDX (Phase 2: Events, Commands, Queries)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:18 |
| **Updated** | 2026-01-11 00:59 |
| **Closed** | 2026-01-11 00:59 |

### Description

Execute Phase 4-6 of the transformation workflow from event-catalog-qlerify.md.

## Prerequisites

- Domain and services created (ironstar-2it.8)

## Phase 4: Extract events from domainEvents

For each domainEvent, generate event MDX + schema.json:

`services/SessionService/events/SessionCreated/index.mdx`:
```yaml
---
id: SessionCreated
name: Session Created
version: 0.0.1
summary: User session successfully established
producers:
  - SessionService
consumers: []
schemaPath: schema.json
---
```

`services/SessionService/events/SessionCreated/schema.json`:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SessionCreated",
  "type": "object",
  "properties": {
    "sessionId": { "type": "string", "format": "uuid" },
    "userId": { "type": "string", "format": "uuid" },
    "timestamp": { "type": "string", "format": "date-time" }
  },
  "required": ["sessionId", "userId", "timestamp"]
}
```

## Phase 5: Extract commands

Generate command MDX + schema.json following same pattern.

## Phase 6: Extract queries (read models)

Generate query MDX + schema.json for each ReadModel card.

## Schema translation rules

Per event-catalog-qlerify.md:
- uuid → string + format: uuid
- timestamp → string + format: date-time
- int → integer
- enum → string + enum array from exampleData
- null → infer from field name patterns

## Output

- Event MDX + schema.json for all events
- Command MDX + schema.json for all commands
- Query MDX + schema.json for all read models

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.8`

---

<a id="ironstar-2it-8-transform-qlerify-json-to-eventcatalog-mdx-phase-1-domain-and-services"></a>

## 📋 ironstar-2it.8 Transform Qlerify JSON to EventCatalog MDX (Phase 1: Domain and Services)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:18 |
| **Updated** | 2026-01-11 00:48 |
| **Closed** | 2026-01-11 00:48 |

### Description

Execute Phase 1-3 of the transformation workflow from event-catalog-qlerify.md.

## Prerequisites

- EventCatalog infrastructure ready (ironstar-2it.7)
- Qlerify JSON exports validated (ironstar-2it.5)

## Phase 1: Discover structure

Run discovery queries:
```bash
# Schema types and counts
duckdb -c "SELECT s.type, COUNT(*) FROM read_json_auto('export.json') AS r, unnest(r.schemas) AS s GROUP BY s.type"

# Lane distribution
jaq '.lanes | map(.name)' export.json
```

## Phase 2: Extract domain

Generate `domains/ironstar/index.mdx`:
```yaml
---
id: ironstar
name: Ironstar
version: 1.0.0
summary: Event-driven reactive web application template
owners: []
---
```

## Phase 3: Extract services from lanes

For each Qlerify lane, generate service:

`domains/ironstar/services/SessionService/index.mdx`
`domains/ironstar/services/TodoService/index.mdx`
`domains/ironstar/services/AnalyticsService/index.mdx`

Service template:
```yaml
---
id: SessionService
name: Session Service
version: 0.0.1
summary: Session and authentication management
owners: []
sends: []
receives: []
entities: []
---
```

## Output

- Domain MDX file
- Service MDX files for all bounded contexts
- sends/receives arrays populated in Phase 9

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.7`

---

<a id="ironstar-2it-7-instantiate-eventcatalog-infrastructure"></a>

## 📋 ironstar-2it.7 Instantiate EventCatalog infrastructure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:18 |
| **Updated** | 2026-01-11 00:45 |
| **Closed** | 2026-01-11 00:45 |

### Description

Set up EventCatalog instance for ironstar documentation.

## Prerequisites

- Validated Qlerify JSON exports (ironstar-2it.5)

## Setup

### Create EventCatalog project

Using create-eventcatalog (see ~/projects/lakescope-workspace/create-eventcatalog):
```bash
cd packages/docs
npx @eventcatalog/create-eventcatalog@latest eventcatalog
```

Or integrate into existing Astro Starlight docs structure.

### Directory structure

```
packages/docs/src/content/eventcatalog/
├── domains/
│   └── ironstar/
│       ├── index.mdx
│       ├── services/
│       ├── entities/
│       └── flows/
├── eventcatalog.config.js
└── catalog-info.yaml
```

### Configuration

eventcatalog.config.js:
```javascript
module.exports = {
  title: 'Ironstar Event Catalog',
  tagline: 'Event-driven architecture documentation',
  editUrl: 'https://github.com/user/ironstar/edit/main',
  trailingSlash: false,
  logo: { alt: 'Ironstar', src: '/logo.png' },
};
```

## Integration with Starlight

Options:
1. Separate EventCatalog site at /eventcatalog/
2. Integrated into Starlight docs as reference section
3. Symlinks from docs/reference/eventcatalog/

## Output

- Working EventCatalog instance
- Domain structure matching bounded contexts
- Build integrated with nix flake

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.5`

---

<a id="ironstar-2it-6-refine-d2-diagrams-to-match-qlerify-exports"></a>

## 📋 ironstar-2it.6 Refine D2 diagrams to match Qlerify exports

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:18 |
| **Updated** | 2026-01-11 00:38 |
| **Closed** | 2026-01-11 00:38 |

### Description

Update existing D2 diagrams to align with refined Qlerify Event Models, ensuring consistency between visual artifacts.

## Prerequisites

- Qlerify JSON exports available (ironstar-2it.5)

## Existing D2 files

`docs/notes/event-modeling/d2/`:
- bounded-contexts.d2 (context map with integration patterns)
- analytics-context-timeline.d2
- session-context-timeline.d2
- todo-context-timeline.d2

## Alignment work

### Cross-validate structure
- D2 swimlane names match Qlerify lane names
- D2 event nodes match Qlerify domainEvent descriptions
- D2 command nodes match Qlerify Command schema names
- Cross-container arrows match parent-child relationships

### Color conventions (per event-modeling.md)
- Commands: #3498db (blue)
- Events: #e67e22 (orange)
- Read Models: #2ecc71 (green)
- Aggregates: #f1c40f (yellow)
- External Systems: #9b59b6 (purple)

### Updates needed

For each timeline diagram:
- Add/remove events to match Qlerify exports
- Update command names to match imperative mood
- Add read model nodes if missing
- Verify temporal ordering matches parent-child chains

## Output

- Updated D2 files matching Qlerify exports
- Rendered diagrams in `docs/notes/event-modeling/rendered/`

## Rendering command

```bash
d2 --layout elk bounded-contexts.d2 bounded-contexts.svg
```

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.5`

---

<a id="ironstar-2it-5-export-qlerify-json-and-validate-structure"></a>

## 📋 ironstar-2it.5 Export Qlerify JSON and validate structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:18 |
| **Updated** | 2026-01-11 00:38 |
| **Closed** | 2026-01-11 00:38 |

### Description

Export completed Event Modeling workflows from Qlerify as JSON and validate against expected structure.

## Prerequisites

- GWT scenarios complete (ironstar-2it.4)

## Export process

1. In Qlerify: Export workflow as JSON
2. Save to `docs/notes/event-modeling/qlerify/exports/`:
   - session-context.json
   - todo-context.json
   - analytics-context.json

## Validation queries

Run discovery queries from event-catalog-qlerify.md:

### duckdb exploration
```sql
-- Count entities by type
SELECT s.type, COUNT(*) as count
FROM read_json_auto('export.json') AS root,
    unnest(root.schemas) AS s
GROUP BY s.type;

-- Lane distribution
SELECT l.name, COUNT(e.id) as event_count
FROM read_json_auto('export.json') AS root,
    unnest(root.lanes) AS l
LEFT JOIN unnest(root.domainEvents) AS e ON e.laneId = l.id
GROUP BY l.name;
```

### jaq exploration
```bash
# Card type distribution
jaq '[.domainEvents[].cards[].cardType.domainModelRole] |
     group_by(.) | map({role: .[0], count: length})' export.json

# Events with no parents (flow entry points)
jaq '[.domainEvents[] | select(.parents | length == 0) | .description]' export.json
```

## Expected structure

Per event-catalog-qlerify.md:
- domainEvents[] with cards[]
- schemas[] with Command/Query/Entity types
- lanes[] for service boundaries
- parent-child relationships for flow

## Output

- Validated JSON exports in `docs/notes/event-modeling/qlerify/exports/`
- Validation report documenting counts and structure

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.4`

---

<a id="ironstar-2it-4-elaborate-gwt-scenarios-and-prioritize-releases-step-7"></a>

## 📋 ironstar-2it.4 Elaborate GWT scenarios and prioritize releases (Step 7)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:18 |
| **Updated** | 2026-01-11 00:38 |
| **Closed** | 2026-01-11 00:38 |

### Description

Complete Event Modeling Step 7: write Given-When-Then scenarios, prioritize into releases, and validate end-to-end flow.

## Prerequisites

- Bounded contexts assigned (ironstar-2it.3)

## GWT scenario types

### Regular patterns (validation rules)
Example: CreateTodo
```
Given: User has valid session
And: Todo text is non-empty
When: User submits CreateTodo
Then: TodoCreated event emitted
Then: SSE patch sent to subscribers
```

### Translation patterns (conditional logic)
Example: OAuth callback interpretation
```
Given: OAuth provider returns authorization code
When: Code exchange succeeds
Then: SessionCreated event emitted
```

### Automation patterns (eligibility criteria)
Example: Cache invalidation
```
Given: QueryCompleted event received
And: Cache entry exists for query hash
When: Invalidation policy runs
Then: Cache entry evicted
```

## Release prioritization

### Release 1 (MVP)
- Session: Login, logout, session validation
- Todo: Create, complete, delete, list
- Basic SSE streaming

### Release 2
- Analytics: Query execution, chart rendering
- Cache management

### Release 3
- Advanced analytics: HuggingFace datasets, DuckLake integration
- Performance optimizations

## Output

- GWT scenarios attached to all events in Qlerify
- User Story Map tab organized by release
- Release-filtered views validating end-to-end coherence

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.3`

---

<a id="ironstar-2it-3-apply-conway-s-law-and-assign-bounded-contexts-step-6"></a>

## 📋 ironstar-2it.3 Apply Conway's Law and assign bounded contexts (Step 6)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:17 |
| **Updated** | 2026-01-11 00:38 |
| **Closed** | 2026-01-11 00:38 |

### Description

Complete Event Modeling Step 6: assign bounded contexts to aggregate roots, establishing autonomous component boundaries.

## Prerequisites

- Qlerify sessions refined (ironstar-2it.2)

## Work

### Domain Model tab

For each aggregate root (yellow box):
- Assign bounded context
- Group related aggregates into same context

### Bounded context assignments

Based on existing D2 diagram:
- **Session Context**: Session aggregate
- **Todo Context**: Todo aggregate  
- **Analytics Context**: QuerySession aggregate

### Validate Conway's Law alignment

Each context should represent:
- Independently deployable component
- Clear team ownership boundary (even if single developer for template)
- Minimal coordination with other contexts

## Context map patterns

Document integration patterns between contexts:
- Session → Todo: Customer-Supplier
- Session → Analytics: Customer-Supplier
- Todo ↔ Analytics: Partnership (shared Zenoh event schemas)

## Output

Updated Qlerify models with bounded context assignments that map directly to EventCatalog service organization.

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.2`

---

<a id="ironstar-2it-2-execute-qlerify-sessions-and-refine-generated-models"></a>

## 📋 ironstar-2it.2 Execute Qlerify sessions and refine generated models

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:17 |
| **Updated** | 2026-01-11 00:38 |
| **Closed** | 2026-01-11 00:38 |

### Description

Execute the generated prompts in Qlerify, review AI output, and apply Steps 2-5 of the Event Modeling methodology.

## Prerequisites

- Qlerify prompts generated (ironstar-2it.1)
- User logged into Qlerify app

## Steps per bounded context

### Step 2: The Plot
- Review generated swimlanes
- Reorganize from systems to actors (Guest, Manager, Automation)
- Validate temporal ordering tells coherent narrative

### Step 3: The Storyboard
- Design UI mockups via command field ordering
- Add/update/remove fields for natural form flow
- For automation events: imagine robot filling form

### Step 4: Identify Inputs
- Validate command names match domain language
- Ensure imperative mood (CreateTodo not TodoCreation)

### Step 5: Identify Outputs
- Define read models representing data needed before commands
- Actor-centric: what does actor need to see before acting?

## Output

Refined Qlerify workflows for:
- Session Context
- Todo Context  
- Analytics Context

## Patterns to apply

Per event-modeling.md:
- Regular input form pattern (human actor fills form)
- External event pattern (OAuth callbacks)
- Translation pattern (GPS-style conditional logic if applicable)
- Automation pattern (scheduled cache invalidation)

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.1`

---

<a id="ironstar-2it-1-generate-qlerify-prompts-for-all-bounded-contexts"></a>

## 📋 ironstar-2it.1 Generate Qlerify prompts for all bounded contexts

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:17 |
| **Updated** | 2026-01-11 00:38 |
| **Closed** | 2026-01-11 00:38 |

### Description

Create natural language workflow prompts for Qlerify AI generation following the Event Modeling Step 1 (Brainstorming) methodology.

## Bounded contexts to model

Based on `docs/notes/event-modeling/d2/bounded-contexts.d2`:

1. **Session Context** (Supporting) - OAuth authentication, session lifecycle
2. **Todo Context** (Generic Example) - CRUD operations for demonstration
3. **Analytics Context** (Core) - Query execution, chart rendering, cache management

## Prompt structure per context

Each prompt should include (per event-modeling.md):
- Primary actors and roles
- Key state-changing events in chronological order
- External systems involved (OAuth providers, DuckDB, Zenoh)
- Success and failure scenarios

## Output

`docs/notes/event-modeling/qlerify/prompts/`:
- session-context-prompt.md
- todo-context-prompt.md
- analytics-context-prompt.md

Prompts should reference existing architecture docs:
- docs/notes/architecture/core/design-principles.md
- docs/notes/architecture/core/bounded-contexts.md
- docs/notes/architecture/cqrs/event-sourcing-core.md

## Alignment with fmodel-rust

Prompts should anticipate fmodel-rust Decider pattern:
- Commands as inputs to `decide` function
- Events as outputs of `decide` function
- State transitions via `evolve` function

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.16`

---

<a id="ironstar-2it-complete-event-modeling-and-eventcatalog-documentation"></a>

## 🚀 ironstar-2it Complete Event Modeling and EventCatalog documentation

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:17 |
| **Updated** | 2026-01-15 18:25 |
| **Closed** | 2026-01-15 18:25 |

### Description

Complete the Event Modeling workflow to produce implementation-ready specifications and populate EventCatalog with discoverable documentation for ironstar's architecture.

## Context

Ironstar's architecture is documented in `docs/notes/architecture/` but lacks formal Event Modeling artifacts that trace domain discoveries to implementation specifications. This epic bridges collaborative domain discovery to formal EventCatalog documentation following the workflows in:

- `~/.claude/commands/preferences/event-modeling.md` (7-step methodology)
- `~/.claude/commands/preferences/event-catalog-qlerify.md` (transformation workflow)
- `~/.claude/commands/preferences/event-catalog-tooling.md` (algebraic foundations)

## Current state

- D2 diagrams exist in `docs/notes/event-modeling/d2/` (4 files: bounded-contexts, analytics-context-timeline, session-context-timeline, todo-context-timeline)
- Empty `qlerify/` directory awaits JSON exports
- No EventCatalog infrastructure exists yet
- Architecture documentation describes intended design but lacks formal event/command/aggregate schemas

## Deliverables

1. Completed Event Modeling sessions for all bounded contexts
2. Refined D2 diagrams following Event Modeling color conventions
3. Qlerify JSON exports with field-level schemas
4. EventCatalog instance with MDX artifacts:
   - Domain, Service, Event, Command, Query, Entity, Flow definitions
   - JSON Schema for all artifacts
   - Producer/consumer relationships
5. Integration with fmodel-rust Decider pattern implementation

## Strategic alignment

This work supports ironstar-a9b (fmodel-rust event sourcing foundation) by providing formal specifications that map directly to Decider implementations. The Event Modeling artifacts trace to:

- Events → `Event` type parameter in Decider
- Commands → `Command` type parameter in Decider  
- Aggregates → `State` type and `evolve` function
- Read Models → View projections

## References

- docs/notes/architecture/core/discovery-and-specification.md
- docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md
- ~/projects/lakescope-workspace/eventcatalog (EventCatalog source)
- ~/projects/rust-workspace/fmodel-rust (Decider pattern implementation)

---

<a id="ironstar-ny3-18-add-cube-css-composition-layer"></a>

## 📋 ironstar-ny3.18 Add CUBE CSS composition layer

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 11:12 |
| **Updated** | 2026-01-19 18:18 |
| **Closed** | 2026-01-19 18:18 |

### Description

Implement CUBE CSS composition primitives as a dedicated CSS layer.

## Rationale

The current CSS architecture has Open Props tokens and UI components but lacks explicit layout composition primitives.
CUBE CSS methodology's Composition layer fills this gap, enabling:
- Semantic layout classes in server-rendered HTML
- Algebraic composition (primitives form a semiring over layout space)
- Local reasoning in hypertext templates
- Reduced ad-hoc CSS in components

## Implementation

Add `compositions` layer between `theme` and `components`:
```css
@layer openprops, normalize, theme, compositions, components, utilities, app;
```

Create 8 composition primitives in `web-components/styles/compositions/`:
- stack.css, box.css, center.css, cluster.css
- sidebar.css, switcher.css, cover.css, grid.css

Each primitive:
- Uses Open Props tokens exclusively
- Exposes CSS custom properties for customization
- Works with Light DOM inheritance

## References

- Architecture doc: docs/notes/architecture/frontend/css-architecture.md (updated with composition layer section)
- CUBE patterns: ~/.claude/commands/preferences/hypermedia-development/04-css-architecture.md

### Dependencies

- 🔗 **parent-child**: `ironstar-ny3`
- ⛔ **blocks**: `ironstar-ny3.5`

### Comments

> **crs58** (2026-01-06)
>
> Implementation deferred until ny3.5 (CSS cascade layers) is closed. Documentation and architecture spec complete in docs/notes/architecture/frontend/css-architecture.md. CSS patterns available in ~/.claude/commands/preferences/hypermedia-development/04-css-architecture.md.

---

<a id="ironstar-a9b-13-implement-view-specification-tests"></a>

## 📋 ironstar-a9b.13 Implement View specification tests

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:19 |
| **Updated** | 2026-01-18 21:30 |
| **Closed** | 2026-01-18 21:30 |

### Description

Implement tests for Todo View projection logic.

## Implementation

Test the View's evolve function with various event sequences:

```rust
#[test]
fn test_todo_view_projection() {
    let view = todo_view();
    let initial = (view.initial_state)();
    
    let state = (view.evolve)(&initial, &TodoEvent::Created { ... });
    assert_eq!(state.todos.len(), 1);
    
    let state = (view.evolve)(&state, &TodoEvent::Completed { ... });
    assert_eq!(state.completed_count, 1);
}
```

## Test Scenarios

- Empty state → Created → 1 todo
- Multiple creates → correct count
- Create → Complete → completed_count increments
- Create → Delete → todo removed

## Acceptance Criteria

- [ ] All event types tested for correct projection
- [ ] State accumulation tested with multiple events
- [ ] Tests in crates/ironstar-domain/tests/

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-a9b.5`

---

<a id="ironstar-a9b-11-implement-todo-query-handler"></a>

## 📋 ironstar-a9b.11 Implement Todo query handler

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:19 |
| **Updated** | 2026-01-18 21:37 |
| **Closed** | 2026-01-18 21:37 |

### Description

Implement Axum HTTP handler for Todo queries.

## Implementation

```rust
pub async fn get_todos(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let view_state = state.todo_view.current_state().await?;
    Ok(Json(view_state))
}
```

## Routes

- GET /api/todos - list all todos
- GET /api/todos/:id - get single todo

## SSE Alternative

For real-time updates, serve via datastar SSE:
- GET /api/todos/sse - stream todo list updates

## Acceptance Criteria

- [ ] Query handler reads from MaterializedView
- [ ] Returns current projection state
- [ ] SSE endpoint streams updates

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-a9b.8`

---

<a id="ironstar-a9b-10-implement-todo-command-handler"></a>

## 📋 ironstar-a9b.10 Implement Todo command handler

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:19 |
| **Updated** | 2026-01-18 21:53 |
| **Closed** | 2026-01-18 21:53 |

### Description

Implement Axum HTTP handler for Todo commands.

## Implementation

```rust
pub async fn handle_todo_command(
    State(state): State<Arc<AppState>>,
    Json(command): Json<TodoCommand>,
) -> Result<impl IntoResponse, AppError> {
    let events = handle_todo_command(
        state.event_repository.clone(),
        state.event_bus.clone(),
        command,
    ).await?;
    
    Ok((StatusCode::ACCEPTED, Json(events)))
}
```

## Routes

- POST /api/todos - create todo
- POST /api/todos/:id/complete - complete todo
- DELETE /api/todos/:id - delete todo

## Acceptance Criteria

- [ ] Axum handler receives TodoCommand
- [ ] Calls application layer handle_todo_command
- [ ] Returns 202 Accepted (SSE delivers updates)
- [ ] Error conversion from CommandError to AppError

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-a9b.7`
- ⛔ **blocks**: `ironstar-a9b.9`

---

<a id="ironstar-a9b-8-implement-todo-query-service-with-view-replay"></a>

## 📋 ironstar-a9b.8 Implement Todo query service with View replay

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:18 |
| **Updated** | 2026-01-18 16:11 |
| **Closed** | 2026-01-18 16:11 |

### Description

Implement query-side handler for Todo aggregate using compute-on-demand pattern.

## Pattern
- Fetch all events from EventRepository for a given aggregate
- Fold through `todo_view()` to compute current state
- Return state for query response

This bypasses MaterializedView (which requires ViewStateRepository) in favor of direct View usage with EventRepository as event source. Suitable for small event streams like the Todo canary app.

## Implementation
```rust
pub async fn query_todo_state(
    repo: &SqliteEventRepository<TodoCommand, TodoEvent>,
    todo_id: &TodoId,
) -> Result<TodoViewState, InfrastructureError> {
    let events = repo.fetch_events_by_id(todo_id).await?;
    let view = todo_view();
    let state = events.iter().fold(
        (view.initial_state)(),
        |state, (event, _version)| (view.evolve)(state, event)
    );
    Ok(state)
}
```

## Acceptance criteria
- [ ] Query handler fetches events from EventRepository
- [ ] Events folded through todo_view() to compute state
- [ ] Handler exported from application layer
- [ ] Integration test validates round-trip (command → query)

## Pattern progression
- a9b.5: Define View (pure fold function) ✓
- a9b.8: Use View for queries (full event replay) ← this issue
- 58f (future): Add MaterializedView for incremental updates when scale requires

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-a9b.5`
- 🔗 **blocked-by**: `ironstar-58f`

---

<a id="ironstar-a9b-6-implement-querysession-decider"></a>

## 📋 ironstar-a9b.6 Implement QuerySession Decider

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:18 |
| **Updated** | 2026-01-17 21:54 |
| **Closed** | 2026-01-17 21:54 |

### Description

REWRITE QuerySession domain using fmodel-rust Decider pattern. Source of truth: `spec/Analytics/QuerySession.idr`

## Files to DELETE (no preservation, no backward compatibility)

- `crates/ironstar/src/domain/query_session/aggregate.rs`

## Files to CREATE

- `crates/ironstar/src/domain/query_session/decider.rs` — query_session_decider() factory function

## Principled decisions

1. **Error handling**: Err(AggregateError) when query already running
2. **Timestamp injection**: CommandContext parameter, not Utc::now()
3. **No backward compatibility**: Delete aggregate.rs entirely

## State model (from Idris spec)

- Idle → Running → (Completed | Failed), or Cancelled → Idle

## Key pattern: spawn-after-persist

Decider is purely synchronous. DuckDB execution happens in application layer after events are persisted.

## Acceptance criteria

- [ ] DELETE crates/ironstar/src/domain/query_session/aggregate.rs
- [ ] CREATE crates/ironstar/src/domain/query_session/decider.rs
- [ ] Pure decide/evolve (no async, no I/O)
- [ ] Timestamps via CommandContext
- [ ] DeciderTestSpecification tests for all transitions


### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- 🔗 **blocked-by**: `ironstar-b43`
- 🔗 **blocked-by**: `ironstar-a9b.2`

---

<a id="ironstar-a9b-5-implement-todo-view"></a>

## 📋 ironstar-a9b.5 Implement Todo View

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:18 |
| **Updated** | 2026-01-18 00:44 |
| **Closed** | 2026-01-18 00:44 |

### Description

Implement Todo read model using fmodel-rust's View pattern.

## Implementation

```rust
pub fn todo_view<'a>() -> View<'a, TodoViewState, TodoEvent> {
    View {
        evolve: Box::new(|state, event| { /* projection logic */ }),
        initial_state: Box::new(|| TodoViewState::default()),
    }
}
```

## View State

TodoViewState for list queries:
- todos: Vec<TodoItem> (id, text, completed, created_at)
- count: usize
- completed_count: usize

## Acceptance Criteria

- [ ] View function factory in ironstar-domain/src/views/todo.rs
- [ ] Pure evolve function
- [ ] Materializes into queryable projection state

### Dependencies

- 🔗 **parent-child**: `ironstar-a9b`
- ⛔ **blocks**: `ironstar-a9b.4`

---

<a id="ironstar-a9b-implement-fmodel-rust-event-sourcing-foundation"></a>

## 🚀 ironstar-a9b Implement fmodel-rust event sourcing foundation

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-05 10:17 |
| **Updated** | 2026-01-18 23:14 |
| **Closed** | 2026-01-18 23:14 |

### Description

Implement fmodel-rust as ironstar's event sourcing foundation, replacing the custom Aggregate trait pattern.

## Principled decisions

Three design decisions govern the fmodel-rust integration:

1. **Error handling (hybrid approach)**: `Err(ValidationError)` for precondition violations that prevent command processing; `Ok(vec![FailureEvent])` for domain rejections that should be recorded in the event stream for auditability.

2. **Timestamp injection**: Application layer generates timestamps via `CommandContext`, passed into `decide()`. NO `Utc::now()` calls inside decide functions — preserves referential transparency and testability.

3. **Aggregate trait deprecation**: Immediate removal after rewrite. NO backward compatibility layer, NO feature flags, NO dual implementation period.

## Approach: rewrite from Idris specs

This is a **rewrite**, not a migration. Source of truth is the Idris specifications:
- `spec/Todo/Todo.idr` for Todo domain
- `spec/Analytics/QuerySession.idr` for QuerySession domain

Delete old aggregate implementations entirely; create fresh Decider implementations aligned with fmodel-rust patterns.

## Files to DELETE

- `crates/ironstar/src/domain/aggregate.rs` — custom Aggregate trait, AggregateRoot wrapper
- `crates/ironstar/src/domain/todo/aggregate.rs` — TodoAggregate implementation
- `crates/ironstar/src/domain/query_session/aggregate.rs` — QuerySessionAggregate implementation

## Files to CREATE

- `crates/ironstar/src/domain/todo/decider.rs` — todo_decider() factory function
- `crates/ironstar/src/domain/query_session/decider.rs` — query_session_decider() factory function
- `crates/ironstar/src/domain/context.rs` — CommandContext struct for timestamp/correlation injection

## References

- Idris specs: spec/Todo/Todo.idr, spec/Analytics/QuerySession.idr
- Evaluation: docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md


### Dependencies

- 🔗 **informs**: `ironstar-2it`

---

<a id="ironstar-3gd-4-implement-embedded-ducklake-catalog-pattern-with-rust-embed"></a>

## 📋 ironstar-3gd.4 Implement embedded DuckLake catalog pattern with rust_embed

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-28 23:54 |
| **Updated** | 2026-02-02 15:51 |
| **Closed** | 2026-02-02 15:51 |

### Description

Embed DuckLake catalogs in the ironstar binary to eliminate ATTACH latency for known datasets.

## Implementation

1. Create assets/ducklake-catalogs/ directory
2. Add rust_embed derive for DuckLakeCatalogs struct
3. Extract catalog to temp file at connection pool init
4. ATTACH locally (no network, ~0ms)

## Cache key strategy

Use build-time versioning for embedded catalogs:
```rust
const CATALOG_VERSION: &str = env!("CARGO_PKG_VERSION");
let cache_key = format!("embedded:{}:{}:{}:{:x}", 
    catalog_name, CATALOG_VERSION, table_name, query_hash);
```

No runtime ducklake_current_snapshot() query needed.

## Build integration

Copy sciexp-fixtures space.db into assets/ducklake-catalogs/ during build.
Consider nix flake to fetch from HuggingFace at build time.

## Refs

- docs/notes/architecture/infrastructure/analytics-cache-patterns.md (DuckLake catalog pattern)
- ~/projects/omicslake-workspace/sciexp-fixtures/lakes/frozen/space.db

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- 🔗 **related**: `ironstar-c7z`

---

<a id="ironstar-3gd-3-implement-cachedependency-struct-for-zenoh-based-cache-invalidation"></a>

## 📋 ironstar-3gd.3 Implement CacheDependency struct for Zenoh-based cache invalidation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-26 18:29 |
| **Updated** | 2026-02-02 15:27 |
| **Closed** | 2026-02-02 15:27 |

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

### Notes

Implemented in cache_dependency.rs — CacheDependency with Zenoh pattern matching, 19 tests passing

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- 🔗 **depends-on**: `ironstar-nyp.27`
- 🔗 **depends-on**: `ironstar-nyp.25`

---

<a id="ironstar-nyp-31-implement-health-check-endpoints-health-health-ready-health-live"></a>

## 📋 ironstar-nyp.31 Implement health check endpoints (/health, /health/ready, /health/live)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-26 15:53 |
| **Updated** | 2026-01-19 13:49 |
| **Closed** | 2026-01-19 13:49 |

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

---

<a id="ironstar-nyp-27-implement-zenoheventbus-struct-with-publish-subscribe-methods"></a>

## 📋 ironstar-nyp.27 Implement ZenohEventBus struct with publish/subscribe methods

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-26 15:25 |
| **Updated** | 2026-01-19 13:42 |
| **Closed** | 2026-01-19 13:42 |

### Description

Wrap zenoh::Session in Arc. Implement subscribe(&self, pattern: &str) returning Subscriber. Implement publish(&self, key: &str, payload: Vec<u8>) returning Result. See docs/notes/architecture/infrastructure/zenoh-event-bus.md for implementation patterns. Local refs: ~/projects/rust-workspace/zenoh

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.19`
- ⛔ **blocks**: `ironstar-2nt.2`
- ⛔ **blocks**: `ironstar-nyp.26`
- ⛔ **blocks**: `ironstar-nyp.25`

---

<a id="ironstar-nyp-25-define-zenoh-key-expression-patterns-for-event-routing"></a>

## 📋 ironstar-nyp.25 Define Zenoh key expression patterns for event routing

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2026-01-19 13:40 |
| **Closed** | 2026-01-19 13:40 |

### Description

Key expressions: ironstar/{aggregate_type}/{aggregate_id}/events for scoped pub/sub.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

---

<a id="ironstar-2nt-10-define-errorcode-enum-for-http-error-mapping"></a>

## 📋 ironstar-2nt.10 Define ErrorCode enum for HTTP error mapping

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 10:02 |
| **Updated** | 2026-01-17 00:39 |
| **Closed** | 2026-01-17 00:39 |

### Description

Implement ErrorCode enum with ValidationFailed, NotFound, Conflict, Unauthorized, etc. and http_status() method. Part of error type hierarchy. See docs/notes/architecture/decisions/error-types.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.8`

---

<a id="ironstar-nyp-19-create-eventbus-trait-abstraction"></a>

## 📋 ironstar-nyp.19 Create EventBus trait abstraction

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 00:54 |
| **Updated** | 2026-01-19 13:41 |
| **Closed** | 2026-01-19 13:41 |

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

---

<a id="ironstar-nyp-15-implement-moka-analytics-cache-with-rkyv-serialization"></a>

## 📋 ironstar-nyp.15 Implement moka analytics cache with rkyv serialization

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-23 23:23 |
| **Updated** | 2026-02-02 15:27 |
| **Closed** | 2026-02-02 15:27 |

### Description

Implement moka analytics cache with rkyv serialization, memoizing the query profunctor via TTL-based cache invalidation.

Cache structure implements memoization:
- Key: (Query, DatasetRef, ChartConfig)
- Value: QueryResult (serialized via rkyv for zero-copy)
- Invalidation: TTL (5 min) approximates naturality failure

Cache invalidation = naturality failure: cached result invalid when underlying data changes such that the naturality square no longer commutes.

Reference: denotational-semantics.md 'Analytics as quotients with memoization' section.

### Notes

Implemented in analytics_cache.rs — moka cache with rkyv serialization, 14 tests passing

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.12`
- ⛔ **blocks**: `ironstar-nyp.27`

---

<a id="ironstar-edx-review-narrative-arc-and-timing-estimates"></a>

## 📋 ironstar-edx Review narrative arc and timing estimates

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-26 20:51 |

### Description

Validate the 4-act structure (15-15-10-5 min). Ensure logical flow from problem to solution to interface to vision. Check that each slide has one clear concept.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`

---

<a id="ironstar-0tk-omicslake-presentation-slide-deck"></a>

## 🚀 ironstar-0tk Omicslake presentation slide deck

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-19 01:15 |
| **Updated** | 2025-12-26 20:51 |

### Description

Perfect the ~45 minute Omicslake presentation tracing HDF5/AnnData → DuckLake → ironstar/Datastar stack. Located in docs/slides/ironstar-overview/. Target: compelling technical narrative for genomics/data engineering audience.

---

<a id="ironstar-e6k-8-implement-todo-example-route-mounting"></a>

## 📋 ironstar-e6k.8 Implement todo example route mounting

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 18:47 |
| **Closed** | 2026-01-19 18:47 |

### Description

Create routes() function mounting todo example endpoints:

- GET /todos - render todo list via MaterializedView query
- POST /add-todo - send CreateTodo command via EventSourcedAggregate
- POST /mark-todo - send CompleteTodo command
- POST /delete-todo - send DeleteTodo command
- GET /todos-feed - SSE stream from Zenoh subscription

Wire AppState with:
- EventSourcedAggregate<TodoDecider, SqliteEventRepository> for commands
- MaterializedView<TodoView, SqliteViewRepository> for queries
- Zenoh session for SSE broadcasting

Mount under /api prefix in main Router.

Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust


### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.6`
- ⛔ **blocks**: `ironstar-e6k.3`
- ⛔ **blocks**: `ironstar-e6k.4`
- ⛔ **blocks**: `ironstar-e6k.5`
- ⛔ **blocks**: `ironstar-e6k.7`

---

<a id="ironstar-e6k-7-implement-todo-list-template-rendering-function"></a>

## 📋 ironstar-e6k.7 Implement todo_list_template rendering function

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 18:40 |
| **Closed** | 2026-01-19 18:40 |

### Description

Create hypertext function fn todo_list_template(todos: &[TodoItem]) -> impl Renderable that renders ul#todo-list with li items, checkboxes with data-on:change, delete buttons with data-on:click, and add-todo form with input data-bind. Demonstrates complete Datastar integration for todo app.
Local refs: ~/projects/rust-workspace/hypertext, ~/projects/lakescope-workspace/datastar-go-nats-template-northstar

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-r62.10`

---

<a id="ironstar-e6k-6-implement-get-todos-sse-feed-endpoint"></a>

## 📋 ironstar-e6k.6 Implement GET /todos SSE feed endpoint

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 18:47 |
| **Closed** | 2026-01-19 18:47 |

### Description

Create async handler returning Sse<impl Stream> that on initial connection sends TodoListProjection current state as PatchElements(todo_list_template(todos)), then streams incremental updates from broadcast channel. Implements Tao of Datastar principle 1 (backend is source of truth) with fat morph initial state.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.2`
- ⛔ **blocks**: `ironstar-r62.5`

---

<a id="ironstar-e6k-5-implement-delete-todo-handler-post-delete-todo"></a>

## 📋 ironstar-e6k.5 Implement delete_todo handler (POST /delete-todo)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 18:47 |
| **Closed** | 2026-01-19 18:47 |

### Description

Create async handler accepting ReadSignals<{id: Uuid}> that emits TodoDeleted event, appends to event store, broadcasts, returns 202. SSE morphs todo-list to remove deleted item or replaces entire list.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.3`

---

<a id="ironstar-e6k-4-implement-mark-todo-handler-post-mark-todo"></a>

## 📋 ironstar-e6k.4 Implement mark_todo handler (POST /mark-todo)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 18:47 |
| **Closed** | 2026-01-19 18:47 |

### Description

Create async handler accepting ReadSignals<{id: Uuid}> that emits TodoCompleted event, appends to event store, broadcasts, returns 202. SSE updates todo item to show completed state via hypertext morphing.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.3`

---

<a id="ironstar-e6k-3-implement-add-todo-handler-post-add-todo"></a>

## 📋 ironstar-e6k.3 Implement add_todo handler (POST /add-todo)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 18:47 |
| **Closed** | 2026-01-19 18:47 |

### Description

Create async handler accepting ReadSignals<AddTodoCommand> with text field. Validates non-empty, emits TodoCreated event, appends to event store, broadcasts to subscribers, returns 202. Frontend removes loading indicator via SSE update. Demonstrates write path with immediate response.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.2`
- ⛔ **blocks**: `ironstar-r62.6`

---

<a id="ironstar-e6k-2-implement-todolistprojection-with-in-memory-rebuild"></a>

## 📋 ironstar-e6k.2 Implement TodoListProjection with in-memory rebuild

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 18:47 |
| **Closed** | 2026-01-19 18:47 |

### Description

Create struct TodoListProjection(Vec<TodoItem>) implementing Projection trait. rebuild() method replays all TodoCreated/TodoCompleted/TodoDeleted events to reconstruct current state. apply() method handles incremental event updates. Demonstrates projection pattern.
Local refs: ~/projects/rust-workspace/datastar-rust-lince

### Dependencies

- 🔗 **parent-child**: `ironstar-e6k`
- ⛔ **blocks**: `ironstar-e6k.1`
- ⛔ **blocks**: `ironstar-a9b.8`

---

<a id="ironstar-e6k-1-define-todo-domain-model-aggregate-events-commands"></a>

## 📋 ironstar-e6k.1 Define Todo domain model (aggregate, events, commands)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-27 22:26 |
| **Closed** | 2025-12-27 22:26 |

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

---

<a id="ironstar-r62-13-wire-all-components-together-in-main-rs"></a>

## 📋 ironstar-r62.13 Wire all components together in main.rs

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-28 11:43 |
| **Closed** | 2026-01-28 11:43 |

### Description

Create main.rs that initializes all services and starts the axum server:

```rust
#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::from_env()?;
    
    // Initialize AppState (EventRepository, ViewRepository, Zenoh, Aggregates)
    let state = AppState::new(&config).await?;
    
    // Compose Router
    let app = Router::new()
        .merge(todo_routes())
        .merge(auth_routes())
        .layer(SessionLayer::new(state.session_store.clone()))
        .with_state(state);
    
    // Start server with graceful shutdown
    let listener = TcpListener::bind(&config.bind_address).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    Ok(())
}
```

Orchestration layer tying EventRepository, ViewRepository, EventSourcedAggregate, MaterializedView, and Zenoh together.

Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/tokio


### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.12`

### Comments

> **Cameron Smith** (2026-01-20)
>
> Checkpoint 2026-01-20: Implementation complete, pending commit.
> 
> Done:
> - Transformed placeholder main.rs into production composition root
> - 10-step startup sequence with proper error handling
> - Config struct with IRONSTAR_* env vars
> - SQLite pool with auto-migration
> - Zenoh embedded mode initialization (graceful fallback)
> - Router composition with FromRef pattern
> - Graceful shutdown handling
> 
> Files created/modified:
> - crates/ironstar/src/config.rs (new)
> - crates/ironstar/src/state.rs (new)
> - crates/ironstar/src/lib.rs (exports)
> - crates/ironstar/src/main.rs (full implementation)
> 
> Blocked: Git commits failing due to SSH signing key passphrase issue.

---

<a id="ironstar-r62-10-implement-component-level-hypertext-templates"></a>

## 📋 ironstar-r62.10 Implement component-level hypertext templates

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 18:37 |
| **Closed** | 2026-01-19 18:37 |

### Description

Create reusable component functions (e.g., button, form_field, loading_spinner) returning impl Renderable. Components accept data and emit proper Datastar attributes (data-on:, data-show, data-bind). These compose into page templates.
Local refs: ~/projects/rust-workspace/hypertext

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.9`
- ⛔ **blocks**: `ironstar-ny3.18`

### Comments

> **crs58** (2026-01-06)
>
> Note: Templates should use CUBE CSS composition classes (.stack, .cluster, .center, .sidebar, .switcher, .box, .cover, .grid) for layout. See ironstar-ny3.18 for composition layer implementation and docs/notes/architecture/frontend/css-architecture.md for usage patterns.

> **Cameron Smith** (2026-01-19)
>
> Implementation complete. Review identified XSS concern with Raw::dangerously_create() but threat model is limited: parameters are hardcoded developer code, not user input. User data flows through content rendering with proper maud escaping. Follow-up: consider migrating to maud! macro for consistency with layout.rs pattern.

---

<a id="ironstar-nyp-11-create-session-axum-extractor"></a>

## 📋 ironstar-nyp.11 Create Session axum extractor

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-20 00:14 |
| **Closed** | 2026-01-20 00:14 |

### Description

Implement FromRequestParts for Session type extracting session_id from CookieJar. Load or initialize SessionData from SessionStore. Return Session struct with id and data fields for use in handlers.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.10`

### Comments

> **Cameron Smith** (2026-01-20)
>
> Implemented in commits 70fb541..fb25588. SessionExtractor with FromRequestParts, SessionRejection error types, session_cookie helper, 9 tests.

---

<a id="ironstar-nyp-10-add-session-ttl-cleanup-background-task"></a>

## 📋 ironstar-nyp.10 Add session TTL cleanup background task

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 23:53 |
| **Closed** | 2026-01-19 23:53 |

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

---

<a id="ironstar-nyp-9-implement-sqlite-session-store-with-sessionstore-trait"></a>

## 📋 ironstar-nyp.9 Implement SQLite session store with SessionStore trait

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 23:29 |
| **Closed** | 2026-01-19 23:29 |

### Description

Implement SessionStore trait with SQLite backend. Schema: id TEXT PRIMARY KEY, user_id TEXT, created_at, last_seen_at, expires_at TIMESTAMP, data JSON. Methods: create, get, update_data, touch, delete, cleanup_expired. Use 24-byte cryptographic session IDs. See docs/notes/architecture/infrastructure/session-management.md and session-implementation.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`

---

<a id="ironstar-nyp-7-implement-projectionmanager-with-in-memory-state"></a>

## 📋 ironstar-nyp.7 Implement ProjectionManager with in-memory state

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-06 13:46 |
| **Closed** | 2026-01-06 13:46 |

### Description

Create ProjectionManager using fmodel-rust's MaterializedView pattern:

```rust
struct ProjectionManager<V: View> {
    materialized_view: MaterializedView<V, ViewStateRepository>,
    zenoh_session: Arc<Session>,
}

impl<V: View> ProjectionManager<V> {
    async fn init(
        event_repo: Arc<dyn EventRepository>,
        view_repo: Arc<dyn ViewStateRepository>,
        zenoh: Arc<Session>,
    ) -> Self {
        // Subscribe to Zenoh key expressions for incremental updates
        let subscriber = zenoh.subscribe("events/**").await;
        // MaterializedView handles state persistence
    }
    
    async fn query<Q>(&self, query: Q) -> V::State {
        self.materialized_view.fetch_state(&query).await
    }
}
```

Replays events via EventRepository to build initial state, then subscribes to Zenoh for incremental updates.
Uses MaterializedView + ViewStateRepository from fmodel-rust.

Note: Consider if this duplicates ironstar-a9b.8 (Wire Todo MaterializedView) - may need consolidation.

Local refs: ~/projects/rust-workspace/tokio, ~/projects/rust-workspace/fmodel-rust


### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.6`
- ⛔ **blocks**: `ironstar-nyp.5`
- ⛔ **blocks**: `ironstar-nyp.3`
- ⛔ **blocks**: `ironstar-nyp.27`

---

<a id="ironstar-nyp-3-implement-sqlite-eventrepository-with-sqlx"></a>

## 📋 ironstar-nyp.3 Implement SQLite EventRepository with sqlx

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-06 13:46 |
| **Closed** | 2026-01-06 13:46 |

### Description

Create SqliteEventRepository struct implementing fmodel-rust's EventRepository trait:

```rust
impl<C, E> EventRepository<C, E, Uuid, EventStoreError> for SqliteEventRepository
where
    C: Identifier + DeciderType + Sync,
    E: Identifier + EventType + DeciderType + IsFinal + Serialize + DeserializeOwned + Clone + Sync,
{
    fn fetch_events(&self, command: &C) -> impl Future<Output = Result<Vec<(E, Uuid)>, Error>>;
    fn save(&self, events: &[E]) -> impl Future<Output = Result<Vec<(E, Uuid)>, Error>>;
    fn version_provider(&self, event: &E) -> impl Future<Output = Result<Option<Uuid>, Error>>;
}
```

Extension methods for SSE Last-Event-ID support:
- query_since_sequence(offset: i64) -> Vec<StoredEvent>
- earliest_sequence() -> Option<i64>
- latest_sequence() -> Option<i64>
- query_all() -> Vec<StoredEvent>

Use sqlx compile-time query validation with query!() macro.
This is a wrapper/extension of ironstar-a9b.1 for projection support.

Local refs: ~/projects/rust-workspace/sqlx


### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.1`
- ⛔ **blocks**: `ironstar-a9b.1`

---

<a id="ironstar-ny3-14-create-web-components-components-directory-for-vanilla-web-components"></a>

## 📋 ironstar-ny3.14 Create web-components/components/ directory for vanilla web components

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-02-03 00:02 |
| **Closed** | 2026-02-03 00:02 |

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

---

<a id="ironstar-2nt-8-define-application-error-types"></a>

## 📋 ironstar-2nt.8 Define application error types

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-17 00:33 |
| **Closed** | 2026-01-17 00:33 |

### Description

Define layered error type hierarchy: ValidationError (field-level), DomainError (business rules), AggregateError (command handling), InfrastructureError (storage/bus), AppError (HTTP boundary). See docs/notes/architecture/decisions/error-types.md for complete type definitions and From implementations.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`
- ⛔ **blocks**: `ironstar-2nt.2`

---

<a id="ironstar-o66-4-remove-ironstar-release-from-ci-packages-build"></a>

## 📋 ironstar-o66.4 Remove ironstar-release from CI packages build

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-02-04 21:51 |
| **Updated** | 2026-02-05 00:59 |
| **Closed** | 2026-02-05 00:59 |

### Description

Remove ironstar-release from CI packages build. No crate publishing or production deployment exists yet. Filter in ci-build-category.sh or remove from packages output. The dev-profile ironstar package validates compilation correctness. This is independent of the rust-flake removal. Reference: modules/rust.nix ironstar-release definition, scripts/ci/ci-build-category.sh packages iteration.

### Dependencies

- 🔗 **parent-child**: `ironstar-o66`
- ⛔ **blocks**: `ironstar-o66.1`

---

<a id="ironstar-507-3-implement-session-decider-specification-tests"></a>

## 📋 ironstar-507.3 Implement Session Decider specification tests

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-18 12:56 |
| **Updated** | 2026-01-18 22:08 |
| **Closed** | 2026-01-18 22:08 |

### Description

Implement given/when/then specification tests for Session Decider.

## Test coverage

- CreateSession success (NoSession → Active)
- CreateSession failure (already active, expired, invalidated)
- RefreshSession success (Active → Active with new expiry)
- RefreshSession failure (no session, expired, invalidated, ID mismatch)
- InvalidateSession success (Active → Invalidated)
- InvalidateSession failure (no session, expired, already invalidated, ID mismatch)

## Pattern

```rust
use fmodel_rust::specification::DeciderTestSpecification;

#[test]
fn create_session_succeeds() {
    DeciderTestSpecification::default()
        .for_decider(session_decider())
        .given(vec![])
        .when(SessionCommand::CreateSession { ... })
        .then(vec![SessionEvent::SessionCreated { ... }]);
}
```

## Acceptance criteria

- [ ] All command success paths tested
- [ ] All error conditions tested
- [ ] State transition coverage complete
- [ ] Tests pass with cargo nextest

## References

- ironstar-a9b.12 task description
- spec/Session/Session.idr (decide function patterns)

### Dependencies

- 🔗 **parent-child**: `ironstar-507`

---

<a id="ironstar-7a2-14-implement-workspace-view-specification-tests"></a>

## 📋 ironstar-7a2.14 Implement Workspace View specification tests

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-18 12:27 |
| **Updated** | 2026-02-03 18:09 |
| **Closed** | 2026-02-03 18:09 |

### Description

Implement specification tests for Workspace MaterializedView projections following ironstar-a9b.13 pattern.

## Test coverage per view

For each view, test:
- Initial state correctness
- Event folding produces expected projections
- Invariant preservation on edge cases (delete non-existent, etc.)
- Multi-event sequences

## Pattern

```rust
use fmodel_rust::view::ViewStateComputation;

#[test]
fn workspace_list_view_projects_created_event() {
    let view = workspace_list_view();
    let initial = (view.initial_state)();
    
    let event = WorkspaceEvent::Created { ... };
    let projected = view.compute_new_state(&initial, &[&event]);
    
    assert_eq!(projected.workspaces.len(), 1);
    assert_eq!(projected.workspaces[0].name, "Test Workspace");
}

#[test]
fn workspace_list_view_preserves_invariants_on_delete_nonexistent() {
    let view = workspace_list_view();
    let state = WorkspaceListState { workspaces: vec![...], count: 1 };
    
    let event = WorkspaceEvent::Deleted { id: nonexistent_id, ... };
    let projected = view.compute_new_state(&state, &[&event]);
    
    // Invariant preserved: count == workspaces.len()
    assert_eq!(projected.count, projected.workspaces.len());
}
```

## Acceptance criteria

- [ ] WorkspaceListView projection tests
- [ ] DashboardLayoutView projection tests
- [ ] SavedQueryListView projection tests
- [ ] UserPreferencesView projection tests
- [ ] Invariant preservation tests for all views

## References

- ironstar-a9b.13 task description
- crates/ironstar/tests/ (Todo view tests reference)

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.9`

---

<a id="ironstar-7a2-13-implement-workspace-decider-specification-tests"></a>

## 📋 ironstar-7a2.13 Implement Workspace Decider specification tests

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-18 12:27 |
| **Updated** | 2026-02-03 18:08 |
| **Closed** | 2026-02-03 18:08 |

### Description

Implement given/when/then specification tests for all Workspace Deciders following ironstar-a9b.12 pattern.

## Test coverage per aggregate

For each of the 5 aggregates, test:
- All command success paths
- All error conditions
- Idempotency behavior (where applicable)
- Full lifecycle sequences

## Pattern

```rust
use fmodel_rust::specification::DeciderTestSpecification;

#[test]
fn create_workspace_succeeds() {
    DeciderTestSpecification::default()
        .for_decider(workspace_decider())
        .given(vec![])
        .when(WorkspaceCommand::Create { ... })
        .then(vec![WorkspaceEvent::Created { ... }]);
}

#[test]
fn create_workspace_when_exists_fails() {
    DeciderTestSpecification::default()
        .for_decider(workspace_decider())
        .given(vec![WorkspaceEvent::Created { ... }])
        .when(WorkspaceCommand::Create { ... })
        .then_error(WorkspaceError::already_exists());
}
```

## Acceptance criteria

- [ ] WorkspaceAggregate decider tests
- [ ] WorkspacePreferences decider tests
- [ ] Dashboard decider tests
- [ ] SavedQuery decider tests
- [ ] UserPreferences decider tests
- [ ] All tests pass with cargo nextest

## References

- ironstar-a9b.12 task description
- crates/ironstar/tests/ (Todo decider tests reference)

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.2`
- ⛔ **blocks**: `ironstar-7a2.3`
- ⛔ **blocks**: `ironstar-7a2.4`
- ⛔ **blocks**: `ironstar-7a2.5`
- ⛔ **blocks**: `ironstar-7a2.6`

---

<a id="ironstar-7a2-12-implement-workspace-query-handlers"></a>

## 📋 ironstar-7a2.12 Implement Workspace query handlers

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-18 12:27 |
| **Updated** | 2026-02-03 18:06 |
| **Closed** | 2026-02-03 18:06 |

### Description

Implement axum query handlers (GET routes) for Workspace bounded context following ironstar-a9b.11 pattern.

## Routes to implement

- GET /workspaces (list workspaces for user)
- GET /workspace/{id} (workspace details)
- GET /workspace/{id}/dashboard/{name} (dashboard layout)
- GET /workspace/{id}/queries (list saved queries)
- GET /user/preferences (user preferences)

## Pattern

```rust
pub async fn get_workspaces(
    State(view): State<Arc<WorkspaceListView>>,
    Extension(user_id): Extension<UserId>,
) -> Result<impl IntoResponse, AppError> {
    let state = view.current_state().await?;
    let filtered = state.workspaces_for_user(&user_id);
    Ok(Json(filtered))
}
```

## Acceptance criteria

- [ ] Query handlers for all read models
- [ ] User-scoped filtering where appropriate
- [ ] SSE endpoints for real-time updates
- [ ] Integration with MaterializedView projections

## References

- ironstar-a9b.11 task description

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.9`

---

<a id="ironstar-7a2-11-implement-workspace-command-handlers"></a>

## 📋 ironstar-7a2.11 Implement Workspace command handlers

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-18 12:27 |
| **Updated** | 2026-02-03 15:51 |
| **Closed** | 2026-02-03 15:51 |

### Description

Implement axum command handlers (POST routes) for Workspace bounded context following ironstar-a9b.10 pattern.

## Routes to implement

- POST /workspace (CreateWorkspace)
- POST /workspace/{id}/rename (RenameWorkspace)
- POST /workspace/{id}/visibility (SetVisibility)
- POST /workspace/{id}/dashboard (CreateDashboard)
- POST /workspace/{id}/dashboard/{name}/chart (AddChart)
- POST /workspace/{id}/query (SaveQuery)
- POST /user/preferences (InitializePreferences, SetTheme, SetLocale)

## Pattern

```rust
pub async fn create_workspace(
    State(aggregate): State<Arc<WorkspaceAggregate>>,
    Json(payload): Json<CreateWorkspacePayload>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();  // Timestamp injection at boundary
    let command = WorkspaceCommand::Create { ..., created_at: now };
    let events = aggregate.handle(&command).await?;
    Ok(Json(events))
}
```

## Acceptance criteria

- [ ] Command handlers for all 5 aggregates
- [ ] Timestamp injection at boundary
- [ ] Proper error response mapping
- [ ] Integration with EventSourcedAggregate

## References

- ironstar-a9b.10 task description

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.8`
- ⛔ **blocks**: `ironstar-7a2.10`

---

<a id="ironstar-7a2-10-integrate-zenoh-event-publishing-for-workspace"></a>

## 📋 ironstar-7a2.10 Integrate Zenoh event publishing for Workspace

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-18 12:27 |
| **Updated** | 2026-02-03 15:38 |
| **Closed** | 2026-02-03 15:38 |

### Description

Integrate Zenoh pub/sub for Workspace bounded context event distribution following ironstar-a9b.9 pattern.

## Key expression patterns

```
events/Workspace/{workspace_id}
events/Workspace/{workspace_id}/preferences
events/Workspace/{workspace_id}/dashboard/{name}
events/Workspace/{workspace_id}/query/{name}
events/User/{user_id}/preferences
```

## Location

`crates/ironstar/src/infrastructure/event_bus/workspace.rs`

## Acceptance criteria

- [ ] WorkspaceEventPublisher for all 5 aggregate types
- [ ] Key expression routing by aggregate type and ID
- [ ] Subscriber factory for SSE feed integration
- [ ] Cache invalidation hooks

## References

- ironstar-a9b.9 task description
- docs/notes/architecture/infrastructure/zenoh-event-bus.md

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.8`

---

<a id="ironstar-7a2-9-wire-workspace-materializedview-projections"></a>

## 📋 ironstar-7a2.9 Wire Workspace MaterializedView projections

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-18 12:27 |
| **Updated** | 2026-02-03 15:08 |
| **Closed** | 2026-02-03 15:08 |

### Description

Implement MaterializedView projections for Workspace bounded context read models following ironstar-a9b.8 pattern.

## Views to implement

1. **WorkspaceListView**: List of workspaces for a user
2. **DashboardLayoutView**: Full dashboard state for rendering
3. **SavedQueryListView**: List of saved queries in a workspace
4. **UserPreferencesView**: Current user preferences

## Location

`crates/ironstar/src/domain/views/workspace.rs`

## Acceptance criteria

- [ ] WorkspaceListView with user filtering
- [ ] DashboardLayoutView with full tab/chart state
- [ ] SavedQueryListView with workspace filtering
- [ ] UserPreferencesView singleton per user
- [ ] All views preserve invariants on edge cases

## References

- ironstar-a9b.8 task description
- crates/ironstar/src/domain/views/todo.rs (reference implementation)

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.2`
- ⛔ **blocks**: `ironstar-7a2.3`
- ⛔ **blocks**: `ironstar-7a2.4`
- ⛔ **blocks**: `ironstar-7a2.5`
- ⛔ **blocks**: `ironstar-7a2.6`

### Comments

> **Cameron Smith** (2026-02-03)
>
> Checkpoint: 7a2.8 complete, 7a2.9 next. All 5 workspace handler modules exist in application layer. MaterializedView projections should follow the catalog queries pattern (application/catalog/queries.rs). No code written yet for 7a2.9.

---

<a id="ironstar-7a2-8-wire-workspace-eventsourcedaggregate-to-sqlite"></a>

## 📋 ironstar-7a2.8 Wire Workspace EventSourcedAggregate to SQLite

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-18 12:26 |
| **Updated** | 2026-02-03 10:43 |
| **Closed** | 2026-02-03 10:43 |

### Description

Wire Workspace Deciders to SQLite event store (following a9b.7 pattern)

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.2`
- ⛔ **blocks**: `ironstar-7a2.3`
- ⛔ **blocks**: `ironstar-7a2.4`
- ⛔ **blocks**: `ironstar-7a2.5`
- ⛔ **blocks**: `ironstar-7a2.6`

---

<a id="ironstar-1ks-wire-workspace-eventsourcedaggregate-to-sqlite"></a>

## 📋 ironstar-1ks Wire Workspace EventSourcedAggregate to SQLite

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2026-01-18 12:25 |
| **Updated** | 2026-01-18 12:26 |

### Description

Wire Workspace bounded context Deciders to SQLite event store using fmodel-rust EventSourcedAggregate pattern.

## Context

Following ironstar-a9b.7 (Wire Todo EventSourcedAggregate), this task connects the Workspace Deciders to SQLite persistence.

## Implementation requirements

**Location**: `crates/ironstar/src/infrastructure/event_store/workspace.rs`

**Per-aggregate event stores**: Create separate EventRepository implementations for each Workspace aggregate:
- WorkspaceEventStore (for WorkspaceAggregate)
- DashboardEventStore
- SavedQueryEventStore
- WorkspacePreferencesEventStore
- UserPreferencesEventStore

**Aggregate ID routing**: Use the aggregate ID patterns defined in the Deciders.

**Optimistic locking**: Use `previous_id` chain from event store schema.

## Acceptance criteria

- [ ] EventRepository trait implementations for all 5 aggregates
- [ ] Aggregate ID parsing and routing
- [ ] Optimistic locking via previous_id
- [ ] Integration tests with in-memory SQLite

## References

- ironstar-a9b.7 task description
- crates/ironstar/src/infrastructure/ (Todo event store reference)

---

<a id="ironstar-7a2-7-implement-workspacecontextdecider-composition"></a>

## 📋 ironstar-7a2.7 Implement workspaceContextDecider composition

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-16 19:13 |
| **Updated** | 2026-02-03 10:32 |
| **Closed** | 2026-02-03 10:32 |

### Description

Implement workspaceContextDecider composition using fmodel-rust combine pattern.

## Context

The Workspace bounded context contains 5 aggregates that can be composed into a single context-level Decider using fmodel-rust's monoidal composition. This task implements the composition if needed for unified command routing.

## Design decision

**Option A (Recommended)**: Manage each aggregate separately with distinct routes
- Each aggregate has its own HTTP routes (e.g., POST /workspace, POST /workspace/{id}/dashboard)
- Simpler routing, clearer API surface
- No need for Sum5 type complexity
- **This is how a9b (Todo) works**

**Option B**: Unified workspaceContextDecider via combine5
- Single entry point for all Workspace commands
- Commands tagged with Sum5 variants for routing
- State is product type (ws, prefs, dashboard, query, user_prefs)
- More complex but enables cross-aggregate transactions

## Implementation (if Option B chosen)

**Location**: `crates/ironstar/src/domain/workspace/context.rs`

```rust
use fmodel_rust::decider::combine5;

pub type WorkspaceContextCommand = Sum5<
    WorkspaceCommand,
    WorkspacePreferencesCommand,
    DashboardCommand,
    SavedQueryCommand,
    UserPreferencesCommand,
>;

pub type WorkspaceContextState = (
    WorkspaceState,
    WorkspacePreferencesState,
    DashboardState,
    SavedQueryState,
    UserPreferencesState,
);

pub type WorkspaceContextEvent = Sum5<
    WorkspaceEvent,
    WorkspacePreferencesEvent,
    DashboardEvent,
    SavedQueryEvent,
    UserPreferencesEvent,
>;

pub fn workspace_context_decider<'a>() -> WorkspaceContextDecider<'a> {
    combine5(
        workspace_decider(),
        workspace_preferences_decider(),
        dashboard_decider(),
        saved_query_decider(),
        user_preferences_decider(),
    )
}
```

## Recommendation

Start with **Option A** (separate aggregates, separate routes). The combine5 composition is demonstrated in the Idris2 spec to prove mathematical correctness, but production systems typically manage aggregates independently for simpler routing and clearer boundaries.

If cross-aggregate transactions become necessary, revisit this task to implement Option B.

## Acceptance criteria

- [ ] Decision documented: Option A (separate) vs Option B (combined)
- [ ] If Option B: Sum5 type aliases defined
- [ ] If Option B: workspace_context_decider() factory implemented
- [ ] If Option B: Tests verify command routing to correct sub-decider

## References

- spec/Workspace/Workspace.idr (combine5 formal specification)
- fmodel-rust combine3/combine5 documentation

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.2`
- ⛔ **blocks**: `ironstar-7a2.3`
- ⛔ **blocks**: `ironstar-7a2.4`
- ⛔ **blocks**: `ironstar-7a2.5`
- ⛔ **blocks**: `ironstar-7a2.6`

### Comments

> **Cameron Smith** (2026-02-03)
>
> Decision: Option A — separate aggregates with independent routes and event streams. The combine5 composition (Option B) adds Sum5 type complexity without benefit when each aggregate has independent HTTP routes and no cross-aggregate transaction requirements. This matches the established a9b (Todo) pattern.

---

<a id="ironstar-7a2-6-implement-userpreferences-aggregate-user-scoped-only"></a>

## 📋 ironstar-7a2.6 Implement UserPreferences aggregate (user-scoped only)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-16 12:06 |
| **Updated** | 2026-02-03 09:33 |
| **Closed** | 2026-02-03 09:33 |

### Description

Implement UserPreferences Decider following fmodel-rust patterns from ironstar-a9b.

## Context

UserPreferences manages user-scoped personal settings that apply across all workspaces: theme, locale, and UI state. Each user has exactly one preferences aggregate.

## Implementation requirements

**Location**: `crates/ironstar/src/domain/user_preferences/`

**Module structure**:
```
user_preferences/
├── mod.rs
├── decider.rs          # user_preferences_decider() factory
├── commands.rs         # UserPreferencesCommand enum
├── events.rs           # UserPreferencesEvent enum
├── state.rs            # UserPreferencesState
├── errors.rs           # UserPreferencesError
└── values.rs           # Theme, Locale, UiState
```

**Commands** (from spec/Workspace/UserPreferences.idr):
- `InitializePreferences { user_id: UserId, initialized_at: DateTime<Utc> }`
- `SetTheme { user_id: UserId, theme: Theme, set_at: DateTime<Utc> }`
- `SetLocale { user_id: UserId, locale: Locale, set_at: DateTime<Utc> }`
- `UpdateUiState { user_id: UserId, ui_state: UiState, updated_at: DateTime<Utc> }`

**Events**:
- `UserPreferencesInitialized { user_id: UserId, initialized_at: DateTime<Utc> }`
- `ThemeSet { user_id: UserId, theme: Theme, set_at: DateTime<Utc> }`
- `LocaleSet { user_id: UserId, locale: Locale, set_at: DateTime<Utc> }`
- `UiStateUpdated { user_id: UserId, ui_state: UiState, updated_at: DateTime<Utc> }`

**Aggregate ID pattern**: `user_{user_id}/preferences` (singleton per user)

**Value objects**:
```rust
pub enum Theme { Light, Dark, System }
pub struct Locale(String);  // e.g., "en-US", validated via smart constructor
pub struct UiState(serde_json::Value);  // Opaque JSON for extensibility
```

**Idempotency**: All operations after initialization are idempotent.

## Acceptance criteria

- [ ] UserPreferencesCommand enum with routing traits
- [ ] UserPreferencesEvent enum with event traits
- [ ] UserPreferencesState with preferences fields
- [ ] user_preferences_decider() factory function
- [ ] Value objects: Theme, Locale, UiState
- [ ] All types export to TypeScript via ts-rs

## References

- spec/Workspace/UserPreferences.idr (formal specification)
- crates/ironstar/src/domain/todo/decider.rs (reference pattern)

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.1`

---

<a id="ironstar-7a2-5-implement-savedquery-aggregate-with-workspaceid-scope"></a>

## 📋 ironstar-7a2.5 Implement SavedQuery aggregate with workspaceId scope

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-16 12:06 |
| **Updated** | 2026-02-03 09:33 |
| **Closed** | 2026-02-03 09:33 |

### Description

Implement SavedQuery Decider following fmodel-rust patterns from ironstar-a9b.

## Context

SavedQuery manages named DuckDB queries with dataset references within a workspace. Queries can be saved, renamed, updated, and deleted.

## Implementation requirements

**Location**: `crates/ironstar/src/domain/saved_query/`

**Module structure**:
```
saved_query/
├── mod.rs
├── decider.rs          # saved_query_decider() factory
├── commands.rs         # SavedQueryCommand enum
├── events.rs           # SavedQueryEvent enum
├── state.rs            # SavedQueryState + SavedQueryStatus
├── errors.rs           # SavedQueryError
└── values.rs           # SavedQueryId, QueryName, SqlQuery, DatasetRef
```

**Commands** (from spec/Workspace/SavedQuery.idr):
- `SaveQuery { workspace_id: WorkspaceId, name: String, sql: String, dataset_ref: Option<DatasetRef>, saved_at: DateTime<Utc> }`
- `RenameSavedQuery { query_id: SavedQueryId, new_name: String, renamed_at: DateTime<Utc> }`
- `DeleteSavedQuery { query_id: SavedQueryId, deleted_at: DateTime<Utc> }`
- `UpdateSavedQuerySql { query_id: SavedQueryId, new_sql: String, updated_at: DateTime<Utc> }`
- `UpdateDatasetReference { query_id: SavedQueryId, dataset_ref: Option<DatasetRef>, updated_at: DateTime<Utc> }`

**Events**:
- `SavedQueryCreated { id: SavedQueryId, workspace_id: WorkspaceId, name: QueryName, sql: SqlQuery, dataset_ref: Option<DatasetRef>, saved_at: DateTime<Utc> }`
- `SavedQueryRenamed`, `SavedQueryDeleted`, `SavedQuerySqlUpdated`, `DatasetRefUpdated`

**State machine**:
```rust
pub enum SavedQueryStatus {
    NoQuery,
    QueryExists,
    QueryDeleted,  // Terminal state
}
```

**Aggregate ID pattern**: `workspace_{workspace_id}/query_{query_name}`

**Validation in decide()**: Name and SQL cannot be empty (return `Err(SavedQueryError::empty_name())` or `Err(SavedQueryError::empty_sql())`).

**Cross-context reference**: `DatasetRef` references DuckDB catalog/dataset. Validation that dataset exists happens at runtime (DuckDB query execution), not in the pure decider.

**NOT idempotent**: SaveQuery on existing query returns error (use UpdateSavedQuerySql instead). DeleteSavedQuery on deleted query returns error.

## Acceptance criteria

- [ ] SavedQueryCommand enum with routing traits
- [ ] SavedQueryEvent enum with event traits  
- [ ] SavedQueryState with query metadata
- [ ] saved_query_decider() factory function
- [ ] Value objects: SavedQueryId, QueryName, SqlQuery, DatasetRef
- [ ] Validation for non-empty name and SQL
- [ ] All types export to TypeScript via ts-rs

## References

- spec/Workspace/SavedQuery.idr (formal specification)
- crates/ironstar/src/domain/todo/decider.rs (reference pattern)

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.2`

---

<a id="ironstar-7a2-4-implement-dashboard-aggregate-with-workspaceid-scope"></a>

## 📋 ironstar-7a2.4 Implement Dashboard aggregate with workspaceId scope

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-16 12:06 |
| **Updated** | 2026-02-03 09:33 |
| **Closed** | 2026-02-03 09:33 |

### Description

Implement Dashboard Decider following fmodel-rust patterns from ironstar-a9b.

## Context

Dashboard manages chart placement and layout configuration within a workspace. Each workspace can have multiple dashboards, each with tabs and chart placements.

## Implementation requirements

**Location**: `crates/ironstar/src/domain/dashboard/`

**Module structure**:
```
dashboard/
├── mod.rs
├── decider.rs          # dashboard_decider() factory
├── commands.rs         # DashboardCommand enum
├── events.rs           # DashboardEvent enum
├── state.rs            # DashboardState + DashboardStatus
├── errors.rs           # DashboardError
└── values.rs           # DashboardId, DashboardName, TabId, ChartId, GridPosition
```

**Commands** (from spec/Workspace/Dashboard.idr):
- `CreateDashboard { workspace_id: WorkspaceId, name: String, created_at: DateTime<Utc> }`
- `RenameDashboard { dashboard_id: DashboardId, new_name: String, renamed_at: DateTime<Utc> }`
- `AddTab { dashboard_id: DashboardId, tab_name: String, added_at: DateTime<Utc> }`
- `RemoveTab { dashboard_id: DashboardId, tab_id: TabId, removed_at: DateTime<Utc> }`
- `AddChart { dashboard_id: DashboardId, tab_id: TabId, chart_ref: ChartDefinitionRef, position: GridPosition, added_at: DateTime<Utc> }`
- `RemoveChart { dashboard_id: DashboardId, chart_id: ChartId, removed_at: DateTime<Utc> }`
- `MoveChartToTab { dashboard_id: DashboardId, chart_id: ChartId, target_tab_id: TabId, new_position: GridPosition, moved_at: DateTime<Utc> }`

**Events**:
- `DashboardCreated`, `DashboardRenamed`
- `TabAdded`, `TabRemoved`
- `ChartAdded`, `ChartRemoved`, `ChartMovedToTab`

**State machine**:
```rust
pub enum DashboardStatus {
    NoDashboard,
    DashboardExists,
}
```

**Aggregate ID pattern**: `workspace_{workspace_id}/dashboard_{dashboard_name}`

**Cross-context reference**: `ChartDefinitionRef` references Analytics.ChartDefinition (Customer-Supplier pattern). Validation that the referenced chart exists happens at the boundary, not in the pure decider.

**Idempotency**: RemoveChart on non-existent chart returns `Ok(vec![])` (idempotent). RemoveTab requires tab to be empty or relocate charts first.

## Acceptance criteria

- [ ] DashboardCommand enum with routing traits
- [ ] DashboardEvent enum with event traits
- [ ] DashboardState with tabs and chart placements
- [ ] dashboard_decider() factory function
- [ ] Value objects: DashboardId, DashboardName, TabId, ChartId, GridPosition, ChartDefinitionRef
- [ ] All types export to TypeScript via ts-rs

## References

- spec/Workspace/Dashboard.idr (formal specification)
- crates/ironstar/src/domain/todo/decider.rs (reference pattern)

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.2`

---

<a id="ironstar-7a2-3-implement-workspacepreferences-aggregate"></a>

## 📋 ironstar-7a2.3 Implement WorkspacePreferences aggregate

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-16 12:06 |
| **Updated** | 2026-02-03 00:32 |
| **Closed** | 2026-02-03 00:32 |

### Description

Implement WorkspacePreferences Decider following fmodel-rust patterns from ironstar-a9b.

## Context

WorkspacePreferences manages workspace-scoped settings: default catalog URI and layout defaults. It is scoped to a single workspace via aggregate ID pattern `workspace_{id}/preferences`.

## Implementation requirements

**Location**: `crates/ironstar/src/domain/workspace_preferences/`

**Module structure**:
```
workspace_preferences/
├── mod.rs
├── decider.rs          # workspace_preferences_decider() factory
├── commands.rs         # WorkspacePreferencesCommand enum
├── events.rs           # WorkspacePreferencesEvent enum
├── state.rs            # WorkspacePreferencesState
├── errors.rs           # WorkspacePreferencesError
└── values.rs           # CatalogUri, LayoutDefaults
```

**Commands** (from spec/Workspace/WorkspacePreferences.idr):
- `InitializeWorkspacePreferences { workspace_id: WorkspaceId, initialized_at: DateTime<Utc> }`
- `SetDefaultCatalog { workspace_id: WorkspaceId, catalog_uri: CatalogUri, set_at: DateTime<Utc> }`
- `ClearDefaultCatalog { workspace_id: WorkspaceId, cleared_at: DateTime<Utc> }`
- `UpdateLayoutDefaults { workspace_id: WorkspaceId, layout_defaults: LayoutDefaults, updated_at: DateTime<Utc> }`

**Events**:
- `WorkspacePreferencesInitialized { workspace_id: WorkspaceId, initialized_at: DateTime<Utc> }`
- `DefaultCatalogSet { workspace_id: WorkspaceId, catalog_uri: CatalogUri, set_at: DateTime<Utc> }`
- `DefaultCatalogCleared { workspace_id: WorkspaceId, cleared_at: DateTime<Utc> }`
- `LayoutDefaultsUpdated { workspace_id: WorkspaceId, layout_defaults: LayoutDefaults, updated_at: DateTime<Utc> }`

**Aggregate ID pattern**: `workspace_{workspace_id}/preferences`

**Idempotency**: All operations after initialization are idempotent (setting same value returns empty event list).

## Acceptance criteria

- [ ] WorkspacePreferencesCommand enum with routing traits
- [ ] WorkspacePreferencesEvent enum with event traits
- [ ] WorkspacePreferencesState with initialization tracking
- [ ] workspace_preferences_decider() factory function
- [ ] Value objects: CatalogUri, LayoutDefaults
- [ ] All types export to TypeScript via ts-rs

## References

- spec/Workspace/WorkspacePreferences.idr (formal specification)
- crates/ironstar/src/domain/todo/decider.rs (reference pattern)

### Dependencies

- 🔗 **parent-child**: `ironstar-7a2`
- ⛔ **blocks**: `ironstar-7a2.2`

---

<a id="ironstar-2nt-19-add-smart-constructors-for-refined-types"></a>

## 📋 ironstar-2nt.19 Add smart constructors for refined types

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-15 12:38 |
| **Updated** | 2026-01-17 00:38 |
| **Closed** | 2026-01-17 00:38 |

### Description

Add smart constructors for invariant enforcement per cross-layer audit D8/D14: GridSize (width >= 1, height >= 1), BoundedString pattern for titles (TodoTitle 1-500, DashboardTitle 1-200, TabTitle 1-100). Use opaque types with private MkX constructors and public smart constructor functions that return Maybe/Either.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`

---

<a id="ironstar-2nt-18-add-missing-enums-to-idris-specifications"></a>

## 📋 ironstar-2nt.18 Add missing enums to Idris specifications

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-15 12:38 |
| **Updated** | 2026-01-17 00:33 |
| **Closed** | 2026-01-17 00:33 |

### Description

Add enums identified in cross-layer audit D13: RevocationReason (UserLogout, AdminAction, SecurityConcern), QueryErrorCode (SyntaxError, PermissionDenied, Timeout, ResourceExhausted, InternalError), SessionStatus (Active, Expired, Revoked), QueryStatus (Submitted, Running, Completed, Failed). Add to Types.idr or appropriate domain modules.

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`

---

<a id="ironstar-2it-24-generate-eventcatalog-entity-documentation-for-value-objects"></a>

## 📋 ironstar-2it.24 Generate EventCatalog entity documentation for value objects

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-15 12:38 |
| **Updated** | 2026-01-15 14:25 |
| **Closed** | 2026-01-15 14:25 |

### Description

Create entity MDX files for 19 value objects identified in cross-layer audit: SessionId, CatalogId, DashboardId, ChartId, TabId, QueryId, SavedQueryId, Timestamp, Duration, SqlStatement, DatasetRef, QueryResult, ErrorInfo, ChartConfig, Position, UiState, Theme, CatalogMetadata. Extract definitions from Idris specs; add invariants and usage context. Reference: docs/notes/architecture/cross-layer-consistency-report.md D6.

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`

---

<a id="ironstar-2it-22-formalize-workspace-bounded-context-in-idris2-supporting-domain"></a>

## 📋 ironstar-2it.22 Formalize Workspace bounded context in Idris2 (Supporting domain)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 15:47 |
| **Updated** | 2026-01-06 19:48 |
| **Closed** | 2026-01-06 19:48 |

### Description

Define the Workspace bounded context as Idris2 module. This is a Supporting domain for user's persistent saved state.

## Business context

Workspace manages user's persistent configuration that survives across session boundaries:
- Dashboards (layouts, tabs, chart placements)
- Saved queries
- User preferences

## Relationship to other contexts

- **Consumes Analytics**: References ChartDefinitions (Customer-Supplier relationship)
- **Requires Session**: User identity from authenticated session (Shared Kernel)
- **Persists across sessions**: Unlike Session state, Workspace state is durable

## Module: spec/src/Domain/Workspace.idr

```idris
-- Dashboard aggregate
data DashboardCommand
  = CreateDashboard DashboardName
  | AddChartToDashboard DashboardId ChartPlacement
  | RemoveChart DashboardId ChartId
  | UpdateLayout DashboardId Layout
  | AddTab DashboardId TabName
  | MoveChartToTab DashboardId ChartId TabId

data DashboardEvent
  = DashboardCreated DashboardId DashboardName UserId Timestamp
  | ChartAdded DashboardId ChartId ChartPlacement Timestamp
  | ChartRemoved DashboardId ChartId Timestamp
  | LayoutUpdated DashboardId Layout Timestamp
  | TabAdded DashboardId TabId TabName Timestamp
  | ChartMovedToTab DashboardId ChartId TabId Timestamp

-- ChartPlacement references Analytics ChartDefinition
record ChartPlacement where
  constructor MkChartPlacement
  chartDefinitionRef : ChartDefinitionRef  -- from Analytics context
  position : GridPosition
  size : GridSize
  tabId : Maybe TabId

record Layout where
  constructor MkLayout
  columns : Nat
  rows : Nat
  placements : List ChartPlacement

-- SavedQuery aggregate  
data SavedQueryCommand
  = SaveQuery QueryName SqlQuery DatasetRef
  | DeleteSavedQuery SavedQueryId
  | RenameQuery SavedQueryId QueryName

data SavedQueryEvent
  = QuerySaved SavedQueryId QueryName SqlQuery DatasetRef UserId Timestamp
  | QueryDeleted SavedQueryId Timestamp
  | QueryRenamed SavedQueryId QueryName Timestamp
```

## Dependent type opportunities

1. **Layout validity**: Placements must not overlap within grid
2. **Tab references**: ChartMovedToTab requires tab exists in dashboard
3. **User ownership**: Commands require UserId matches dashboard owner

## Reference materials

### Architecture docs
- docs/notes/architecture/core/bounded-contexts.md (newly created)

### Domain modeling methodology
- Wlaschin: ~/projects/functional-programming-workspace/domain-modeling-made-functional
- Ghosh: ~/projects/functional-programming-workspace/functional-and-reactive-domain-modeling

### Idris2 reference
- ~/projects/functional-programming-workspace/Idris2/docs/

## Deliverables

- [ ] spec/src/Domain/Workspace.idr (or Workspace/ directory if multiple modules)
- [ ] Module type-checks
- [ ] Cross-reference with bounded-contexts.md

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.17`

---

<a id="ironstar-2it-19-formalize-session-bounded-context-in-idris2-supporting-domain"></a>

## 📋 ironstar-2it.19 Formalize Session bounded context in Idris2 (Supporting domain)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 15:01 |
| **Updated** | 2026-01-06 19:48 |
| **Closed** | 2026-01-06 19:48 |

### Description

Define the Session bounded context as Idris2 module. This is a Supporting domain (authentication infrastructure).

## Business context

Session management for authenticated access. OAuth-based authentication with session lifecycle.

## Module: spec/src/Domain/Session.idr

```idris
-- Commands
data SessionCommand
  = CreateSession UserId OAuthProvider
  | RefreshSession SessionId
  | InvalidateSession SessionId

-- Events
data SessionEvent
  = SessionCreated SessionId UserId OAuthProvider Timestamp ExpiresAt
  | SessionRefreshed SessionId NewExpiresAt Timestamp
  | SessionInvalidated SessionId Timestamp
  | SessionExpired SessionId Timestamp

-- State
data SessionState
  = NoSession
  | Active SessionId UserId ExpiresAt
  | Expired SessionId

-- Value objects
record SessionId where
  constructor MkSessionId
  value : UUID

data OAuthProvider = GitHub | Google

record UserId where
  constructor MkUserId
  provider : OAuthProvider
  externalId : String
```

## Dependent type opportunities

1. **Session expiration**: Active state requires ExpiresAt > now
2. **State transitions**: Cannot refresh an Expired session
3. **Provider consistency**: UserId.provider matches session's OAuthProvider

## Reference materials

### D2 diagram
- docs/notes/event-modeling/d2/session-context-timeline.d2

### Architecture docs
- docs/notes/architecture/infrastructure/session-management.md
- docs/notes/architecture/infrastructure/session-security.md
- docs/notes/architecture/decisions/oauth-authentication.md

### Domain modeling methodology
- Wlaschin: ~/projects/functional-programming-workspace/domain-modeling-made-functional
- Ghosh: ~/projects/functional-programming-workspace/functional-and-reactive-domain-modeling

### Idris2 reference
- ~/projects/functional-programming-workspace/Idris2/docs/

## Deliverables

- [ ] spec/src/Domain/Session.idr
- [ ] Module type-checks
- [ ] Cross-reference with D2 diagram

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.17`

---

<a id="ironstar-2it-12-document-fmodel-rust-decider-mapping-in-eventcatalog"></a>

## 📋 ironstar-2it.12 Document fmodel-rust Decider mapping in EventCatalog

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 13:19 |
| **Updated** | 2026-01-15 14:19 |
| **Closed** | 2026-01-15 14:19 |

### Description

Enhance EventCatalog documentation to explicitly show mapping between Event Modeling artifacts and fmodel-rust Decider pattern.

## Prerequisites

- Cross-referenced EventCatalog complete (ironstar-2it.11)

## Decider pattern mapping

For each aggregate entity, document:

### decide function
```rust
fn decide(command: Command, state: &State) -> Result<Vec<Event>, Error>
```

Map EventCatalog commands to decide inputs, events to decide outputs.

### evolve function
```rust
fn evolve(state: State, event: Event) -> State
```

Document state transitions per event type.

### initial_state
```rust
fn initial_state() -> State
```

Document initial aggregate state.

## Entity MDX enhancements

Add to each entity:
```markdown
## fmodel-rust implementation

This aggregate implements the Decider pattern from fmodel-rust:

- **decide**: Validates commands and produces events
- **evolve**: Applies events to update state
- **initial_state**: Returns empty/default state

See `crates/ironstar/src/domain/{aggregate}/decider.rs`
```

## References section

Link each entity to:
- fmodel-rust source: ~/projects/rust-workspace/fmodel-rust
- fmodel-rust-demo patterns: ~/projects/rust-workspace/fmodel-rust-demo
- ironstar evaluation: docs/notes/architecture/decisions/fmodel-rust-adoption-evaluation.md

## Output

- Enhanced entity MDX with Decider documentation
- Clear traceability from EventCatalog to implementation

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.11`

---

<a id="ironstar-e8d-refactor-domain-module-into-aggregate-based-subdirectories"></a>

## 🧹 ironstar-e8d Refactor domain module into aggregate-based subdirectories

| Property | Value |
|----------|-------|
| **Type** | 🧹 chore |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-29 11:08 |
| **Updated** | 2025-12-29 11:08 |
| **Closed** | 2025-12-29 11:08 |

### Description

Discovered during ironstar-2nt.14 implementation: query_session.rs grew to 1224 lines.

Refactored domain/ into subdirectories for scalability:
- domain/todo/ - TodoAggregate (values, errors, commands, events, state, aggregate)
- domain/query_session/ - QuerySessionAggregate (errors, commands, events, state, aggregate)
- domain/analytics/ - Analytics value objects (errors, values)

Pattern: Each aggregate gets a subdirectory with consistent file structure.
This enables clean separation as more aggregates are added.

Implemented in e881db6 (23 refactoring commits).

### Dependencies

- 🔗 **discovered-from**: `ironstar-2nt.14`

---

<a id="ironstar-nyp-33-implement-session-cleanup-background-task"></a>

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

<a id="ironstar-3gd-2-implement-event-driven-cache-invalidation"></a>

## 📋 ironstar-3gd.2 Implement event-driven cache invalidation

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2026-02-02 15:56 |
| **Closed** | 2026-02-02 15:56 |

### Description

Subscribe to Zenoh key expressions for aggregate-type events. Invalidate moka cache entries matching the aggregate type when events arrive. Pattern: ironstar/events/{aggregate_type}/**. See docs/notes/architecture/infrastructure/analytics-cache-architecture.md Pattern 4.

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- 🔗 **depends-on**: `ironstar-3gd.3`

### Comments

> **Cameron Smith** (2026-02-02)
>
> Context from 3gd.3 implementation: matches_key_expression() is a standalone pub fn in cache_dependency.rs (not a method on CacheDependency). It takes (pattern: &str, key: &str) and implements Zenoh-compatible glob matching with ** (multi-level) and * (single-level) wildcards. CacheDependency::matches() delegates to it for each pattern in depends_on. 19 unit tests validate pattern matching including edge cases. The invalidation subscriber should call cache_dep.matches(incoming_key_expr) to determine which cache entries to evict.

---

<a id="ironstar-3gd-1-implement-cache-aside-pattern-for-duckdb-analytics"></a>

## 📋 ironstar-3gd.1 Implement cache-aside pattern for DuckDB analytics

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 15:37 |
| **Updated** | 2026-02-02 15:54 |
| **Closed** | 2026-02-02 15:54 |

### Description

moka get_or_compute wrapper with query hash key, TTL and idle eviction.

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- ⛔ **blocks**: `ironstar-nyp.15`

### Comments

> **Cameron Smith** (2026-02-02)
>
> Context from nyp.15 implementation: AnalyticsCache in analytics_cache.rs uses rkyv 0.8 high-level API (to_bytes/from_bytes with rancor::Error). Cache key is query_hash:u64 computed via DefaultHasher. Cache type is moka::future::Cache<u64, Vec<u8>> where values are rkyv-serialized ArchivedAnalyticsResult. The cache-aside wrapper should use AnalyticsCache::get() and AnalyticsCache::insert() which handle rkyv ser/de internally. 14 unit tests validate the cache contract.

---

<a id="ironstar-6lq-9-add-workspace-lint-configuration-to-cargo-toml"></a>

## 📋 ironstar-6lq.9 Add workspace lint configuration to Cargo.toml

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 10:02 |
| **Updated** | 2025-12-27 18:19 |
| **Closed** | 2025-12-27 18:19 |

### Description

Add [workspace.lints.rust] and [workspace.lints.clippy] sections following Hyperswitch pattern. See docs/notes/architecture/core/crate-services-composition.md section 'Workspace lint configuration'.

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`

---

<a id="ironstar-ny3-15-configure-rolldown-for-lit-web-component-bundling"></a>

## 📋 ironstar-ny3.15 Configure Rolldown for Lit web component bundling

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 00:44 |
| **Updated** | 2026-02-03 00:05 |
| **Closed** | 2026-02-03 00:05 |

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

---

<a id="ironstar-89k-integrate-analytics-cache-with-dashboard-sse-streams"></a>

## 📋 ironstar-89k Integrate analytics cache with dashboard SSE streams

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-24 00:43 |
| **Updated** | 2026-02-02 16:01 |
| **Closed** | 2026-02-02 16:01 |

### Description

Create separate analytics_bus broadcast channel distinct from main event bus. Wire cache refresh to SSE patch updates for dashboards. Implement multi-dashboard concurrent cache access. Reference: analytics-cache-patterns.md SSE/Datastar integration section.

### Dependencies

- 🔗 **parent-child**: `ironstar-3gd`
- ⛔ **blocks**: `ironstar-nyp.12`
- ⛔ **blocks**: `ironstar-3gd.1`

---

<a id="ironstar-9oj-implement-cache-invalidation-for-analytics-queries"></a>

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

<a id="ironstar-nqq-1-implement-cqrs-performance-tuning"></a>

## 📋 ironstar-nqq.1 Implement CQRS performance tuning

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2025-12-26 20:51 |

### Description

Channel sizing, backpressure handling, metrics instrumentation. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/cqrs/performance-tuning.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nqq`

---

<a id="ironstar-nqq-performance-optimization"></a>

## 🚀 ironstar-nqq Performance optimization

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-22 00:25 |
| **Updated** | 2025-12-26 20:51 |

### Description

Optional performance patterns for CQRS pipeline including channel sizing, backpressure, debouncing, batching, and rate limiting. See ~/projects/rust-workspace/ironstar/docs/notes/architecture/cqrs/performance-tuning.md and performance-advanced-patterns.md

---

<a id="ironstar-avp-verify-code-examples-compile-and-run"></a>

## 📋 ironstar-avp Verify code examples compile and run

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-26 20:51 |

### Description

Test all SQL and Rust code snippets. Ensure hf:// queries work with real datasets. Check that example commands are accurate.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`

---

<a id="ironstar-ym1-polish-diagrams-for-visual-consistency"></a>

## 📋 ironstar-ym1 Polish diagrams for visual consistency

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-26 20:51 |

### Description

Ensure all fletcher diagrams use consistent: node sizing, spacing, colors, edge styles. Consider adding subtle animations or build-up for complex diagrams.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`

---

<a id="ironstar-63r-verify-technical-accuracy-of-benchmarks"></a>

## 📋 ironstar-63r Verify technical accuracy of benchmarks

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-26 20:51 |

### Description

Cross-check all performance claims against source papers. Verify: AnnSQL 700x speedup context, 4.4M cell benchmark details, tiledbsoma AWS region claims, DuckLake release date.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`

---

<a id="ironstar-z4s-act-4-expand-vision-slides"></a>

## 📋 ironstar-z4s Act 4: Expand vision slides

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-26 20:51 |

### Description

Flesh out slides 20-24. Add speaker notes. Consider: more compelling architecture diagram, concrete demo scenario, stronger call-to-action.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`
- ⛔ **blocks**: `ironstar-edx`

---

<a id="ironstar-b8d-act-3-expand-web-interface-slides"></a>

## 📋 ironstar-b8d Act 3: Expand web interface slides

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-26 20:51 |

### Description

Flesh out slides 15-19. Add speaker notes. Consider: CellXGene screenshot for comparison, Datastar event flow animation concept, ironstar code snippet refinement.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`
- ⛔ **blocks**: `ironstar-edx`

---

<a id="ironstar-a15-act-2-expand-solution-stack-slides"></a>

## 📋 ironstar-a15 Act 2: Expand solution stack slides

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-26 20:51 |

### Description

Flesh out slides 9-14. Add speaker notes. Consider: DuckLake metadata schema visualization, httpfs query flow diagram, concrete hf:// query examples with real datasets.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`
- ⛔ **blocks**: `ironstar-edx`

---

<a id="ironstar-ubj-act-1-expand-data-problem-slides"></a>

## 📋 ironstar-ubj Act 1: Expand data problem slides

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-26 20:51 |

### Description

Flesh out slides 2-8. Add speaker notes. Consider: more concrete examples of AnnData failures, visual showing exponential runtime growth, clearer AnnSQL benchmark presentation.

### Dependencies

- 🔗 **parent-child**: `ironstar-0tk`
- ⛔ **blocks**: `ironstar-edx`

---

<a id="ironstar-r5f-ironstar-6lq"></a>

## 📋 ironstar-r5f ironstar-6lq

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-18 18:03 |
| **Updated** | 2025-12-26 20:51 |

---

<a id="ironstar-6lq-8-create-reusable-rust-ci-workflow-with-workflow-call-dispatch"></a>

## 📋 ironstar-6lq.8 Create reusable Rust CI workflow with workflow_call dispatch

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 18:02 |
| **Updated** | 2025-12-27 18:19 |
| **Closed** | 2025-12-27 18:19 |

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

---

<a id="ironstar-r62-12-implement-graceful-shutdown-signal-handling"></a>

## 📋 ironstar-r62.12 Implement graceful shutdown signal handling

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-28 11:42 |
| **Closed** | 2026-01-28 11:42 |

### Description

Add tokio signal handling for SIGTERM/SIGINT:

```rust
async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };
    
    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C"),
        _ = terminate => info!("Received SIGTERM"),
    }
    
    info!("Initiating graceful shutdown...");
    // Zenoh session closes automatically via Drop
    // SQLite pool closes automatically via Drop
}
```

Clean shutdown of EventRepository connections and Zenoh session.

Local refs: ~/projects/rust-workspace/tokio, ~/projects/rust-workspace/zenoh


### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.11`

---

<a id="ironstar-r62-11-implement-router-composition-with-feature-routes"></a>

## 📋 ironstar-r62.11 Implement router composition with feature routes

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-22 02:08 |
| **Closed** | 2026-01-22 02:08 |

### Description

Create main Router that merges feature modules. Each feature provides route() -> Router<AppState> composing GET/POST/SSE handlers. Use Router::merge to combine features and apply State layer to inject AppState.
Local refs: ~/projects/rust-workspace/axum

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-r62.5`
- ⛔ **blocks**: `ironstar-r62.6`
- ⛔ **blocks**: `ironstar-r62.7`

---

<a id="ironstar-r62-6-implement-command-post-handlers"></a>

## 📋 ironstar-r62.6 Implement command POST handlers

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-20 17:21 |
| **Closed** | 2026-01-20 17:21 |

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

### Comments

> **Cameron Smith** (2026-01-20)
>
> Verified: create_todo, complete_todo, delete_todo handlers in presentation/todo.rs with 202 Accepted responses.

---

<a id="ironstar-r62-5-implement-sse-feed-endpoint-with-event-replay"></a>

## 📋 ironstar-r62.5 Implement SSE feed endpoint with event replay

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-22 01:43 |
| **Closed** | 2026-01-22 01:43 |

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

---

<a id="ironstar-r62-4-define-appstate-struct-with-all-dependencies"></a>

## 📋 ironstar-r62.4 Define AppState struct with all dependencies

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-20 17:20 |
| **Closed** | 2026-01-20 17:20 |

### Description

Create AppState struct holding all application dependencies:

```rust
pub struct AppState {
    pub event_repository: Arc<SqliteEventRepository>,
    pub view_repository: Arc<SqliteViewRepository>,
    pub session_store: Arc<SessionStore>,
    pub zenoh_session: Arc<zenoh::Session>,
    pub todo_aggregate: Arc<EventSourcedAggregate<TodoDecider, SqliteEventRepository>>,
    pub todo_view: Arc<MaterializedView<TodoView, SqliteViewRepository>>,
}
```

Initialize services and rebuild projections at startup:

```rust
impl AppState {
    pub async fn new(config: &Config) -> Result<Self, AppError> {
        let pool = SqlitePool::connect(&config.database_url).await?;
        let event_repository = Arc::new(SqliteEventRepository::new(pool.clone()));
        let view_repository = Arc::new(SqliteViewRepository::new(pool.clone()));
        
        // Wire EventSourcedAggregate with Decider + Repository
        let todo_aggregate = Arc::new(EventSourcedAggregate::new(
            event_repository.clone(),
            todo_decider(),
        ));
        
        // Wire MaterializedView
        let todo_view = Arc::new(MaterializedView::new(
            view_repository.clone(),
            todo_view(),
        ));
        
        Ok(Self { ... })
    }
}
```

Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/fmodel-rust


### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-nyp.3`
- ⛔ **blocks**: `ironstar-nyp.10`
- ⛔ **blocks**: `ironstar-nyp.7`
- ⛔ **blocks**: `ironstar-nyp.5`
- ⛔ **blocks**: `ironstar-nyp.27`
- ⛔ **blocks**: `ironstar-a9b.7`
- ⛔ **blocks**: `ironstar-a9b.8`

### Comments

> **Cameron Smith** (2026-01-20)
>
> Checkpoint 2026-01-20: Implementation complete, pending commit.
> 
> Done:
> - Created crates/ironstar/src/state.rs with AppState struct
> - Required fields: db_pool (SqlitePool), assets (AssetManifest)
> - Optional fields: event_bus, session_store, analytics (all Option<Arc<...>>)
> - Builder pattern methods for optional capabilities
> - FromRef implementations for TodoAppState, HealthState, AssetManifest, SqlitePool
> 
> Blocked: Git commits failing due to SSH signing key passphrase issue.

> **Cameron Smith** (2026-01-20)
>
> Removed nyp.5 blocker: Zenoh (nyp.25-29) replaces tokio broadcast per architecture decision. nyp.27 (ZenohEventBus) is complete.

> **Cameron Smith** (2026-01-20)
>
> Verified: AppState struct committed at 5a0c220 with builder pattern and FromRef implementations. Concrete SqliteSessionStore type appropriate for single-backend design.

---

<a id="ironstar-2it-20-formalize-todo-bounded-context-in-idris2-generic-example"></a>

## 📋 ironstar-2it.20 Formalize Todo bounded context in Idris2 (Generic Example)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ☕ Low (P3) |
| **Status** | ⚫ closed |
| **Created** | 2026-01-06 15:01 |
| **Updated** | 2026-01-06 19:48 |
| **Closed** | 2026-01-06 19:48 |

### Description

Define the Todo bounded context as Idris2 module. This is a Generic Example domain for template demonstration.

## Business context

Simple Todo list for demonstrating event sourcing patterns. Minimal complexity, clear state machine.

## Module: spec/src/Domain/Todo.idr

```idris
-- Commands
data TodoCommand
  = CreateTodo Text
  | CompleteTodo TodoId
  | UndoComplete TodoId
  | DeleteTodo TodoId

-- Events
data TodoEvent
  = TodoCreated TodoId Text Timestamp
  | TodoCompleted TodoId Timestamp
  | TodoUncompleted TodoId Timestamp
  | TodoDeleted TodoId Timestamp

-- State (per Todo item)
data TodoItemState
  = Active TodoId Text
  | Completed TodoId Text
  | Deleted TodoId

-- Aggregate state (list of items)
record TodoListState where
  constructor MkTodoListState
  items : List TodoItemState
  
-- Value objects
record TodoId where
  constructor MkTodoId
  value : UUID
```

## Dependent type opportunities

1. **State transitions**: Cannot complete a Deleted todo
2. **Idempotency**: Completing an already-completed todo is no-op
3. **Existence**: CompleteTodo requires TodoId exists in items

## Reference materials

### D2 diagram
- docs/notes/event-modeling/d2/todo-context-timeline.d2

### Domain modeling methodology
- Wlaschin: ~/projects/functional-programming-workspace/domain-modeling-made-functional
- Ghosh: ~/projects/functional-programming-workspace/functional-and-reactive-domain-modeling

### Idris2 reference
- ~/projects/functional-programming-workspace/Idris2/docs/

## Deliverables

- [ ] spec/src/Domain/Todo.idr
- [ ] Module type-checks
- [ ] Cross-reference with D2 diagram

### Dependencies

- 🔗 **parent-child**: `ironstar-2it`
- ⛔ **blocks**: `ironstar-2it.17`

---

<a id="ironstar-nyp-16-implement-dualeventbus-for-tokio-broadcast-to-zenoh-migration"></a>

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

<a id="ironstar-nqq-2-implement-advanced-performance-patterns"></a>

## 📋 ironstar-nqq.2 Implement advanced performance patterns

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ☕ Low (P3) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-22 00:28 |
| **Updated** | 2025-12-26 20:51 |

### Description

Debouncing, batching, rate limiting (optional optimizations). See ~/projects/rust-workspace/ironstar/docs/notes/architecture/cqrs/performance-advanced-patterns.md

### Dependencies

- 🔗 **parent-child**: `ironstar-nqq`
- ⛔ **blocks**: `ironstar-nqq.1`

---

<a id="ironstar-k1z-final-review-and-presentation-dry-run"></a>

## 📋 ironstar-k1z Final review and presentation dry-run

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ☕ Low (P3) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-19 01:16 |
| **Updated** | 2025-12-26 20:51 |

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

---

<a id="ironstar-e6k-example-application-todo"></a>

## 🚀 ironstar-e6k Example application (Todo)

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | ☕ Low (P3) |
| **Status** | ⚫ closed |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2026-01-19 19:08 |
| **Closed** | 2026-01-19 19:08 |

### Description

Pedagogical TodoMVC demonstration integrating all architectural layers.

## Role in ironstar

This epic provides **pedagogical scaffolding**, not the primary use case:

- **Pedagogical**: Familiar reference application (TodoMVC) for learning patterns
- **Pattern source**: Todo implementation in 2nt.2/3/4/7 established patterns for QuerySession
- **Full stack demo**: When complete, demonstrates SSE, hypertext, CQRS end-to-end

Per architecture-decisions.md: "The domain model centers on QuerySession as the primary aggregate, not a generic Todo placeholder."

## Current status

- ✓ Domain model complete (superseded e6k.1 - done in domain layer epic)
  - TodoAggregate, TodoEvent, TodoCommand, TodoState in crates/ironstar/src/domain/
- → Remaining work: projection, handlers, SSE feed, templates, routing

## Relationship to QuerySession

The patterns established here inform QuerySession implementation:
- Value objects with smart constructors → DatasetRef, SqlQuery, QueryId
- Pure aggregate state machine → QuerySession lifecycle
- Railway-oriented Result validation → Analytics error handling

QuerySession (ironstar-2nt.14) is the "true use case"; Todo is the learning vehicle.

Local refs: ~/projects/lakescope-workspace/datastar-go-nats-template-northstar (northstar patterns)

### Dependencies

- ⛔ **blocks**: `ironstar-r62`

---

<a id="ironstar-v4y-3-define-common-utils-crate-structure"></a>

## 📋 ironstar-v4y.3 Define common-utils crate structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 💤 Backlog (P4) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-24 00:54 |
| **Updated** | 2025-12-26 20:51 |

### Description

Layer 0 foundation crate. Contains: crypto helpers, validation utilities, serialization helpers, extension traits. See crate-architecture.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-v4y`

---

<a id="ironstar-v4y-2-define-common-types-crate-structure"></a>

## 📋 ironstar-v4y.2 Define common-types crate structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 💤 Backlog (P4) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-24 00:54 |
| **Updated** | 2025-12-26 20:51 |

### Description

Layer 0 foundation crate. Contains: MinorUnit, Timestamp, Sequence newtypes. TodoId, TodoText smart constructors with validation. See crate-architecture.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-v4y`

---

<a id="ironstar-v4y-1-define-common-enums-crate-structure"></a>

## 📋 ironstar-v4y.1 Define common-enums crate structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 💤 Backlog (P4) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-24 00:54 |
| **Updated** | 2025-12-26 20:51 |

### Description

Layer 0 foundation crate. Contains: AggregateType, EventType, ErrorCode, FilterType enums. No ironstar dependencies. See crate-architecture.md.

### Dependencies

- 🔗 **parent-child**: `ironstar-v4y`

---

<a id="ironstar-v4y-multi-crate-workspace-decomposition"></a>

## 🚀 ironstar-v4y Multi-crate workspace decomposition

| Property | Value |
|----------|-------|
| **Type** | 🚀 epic |
| **Priority** | 💤 Backlog (P4) |
| **Status** | ⚫ tombstone |
| **Created** | 2025-12-24 00:44 |
| **Updated** | 2025-12-26 20:51 |

### Description

Implement 8-layer crate decomposition from crate-architecture.md. Includes common-enums/types/utils (Layer 0), ironstar-domain/commands/events (Layer 1), ironstar-app (Layer 2), ironstar-interfaces (Layer 3), ironstar-adapters/analytics/projections/config (Layer 4), ironstar-services (Layer 5), ironstar-web (Layer 6), ironstar binary (Layer 7). Deferred until single-crate grows beyond 800 lines or coupling becomes problematic.

---

