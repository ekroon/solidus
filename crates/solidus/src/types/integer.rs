//! Ruby integer types: Fixnum (immediate) and Bignum (heap).

use crate::convert::{IntoValue, TryConvert};
use crate::error::Error;
use crate::value::{ReprValue, Value};

/// Small integer that fits in a VALUE (immediate value).
///
/// Ruby Fixnum represents integers that can be encoded directly in the VALUE.
/// On most platforms, this is roughly ±2^62 (about ±4.6 × 10^18).
///
/// Fixnum is an immediate value and does not require GC protection or pinning.
///
/// # Example
///
/// ```ignore
/// use solidus::types::Fixnum;
///
/// let num = Fixnum::from_i64(42).expect("42 fits in a Fixnum");
/// assert_eq!(num.to_i64(), 42);
/// ```
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Fixnum(Value);

impl Fixnum {
    /// Create a Fixnum from an i64.
    ///
    /// Returns `None` if the value doesn't fit in a Fixnum (too large or too small).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::Fixnum;
    ///
    /// assert!(Fixnum::from_i64(42).is_some());
    /// assert!(Fixnum::from_i64(i64::MAX).is_none()); // Too large
    /// ```
    pub fn from_i64(n: i64) -> Option<Self> {
        // Check if the value fits in isize before calling rb_int2inum
        // On 32-bit platforms, isize is i32, so we need to check the range
        if n < isize::MIN as i64 || n > isize::MAX as i64 {
            return None;
        }
        
        // SAFETY: We've verified n fits in isize, and rb_int2inum handles the conversion
        let val = unsafe { rb_sys::rb_int2inum(n as isize) };
        let val = unsafe { Value::from_raw(val) };
        
        // Check if it's actually a Fixnum (not a Bignum)
        if rb_sys::FIXNUM_P(val.as_raw()) {
            Some(Fixnum(val))
        } else {
            None
        }
    }

    /// Get the value as i64.
    ///
    /// This always succeeds because all Fixnum values fit in i64.
    #[inline]
    pub fn to_i64(self) -> i64 {
        // SAFETY: self.0 is a valid Fixnum, rb_num2ll extracts the value
        unsafe { rb_sys::rb_num2ll(self.0.as_raw()) as i64 }
    }
}

impl ReprValue for Fixnum {
    #[inline]
    fn as_value(self) -> Value {
        self.0
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        Fixnum(val)
    }
}

impl TryConvert for Fixnum {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if rb_sys::FIXNUM_P(val.as_raw()) {
            // SAFETY: We've verified it's a Fixnum
            Ok(unsafe { Fixnum::from_value_unchecked(val) })
        } else {
            Err(Error::type_error("expected Fixnum"))
        }
    }
}

impl IntoValue for Fixnum {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

// Conversions for standard integer types

impl TryConvert for i64 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let fixnum = Fixnum::try_convert(val)?;
        Ok(fixnum.to_i64())
    }
}

impl IntoValue for i64 {
    fn into_value(self) -> Value {
        // For now, we panic if it doesn't fit. In Phase 3 we'll use Bignum.
        Fixnum::from_i64(self)
            .expect("i64 value too large for Fixnum (Bignum not yet implemented)")
            .into_value()
    }
}

impl TryConvert for i32 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into()
            .map_err(|_| Error::new(crate::ExceptionClass::RangeError, format!("integer {} out of range for i32", n)))
    }
}

impl IntoValue for i32 {
    fn into_value(self) -> Value {
        (self as i64).into_value()
    }
}

impl TryConvert for i16 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into()
            .map_err(|_| Error::new(crate::ExceptionClass::RangeError, format!("integer {} out of range for i16", n)))
    }
}

impl IntoValue for i16 {
    fn into_value(self) -> Value {
        (self as i64).into_value()
    }
}

impl TryConvert for i8 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into()
            .map_err(|_| Error::new(crate::ExceptionClass::RangeError, format!("integer {} out of range for i8", n)))
    }
}

impl IntoValue for i8 {
    fn into_value(self) -> Value {
        (self as i64).into_value()
    }
}

impl TryConvert for isize {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into()
            .map_err(|_| Error::new(crate::ExceptionClass::RangeError, format!("integer {} out of range for isize", n)))
    }
}

impl IntoValue for isize {
    fn into_value(self) -> Value {
        (self as i64).into_value()
    }
}

// Unsigned integer conversions (may need range checking)

impl TryConvert for u64 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into()
            .map_err(|_| Error::new(crate::ExceptionClass::RangeError, format!("integer {} out of range for u64", n)))
    }
}

impl IntoValue for u64 {
    fn into_value(self) -> Value {
        // For now, we panic if it doesn't fit. In Phase 3 we'll use Bignum.
        self.try_into()
            .ok()
            .and_then(|n: i64| Fixnum::from_i64(n))
            .expect("u64 value too large for Fixnum (Bignum not yet implemented)")
            .into_value()
    }
}

impl TryConvert for u32 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into()
            .map_err(|_| Error::new(crate::ExceptionClass::RangeError, format!("integer {} out of range for u32", n)))
    }
}

impl IntoValue for u32 {
    fn into_value(self) -> Value {
        (self as i64).into_value()
    }
}

impl TryConvert for u16 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into()
            .map_err(|_| Error::new(crate::ExceptionClass::RangeError, format!("integer {} out of range for u16", n)))
    }
}

impl IntoValue for u16 {
    fn into_value(self) -> Value {
        (self as i64).into_value()
    }
}

impl TryConvert for u8 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into()
            .map_err(|_| Error::new(crate::ExceptionClass::RangeError, format!("integer {} out of range for u8", n)))
    }
}

impl IntoValue for u8 {
    fn into_value(self) -> Value {
        (self as i64).into_value()
    }
}

impl TryConvert for usize {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into()
            .map_err(|_| Error::new(crate::ExceptionClass::RangeError, format!("integer {} out of range for usize", n)))
    }
}

impl IntoValue for usize {
    fn into_value(self) -> Value {
        (self as i64).into_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixnum_small() {
        let num = Fixnum::from_i64(42).unwrap();
        assert_eq!(num.to_i64(), 42);
    }

    #[test]
    fn test_fixnum_zero() {
        let num = Fixnum::from_i64(0).unwrap();
        assert_eq!(num.to_i64(), 0);
    }

    #[test]
    fn test_fixnum_negative() {
        let num = Fixnum::from_i64(-123).unwrap();
        assert_eq!(num.to_i64(), -123);
    }

    #[test]
    fn test_i32_conversion() {
        let val = 42i32.into_value();
        let n = i32::try_convert(val).unwrap();
        assert_eq!(n, 42);
    }

    #[test]
    fn test_u8_conversion() {
        let val = 255u8.into_value();
        let n = u8::try_convert(val).unwrap();
        assert_eq!(n, 255);
    }
}
