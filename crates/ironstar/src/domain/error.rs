//! Domain layer error types with UUID tracking for distributed tracing.
//!
//! These types are defined in `ironstar-core` and re-exported here for
//! backward compatibility with existing import paths.

pub use ironstar_core::error::{
    DomainError, DomainErrorKind, ValidationError, ValidationErrorKind,
};
