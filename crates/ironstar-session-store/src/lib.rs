//! SQLite session store infrastructure for ironstar.
//!
//! This crate provides session persistence: `SqliteSessionStore` implementing
//! the `SessionStore` trait, cryptographic session ID generation, and background
//! TTL cleanup. Maps to the infrastructure layer for the Session bounded context.

pub mod error;
pub mod session_store;

pub use error::{SessionStoreError, SessionStoreErrorKind};
pub use session_store::{
    SESSIONS_MIGRATION_SQL, Session, SessionStore, SqliteSessionStore, generate_session_id,
    spawn_session_cleanup,
};
