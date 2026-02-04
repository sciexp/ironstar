//! Core domain abstractions for ironstar.
//!
//! This crate contains the foundational types shared across all ironstar
//! bounded context crates: domain traits, error types, validated value
//! objects, error codes, and re-exports from fmodel-rust.
//!
//! Maps to `spec/Core/*` in the Idris2 specification.

pub mod error;
pub mod error_code;
pub mod traits;
pub mod values;

// Re-export core domain traits
pub use error::{DomainError, DomainErrorKind, ValidationError, ValidationErrorKind};
pub use error_code::ErrorCode;
pub use traits::{DeciderType, EventType, IsFinal};
pub use values::BoundedString;

// Re-export fmodel-rust core abstractions.
//
// Domain crates depend only on ironstar-core, not fmodel-rust directly.
// This centralizes version management and aligns with the Idris2 spec
// where Core.Decider, Core.View, and Core.Saga define these abstractions.
pub use fmodel_rust::Identifier;
pub use fmodel_rust::Sum;
pub use fmodel_rust::decider::Decider;
pub use fmodel_rust::saga::Saga;
pub use fmodel_rust::view::View;

// Re-export fmodel-rust computation traits used by application layer
pub use fmodel_rust::decider::EventComputation;
pub use fmodel_rust::view::ViewStateComputation;

// Re-export fmodel-rust aggregate types used for event-sourced wiring
pub use fmodel_rust::aggregate::{EventRepository, EventSourcedAggregate};

// Re-export fmodel-rust test specification for aggregate testing
pub use fmodel_rust::specification::DeciderTestSpecification;
