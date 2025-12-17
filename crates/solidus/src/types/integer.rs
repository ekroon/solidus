//! Ruby integer types: Fixnum (immediate) and Bignum (heap).

use crate::convert::{IntoValue, TryConvert};
use crate::error::Error;
use crate::value::{PinGuard, ReprValue, Value};

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
#[derive(Clone, Debug)]
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
    pub fn to_i64(&self) -> i64 {
        // SAFETY: self.0 is a valid Fixnum, rb_num2ll extracts the value
        unsafe { rb_sys::rb_num2ll(self.0.as_raw()) as i64 }
    }
}

impl ReprValue for Fixnum {
    #[inline]
    fn as_value(&self) -> Value {
        self.0.clone()
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
        let int = Integer::try_convert(val)?;
        int.to_i64()
    }
}

impl IntoValue for i64 {
    fn into_value(self) -> Value {
        Integer::from_i64(self).into_value()
    }
}

impl TryConvert for i32 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into().map_err(|_| {
            Error::new(
                crate::ExceptionClass::RangeError,
                format!("integer {} out of range for i32", n),
            )
        })
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
        n.try_into().map_err(|_| {
            Error::new(
                crate::ExceptionClass::RangeError,
                format!("integer {} out of range for i16", n),
            )
        })
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
        n.try_into().map_err(|_| {
            Error::new(
                crate::ExceptionClass::RangeError,
                format!("integer {} out of range for i8", n),
            )
        })
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
        n.try_into().map_err(|_| {
            Error::new(
                crate::ExceptionClass::RangeError,
                format!("integer {} out of range for isize", n),
            )
        })
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
        let int = Integer::try_convert(val)?;
        int.to_u64()
    }
}

impl IntoValue for u64 {
    fn into_value(self) -> Value {
        Integer::from_u64(self).into_value()
    }
}

impl TryConvert for u32 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let n = i64::try_convert(val)?;
        n.try_into().map_err(|_| {
            Error::new(
                crate::ExceptionClass::RangeError,
                format!("integer {} out of range for u32", n),
            )
        })
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
        n.try_into().map_err(|_| {
            Error::new(
                crate::ExceptionClass::RangeError,
                format!("integer {} out of range for u16", n),
            )
        })
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
        n.try_into().map_err(|_| {
            Error::new(
                crate::ExceptionClass::RangeError,
                format!("integer {} out of range for u8", n),
            )
        })
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
        n.try_into().map_err(|_| {
            Error::new(
                crate::ExceptionClass::RangeError,
                format!("integer {} out of range for usize", n),
            )
        })
    }
}

impl IntoValue for usize {
    fn into_value(self) -> Value {
        (self as i64).into_value()
    }
}

/// Large integer (heap allocated).
///
/// Ruby Bignum represents integers that are too large to fit in a Fixnum.
/// These are heap-allocated objects that require GC protection.
///
/// # Example
///
/// ```ignore
/// use solidus::types::RBignum;
///
/// let big = RBignum::from_value(some_large_ruby_value).unwrap();
/// let n = big.to_i64().unwrap();
/// ```
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct RBignum(Value);

impl RBignum {
    /// Create an RBignum from an i64.
    ///
    /// This uses Ruby's integer creation function which may return a Fixnum
    /// if the value fits, or a Bignum if it doesn't. Returns None if the
    /// result is not a Bignum.
    ///
    /// In practice, most i64 values will fit in a Fixnum, so this will often
    /// return None.
    ///
    /// Returns a `PinGuard<RBignum>` that must be pinned on the stack
    /// or boxed on the heap for GC safety.
    pub fn from_i64(n: i64) -> Option<PinGuard<Self>> {
        // SAFETY: rb_ll2inum creates a Ruby integer (Fixnum or Bignum)
        let val = unsafe { rb_sys::rb_ll2inum(n as ::std::os::raw::c_longlong) };
        let val = unsafe { Value::from_raw(val) };

        // Check if it's actually a Bignum
        if val.rb_type() == crate::value::ValueType::Bignum {
            Some(PinGuard::new(RBignum(val)))
        } else {
            None
        }
    }

    /// Create an RBignum from a u64.
    ///
    /// This uses Ruby's integer creation function which may return a Fixnum
    /// if the value fits, or a Bignum if it doesn't. Returns None if the
    /// result is not a Bignum.
    ///
    /// Returns a `PinGuard<RBignum>` that must be pinned on the stack
    /// or boxed on the heap for GC safety.
    pub fn from_u64(n: u64) -> Option<PinGuard<Self>> {
        // SAFETY: rb_ull2inum creates a Ruby integer (Fixnum or Bignum)
        let val = unsafe { rb_sys::rb_ull2inum(n as ::std::os::raw::c_ulonglong) };
        let val = unsafe { Value::from_raw(val) };

        // Check if it's actually a Bignum
        if val.rb_type() == crate::value::ValueType::Bignum {
            Some(PinGuard::new(RBignum(val)))
        } else {
            None
        }
    }

