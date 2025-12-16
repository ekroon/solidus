//! Traits for types that represent Ruby values.

use super::Value;

/// Trait for types that wrap a Ruby VALUE.
///
/// This trait is implemented by all Solidus types that represent Ruby values
/// (e.g., `RString`, `RArray`, `RHash`, etc.). It provides the foundation
/// for converting between typed wrappers and the base `Value` type.
///
/// # Safety
///
/// Implementors must ensure that:
/// - `as_value()` returns the correct underlying VALUE
/// - `from_value_unchecked()` creates a valid instance when given a VALUE of the correct type
///
/// # Example
///
/// ```ignore
/// use solidus::value::{Value, ReprValue};
///
/// // All Ruby type wrappers implement ReprValue
/// let string: RString = /* ... */;
/// let value: Value = string.as_value();
///
/// // Convert back (unchecked)
/// let string_again: RString = unsafe { RString::from_value_unchecked(value) };
/// ```
pub trait ReprValue: Copy {
    /// Get this value as a base Value.
    fn as_value(self) -> Value;

    /// Create from a Value without type checking.
    ///
    /// # Safety
    ///
    /// The value must actually be of the implementing type.
    /// Calling this with a VALUE of the wrong type leads to undefined behavior.
    unsafe fn from_value_unchecked(val: Value) -> Self;

    /// Get the raw Ruby VALUE.
    ///
    /// This is a convenience method that calls `as_value().as_raw()`.
    #[inline]
    fn as_raw(self) -> rb_sys::VALUE {
        self.as_value().as_raw()
    }

    /// Check if this value is nil.
    #[inline]
    fn is_nil(self) -> bool {
        self.as_value().is_nil()
    }

    /// Check if this value is truthy (not nil or false).
    #[inline]
    fn is_truthy(self) -> bool {
        self.as_value().is_truthy()
    }
}

// Implement ReprValue for Value itself
impl ReprValue for Value {
    #[inline]
    fn as_value(self) -> Value {
        self
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        val
    }
}
