# Beads Export

*Generated: Thu, 18 Dec 2025 09:47:17 EST*

## Summary

| Metric | Count |
|--------|-------|
| **Total** | 96 |
| Open | 96 |
| In Progress | 0 |
| Blocked | 0 |
| Closed | 0 |

## Quick Actions

Ready-to-run commands for bulk operations:

```bash
# Close open items (96 total, showing first 10)
bd close ironstar-r62.12 ironstar-r62.11 ironstar-r62.9 ironstar-r62.8 ironstar-r62.7 ironstar-r62.6 ironstar-r62.5 ironstar-r62.4 ironstar-r62.2 ironstar-r62.1

# View high-priority items (P0/P1)
bd show ironstar-r62.12 ironstar-r62.11 ironstar-r62.9 ironstar-r62.8 ironstar-r62.7 ironstar-r62.6 ironstar-r62.5 ironstar-r62.4 ironstar-r62.2 ironstar-r62.1 ironstar-nyp.8 ironstar-nyp.7 ironstar-nyp.6 ironstar-nyp.5 ironstar-nyp.3 ironstar-nyp.2 ironstar-nyp.1 ironstar-nyp ironstar-ny3.13 ironstar-ny3.12 ironstar-ny3.11 ironstar-ny3.10 ironstar-ny3.9 ironstar-ny3.8 ironstar-ny3.7 ironstar-ny3.5 ironstar-ny3.4 ironstar-ny3.3 ironstar-ny3.2 ironstar-ny3.1 ironstar-2nt.7 ironstar-2nt.6 ironstar-2nt.5 ironstar-2nt.4 ironstar-2nt.3 ironstar-2nt.2 ironstar-2nt.1 ironstar-2nt ironstar-f8b.5 ironstar-f8b.4 ironstar-f8b.3 ironstar-f8b.2 ironstar-f8b.1 ironstar-f8b ironstar-6lq.7 ironstar-6lq.6 ironstar-6lq.5 ironstar-6lq.4 ironstar-6lq.3 ironstar-6lq.2 ironstar-6lq.1 ironstar-6lq ironstar-cxe.5 ironstar-cxe.4 ironstar-cxe.3 ironstar-cxe.2 ironstar-cxe.1 ironstar-cxe ironstar-e6k.8 ironstar-e6k.7 ironstar-e6k.6 ironstar-e6k.5 ironstar-e6k.4 ironstar-e6k.3 ironstar-e6k.2 ironstar-e6k.1 ironstar-r62.13 ironstar-r62.10 ironstar-r62.3 ironstar-r62 ironstar-nyp.11 ironstar-nyp.10 ironstar-nyp.9 ironstar-nyp.4 ironstar-ny3.14 ironstar-ny3.6 ironstar-ny3 ironstar-2nt.8

```

## Table of Contents

