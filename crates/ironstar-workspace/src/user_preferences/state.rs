//! UserPreferences aggregate state types.
//!
//! State is derived from events via replay. Uses a sum type enum following
//! the WorkspacePreferences aggregate pattern for clean state machine semantics.

use super::values::{Locale, PreferencesId, Theme, UiState};
use ironstar_shared_kernel::UserId;

/// State of user preferences, derived from events.
///
/// ```text
///                    ┌──────────────────┐
///  Initialize ──────►│   Initialized    │
///                    └────────┬─────────┘
///                             │
///          ┌──────────────────┼──────────────────┐
///          │                  │                   │
///       SetTheme          SetLocale        UpdateUiState
///          │                  │                   │
///          └──────────────────┴──────────────────-┘
///                             │
///                             ▼
///                    ┌──────────────────┐
///                    │   Initialized    │ (updated fields)
///                    └──────────────────┘
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub enum UserPreferencesState {
    /// Initial state before initialization.
    #[default]
    NotInitialized,

    /// Preferences exist for the user.
    Initialized {
        /// Unique identifier for this preferences instance.
        preferences_id: PreferencesId,
        /// The user these preferences belong to.
        user_id: UserId,
        /// Visual theme selection.
        theme: Theme,
        /// BCP-47 locale tag.
        locale: Locale,
        /// Arbitrary UI state as JSON.
        ui_state: UiState,
    },
}

impl UserPreferencesState {
    /// Check if preferences have been initialized.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        matches!(self, Self::Initialized { .. })
    }

    /// Get the preferences ID, if initialized.
    #[must_use]
    pub fn preferences_id(&self) -> Option<&PreferencesId> {
        match self {
            Self::NotInitialized => None,
            Self::Initialized { preferences_id, .. } => Some(preferences_id),
        }
    }

    /// Get the user ID, if initialized.
    #[must_use]
    pub fn user_id(&self) -> Option<&UserId> {
        match self {
            Self::NotInitialized => None,
            Self::Initialized { user_id, .. } => Some(user_id),
        }
    }

    /// Get the theme, if initialized.
    #[must_use]
    pub fn theme(&self) -> Option<&Theme> {
        match self {
            Self::NotInitialized => None,
            Self::Initialized { theme, .. } => Some(theme),
        }
    }

    /// Get the locale, if initialized.
    #[must_use]
    pub fn locale(&self) -> Option<&Locale> {
        match self {
            Self::NotInitialized => None,
            Self::Initialized { locale, .. } => Some(locale),
        }
    }

    /// Get the UI state, if initialized.
    #[must_use]
    pub fn ui_state(&self) -> Option<&UiState> {
        match self {
            Self::NotInitialized => None,
            Self::Initialized { ui_state, .. } => Some(ui_state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_not_initialized() {
        let state = UserPreferencesState::default();
        assert!(!state.is_initialized());
        assert!(state.preferences_id().is_none());
        assert!(state.user_id().is_none());
        assert!(state.theme().is_none());
        assert!(state.locale().is_none());
        assert!(state.ui_state().is_none());
    }

    #[test]
    fn initialized_state() {
        let pref_id = PreferencesId::from_uuid(uuid::Uuid::nil());
        let user_id = UserId::from_uuid(uuid::Uuid::nil());
        let state = UserPreferencesState::Initialized {
            preferences_id: pref_id,
            user_id,
            theme: Theme::Dark,
            locale: Locale::new("fr-FR").unwrap(),
            ui_state: UiState::new(r#"{"sidebar": "open"}"#),
        };

        assert!(state.is_initialized());
        assert_eq!(state.preferences_id(), Some(&pref_id));
        assert_eq!(state.user_id(), Some(&user_id));
        assert_eq!(state.theme(), Some(&Theme::Dark));
        assert_eq!(state.locale().unwrap().as_str(), "fr-FR");
        assert_eq!(state.ui_state().unwrap().as_str(), r#"{"sidebar": "open"}"#);
    }
}
