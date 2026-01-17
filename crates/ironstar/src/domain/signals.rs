//! Datastar signal types with ts-rs derives.
//!
//! Signals represent client-side reactive state for Datastar's FRP model.
//! They flow bidirectionally via SSE: server pushes signal updates,
//! browser sends signal state with requests.
//!
//! # Comonad structure
//!
//! Datastar signals form a comonad (dual to server-side monads):
//!
//! - `extract`: `Signal a → a` (access via `$signal.value`)
//! - `extend`: `(Signal a → b) → Signal a → Signal b` (computed signals)
//!
//! The comonad laws ensure signal composition is well-behaved:
//!
//! - `extend extract = id`
//! - `extract ∘ extend f = f`
//! - `extend f ∘ extend g = extend (f ∘ extend g)`
//!
//! # Type generation
//!
//! Signal types use ts-rs to generate TypeScript definitions during
//! `cargo test --lib`. The generated types ensure type safety across
//! the JSON serialization boundary between Rust handlers and browser.
//!
//! # Example
//!
//! ```rust,ignore
//! use ironstar::domain::signals::{TodoSignals, TodoFilter};
//!
//! let signals = TodoSignals {
//!     input: None,
//!     filter: TodoFilter::All,
//!     editing_id: None,
//!     loading: false,
//! };
//!
//! // Serialize for data-signals attribute
//! let json = serde_json::to_string(&signals)?;
//! ```

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Todo application signals.
///
/// Represents the client-side state for the todo list UI:
/// - Form input value
/// - Active filter mode
/// - Currently editing item (if any)
/// - Loading indicator
///
/// This struct is used both for:
/// - Server → Browser: Initial state in `data-signals` attribute
/// - Browser → Server: Request body via `ReadSignals<TodoSignals>`
#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "web-components/types/signals/")]
pub struct TodoSignals {
    /// Current input field value.
    ///
    /// Bound to the todo input field via `data-bind:input`.
    /// None when the field is empty or cleared.
    #[ts(optional)]
    pub input: Option<String>,

    /// Active filter mode for the todo list.
    ///
    /// Controls which todos are displayed: all, active only, or completed only.
    pub filter: TodoFilter,

    /// ID of the todo currently being edited.
    ///
    /// When Some, the UI shows an inline edit field for this todo.
    /// When None, no todo is in edit mode.
    #[ts(optional)]
    pub editing_id: Option<Uuid>,

    /// Loading indicator state.
    ///
    /// True while a request is in flight, enabling UI feedback.
    #[serde(default)]
    pub loading: bool,
}

/// Filter options for the todo list.
///
/// Determines which todos are visible based on their completion status.
/// Serializes to lowercase strings for JavaScript compatibility.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "web-components/types/signals/")]
pub enum TodoFilter {
    /// Show all todos regardless of status.
    #[default]
    #[serde(rename = "all")]
    All,

    /// Show only active (not completed) todos.
    #[serde(rename = "active")]
    Active,

    /// Show only completed todos.
    #[serde(rename = "completed")]
    Completed,
}

/// View projection of a single todo item for list rendering.
///
/// This is a read-only projection derived from [`super::TodoState`],
/// shaped for efficient list rendering via SSE updates.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "web-components/types/signals/")]
pub struct TodoItemView {
    /// Unique identifier for this todo.
    pub id: Uuid,

    /// The todo text content.
    pub text: String,

    /// Whether the todo is completed.
    pub completed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_signals_default() {
        let signals = TodoSignals::default();
        assert_eq!(signals.input, None);
        assert_eq!(signals.filter, TodoFilter::All);
        assert_eq!(signals.editing_id, None);
        assert!(!signals.loading);
    }

    #[test]
    fn todo_filter_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&TodoFilter::All).unwrap(), r#""all""#);
        assert_eq!(
            serde_json::to_string(&TodoFilter::Active).unwrap(),
            r#""active""#
        );
        assert_eq!(
            serde_json::to_string(&TodoFilter::Completed).unwrap(),
            r#""completed""#
        );
    }

    #[test]
    fn todo_filter_deserializes_lowercase() {
        assert_eq!(
            serde_json::from_str::<TodoFilter>(r#""all""#).unwrap(),
            TodoFilter::All
        );
        assert_eq!(
            serde_json::from_str::<TodoFilter>(r#""active""#).unwrap(),
            TodoFilter::Active
        );
        assert_eq!(
            serde_json::from_str::<TodoFilter>(r#""completed""#).unwrap(),
            TodoFilter::Completed
        );
    }

    #[test]
    fn todo_signals_roundtrip() {
        let signals = TodoSignals {
            input: Some("Buy groceries".to_string()),
            filter: TodoFilter::Active,
            editing_id: Some(Uuid::nil()),
            loading: true,
        };

        let json = serde_json::to_string(&signals).unwrap();
        let parsed: TodoSignals = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.input, signals.input);
        assert_eq!(parsed.filter, signals.filter);
        assert_eq!(parsed.editing_id, signals.editing_id);
        assert_eq!(parsed.loading, signals.loading);
    }

    #[test]
    fn todo_signals_optional_fields_omitted() {
        // Deserialize with missing optional fields
        let json = r#"{"filter": "all", "loading": false}"#;
        let signals: TodoSignals = serde_json::from_str(json).unwrap();

        assert_eq!(signals.input, None);
        assert_eq!(signals.editing_id, None);
    }

    #[test]
    fn todo_item_view_serializes() {
        let item = TodoItemView {
            id: Uuid::nil(),
            text: "Test todo".to_string(),
            completed: false,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"text\""));
        assert!(json.contains("\"completed\""));
    }

    #[test]
    fn todo_signals_uses_camel_case() {
        let signals = TodoSignals {
            input: None,
            filter: TodoFilter::All,
            editing_id: Some(Uuid::nil()),
            loading: false,
        };

        let json = serde_json::to_string(&signals).unwrap();
        // editing_id should serialize as editingId
        assert!(json.contains("\"editingId\""));
        assert!(!json.contains("\"editing_id\""));
    }

    #[test]
    fn todo_signals_deserializes_camel_case() {
        // JSON uses camelCase field names
        let json = r#"{"filter": "active", "editingId": "00000000-0000-0000-0000-000000000000", "loading": true}"#;
        let signals: TodoSignals = serde_json::from_str(json).unwrap();

        assert_eq!(signals.filter, TodoFilter::Active);
        assert_eq!(signals.editing_id, Some(Uuid::nil()));
        assert!(signals.loading);
    }
}
