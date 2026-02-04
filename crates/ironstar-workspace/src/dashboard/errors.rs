//! Domain error types for the Dashboard aggregate.
//!
//! Error types represent business rule violations in the dashboard
//! lifecycle. Infrastructure failures belong in the application layer.

use std::backtrace::Backtrace;
use std::fmt;
use uuid::Uuid;

/// Domain error for the Dashboard aggregate with UUID tracking.
#[derive(Debug)]
pub struct DashboardError {
    id: Uuid,
    kind: DashboardErrorKind,
    backtrace: Backtrace,
}

/// Error variants for the Dashboard decider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DashboardErrorKind {
    /// Dashboard already exists with this ID.
    AlreadyExists,

    /// Dashboard does not exist (must create first).
    NotFound,

    /// Tab not found in this dashboard.
    TabNotFound,

    /// Chart not found in this dashboard.
    ChartNotFound,
}

impl DashboardError {
    /// Creates a new error with a generated UUID and captured backtrace.
    pub fn new(kind: DashboardErrorKind) -> Self {
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
    pub fn kind(&self) -> &DashboardErrorKind {
        &self.kind
    }

    /// Returns a reference to the captured backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    /// Create a new error with an explicit UUID, preserving error tracking.
    #[must_use]
    pub fn with_id(id: Uuid, kind: DashboardErrorKind) -> Self {
        Self {
            id,
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    pub fn already_exists() -> Self {
        Self::new(DashboardErrorKind::AlreadyExists)
    }

    pub fn not_found() -> Self {
        Self::new(DashboardErrorKind::NotFound)
    }

    pub fn tab_not_found() -> Self {
        Self::new(DashboardErrorKind::TabNotFound)
    }

    pub fn chart_not_found() -> Self {
        Self::new(DashboardErrorKind::ChartNotFound)
    }
}

impl fmt::Display for DashboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            DashboardErrorKind::AlreadyExists => {
                write!(f, "dashboard already exists")
            }
            DashboardErrorKind::NotFound => {
                write!(f, "dashboard not found")
            }
            DashboardErrorKind::TabNotFound => {
                write!(f, "tab not found in dashboard")
            }
            DashboardErrorKind::ChartNotFound => {
                write!(f, "chart not found in dashboard")
            }
        }
    }
}

impl PartialEq for DashboardError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for DashboardError {}

impl std::error::Error for DashboardError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(
            DashboardError::already_exists().to_string(),
            "dashboard already exists"
        );
        assert_eq!(
            DashboardError::not_found().to_string(),
            "dashboard not found"
        );
        assert_eq!(
            DashboardError::tab_not_found().to_string(),
            "tab not found in dashboard"
        );
        assert_eq!(
            DashboardError::chart_not_found().to_string(),
            "chart not found in dashboard"
        );
    }

    #[test]
    fn error_has_unique_uuid() {
        let err1 = DashboardError::not_found();
        let err2 = DashboardError::not_found();
        assert_ne!(err1.error_id(), err2.error_id());
    }

    #[test]
    fn error_kind_accessible() {
        let err = DashboardError::tab_not_found();
        assert_eq!(err.kind(), &DashboardErrorKind::TabNotFound);
    }
}
