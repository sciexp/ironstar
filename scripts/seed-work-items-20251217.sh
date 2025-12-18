#!/usr/bin/env bash
# Ironstar Work Items Seed Script
# Generated: 2025-12-17
# Refactored: Restructured around template instantiation workflow
#
# Creates epics, tasks, and dependencies for the ironstar project development roadmap.
# The workflow starts with template instantiation (not reference study) since only
# implementation produces git diffs that can be tracked as "done".
#
# Epic structure (11 epics):
#   1. Template Instantiation (om init, secrets, verification)
#   2. Rust Workspace Integration (crane, rust-overlay, Cargo workspace)
#   3. Process Compose Integration (flake integration, service orchestration)
#   4. CI/CD Pipeline (GitHub Actions, flake checks, caching)
#   5. Domain Layer (algebraic types, aggregates, signals)
#   6. Frontend Build Pipeline (Rolldown, PostCSS, Open Props, web components)
#   7. Event Sourcing Infrastructure (SQLite, redb, tokio broadcast, DuckDB)
#   8. Presentation Layer (axum, SSE, hypertext templates)
#   9. Example Application - Todo (complete demo)
#   10. Testing and Integration (unit tests, integration tests, CI)
#   11. Documentation and Template (bootstrap guide, om CLI integration)
#
# Usage: ./scripts/seed-work-items-20251217.sh
#
# Prerequisites:
#   - beads CLI (bd) installed and configured
#   - .beads/ directory initialized in repository root

set -euo pipefail

# Color output helpers
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Verify beads is available
if ! command -v bd &> /dev/null; then
    log_error "beads CLI (bd) not found. Please install beads first."
    exit 1
fi

# Change to repository root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

log_info "Creating ironstar work items..."
log_info "Working directory: $(pwd)"

# Helper to extract issue ID from bd create output
# bd create outputs: "Created issue: ironstar-xyz"
extract_id() {
    grep -oE 'ironstar-[a-z0-9]+(\.[0-9]+)?' | head -1
}

# Helper to create issue and capture ID
create_issue() {
    local output
    output=$(bd create "$@" 2>&1)
    echo "$output" | extract_id
}

###############################################################################
# EPIC 1: Template Instantiation
###############################################################################
log_info "Creating Epic 1: Template Instantiation..."

EPIC_TEMPLATE=$(create_issue "Template instantiation" -t epic -p 0)
log_success "Created epic: $EPIC_TEMPLATE"

TASK_OM_INIT=$(bd create "Run om init with typescript-nix-template parameters" \
    -p 0 --parent "$EPIC_TEMPLATE" 2>&1 | extract_id)
bd update "$TASK_OM_INIT" --description "Execute om init github:user/typescript-nix-template/main with parameters: project-name (ironstar), github-ci (true), nix-template (true). This generates the initial flake structure with flake-parts, import-tree module composition, and GitHub Actions workflows.
Local refs: ~/projects/nix-workspace/typescript-nix-template"

TASK_SECRETS_CONFIG=$(bd create "Configure secrets management and string replacement" \
    -p 0 --parent "$EPIC_TEMPLATE" 2>&1 | extract_id)
bd update "$TASK_SECRETS_CONFIG" --description "Create .env.development template with DATABASE_URL, LOG_LEVEL, SERVER_PORT, RELOAD_ENABLED. Replace template placeholder strings with ironstar-specific values. Add .env* to .gitignore to prevent secret commits.
Local refs: ~/.claude/commands/preferences/secrets.md"
bd dep add "$TASK_SECRETS_CONFIG" "$TASK_OM_INIT"

TASK_VERIFY_SHELL=$(bd create "Verify nix develop enters working development shell" \
    -p 0 --parent "$EPIC_TEMPLATE" 2>&1 | extract_id)
bd update "$TASK_VERIFY_SHELL" --description "Test that nix develop successfully enters the devShell with basic tooling available. Verify nixd, direnv, and foundational utilities are present. This validates the template instantiation before proceeding to Rust integration."
bd dep add "$TASK_VERIFY_SHELL" "$TASK_OM_INIT"

TASK_INIT_COMMIT=$(bd create "Create initial git commit with generated structure" \
    -p 0 --parent "$EPIC_TEMPLATE" 2>&1 | extract_id)
bd update "$TASK_INIT_COMMIT" --description "Stage all generated files from om init and create initial commit with message: 'feat: initialize ironstar from typescript-nix-template'. Establishes baseline for tracking subsequent changes."
bd dep add "$TASK_INIT_COMMIT" "$TASK_VERIFY_SHELL"
bd dep add "$TASK_INIT_COMMIT" "$TASK_SECRETS_CONFIG"

TASK_GITIGNORE=$(bd create "Create .gitignore with comprehensive patterns" \
    -p 0 --parent "$EPIC_TEMPLATE" 2>&1 | extract_id)
bd update "$TASK_GITIGNORE" --description "Create .gitignore at repository root with patterns: /target/, Cargo.lock, /static/dist/, web-components/dist, node_modules, .env*, dev.db*, .DS_Store, .direnv, result, .beads/. Protects against accidental secret commits and build artifacts."
bd dep add "$TASK_GITIGNORE" "$TASK_INIT_COMMIT"

###############################################################################
# EPIC 2: Rust Workspace Integration
###############################################################################
log_info "Creating Epic 2: Rust Workspace Integration..."

EPIC_RUST=$(create_issue "Rust workspace integration" -t epic -p 0)
log_success "Created epic: $EPIC_RUST"
bd dep add "$EPIC_RUST" "$EPIC_TEMPLATE"

TASK_RUST_FLAKE=$(bd create "Integrate rust-flake patterns (crane, rust-overlay)" \
    -p 0 --parent "$EPIC_RUST" 2>&1 | extract_id)
bd update "$TASK_RUST_FLAKE" --description "Create nix/modules/rust.nix importing rust-flake module. Configure crane for Rust builds, rust-overlay for toolchain management, and per-crate crane.args pattern. Add platform-specific buildInputs (darwin frameworks, openssl). Establishes deterministic Rust build infrastructure.
Local refs: ~/projects/rust-workspace/rust-nix-template/nix/modules/rust.nix, ~/projects/rust-workspace/rustlings-workspace/nix/modules/rust.nix"

TASK_RUST_TOOLCHAIN=$(bd create "Add rust-toolchain.toml with required components" \
    -p 0 --parent "$EPIC_RUST" 2>&1 | extract_id)
