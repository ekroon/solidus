//! Ruby immediate singleton types: nil, true, false.
//!
//! These types represent Ruby's singleton values and are immediate values
//! (encoded directly in the VALUE, not allocated on the heap).

use crate::convert::{IntoValue, TryConvert};
use crate::error::Error;
use crate::value::{ReprValue, Value};

/// Ruby `nil` value.
///
/// This is a zero-sized type representing Ruby's singleton `nil` value.
///
/// # Example
///
/// ```ignore
/// use solidus::types::Qnil;
///
/// let nil = Qnil::new();
/// assert!(nil.as_value().is_nil());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Qnil;

impl Qnil {
    /// Create a new nil value.
    #[inline]
    pub fn new() -> Self {
        Qnil
    }
}

impl Default for Qnil {
    fn default() -> Self {
        Qnil::new()
    }
}

impl ReprValue for Qnil {
    #[inline]
    fn as_value(&self) -> Value {
        Value::nil()
    }

    #[inline]
    unsafe fn from_value_unchecked(_val: Value) -> Self {
        Qnil
    }
}

impl TryConvert for Qnil {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if val.is_nil() {
            Ok(Qnil)
        } else {
            Err(Error::type_error("expected nil"))
        }
    }
}

impl IntoValue for Qnil {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

/// Ruby `true` value.
///
/// This is a zero-sized type representing Ruby's singleton `true` value.
///
/// # Example
///
/// ```ignore
/// use solidus::types::Qtrue;
///
/// let t = Qtrue::new();
/// assert!(t.as_value().is_true());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Qtrue;

impl Qtrue {
    /// Create a new true value.
    #[inline]
    pub fn new() -> Self {
        Qtrue
    }
}

impl Default for Qtrue {
    fn default() -> Self {
        Qtrue::new()
    }
}

impl ReprValue for Qtrue {
    #[inline]
    fn as_value(&self) -> Value {
        Value::r#true()
    }

    #[inline]
    unsafe fn from_value_unchecked(_val: Value) -> Self {
        Qtrue
    }
}

impl TryConvert for Qtrue {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if val.is_true() {
            Ok(Qtrue)
        } else {
            Err(Error::type_error("expected true"))
        }
    }
}

impl IntoValue for Qtrue {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

/// Ruby `false` value.
///
/// This is a zero-sized type representing Ruby's singleton `false` value.
///
/// # Example
///
/// ```ignore
/// use solidus::types::Qfalse;
///
/// let f = Qfalse::new();
/// assert!(f.as_value().is_false());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Qfalse;

impl Qfalse {
    /// Create a new false value.
    #[inline]
    pub fn new() -> Self {
        Qfalse
    }
}

impl Default for Qfalse {
    fn default() -> Self {
        Qfalse::new()
    }
}

impl ReprValue for Qfalse {
    #[inline]
    fn as_value(&self) -> Value {
        Value::r#false()
    }

    #[inline]
    unsafe fn from_value_unchecked(_val: Value) -> Self {
        Qfalse
    }
}

impl TryConvert for Qfalse {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if val.is_false() {
            Ok(Qfalse)
        } else {
            Err(Error::type_error("expected false"))
        }
    }
}

impl IntoValue for Qfalse {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

/// Convert Rust bool to Ruby true/false.
///
/// This allows using Rust booleans directly with Ruby.
impl IntoValue for bool {
    #[inline]
    fn into_value(self) -> Value {
        if self {
            Qtrue.into_value()
        } else {
            Qfalse.into_value()
        }
    }
}

/// Convert Ruby value to Rust bool.
///
/// In Ruby, only `nil` and `false` are falsy; everything else is truthy.
impl TryConvert for bool {
    fn try_convert(val: Value) -> Result<Self, Error> {
        Ok(!val.is_nil() && !val.is_false())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qnil() {
        let nil = Qnil::new();
        let val = nil.as_value();
        assert!(val.is_nil());
        assert_eq!(Qnil::try_convert(val).unwrap(), nil);
    }

    #[test]
    fn test_qtrue() {
        let t = Qtrue::new();
        let val = t.as_value();
        assert!(val.is_true());
        assert_eq!(Qtrue::try_convert(val).unwrap(), t);
    }

    #[test]
    fn test_qfalse() {
        let f = Qfalse::new();
        let val = f.as_value();
        assert!(val.is_false());
        assert_eq!(Qfalse::try_convert(val).unwrap(), f);
    }

    #[test]
    fn test_bool_conversion() {
        assert!(true.into_value().is_true());
        assert!(false.into_value().is_false());
    }

    #[test]
    fn test_bool_truthiness() {
        // Only nil and false are falsy
        assert!(!bool::try_convert(Value::nil()).unwrap());
        assert!(!bool::try_convert(Value::r#false()).unwrap());

        // Everything else is truthy
        assert!(bool::try_convert(Value::r#true()).unwrap());
    }
}
