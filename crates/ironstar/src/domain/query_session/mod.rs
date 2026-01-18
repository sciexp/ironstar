//! QuerySession Decider module.
//!
//! The QuerySession Decider manages the lifecycle of an analytics query session
//! using the fmodel-rust Decider pattern. Unlike the Todo aggregate (which is
//! synchronous end-to-end), QuerySession demonstrates the **spawn-after-persist**
//! pattern where long-running async work (DuckDB query execution) happens AFTER
//! event persistence.
//!
//! # Module organization
//!
//! - `decider`: Pure QuerySession Decider (fmodel-rust pattern)
//! - `commands`: QuerySessionCommand enum with timestamp fields
//! - `errors`: QuerySessionError enum with factory methods
//! - `events`: QuerySessionEvent enum
//! - `state`: QuerySessionState and QuerySessionStatus
//!
//! # Public API
//!
//! All public types are re-exported at the module level for convenience:
//!
//! ```no_run
//! use ironstar::domain::query_session::{
//!     QuerySessionDecider,
//!     query_session_decider,
//!     QuerySessionCommand,
//!     QuerySessionError,
//!     QuerySessionEvent,
//!     QuerySessionState,
//!     QuerySessionStatus,
//! };
//! ```

mod commands;
mod decider;
mod errors;
mod events;
mod state;

// Re-export all public types
pub use commands::QuerySessionCommand;
pub use decider::{QuerySessionDecider, query_session_decider};
pub use errors::{QuerySessionError, QuerySessionErrorKind};
pub use events::QuerySessionEvent;
pub use state::{QuerySessionState, QuerySessionStatus};
