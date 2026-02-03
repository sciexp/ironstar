//! Value objects for the UserPreferences aggregate.
//!
//! - `PreferencesId`: UUID wrapper for preferences identity
//! - `Theme`: Visual theme selection (Light, Dark, System)
//! - `Locale`: Validated BCP-47 language tag
//! - `UiState`: Opaque JSON string for arbitrary UI state

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::error::{ValidationError, ValidationErrorKind};

/// Maximum length for a BCP-47 locale tag in characters.
pub const LOCALE_MAX_LENGTH: usize = 35;

/// Unique identifier for a UserPreferences instance.
///
/// Wraps a UUID v4, providing type safety to prevent mixing up different
/// ID types.
///
/// # Construction
///
/// - `PreferencesId::new()` - Generate a new random ID
/// - `PreferencesId::from_uuid(uuid)` - Wrap an existing UUID (for deserialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct PreferencesId(Uuid);

impl PreferencesId {
    /// Generate a new random PreferencesId.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID as a PreferencesId.
    ///
    /// Use this when deserializing from storage or parsing from input.
    /// For new preferences, prefer `PreferencesId::new()`.
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Extract the inner UUID.
    #[must_use]
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl Default for PreferencesId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PreferencesId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Visual theme selection for the user interface.
///
/// Defaults to `System`, which defers to the operating system preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    System,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "light"),
            Self::Dark => write!(f, "dark"),
            Self::System => write!(f, "system"),
        }
    }
}

/// Validated BCP-47 language tag.
///
/// Structural guarantees:
/// - Non-empty (at least one non-whitespace character)
/// - At most [`LOCALE_MAX_LENGTH`] characters
/// - Trimmed of leading/trailing whitespace
///
/// Semantic validation (e.g., whether the tag references a real locale)
/// is deferred to the boundary layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(try_from = "String", into = "String")]
pub struct Locale(String);

impl Locale {
    /// Create a new Locale, validating and normalizing the input.
    ///
    /// # Errors
    ///
    /// - [`ValidationError`] with `EmptyField` if the trimmed locale is empty
    /// - [`ValidationError`] with `TooLong` if it exceeds [`LOCALE_MAX_LENGTH`]
    pub fn new(locale: impl Into<String>) -> Result<Self, ValidationError> {
        let locale = locale.into();
        let trimmed = locale.trim();

        if trimmed.is_empty() {
            return Err(ValidationError::new(ValidationErrorKind::EmptyField {
                field: "locale".to_string(),
            }));
        }

        let char_count = trimmed.chars().count();
        if char_count > LOCALE_MAX_LENGTH {
            return Err(ValidationError::new(ValidationErrorKind::TooLong {
                field: "locale".to_string(),
                max_length: LOCALE_MAX_LENGTH,
                actual_length: char_count,
            }));
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Get the locale tag as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self("en-US".to_string())
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Locale {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Locale> for String {
    fn from(locale: Locale) -> Self {
        locale.0
    }
}

/// JSON string representing arbitrary UI state.
///
/// The domain layer treats this as an opaque string.
/// JSON validation is deferred to the boundary layer per Hoffman's Law 7
/// (work is a side effect).
///
/// Default value is `"{}"` (empty JSON object).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct UiState(String);

impl UiState {
    /// Create UiState from a string.
    ///
    /// No validation is performed in the domain layer; JSON validity
    /// is enforced at the boundary before command construction.
    #[must_use]
    pub fn new(json: impl Into<String>) -> Self {
        Self(json.into())
    }

    /// Get the JSON string as a slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self("{}".to_string())
    }
}

impl std::fmt::Display for UiState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<UiState> for String {
    fn from(ui_state: UiState) -> Self {
        ui_state.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod preferences_id {
        use super::*;

        #[test]
        fn new_generates_unique_ids() {
            let id1 = PreferencesId::new();
            let id2 = PreferencesId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn from_uuid_roundtrips() {
            let uuid = Uuid::nil();
            let id = PreferencesId::from_uuid(uuid);
            assert_eq!(id.into_inner(), uuid);
        }

        #[test]
        fn display_shows_uuid() {
            let id = PreferencesId::from_uuid(Uuid::nil());
            assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
        }

        #[test]
        fn serde_roundtrip() {
            let original = PreferencesId::from_uuid(Uuid::nil());
            let json = serde_json::to_string(&original).unwrap();
            let parsed: PreferencesId = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }
    }

    mod theme {
        use super::*;

        #[test]
        fn default_is_system() {
            assert_eq!(Theme::default(), Theme::System);
        }

        #[test]
        fn display_lowercase() {
            assert_eq!(Theme::Light.to_string(), "light");
            assert_eq!(Theme::Dark.to_string(), "dark");
            assert_eq!(Theme::System.to_string(), "system");
        }

        #[test]
        fn serde_roundtrip() {
            let original = Theme::Dark;
            let json = serde_json::to_string(&original).unwrap();
            let parsed: Theme = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }
    }

    mod locale {
        use super::*;

        #[test]
        fn accepts_valid_locale() {
            let locale = Locale::new("en-US").unwrap();
            assert_eq!(locale.as_str(), "en-US");
        }

        #[test]
        fn trims_whitespace() {
            let locale = Locale::new("  fr-FR  ").unwrap();
            assert_eq!(locale.as_str(), "fr-FR");
        }

        #[test]
        fn default_is_en_us() {
            assert_eq!(Locale::default().as_str(), "en-US");
        }

        #[test]
        fn rejects_empty_string() {
            let result = Locale::new("");
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                ValidationErrorKind::EmptyField { .. }
            ));
        }

        #[test]
        fn rejects_whitespace_only() {
            let result = Locale::new("   \t\n  ");
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                ValidationErrorKind::EmptyField { .. }
            ));
        }

        #[test]
        fn rejects_too_long_locale() {
            let long_locale = "a".repeat(LOCALE_MAX_LENGTH + 1);
            let result = Locale::new(&long_locale);
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err().kind(),
                ValidationErrorKind::TooLong { .. }
            ));
        }

        #[test]
        fn accepts_max_length_locale() {
            let max_locale = "a".repeat(LOCALE_MAX_LENGTH);
            assert!(Locale::new(&max_locale).is_ok());
        }

        #[test]
        fn serde_roundtrip() {
            let original = Locale::new("de-DE").unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: Locale = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn serde_rejects_empty() {
            let json = r#""""#;
            let result: Result<Locale, _> = serde_json::from_str(json);
            assert!(result.is_err());
        }
    }

    mod ui_state {
        use super::*;

        #[test]
        fn default_is_empty_object() {
            assert_eq!(UiState::default().as_str(), "{}");
        }

        #[test]
        fn accepts_any_string() {
            let state = UiState::new(r#"{"sidebar": "collapsed"}"#);
            assert_eq!(state.as_str(), r#"{"sidebar": "collapsed"}"#);
        }

        #[test]
        fn serde_roundtrip() {
            let original = UiState::new(r#"{"panel": "open"}"#);
            let json = serde_json::to_string(&original).unwrap();
            let parsed: UiState = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }
    }
}
