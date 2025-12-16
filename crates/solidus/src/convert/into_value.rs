//! Conversion from Rust types to Ruby values.

use crate::value::Value;

/// Convert a Rust type to a Ruby Value.
///
/// This trait is the primary way to convert Rust values into Ruby objects.
/// Unlike [`TryConvert`](super::TryConvert), this conversion is infallible.
///
/// # Example
///
/// ```ignore
/// use solidus::prelude::*;
///
/// fn example() -> Value {
///     let n: i64 = 42;
///     n.into_value()
/// }
/// ```
///
/// # Implementors
///
/// This trait is implemented for:
/// - All Ruby wrapper types (`RString`, `RArray`, etc.)
/// - Rust primitives (`i8`-`i64`, `u8`-`u64`, `f32`, `f64`, `bool`)
/// - `String`, `&str`, `Vec<T>`, etc.
pub trait IntoValue {
    /// Convert self into a Ruby Value.
    fn into_value(self) -> Value;
}

// Implement IntoValue for Value itself (identity conversion)
impl IntoValue for Value {
    #[inline]
    fn into_value(self) -> Value {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_into_value() {
        let val = Value::nil();
        let result = val.into_value();
        assert_eq!(val.as_raw(), result.as_raw());
    }
}
