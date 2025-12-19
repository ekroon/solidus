//! Return value handling for Ruby methods.
//!
//! This module provides the [`ReturnValue`] trait which handles converting method
//! return values to Ruby VALUE types. It supports both infallible returns (direct values)
//! and fallible returns (`Result<T, Error>`).

use crate::error::Error;
use crate::value::Value;

/// Trait for types that can be returned from Ruby methods.
///
/// This trait is implemented for `Result<T, Error>` where T implements `IntoValue`.
/// The method registration system uses this trait to handle both successful returns
/// and error propagation.
///
/// # Design Note
///
/// We only implement this for `Result<T, Error>` rather than for all `IntoValue` types
/// to avoid trait coherence conflicts. The method macro will handle wrapping non-Result
/// returns in `Ok()` before calling this trait.
///
/// # Example
///
/// ```no_run
/// use solidus::prelude::*;
/// use solidus::method::ReturnValue;
///
/// fn example() -> Result<PinGuard<RString>, Error> {
///     Ok(RString::new("hello"))
/// }
/// ```
pub trait ReturnValue {
    /// Convert this value into a Ruby return value.
    ///
    /// Returns `Ok(Value)` on success, or `Err(Error)` if an error occurred.
    fn into_return_value(self) -> Result<Value, Error>;
}

// Implement for Result<T, Error> where T can be converted to Value
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
    use super::*;
    use crate::convert::IntoValue;

    #[test]
    fn test_return_value_ok() {
        let result: Result<i64, Error> = Ok(42);
        let value = result.into_return_value().unwrap();
        assert_eq!(value.as_raw(), 42i64.into_value().as_raw());
    }

    #[test]
    fn test_return_value_err() {
        let result: Result<i64, Error> = Err(Error::type_error("test error"));
        let err = result.into_return_value().unwrap_err();
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn test_return_value_bool() {
        let result: Result<bool, Error> = Ok(true);
        let value = result.into_return_value().unwrap();
        assert!(value.is_true());
    }

    #[test]
    fn test_return_value_unit() {
        let result: Result<(), Error> = Ok(());
        let value = result.into_return_value().unwrap();
        assert!(value.is_nil());
    }
}
