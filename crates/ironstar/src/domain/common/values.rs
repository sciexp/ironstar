//! Value objects re-exported from ironstar-core.
//!
//! All value objects with smart constructors are now defined in `ironstar-core`.
//! This module re-exports them for backward compatibility within the monolith.

pub use ironstar_core::{
    BoundedString, DASHBOARD_TITLE_MAX_LENGTH, DASHBOARD_TITLE_MIN_LENGTH, DashboardTitle,
    GRID_HEIGHT_MIN, GRID_WIDTH_MIN, GridSize, TAB_TITLE_MAX_LENGTH, TAB_TITLE_MIN_LENGTH,
    TabTitle,
};
