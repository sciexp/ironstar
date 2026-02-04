//! Dashboard aggregate for persistent UI layout configuration.
//!
//! Manages chart placements, tab organization, and grid positioning
//! within a workspace. Multiple dashboards per workspace are supported,
//! each identified by a unique `DashboardId`.
//!
//! # State Machine
//!
//! ```text
//!                     ┌───────────────────┐
//!  CreateDashboard ──►│  DashboardExists  │
//!                     └────────┬──────────┘
//!                              │
//!          ┌───────────────────┼───────────────────┐
//!          │         │         │         │          │
//!       Rename   AddChart  RemoveChart  AddTab  RemoveTab  MoveChartToTab
//!          │         │         │         │          │
//!          └───────────────────┴───────────────────-┘
//!                              │
//!                              ▼
//!                     ┌───────────────────┐
//!                     │  DashboardExists  │ (updated fields)
//!                     └───────────────────┘
//! ```
//!
//! # Aggregate ID pattern
//!
//! `dashboard_{dashboard_id}` — per-dashboard instance.
//!
//! # Idempotency
//!
//! Most operations are idempotent: renaming to the same name, adding a
//! chart that already exists, or removing a chart that does not exist
//! all return `Ok(vec![])` with no events emitted.
//!
//! # Module Organization
//!
//! - [`commands`]: DashboardCommand enum
//! - [`decider`]: dashboard_decider() factory with pure decide/evolve
//! - [`errors`]: DashboardError with UUID tracking
//! - [`events`]: DashboardEvent enum
//! - [`state`]: DashboardState enum (NoDashboard | DashboardExists)
//! - [`values`]: Value objects (DashboardId, TabId, ChartId, etc.)

pub mod commands;
pub mod decider;
pub mod errors;
pub mod events;
pub mod state;
pub mod values;

pub use commands::DashboardCommand;
pub use decider::{DashboardDecider, dashboard_decider};
pub use errors::{DashboardError, DashboardErrorKind};
pub use events::DashboardEvent;
pub use state::DashboardState;
pub use values::{
    ChartDefinitionRef, ChartId, ChartPlacement, DashboardId, GridPosition, TabId, TabInfo,
};
