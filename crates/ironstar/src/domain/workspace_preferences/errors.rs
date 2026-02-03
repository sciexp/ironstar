//! Domain error types for the WorkspacePreferences aggregate.
//!
//! Error types represent business rule violations in the workspace preferences
//! lifecycle. Infrastructure failures belong in the application layer.

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Domain error for the WorkspacePreferences aggregate with UUID tracking.
#[derive(Debug)]
pub struct WorkspacePreferencesError {
    id: Uuid,
    kind: WorkspacePreferencesErrorKind,
    backtrace: Backtrace,
}

/// Error variants for the WorkspacePreferences decider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspacePreferencesErrorKind {
    /// Preferences already initialized for this workspace.
    AlreadyInitialized,

    /// Preferences not yet initialized (must initialize first).
    NotInitialized,

    /// Catalog URI is empty.
    EmptyCatalogUri,

    /// Catalog URI exceeds maximum length.
    CatalogUriTooLong { max: usize, actual: usize },
}

impl WorkspacePreferencesError {
    /// Creates a new error with a generated UUID and captured backtrace.
    pub fn new(kind: WorkspacePreferencesErrorKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    /// Returns the unique error identifier for distributed tracing.
    pub fn error_id(&self) -> Uuid {
        self.id
    }

    /// Returns a reference to the error kind.
    pub fn kind(&self) -> &WorkspacePreferencesErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    pub fn already_initialized() -> Self {
        Self::new(WorkspacePreferencesErrorKind::AlreadyInitialized)
    }

    pub fn not_initialized() -> Self {
        Self::new(WorkspacePreferencesErrorKind::NotInitialized)
    }

    pub fn empty_catalog_uri() -> Self {
        Self::new(WorkspacePreferencesErrorKind::EmptyCatalogUri)
    }

    pub fn catalog_uri_too_long(max: usize, actual: usize) -> Self {
        Self::new(WorkspacePreferencesErrorKind::CatalogUriTooLong { max, actual })
    }
}

impl fmt::Display for WorkspacePreferencesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            WorkspacePreferencesErrorKind::AlreadyInitialized => {
                write!(f, "workspace preferences already initialized")
            }
            WorkspacePreferencesErrorKind::NotInitialized => {
                write!(f, "workspace preferences not initialized")
            }
            WorkspacePreferencesErrorKind::EmptyCatalogUri => {
                write!(f, "catalog URI cannot be empty")
            }
            WorkspacePreferencesErrorKind::CatalogUriTooLong { max, actual } => {
                write!(
                    f,
                    "catalog URI cannot exceed {max} characters (got {actual})"
                )
            }
        }
    }
}

impl PartialEq for WorkspacePreferencesError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for WorkspacePreferencesError {}

impl std::error::Error for WorkspacePreferencesError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(
            WorkspacePreferencesError::already_initialized().to_string(),
            "workspace preferences already initialized"
        );
        assert_eq!(
            WorkspacePreferencesError::not_initialized().to_string(),
            "workspace preferences not initialized"
        );
        assert_eq!(
            WorkspacePreferencesError::empty_catalog_uri().to_string(),
            "catalog URI cannot be empty"
        );
        assert_eq!(
            WorkspacePreferencesError::catalog_uri_too_long(512, 600).to_string(),
            "catalog URI cannot exceed 512 characters (got 600)"
        );
    }

    #[test]
    fn error_has_unique_uuid() {
        let err1 = WorkspacePreferencesError::not_initialized();
        let err2 = WorkspacePreferencesError::not_initialized();
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_kind_accessible() {
        let err = WorkspacePreferencesError::catalog_uri_too_long(512, 600);
        assert_eq!(
            err.kind(),
            &WorkspacePreferencesErrorKind::CatalogUriTooLong {
                max: 512,
                actual: 600
            }
        );
    }
}
