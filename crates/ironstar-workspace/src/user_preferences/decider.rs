//! Pure UserPreferences Decider implementing fmodel-rust patterns.
//!
//! The Decider embodies the state machine from
//! `spec/Workspace/UserPreferences.idr`. It is a pure function with
//! no side effects: all I/O (timestamps, locale validation) happens at
//! boundaries.
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
//! # Idempotency
//!
//! All operations after initialization are idempotent:
//! - SetTheme with same theme returns `Ok(vec![])`
//! - SetLocale with same locale returns `Ok(vec![])`
//! - UpdateUiState with same state returns `Ok(vec![])`

use ironstar_core::Decider;
use tracing::instrument;

use super::commands::UserPreferencesCommand;
use super::errors::UserPreferencesError;
use super::events::UserPreferencesEvent;
use super::state::UserPreferencesState;
use super::values::{Locale, Theme, UiState};

/// Type alias for the UserPreferences Decider.
pub type UserPreferencesDecider<'a> = Decider<
    'a,
    UserPreferencesCommand,
    UserPreferencesState,
    UserPreferencesEvent,
    UserPreferencesError,
>;

/// Factory function creating a pure UserPreferences Decider.
///
/// Translates the specification from `spec/Workspace/UserPreferences.idr`
/// into Rust, preserving the state machine transitions and idempotency invariants.
pub fn user_preferences_decider<'a>() -> UserPreferencesDecider<'a> {
    Decider {
        decide: Box::new(decide),
        evolve: Box::new(evolve),
        initial_state: Box::new(UserPreferencesState::default),
    }
}

/// Pure decide function: (Command, State) -> Result<Vec<Event>, Error>
#[instrument(
    name = "decider.user_preferences.decide",
    skip_all,
    fields(
        command_type = command.command_type(),
        aggregate_type = "UserPreferences",
    )
)]
fn decide(
    command: &UserPreferencesCommand,
    state: &UserPreferencesState,
) -> Result<Vec<UserPreferencesEvent>, UserPreferencesError> {
    let result = match (command, state) {
        // Initialize: NotInitialized -> Initialized
        (
            UserPreferencesCommand::InitializePreferences {
                preferences_id,
                user_id,
                initialized_at,
            },
            UserPreferencesState::NotInitialized,
        ) => Ok(vec![UserPreferencesEvent::PreferencesInitialized {
            preferences_id: *preferences_id,
            user_id: *user_id,
            initialized_at: *initialized_at,
        }]),

        // Initialize when already initialized
        (
            UserPreferencesCommand::InitializePreferences { .. },
            UserPreferencesState::Initialized { .. },
        ) => Err(UserPreferencesError::already_initialized()),

        // SetTheme: Initialized -> Initialized (idempotent if same theme)
        (
            UserPreferencesCommand::SetTheme {
                user_id,
                theme,
                set_at,
            },
            UserPreferencesState::Initialized {
                theme: current_theme,
                ..
            },
        ) => {
            if current_theme == theme {
                return Ok(vec![]);
            }

            Ok(vec![UserPreferencesEvent::ThemeSet {
                user_id: *user_id,
                theme: *theme,
                set_at: *set_at,
            }])
        }

        // SetTheme when not initialized
        (UserPreferencesCommand::SetTheme { .. }, UserPreferencesState::NotInitialized) => {
            Err(UserPreferencesError::not_initialized())
        }

        // SetLocale: Initialized -> Initialized (idempotent if same locale)
        (
            UserPreferencesCommand::SetLocale {
                user_id,
                locale,
                set_at,
            },
            UserPreferencesState::Initialized {
                locale: current_locale,
                ..
            },
        ) => {
            if current_locale == locale {
                return Ok(vec![]);
            }

            Ok(vec![UserPreferencesEvent::LocaleSet {
                user_id: *user_id,
                locale: locale.clone(),
                set_at: *set_at,
            }])
        }

        // SetLocale when not initialized
        (UserPreferencesCommand::SetLocale { .. }, UserPreferencesState::NotInitialized) => {
            Err(UserPreferencesError::not_initialized())
        }

        // UpdateUiState: Initialized -> Initialized (idempotent if same state)
        (
            UserPreferencesCommand::UpdateUiState {
                user_id,
                ui_state,
                updated_at,
            },
            UserPreferencesState::Initialized {
                ui_state: current_ui_state,
                ..
            },
        ) => {
            if current_ui_state == ui_state {
                return Ok(vec![]);
            }

            Ok(vec![UserPreferencesEvent::UiStateUpdated {
                user_id: *user_id,
                ui_state: ui_state.clone(),
                updated_at: *updated_at,
            }])
        }

        // UpdateUiState when not initialized
        (UserPreferencesCommand::UpdateUiState { .. }, UserPreferencesState::NotInitialized) => {
            Err(UserPreferencesError::not_initialized())
        }
    };
    if let Ok(ref events) = result {
        tracing::debug!(event_count = events.len(), "decision complete");
    }
    result
}

