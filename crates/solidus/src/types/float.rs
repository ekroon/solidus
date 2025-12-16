//! Ruby Float types: Flonum (immediate, 64-bit only) and RFloat (heap).

use crate::convert::{IntoValue, TryConvert};
use crate::error::Error;
use crate::value::{ReprValue, Value, ValueType};

/// Immediate float value (only on 64-bit platforms).
///
/// On 64-bit platforms, Ruby can encode small floats directly in the VALUE
/// (called Flonum). This is an immediate value that doesn't require GC protection.
///
/// Note: This type is only available on 64-bit platforms. On 32-bit platforms,
/// all floats are heap-allocated.
///
/// # Example
///
/// ```ignore
/// use solidus::types::Flonum;
///
/// let num = Flonum::from_f64(3.14).expect("3.14 can be a Flonum");
/// assert!((num.to_f64() - 3.14).abs() < 0.001);
/// ```
#[cfg(target_pointer_width = "64")]
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Flonum(Value);

#[cfg(target_pointer_width = "64")]
impl Flonum {
    /// Create a Flonum from an f64.
    ///
    /// Returns `None` if the value cannot be represented as a Flonum
    /// (requires heap allocation as RFloat instead).
    pub fn from_f64(n: f64) -> Option<Self> {
        // SAFETY: rb_float_new creates a Float VALUE
        let val = unsafe { Value::from_raw(rb_sys::rb_float_new(n)) };
        
        // Check if it's actually a Flonum (not a heap float)
        if rb_sys::FLONUM_P(val.as_raw()) {
            Some(Flonum(val))
        } else {
            None
        }
    }

    /// Get the value as f64.
    #[inline]
    pub fn to_f64(self) -> f64 {
        // SAFETY: self.0 is a valid Flonum, rb_float_value extracts the value
        unsafe { rb_sys::rb_float_value(self.0.as_raw()) }
    }
}

#[cfg(target_pointer_width = "64")]
impl ReprValue for Flonum {
    #[inline]
    fn as_value(self) -> Value {
        self.0
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        Flonum(val)
    }
}

#[cfg(target_pointer_width = "64")]
impl TryConvert for Flonum {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if rb_sys::FLONUM_P(val.as_raw()) {
            // SAFETY: We've verified it's a Flonum
            Ok(unsafe { Flonum::from_value_unchecked(val) })
        } else {
            Err(Error::type_error("expected Flonum"))
        }
    }
}

#[cfg(target_pointer_width = "64")]
impl IntoValue for Flonum {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

// f64 and f32 conversions
// These create floats (either Flonum or RFloat depending on platform and value)

impl TryConvert for f64 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        // Accept any Float type (Flonum or RFloat)
        if val.rb_type() == ValueType::Float || rb_sys::FLONUM_P(val.as_raw()) {
            // SAFETY: We've verified it's a Float
            Ok(unsafe { rb_sys::rb_float_value(val.as_raw()) })
        } else {
            Err(Error::type_error("expected Float"))
        }
    }
}

impl IntoValue for f64 {
    fn into_value(self) -> Value {
        // SAFETY: rb_float_new creates a Float VALUE
        unsafe { Value::from_raw(rb_sys::rb_float_new(self)) }
    }
}

impl TryConvert for f32 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let f = f64::try_convert(val)?;
        Ok(f as f32)
    }
}

impl IntoValue for f32 {
    fn into_value(self) -> Value {
        (self as f64).into_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f64_conversion() {
        let val = 3.14f64.into_value();
        let f = f64::try_convert(val).unwrap();
        assert!((f - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_f32_conversion() {
        let val = 2.5f32.into_value();
        let f = f32::try_convert(val).unwrap();
        assert!((f - 2.5).abs() < 0.001);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_flonum() {
        // Small floats should be Flonums on 64-bit
        let num = Flonum::from_f64(1.5).expect("1.5 should be a Flonum");
        assert!((num.to_f64() - 1.5).abs() < 0.001);
    }
}