bd update "$TASK_RUST_TOOLCHAIN" --description "Create rust-toolchain.toml at repository root specifying stable channel with components: rustfmt, clippy, rust-analyzer, rust-src. Ensures consistent Rust version across development environments and CI.
Local refs: ~/projects/rust-workspace/rust-nix-template/rust-toolchain.toml"
bd dep add "$TASK_RUST_TOOLCHAIN" "$TASK_RUST_FLAKE"

TASK_CARGO_WS=$(bd create "Configure Cargo.toml with workspace structure (resolver = 2)" \
    -p 0 --parent "$EPIC_RUST" 2>&1 | extract_id)
bd update "$TASK_CARGO_WS" --description "Create Cargo.toml at repository root with [workspace], resolver = \"2\", members array, and workspace.dependencies section for DRY dependency management. Include all core dependencies: axum, tokio, sqlx, duckdb, ts-rs, datastar, hypertext, redb, rust-embed, thiserror. Add release profile optimizations.
Local refs: ~/projects/rust-workspace/rustlings-workspace/Cargo.toml"
bd dep add "$TASK_CARGO_WS" "$TASK_RUST_TOOLCHAIN"

TASK_CRATE_NIX=$(bd create "Set up per-crate crate.nix pattern for crane args" \
    -p 0 --parent "$EPIC_RUST" 2>&1 | extract_id)
bd update "$TASK_CRATE_NIX" --description "Create crate.nix files for each workspace crate defining crane-specific build arguments. Implements pattern from rustlings-workspace for granular build customization. Currently only needed for main ironstar crate, but establishes pattern for future workspace expansion.
Local refs: ~/projects/rust-workspace/rustlings-workspace/"
bd dep add "$TASK_CRATE_NIX" "$TASK_CARGO_WS"

TASK_CARGO_CHECK=$(bd create "Verify cargo check passes with workspace configuration" \
    -p 0 --parent "$EPIC_RUST" 2>&1 | extract_id)
bd update "$TASK_CARGO_CHECK" --description "Run cargo check to validate workspace configuration, dependency resolution, and that all crates compile. Fix any issues with feature flags or dependency versions. Ensures Rust workspace is properly configured before proceeding to process orchestration."
bd dep add "$TASK_CARGO_CHECK" "$TASK_CRATE_NIX"

###############################################################################
# EPIC 3: Process Compose Integration
###############################################################################
log_info "Creating Epic 3: Process Compose Integration..."

EPIC_PROCESS=$(create_issue "Process compose integration" -t epic -p 0)
log_success "Created epic: $EPIC_PROCESS"
bd dep add "$EPIC_PROCESS" "$EPIC_RUST"

TASK_PC_FLAKE=$(bd create "Integrate process-compose-flake patterns into devShell" \
    -p 0 --parent "$EPIC_PROCESS" 2>&1 | extract_id)
bd update "$TASK_PC_FLAKE" --description "Create nix/modules/process-compose.nix importing process-compose-flake.flakeModule. Define perSystem process-compose configurations. Expose as packages.dev runnable via nix run .#dev. Integrates declarative process orchestration into Nix workflow.
Local refs: ~/projects/nix-workspace/process-compose-flake"

TASK_PC_YAML=$(bd create "Configure process-compose.yaml for dev services" \
    -p 0 --parent "$EPIC_PROCESS" 2>&1 | extract_id)
bd update "$TASK_PC_YAML" --description "Create process-compose.yaml with processes: db-init (one-shot SQLite schema), frontend (Rolldown watch), typegen (ts-rs watch), backend (cargo watch), hotreload (browser SSE trigger). Define process dependencies, readiness probes, and log_location for each service.
Local refs: ~/projects/nix-workspace/process-compose"
bd dep add "$TASK_PC_YAML" "$TASK_PC_FLAKE"

TASK_SERVICE_ORCH=$(bd create "Set up service orchestration (frontend bundler, cargo-watch)" \
    -p 0 --parent "$EPIC_PROCESS" 2>&1 | extract_id)
bd update "$TASK_SERVICE_ORCH" --description "Configure service startup order and dependencies in process-compose.yaml. Ensure db-init completes before backend starts, typegen runs when Rust files change, frontend rebuilds on TypeScript changes, backend restarts on Rust changes, hotreload triggers browser refresh after successful backend build."
bd dep add "$TASK_SERVICE_ORCH" "$TASK_PC_YAML"

TASK_PC_VERIFY=$(bd create "Verify process-compose up works with all services" \
    -p 0 --parent "$EPIC_PROCESS" 2>&1 | extract_id)
bd update "$TASK_PC_VERIFY" --description "Test that process-compose up successfully starts all services in correct order. Verify readiness probes work, dependencies are respected, and logs are properly separated. Test that services restart appropriately when files change."
bd dep add "$TASK_PC_VERIFY" "$TASK_SERVICE_ORCH"

###############################################################################
# EPIC 4: CI/CD Pipeline
###############################################################################
log_info "Creating Epic 4: CI/CD Pipeline..."

EPIC_CI=$(create_issue "CI/CD pipeline" -t epic -p 0)
log_success "Created epic: $EPIC_CI"
bd dep add "$EPIC_CI" "$EPIC_PROCESS"

TASK_GHA_WORKFLOW=$(bd create "Create reusable GitHub Actions workflow for Rust builds" \
    -p 0 --parent "$EPIC_CI" 2>&1 | extract_id)
bd update "$TASK_GHA_WORKFLOW" --description "Create .github/workflows/ci.yml with jobs: check (cargo check), test (cargo test), lint (cargo fmt --check, cargo clippy), frontend (pnpm typecheck, pnpm lint). Use actions-rs/toolchain for Rust setup. Follows template CI structure pattern.
Local refs: ~/projects/rust-workspace/rust-nix-template/.github/, ~/projects/nix-workspace/typescript-nix-template/.github/"

TASK_TEMPLATE_CI=$(bd create "Integrate with template CI structure" \
    -p 0 --parent "$EPIC_CI" 2>&1 | extract_id)
bd update "$TASK_TEMPLATE_CI" --description "Adapt typescript-nix-template CI patterns for Rust: category-based workflows (build, test, lint), content-addressed caching, matrix builds for multiple platforms. Ensure CI integrates seamlessly with Nix flake checks.
Local refs: ~/projects/nix-workspace/typescript-nix-template/.github/workflows/"
bd dep add "$TASK_TEMPLATE_CI" "$TASK_GHA_WORKFLOW"

TASK_FLAKE_CHECKS=$(bd create "Add flake checks and nix build verification" \
    -p 0 --parent "$EPIC_CI" 2>&1 | extract_id)
