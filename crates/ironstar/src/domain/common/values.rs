//! Value objects with smart constructors.
//!
//! Value objects are immutable, equality-compared by value (not identity),
//! and validated at construction time. This module implements the "parse,
//! don't validate" principle: if you have a value object, it is guaranteed
//! to satisfy its invariants.
//!
//! [`BoundedString`] is defined in `ironstar-core` and re-exported here.
//! Domain-specific newtypes built on top of it remain in this module.

use serde::{Deserialize, Serialize};
use std::fmt;
use ts_rs::TS;

use crate::domain::error::{ValidationError, ValidationErrorKind};

// Re-export BoundedString from ironstar-core
pub use ironstar_core::BoundedString;

// ============================================================================
// DashboardTitle - Title for dashboard entities
// ============================================================================

/// Maximum length for dashboard titles in characters.
pub const DASHBOARD_TITLE_MAX_LENGTH: usize = 200;

/// Minimum length for dashboard titles in characters.
pub const DASHBOARD_TITLE_MIN_LENGTH: usize = 1;

/// Validated dashboard title.
///
/// Guarantees:
/// - Non-empty (at least 1 character)
/// - At most 200 characters
/// - Trimmed of leading/trailing whitespace
///
/// # Example
///
/// ```rust,ignore
/// let title = DashboardTitle::new("Sales Overview 2024")?;
/// assert!(!title.as_str().is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(try_from = "String", into = "String")]
pub struct DashboardTitle(BoundedString<DASHBOARD_TITLE_MIN_LENGTH, DASHBOARD_TITLE_MAX_LENGTH>);

impl DashboardTitle {
    /// Create a new DashboardTitle, validating and normalizing the input.
    ///
    /// # Errors
    ///
    /// - [`ValidationError`] with `TooShort` if the trimmed title is empty
    /// - [`ValidationError`] with `TooLong` if the title exceeds 200 characters
    pub fn new(title: impl Into<String>) -> Result<Self, ValidationError> {
        let bounded = BoundedString::new(title, "dashboard_title")?;
        Ok(Self(bounded))
    }

    /// Get the title as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consume self and return the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0.into_inner()
    }
}

impl fmt::Display for DashboardTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for DashboardTitle {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<DashboardTitle> for String {
    fn from(title: DashboardTitle) -> Self {
        title.into_inner()
    }
}

// ============================================================================
// TabTitle - Title for tab entities
// ============================================================================

/// Maximum length for tab titles in characters.
pub const TAB_TITLE_MAX_LENGTH: usize = 100;

/// Minimum length for tab titles in characters.
pub const TAB_TITLE_MIN_LENGTH: usize = 1;

/// Validated tab title.
///
/// Guarantees:
/// - Non-empty (at least 1 character)
/// - At most 100 characters
/// - Trimmed of leading/trailing whitespace
///
/// # Example
///
/// ```rust,ignore
/// let title = TabTitle::new("Revenue Trends")?;
/// assert!(!title.as_str().is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/", type = "string")]
#[serde(try_from = "String", into = "String")]
pub struct TabTitle(BoundedString<TAB_TITLE_MIN_LENGTH, TAB_TITLE_MAX_LENGTH>);

impl TabTitle {
    /// Create a new TabTitle, validating and normalizing the input.
    ///
    /// # Errors
    ///
    /// - [`ValidationError`] with `TooShort` if the trimmed title is empty
    /// - [`ValidationError`] with `TooLong` if the title exceeds 100 characters
    pub fn new(title: impl Into<String>) -> Result<Self, ValidationError> {
        let bounded = BoundedString::new(title, "tab_title")?;
        Ok(Self(bounded))
    }

    /// Get the title as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consume self and return the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0.into_inner()
    }
}

impl fmt::Display for TabTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for TabTitle {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<TabTitle> for String {
    fn from(title: TabTitle) -> Self {
        title.into_inner()
    }
}

// ============================================================================
// GridSize - Validated grid dimensions
// ============================================================================

/// Minimum width for grid size.
pub const GRID_WIDTH_MIN: u32 = 1;

/// Minimum height for grid size.
pub const GRID_HEIGHT_MIN: u32 = 1;

/// Validated grid dimensions.
///
/// Guarantees:
/// - Width >= 1
/// - Height >= 1
///
/// Represents the size of a grid layout in cells. Both dimensions must be
/// at least 1 to ensure a valid, displayable grid.
///
/// # Example
///
/// ```rust,ignore
/// let size = GridSize::new(4, 3)?; // 4 columns, 3 rows
/// assert_eq!(size.width(), 4);
/// assert_eq!(size.height(), 3);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "domain/")]
pub struct GridSize {
    width: u32,
    height: u32,
}

impl GridSize {
    /// Create a new GridSize with validated dimensions.
    ///
    /// # Errors
    ///
    /// - [`ValidationError`] with `OutOfRange` if width < 1
    /// - [`ValidationError`] with `OutOfRange` if height < 1
    pub fn new(width: u32, height: u32) -> Result<Self, ValidationError> {
        if width < GRID_WIDTH_MIN {
            return Err(ValidationError::new(ValidationErrorKind::OutOfRange {
                field: "grid_width".to_string(),
                min: i64::from(GRID_WIDTH_MIN),
                max: i64::MAX,
                actual: i64::from(width),
            }));
        }

        if height < GRID_HEIGHT_MIN {
            return Err(ValidationError::new(ValidationErrorKind::OutOfRange {
                field: "grid_height".to_string(),
                min: i64::from(GRID_HEIGHT_MIN),
                max: i64::MAX,
                actual: i64::from(height),
            }));
        }

        Ok(Self { width, height })
    }