- [🟢 ironstar-r62.12 Implement graceful shutdown signal handling](#ironstar-r62-12)
- [🟢 ironstar-r62.11 Implement router composition with feature routes](#ironstar-r62-11)
- [🟢 ironstar-r62.9 Create base layout template with Datastar initialization](#ironstar-r62-9)
- [🟢 ironstar-r62.8 Implement RenderableToDatastar conversion trait](#ironstar-r62-8)
- [🟢 ironstar-r62.7 Implement query GET handlers](#ironstar-r62-7)
- [🟢 ironstar-r62.6 Implement command POST handlers](#ironstar-r62-6)
- [🟢 ironstar-r62.5 Implement SSE feed endpoint with event replay](#ironstar-r62-5)
- [🟢 ironstar-r62.4 Define AppState struct with all dependencies](#ironstar-r62-4)
- [🟢 ironstar-r62.2 Create devShell module with tools and environment](#ironstar-r62-2)
- [🟢 ironstar-r62.1 Add justfile with development and build tasks](#ironstar-r62-1)
- [🟢 ironstar-nyp.8 Implement SSE 15-second keep-alive comment stream](#ironstar-nyp-8)
- [🟢 ironstar-nyp.7 Implement ProjectionManager with in-memory state](#ironstar-nyp-7)
- [🟢 ironstar-nyp.6 Create Projection trait for read models](#ironstar-nyp-6)
- [🟢 ironstar-nyp.5 Implement tokio broadcast event bus](#ironstar-nyp-5)
- [🟢 ironstar-nyp.3 Implement SQLite event store with sqlx](#ironstar-nyp-3)
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
- [🟢 ironstar-ny3.1 Create web-components/ project structure with package.json](#ironstar-ny3-1)
- [🟢 ironstar-2nt.7 Implement command validation pattern with Result types](#ironstar-2nt-7)
- [🟢 ironstar-2nt.6 Enforce camelCase convention for Datastar signal fields](#ironstar-2nt-6)
- [🟢 ironstar-2nt.5 Create Datastar signal types with ts-rs derives](#ironstar-2nt-5)
- [🟢 ironstar-2nt.4 Design aggregate root state machines](#ironstar-2nt-4)
- [🟢 ironstar-2nt.3 Implement value objects and smart constructors](#ironstar-2nt-3)
- [🟢 ironstar-2nt.2 Define algebraic domain types and aggregate structure](#ironstar-2nt-2)
- [🟢 ironstar-2nt.1 Initialize src/ directory structure with modular organization](#ironstar-2nt-1)
- [🟢 ironstar-2nt Domain layer](#ironstar-2nt)
- [🟢 ironstar-f8b.5 Verify process-compose up works with all services](#ironstar-f8b-5)
- [🟢 ironstar-f8b.4 Configure cargo-watch to curl hotreload trigger on success](#ironstar-f8b-4)
- [🟢 ironstar-f8b.3 Set up service orchestration (frontend bundler, cargo-watch)](#ironstar-f8b-3)
- [🟢 ironstar-f8b.2 Configure process-compose.yaml for dev services](#ironstar-f8b-2)
- [🟢 ironstar-f8b.1 Integrate process-compose-flake patterns into devShell](#ironstar-f8b-1)
- [🟢 ironstar-f8b Process compose integration](#ironstar-f8b)
- [🟢 ironstar-6lq.7 Add Rust to CI matrix and extend inherited workflows](#ironstar-6lq-7)
- [🟢 ironstar-6lq.6 Add Rust checks to flake.checks for CI integration](#ironstar-6lq-6)
- [🟢 ironstar-6lq.5 Verify cargo check passes with workspace configuration](#ironstar-6lq-5)
- [🟢 ironstar-6lq.4 Set up per-crate crate.nix pattern for crane args](#ironstar-6lq-4)
- [🟢 ironstar-6lq.3 Configure Cargo.toml with workspace structure (resolver = 2)](#ironstar-6lq-3)
- [🟢 ironstar-6lq.2 Add rust-toolchain.toml with required components](#ironstar-6lq-2)
- [🟢 ironstar-6lq.1 Integrate rust-flake patterns (crane, rust-overlay)](#ironstar-6lq-1)
- [🟢 ironstar-6lq Rust workspace integration](#ironstar-6lq)
- [🟢 ironstar-cxe.5 Create .gitignore with comprehensive patterns](#ironstar-cxe-5)
- [🟢 ironstar-cxe.4 Create initial git commit with generated structure](#ironstar-cxe-4)
- [🟢 ironstar-cxe.3 Verify nix develop enters working development shell](#ironstar-cxe-3)
- [🟢 ironstar-cxe.2 Configure secrets management and string replacement](#ironstar-cxe-2)
- [🟢 ironstar-cxe.1 Run om init with typescript-nix-template parameters](#ironstar-cxe-1)
- [🟢 ironstar-cxe Template instantiation](#ironstar-cxe)
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
- [🟢 ironstar-r62 Presentation layer](#ironstar-r62)
- [🟢 ironstar-nyp.11 Create Session axum extractor](#ironstar-nyp-11)
- [🟢 ironstar-nyp.10 Add session TTL cleanup background task](#ironstar-nyp-10)
- [🟢 ironstar-nyp.9 Implement redb session store with ACID guarantees](#ironstar-nyp-9)
- [🟢 ironstar-nyp.4 Implement SQLite connection pooling and configuration](#ironstar-nyp-4)
- [🟢 ironstar-ny3.14 Create web-components/components/ directory for vanilla web components](#ironstar-ny3-14)
- [🟢 ironstar-ny3.6 Copy Open Props UI component CSS files](#ironstar-ny3-6)
- [🟢 ironstar-ny3 Frontend build pipeline](#ironstar-ny3)
- [🟢 ironstar-2nt.8 Define application error types](#ironstar-2nt-8)
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
- [🟢 ironstar-nyp.12 Implement DuckDB analytics service](#ironstar-nyp-12)
- [🟢 ironstar-apx.3 Define om CLI instantiation tests and metadata](#ironstar-apx-3)
- [🟢 ironstar-apx Documentation and template](#ironstar-apx)

---

## Dependency Graph

```mermaid
graph TD
    classDef open fill:#50FA7B,stroke:#333,color:#000
    classDef inprogress fill:#8BE9FD,stroke:#333,color:#000
    classDef blocked fill:#FF5555,stroke:#333,color:#000
    classDef closed fill:#6272A4,stroke:#333,color:#fff

    ironstar-r6212["ironstar-r62.12<br/>Implement graceful shutdown signal ha..."]
    class ironstar-r6212 open
    ironstar-r6212 -.-> ironstar-r62
    ironstar-r6212 ==> ironstar-r6211
    ironstar-r6211["ironstar-r62.11<br/>Implement router composition with fea..."]
    class ironstar-r6211 open
    ironstar-r6211 -.-> ironstar-r62
    ironstar-r6211 ==> ironstar-r625
    ironstar-r6211 ==> ironstar-r626
    ironstar-r6211 ==> ironstar-r627
    ironstar-r629["ironstar-r62.9<br/>Create base layout template with Data..."]
    class ironstar-r629 open
    ironstar-r629 -.-> ironstar-r62
    ironstar-r629 ==> ironstar-r628
    ironstar-r629 ==> ironstar-ny313
    ironstar-r628["ironstar-r62.8<br/>Implement RenderableToDatastar conver..."]
    class ironstar-r628 open
    ironstar-r628 -.-> ironstar-r62
    ironstar-r627["ironstar-r62.7<br/>Implement query GET handlers"]
    class ironstar-r627 open
    ironstar-r627 -.-> ironstar-r62
    ironstar-r627 ==> ironstar-r624
    ironstar-r627 ==> ironstar-nyp7
    ironstar-r626["ironstar-r62.6<br/>Implement command POST handlers"]
    class ironstar-r626 open
    ironstar-r626 -.-> ironstar-r62
    ironstar-r626 ==> ironstar-r624
    ironstar-r626 ==> ironstar-2nt4
    ironstar-r625["ironstar-r62.5<br/>Implement SSE feed endpoint with even..."]
    class ironstar-r625 open
    ironstar-r625 -.-> ironstar-r62
    ironstar-r625 ==> ironstar-r624
    ironstar-r625 ==> ironstar-nyp8
    ironstar-r624["ironstar-r62.4<br/>Define AppState struct with all depen..."]
    class ironstar-r624 open
    ironstar-r624 -.-> ironstar-r62
    ironstar-r624 ==> ironstar-nyp3
    ironstar-r624 ==> ironstar-nyp10
    ironstar-r624 ==> ironstar-nyp7
    ironstar-r624 ==> ironstar-nyp5
    ironstar-r622["ironstar-r62.2<br/>Create devShell module with tools and..."]
    class ironstar-r622 open
    ironstar-r622 -.-> ironstar-r62
    ironstar-r621["ironstar-r62.1<br/>Add justfile with development and bui..."]
    class ironstar-r621 open
    ironstar-r621 -.-> ironstar-r62
    ironstar-nyp8["ironstar-nyp.8<br/>Implement SSE 15-second keep-alive co..."]
    class ironstar-nyp8 open
    ironstar-nyp8 -.-> ironstar-nyp
    ironstar-nyp8 ==> ironstar-nyp5
    ironstar-nyp7["ironstar-nyp.7<br/>Implement ProjectionManager with in-m..."]
    class ironstar-nyp7 open
    ironstar-nyp7 -.-> ironstar-nyp
    ironstar-nyp7 ==> ironstar-nyp6
    ironstar-nyp7 ==> ironstar-nyp5
    ironstar-nyp7 ==> ironstar-nyp3
    ironstar-nyp6["ironstar-nyp.6<br/>Create Projection trait for read models"]
    class ironstar-nyp6 open
    ironstar-nyp6 -.-> ironstar-nyp
    ironstar-nyp6 ==> ironstar-2nt2
    ironstar-nyp5["ironstar-nyp.5<br/>Implement tokio broadcast event bus"]
    class ironstar-nyp5 open
    ironstar-nyp5 -.-> ironstar-nyp
    ironstar-nyp5 ==> ironstar-2nt2
    ironstar-nyp3["ironstar-nyp.3<br/>Implement SQLite event store with sqlx"]
    class ironstar-nyp3 open
    ironstar-nyp3 -.-> ironstar-nyp
    ironstar-nyp3 ==> ironstar-nyp2
    ironstar-nyp3 ==> ironstar-nyp1
    ironstar-nyp2["ironstar-nyp.2<br/>Create EventStore trait abstraction"]
    class ironstar-nyp2 open
    ironstar-nyp2 -.-> ironstar-nyp
    ironstar-nyp2 ==> ironstar-2nt2
    ironstar-nyp1["ironstar-nyp.1<br/>Create database migrations/ directory..."]
    class ironstar-nyp1 open
    ironstar-nyp1 -.-> ironstar-nyp
    ironstar-nyp["ironstar-nyp<br/>Event sourcing infrastructure"]
    class ironstar-nyp open
    ironstar-nyp ==> ironstar-2nt
    ironstar-ny313["ironstar-ny3.13<br/>Implement rust-embed conditional asse..."]
    class ironstar-ny313 open
    ironstar-ny313 -.-> ironstar-ny3
    ironstar-ny313 ==> ironstar-ny312
    ironstar-ny312["ironstar-ny3.12<br/>Implement manifest.json parser for ha..."]
    class ironstar-ny312 open
    ironstar-ny312 -.-> ironstar-ny3
    ironstar-ny312 ==> ironstar-ny311
    ironstar-ny311["ironstar-ny3.11<br/>Create static/dist/ output directory ..."]
    class ironstar-ny311 open
    ironstar-ny311 -.-> ironstar-ny3
    ironstar-ny311 ==> ironstar-ny32
    ironstar-ny310["ironstar-ny3.10<br/>Configure ts-rs export directory and ..."]
    class ironstar-ny310 open
    ironstar-ny310 -.-> ironstar-ny3
    ironstar-ny310 ==> ironstar-ny39
    ironstar-ny310 ==> ironstar-2nt5
    ironstar-ny39["ironstar-ny3.9<br/>Add ts-rs dependency to Cargo.toml"]
    class ironstar-ny39 open
    ironstar-ny39 -.-> ironstar-ny3
    ironstar-ny38["ironstar-ny3.8<br/>Create web-components/index.ts entry ..."]
    class ironstar-ny38 open
    ironstar-ny38 -.-> ironstar-ny3
    ironstar-ny38 ==> ironstar-ny37
    ironstar-ny37["ironstar-ny3.7<br/>Create TypeScript configuration (tsco..."]
    class ironstar-ny37 open
    ironstar-ny37 -.-> ironstar-ny3
    ironstar-ny37 ==> ironstar-ny31
    ironstar-ny35["ironstar-ny3.5<br/>Configure CSS cascade layers for pred..."]
    class ironstar-ny35 open
    ironstar-ny35 -.-> ironstar-ny3
    ironstar-ny35 ==> ironstar-ny34
    ironstar-ny34["ironstar-ny3.4<br/>Setup Open Props design tokens and th..."]
    class ironstar-ny34 open
    ironstar-ny34 -.-> ironstar-ny3
    ironstar-ny34 ==> ironstar-ny33
    ironstar-ny33["ironstar-ny3.3<br/>Setup PostCSS configuration for moder..."]
    class ironstar-ny33 open
    ironstar-ny33 -.-> ironstar-ny3
    ironstar-ny33 ==> ironstar-ny31
    ironstar-ny32["ironstar-ny3.2<br/>Configure Rolldown bundler with conte..."]
    class ironstar-ny32 open
    ironstar-ny32 -.-> ironstar-ny3
    ironstar-ny32 ==> ironstar-ny31
    ironstar-ny31["ironstar-ny3.1<br/>Create web-components/ project struct..."]
    class ironstar-ny31 open
    ironstar-ny31 -.-> ironstar-ny3
    ironstar-2nt7["ironstar-2nt.7<br/>Implement command validation pattern ..."]
    class ironstar-2nt7 open
    ironstar-2nt7 -.-> ironstar-2nt
    ironstar-2nt7 ==> ironstar-2nt4
    ironstar-2nt6["ironstar-2nt.6<br/>Enforce camelCase convention for Data..."]
    class ironstar-2nt6 open
    ironstar-2nt6 -.-> ironstar-2nt
    ironstar-2nt6 ==> ironstar-2nt5
    ironstar-2nt5["ironstar-2nt.5<br/>Create Datastar signal types with ts-..."]
    class ironstar-2nt5 open
    ironstar-2nt5 -.-> ironstar-2nt
    ironstar-2nt5 ==> ironstar-2nt2
    ironstar-2nt4["ironstar-2nt.4<br/>Design aggregate root state machines"]
    class ironstar-2nt4 open
    ironstar-2nt4 -.-> ironstar-2nt
    ironstar-2nt4 ==> ironstar-2nt3
    ironstar-2nt3["ironstar-2nt.3<br/>Implement value objects and smart con..."]
    class ironstar-2nt3 open
    ironstar-2nt3 -.-> ironstar-2nt
    ironstar-2nt3 ==> ironstar-2nt2
    ironstar-2nt2["ironstar-2nt.2<br/>Define algebraic domain types and agg..."]
    class ironstar-2nt2 open
    ironstar-2nt2 -.-> ironstar-2nt
    ironstar-2nt2 ==> ironstar-2nt1
    ironstar-2nt1["ironstar-2nt.1<br/>Initialize src/ directory structure w..."]
    class ironstar-2nt1 open
    ironstar-2nt1 -.-> ironstar-2nt
    ironstar-2nt["ironstar-2nt<br/>Domain layer"]
    class ironstar-2nt open
    ironstar-2nt ==> ironstar-6lq5
    ironstar-f8b5["ironstar-f8b.5<br/>Verify process-compose up works with ..."]
    class ironstar-f8b5 open
    ironstar-f8b5 -.-> ironstar-f8b
    ironstar-f8b5 ==> ironstar-f8b4
    ironstar-f8b4["ironstar-f8b.4<br/>Configure cargo-watch to curl hotrelo..."]
    class ironstar-f8b4 open
    ironstar-f8b4 -.-> ironstar-f8b
    ironstar-f8b4 ==> ironstar-f8b3
    ironstar-f8b3["ironstar-f8b.3<br/>Set up service orchestration (fronten..."]
    class ironstar-f8b3 open
    ironstar-f8b3 -.-> ironstar-f8b
    ironstar-f8b3 ==> ironstar-f8b2
    ironstar-f8b2["ironstar-f8b.2<br/>Configure process-compose.yaml for de..."]
    class ironstar-f8b2 open
    ironstar-f8b2 -.-> ironstar-f8b
    ironstar-f8b2 ==> ironstar-f8b1
    ironstar-f8b1["ironstar-f8b.1<br/>Integrate process-compose-flake patte..."]
    class ironstar-f8b1 open
    ironstar-f8b1 -.-> ironstar-f8b
    ironstar-f8b["ironstar-f8b<br/>Process compose integration"]
    class ironstar-f8b open
    ironstar-f8b ==> ironstar-6lq
    ironstar-6lq7["ironstar-6lq.7<br/>Add Rust to CI matrix and extend inhe..."]
    class ironstar-6lq7 open
    ironstar-6lq7 -.-> ironstar-6lq
    ironstar-6lq7 ==> ironstar-6lq6
    ironstar-6lq6["ironstar-6lq.6<br/>Add Rust checks to flake.checks for C..."]
    class ironstar-6lq6 open
    ironstar-6lq6 -.-> ironstar-6lq
    ironstar-6lq6 ==> ironstar-6lq4
    ironstar-6lq5["ironstar-6lq.5<br/>Verify cargo check passes with worksp..."]
    class ironstar-6lq5 open
    ironstar-6lq5 -.-> ironstar-6lq
    ironstar-6lq5 ==> ironstar-6lq4
    ironstar-6lq4["ironstar-6lq.4<br/>Set up per-crate crate.nix pattern fo..."]
    class ironstar-6lq4 open
    ironstar-6lq4 -.-> ironstar-6lq
    ironstar-6lq4 ==> ironstar-6lq3
    ironstar-6lq3["ironstar-6lq.3<br/>Configure Cargo.toml with workspace s..."]
    class ironstar-6lq3 open
    ironstar-6lq3 -.-> ironstar-6lq
    ironstar-6lq3 ==> ironstar-6lq2
    ironstar-6lq2["ironstar-6lq.2<br/>Add rust-toolchain.toml with required..."]
    class ironstar-6lq2 open
    ironstar-6lq2 -.-> ironstar-6lq
    ironstar-6lq2 ==> ironstar-6lq1
    ironstar-6lq1["ironstar-6lq.1<br/>Integrate rust-flake patterns (crane,..."]
    class ironstar-6lq1 open
    ironstar-6lq1 -.-> ironstar-6lq
    ironstar-6lq["ironstar-6lq<br/>Rust workspace integration"]
    class ironstar-6lq open
    ironstar-6lq ==> ironstar-cxe
    ironstar-cxe5["ironstar-cxe.5<br/>Create .gitignore with comprehensive ..."]
    class ironstar-cxe5 open
    ironstar-cxe5 -.-> ironstar-cxe
    ironstar-cxe4["ironstar-cxe.4<br/>Create initial git commit with genera..."]
    class ironstar-cxe4 open
    ironstar-cxe4 -.-> ironstar-cxe
    ironstar-cxe4 ==> ironstar-cxe3
    ironstar-cxe4 ==> ironstar-cxe2
    ironstar-cxe3["ironstar-cxe.3<br/>Verify nix develop enters working dev..."]
    class ironstar-cxe3 open
    ironstar-cxe3 -.-> ironstar-cxe
    ironstar-cxe3 ==> ironstar-cxe1
    ironstar-cxe2["ironstar-cxe.2<br/>Configure secrets management and stri..."]
    class ironstar-cxe2 open
    ironstar-cxe2 -.-> ironstar-cxe
    ironstar-cxe2 ==> ironstar-cxe1
    ironstar-cxe1["ironstar-cxe.1<br/>Run om init with typescript-nix-templ..."]
    class ironstar-cxe1 open
    ironstar-cxe1 -.-> ironstar-cxe
    ironstar-cxe["ironstar-cxe<br/>Template instantiation"]
    class ironstar-cxe open
    ironstar-e6k8["ironstar-e6k.8<br/>Implement todo example route mounting"]
    class ironstar-e6k8 open
    ironstar-e6k8 -.-> ironstar-e6k
    ironstar-e6k8 ==> ironstar-e6k6
    ironstar-e6k8 ==> ironstar-e6k3
    ironstar-e6k8 ==> ironstar-e6k4
    ironstar-e6k8 ==> ironstar-e6k5
    ironstar-e6k8 ==> ironstar-e6k7
    ironstar-e6k7["ironstar-e6k.7<br/>Implement todo_list_template renderin..."]
    class ironstar-e6k7 open
    ironstar-e6k7 -.-> ironstar-e6k
    ironstar-e6k7 ==> ironstar-r6210
    ironstar-e6k6["ironstar-e6k.6<br/>Implement GET /todos SSE feed endpoint"]
    class ironstar-e6k6 open
    ironstar-e6k6 -.-> ironstar-e6k
    ironstar-e6k6 ==> ironstar-e6k2
    ironstar-e6k6 ==> ironstar-r625
    ironstar-e6k5["ironstar-e6k.5<br/>Implement delete_todo handler (POST /..."]
    class ironstar-e6k5 open
    ironstar-e6k5 -.-> ironstar-e6k
    ironstar-e6k5 ==> ironstar-e6k3
    ironstar-e6k4["ironstar-e6k.4<br/>Implement mark_todo handler (POST /ma..."]
    class ironstar-e6k4 open
    ironstar-e6k4 -.-> ironstar-e6k
    ironstar-e6k4 ==> ironstar-e6k3
    ironstar-e6k3["ironstar-e6k.3<br/>Implement add_todo handler (POST /add..."]
    class ironstar-e6k3 open
    ironstar-e6k3 -.-> ironstar-e6k
    ironstar-e6k3 ==> ironstar-e6k2
    ironstar-e6k3 ==> ironstar-r626
    ironstar-e6k2["ironstar-e6k.2<br/>Implement TodoListProjection with in-..."]
    class ironstar-e6k2 open
    ironstar-e6k2 -.-> ironstar-e6k
    ironstar-e6k2 ==> ironstar-e6k1
    ironstar-e6k2 ==> ironstar-nyp7
    ironstar-e6k1["ironstar-e6k.1<br/>Define Todo domain model (aggregate, ..."]
    class ironstar-e6k1 open
    ironstar-e6k1 -.-> ironstar-e6k
    ironstar-e6k1 ==> ironstar-2nt2
    ironstar-e6k1 ==> ironstar-2nt4
    ironstar-r6213["ironstar-r62.13<br/>Wire all components together in main.rs"]
    class ironstar-r6213 open
    ironstar-r6213 -.-> ironstar-r62
    ironstar-r6213 ==> ironstar-r6212
    ironstar-r6210["ironstar-r62.10<br/>Implement component-level hypertext t..."]
    class ironstar-r6210 open
    ironstar-r6210 -.-> ironstar-r62
    ironstar-r6210 ==> ironstar-r629
    ironstar-r623["ironstar-r62.3<br/>Configure pre-commit hooks for code q..."]
    class ironstar-r623 open
    ironstar-r623 -.-> ironstar-r62
    ironstar-r623 ==> ironstar-r622
    ironstar-r62["ironstar-r62<br/>Presentation layer"]
    class ironstar-r62 open
    ironstar-r62 ==> ironstar-nyp
    ironstar-r62 ==> ironstar-ny3
    ironstar-nyp11["ironstar-nyp.11<br/>Create Session axum extractor"]
    class ironstar-nyp11 open
    ironstar-nyp11 -.-> ironstar-nyp
    ironstar-nyp11 ==> ironstar-nyp10
    ironstar-nyp10["ironstar-nyp.10<br/>Add session TTL cleanup background task"]
    class ironstar-nyp10 open
    ironstar-nyp10 -.-> ironstar-nyp
    ironstar-nyp10 ==> ironstar-nyp9
    ironstar-nyp9["ironstar-nyp.9<br/>Implement redb session store with ACI..."]
    class ironstar-nyp9 open
    ironstar-nyp9 -.-> ironstar-nyp
    ironstar-nyp9 ==> ironstar-2nt2
    ironstar-nyp4["ironstar-nyp.4<br/>Implement SQLite connection pooling a..."]
    class ironstar-nyp4 open
    ironstar-nyp4 -.-> ironstar-nyp
    ironstar-nyp4 ==> ironstar-nyp3
    ironstar-ny314["ironstar-ny3.14<br/>Create web-components/components/ dir..."]
    class ironstar-ny314 open
    ironstar-ny314 -.-> ironstar-ny3
    ironstar-ny314 ==> ironstar-ny38
    ironstar-ny36["ironstar-ny3.6<br/>Copy Open Props UI component CSS files"]
    class ironstar-ny36 open
    ironstar-ny36 -.-> ironstar-ny3
    ironstar-ny36 ==> ironstar-ny35
    ironstar-ny3["ironstar-ny3<br/>Frontend build pipeline"]
    class ironstar-ny3 open
    ironstar-ny3 ==> ironstar-6lq7
    ironstar-2nt8["ironstar-2nt.8<br/>Define application error types"]
    class ironstar-2nt8 open
    ironstar-2nt8 -.-> ironstar-2nt
    ironstar-2nt8 ==> ironstar-2nt2
    ironstar-apx5["ironstar-apx.5<br/>Add structured logging with tracing"]
    class ironstar-apx5 open
    ironstar-apx5 -.-> ironstar-apx
    ironstar-apx5 ==> ironstar-r6213
    ironstar-apx4["ironstar-apx.4<br/>Create .env.development template file"]
    class ironstar-apx4 open
    ironstar-apx4 -.-> ironstar-apx
    ironstar-apx4 ==> ironstar-nyp3
    ironstar-apx2["ironstar-apx.2<br/>Create template parameters and condit..."]
    class ironstar-apx2 open
    ironstar-apx2 -.-> ironstar-apx
    ironstar-apx2 ==> ironstar-6lq1
    ironstar-apx1["ironstar-apx.1<br/>Create BOOTSTRAP.md with complete set..."]
    class ironstar-apx1 open
    ironstar-apx1 -.-> ironstar-apx
    ironstar-apx1 ==> ironstar-r6213
    ironstar-zuv3["ironstar-zuv.3<br/>Create end-to-end handler tests"]
    class ironstar-zuv3 open
    ironstar-zuv3 -.-> ironstar-zuv
    ironstar-zuv3 ==> ironstar-r6213
    ironstar-zuv2["ironstar-zuv.2<br/>Create projection tests"]
    class ironstar-zuv2 open
    ironstar-zuv2 -.-> ironstar-zuv
    ironstar-zuv2 ==> ironstar-nyp7
    ironstar-zuv1["ironstar-zuv.1<br/>Create event store integration tests"]
    class ironstar-zuv1 open
    ironstar-zuv1 -.-> ironstar-zuv
    ironstar-zuv1 ==> ironstar-nyp3
    ironstar-zuv["ironstar-zuv<br/>Testing and integration"]
    class ironstar-zuv open
    ironstar-zuv ==> ironstar-e6k
    ironstar-7533["ironstar-753.3<br/>Set up Lucide icon build-time inlining"]
    class ironstar-7533 open
    ironstar-7533 -.-> ironstar-753
    ironstar-7533 ==> ironstar-ny32
    ironstar-7532["ironstar-753.2<br/>Implement sortable-list web component..."]
    class ironstar-7532 open
    ironstar-7532 -.-> ironstar-753
    ironstar-7532 ==> ironstar-ny314
    ironstar-7531["ironstar-753.1<br/>Implement VegaChart web component wra..."]
    class ironstar-7531 open
    ironstar-7531 -.-> ironstar-753
    ironstar-7531 ==> ironstar-ny314
    ironstar-753["ironstar-753<br/>Third-party library integration"]
    class ironstar-753 open
    ironstar-753 ==> ironstar-ny3
    ironstar-e6k["ironstar-e6k<br/>Example application (Todo)"]
    class ironstar-e6k open
    ironstar-e6k ==> ironstar-r62
    ironstar-r6215["ironstar-r62.15<br/>Implement health check endpoint for p..."]
    class ironstar-r6215 open
    ironstar-r6215 -.-> ironstar-r62
    ironstar-r6215 ==> ironstar-r6211
    ironstar-r6214["ironstar-r62.14<br/>Implement dev-only hotreload SSE endp..."]
    class ironstar-r6214 open
    ironstar-r6214 -.-> ironstar-r62
    ironstar-r6214 ==> ironstar-r625
    ironstar-nyp12["ironstar-nyp.12<br/>Implement DuckDB analytics service"]
    class ironstar-nyp12 open
    ironstar-nyp12 -.-> ironstar-nyp
    ironstar-nyp12 ==> ironstar-2nt2
    ironstar-apx3["ironstar-apx.3<br/>Define om CLI instantiation tests and..."]
    class ironstar-apx3 open
    ironstar-apx3 -.-> ironstar-apx
    ironstar-apx3 ==> ironstar-apx2
    ironstar-apx["ironstar-apx<br/>Documentation and template"]
    class ironstar-apx open
    ironstar-apx ==> ironstar-zuv
```

---

## 📋 ironstar-r62.12 Implement graceful shutdown signal handling

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

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
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

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
| **Updated** | 2025-12-18 09:36 |

### Description

Create extension trait for hypertext::Renderable:
trait RenderableToDatastar {
    fn to_patch_elements(self) -> PatchElements;
    fn append_to(self, selector: &str) -> PatchElements;
    fn replace_inner(self, selector: &str) -> PatchElements;
}
impl<T: Renderable> RenderableToDatastar for T {
    fn to_patch_elements(self) -> PatchElements { PatchElements::new(self.render().to_string()) }
    fn append_to(self, selector: &str) -> PatchElements { PatchElements::new(self.render().to_string()).append(selector) }
    fn replace_inner(self, selector: &str) -> PatchElements { PatchElements::new(self.render().to_string()).replace_inner(selector) }
}
Bridges hypertext templates to Datastar SSE without manual boilerplate.
Local refs: ~/projects/rust-workspace/hypertext, ~/projects/rust-workspace/datastar-rust

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

## 📋 ironstar-r62.6 Implement command POST handlers

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

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
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

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
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create AppState struct holding Arc<EventStore>, Arc<SessionStore>, Arc<Projections>, broadcast::Sender<StoredEvent>, and optional debug-only reload channel. Implement AppState::new() to initialize all services and replay events to rebuild projections at startup.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/tokio

### Dependencies

- 🔗 **parent-child**: `ironstar-r62`
- ⛔ **blocks**: `ironstar-nyp.3`
- ⛔ **blocks**: `ironstar-nyp.10`
- ⛔ **blocks**: `ironstar-nyp.7`
- ⛔ **blocks**: `ironstar-nyp.5`

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

## 📋 ironstar-nyp.8 Implement SSE 15-second keep-alive comment stream

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Add tokio::time::interval(Duration::from_secs(15)) to SSE stream merging:
let keepalive = tokio_stream::wrappers::IntervalStream::new(interval(Duration::from_secs(15)))
    .map(|_| Event::default().comment("keepalive"));
let events_stream = BroadcastStream::new(rx).filter_map(|e| e.ok());
let merged = stream::select(keepalive, events_stream);
Prevents proxy/firewall timeouts and allows clients to detect broken connections.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-nyp.5`

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

## 📋 ironstar-nyp.7 Implement ProjectionManager with in-memory state

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

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
- ⛔ **blocks**: `ironstar-nyp.5`
- ⛔ **blocks**: `ironstar-nyp.3`

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

## 📋 ironstar-nyp.6 Create Projection trait for read models

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Define async trait with associated types:
#[async_trait]
trait Projection: Send + Sync + 'static {
    type State: Clone + Send + Sync;
    async fn rebuild(&self, events: Vec<StoredEvent>) -> Self::State;
    async fn apply(&self, state: &mut Self::State, event: StoredEvent);
    async fn to_sse_event(&self, state: &Self::State, sequence: u64) -> Result<datastar::Event>;
}
Enables multiple projection types to independently subscribe to events and maintain their own read models.

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

## 📋 ironstar-nyp.5 Implement tokio broadcast event bus

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create EventBus wrapper around tokio::sync::broadcast::channel with Sender holding domain events. Implement publish() method returning Result and subscribe() method returning Receiver. Set default capacity to 256 events. Enables in-process fan-out to multiple subscribers.
Local refs: ~/projects/rust-workspace/tokio

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`

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

## 📋 ironstar-nyp.3 Implement SQLite event store with sqlx

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

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

## 📋 ironstar-nyp.2 Create EventStore trait abstraction

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Define async trait with append, query_all, query_since_sequence, query_aggregate methods using async_trait. Enables swapping implementations (SQLite now, Zenoh later) without changing application code.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`

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
| **Updated** | 2025-12-18 09:36 |

### Description

Initialize migrations/ subdirectory with migrations/schema.sql containing SQLite DDL: events table (id, aggregate_type, aggregate_id, sequence, event_type, payload, metadata, created_at), unique constraint, indexes. Referenced by process-compose db-init.

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`

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

## 🏔️ ironstar-nyp Event sourcing infrastructure

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Event sourcing and CQRS infrastructure implementing append-only event log (SQLite + sqlx), in-memory projection manager with RwLock, tokio broadcast for event distribution, redb for ACID session storage, and optional DuckDB for OLAP analytics. Separates write path (commands emit events) from read path (projections subscribe to events and maintain denormalized views).

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
| **Updated** | 2025-12-18 09:36 |

### Description

Create web-components/tsconfig.json with strict mode enabled, ESNext target and module, bundler moduleResolution, include glob patterns for all TypeScript files and generated types directory. Add path mapping for @types alias.
Local refs: ~/projects/rust-workspace/ts-rs

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
| **Updated** | 2025-12-18 09:36 |

### Description

Add @layer declarations to web-components/styles/main.css:
@layer openprops, normalize, theme, components, utilities, app;
@import 'open-props/style' layer(openprops);
@import 'open-props/normalize' layer(normalize);
@import './theme.css' layer(theme);
@import './components/button.css' layer(components);
Establishes explicit cascade order preventing specificity wars and making component styles easily overridable at app layer.

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
| **Updated** | 2025-12-18 09:36 |

### Description

Create web-components/rolldown.config.ts:
import { defineConfig } from 'rolldown';
import postcss from 'rolldown-plugin-postcss';
export default defineConfig({
  input: { bundle: 'index.ts', components: 'components/index.ts' },
  output: { dir: '../static/dist', format: 'esm', entryFileNames: '[name].[hash].js' },
  plugins: [postcss({ extract: true, modules: false })],
});
Outputs content-hashed assets (bundle.[hash].js, bundle.[hash].css, manifest.json) for cache-busting and single-binary asset embedding.
Local refs: ~/projects/rust-workspace/rolldown (clone needed: https://github.com/rolldown/rolldown)

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

## 📋 ironstar-ny3.1 Create web-components/ project structure with package.json

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

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
| **Updated** | 2025-12-18 09:36 |

### Description

Implement sum types for DomainEvent, Command, and aggregate states as Rust enums with serde serialization:
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum DomainEvent {
    TodoCreated { id: Uuid, text: String, created_at: DateTime<Utc> },
    TodoCompleted { id: Uuid, completed_at: DateTime<Utc> },
    TodoDeleted { id: Uuid, deleted_at: DateTime<Utc> },
}
Establishes the core algebraic vocabulary making invalid states unrepresentable and ensures type-level guarantees for all domain logic.
Local refs: ~/projects/rust-workspace/ironstar

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

## 📋 ironstar-2nt.1 Initialize src/ directory structure with modular organization

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create src/ subdirectories for domain/ (aggregates, events, commands, values, signals.rs), application/ (command_handlers, query_handlers, projections), infrastructure/ (event_store, session_store, analytics, event_bus), and presentation/ (routes, handlers, templates). Create placeholder mod.rs files.
Local refs: CLAUDE.md Project structure section

### Dependencies

- 🔗 **parent-child**: `ironstar-2nt`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-2nt.1 -s in_progress

# Add a comment
bd comment ironstar-2nt.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-2nt.1 -p 1

# View full details
bd show ironstar-2nt.1
```

</details>

---

## 🏔️ ironstar-2nt Domain layer

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Implement the domain layer using algebraic data types to make invalid states unrepresentable. Defines sum types for domain events and commands, product types for aggregates and value objects, smart constructors for validation, and ts-rs derives for TypeScript signal contract generation. Establishes the core vocabulary of the application with type-level guarantees.

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

## 🏔️ ironstar-f8b Process compose integration

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
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
| **Updated** | 2025-12-18 09:36 |

### Description

Extend template's .github/workflows/ci.yml inherited from typescript-nix-template with Rust jobs:
- Build matrix for multiple platforms (Linux x86_64, macOS aarch64)
- Category-based workflows: build (nix build), test (cargo test), lint (cargo fmt/clippy)
- Content-addressed caching via cachix or GitHub Actions cache
- Integrate with flake checks: nix flake check
Follows template CI patterns adapted for Rust toolchain.
Local refs: ~/projects/nix-workspace/typescript-nix-template/.github/workflows/, ~/projects/rust-workspace/rust-nix-template/.github/

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.6`

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

## 📋 ironstar-6lq.6 Add Rust checks to flake.checks for CI integration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create nix/modules/checks.nix defining perSystem.checks with:
- cargo-test: cargo test --workspace --all-features
- cargo-clippy: cargo clippy --workspace --all-targets -- -D warnings
- cargo-fmt: cargo fmt --all -- --check
- cargo-doc: cargo doc --workspace --no-deps --document-private-items
Expose as flake.checks.* for nix flake check and CI.
Local refs: ~/projects/nix-workspace/typescript-nix-template/modules/checks/, ~/projects/rust-workspace/rust-nix-template

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.4`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-6lq.6 -s in_progress

# Add a comment
bd comment ironstar-6lq.6 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-6lq.6 -p 1

# View full details
bd show ironstar-6lq.6
```

</details>

---

## 📋 ironstar-6lq.5 Verify cargo check passes with workspace configuration

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Run cargo check to validate workspace configuration, dependency resolution, and that all crates compile. Fix any issues with feature flags or dependency versions. Ensures Rust workspace is properly configured before proceeding to process orchestration.

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.4`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-6lq.5 -s in_progress

# Add a comment
bd comment ironstar-6lq.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-6lq.5 -p 1

# View full details
bd show ironstar-6lq.5
```

</details>

---

## 📋 ironstar-6lq.4 Set up per-crate crate.nix pattern for crane args

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

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

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-6lq.4 -s in_progress

# Add a comment
bd comment ironstar-6lq.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-6lq.4 -p 1

# View full details
bd show ironstar-6lq.4
```

</details>

---

## 📋 ironstar-6lq.3 Configure Cargo.toml with workspace structure (resolver = 2)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create Cargo.toml at repository root with [workspace], resolver = "2", members array, and workspace.dependencies section for DRY dependency management. Include all core dependencies: axum, tokio, sqlx, duckdb, ts-rs, datastar, hypertext, redb, rust-embed, thiserror. Add release profile optimizations.
Local refs: ~/projects/rust-workspace/rustlings-workspace/Cargo.toml

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-6lq.3 -s in_progress

# Add a comment
bd comment ironstar-6lq.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-6lq.3 -p 1

# View full details
bd show ironstar-6lq.3
```

</details>

---

## 📋 ironstar-6lq.2 Add rust-toolchain.toml with required components

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create rust-toolchain.toml at repository root specifying stable channel with components: rustfmt, clippy, rust-analyzer, rust-src. Ensures consistent Rust version across development environments and CI.
Local refs: ~/projects/rust-workspace/rust-nix-template/rust-toolchain.toml

### Dependencies

- 🔗 **parent-child**: `ironstar-6lq`
- ⛔ **blocks**: `ironstar-6lq.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-6lq.2 -s in_progress

# Add a comment
bd comment ironstar-6lq.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-6lq.2 -p 1

# View full details
bd show ironstar-6lq.2
```

</details>

---

## 📋 ironstar-6lq.1 Integrate rust-flake patterns (crane, rust-overlay)

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

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

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-6lq.1 -s in_progress

# Add a comment
bd comment ironstar-6lq.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-6lq.1 -p 1

# View full details
bd show ironstar-6lq.1
```

</details>

---

## 🏔️ ironstar-6lq Rust workspace integration

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
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

## 📋 ironstar-cxe.5 Create .gitignore with comprehensive patterns

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create .gitignore at repository root with patterns: /target/, Cargo.lock, /static/dist/, web-components/dist, node_modules, .env*, dev.db*, .DS_Store, .direnv, result, .beads/. Protects against accidental secret commits and build artifacts.

### Dependencies

- 🔗 **parent-child**: `ironstar-cxe`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-cxe.5 -s in_progress

# Add a comment
bd comment ironstar-cxe.5 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-cxe.5 -p 1

# View full details
bd show ironstar-cxe.5
```

</details>

---

## 📋 ironstar-cxe.4 Create initial git commit with generated structure

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Stage all generated files from om init and create initial commit with message: 'feat: initialize ironstar from typescript-nix-template'. Establishes baseline for tracking subsequent changes.

### Dependencies

- 🔗 **parent-child**: `ironstar-cxe`
- ⛔ **blocks**: `ironstar-cxe.3`
- ⛔ **blocks**: `ironstar-cxe.2`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-cxe.4 -s in_progress

# Add a comment
bd comment ironstar-cxe.4 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-cxe.4 -p 1

# View full details
bd show ironstar-cxe.4
```

</details>

---

## 📋 ironstar-cxe.3 Verify nix develop enters working development shell

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Test that nix develop successfully enters the devShell with basic tooling available. Verify nixd, direnv, and foundational utilities are present. This validates the template instantiation before proceeding to Rust integration.

### Dependencies

- 🔗 **parent-child**: `ironstar-cxe`
- ⛔ **blocks**: `ironstar-cxe.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-cxe.3 -s in_progress

# Add a comment
bd comment ironstar-cxe.3 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-cxe.3 -p 1

# View full details
bd show ironstar-cxe.3
```

</details>

---

## 📋 ironstar-cxe.2 Configure secrets management and string replacement

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create .env.development template with DATABASE_URL, LOG_LEVEL, SERVER_PORT, RELOAD_ENABLED. Replace template placeholder strings with ironstar-specific values. Add .env* to .gitignore to prevent secret commits.
Local refs: ~/.claude/commands/preferences/secrets.md

### Dependencies

- 🔗 **parent-child**: `ironstar-cxe`
- ⛔ **blocks**: `ironstar-cxe.1`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-cxe.2 -s in_progress

# Add a comment
bd comment ironstar-cxe.2 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-cxe.2 -p 1

# View full details
bd show ironstar-cxe.2
```

</details>

---

## 📋 ironstar-cxe.1 Run om init with typescript-nix-template parameters

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Execute om init github:user/typescript-nix-template/main with parameters: project-name (ironstar), github-ci (true), nix-template (true). This generates the initial flake structure with flake-parts, import-tree module composition, and GitHub Actions workflows.
Local refs: ~/projects/nix-workspace/typescript-nix-template

### Dependencies

- 🔗 **parent-child**: `ironstar-cxe`

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-cxe.1 -s in_progress

# Add a comment
bd comment ironstar-cxe.1 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-cxe.1 -p 1

# View full details
bd show ironstar-cxe.1
```

</details>

---

## 🏔️ ironstar-cxe Template instantiation

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
| **Priority** | 🔥 Critical (P0) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Bootstrap the ironstar project from typescript-nix-template using omnix om CLI. This epic establishes the foundational Nix flake structure with deterministic development environments, secrets management patterns, and git repository initialization. Validates that the template instantiation succeeds before proceeding to Rust-specific integration.

<details>
<summary>📋 Commands</summary>

```bash
# Start working on this issue
bd update ironstar-cxe -s in_progress

# Add a comment
bd comment ironstar-cxe 'Your comment here'

# Change priority (0=Critical, 1=High, 2=Medium, 3=Low)
bd update ironstar-cxe -p 1

# View full details
bd show ironstar-cxe
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

## 🏔️ ironstar-r62 Presentation layer

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

HTTP presentation layer using axum for routing and extractors, SSE for server-sent events with Last-Event-ID reconnection, hypertext for lazy HTML template rendering, datastar-rust for SSE generation conforming to Datastar SDK specification, and devShell configuration. Implements Tao of Datastar principles: backend as source of truth, fat morph for resilience, CQRS separation of read/write endpoints.

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

## 📋 ironstar-nyp.9 Implement redb session store with ACID guarantees

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create SessionStore wrapper around redb::Database with get(session_id) and put(session_id, data) methods. Use bincode serialization for SessionData struct. Implement linear type semantics with WriteTransaction bracket pattern. Provides server-side session storage with ACID durability.
Local refs: ~/projects/rust-workspace/redb, ~/projects/rust-workspace/redb/docs/design.md

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

## 🏔️ ironstar-ny3 Frontend build pipeline

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
| **Priority** | ⚡ High (P1) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Build pipeline for frontend assets using Rolldown (Rust-native bundler), PostCSS for modern CSS (OKLch, light-dark, container queries), Open Props design tokens, Open Props UI component CSS, TypeScript for type safety, and ts-rs for Rust-to-TypeScript type generation. Outputs content-hashed bundles for single-binary embedding via rust-embed.

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
| **Updated** | 2025-12-18 09:36 |

### Description

Create AppError enum using thiserror::Error with variants for Validation, NotFound, Database, Internal. Implement From conversions and IntoResponse for proper HTTP responses.

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

## 🏔️ ironstar-zuv Testing and integration

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
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
| **Updated** | 2025-12-18 09:36 |

### Description

Create vanilla TypeScript web component (web-components/components/vega-chart.ts) that wraps vega-embed, stores Result and View instances, implements observedAttributes=['spec-url', 'data-url', 'signal-values'], and calls result?.finalize() on disconnect. Must use data-ignore-morph to prevent Datastar from morphing Vega's DOM.
Local refs: ~/projects/lakescope-workspace/vega-embed, ~/projects/lakescope-workspace/datastar-go-nats-template-northstar

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

## 🏔️ ironstar-753 Third-party library integration

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Integration patterns for third-party JavaScript libraries with Datastar: Pattern 1 vanilla web components as thin wrappers (sortable-list), Pattern 2 Vega-Lite with data-ignore-morph and View API updates, and build-time Lucide icon inlining. Demonstrates how to bridge imperative JS libraries with declarative hypermedia-driven architecture while preserving Datastar's backend-as-source-of-truth principle.

### Dependencies

- ⛔ **blocks**: `ironstar-ny3`

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

## 🏔️ ironstar-e6k Example application (Todo)

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
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

## 📋 ironstar-nyp.12 Implement DuckDB analytics service

| Property | Value |
|----------|-------|
| **Type** | 📋 task |
| **Priority** | 🔹 Medium (P2) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Create AnalyticsService wrapping duckdb::Connection using one-connection-per-task pattern. Implement query methods returning Vec of analytics results. Wrap blocking operations in tokio::task::block_in_place() for quick queries. Enables OLAP queries over event history.
Local refs: ~/projects/omicslake-workspace/duckdb-rs

### Dependencies

- 🔗 **parent-child**: `ironstar-nyp`
- ⛔ **blocks**: `ironstar-2nt.2`

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

## 🏔️ ironstar-apx Documentation and template

| Property | Value |
|----------|-------|
| **Type** | 🏔️ epic |
| **Priority** | ☕ Low (P3) |
| **Status** | 🟢 open |
| **Created** | 2025-12-18 09:36 |
| **Updated** | 2025-12-18 09:36 |

### Description

Template finalization with omnix integration (om CLI parameters, conditional file inclusion), comprehensive BOOTSTRAP.md documentation, environment configuration templates, and structured logging via tracing. Enables users to instantiate ironstar as a template project with parameterized customization following typescript-nix-template and rust-nix-template patterns.

### Dependencies

- ⛔ **blocks**: `ironstar-zuv`

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

