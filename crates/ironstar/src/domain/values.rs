//! Value objects with smart constructors.
//!
//! Value objects are immutable, equality-compared by value (not identity),
//! and validated at construction time. This module implements the "parse,
//! don't validate" principle: if you have a `TodoText`, it is guaranteed
//! to be non-empty and within length limits.
//!
//! # Smart constructor pattern
//!
//! Each value object has a private inner field and a `new()` constructor
//! that validates input and returns `Result<Self, Error>`. This makes
//! invalid states unrepresentable at the type level.
//!
//! ```rust,ignore
//! // Construction can fail
//! let text = TodoText::new("")?; // Err(TodoError::EmptyText)
//!
//! // But once you have a TodoText, it's guaranteed valid
//! fn process(text: TodoText) {
//!     // No validation needed - the type is the proof
//! }
//! ```

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::errors::TodoError;

/// Maximum length for todo text in characters.
pub const TODO_TEXT_MAX_LENGTH: usize = 500;

/// Unique identifier for a Todo item.
///
/// Wraps a UUID v4, providing type safety to prevent mixing up different
/// ID types (e.g., passing a `UserId` where a `TodoId` is expected).
///
/// # Construction
///
/// - `TodoId::new()` - Generate a new random ID
/// - `TodoId::from_uuid(uuid)` - Wrap an existing UUID (for deserialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
#[serde(transparent)]
pub struct TodoId(Uuid);

impl TodoId {
    /// Generate a new random TodoId.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID as a TodoId.
    ///
    /// Use this when deserializing from storage or parsing from input.
    /// For new todos, prefer `TodoId::new()`.
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

impl Default for TodoId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TodoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validated todo text.
///
/// Guarantees:
/// - Non-empty (at least one non-whitespace character)
/// - At most [`TODO_TEXT_MAX_LENGTH`] characters
/// - Trimmed of leading/trailing whitespace
///
/// # Example
///
/// ```rust,ignore
/// let text = TodoText::new("  Buy groceries  ")?;
/// assert_eq!(text.as_str(), "Buy groceries"); // Trimmed
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
#[serde(try_from = "String", into = "String")]
pub struct TodoText(String);

impl TodoText {
    /// Create a new TodoText, validating and normalizing the input.
    ///
    /// # Errors
    ///
    /// - [`TodoError::EmptyText`] if the trimmed text is empty
    /// - [`TodoError::TextTooLong`] if the text exceeds [`TODO_TEXT_MAX_LENGTH`]
    pub fn new(text: impl Into<String>) -> Result<Self, TodoError> {
        let text = text.into();
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return Err(TodoError::EmptyText);
        }

        let char_count = trimmed.chars().count();
        if char_count > TODO_TEXT_MAX_LENGTH {
            return Err(TodoError::TextTooLong {
                max: TODO_TEXT_MAX_LENGTH,
                actual: char_count,
            });
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Get the text as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume self and return the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for TodoText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for TodoText {
    type Error = TodoError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<TodoText> for String {
    fn from(text: TodoText) -> Self {
        text.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod todo_id {
        use super::*;

        #[test]
        fn new_generates_unique_ids() {
            let id1 = TodoId::new();
            let id2 = TodoId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn from_uuid_roundtrips() {
            let uuid = Uuid::new_v4();
            let id = TodoId::from_uuid(uuid);
            assert_eq!(id.into_inner(), uuid);
        }

        #[test]
        fn serializes_as_string() {
            let id = TodoId::from_uuid(Uuid::nil());
            let json = serde_json::to_string(&id).unwrap();
            assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
        }
    }

    mod todo_text {
        use super::*;

        #[test]
        fn accepts_valid_text() {
            let text = TodoText::new("Buy groceries").unwrap();
            assert_eq!(text.as_str(), "Buy groceries");
        }

        #[test]
        fn trims_whitespace() {
            let text = TodoText::new("  Buy groceries  ").unwrap();
            assert_eq!(text.as_str(), "Buy groceries");
        }

        #[test]
        fn rejects_empty_string() {
            let result = TodoText::new("");
            assert_eq!(result, Err(TodoError::EmptyText));
        }

        #[test]
        fn rejects_whitespace_only() {
            let result = TodoText::new("   \t\n  ");
            assert_eq!(result, Err(TodoError::EmptyText));
        }

        #[test]
        fn rejects_too_long_text() {
            let long_text = "a".repeat(TODO_TEXT_MAX_LENGTH + 1);
            let result = TodoText::new(&long_text);
            assert_eq!(
                result,
                Err(TodoError::TextTooLong {
                    max: TODO_TEXT_MAX_LENGTH,
                    actual: TODO_TEXT_MAX_LENGTH + 1,
                })
            );
        }

        #[test]
        fn accepts_max_length_text() {
            let max_text = "a".repeat(TODO_TEXT_MAX_LENGTH);
            let result = TodoText::new(&max_text);
            assert!(result.is_ok());
        }

        #[test]
        fn serde_roundtrip_valid() {
            let original = TodoText::new("Test todo").unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: TodoText = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn serde_rejects_invalid() {
            let json = r#""""#; // Empty string
            let result: Result<TodoText, _> = serde_json::from_str(json);
            assert!(result.is_err());
        }
    }
}