    /// Get the grid width.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Get the grid height.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Calculate the total number of cells in the grid.
    #[must_use]
    pub const fn cell_count(&self) -> u32 {
        self.width * self.height
    }
}

impl fmt::Display for GridSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::ValidationErrorKind;

    mod dashboard_title {
        use super::*;

        #[test]
        fn accepts_valid_title() {
            let title = DashboardTitle::new("Sales Overview").unwrap();
            assert_eq!(title.as_str(), "Sales Overview");
        }

        #[test]
        fn trims_whitespace() {
            let title = DashboardTitle::new("  Revenue Report  ").unwrap();
            assert_eq!(title.as_str(), "Revenue Report");
        }

        #[test]
        fn rejects_empty_string() {
            let result = DashboardTitle::new("");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.kind(),
                ValidationErrorKind::TooShort {
                    min_length: 1,
                    actual_length: 0,
                    ..
                }
            ));
        }

        #[test]
        fn rejects_whitespace_only() {
            let result = DashboardTitle::new("   \t\n  ");
            assert!(result.is_err());
        }

        #[test]
        fn rejects_too_long() {
            let long_title = "a".repeat(DASHBOARD_TITLE_MAX_LENGTH + 1);
            let result = DashboardTitle::new(&long_title);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.kind(),
                ValidationErrorKind::TooLong {
                    max_length: 200,
                    actual_length: 201,
                    ..
                }
            ));
        }

        #[test]
        fn accepts_max_length() {
            let max_title = "a".repeat(DASHBOARD_TITLE_MAX_LENGTH);
            let result = DashboardTitle::new(&max_title);
            assert!(result.is_ok());
        }

        #[test]
        fn serde_roundtrip() {
            let original = DashboardTitle::new("Test Dashboard").unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: DashboardTitle = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn serde_rejects_empty() {
            let json = r#""""#;
            let result: Result<DashboardTitle, _> = serde_json::from_str(json);
            assert!(result.is_err());
        }
    }

    mod tab_title {
        use super::*;

        #[test]
        fn accepts_valid_title() {
            let title = TabTitle::new("Overview").unwrap();
            assert_eq!(title.as_str(), "Overview");
        }

        #[test]
        fn trims_whitespace() {
            let title = TabTitle::new("  Details  ").unwrap();
            assert_eq!(title.as_str(), "Details");
        }

        #[test]
        fn rejects_empty_string() {
            let result = TabTitle::new("");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.kind(),
                ValidationErrorKind::TooShort {
                    min_length: 1,
                    actual_length: 0,
                    ..
                }
            ));
        }

        #[test]
        fn rejects_too_long() {
            let long_title = "a".repeat(TAB_TITLE_MAX_LENGTH + 1);
            let result = TabTitle::new(&long_title);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.kind(),
                ValidationErrorKind::TooLong {
                    max_length: 100,
                    actual_length: 101,
                    ..
                }
            ));
        }

        #[test]
        fn accepts_max_length() {
            let max_title = "a".repeat(TAB_TITLE_MAX_LENGTH);
            let result = TabTitle::new(&max_title);
            assert!(result.is_ok());
        }

        #[test]
        fn serde_roundtrip() {
            let original = TabTitle::new("Test Tab").unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: TabTitle = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn serde_rejects_empty() {
            let json = r#""""#;
            let result: Result<TabTitle, _> = serde_json::from_str(json);
            assert!(result.is_err());
        }
    }

    mod grid_size {
        use super::*;

        #[test]
        fn accepts_valid_size() {
            let size = GridSize::new(4, 3).unwrap();
            assert_eq!(size.width(), 4);
            assert_eq!(size.height(), 3);
        }

        #[test]
        fn accepts_minimum_size() {
            let size = GridSize::new(1, 1).unwrap();
            assert_eq!(size.width(), 1);
            assert_eq!(size.height(), 1);
        }

        #[test]
        fn rejects_zero_width() {
            let result = GridSize::new(0, 3);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.kind(),
                ValidationErrorKind::OutOfRange {
                    field,
                    min: 1,
                    actual: 0,
                    ..
                } if field == "grid_width"
            ));
        }

        #[test]
        fn rejects_zero_height() {
            let result = GridSize::new(4, 0);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.kind(),
                ValidationErrorKind::OutOfRange {
                    field,
                    min: 1,
                    actual: 0,
                    ..
                } if field == "grid_height"
            ));
        }

        #[test]
        fn cell_count_calculation() {
            let size = GridSize::new(4, 3).unwrap();
            assert_eq!(size.cell_count(), 12);
        }

        #[test]
        fn display_format() {
            let size = GridSize::new(4, 3).unwrap();
            assert_eq!(size.to_string(), "4x3");
        }

        #[test]
        fn serde_roundtrip() {
            let original = GridSize::new(4, 3).unwrap();
            let json = serde_json::to_string(&original).unwrap();
            let parsed: GridSize = serde_json::from_str(&json).unwrap();
            assert_eq!(original, parsed);
        }

        #[test]
        fn copy_semantics() {
            let size1 = GridSize::new(4, 3).unwrap();
            let size2 = size1; // Copy
            assert_eq!(size1.width(), size2.width());
            assert_eq!(size1.height(), size2.height());
        }
    }
}