/// Pure evolve function: (State, Event) -> State
#[instrument(
    name = "decider.user_preferences.evolve",
    level = "trace",
    skip_all,
    fields(aggregate_type = "UserPreferences")
)]
fn evolve(state: &UserPreferencesState, event: &UserPreferencesEvent) -> UserPreferencesState {
    match event {
        UserPreferencesEvent::PreferencesInitialized {
            preferences_id,
            user_id,
            ..
        } => UserPreferencesState::Initialized {
            preferences_id: *preferences_id,
            user_id: *user_id,
            theme: Theme::default(),
            locale: Locale::default(),
            ui_state: UiState::default(),
        },

        UserPreferencesEvent::ThemeSet { theme, .. } => match state {
            UserPreferencesState::Initialized {
                preferences_id,
                user_id,
                locale,
                ui_state,
                ..
            } => UserPreferencesState::Initialized {
                preferences_id: *preferences_id,
                user_id: *user_id,
                theme: *theme,
                locale: locale.clone(),
                ui_state: ui_state.clone(),
            },
            UserPreferencesState::NotInitialized => state.clone(),
        },

        UserPreferencesEvent::LocaleSet { locale, .. } => match state {
            UserPreferencesState::Initialized {
                preferences_id,
                user_id,
                theme,
                ui_state,
                ..
            } => UserPreferencesState::Initialized {
                preferences_id: *preferences_id,
                user_id: *user_id,
                theme: *theme,
                locale: locale.clone(),
                ui_state: ui_state.clone(),
            },
            UserPreferencesState::NotInitialized => state.clone(),
        },

        UserPreferencesEvent::UiStateUpdated { ui_state, .. } => match state {
            UserPreferencesState::Initialized {
                preferences_id,
                user_id,
                theme,
                locale,
                ..
            } => UserPreferencesState::Initialized {
                preferences_id: *preferences_id,
                user_id: *user_id,
                theme: *theme,
                locale: locale.clone(),
                ui_state: ui_state.clone(),
            },
            UserPreferencesState::NotInitialized => state.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use ironstar_core::DeciderTestSpecification;

    use super::super::values::PreferencesId;
    use ironstar_shared_kernel::UserId;

    fn sample_user_id() -> UserId {
        UserId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_pref_id() -> PreferencesId {
        PreferencesId::from_uuid(uuid::Uuid::nil())
    }

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn initialized_event() -> UserPreferencesEvent {
        UserPreferencesEvent::PreferencesInitialized {
            preferences_id: sample_pref_id(),
            user_id: sample_user_id(),
            initialized_at: sample_time(),
        }
    }

    // --- Initialize transitions ---

    #[test]
    fn initialize_from_not_initialized_succeeds() {
        let user_id = sample_user_id();
        let pref_id = sample_pref_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![])
            .when(UserPreferencesCommand::InitializePreferences {
                preferences_id: pref_id,
                user_id,
                initialized_at: ts,
            })
            .then(vec![UserPreferencesEvent::PreferencesInitialized {
                preferences_id: pref_id,
                user_id,
                initialized_at: ts,
            }]);
    }

    #[test]
    fn initialize_when_already_initialized_fails() {
        let user_id = sample_user_id();
        let pref_id = sample_pref_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![initialized_event()])
            .when(UserPreferencesCommand::InitializePreferences {
                preferences_id: pref_id,
                user_id,
                initialized_at: ts,
            })
            .then_error(UserPreferencesError::already_initialized());
    }

    // --- SetTheme transitions ---

    #[test]
    fn set_theme_succeeds() {
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![initialized_event()])
            .when(UserPreferencesCommand::SetTheme {
                user_id,
                theme: Theme::Dark,
                set_at: ts,
            })
            .then(vec![UserPreferencesEvent::ThemeSet {
                user_id,
                theme: Theme::Dark,
                set_at: ts,
            }]);
    }

    #[test]
    fn set_theme_same_value_is_idempotent() {
        let user_id = sample_user_id();
        let ts = sample_time();

        // Default theme is System, so setting System again is idempotent
        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![initialized_event()])
            .when(UserPreferencesCommand::SetTheme {
                user_id,
                theme: Theme::System,
                set_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn set_theme_not_initialized_fails() {
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![])
            .when(UserPreferencesCommand::SetTheme {
                user_id,
                theme: Theme::Dark,
                set_at: ts,
            })
            .then_error(UserPreferencesError::not_initialized());
    }

    // --- SetLocale transitions ---

    #[test]
    fn set_locale_succeeds() {
        let user_id = sample_user_id();
        let ts = sample_time();
        let locale = Locale::new("ja-JP").unwrap();

        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![initialized_event()])
            .when(UserPreferencesCommand::SetLocale {
                user_id,
                locale: locale.clone(),
                set_at: ts,
            })
            .then(vec![UserPreferencesEvent::LocaleSet {
                user_id,
                locale,
                set_at: ts,
            }]);
    }

    #[test]
    fn set_locale_same_value_is_idempotent() {
        let user_id = sample_user_id();
        let ts = sample_time();

        // Default locale is en-US, so setting en-US again is idempotent
        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![initialized_event()])
            .when(UserPreferencesCommand::SetLocale {
                user_id,
                locale: Locale::default(),
                set_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn set_locale_not_initialized_fails() {
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![])
            .when(UserPreferencesCommand::SetLocale {
                user_id,
                locale: Locale::new("de-DE").unwrap(),
                set_at: ts,
            })
            .then_error(UserPreferencesError::not_initialized());
    }

    // --- UpdateUiState transitions ---

    #[test]
    fn update_ui_state_succeeds() {
        let user_id = sample_user_id();
        let ts = sample_time();
        let ui_state = UiState::new(r#"{"sidebar": "collapsed"}"#);

        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![initialized_event()])
            .when(UserPreferencesCommand::UpdateUiState {
                user_id,
                ui_state: ui_state.clone(),
                updated_at: ts,
            })
            .then(vec![UserPreferencesEvent::UiStateUpdated {
                user_id,
                ui_state,
                updated_at: ts,
            }]);
    }

    #[test]
    fn update_ui_state_same_value_is_idempotent() {
        let user_id = sample_user_id();
        let ts = sample_time();

        // Default UI state is "{}", so setting "{}" again is idempotent
        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![initialized_event()])
            .when(UserPreferencesCommand::UpdateUiState {
                user_id,
                ui_state: UiState::default(),
                updated_at: ts,
            })
            .then(vec![]);
    }

    #[test]
    fn update_ui_state_not_initialized_fails() {
        let user_id = sample_user_id();
        let ts = sample_time();

        DeciderTestSpecification::default()
            .for_decider(user_preferences_decider())
            .given(vec![])
            .when(UserPreferencesCommand::UpdateUiState {
                user_id,
                ui_state: UiState::new(r#"{"foo": 1}"#),
                updated_at: ts,
            })
            .then_error(UserPreferencesError::not_initialized());
    }

    // --- Full lifecycle ---

    #[test]
    fn full_lifecycle() {
        let user_id = sample_user_id();
        let pref_id = sample_pref_id();
        let ts = sample_time();

        // Initialize
        let events = decide(
            &UserPreferencesCommand::InitializePreferences {
                preferences_id: pref_id,
                user_id,
                initialized_at: ts,
            },
            &UserPreferencesState::default(),
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&UserPreferencesState::default(), &events[0]);
        assert!(state.is_initialized());
        assert_eq!(state.theme(), Some(&Theme::System));
        assert_eq!(state.locale().unwrap().as_str(), "en-US");
        assert_eq!(state.ui_state().unwrap().as_str(), "{}");

        // Set theme
        let events = decide(
            &UserPreferencesCommand::SetTheme {
                user_id,
                theme: Theme::Dark,
                set_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.theme(), Some(&Theme::Dark));

        // Idempotent: set same theme
        let events = decide(
            &UserPreferencesCommand::SetTheme {
                user_id,
                theme: Theme::Dark,
                set_at: ts,
            },
            &state,
        )
        .unwrap();
        assert!(events.is_empty());

        // Set locale
        let locale = Locale::new("fr-FR").unwrap();
        let events = decide(
            &UserPreferencesCommand::SetLocale {
                user_id,
                locale: locale.clone(),
                set_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.locale().unwrap(), &locale);

        // Update UI state
        let ui_state = UiState::new(r#"{"panels": ["left", "right"]}"#);
        let events = decide(
            &UserPreferencesCommand::UpdateUiState {
                user_id,
                ui_state: ui_state.clone(),
                updated_at: ts,
            },
            &state,
        )
        .unwrap();
        assert_eq!(events.len(), 1);

        let state = evolve(&state, &events[0]);
        assert_eq!(state.ui_state().unwrap(), &ui_state);

        // Idempotent: same UI state
        let events = decide(
            &UserPreferencesCommand::UpdateUiState {
                user_id,
                ui_state,
                updated_at: ts,
            },
            &state,
        )
        .unwrap();
        assert!(events.is_empty());
    }
}
