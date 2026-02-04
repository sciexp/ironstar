//! Generic validated value objects with smart constructors.
//!
//! Value objects are immutable, equality-compared by value (not identity),
//! and validated at construction time. This module implements the "parse,
//! don't validate" principle: if you have a value object, it is guaranteed
//! to satisfy its invariants.

use crate::error::{ValidationError, ValidationErrorKind};
use std::fmt;
use std::marker::PhantomData;

/// A string with compile-time length bounds.
///
/// Guarantees:
/// - Non-empty (at least `MIN` non-whitespace characters)
/// - At most `MAX` characters
/// - Trimmed of leading/trailing whitespace
///
/// Uses const generics to encode length constraints in the type system.
/// This allows different string types with different bounds to share
/// implementation while remaining distinct types.
///
/// # Example
///
/// ```rust,ignore
/// // Define a custom bounded string type
/// type ShortName = BoundedString<1, 50>;
///
/// let name = ShortName::new("  Alice  ")?;
/// assert_eq!(name.as_str(), "Alice"); // Trimmed
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BoundedString<const MIN: usize, const MAX: usize> {
    value: String,
    /// PhantomData to use the const generics in the type signature
    _marker: PhantomData<()>,
}

impl<const MIN: usize, const MAX: usize> BoundedString<MIN, MAX> {
    /// Create a new BoundedString, validating and normalizing the input.
    ///
    /// The input is trimmed before validation. The character count
    /// (not byte count) is used for length checks.
    ///
    /// # Errors
    ///
    /// - [`ValidationError`] with `TooShort` if trimmed length < MIN
    /// - [`ValidationError`] with `TooLong` if trimmed length > MAX
    pub fn new(value: impl Into<String>, field_name: &str) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        let char_count = trimmed.chars().count();

        if char_count < MIN {
            return Err(ValidationError::new(ValidationErrorKind::TooShort {
                field: field_name.to_string(),
                min_length: MIN,
                actual_length: char_count,
            }));
        }

        if char_count > MAX {
            return Err(ValidationError::new(ValidationErrorKind::TooLong {
                field: field_name.to_string(),
                max_length: MAX,
                actual_length: char_count,
            }));
        }

        Ok(Self {
            value: trimmed.to_string(),
            _marker: PhantomData,
        })
    }

    /// Get the string as a slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Consume self and return the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.value
    }

    /// Get the minimum allowed length.
    #[must_use]
    pub const fn min_length() -> usize {
        MIN
    }

    /// Get the maximum allowed length.
    #[must_use]
    pub const fn max_length() -> usize {
        MAX
    }
}

impl<const MIN: usize, const MAX: usize> fmt::Debug for BoundedString<MIN, MAX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoundedString")
            .field("value", &self.value)
            .field("min", &MIN)
            .field("max", &MAX)
            .finish()
    }
}

impl<const MIN: usize, const MAX: usize> fmt::Display for BoundedString<MIN, MAX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<const MIN: usize, const MAX: usize> AsRef<str> for BoundedString<MIN, MAX> {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ValidationErrorKind;

    mod bounded_string {
        use super::*;

        #[test]
        fn accepts_valid_string() {
            let s: BoundedString<1, 100> = BoundedString::new("hello", "test").unwrap();
            assert_eq!(s.as_str(), "hello");
        }

        #[test]
        fn trims_whitespace() {
            let s: BoundedString<1, 100> = BoundedString::new("  hello  ", "test").unwrap();
            assert_eq!(s.as_str(), "hello");
        }

        #[test]
        fn rejects_too_short() {
            let result: Result<BoundedString<5, 100>, _> = BoundedString::new("abc", "test");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.kind(),
                ValidationErrorKind::TooShort {
                    min_length: 5,
                    actual_length: 3,
                    ..
                }
            ));
        }

        #[test]
        fn rejects_too_long() {
            let result: Result<BoundedString<1, 5>, _> = BoundedString::new("hello world", "test");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.kind(),
                ValidationErrorKind::TooLong {
                    max_length: 5,
                    actual_length: 11,
                    ..
                }
            ));
        }

        #[test]
        fn rejects_empty_when_min_is_positive() {
            let result: Result<BoundedString<1, 100>, _> = BoundedString::new("", "test");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.kind(),
                ValidationErrorKind::TooShort {
                    min_length: 1,
                    actual_length: 0,
                    ..
                }
            ));
        }

        #[test]
        fn rejects_whitespace_only() {
            let result: Result<BoundedString<1, 100>, _> = BoundedString::new("   \t\n  ", "test");
            assert!(result.is_err());
        }

        #[test]
        fn accepts_exact_min_length() {
            let result: Result<BoundedString<5, 100>, _> = BoundedString::new("hello", "test");
            assert!(result.is_ok());
        }

        #[test]
        fn accepts_exact_max_length() {
            let result: Result<BoundedString<1, 5>, _> = BoundedString::new("hello", "test");
            assert!(result.is_ok());
        }

        #[test]
        fn const_accessors_work() {
            assert_eq!(BoundedString::<1, 100>::min_length(), 1);
            assert_eq!(BoundedString::<1, 100>::max_length(), 100);
        }
    }
}