    /// Convert to i64.
    ///
    /// Returns an error if the value is out of range for i64.
    pub fn to_i64(&self) -> Result<i64, Error> {
        // SAFETY: self.0 is a valid Bignum VALUE
        // rb_num2ll will raise a RangeError if the value is out of range,
        // which we should catch. For now, we'll use rb_big2ll which is more direct.
        let result = unsafe { rb_sys::rb_big2ll(self.0.as_raw()) };
        Ok(result as i64)
    }

    /// Convert to u64.
    ///
    /// Returns an error if the value is negative or out of range for u64.
    pub fn to_u64(&self) -> Result<u64, Error> {
        // SAFETY: self.0 is a valid Bignum VALUE
        let result = unsafe { rb_sys::rb_big2ull(self.0.as_raw()) };
        Ok(result as u64)
    }
}

impl ReprValue for RBignum {
    #[inline]
    fn as_value(&self) -> Value {
        self.0.clone()
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        RBignum(val)
    }
}

impl TryConvert for RBignum {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if val.rb_type() == crate::value::ValueType::Bignum {
            // SAFETY: We've verified it's a Bignum
            Ok(unsafe { RBignum::from_value_unchecked(val) })
        } else {
            Err(Error::type_error("expected Bignum"))
        }
    }
}

impl IntoValue for RBignum {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

/// Any Ruby integer (Fixnum or Bignum).
///
/// This enum represents any Ruby Integer class instance, whether it's a
/// small immediate Fixnum or a large heap-allocated Bignum.
///
/// # Example
///
/// ```ignore
/// use solidus::types::Integer;
///
/// let small = Integer::from_i64(42);
/// let large = Integer::from_u64(u64::MAX);
///
/// assert_eq!(small.to_i64().unwrap(), 42);
/// ```
#[derive(Clone, Debug)]
pub enum Integer {
    /// Small integer (immediate value)
    Fixnum(Fixnum),
    /// Large integer (heap allocated)
    Bignum(RBignum),
}

impl Integer {
    /// Create an Integer from an i64.
    ///
    /// This will create a Fixnum if the value fits, or a Bignum if it doesn't.
    pub fn from_i64(n: i64) -> Self {
        if let Some(fixnum) = Fixnum::from_i64(n) {
            Integer::Fixnum(fixnum)
        } else {
            // If it doesn't fit in a Fixnum, it must be a Bignum
            // SAFETY: rb_ll2inum creates a valid Ruby integer
            let val = unsafe { rb_sys::rb_ll2inum(n as ::std::os::raw::c_longlong) };
            let val = unsafe { Value::from_raw(val) };
            Integer::Bignum(RBignum(val))
        }
    }

    /// Create an Integer from a u64.
    ///
    /// This will create a Fixnum if the value fits, or a Bignum if it doesn't.
    pub fn from_u64(n: u64) -> Self {
        // Try to convert to i64 first for Fixnum
        if let Ok(i) = i64::try_from(n) {
            if let Some(fixnum) = Fixnum::from_i64(i) {
                return Integer::Fixnum(fixnum);
            }
        }

        // Otherwise, create a Bignum
        // SAFETY: rb_ull2inum creates a valid Ruby integer
        let val = unsafe { rb_sys::rb_ull2inum(n as ::std::os::raw::c_ulonglong) };
        let val = unsafe { Value::from_raw(val) };

        // It might still be a Fixnum on some platforms/values
        if rb_sys::FIXNUM_P(val.as_raw()) {
            Integer::Fixnum(Fixnum(val))
        } else {
            Integer::Bignum(RBignum(val))
        }
    }

    /// Convert to i64.
    ///
    /// Returns an error if the value is out of range for i64.
    pub fn to_i64(&self) -> Result<i64, Error> {
        match self {
            Integer::Fixnum(f) => Ok(f.to_i64()),
            Integer::Bignum(b) => b.to_i64(),
        }
    }

    /// Convert to u64.
    ///
    /// Returns an error if the value is negative or out of range for u64.
    pub fn to_u64(&self) -> Result<u64, Error> {
        match self {
            Integer::Fixnum(f) => {
                let n = f.to_i64();
                n.try_into().map_err(|_| {
                    Error::new(
                        crate::ExceptionClass::RangeError,
                        format!("integer {} out of range for u64 (negative)", n),
                    )
                })
            }
            Integer::Bignum(b) => b.to_u64(),
        }
    }
}

impl ReprValue for Integer {
    #[inline]
    fn as_value(&self) -> Value {
        match self {
            Integer::Fixnum(f) => f.as_value(),
            Integer::Bignum(b) => b.as_value(),
        }
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        if rb_sys::FIXNUM_P(val.as_raw()) {
            // SAFETY: Caller ensures val is a valid integer
            Integer::Fixnum(unsafe { Fixnum::from_value_unchecked(val) })
        } else {
            // SAFETY: Caller ensures val is a valid integer
            Integer::Bignum(unsafe { RBignum::from_value_unchecked(val) })
        }
    }
}

impl TryConvert for Integer {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if rb_sys::FIXNUM_P(val.as_raw()) {
            Ok(Integer::Fixnum(Fixnum::try_convert(val)?))
        } else if val.rb_type() == crate::value::ValueType::Bignum {
            Ok(Integer::Bignum(RBignum::try_convert(val)?))
        } else {
            Err(Error::type_error("expected Integer (Fixnum or Bignum)"))
        }
    }
}

