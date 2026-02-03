//! Commands for the UserPreferences aggregate.
//!
//! Commands represent requests to change user preferences state.
//! Timestamps are injected at the boundary layer; the pure decider
//! does not call `Utc::now()`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{Locale, PreferencesId, Theme, UiState};
use crate::domain::session::UserId;
use crate::domain::traits::{DeciderType, Identifier};

/// Commands that can be sent to the UserPreferences aggregate.
///
/// The aggregate ID follows the pattern `user_{user_id}/preferences`,
/// making this a per-user singleton.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "commands/")]
pub enum UserPreferencesCommand {
    /// Initialize preferences for a user.
    ///
    /// Can only succeed when preferences do not yet exist.
    InitializePreferences {
        preferences_id: PreferencesId,
        user_id: UserId,
        initialized_at: DateTime<Utc>,
    },

    /// Set the visual theme for this user.
    ///
    /// Requires preferences to be initialized. Idempotent when
    /// setting the same theme.
    SetTheme {
        user_id: UserId,
        theme: Theme,
        set_at: DateTime<Utc>,
    },

    /// Set the locale for this user.
    ///
    /// Requires preferences to be initialized. Idempotent when
    /// setting the same locale.
    SetLocale {
        user_id: UserId,
        locale: Locale,
        set_at: DateTime<Utc>,
    },

    /// Update arbitrary UI state for this user.
    ///
    /// Requires preferences to be initialized. Idempotent when
    /// setting the same UI state.
    UpdateUiState {
        user_id: UserId,
        ui_state: UiState,
        updated_at: DateTime<Utc>,
    },
}

impl UserPreferencesCommand {
    /// Extract the user ID from the command.
    #[must_use]
    pub fn user_id(&self) -> UserId {
        match self {
            Self::InitializePreferences { user_id, .. }
            | Self::SetTheme { user_id, .. }
            | Self::SetLocale { user_id, .. }
            | Self::UpdateUiState { user_id, .. } => *user_id,
        }
    }

    /// Get the command type name for logging and metrics.
    #[must_use]
    pub fn command_type(&self) -> &'static str {
        match self {
            Self::InitializePreferences { .. } => "InitializePreferences",
            Self::SetTheme { .. } => "SetTheme",
            Self::SetLocale { .. } => "SetLocale",
            Self::UpdateUiState { .. } => "UpdateUiState",
        }
    }
}

impl Identifier for UserPreferencesCommand {
    fn identifier(&self) -> String {
        format!("user_{}/preferences", self.user_id())
    }
}

impl DeciderType for UserPreferencesCommand {
    fn decider_type(&self) -> String {
        "UserPreferences".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn initialize_command_serializes_with_type_tag() {
        let user_id = UserId::from_uuid(uuid::Uuid::nil());
        let pref_id = PreferencesId::from_uuid(uuid::Uuid::nil());
        let cmd = UserPreferencesCommand::InitializePreferences {
            preferences_id: pref_id,
            user_id,
            initialized_at: sample_time(),
        };

        let json = serde_json::to_value(&cmd).unwrap();
        assert_eq!(json["type"], "InitializePreferences");
        assert_eq!(json["user_id"], "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn command_roundtrips_through_json() {
        let original = UserPreferencesCommand::SetTheme {
            user_id: UserId::new(),
            theme: Theme::Dark,
            set_at: sample_time(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: UserPreferencesCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn identifier_follows_aggregate_id_pattern() {
        let user_id = UserId::from_uuid(uuid::Uuid::nil());
        let pref_id = PreferencesId::from_uuid(uuid::Uuid::nil());
        let cmd = UserPreferencesCommand::InitializePreferences {
            preferences_id: pref_id,
            user_id,
            initialized_at: sample_time(),
        };

        assert_eq!(
            cmd.identifier(),
            "user_00000000-0000-0000-0000-000000000000/preferences"
        );
    }

    #[test]
    fn user_id_extracts_correctly() {
        let user_id = UserId::new();
        let ts = sample_time();

        let commands = vec![
            UserPreferencesCommand::InitializePreferences {
                preferences_id: PreferencesId::new(),
                user_id,
                initialized_at: ts,
            },
            UserPreferencesCommand::SetTheme {
                user_id,
                theme: Theme::Dark,
                set_at: ts,
            },
            UserPreferencesCommand::SetLocale {
                user_id,
                locale: Locale::default(),
                set_at: ts,
            },
            UserPreferencesCommand::UpdateUiState {
                user_id,
                ui_state: UiState::default(),
                updated_at: ts,
            },
        ];

        for cmd in commands {
            assert_eq!(cmd.user_id(), user_id);
        }
    }
}
