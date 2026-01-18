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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
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

/// ECharts selection event data.
///
/// Captures the data point selected by user interaction with a chart.
/// Populated from ECharts click/select events and sent to server
/// for coordinated updates (e.g., filtering related data).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "web-components/types/signals/")]
pub struct ChartSelection {
    /// Name of the series containing the selected data point.
    pub series_name: String,

    /// Index of the data point within the series.
    pub data_index: usize,

    /// Name/label of the selected data point (e.g., category name).
    pub name: String,

    /// Value of the selected data point.
    ///
    /// Uses `serde_json::Value` to accommodate ECharts' flexible data types
    /// (numbers, arrays, objects depending on chart type).
    #[ts(type = "unknown")]
    pub value: serde_json::Value,
}

/// Datastar signals for ECharts integration.
///
/// Represents the client-side state for chart components:
/// - Chart configuration (ECharts option object)
/// - Currently selected data point (if any)
/// - Loading indicator
///
/// This struct is used both for:
/// - Server → Browser: Chart updates via SSE `datastar-merge-signals`
/// - Browser → Server: Selection events via `ReadSignals<ChartSignals>`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "web-components/types/signals/")]
pub struct ChartSignals {
    /// ECharts option configuration.
    ///
    /// Uses `serde_json::Value` to pass through the complete ECharts option
    /// object without requiring Rust types for all ECharts options.
    #[ts(type = "unknown")]
    pub chart_option: serde_json::Value,

    /// Currently selected data point, if any.
    ///
    /// Populated when user clicks/selects a chart element.
    /// None when no selection is active.
    #[ts(optional)]
    pub selected: Option<ChartSelection>,

    /// Loading indicator state.
    ///
    /// True while chart data is being fetched, enabling loading UI.
    #[serde(default)]
    pub loading: bool,
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

    #[test]
    fn chart_selection_roundtrip() {
        let selection = ChartSelection {
            series_name: "Sales".to_string(),
            data_index: 3,
            name: "Q4".to_string(),
            value: serde_json::json!(1250.50),
        };

        let json = serde_json::to_string(&selection).unwrap();
        let parsed: ChartSelection = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.series_name, selection.series_name);
        assert_eq!(parsed.data_index, selection.data_index);
        assert_eq!(parsed.name, selection.name);
        assert_eq!(parsed.value, selection.value);
    }

    #[test]
    fn chart_selection_uses_camel_case() {
        let selection = ChartSelection {
            series_name: "Revenue".to_string(),
            data_index: 0,
            name: "January".to_string(),
            value: serde_json::json!([100, 200]),
        };

        let json = serde_json::to_string(&selection).unwrap();
        // series_name should serialize as seriesName
        assert!(json.contains("\"seriesName\""));
        assert!(!json.contains("\"series_name\""));
        // data_index should serialize as dataIndex
        assert!(json.contains("\"dataIndex\""));
        assert!(!json.contains("\"data_index\""));
    }

    #[test]
    fn chart_signals_roundtrip() {
        let signals = ChartSignals {
            chart_option: serde_json::json!({
                "xAxis": {"type": "category"},
                "yAxis": {"type": "value"},
                "series": [{"type": "bar", "data": [10, 20, 30]}]
            }),
            selected: Some(ChartSelection {
                series_name: "Series1".to_string(),
                data_index: 1,
                name: "B".to_string(),
                value: serde_json::json!(20),
            }),
            loading: false,
        };

        let json = serde_json::to_string(&signals).unwrap();
        let parsed: ChartSignals = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.chart_option, signals.chart_option);
        assert_eq!(parsed.selected, signals.selected);
        assert_eq!(parsed.loading, signals.loading);
    }

    #[test]
    fn chart_signals_uses_camel_case() {
        let signals = ChartSignals {
            chart_option: serde_json::json!({}),
            selected: None,
            loading: true,
        };

        let json = serde_json::to_string(&signals).unwrap();
        // chart_option should serialize as chartOption
        assert!(json.contains("\"chartOption\""));
        assert!(!json.contains("\"chart_option\""));
    }

    #[test]
    fn chart_signals_optional_selected() {
        // Deserialize with missing optional selected field
        let json = r#"{"chartOption": {}, "loading": false}"#;
        let signals: ChartSignals = serde_json::from_str(json).unwrap();

        assert_eq!(signals.selected, None);
        assert!(!signals.loading);
    }

    #[test]
    fn chart_signals_default_loading() {
        // loading defaults to false when omitted
        let json = r#"{"chartOption": {}}"#;
        let signals: ChartSignals = serde_json::from_str(json).unwrap();

        assert!(!signals.loading);
    }

    #[test]
    fn chart_selection_complex_value() {
        // ECharts can have complex value types (arrays, objects)
        let selection = ChartSelection {
            series_name: "scatter".to_string(),
            data_index: 0,
            name: "Point A".to_string(),
            value: serde_json::json!({"x": 10, "y": 20, "size": 5}),
        };

        let json = serde_json::to_string(&selection).unwrap();
        let parsed: ChartSelection = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.value["x"], 10);
        assert_eq!(parsed.value["y"], 20);
        assert_eq!(parsed.value["size"], 5);
    }
}