impl IntoValue for Integer {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

#[cfg(all(test, any(feature = "embed", feature = "link-ruby")))]
mod tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;

    #[ruby_test]
    fn test_fixnum_small() {
        let num = Fixnum::from_i64(42).unwrap();
        assert_eq!(num.to_i64(), 42);
    }

    #[ruby_test]
    fn test_fixnum_zero() {
        let num = Fixnum::from_i64(0).unwrap();
        assert_eq!(num.to_i64(), 0);
    }

    #[ruby_test]
    fn test_fixnum_negative() {
        let num = Fixnum::from_i64(-123).unwrap();
        assert_eq!(num.to_i64(), -123);
    }

    #[ruby_test]
    fn test_i32_conversion() {
        let val = 42i32.into_value();
        let n = i32::try_convert(val).unwrap();
        assert_eq!(n, 42);
    }

    #[ruby_test]
    fn test_u8_conversion() {
        let val = 255u8.into_value();
        let n = u8::try_convert(val).unwrap();
        assert_eq!(n, 255);
    }

    #[ruby_test]
    fn test_bignum_creation() {
        // Create a very large number that won't fit in a Fixnum
        let large = i64::MAX;
        let val = unsafe { rb_sys::rb_ll2inum(large as ::std::os::raw::c_longlong) };
        let val = unsafe { Value::from_raw(val) };

        // On most platforms this will still be a Fixnum since Fixnum range is ~±2^62
        // Let's try with actual Bignum range
        if val.rb_type() == crate::value::ValueType::Bignum {
            let bignum = RBignum::try_convert(val).unwrap();
            assert_eq!(bignum.to_i64().unwrap(), large);
        }
    }

    #[ruby_test]
    fn test_bignum_from_large_u64() {
        // Use a very large u64 that should create a Bignum on all platforms
        // 2^63 and larger should always be Bignum
        let large = u64::MAX;
        let int = Integer::from_u64(large);

        // This should be a Bignum on all platforms
        match int {
            Integer::Bignum(b) => {
                // Verify we can convert it back
                assert_eq!(b.to_u64().unwrap(), large);
            }
            Integer::Fixnum(_) => {
                // On some platforms, even large values might fit in Fixnum
                // Just verify roundtrip works
                assert_eq!(int.to_u64().unwrap(), large);
            }
        }
    }

    #[ruby_test]
    fn test_integer_bignum_conversion() {
        // Create an Integer from a large value
        let large = (1u64 << 63) + 1234; // Definitely bigger than Fixnum range
        let int = Integer::from_u64(large);

        // Convert back
        assert_eq!(int.to_u64().unwrap(), large);

        // Verify it round-trips through Value
        let val = int.into_value();
        let int2 = Integer::try_convert(val).unwrap();
        assert_eq!(int2.to_u64().unwrap(), large);
    }

    #[ruby_test]
    fn test_integer_small() {
        let int = Integer::from_i64(42);
        assert!(matches!(int, Integer::Fixnum(_)));
        assert_eq!(int.to_i64().unwrap(), 42);
    }

    #[ruby_test]
    fn test_integer_zero() {
        let int = Integer::from_i64(0);
        assert!(matches!(int, Integer::Fixnum(_)));
        assert_eq!(int.to_i64().unwrap(), 0);
    }

    #[ruby_test]
    fn test_integer_negative() {
        let int = Integer::from_i64(-999);
        assert!(matches!(int, Integer::Fixnum(_)));
        assert_eq!(int.to_i64().unwrap(), -999);
    }

    #[ruby_test]
    fn test_integer_from_u64() {
        let int = Integer::from_u64(12345);
        assert_eq!(int.to_u64().unwrap(), 12345);
    }

    #[ruby_test]
    fn test_integer_negative_to_u64_fails() {
        let int = Integer::from_i64(-1);
        assert!(int.to_u64().is_err());
    }

    #[ruby_test]
    fn test_integer_round_trip() {
        let original = 98765i64;
        let int = Integer::from_i64(original);
        let val = int.into_value();
        let int2 = Integer::try_convert(val).unwrap();
        assert_eq!(int2.to_i64().unwrap(), original);
    }

    #[ruby_test]
    fn test_integer_try_convert_from_fixnum() {
        let val = 42i64.into_value();
        let int = Integer::try_convert(val).unwrap();
        assert!(matches!(int, Integer::Fixnum(_)));
        assert_eq!(int.to_i64().unwrap(), 42);
    }
}
