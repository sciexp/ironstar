//! Session store re-exported from the `ironstar-session-store` crate.
//!
//! This module re-exports all public types from `ironstar_session_store`
//! to maintain backward compatibility with existing import paths.

pub use ironstar_session_store::{
    SESSIONS_MIGRATION_SQL, Session, SessionStore, SessionStoreError, SessionStoreErrorKind,
    SqliteSessionStore, generate_session_id, spawn_session_cleanup,
};
