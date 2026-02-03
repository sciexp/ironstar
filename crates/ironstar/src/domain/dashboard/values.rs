//! Value objects for the Dashboard aggregate.
//!
//! - `DashboardId`: Unique identifier for a dashboard instance
//! - `TabId`: Unique identifier for a tab within a dashboard
//! - `ChartId`: Unique identifier for a chart within a dashboard
//! - `ChartDefinitionRef`: Reference to an Analytics ChartDefinition
//! - `GridPosition`: Zero-indexed row/col grid position
//! - `ChartPlacement`: Full chart placement including position, size, and tab
//! - `TabInfo`: Tab metadata with ID and title

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::analytics::ChartType;
use crate::domain::common::{GridSize, TabTitle};

// ============================================================================
// DashboardId - Unique dashboard identifier
// ============================================================================

/// Unique identifier for a dashboard instance.
///
/// Wraps a UUID with domain semantics. Each dashboard has a unique ID
/// independent of the workspace it belongs to (multiple dashboards per
/// workspace are supported).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct DashboardId(Uuid);

impl DashboardId {
    /// Generate a new random DashboardId.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID as a DashboardId.
    ///
    /// Use this when deserializing from storage or parsing from input.
    /// For new dashboards, prefer `DashboardId::new()`.
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

impl Default for DashboardId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DashboardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// TabId - Unique tab identifier
// ============================================================================

/// Unique identifier for a tab within a dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct TabId(Uuid);

impl TabId {
    /// Generate a new random TabId.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID as a TabId.
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

impl Default for TabId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// ChartId - Unique chart identifier
// ============================================================================

/// Unique identifier for a chart within a dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(transparent)]
pub struct ChartId(Uuid);

impl ChartId {
    /// Generate a new random ChartId.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID as a ChartId.
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

impl Default for ChartId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ChartId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// ChartDefinitionRef - Reference to an Analytics ChartDefinition
// ============================================================================

/// Reference to an Analytics domain ChartDefinition.
///
/// This is a cross-aggregate reference following the Customer-Supplier
/// pattern: Dashboard (Workspace context) references ChartDefinition
/// (Analytics context) by ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub struct ChartDefinitionRef {
    /// The reference identifier for the chart definition.
    pub ref_id: String,
    /// Optional hint about the chart type for rendering purposes.
    pub chart_type_hint: Option<ChartType>,
}

// ============================================================================
// GridPosition - Zero-indexed row/col position
// ============================================================================

/// Zero-indexed grid position for chart placement.
///
/// Both row and col start at 0. No validation is needed since zero is a
/// valid position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub struct GridPosition {
    /// Zero-indexed row.
    pub row: u32,
    /// Zero-indexed column.
    pub col: u32,
}

// ============================================================================
// ChartPlacement - Full chart placement within a dashboard
// ============================================================================

/// A chart placed on a dashboard with position, size, and optional tab assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub struct ChartPlacement {
    /// Unique identifier for this chart placement.
    pub chart_id: ChartId,
    /// Reference to the Analytics ChartDefinition.
    pub chart_def_ref: ChartDefinitionRef,
    /// Grid position of the chart.
    pub position: GridPosition,
    /// Grid size of the chart.
    pub size: GridSize,
    /// Optional tab assignment. None means the chart is on the default view.
    pub tab_id: Option<TabId>,
}

// ============================================================================
// TabInfo - Tab metadata
// ============================================================================

/// Tab metadata for dashboard tab organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub struct TabInfo {
    /// Unique identifier for this tab.
    pub tab_id: TabId,
    /// Display name of the tab.
    pub name: TabTitle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_id_display() {
        let id = DashboardId::from_uuid(Uuid::nil());
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn dashboard_id_roundtrip() {
        let original = DashboardId::new();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: DashboardId = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn tab_id_display() {
        let id = TabId::from_uuid(Uuid::nil());
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn chart_id_display() {
        let id = ChartId::from_uuid(Uuid::nil());
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn grid_position_copy_semantics() {
        let pos = GridPosition { row: 0, col: 0 };
        let pos2 = pos;
        assert_eq!(pos, pos2);
    }

    #[test]
    fn chart_definition_ref_serde_roundtrip() {
        let def_ref = ChartDefinitionRef {
            ref_id: "chart-def-001".to_string(),
            chart_type_hint: Some(ChartType::Bar),
        };
        let json = serde_json::to_string(&def_ref).unwrap();
        let parsed: ChartDefinitionRef = serde_json::from_str(&json).unwrap();
        assert_eq!(def_ref, parsed);
    }

    #[test]
    fn chart_placement_serde_roundtrip() {
        let placement = ChartPlacement {
            chart_id: ChartId::from_uuid(Uuid::nil()),
            chart_def_ref: ChartDefinitionRef {
                ref_id: "ref-1".to_string(),
                chart_type_hint: None,
            },
            position: GridPosition { row: 1, col: 2 },
            size: GridSize::new(4, 3).unwrap(),
            tab_id: None,
        };
        let json = serde_json::to_string(&placement).unwrap();
        let parsed: ChartPlacement = serde_json::from_str(&json).unwrap();
        assert_eq!(placement, parsed);
    }

    #[test]
    fn tab_info_serde_roundtrip() {
        let tab = TabInfo {
            tab_id: TabId::from_uuid(Uuid::nil()),
            name: TabTitle::new("Overview").unwrap(),
        };
        let json = serde_json::to_string(&tab).unwrap();
        let parsed: TabInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(tab, parsed);
    }
}
