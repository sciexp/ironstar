//! Pure WorkspacePreferences Decider implementing fmodel-rust patterns.
//!
//! The Decider embodies the state machine from
//! `spec/Workspace/WorkspacePreferences.idr`. It is a pure function with
//! no side effects: all I/O (timestamps, catalog validation, JSON validation)
//! happens at boundaries.
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
//! # Idempotency
//!
//! All operations after initialization are idempotent:
//! - SetDefaultCatalog with same URI returns `Ok(vec![])`
//! - ClearDefaultCatalog when already cleared returns `Ok(vec![])`
//! - UpdateLayoutDefaults with same JSON returns `Ok(vec![])`

use fmodel_rust::decider::Decider;

use super::commands::WorkspacePreferencesCommand;
use super::errors::WorkspacePreferencesError;
use super::events::WorkspacePreferencesEvent;
use super::state::WorkspacePreferencesState;
use super::values::LayoutDefaults;

/// Type alias for the WorkspacePreferences Decider.
pub type WorkspacePreferencesDecider<'a> = Decider<
    'a,
    WorkspacePreferencesCommand,
    WorkspacePreferencesState,
    WorkspacePreferencesEvent,
    WorkspacePreferencesError,
>;

/// Factory function creating a pure WorkspacePreferences Decider.
///
/// Translates the specification from `spec/Workspace/WorkspacePreferences.idr`
/// into Rust, preserving the state machine transitions and idempotency invariants.
pub fn workspace_preferences_decider<'a>() -> WorkspacePreferencesDecider<'a> {
    Decider {
        decide: Box::new(decide),
        evolve: Box::new(evolve),
        initial_state: Box::new(WorkspacePreferencesState::default),
    }
}

/// Pure decide function: (Command, State) -> Result<Vec<Event>, Error>
fn decide(
    command: &WorkspacePreferencesCommand,
    state: &WorkspacePreferencesState,
) -> Result<Vec<WorkspacePreferencesEvent>, WorkspacePreferencesError> {
    match (command, state) {
        // Initialize: NotInitialized → Initialized
        (
            WorkspacePreferencesCommand::InitializeWorkspacePreferences {
                workspace_id,
                initialized_at,
            },
            WorkspacePreferencesState::NotInitialized,
        ) => Ok(vec![
            WorkspacePreferencesEvent::WorkspacePreferencesInitialized {
                workspace_id: *workspace_id,
                initialized_at: *initialized_at,
            },
        ]),

        // Initialize when already initialized
        (
            WorkspacePreferencesCommand::InitializeWorkspacePreferences { .. },
            WorkspacePreferencesState::Initialized { .. },
        ) => Err(WorkspacePreferencesError::already_initialized()),

        // SetDefaultCatalog: Initialized → Initialized (idempotent if same URI)
        (
            WorkspacePreferencesCommand::SetDefaultCatalog {
                workspace_id,
                catalog_uri,
                set_at,
            },
            WorkspacePreferencesState::Initialized {
                default_catalog, ..
            },
        ) => {
            if default_catalog.as_ref() == Some(catalog_uri) {
                return Ok(vec![]);
            }

            Ok(vec![WorkspacePreferencesEvent::DefaultCatalogSet {
                workspace_id: *workspace_id,
                catalog_uri: catalog_uri.clone(),
                set_at: *set_at,
            }])
        }

        // SetDefaultCatalog when not initialized
        (
            WorkspacePreferencesCommand::SetDefaultCatalog { .. },
            WorkspacePreferencesState::NotInitialized,
        ) => Err(WorkspacePreferencesError::not_initialized()),

        // ClearDefaultCatalog: Initialized → Initialized (idempotent if already cleared)
        (
            WorkspacePreferencesCommand::ClearDefaultCatalog {
                workspace_id,
                cleared_at,
            },
            WorkspacePreferencesState::Initialized {
                default_catalog, ..
            },
        ) => {
            if default_catalog.is_none() {
                return Ok(vec![]);
            }

            Ok(vec![WorkspacePreferencesEvent::DefaultCatalogCleared {
                workspace_id: *workspace_id,
                cleared_at: *cleared_at,
            }])
        }

        // ClearDefaultCatalog when not initialized
        (
            WorkspacePreferencesCommand::ClearDefaultCatalog { .. },
            WorkspacePreferencesState::NotInitialized,
        ) => Err(WorkspacePreferencesError::not_initialized()),

        // UpdateLayoutDefaults: Initialized → Initialized (idempotent if same defaults)
        (
            WorkspacePreferencesCommand::UpdateLayoutDefaults {
                workspace_id,
                layout_defaults,
                updated_at,
            },
            WorkspacePreferencesState::Initialized {
                layout_defaults: current_defaults,
                ..
            },
        ) => {
            if current_defaults == layout_defaults {
                return Ok(vec![]);
            }

            Ok(vec![WorkspacePreferencesEvent::LayoutDefaultsUpdated {
                workspace_id: *workspace_id,
                layout_defaults: layout_defaults.clone(),
                updated_at: *updated_at,
            }])
        }

        // UpdateLayoutDefaults when not initialized
        (
            WorkspacePreferencesCommand::UpdateLayoutDefaults { .. },
            WorkspacePreferencesState::NotInitialized,
        ) => Err(WorkspacePreferencesError::not_initialized()),
    }
}

