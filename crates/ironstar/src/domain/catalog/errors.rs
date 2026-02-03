//! Domain errors for the Catalog aggregate.
//!
//! These represent business rule violations during command processing.
//!
//! # Design principles
//!
//! - **UUID tracking**: Every error instance gets a unique ID for distributed tracing
//! - **Backtrace capture**: Backtraces are captured at error creation for debugging
//! - **Kind enum for pattern matching**: The `CatalogErrorKind` enum enables
//!   exhaustive matching without losing UUID/backtrace context

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Domain error for the Catalog aggregate with UUID tracking.
#[derive(Debug)]
pub struct CatalogError {
    id: Uuid,
    kind: CatalogErrorKind,
    backtrace: Backtrace,
}

/// Error variants for the Catalog aggregate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogErrorKind {
    /// Catalog reference URI cannot be empty.
    EmptyRef,

    /// Catalog reference URI exceeds maximum length.
    RefTooLong { max: usize, actual: usize },

    /// Cannot change to a different catalog while one is active.
    CatalogAlreadyActive,

    /// Cannot refresh metadata when no catalog is selected.
    NoCatalogSelected,
}

impl CatalogError {
    /// Creates a new error with a generated UUID and captured backtrace.
    pub fn new(kind: CatalogErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates an error with a specific UUID, preserving the original error ID
    /// across error mapping boundaries.
    pub fn with_id(id: Uuid, kind: CatalogErrorKind) -> Self {
        Self {
            id,
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    /// Returns the unique error identifier for distributed tracing.
    pub fn error_id(&self) -> Uuid {
        self.id
    }

    /// Returns a reference to the error kind.
    pub fn kind(&self) -> &CatalogErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    // Convenience constructors

    /// Creates an `EmptyRef` error.
    pub fn empty_ref() -> Self {
        Self::new(CatalogErrorKind::EmptyRef)
    }

    /// Creates a `RefTooLong` error.
    pub fn ref_too_long(max: usize, actual: usize) -> Self {
        Self::new(CatalogErrorKind::RefTooLong { max, actual })
    }

    /// Creates a `CatalogAlreadyActive` error.
    pub fn catalog_already_active() -> Self {
        Self::new(CatalogErrorKind::CatalogAlreadyActive)
    }

    /// Creates a `NoCatalogSelected` error.
    pub fn no_catalog_selected() -> Self {
        Self::new(CatalogErrorKind::NoCatalogSelected)
    }
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            CatalogErrorKind::EmptyRef => write!(f, "catalog reference cannot be empty"),
            CatalogErrorKind::RefTooLong { max, actual } => {
                write!(
                    f,
                    "catalog reference cannot exceed {max} characters (got {actual})"
                )
            }
            CatalogErrorKind::CatalogAlreadyActive => {
                write!(f, "cannot change active catalog; deselect first")
            }
            CatalogErrorKind::NoCatalogSelected => write!(f, "no catalog selected"),
        }
    }
}

impl std::error::Error for CatalogError {}

/// PartialEq compares errors by kind only, ignoring UUID and backtrace.
/// This enables testing with DeciderTestSpecification::then_error().
impl PartialEq for CatalogError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_has_unique_uuid() {
        let err1 = CatalogError::no_catalog_selected();
        let err2 = CatalogError::no_catalog_selected();
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_display_messages() {
        assert_eq!(
            CatalogError::empty_ref().to_string(),
            "catalog reference cannot be empty"
        );
        assert_eq!(
            CatalogError::catalog_already_active().to_string(),
            "cannot change active catalog; deselect first"
        );
        assert_eq!(
            CatalogError::no_catalog_selected().to_string(),
            "no catalog selected"
        );
    }

    #[test]
    fn error_kind_accessible() {
        let err = CatalogError::ref_too_long(1024, 2000);
        assert_eq!(
            err.kind(),
            &CatalogErrorKind::RefTooLong {
                max: 1024,
                actual: 2000
            }
        );
    }
}
