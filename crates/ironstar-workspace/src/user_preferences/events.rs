//! Domain events for the UserPreferences aggregate.
//!
//! Events represent facts that have occurred. They are immutable, past-tense,
//! and self-describing per Hoffman's Law 1.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::values::{Locale, PreferencesId, Theme, UiState};
use ironstar_core::{DeciderType, EventType, Identifier, IsFinal};
use ironstar_shared_kernel::UserId;

/// Events emitted by the UserPreferences aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "events/")]
pub enum UserPreferencesEvent {
    /// Preferences were initialized for a user.
    PreferencesInitialized {
        preferences_id: PreferencesId,
        user_id: UserId,
        initialized_at: DateTime<Utc>,
    },

    /// Visual theme was set.
    ThemeSet {
        user_id: UserId,
        theme: Theme,
        set_at: DateTime<Utc>,
    },

    /// Locale was set.
    LocaleSet {
        user_id: UserId,
        locale: Locale,
        set_at: DateTime<Utc>,
    },

    /// UI state was updated.
    UiStateUpdated {
        user_id: UserId,
        ui_state: UiState,
        updated_at: DateTime<Utc>,
    },
}

impl UserPreferencesEvent {
    /// Extract the user ID this event belongs to.
    #[must_use]
    pub fn user_id(&self) -> UserId {
        match self {
            Self::PreferencesInitialized { user_id, .. }
            | Self::ThemeSet { user_id, .. }
            | Self::LocaleSet { user_id, .. }
            | Self::UiStateUpdated { user_id, .. } => *user_id,
        }
    }

    /// Get the event type name for storage and routing.
    #[must_use]
    pub fn event_type_str(&self) -> &'static str {
        match self {
            Self::PreferencesInitialized { .. } => "PreferencesInitialized",
            Self::ThemeSet { .. } => "ThemeSet",
            Self::LocaleSet { .. } => "LocaleSet",
            Self::UiStateUpdated { .. } => "UiStateUpdated",
        }
    }

    /// Get the event version for schema evolution.
    #[must_use]
    pub fn event_version(&self) -> &'static str {
        "1"
    }
}

impl Identifier for UserPreferencesEvent {
    fn identifier(&self) -> String {
        format!("user_{}/preferences", self.user_id())
    }
}

impl EventType for UserPreferencesEvent {
    fn event_type(&self) -> String {
        self.event_type_str().to_string()
    }
}

impl DeciderType for UserPreferencesEvent {
    fn decider_type(&self) -> String {
        "UserPreferences".to_string()
    }
}

impl IsFinal for UserPreferencesEvent {
    fn is_final(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_id() -> UserId {
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

    #[test]
    fn initialized_event_serializes_with_type_tag() {
        let event = UserPreferencesEvent::PreferencesInitialized {
            preferences_id: sample_pref_id(),
            user_id: sample_id(),
            initialized_at: sample_time(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "PreferencesInitialized");
        assert_eq!(json["user_id"], "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn event_roundtrips_through_json() {
        let original = UserPreferencesEvent::ThemeSet {
            user_id: sample_id(),
            theme: Theme::Dark,
            set_at: sample_time(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: UserPreferencesEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn identifier_follows_aggregate_id_pattern() {
        let event = UserPreferencesEvent::PreferencesInitialized {
            preferences_id: sample_pref_id(),
            user_id: sample_id(),
            initialized_at: sample_time(),
        };

        assert_eq!(
            event.identifier(),
            "user_00000000-0000-0000-0000-000000000000/preferences"
        );
    }

    #[test]
    fn event_type_matches_serde_tag() {
        let events: Vec<(UserPreferencesEvent, &str)> = vec![
            (
                UserPreferencesEvent::PreferencesInitialized {
                    preferences_id: sample_pref_id(),
                    user_id: sample_id(),
                    initialized_at: sample_time(),
                },
                "PreferencesInitialized",
            ),
            (
                UserPreferencesEvent::ThemeSet {
                    user_id: sample_id(),
                    theme: Theme::Light,
                    set_at: sample_time(),
                },
                "ThemeSet",
            ),
            (
                UserPreferencesEvent::LocaleSet {
                    user_id: sample_id(),
                    locale: Locale::new("ja-JP").unwrap(),
                    set_at: sample_time(),
                },
                "LocaleSet",
            ),
            (
                UserPreferencesEvent::UiStateUpdated {
                    user_id: sample_id(),
                    ui_state: UiState::default(),
                    updated_at: sample_time(),
                },
                "UiStateUpdated",
            ),
        ];

        for (event, expected_type) in events {
            assert_eq!(event.event_type_str(), expected_type);

            let json = serde_json::to_value(&event).unwrap();
            assert_eq!(json["type"], expected_type);
        }
    }

    #[test]
    fn is_final_returns_false_for_all_events() {
        let events = vec![
            UserPreferencesEvent::PreferencesInitialized {
                preferences_id: sample_pref_id(),
                user_id: sample_id(),
                initialized_at: sample_time(),
            },
            UserPreferencesEvent::ThemeSet {
                user_id: sample_id(),
                theme: Theme::Dark,
                set_at: sample_time(),
            },
        ];

        for event in events {
            assert!(!event.is_final());
        }
    }
}