/// Pure evolve function: (State, Event) -> State
fn evolve(
    state: &WorkspacePreferencesState,
    event: &WorkspacePreferencesEvent,
) -> WorkspacePreferencesState {
    match event {
        WorkspacePreferencesEvent::WorkspacePreferencesInitialized { workspace_id, .. } => {
            WorkspacePreferencesState::Initialized {
                workspace_id: *workspace_id,
                default_catalog: None,
                layout_defaults: LayoutDefaults::default(),
            }
        }

        WorkspacePreferencesEvent::DefaultCatalogSet { catalog_uri, .. } => match state {
            WorkspacePreferencesState::Initialized {
                workspace_id,
                layout_defaults,
                ..
            } => WorkspacePreferencesState::Initialized {
                workspace_id: *workspace_id,
                default_catalog: Some(catalog_uri.clone()),
                layout_defaults: layout_defaults.clone(),
            },
            WorkspacePreferencesState::NotInitialized => state.clone(),
        },

        WorkspacePreferencesEvent::DefaultCatalogCleared { .. } => match state {
            WorkspacePreferencesState::Initialized {
                workspace_id,
                layout_defaults,
                ..
            } => WorkspacePreferencesState::Initialized {
                workspace_id: *workspace_id,
                default_catalog: None,
                layout_defaults: layout_defaults.clone(),
            },
            WorkspacePreferencesState::NotInitialized => state.clone(),
        },

        WorkspacePreferencesEvent::LayoutDefaultsUpdated {
            layout_defaults, ..
        } => match state {
            WorkspacePreferencesState::Initialized {
                workspace_id,
                default_catalog,
                ..
            } => WorkspacePreferencesState::Initialized {
                workspace_id: *workspace_id,
                default_catalog: default_catalog.clone(),
                layout_defaults: layout_defaults.clone(),
            },
            WorkspacePreferencesState::NotInitialized => state.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use fmodel_rust::specification::DeciderTestSpecification;

    use crate::domain::workspace::WorkspaceId;
    use super::super::values::CatalogUri;

    fn sample_workspace_id() -> WorkspaceId {
        WorkspaceId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn sample_catalog_uri() -> CatalogUri {
        CatalogUri::new("ducklake:hf://datasets/sciexp").unwrap()
    }

    fn initialized_event() -> WorkspacePreferencesEvent {
        WorkspacePreferencesEvent::WorkspacePreferencesInitialized {
            workspace_id: sample_workspace_id(),
            initialized_at: sample_time(),
        }
    }

    // --- Initialize transitions ---

    #[test]
    fn initialize_from_not_initialized_succeeds() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![])
            .when(WorkspacePreferencesCommand::InitializeWorkspacePreferences {
                workspace_id: ws_id,
                initialized_at: ts,
            })
            .then(vec![
                WorkspacePreferencesEvent::WorkspacePreferencesInitialized {
                    workspace_id: ws_id,
                    initialized_at: ts,
                },
            ]);
    }

    #[test]
    fn initialize_when_already_initialized_fails() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![initialized_event()])
            .when(WorkspacePreferencesCommand::InitializeWorkspacePreferences {
                workspace_id: ws_id,
                initialized_at: ts,
            })
            .then_error(WorkspacePreferencesError::already_initialized());
    }

    // --- SetDefaultCatalog transitions ---

    #[test]
    fn set_default_catalog_succeeds() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();
        let uri = sample_catalog_uri();

        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![initialized_event()])
            .when(WorkspacePreferencesCommand::SetDefaultCatalog {
                workspace_id: ws_id,
                catalog_uri: uri.clone(),
                set_at: ts,
            })
            .then(vec![WorkspacePreferencesEvent::DefaultCatalogSet {
                workspace_id: ws_id,
                catalog_uri: uri,
                set_at: ts,
            }]);
    }

    #[test]
    fn set_default_catalog_same_uri_is_idempotent() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();
        let uri = sample_catalog_uri();

        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![
                initialized_event(),
                WorkspacePreferencesEvent::DefaultCatalogSet {
                    workspace_id: ws_id,
                    catalog_uri: uri.clone(),
                    set_at: ts,
                },
            ])
            .when(WorkspacePreferencesCommand::SetDefaultCatalog {
                workspace_id: ws_id,
                catalog_uri: uri,
                set_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn set_default_catalog_not_initialized_fails() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![])
            .when(WorkspacePreferencesCommand::SetDefaultCatalog {
                workspace_id: ws_id,
                catalog_uri: sample_catalog_uri(),
                set_at: ts,
            })
            .then_error(WorkspacePreferencesError::not_initialized());
    }

    // --- ClearDefaultCatalog transitions ---

    #[test]
    fn clear_default_catalog_succeeds() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();
        let uri = sample_catalog_uri();

        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![
                initialized_event(),
                WorkspacePreferencesEvent::DefaultCatalogSet {
                    workspace_id: ws_id,
                    catalog_uri: uri,
                    set_at: ts,
                },
            ])
            .when(WorkspacePreferencesCommand::ClearDefaultCatalog {
                workspace_id: ws_id,
                cleared_at: ts,
            })
            .then(vec![WorkspacePreferencesEvent::DefaultCatalogCleared {
                workspace_id: ws_id,
                cleared_at: ts,
            }]);
    }

    #[test]
    fn clear_default_catalog_when_none_is_idempotent() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![initialized_event()])
            .when(WorkspacePreferencesCommand::ClearDefaultCatalog {
                workspace_id: ws_id,
                cleared_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn clear_default_catalog_not_initialized_fails() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![])
            .when(WorkspacePreferencesCommand::ClearDefaultCatalog {
                workspace_id: ws_id,
                cleared_at: ts,
            })
            .then_error(WorkspacePreferencesError::not_initialized());
    }

    // --- UpdateLayoutDefaults transitions ---

    #[test]
    fn update_layout_defaults_succeeds() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();
        let ld = LayoutDefaults::new(r#"{"columns": 3}"#);

        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![initialized_event()])
            .when(WorkspacePreferencesCommand::UpdateLayoutDefaults {
                workspace_id: ws_id,
                layout_defaults: ld.clone(),
                updated_at: ts,
            })
            .then(vec![WorkspacePreferencesEvent::LayoutDefaultsUpdated {
                workspace_id: ws_id,
                layout_defaults: ld,
                updated_at: ts,
            }]);
    }

    #[test]
    fn update_layout_defaults_same_value_is_idempotent() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        // Default is "{}" and we send "{}" again
        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![initialized_event()])
            .when(WorkspacePreferencesCommand::UpdateLayoutDefaults {
                workspace_id: ws_id,
                layout_defaults: LayoutDefaults::default(),
                updated_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn update_layout_defaults_not_initialized_fails() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(workspace_preferences_decider())
            .given(vec![])
            .when(WorkspacePreferencesCommand::UpdateLayoutDefaults {
                workspace_id: ws_id,
                layout_defaults: LayoutDefaults::new(r#"{"foo": 1}"#),
                updated_at: ts,
            })
            .then_error(WorkspacePreferencesError::not_initialized());
    }

    // --- Full lifecycle ---

    #[test]
    fn full_lifecycle() {
        let ws_id = sample_workspace_id();
        let ts = sample_time();

        // Initialize
        let events = decide(
            &WorkspacePreferencesCommand::InitializeWorkspacePreferences {
                workspace_id: ws_id,
                initialized_at: ts,
            },
            &WorkspacePreferencesState::default(),
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&WorkspacePreferencesState::default(), &events[0]);
        assert!(state.is_initialized());
        assert!(state.default_catalog().is_none());
        assert_eq!(state.layout_defaults().unwrap().as_str(), "{}");

        // Set catalog
        let uri = sample_catalog_uri();
        let events = decide(
            &WorkspacePreferencesCommand::SetDefaultCatalog {
                workspace_id: ws_id,
                catalog_uri: uri.clone(),
                set_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.default_catalog().unwrap(), &uri);

        // Idempotent: set same catalog
        let events = decide(
            &WorkspacePreferencesCommand::SetDefaultCatalog {
                workspace_id: ws_id,
                catalog_uri: uri,
                set_at: ts,
            },
            &state,
        )
        .unwrap();
        assert!(events.is_empty());

        // Update layout defaults
        let ld = LayoutDefaults::new(r#"{"columns": 4, "density": "compact"}"#);
        let events = decide(
            &WorkspacePreferencesCommand::UpdateLayoutDefaults {
                workspace_id: ws_id,
                layout_defaults: ld.clone(),
                updated_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.layout_defaults().unwrap(), &ld);

        // Clear catalog
        let events = decide(
            &WorkspacePreferencesCommand::ClearDefaultCatalog {
                workspace_id: ws_id,
                cleared_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert!(state.default_catalog().is_none());

        // Idempotent: clear again
        let events = decide(
            &WorkspacePreferencesCommand::ClearDefaultCatalog {
                workspace_id: ws_id,
                cleared_at: ts,
            },
            &state,
        )
        .unwrap();
        assert!(events.is_empty());
    }
}
