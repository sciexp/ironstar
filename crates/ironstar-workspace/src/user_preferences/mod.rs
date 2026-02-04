//! UserPreferences aggregate for user-scoped personal settings.
//!
//! Manages per-user settings that follow the user across all workspaces:
//! theme, locale, and arbitrary UI state as JSON.
//! This is distinct from WorkspacePreferences (workspace-scoped, shared
//! across all users in a workspace).
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
//!       SetTheme          SetLocale        UpdateUiState
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
//! `user_{user_id}/preferences` -- per-user singleton.
//!
//! # Idempotency
//!
//! All operations after initialization are idempotent (setting the same
//! value returns `Ok(vec![])` with no events emitted).
//!
//! # Module organization
//!
//! - [`commands`]: UserPreferencesCommand enum
//! - [`decider`]: user_preferences_decider() factory with pure decide/evolve
//! - [`errors`]: UserPreferencesError with UUID tracking
//! - [`events`]: UserPreferencesEvent enum
//! - [`state`]: UserPreferencesState enum (NotInitialized | Initialized)
//! - [`values`]: Value objects (PreferencesId, Theme, Locale, UiState)

pub mod commands;
pub mod decider;
pub mod errors;
pub mod events;
pub mod state;
pub mod values;

pub use commands::UserPreferencesCommand;
pub use decider::{UserPreferencesDecider, user_preferences_decider};
pub use errors::{UserPreferencesError, UserPreferencesErrorKind};
pub use events::UserPreferencesEvent;
pub use state::UserPreferencesState;
pub use values::{LOCALE_MAX_LENGTH, Locale, PreferencesId, Theme, UiState};
