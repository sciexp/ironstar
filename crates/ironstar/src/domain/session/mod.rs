//! Session domain implementation using fmodel-rust Decider pattern.
//!
//! The Session decider is a pure function that embodies the state machine
//! for managing authentication session lifecycles. It implements the patterns
//! from `spec/Session/Session.idr`.
//!
//! # State Machine
//!
//! ```text
//!                    ┌───────────────┐
//!     Create ───────►│    Active     │◄────────┐
//!                    └───────┬───────┘         │
//!                            │                 │
//!            ┌───────────────┼───────────────┐ │
//!            │               │               │ │
//!        Expire          Invalidate      Refresh
//!            │               │               │ │
//!            ▼               ▼               │ │
//!     ┌──────────┐    ┌─────────────┐        │ │
//!     │ Expired  │    │ Invalidated │────────┘ │
//!     └──────────┘    └─────────────┘          │
//!         (terminal)      (terminal)           │
//!                                              │
//!                     (Refresh extends TTL)────┘
//! ```
//!
//! - `NoSession`: Initial state, no session exists
//! - `Active`: Session is valid and usable
//! - `Expired`: Session TTL exceeded (terminal)
//! - `Invalidated`: Session explicitly terminated (terminal)
//!
//! # Boundary responsibilities
//!
//! The decider is pure; all side effects occur at boundaries:
//! - OAuth callback handling (authentication)
//! - SessionId/UserId generation (UUID creation)
//! - Timestamp injection (clock reads)
//! - TTL enforcement (expiration checks)
//! - SessionMetadata extraction (HTTP request context)
//!
//! # Shared Kernel
//!
//! `UserId` is a Shared Kernel type used by both Session and Workspace
//! bounded contexts. It is defined here and re-exported from `domain/mod.rs`.

pub mod commands;
pub mod decider;
pub mod errors;
pub mod events;
pub mod state;
pub mod values;

// Re-export public types for ergonomic imports
pub use commands::SessionCommand;
pub use decider::{SessionDecider, session_decider};
pub use errors::{SessionError, SessionErrorKind};
pub use events::SessionEvent;
pub use state::{SessionState, SessionStatus};
pub use values::{OAuthProvider, SessionId, SessionMetadata, UserId};
