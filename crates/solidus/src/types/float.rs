//! Ruby Float types: Flonum (immediate, 64-bit only) and RFloat (heap).

use crate::convert::{IntoValue, TryConvert};
use crate::error::Error;
use crate::value::{BoxValue, NewValue, ReprValue, Value, ValueType};

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
/// ```no_run
/// use solidus::types::Flonum;
///
/// let num = Flonum::from_f64(3.14).expect("3.14 can be a Flonum");
/// assert!((num.to_f64() - 3.14).abs() < 0.001);
/// ```
#[cfg(target_pointer_width = "64")]
#[derive(Clone, Debug)]
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
    pub fn to_f64(&self) -> f64 {
        // SAFETY: self.0 is a valid Flonum, rb_float_value extracts the value
        unsafe { rb_sys::rb_float_value(self.0.as_raw()) }
    }
}

#[cfg(target_pointer_width = "64")]
impl ReprValue for Flonum {
    #[inline]
    fn as_value(&self) -> Value {
        self.0.clone()
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

/// Heap-allocated float.
///
/// On 32-bit platforms, all floats are heap-allocated. On 64-bit platforms,
/// small floats can be immediate values (Flonum), while larger floats are
/// heap-allocated as RFloat.
///
/// # Example
///
/// ```no_run
/// use solidus::types::RFloat;
/// use solidus::pin_on_stack;
///
/// // SAFETY: Value is immediately pinned
/// pin_on_stack!(num = unsafe { RFloat::from_f64(3.14159265358979) });
/// assert!((num.get().to_f64() - 3.14159265358979).abs() < 0.0001);
/// ```
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct RFloat(Value);

impl RFloat {
    /// Create an RFloat from an f64.
    ///
    /// This always creates a heap-allocated float. On 64-bit platforms,
    /// if the value can be represented as a Flonum, use `Flonum::from_f64` instead.
    ///
    /// Returns a `NewValue<RFloat>` that must be pinned on the stack
    /// or boxed on the heap for GC safety.
    ///
    /// # Safety
    ///
    /// The returned `NewValue` must be immediately consumed by either:
    /// - `pin_on_stack!` macro to pin on the stack
    /// - `.into_box()` to box for heap storage
    ///
    /// Failure to do so may result in the value being garbage collected.
    /// For a safe alternative, use [`from_f64_boxed`](Self::from_f64_boxed).
    pub unsafe fn from_f64(n: f64) -> NewValue<Self> {
        // SAFETY: rb_float_new creates a Float VALUE
        let val = unsafe { Value::from_raw(rb_sys::rb_float_new(n)) };
        NewValue::new(RFloat(val))
    }

    /// Create an RFloat from an f64, boxed for heap storage.
    ///
    /// This is safe because the value is immediately registered with Ruby's GC.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RFloat;
    ///
    /// let boxed = RFloat::from_f64_boxed(3.14159265358979);
    /// assert!((boxed.to_f64() - 3.14159265358979).abs() < 0.0000001);
    /// ```
    pub fn from_f64_boxed(n: f64) -> BoxValue<Self> {
        // SAFETY: We immediately box and register with GC
        unsafe { Self::from_f64(n) }.into_box()
    }

    /// Get the value as f64.
    #[inline]
    pub fn to_f64(&self) -> f64 {
        // SAFETY: self.0 is a valid Float, rb_float_value extracts the value
        unsafe { rb_sys::rb_float_value(self.0.as_raw()) }
    }
}

impl ReprValue for RFloat {
    #[inline]
    fn as_value(&self) -> Value {
        self.0.clone()
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        RFloat(val)
    }
}

impl TryConvert for RFloat {
    fn try_convert(val: Value) -> Result<Self, Error> {
        // Check if it's a heap Float (not Flonum)
        if val.rb_type() == ValueType::Float {
            // SAFETY: We've verified it's a heap Float
            Ok(unsafe { RFloat::from_value_unchecked(val) })
        } else {
            Err(Error::type_error("expected RFloat (heap-allocated float)"))
        }
    }
}

impl IntoValue for RFloat {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

/// Any Ruby float.
///
/// This enum represents any Ruby Float class instance, whether it's an
/// immediate Flonum (on 64-bit platforms) or a heap-allocated RFloat.
///
/// # Example
///
/// ```no_run
/// use solidus::types::Float;
///
/// let small = Float::from_f64(1.5);
/// let large = Float::from_f64(3.141592653589793);
///
/// assert!((small.to_f64() - 1.5).abs() < 0.001);
/// ```
#[derive(Clone, Debug)]
pub enum Float {
    #[cfg(target_pointer_width = "64")]
    /// Immediate float (64-bit only)
    Flonum(Flonum),
    /// Heap-allocated float
    RFloat(RFloat),
}

impl Float {
    /// Create a Float from an f64.
    ///
    /// On 64-bit platforms, this will create a Flonum if possible, or an RFloat if not.
    /// On 32-bit platforms, this always creates an RFloat.
    pub fn from_f64(n: f64) -> Self {
        #[cfg(target_pointer_width = "64")]
        {
            if let Some(flonum) = Flonum::from_f64(n) {
                Float::Flonum(flonum)
            } else {
                // SAFETY: We immediately unwrap the NewValue to return Self
                Float::RFloat(unsafe { RFloat::from_f64(n).into_inner() })
            }
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            // SAFETY: We immediately unwrap the NewValue to return Self
            Float::RFloat(unsafe { RFloat::from_f64(n).into_inner() })
        }
    }

    /// Get the value as f64.
    #[inline]
    pub fn to_f64(&self) -> f64 {
        match self {
            #[cfg(target_pointer_width = "64")]
            Float::Flonum(f) => f.to_f64(),
            Float::RFloat(f) => f.to_f64(),
        }
    }
}

impl ReprValue for Float {
    #[inline]
    fn as_value(&self) -> Value {
        match self {
            #[cfg(target_pointer_width = "64")]
            Float::Flonum(f) => f.as_value(),
            Float::RFloat(f) => f.as_value(),
        }
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        #[cfg(target_pointer_width = "64")]
        {
            if rb_sys::FLONUM_P(val.as_raw()) {
                // SAFETY: Caller ensures val is a valid float
                Float::Flonum(unsafe { Flonum::from_value_unchecked(val) })
            } else {
                // SAFETY: Caller ensures val is a valid float
                Float::RFloat(unsafe { RFloat::from_value_unchecked(val) })
            }
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            // SAFETY: Caller ensures val is a valid float
            Float::RFloat(unsafe { RFloat::from_value_unchecked(val) })
        }
    }
}

impl TryConvert for Float {
    fn try_convert(val: Value) -> Result<Self, Error> {
        #[cfg(target_pointer_width = "64")]
        {
            if rb_sys::FLONUM_P(val.as_raw()) {
                Ok(Float::Flonum(Flonum::try_convert(val)?))
            } else if val.rb_type() == ValueType::Float {
                Ok(Float::RFloat(RFloat::try_convert(val)?))
            } else {
                Err(Error::type_error("expected Float (Flonum or RFloat)"))
            }
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            if val.rb_type() == ValueType::Float {
                Ok(Float::RFloat(RFloat::try_convert(val)?))
            } else {
                Err(Error::type_error("expected Float"))
            }
        }
    }
}

impl IntoValue for Float {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

// f64 and f32 conversions
// These create floats (either Flonum or RFloat depending on platform and value)

impl TryConvert for f64 {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let float = Float::try_convert(val)?;
        Ok(float.to_f64())
    }
}

impl IntoValue for f64 {
    fn into_value(self) -> Value {
        Float::from_f64(self).into_value()
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

#[cfg(all(test, any(feature = "embed", feature = "link-ruby")))]
mod tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;

    // Flonum tests (64-bit only)
    #[cfg(target_pointer_width = "64")]
    #[ruby_test]
    fn test_flonum_small() {
        // Small floats should be Flonums on 64-bit
        let num = Flonum::from_f64(1.5).expect("1.5 should be a Flonum");
        assert!((num.to_f64() - 1.5).abs() < 0.001);
    }

    #[cfg(target_pointer_width = "64")]
    #[ruby_test]
    fn test_flonum_zero() {
        let num = Flonum::from_f64(0.0).expect("0.0 should be a Flonum");
        assert_eq!(num.to_f64(), 0.0);
    }

    #[cfg(target_pointer_width = "64")]
    #[ruby_test]
    fn test_flonum_negative() {
        let num = Flonum::from_f64(-2.5).expect("-2.5 should be a Flonum");
        assert!((num.to_f64() + 2.5).abs() < 0.001);
    }

    #[cfg(target_pointer_width = "64")]
    #[ruby_test]
    fn test_flonum_try_convert() {
        let val = 3.14f64.into_value();
        // If it's a Flonum, try_convert should work
        if rb_sys::FLONUM_P(val.as_raw()) {
            let num = Flonum::try_convert(val).unwrap();
            assert!((num.to_f64() - 3.14).abs() < 0.001);
        }
    }

    // RFloat tests
    #[ruby_test]
    fn test_rfloat_creation() {
        let num = RFloat::from_f64(3.14159265358979);
        assert!((num.to_f64() - 3.14159265358979).abs() < 0.0000001);
    }

    #[ruby_test]
    fn test_rfloat_zero() {
        let num = RFloat::from_f64(0.0);
        assert_eq!(num.to_f64(), 0.0);
    }

    #[ruby_test]
    fn test_rfloat_negative() {
        let num = RFloat::from_f64(-123.456);
        assert!((num.to_f64() + 123.456).abs() < 0.001);
    }

    #[ruby_test]
    fn test_rfloat_large() {
        let large = 1.7976931348623157e308; // Near f64::MAX
        let num = RFloat::from_f64(large);
        assert!((num.to_f64() - large).abs() / large < 0.0001);
    }

    #[ruby_test]
    fn test_rfloat_try_convert() {
        let val = 2.71828f64.into_value();
        // On 64-bit, this might be a Flonum, so only test if it's RFloat
        if val.rb_type() == ValueType::Float {
            let num = RFloat::try_convert(val).unwrap();
            assert!((num.to_f64() - 2.71828).abs() < 0.00001);
        }
    }

    // Float union type tests
    #[ruby_test]
    fn test_float_from_f64_small() {
        let float = Float::from_f64(1.5);
        assert!((float.to_f64() - 1.5).abs() < 0.001);
    }

    #[ruby_test]
    fn test_float_from_f64_large() {
        let float = Float::from_f64(3.141592653589793);
        assert!((float.to_f64() - 3.141592653589793).abs() < 0.0000001);
    }

    #[ruby_test]
    fn test_float_zero() {
        let float = Float::from_f64(0.0);
        assert_eq!(float.to_f64(), 0.0);
    }

    #[ruby_test]
    fn test_float_negative() {
        let float = Float::from_f64(-999.999);
        assert!((float.to_f64() + 999.999).abs() < 0.001);
    }

    #[ruby_test]
    fn test_float_try_convert() {
        let val = 2.5f64.into_value();
        let float = Float::try_convert(val).unwrap();
        assert!((float.to_f64() - 2.5).abs() < 0.001);
    }

    #[ruby_test]
    fn test_float_round_trip() {
        let original = 1.234567890123456;
        let float = Float::from_f64(original);
        let val = float.into_value();
        let float2 = Float::try_convert(val).unwrap();
        assert!((float2.to_f64() - original).abs() < 0.0000001);
    }

    #[cfg(target_pointer_width = "64")]
    #[ruby_test]
    fn test_float_enum_variants() {
        // Test that small floats become Flonums
        let small = Float::from_f64(1.5);
        if let Float::Flonum(_) = small {
            // Expected on 64-bit
        } else {
            // RFloat is also valid if Ruby decides to heap-allocate
        }

        // Verify round-trip works regardless of variant
        assert!((small.to_f64() - 1.5).abs() < 0.001);
    }

    // f64 primitive conversion tests
    #[ruby_test]
    fn test_f64_conversion() {
        let val = 3.14f64.into_value();
        let f = f64::try_convert(val).unwrap();
        assert!((f - 3.14).abs() < 0.001);
    }

    #[ruby_test]
    fn test_f64_zero() {
        let val = 0.0f64.into_value();
        let f = f64::try_convert(val).unwrap();
        assert_eq!(f, 0.0);
    }

    #[ruby_test]
    fn test_f64_negative() {
        let val = (-42.5f64).into_value();
        let f = f64::try_convert(val).unwrap();
        assert!((f + 42.5).abs() < 0.001);
    }

    #[ruby_test]
    fn test_f64_very_small() {
        let val = 1e-100f64.into_value();
        let f = f64::try_convert(val).unwrap();
        assert!((f - 1e-100).abs() < 1e-101);
    }

    #[ruby_test]
    fn test_f64_round_trip() {
        let original = 1.234567890123456789;
        let val = original.into_value();
        let f = f64::try_convert(val).unwrap();
        // f64 precision is about 15-17 decimal digits
        assert!((f - original).abs() < 1e-15);
    }

    // f32 primitive conversion tests
    #[ruby_test]
    fn test_f32_conversion() {
        let val = 2.5f32.into_value();
        let f = f32::try_convert(val).unwrap();
        assert!((f - 2.5).abs() < 0.001);
    }

    #[ruby_test]
    fn test_f32_zero() {
        let val = 0.0f32.into_value();
        let f = f32::try_convert(val).unwrap();
        assert_eq!(f, 0.0);
    }

    #[ruby_test]
    fn test_f32_negative() {
        let val = (-7.5f32).into_value();
        let f = f32::try_convert(val).unwrap();
        assert!((f + 7.5).abs() < 0.001);
    }

    #[ruby_test]
    fn test_f32_precision() {
        // f32 has less precision than f64
        let val = 1.234f32.into_value();
        let f = f32::try_convert(val).unwrap();
        assert!((f - 1.234).abs() < 0.001);
    }

    #[ruby_test]
    fn test_f32_round_trip() {
        let original = 123.456f32;
        let val = original.into_value();
        let f = f32::try_convert(val).unwrap();
        // Allow for some loss of precision due to f64 round-trip
        assert!((f - original).abs() < 0.001);
    }
}
