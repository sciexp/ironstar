//! Catalog aggregate application layer.
//!
//! This module wires the Catalog Decider to the SQLite event repository,
//! providing command handling for the DuckLake catalog lifecycle.
//!
//! Unlike QuerySession, the Catalog aggregate has no spawn-after-persist
//! pattern since all catalog operations are synchronous from the Decider's
//! perspective.

mod handlers;
pub mod queries;

pub use handlers::{handle_catalog_command, handle_catalog_command_zenoh};
pub use queries::{query_catalog_metadata, query_catalog_state};
