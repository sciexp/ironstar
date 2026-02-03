//! Combined Analytics Decider composing Catalog and QuerySession.
//!
//! This module implements the Idris spec's `analyticsDecider = combine catalogDecider queryDecider`
//! using fmodel-rust's `Decider::combine` method. The combined decider enables coordinated
//! command routing where a single endpoint can dispatch to either aggregate.
//!
//! The combined types use fmodel-rust's `Sum` type for commands and events,
//! and a tuple for state:
//!
//! - Commands: `Sum<CatalogCommand, QuerySessionCommand>`
//! - State: `(CatalogState, QuerySessionState)`
//! - Events: `Sum<CatalogEvent, QuerySessionEvent>`
//! - Error: `CombinedDeciderError`

use crate::domain::catalog::{
    CatalogCommand, CatalogError, CatalogEvent, CatalogState, catalog_decider,
};
use crate::domain::query_session::{
    QuerySessionCommand, QuerySessionError, QuerySessionEvent, QuerySessionState,
    query_session_decider,
};
use fmodel_rust::Sum;
use fmodel_rust::decider::Decider;
use std::fmt;

/// Unified error type for the combined Analytics Decider.
///
/// Since fmodel-rust's `combine` requires both deciders to share an error type,
/// this enum wraps the individual domain errors from Catalog and QuerySession.
#[derive(Debug)]
pub enum CombinedDeciderError {
    /// Error from the Catalog Decider.
    Catalog(CatalogError),
    /// Error from the QuerySession Decider.
    QuerySession(QuerySessionError),
}

impl fmt::Display for CombinedDeciderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalog(e) => write!(f, "Catalog: {e}"),
            Self::QuerySession(e) => write!(f, "QuerySession: {e}"),
        }
    }
}

impl std::error::Error for CombinedDeciderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Catalog(e) => Some(e),
            Self::QuerySession(e) => Some(e),
        }
    }
}

impl From<CatalogError> for CombinedDeciderError {
    fn from(e: CatalogError) -> Self {
        Self::Catalog(e)
    }
}

impl From<QuerySessionError> for CombinedDeciderError {
    fn from(e: QuerySessionError) -> Self {
        Self::QuerySession(e)
    }
}

/// Type alias for the combined Analytics command type.
pub type AnalyticsCommand = Sum<CatalogCommand, QuerySessionCommand>;

/// Type alias for the combined Analytics state type.
pub type AnalyticsState = (CatalogState, QuerySessionState);

/// Type alias for the combined Analytics event type.
pub type AnalyticsEvent = Sum<CatalogEvent, QuerySessionEvent>;

/// Type alias for the combined Analytics Decider.
pub type AnalyticsDecider<'a> =
    Decider<'a, AnalyticsCommand, AnalyticsState, AnalyticsEvent, CombinedDeciderError>;

/// Create the combined Analytics Decider.
///
/// Composes `catalog_decider` and `query_session_decider` using fmodel-rust's
/// `Decider::combine`. Commands are routed to the appropriate sub-decider via
/// `Sum::First` (Catalog) or `Sum::Second` (QuerySession).
///
/// State is a product type `(CatalogState, QuerySessionState)` where each
/// sub-decider only evolves its own component.
pub fn analytics_decider<'a>() -> AnalyticsDecider<'a> {
    catalog_decider()
        .map_error(|e: &CatalogError| {
            CombinedDeciderError::Catalog(CatalogError::with_id(e.error_id(), e.kind().clone()))
        })
        .combine(query_session_decider().map_error(|e: &QuerySessionError| {
            CombinedDeciderError::QuerySession(QuerySessionError::with_id(
                e.error_id(),
                e.kind().clone(),
            ))
        }))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::domain::analytics::{QueryId, SqlQuery};
    use crate::domain::catalog::{CatalogMetadata, CatalogRef, DatasetInfo};
    use chrono::Utc;

    fn sample_catalog_ref() -> CatalogRef {
        CatalogRef::try_from("ducklake:test".to_string()).expect("valid catalog ref")
    }

    fn sample_metadata() -> CatalogMetadata {
        CatalogMetadata {
            datasets: vec![DatasetInfo {
                name: "ds".to_string(),
                table_count: 1,
                schema_version: "1.0".to_string(),
            }],
            last_refreshed: Utc::now(),
        }
    }

    #[test]
    fn combined_decider_initial_state() {
        let decider = analytics_decider();
        let state = (decider.initial_state)();
        assert_eq!(state.0, CatalogState::default());
        assert_eq!(state.1, QuerySessionState::default());
    }

    #[test]
    fn combined_decider_routes_catalog_command() {
        let decider = analytics_decider();
        let state = (decider.initial_state)();

        let command = AnalyticsCommand::First(CatalogCommand::SelectCatalog {
            catalog_ref: sample_catalog_ref(),
            selected_at: Utc::now(),
        });

        let events = (decider.decide)(&command, &state);
        let events = events.expect("select catalog should succeed");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            Sum::First(CatalogEvent::CatalogSelected { .. })
        ));
    }

    #[test]
    fn combined_decider_routes_query_session_command() {
        let decider = analytics_decider();
        let state = (decider.initial_state)();

        let command = AnalyticsCommand::Second(QuerySessionCommand::StartQuery {
            query_id: QueryId::new(),
            sql: SqlQuery::try_from("SELECT 1".to_string()).expect("valid SQL"),
            dataset_ref: None,
            chart_config: None,
            started_at: Utc::now(),
        });

        let events = (decider.decide)(&command, &state);
        let events = events.expect("start query should succeed");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            Sum::Second(QuerySessionEvent::QueryStarted { .. })
        ));
    }

    #[test]
    fn combined_decider_evolves_catalog_state_independently() {
        let decider = analytics_decider();
        let state = (decider.initial_state)();

        let event = AnalyticsEvent::First(CatalogEvent::CatalogSelected {
            catalog_ref: sample_catalog_ref(),
            selected_at: Utc::now(),
        });

        let new_state = (decider.evolve)(&state, &event);

        // Catalog state should have changed
        assert!(matches!(new_state.0, CatalogState::CatalogActive { .. }));
        // QuerySession state should be unchanged
        assert_eq!(new_state.1, QuerySessionState::default());
    }

    #[test]
    fn combined_decider_evolves_query_session_state_independently() {
        let decider = analytics_decider();
        let state = (decider.initial_state)();

        let query_id = QueryId::new();
        let event = AnalyticsEvent::Second(QuerySessionEvent::QueryStarted {
            query_id,
            sql: SqlQuery::try_from("SELECT 1".to_string()).expect("valid SQL"),
            dataset_ref: None,
            chart_config: None,
            started_at: Utc::now(),
        });

        let new_state = (decider.evolve)(&state, &event);

        // Catalog state should be unchanged
        assert_eq!(new_state.0, CatalogState::default());
        // QuerySession state should have changed
        assert_ne!(new_state.1, QuerySessionState::default());
    }

    #[test]
    fn combined_decider_catalog_error_preserved() {
        let decider = analytics_decider();
        let state = (decider.initial_state)();

        // Refresh metadata without selecting a catalog should fail
        let command = AnalyticsCommand::First(CatalogCommand::RefreshCatalogMetadata {
            metadata: sample_metadata(),
            refreshed_at: Utc::now(),
        });

        let result = (decider.decide)(&command, &state);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CombinedDeciderError::Catalog(_)
        ));
    }
}
