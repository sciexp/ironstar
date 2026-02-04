//! Catalog Decider module.
//!
//! The Catalog Decider manages the lifecycle of DuckLake catalog selections.

mod commands;
mod decider;
pub mod errors;
mod events;
mod state;
pub mod values;

// Re-export all public types
pub use commands::CatalogCommand;
pub use decider::{CatalogDecider, catalog_decider};
pub use errors::{CatalogError, CatalogErrorKind};
pub use events::CatalogEvent;
pub use state::CatalogState;
pub use values::{CATALOG_REF_MAX_LENGTH, CatalogMetadata, CatalogRef, DatasetInfo};
