//! WorkspacePreferences aggregate for workspace-scoped settings.
//!
//! Manages per-workspace settings: default catalog URI and layout defaults.
//! This is distinct from UserPreferences (user-scoped, follows user across
//! all workspaces).
//!
//! # State Machine
//!
//! ```text
//!                    ┌──────────────────┐
//!  Initialize ──────►│   Initialized    │
//!                    └────────┬─────────┘
//!                             │
//!          ┌──────────────────┼──────────────────┐
//!          │                  │                   │
//!   SetDefaultCatalog  ClearDefaultCatalog  UpdateLayoutDefaults
//!          │                  │                   │
//!          └──────────────────┴──────────────────-┘
//!                             │
//!                             ▼
//!                    ┌──────────────────┐
//!                    │   Initialized    │ (updated fields)
//!                    └──────────────────┘
//! ```
//!
//! # Aggregate ID pattern
//!
//! `workspace_{workspace_id}/preferences` — per-workspace singleton.
//!
//! # Idempotency
//!
//! All operations after initialization are idempotent (setting the same
//! value returns `Ok(vec![])` with no events emitted).
//!
//! # Module Organization
//!
//! - [`commands`]: WorkspacePreferencesCommand enum
//! - [`decider`]: workspace_preferences_decider() factory with pure decide/evolve
//! - [`errors`]: WorkspacePreferencesError with UUID tracking
//! - [`events`]: WorkspacePreferencesEvent enum
//! - [`state`]: WorkspacePreferencesState enum (NotInitialized | Initialized)
//! - [`values`]: Value objects (CatalogUri, LayoutDefaults)

pub mod commands;
pub mod decider;
pub mod errors;
pub mod events;
pub mod state;
pub mod values;

pub use commands::WorkspacePreferencesCommand;
pub use decider::{WorkspacePreferencesDecider, workspace_preferences_decider};
pub use errors::{WorkspacePreferencesError, WorkspacePreferencesErrorKind};
pub use events::WorkspacePreferencesEvent;
pub use state::WorkspacePreferencesState;
pub use values::{CATALOG_URI_MAX_LENGTH, CatalogUri, LayoutDefaults};
