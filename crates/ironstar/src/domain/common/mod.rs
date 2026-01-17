//! Common domain value objects shared across aggregates.
//!
//! This module contains reusable value objects that enforce invariants
//! at construction time. These types implement the "parse, don't validate"
//! principle: if you have a valid instance, its invariants are guaranteed.
//!
//! # Types
//!
//! - [`BoundedString`]: Generic string type with const generic length bounds
//! - [`DashboardTitle`]: Title for dashboards (1-200 chars)
//! - [`TabTitle`]: Title for tabs (1-100 chars)
//! - [`GridSize`]: Grid dimensions with minimum size constraints

pub mod values;

pub use values::{
    BoundedString, DASHBOARD_TITLE_MAX_LENGTH, DASHBOARD_TITLE_MIN_LENGTH, DashboardTitle,
    GRID_HEIGHT_MIN, GRID_WIDTH_MIN, GridSize, TAB_TITLE_MAX_LENGTH, TAB_TITLE_MIN_LENGTH,
    TabTitle,
};
