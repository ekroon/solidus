//! Conversion from Ruby values to Rust types.

use crate::error::Error;
use crate::value::Value;

/// Convert a Ruby Value to a Rust type.
///
/// This trait is the primary way to extract Rust values from Ruby objects.
/// It performs type checking and returns an error if the conversion fails.
///
/// # Example
///
/// ```no_run
/// use solidus::prelude::*;
/// use solidus::convert::TryConvert;
///
/// fn example(val: Value) -> Result<i64, Error> {
///     i64::try_convert(val)
/// }
/// ```
///
/// # Implementors
///
/// This trait is implemented for:
/// - All Ruby wrapper types (`RString`, `RArray`, etc.)
/// - Rust primitives (`i8`-`i64`, `u8`-`u64`, `f32`, `f64`, `bool`)
/// - `String`, `Option<T>`, `Vec<T>`, etc.
pub trait TryConvert: Sized {
    /// Try to convert a Ruby Value to this type.
    ///
    /// Returns an error if the value cannot be converted (e.g., wrong type).
    fn try_convert(val: Value) -> Result<Self, Error>;
}

/// Marker trait for types that can be converted without type checking.
///
/// This is implemented by Ruby wrapper types where the conversion from
/// `Value` is just a transmute (e.g., `RString`, `RArray`).
///
/// # Safety
///
/// Implementors must ensure that `try_convert` only succeeds when the
/// value is actually of the implementing type.
pub trait TryConvertOwned: TryConvert {}

// Implement TryConvert for Value itself (always succeeds)
impl TryConvert for Value {
    #[inline]
    fn try_convert(val: Value) -> Result<Self, Error> {
        Ok(val)
    }
}

impl TryConvertOwned for Value {}
