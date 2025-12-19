//! Return value handling for Ruby methods.
//!
//! This module provides the [`IntoReturnValue`] trait which handles converting
//! various return types to raw Ruby VALUES for FFI return.

use std::pin::Pin;

use crate::error::Error;
use crate::value::{BoxValue, ReprValue, StackPinned, Value};

/// Trait for types that can be returned from Ruby methods.
///
/// This trait handles converting various return types to raw Ruby VALUES:
/// - `Pin<&StackPinned<T>>` - stack-pinned values from Context
/// - `BoxValue<T>` - heap-boxed values  
/// - `i64`, `i32`, `usize`, `bool`, `()` - immediate/primitive values
/// - `Value` - raw values
///
/// The `method!` and `function!` macros use this trait to convert user
/// function return values to raw VALUEs for Ruby.
///
/// # Example
///
/// ```ignore
/// use solidus::method::IntoReturnValue;
///
/// // All these types implement IntoReturnValue:
/// fn returns_pinned<'ctx>(ctx: &'ctx Context) -> Result<Pin<&'ctx StackPinned<RString>>, Error>;
/// fn returns_boxed(ctx: &Context) -> Result<BoxValue<RString>, Error>;
/// fn returns_i64(ctx: &Context) -> Result<i64, Error>;
/// fn returns_bool(ctx: &Context) -> Result<bool, Error>;
/// fn returns_unit(ctx: &Context) -> Result<(), Error>;
/// ```
pub trait IntoReturnValue {
    /// Convert this value into a raw Ruby VALUE for FFI return.
    ///
    /// Returns `Ok(VALUE)` on success, or `Err(Error)` if conversion failed.
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error>;
}

// ============================================================================
// NewValue - guarded values that can be returned directly
// ============================================================================

impl<T: ReprValue> IntoReturnValue for crate::value::NewValue<T> {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        Ok(self.as_raw())
    }
}

// ============================================================================
// Pinned values from Context
// ============================================================================

impl<T: ReprValue> IntoReturnValue for Pin<&StackPinned<T>> {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        Ok(self.get().as_raw())
    }
}

// ============================================================================
// Boxed values
// ============================================================================

impl<T: ReprValue> IntoReturnValue for BoxValue<T> {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        Ok(self.as_raw())
    }
}

// Also allow returning references to BoxValue
impl<T: ReprValue> IntoReturnValue for &BoxValue<T> {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        Ok(self.as_raw())
    }
}

// ============================================================================
// Raw Value
// ============================================================================

impl IntoReturnValue for Value {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        Ok(self.as_raw())
    }
}

impl IntoReturnValue for &Value {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        Ok(self.as_raw())
    }
}

// ============================================================================
// Integer types
// ============================================================================

impl IntoReturnValue for i64 {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        // SAFETY: rb_int2inum is always safe to call
        // Note: Ruby C API uses isize, so we cast (this is safe on 64-bit platforms)
        Ok(unsafe { rb_sys::rb_int2inum(self as isize) })
    }
}

impl IntoReturnValue for i32 {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        (self as i64).into_return_value()
    }
}

impl IntoReturnValue for i16 {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        (self as i64).into_return_value()
    }
}

impl IntoReturnValue for i8 {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        (self as i64).into_return_value()
    }
}

impl IntoReturnValue for isize {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        (self as i64).into_return_value()
    }
}

impl IntoReturnValue for u64 {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        // SAFETY: rb_uint2inum is always safe to call
        // Note: Ruby C API uses usize, so we cast (this is safe on 64-bit platforms)
        Ok(unsafe { rb_sys::rb_uint2inum(self as usize) })
    }
}

impl IntoReturnValue for u32 {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        (self as u64).into_return_value()
    }
}

impl IntoReturnValue for u16 {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        (self as u64).into_return_value()
    }
}

impl IntoReturnValue for u8 {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        (self as u64).into_return_value()
    }
}

impl IntoReturnValue for usize {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        (self as u64).into_return_value()
    }
}

// ============================================================================
// Boolean
// ============================================================================

impl IntoReturnValue for bool {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        if self {
            Ok(rb_sys::Qtrue as rb_sys::VALUE)
        } else {
            Ok(rb_sys::Qfalse as rb_sys::VALUE)
        }
    }
}

// ============================================================================
// Floating point types
// ============================================================================

impl IntoReturnValue for f64 {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        // SAFETY: rb_float_new is always safe to call
        Ok(unsafe { rb_sys::rb_float_new(self) })
    }
}

impl IntoReturnValue for f32 {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        (self as f64).into_return_value()
    }
}

// ============================================================================
// Unit (returns nil)
// ============================================================================

impl IntoReturnValue for () {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        Ok(rb_sys::Qnil as rb_sys::VALUE)
    }
}

// ============================================================================
// Result wrapper - allows using ? operator in methods
// ============================================================================

impl<T: IntoReturnValue> IntoReturnValue for Result<T, Error> {
    #[inline]
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        self.and_then(|v| v.into_return_value())
    }
}

// ============================================================================
// Backward compatibility - keep ReturnValue as an alias
// ============================================================================

/// Deprecated: Use [`IntoReturnValue`] instead.
///
/// This trait is kept for backward compatibility during migration.
#[deprecated(since = "0.2.0", note = "Use IntoReturnValue instead")]
pub trait ReturnValue {
    /// Convert this value into a Ruby return value.
    fn into_return_value(self) -> Result<Value, Error>;
}

#[allow(deprecated)]
impl<T> ReturnValue for Result<T, Error>
where
    T: crate::convert::IntoValue,
{
    #[inline]
    fn into_return_value(self) -> Result<Value, Error> {
        self.map(|v| v.into_value())
    }
}

#[cfg(test)]
mod tests {
    use super::IntoReturnValue;
    use crate::error::Error;
    use crate::value::Value;

    #[test]
    fn test_i64_return_value() {
        let result = 42i64.into_return_value();
        assert!(result.is_ok());
    }

    #[test]
    fn test_i32_return_value() {
        let result = 42i32.into_return_value();
        assert!(result.is_ok());
    }

    #[test]
    fn test_bool_true_return_value() {
        let result = true.into_return_value().unwrap();
        assert_eq!(result, rb_sys::Qtrue as rb_sys::VALUE);
    }

    #[test]
    fn test_bool_false_return_value() {
        let result = false.into_return_value().unwrap();
        assert_eq!(result, rb_sys::Qfalse as rb_sys::VALUE);
    }

    #[test]
    fn test_unit_return_value() {
        let result = ().into_return_value().unwrap();
        assert_eq!(result, rb_sys::Qnil as rb_sys::VALUE);
    }

    #[test]
    fn test_result_ok() {
        let result: Result<i64, Error> = Ok(42);
        let value = result.into_return_value();
        assert!(value.is_ok());
    }

    #[test]
    fn test_result_err() {
        let result: Result<i64, Error> = Err(Error::type_error("test error"));
        let err = result.into_return_value();
        assert!(err.is_err());
    }

    #[test]
    fn test_value_return() {
        // Test that Value can be returned
        let val = unsafe { Value::from_raw(rb_sys::Qnil as rb_sys::VALUE) };
        let result = val.into_return_value().unwrap();
        assert_eq!(result, rb_sys::Qnil as rb_sys::VALUE);
    }
}
