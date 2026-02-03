//! Catalog Decider module.
//!
//! The Catalog Decider manages the lifecycle of DuckLake catalog selections.
//! It uses the fmodel-rust Decider pattern with a simple two-state machine:
//!
//! ```text
//!   NoCatalogSelected ──SelectCatalog──► CatalogActive
//!                                           │
//!                                    RefreshMetadata
//!                                           │
//!                                           ▼
//!                                      CatalogActive (updated metadata)
//! ```
//!
//! # Module organization
//!
//! - `decider`: Pure Catalog Decider (fmodel-rust pattern)
//! - `commands`: CatalogCommand enum
//! - `errors`: CatalogError enum with factory methods
//! - `events`: CatalogEvent enum
//! - `state`: CatalogState enum (NoCatalogSelected | CatalogActive)
//! - `values`: CatalogRef, DatasetInfo, CatalogMetadata value objects

mod commands;
mod decider;
mod errors;
mod events;
mod state;
mod values;

// Re-export all public types
pub use commands::CatalogCommand;
pub use decider::{CatalogDecider, catalog_decider};
pub use errors::{CatalogError, CatalogErrorKind};
pub use events::CatalogEvent;
pub use state::CatalogState;
pub use values::{CATALOG_REF_MAX_LENGTH, CatalogMetadata, CatalogRef, DatasetInfo};