bd update "$TASK_FLAKE_CHECKS" --description "Create nix/modules/checks.nix defining perSystem.checks with: cargo test, cargo clippy (pedantic), cargo fmt --check, cargo doc --no-deps, frontend typecheck. Expose as flake.checks for nix flake check command.
Local refs: ~/projects/nix-workspace/typescript-nix-template/modules/checks/"
bd dep add "$TASK_FLAKE_CHECKS" "$TASK_GHA_WORKFLOW"

TASK_CI_CACHING=$(bd create "Configure caching for cargo and nix" \
    -p 1 --parent "$EPIC_CI" 2>&1 | extract_id)
bd update "$TASK_CI_CACHING" --description "Set up GitHub Actions caching for: cargo registry/git/target directories, nix store paths. Use cachix for Nix binary cache if applicable. Reduces CI build times significantly for incremental changes."
bd dep add "$TASK_CI_CACHING" "$TASK_TEMPLATE_CI"

###############################################################################
# EPIC 5: Domain Layer
###############################################################################
log_info "Creating Epic 5: Domain Layer..."

EPIC_DOMAIN=$(create_issue "Domain layer" -t epic -p 0)
log_success "Created epic: $EPIC_DOMAIN"
bd dep add "$EPIC_DOMAIN" "$EPIC_CI"

TASK_SRC_STRUCTURE=$(bd create "Initialize src/ directory structure with modular organization" \
    -p 0 --parent "$EPIC_DOMAIN" 2>&1 | extract_id)
bd update "$TASK_SRC_STRUCTURE" --description "Create src/ subdirectories for domain/ (aggregates, events, commands, values, signals.rs), application/ (command_handlers, query_handlers, projections), infrastructure/ (event_store, session_store, analytics, event_bus), and presentation/ (routes, handlers, templates). Create placeholder mod.rs files.
Local refs: CLAUDE.md Project structure section"

TASK_DOMAIN_TYPES=$(bd create "Define algebraic domain types and aggregate structure" \
    -p 0 --parent "$EPIC_DOMAIN" 2>&1 | extract_id)
bd update "$TASK_DOMAIN_TYPES" --description "Implement sum types for DomainEvent, Command, and aggregate states as Rust enums with serde serialization. Establishes the core algebraic vocabulary making invalid states unrepresentable and ensures type-level guarantees for all domain logic.
Local refs: ~/projects/rust-workspace/ironstar"
bd dep add "$TASK_DOMAIN_TYPES" "$TASK_SRC_STRUCTURE"

TASK_VALUE_OBJECTS=$(bd create "Implement value objects and smart constructors" \
    -p 0 --parent "$EPIC_DOMAIN" 2>&1 | extract_id)
bd update "$TASK_VALUE_OBJECTS" --description "Create validated value objects (e.g., TodoText, SessionId) with smart constructor functions that enforce invariants at construction time. Product types reject invalid values before they enter the system, preventing bug vectors at the type level."
bd dep add "$TASK_VALUE_OBJECTS" "$TASK_DOMAIN_TYPES"

TASK_AGGREGATES=$(bd create "Design aggregate root state machines" \
    -p 0 --parent "$EPIC_DOMAIN" 2>&1 | extract_id)
bd update "$TASK_AGGREGATES" --description "Model aggregate state machines using Rust enums (e.g., TodoAggregate as an enum of Todo variants with different state). Apply commands to aggregates via pure functions that validate state transitions and emit events as output."
bd dep add "$TASK_AGGREGATES" "$TASK_VALUE_OBJECTS"

TASK_SIGNAL_TYPES=$(bd create "Create Datastar signal types with ts-rs derives" \
    -p 0 --parent "$EPIC_DOMAIN" 2>&1 | extract_id)
bd update "$TASK_SIGNAL_TYPES" --description "Define frontend signal contract types using serde::Serialize + ts_rs::TS derives so TypeScript definitions auto-generate. These types specify the shape of signals flowing from browser to server, ensuring type safety across the HTTP boundary.
Local refs: ~/projects/rust-workspace/ts-rs"
bd dep add "$TASK_SIGNAL_TYPES" "$TASK_DOMAIN_TYPES"

TASK_APP_ERRORS=$(bd create "Define application error types" \
    -p 1 --parent "$EPIC_DOMAIN" 2>&1 | extract_id)
bd update "$TASK_APP_ERRORS" --description "Create AppError enum using thiserror::Error with variants for Validation, NotFound, Database, Internal. Implement From conversions and IntoResponse for proper HTTP responses."
bd dep add "$TASK_APP_ERRORS" "$TASK_DOMAIN_TYPES"

###############################################################################
# EPIC 6: Frontend Build Pipeline
###############################################################################
log_info "Creating Epic 6: Frontend Build Pipeline..."

EPIC_FRONTEND=$(create_issue "Frontend build pipeline" -t epic -p 1)
log_success "Created epic: $EPIC_FRONTEND"
bd dep add "$EPIC_FRONTEND" "$EPIC_CI"

TASK_WC_DIR=$(bd create "Create web-components/ project structure with package.json" \
    -p 0 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_WC_DIR" --description "Initialize web-components/ subdirectory with package.json (type: module, scripts: dev/build for Rolldown), tsconfig.json (target ES2020, experimentalDecorators, strict mode), and PostCSS configuration. Establishes the frontend asset build pipeline.
Local refs: ~/projects/lakescope-workspace/open-props, ~/projects/lakescope-workspace/open-props-ui"

TASK_ROLLDOWN=$(bd create "Configure Rolldown bundler with content-based hashing" \
    -p 0 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_ROLLDOWN" --description "Create web-components/rolldown.config.ts with input entries (bundle: index.ts, components: components/index.ts), output directory (static/dist), ESM format, content-based hashing ([name].[hash].js), and postcss-plugin for CSS extraction. Enables cache-busting and single-binary asset embedding.
Local refs: ~/projects/rust-workspace/rolldown (clone needed: https://github.com/rolldown/rolldown)"
bd dep add "$TASK_ROLLDOWN" "$TASK_WC_DIR"

TASK_POSTCSS=$(bd create "Setup PostCSS configuration for modern CSS features" \
    -p 0 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_POSTCSS" --description "Create web-components/postcss.config.js with plugins: postcss-import, postcss-preset-env (stage 0 for OKLch/light-dark/custom-media), autoprefixer, cssnano. Enables Open Props and modern CSS features.
Local refs: ~/projects/lakescope-workspace/open-props/"
bd dep add "$TASK_POSTCSS" "$TASK_WC_DIR"

TASK_OPEN_PROPS=$(bd create "Setup Open Props design tokens and theme layer" \
    -p 0 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_OPEN_PROPS" --description "Create web-components/styles/main.css importing Open Props design tokens. Create web-components/styles/theme.css with CSS custom properties using light-dark() function for automatic dark mode. Establish CSS cascade layers: openprops, normalize, theme, components, utilities, app.
Local refs: ~/projects/lakescope-workspace/open-props, ~/projects/lakescope-workspace/open-props-ui"
bd dep add "$TASK_OPEN_PROPS" "$TASK_POSTCSS"

TASK_OPUI_COMPONENTS=$(bd create "Copy Open Props UI component CSS files" \
    -p 1 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_OPUI_COMPONENTS" --description "Copy component CSS from ~/projects/lakescope-workspace/open-props-ui into web-components/styles/components/ (button.css, card.css, dialog.css, input.css, field.css, etc). Customize for ironstar theming. This follows the copy-paste ownership model where project owns and customizes component CSS.
Local refs: ~/projects/lakescope-workspace/open-props-ui"
bd dep add "$TASK_OPUI_COMPONENTS" "$TASK_OPEN_PROPS"

TASK_TSCONFIG=$(bd create "Create TypeScript configuration (tsconfig.json)" \
    -p 0 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_TSCONFIG" --description "Create web-components/tsconfig.json with strict mode enabled, ESNext target and module, bundler moduleResolution, include glob patterns for all TypeScript files and generated types directory. Add path mapping for @types alias.
Local refs: ~/projects/rust-workspace/ts-rs"
bd dep add "$TASK_TSCONFIG" "$TASK_WC_DIR"

TASK_WC_INDEX=$(bd create "Create web-components/index.ts entry point" \
    -p 0 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_WC_INDEX" --description "Create index.ts that imports main.css (processed by PostCSS plugin) and auto-registers vanilla web components by importing from components/ subdirectory. Export TypeScript types from web-components/types/ for frontend type safety.
Local refs: ~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/index.ts"
bd dep add "$TASK_WC_INDEX" "$TASK_TSCONFIG"

TASK_TSRS_CARGO=$(bd create "Add ts-rs dependency to Cargo.toml" \
    -p 0 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_TSRS_CARGO" --description "Add ts-rs 11.1+ with features serde-compat and uuid-impl. Enables deriving TS traits on Rust types to generate TypeScript definitions. Ensures frontend and backend signal contracts stay synchronized.
Local refs: ~/projects/rust-workspace/ts-rs"

TASK_TSRS_CONFIG=$(bd create "Configure ts-rs export directory and justfile task" \
    -p 0 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_TSRS_CONFIG" --description "Add [env] section to .cargo/config.toml setting TS_RS_EXPORT_DIR. Create gen-types task in justfile: TS_RS_EXPORT_DIR=web-components/types cargo test --lib. Centralizes type generation configuration.
Local refs: ~/projects/rust-workspace/ts-rs"
bd dep add "$TASK_TSRS_CONFIG" "$TASK_TSRS_CARGO"

TASK_STATIC_DIST=$(bd create "Create static/dist/ output directory structure" \
    -p 0 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_STATIC_DIST" --description "Initialize static/dist/ directory placeholder for Rolldown build outputs (bundle.[hash].css, bundle.[hash].js, manifest.json). Create static/datastar/ for runtime datastar.js. Aligns with single-binary asset embedding in production."
bd dep add "$TASK_STATIC_DIST" "$TASK_ROLLDOWN"

TASK_RUST_EMBED=$(bd create "Implement rust-embed conditional asset serving" \
    -p 0 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_RUST_EMBED" --description "Create dual-mode asset serving: dev mode serves from filesystem via tower-http::ServeDir with no-store cache headers; prod mode embeds static/dist/ via rust-embed with immutable cache headers. Include AssetManifest loader for hashed filename resolution.
Local refs: ~/projects/rust-workspace/rust-embed"
bd dep add "$TASK_RUST_EMBED" "$TASK_STATIC_DIST"

TASK_WC_COMPONENTS=$(bd create "Create web-components/components/ directory for vanilla web components" \
    -p 1 --parent "$EPIC_FRONTEND" 2>&1 | extract_id)
bd update "$TASK_WC_COMPONENTS" --description "Set up web-components/components/ directory structure for vanilla web components. Create index.ts that exports/registers all components. Contains thin wrapper web components for third-party libraries following the data-ignore-morph pattern with Datastar integration.
Local refs: ~/projects/lakescope-workspace/datastar-go-nats-template-northstar/web/components/"
bd dep add "$TASK_WC_COMPONENTS" "$TASK_WC_INDEX"

###############################################################################
# EPIC 7: Event Sourcing Infrastructure
###############################################################################
log_info "Creating Epic 7: Event Sourcing Infrastructure..."

EPIC_EVENTSRC=$(create_issue "Event sourcing infrastructure" -t epic -p 0)
log_success "Created epic: $EPIC_EVENTSRC"
bd dep add "$EPIC_EVENTSRC" "$EPIC_DOMAIN"

TASK_MIGRATIONS=$(bd create "Create database migrations/ directory with schema.sql" \
    -p 0 --parent "$EPIC_EVENTSRC" 2>&1 | extract_id)
bd update "$TASK_MIGRATIONS" --description "Initialize migrations/ subdirectory with migrations/schema.sql containing SQLite DDL: events table (id, aggregate_type, aggregate_id, sequence, event_type, payload, metadata, created_at), unique constraint, indexes. Referenced by process-compose db-init."

TASK_ES_TRAIT=$(bd create "Create EventStore trait abstraction" \
    -p 0 --parent "$EPIC_EVENTSRC" 2>&1 | extract_id)
bd update "$TASK_ES_TRAIT" --description "Define async trait with append, query_all, query_since_sequence, query_aggregate methods using async_trait. Enables swapping implementations (SQLite now, Zenoh later) without changing application code."
bd dep add "$TASK_ES_TRAIT" "$TASK_DOMAIN_TYPES"

TASK_SQLITE_ES=$(bd create "Implement SQLite event store with sqlx" \
    -p 0 --parent "$EPIC_EVENTSRC" 2>&1 | extract_id)
bd update "$TASK_SQLITE_ES" --description "Create SqliteEventStore struct implementing EventStore trait with query_all, query_since_sequence, query_aggregate methods. Use sqlx compile-time query validation. Create events table with sequence, aggregate_type, aggregate_id, event_type, payload JSON columns. Append-only log foundation for CQRS.
Local refs: ~/projects/rust-workspace/sqlx"
bd dep add "$TASK_SQLITE_ES" "$TASK_ES_TRAIT"
bd dep add "$TASK_SQLITE_ES" "$TASK_MIGRATIONS"

TASK_SQLITE_CONFIG=$(bd create "Implement SQLite connection pooling and configuration" \
    -p 1 --parent "$EPIC_EVENTSRC" 2>&1 | extract_id)
bd update "$TASK_SQLITE_CONFIG" --description "Configure SqlitePool with PRAGMA settings for event sourcing: journal_mode=WAL, synchronous=FULL, cache_size=-64000 (64MB), temp_store=MEMORY. Optimizes for durability and read throughput on the event store workload.
Local refs: ~/projects/rust-workspace/sqlx"
bd dep add "$TASK_SQLITE_CONFIG" "$TASK_SQLITE_ES"

TASK_EVENT_BUS=$(bd create "Implement tokio broadcast event bus" \
    -p 0 --parent "$EPIC_EVENTSRC" 2>&1 | extract_id)
bd update "$TASK_EVENT_BUS" --description "Create EventBus wrapper around tokio::sync::broadcast::channel with Sender holding domain events. Implement publish() method returning Result and subscribe() method returning Receiver. Set default capacity to 256 events. Enables in-process fan-out to multiple subscribers.
Local refs: ~/projects/rust-workspace/tokio"
bd dep add "$TASK_EVENT_BUS" "$TASK_DOMAIN_TYPES"

TASK_PROJECTION_TRAIT=$(bd create "Create Projection trait for read models" \
    -p 0 --parent "$EPIC_EVENTSRC" 2>&1 | extract_id)
bd update "$TASK_PROJECTION_TRAIT" --description "Define async trait with rebuild(events), apply(state, event), to_sse_event(state, sequence) methods. Enables multiple projection types to independently subscribe to events and maintain their own read models."
bd dep add "$TASK_PROJECTION_TRAIT" "$TASK_DOMAIN_TYPES"

TASK_PROJECTION_MGR=$(bd create "Implement ProjectionManager with in-memory state" \
    -p 0 --parent "$EPIC_EVENTSRC" 2>&1 | extract_id)
bd update "$TASK_PROJECTION_MGR" --description "Create generic ProjectionManager<P: Projection> wrapping Arc<RwLock<P::State>>. Implement init() to replay all events from event store, spawn background task subscribing to broadcast for incremental updates, and query() to read current state.
Local refs: ~/projects/rust-workspace/tokio"
bd dep add "$TASK_PROJECTION_MGR" "$TASK_PROJECTION_TRAIT"
bd dep add "$TASK_PROJECTION_MGR" "$TASK_EVENT_BUS"
bd dep add "$TASK_PROJECTION_MGR" "$TASK_SQLITE_ES"

TASK_REDB_SESSION=$(bd create "Implement redb session store with ACID guarantees" \
    -p 1 --parent "$EPIC_EVENTSRC" 2>&1 | extract_id)
bd update "$TASK_REDB_SESSION" --description "Create SessionStore wrapper around redb::Database with get(session_id) and put(session_id, data) methods. Use bincode serialization for SessionData struct. Implement linear type semantics with WriteTransaction bracket pattern. Provides server-side session storage with ACID durability.
Local refs: ~/projects/rust-workspace/redb, ~/projects/rust-workspace/redb/docs/design.md"
bd dep add "$TASK_REDB_SESSION" "$TASK_DOMAIN_TYPES"

TASK_SESSION_EXTRACTOR=$(bd create "Create Session axum extractor" \
    -p 1 --parent "$EPIC_EVENTSRC" 2>&1 | extract_id)
bd update "$TASK_SESSION_EXTRACTOR" --description "Implement FromRequestParts for Session type extracting session_id from CookieJar. Load or initialize SessionData from SessionStore. Return Session struct with id and data fields for use in handlers.
Local refs: ~/projects/rust-workspace/axum"
bd dep add "$TASK_SESSION_EXTRACTOR" "$TASK_REDB_SESSION"

TASK_DUCKDB=$(bd create "Implement DuckDB analytics service" \
    -p 2 --parent "$EPIC_EVENTSRC" 2>&1 | extract_id)
bd update "$TASK_DUCKDB" --description "Create AnalyticsService wrapping duckdb::Connection using one-connection-per-task pattern. Implement query methods returning Vec of analytics results. Wrap blocking operations in tokio::task::block_in_place() for quick queries. Enables OLAP queries over event history.
Local refs: ~/projects/omicslake-workspace/duckdb-rs"
bd dep add "$TASK_DUCKDB" "$TASK_DOMAIN_TYPES"

###############################################################################
# EPIC 8: Presentation Layer
###############################################################################
log_info "Creating Epic 8: Presentation Layer..."

EPIC_PRESENTATION=$(create_issue "Presentation layer" -t epic -p 1)
log_success "Created epic: $EPIC_PRESENTATION"
bd dep add "$EPIC_PRESENTATION" "$EPIC_EVENTSRC"
bd dep add "$EPIC_PRESENTATION" "$EPIC_FRONTEND"

TASK_JUSTFILE=$(bd create "Add justfile with development and build tasks" \
    -p 0 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_JUSTFILE" --description "Create justfile at repository root with recipes: dev, dev-bg, gen-types, build-frontend, build-backend, build (full), test, fmt, lint, check, ci. Centralizes task orchestration following Rust conventions.
Local refs: ~/projects/rust-workspace/rust-nix-template/, ~/projects/nix-workspace/typescript-nix-template/justfile"

TASK_DEVSHELL=$(bd create "Create devShell module with tools and environment" \
    -p 0 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_DEVSHELL" --description "Implement nix/modules/devshell.nix defining default devShell with inputsFrom rust devShell and pre-commit hooks, plus packages: just, cargo-watch, pnpm, nodejs, process-compose, sqlite3, nixd, bacon. Complete development environment.
Local refs: ~/projects/rust-workspace/rust-nix-template/nix/modules/devshell.nix, ~/projects/nix-workspace/typescript-nix-template/modules/dev-shell.nix"

TASK_PRECOMMIT=$(bd create "Configure pre-commit hooks for code quality" \
    -p 1 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_PRECOMMIT" --description "Create nix/modules/pre-commit.nix with git hooks for rustfmt, clippy, prettier (frontend), and linters. Set up .pre-commit-config.yaml to integrate with devShell via git-hooks.nix flake module.
Local refs: ~/projects/rust-workspace/rust-nix-template/nix/modules/pre-commit.nix"
bd dep add "$TASK_PRECOMMIT" "$TASK_DEVSHELL"

TASK_APPSTATE=$(bd create "Define AppState struct with all dependencies" \
    -p 0 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_APPSTATE" --description "Create AppState struct holding Arc<EventStore>, Arc<SessionStore>, Arc<Projections>, broadcast::Sender<StoredEvent>, and optional debug-only reload channel. Implement AppState::new() to initialize all services and replay events to rebuild projections at startup.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/tokio"
bd dep add "$TASK_APPSTATE" "$TASK_SQLITE_ES"
bd dep add "$TASK_APPSTATE" "$TASK_REDB_SESSION"
bd dep add "$TASK_APPSTATE" "$TASK_PROJECTION_MGR"
bd dep add "$TASK_APPSTATE" "$TASK_EVENT_BUS"

TASK_SSE_FEED=$(bd create "Implement SSE feed endpoint with event replay" \
    -p 0 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_SSE_FEED" --description "Create async sse_feed(headers, state) -> Sse handler that extracts Last-Event-ID, subscribes to broadcast channel, replays events since that ID from event store, chains with live stream, and emits SSE events with id field set to sequence number. Implements reconnection recovery.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust, ~/projects/lakescope-workspace/datastar/sdk/ADR.md"
bd dep add "$TASK_SSE_FEED" "$TASK_APPSTATE"

TASK_CMD_HANDLERS=$(bd create "Implement command POST handlers" \
    -p 0 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_CMD_HANDLERS" --description "Create POST handlers that extract Command from ReadSignals extractor, call command handler (pure logic), append events to event store, broadcast to subscribers, and return 202 Accepted immediately WITHOUT waiting for SSE update. Implements CQRS write path.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust"
bd dep add "$TASK_CMD_HANDLERS" "$TASK_APPSTATE"
bd dep add "$TASK_CMD_HANDLERS" "$TASK_AGGREGATES"

TASK_QUERY_HANDLERS=$(bd create "Implement query GET handlers" \
    -p 0 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_QUERY_HANDLERS" --description "Create GET handlers that call query handler (reads from projections), render hypertext template, and return as HTML or JSON. No event persistence, just read path. Handlers use State extractor to access AppState containing projections.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/hypertext"
bd dep add "$TASK_QUERY_HANDLERS" "$TASK_APPSTATE"
bd dep add "$TASK_QUERY_HANDLERS" "$TASK_PROJECTION_MGR"

TASK_RENDERABLE_TRAIT=$(bd create "Implement RenderableToDatastar conversion trait" \
    -p 0 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_RENDERABLE_TRAIT" --description "Create extension trait for hypertext::Renderable with to_patch_elements(), append_to(selector), replace_inner(selector) methods that convert HTML to datastar-rust PatchElements. Bridges hypertext templates to Datastar SSE without manual boilerplate.
Local refs: ~/projects/rust-workspace/hypertext, ~/projects/rust-workspace/datastar-rust"

TASK_BASE_LAYOUT=$(bd create "Create base layout template with Datastar initialization" \
    -p 0 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_BASE_LAYOUT" --description "Implement base_layout() function using hypertext::maud! that renders html > head > body with conditional hotreload div (data-init for dev mode), CSS link to bundle.[hash].css, and JS script for datastar.js. Establishes HTML structure for all pages.
Local refs: ~/projects/rust-workspace/hypertext, ~/projects/lakescope-workspace/datastar-go-nats-template-northstar"
bd dep add "$TASK_BASE_LAYOUT" "$TASK_RENDERABLE_TRAIT"
bd dep add "$TASK_BASE_LAYOUT" "$TASK_RUST_EMBED"

TASK_COMPONENT_TEMPLATES=$(bd create "Implement component-level hypertext templates" \
    -p 1 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_COMPONENT_TEMPLATES" --description "Create reusable component functions (e.g., button, form_field, loading_spinner) returning impl Renderable. Components accept data and emit proper Datastar attributes (data-on:, data-show, data-bind). These compose into page templates.
Local refs: ~/projects/rust-workspace/hypertext"
bd dep add "$TASK_COMPONENT_TEMPLATES" "$TASK_BASE_LAYOUT"

TASK_ROUTER=$(bd create "Implement router composition with feature routes" \
    -p 0 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_ROUTER" --description "Create main Router that merges feature modules. Each feature provides route() -> Router<AppState> composing GET/POST/SSE handlers. Use Router::merge to combine features and apply State layer to inject AppState.
Local refs: ~/projects/rust-workspace/axum"
bd dep add "$TASK_ROUTER" "$TASK_SSE_FEED"
bd dep add "$TASK_ROUTER" "$TASK_CMD_HANDLERS"
bd dep add "$TASK_ROUTER" "$TASK_QUERY_HANDLERS"

TASK_MAIN=$(bd create "Wire all components together in main.rs" \
    -p 1 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_MAIN" --description "Create main.rs that initializes EventStore, SessionStore, Projections, EventBus, composes Router, and starts axum server on configured port. Handle graceful shutdown. Orchestration layer tying all services together.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/tokio"
bd dep add "$TASK_MAIN" "$TASK_ROUTER"

TASK_HOTRELOAD=$(bd create "Implement dev-only hotreload SSE endpoint" \
    -p 2 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_HOTRELOAD" --description "Create conditional compilation block (#[cfg(debug_assertions)]) with GET /hotreload SSE endpoint that broadcasts ExecuteScript(window.location.reload()) when triggered, plus POST /hotreload/trigger endpoint. Coordinates with cargo-watch for browser reload on build completion.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust"
bd dep add "$TASK_HOTRELOAD" "$TASK_SSE_FEED"

TASK_HEALTH=$(bd create "Implement health check endpoint for process-compose" \
    -p 2 --parent "$EPIC_PRESENTATION" 2>&1 | extract_id)
bd update "$TASK_HEALTH" --description "Create GET /health endpoint that returns 200 OK when server is ready. Used by process-compose readiness_probe to coordinate startup dependency ordering between db-init, backend, frontend, and hotreload processes.
Local refs: ~/projects/rust-workspace/axum"
bd dep add "$TASK_HEALTH" "$TASK_ROUTER"

###############################################################################
# EPIC 9: Example Application (Todo)
###############################################################################
log_info "Creating Epic 9: Example Application (Todo)..."

EPIC_EXAMPLE=$(create_issue "Example application (Todo)" -t epic -p 2)
log_success "Created epic: $EPIC_EXAMPLE"
bd dep add "$EPIC_EXAMPLE" "$EPIC_PRESENTATION"

TASK_TODO_DOMAIN=$(bd create "Define Todo domain model (aggregate, events, commands)" \
    -p 1 --parent "$EPIC_EXAMPLE" 2>&1 | extract_id)
bd update "$TASK_TODO_DOMAIN" --description "Create src/domain/ with TodoAggregate struct (id, text, completed, created_at, updated_at), TodoCreated/TodoCompleted/TodoDeleted event types, and AddTodoCommand/MarkTodoCommand/DeleteTodoCommand types. Demonstrates algebraic modeling with sum types (events) and product types (aggregates).
Local refs: ~/projects/lakescope-workspace/datastar-go-nats-template-northstar"
bd dep add "$TASK_TODO_DOMAIN" "$TASK_DOMAIN_TYPES"

TASK_TODO_PROJECTION=$(bd create "Implement TodoListProjection with in-memory rebuild" \
    -p 1 --parent "$EPIC_EXAMPLE" 2>&1 | extract_id)
bd update "$TASK_TODO_PROJECTION" --description "Create struct TodoListProjection(Vec<TodoItem>) implementing Projection trait. rebuild() method replays all TodoCreated/TodoCompleted/TodoDeleted events to reconstruct current state. apply() method handles incremental event updates. Demonstrates projection pattern.
Local refs: ~/projects/rust-workspace/datastar-rust-lince"
bd dep add "$TASK_TODO_PROJECTION" "$TASK_TODO_DOMAIN"
bd dep add "$TASK_TODO_PROJECTION" "$TASK_PROJECTION_MGR"

TASK_TODO_ADD=$(bd create "Implement add_todo handler (POST /add-todo)" \
    -p 1 --parent "$EPIC_EXAMPLE" 2>&1 | extract_id)
bd update "$TASK_TODO_ADD" --description "Create async handler accepting ReadSignals<AddTodoCommand> with text field. Validates non-empty, emits TodoCreated event, appends to event store, broadcasts to subscribers, returns 202. Frontend removes loading indicator via SSE update. Demonstrates write path with immediate response.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust"
bd dep add "$TASK_TODO_ADD" "$TASK_TODO_PROJECTION"
bd dep add "$TASK_TODO_ADD" "$TASK_CMD_HANDLERS"

TASK_TODO_MARK=$(bd create "Implement mark_todo handler (POST /mark-todo)" \
    -p 1 --parent "$EPIC_EXAMPLE" 2>&1 | extract_id)
bd update "$TASK_TODO_MARK" --description "Create async handler accepting ReadSignals<{id: Uuid}> that emits TodoCompleted event, appends to event store, broadcasts, returns 202. SSE updates todo item to show completed state via hypertext morphing.
Local refs: ~/projects/rust-workspace/axum"
bd dep add "$TASK_TODO_MARK" "$TASK_TODO_ADD"

TASK_TODO_DELETE=$(bd create "Implement delete_todo handler (POST /delete-todo)" \
    -p 1 --parent "$EPIC_EXAMPLE" 2>&1 | extract_id)
bd update "$TASK_TODO_DELETE" --description "Create async handler accepting ReadSignals<{id: Uuid}> that emits TodoDeleted event, appends to event store, broadcasts, returns 202. SSE morphs todo-list to remove deleted item or replaces entire list.
Local refs: ~/projects/rust-workspace/axum"
bd dep add "$TASK_TODO_DELETE" "$TASK_TODO_ADD"

TASK_TODO_SSE=$(bd create "Implement GET /todos SSE feed endpoint" \
    -p 1 --parent "$EPIC_EXAMPLE" 2>&1 | extract_id)
bd update "$TASK_TODO_SSE" --description "Create async handler returning Sse<impl Stream> that on initial connection sends TodoListProjection current state as PatchElements(todo_list_template(todos)), then streams incremental updates from broadcast channel. Implements Tao of Datastar principle 1 (backend is source of truth) with fat morph initial state.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/datastar-rust"
bd dep add "$TASK_TODO_SSE" "$TASK_TODO_PROJECTION"
bd dep add "$TASK_TODO_SSE" "$TASK_SSE_FEED"

TASK_TODO_TEMPLATES=$(bd create "Implement todo_list_template rendering function" \
    -p 1 --parent "$EPIC_EXAMPLE" 2>&1 | extract_id)
bd update "$TASK_TODO_TEMPLATES" --description "Create hypertext function fn todo_list_template(todos: &[TodoItem]) -> impl Renderable that renders ul#todo-list with li items, checkboxes with data-on:change, delete buttons with data-on:click, and add-todo form with input data-bind. Demonstrates complete Datastar integration for todo app.
Local refs: ~/projects/rust-workspace/hypertext, ~/projects/lakescope-workspace/datastar-go-nats-template-northstar"
bd dep add "$TASK_TODO_TEMPLATES" "$TASK_COMPONENT_TEMPLATES"

TASK_TODO_ROUTES=$(bd create "Implement todo example route mounting" \
    -p 1 --parent "$EPIC_EXAMPLE" 2>&1 | extract_id)
bd update "$TASK_TODO_ROUTES" --description "Create routes() function that mounts GET /todos, POST /add-todo, POST /mark-todo, POST /delete-todo, and GET /todos-feed endpoints. Wire state with TodoStore, EventStore, Projections, and event_bus. Mount under /api prefix in main Router.
Local refs: ~/projects/rust-workspace/axum"
bd dep add "$TASK_TODO_ROUTES" "$TASK_TODO_SSE"
bd dep add "$TASK_TODO_ROUTES" "$TASK_TODO_ADD"
bd dep add "$TASK_TODO_ROUTES" "$TASK_TODO_MARK"
bd dep add "$TASK_TODO_ROUTES" "$TASK_TODO_DELETE"
bd dep add "$TASK_TODO_ROUTES" "$TASK_TODO_TEMPLATES"

###############################################################################
# EPIC 10: Third-Party Library Integration
###############################################################################
log_info "Creating Epic 10: Third-Party Library Integration..."

EPIC_INTEGRATION=$(create_issue "Third-party library integration" -t epic -p 2)
log_success "Created epic: $EPIC_INTEGRATION"
bd dep add "$EPIC_INTEGRATION" "$EPIC_FRONTEND"

TASK_VEGA_WC=$(bd create "Implement VegaChart web component wrapper" \
    -p 2 --parent "$EPIC_INTEGRATION" 2>&1 | extract_id)
bd update "$TASK_VEGA_WC" --description "Create vanilla TypeScript web component (web-components/components/vega-chart.ts) that wraps vega-embed, stores Result and View instances, implements observedAttributes=['spec-url', 'data-url', 'signal-values'], and calls result?.finalize() on disconnect. Must use data-ignore-morph to prevent Datastar from morphing Vega's DOM.
Local refs: ~/projects/lakescope-workspace/vega-embed, ~/projects/lakescope-workspace/datastar-go-nats-template-northstar"
bd dep add "$TASK_VEGA_WC" "$TASK_WC_COMPONENTS"

TASK_SORTABLE_WC=$(bd create "Implement sortable-list web component wrapper" \
    -p 2 --parent "$EPIC_INTEGRATION" 2>&1 | extract_id)
bd update "$TASK_SORTABLE_WC" --description "Create web-components/components/sortable-list.ts implementing Pattern 1 thin wrapper around SortableJS library. Dispatches custom reorder event with detail containing oldIndex/newIndex. Integrates with Datastar via data-on:reorder.
Local refs: ~/projects/lakescope-workspace/datastar-go-nats-template-northstar"
bd dep add "$TASK_SORTABLE_WC" "$TASK_WC_COMPONENTS"

TASK_LUCIDE=$(bd create "Set up Lucide icon build-time inlining" \
    -p 2 --parent "$EPIC_INTEGRATION" 2>&1 | extract_id)
bd update "$TASK_LUCIDE" --description "Configure rolldown.config.ts to import lucide icons and inline SVG into bundle. Create icon helper function in hypertext templates for consistent icon usage. Provides zero-runtime icon system.
Local refs: ~/projects/lakescope-workspace/open-props-ui, ~/projects/rust-workspace/hypertext"
bd dep add "$TASK_LUCIDE" "$TASK_ROLLDOWN"

###############################################################################
# EPIC 11: Testing and Integration
###############################################################################
log_info "Creating Epic 11: Testing and Integration..."

EPIC_TESTING=$(create_issue "Testing and integration" -t epic -p 2)
log_success "Created epic: $EPIC_TESTING"
bd dep add "$EPIC_TESTING" "$EPIC_EXAMPLE"

TASK_ES_TESTS=$(bd create "Create event store integration tests" \
    -p 2 --parent "$EPIC_TESTING" 2>&1 | extract_id)
bd update "$TASK_ES_TESTS" --description "Write tests for SqliteEventStore: append returns monotonic sequences, query_all returns all events, query_since_sequence returns only newer events, index queries work correctly. Use temp SQLite database for isolation.
Local refs: ~/projects/rust-workspace/sqlx"
bd dep add "$TASK_ES_TESTS" "$TASK_SQLITE_ES"

TASK_PROJECTION_TESTS=$(bd create "Create projection tests" \
    -p 2 --parent "$EPIC_TESTING" 2>&1 | extract_id)
bd update "$TASK_PROJECTION_TESTS" --description "Write tests for ProjectionManager: rebuild from events produces correct state, apply increments state correctly, concurrent applies via RwLock don't lose updates. Mock EventStore and Projection trait implementations.
Local refs: ~/projects/rust-workspace/tokio"
bd dep add "$TASK_PROJECTION_TESTS" "$TASK_PROJECTION_MGR"

TASK_E2E_TESTS=$(bd create "Create end-to-end handler tests" \
    -p 2 --parent "$EPIC_TESTING" 2>&1 | extract_id)
bd update "$TASK_E2E_TESTS" --description "Write integration tests for complete command/query flow: POST command -> event appended -> broadcast sent -> projection updated -> SSE responds with new state. Use test utilities to initialize temporary AppState.
Local refs: ~/projects/rust-workspace/axum, ~/projects/rust-workspace/tokio"
bd dep add "$TASK_E2E_TESTS" "$TASK_MAIN"

###############################################################################
# EPIC 12: Documentation and Template
###############################################################################
log_info "Creating Epic 12: Documentation and Template..."

EPIC_DOCS=$(create_issue "Documentation and template" -t epic -p 3)
log_success "Created epic: $EPIC_DOCS"
bd dep add "$EPIC_DOCS" "$EPIC_TESTING"

TASK_BOOTSTRAP_DOC=$(bd create "Create BOOTSTRAP.md with complete setup instructions" \
    -p 2 --parent "$EPIC_DOCS" 2>&1 | extract_id)
bd update "$TASK_BOOTSTRAP_DOC" --description "Write BOOTSTRAP.md documenting: prerequisites (Nix, direnv), flake.nix structure overview, Nix modules organization, devShell contents, process-compose processes, development workflow, frontend/backend build separation. Include troubleshooting section."
bd dep add "$TASK_BOOTSTRAP_DOC" "$TASK_MAIN"

TASK_OMNIX_TEMPLATE=$(bd create "Create template parameters and conditional includes" \
    -p 2 --parent "$EPIC_DOCS" 2>&1 | extract_id)
bd update "$TASK_OMNIX_TEMPLATE" --description "Implement nix/modules/template.nix defining omnix template parameters: project-name, crate-name, github-ci (conditional .github/workflows), example-todo (conditional examples/), nix-template (conditional nix/modules/template.nix). Follow typescript-nix-template pattern.
Local refs: ~/projects/nix-workspace/typescript-nix-template/modules/template.nix"
bd dep add "$TASK_OMNIX_TEMPLATE" "$TASK_RUST_FLAKE"

TASK_OMNIX_TESTS=$(bd create "Define om CLI instantiation tests and metadata" \
    -p 3 --parent "$EPIC_DOCS" 2>&1 | extract_id)
bd update "$TASK_OMNIX_TESTS" --description "Add om.templates.ironstar definition with template description, parameters array, and integration tests validating: Cargo.toml generation, flake.nix presence, .github/workflows/ci.yml conditionally present, packages.default builds successfully.
Local refs: ~/projects/rust-workspace/rust-nix-template/nix/modules/template.nix"
bd dep add "$TASK_OMNIX_TESTS" "$TASK_OMNIX_TEMPLATE"

TASK_ENV_TEMPLATE=$(bd create "Create .env.development template file" \
    -p 2 --parent "$EPIC_DOCS" 2>&1 | extract_id)
bd update "$TASK_ENV_TEMPLATE" --description "Create template .env.development with DATABASE_URL=dev.db, LOG_LEVEL=debug, SERVER_PORT=3000, RELOAD_ENABLED=true, STATIC_DIR=static/dist. Document in README that users should copy to .env for local development. Add .env* to .gitignore."
bd dep add "$TASK_ENV_TEMPLATE" "$TASK_SQLITE_ES"

TASK_TRACING=$(bd create "Add structured logging with tracing" \
    -p 2 --parent "$EPIC_DOCS" 2>&1 | extract_id)
bd update "$TASK_TRACING" --description "Integrate tracing and tracing-subscriber crates for structured logging of events appended, handlers executed, projection updates, and errors. Use span context to correlate logs across request lifecycle."
bd dep add "$TASK_TRACING" "$TASK_MAIN"

###############################################################################
# Summary
###############################################################################
echo ""
log_success "Work items creation complete!"
echo ""
bd stats
echo ""
log_info "View ready queue: bd ready"
log_info "View all epics: bd list --type epic"
log_info "View dependency tree: bd dep tree <epic-id>"
