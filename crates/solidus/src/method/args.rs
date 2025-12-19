//! Method argument type marker.
//!
//! This module provides the [`MethodArg`] trait which marks types that can be used
//! as method arguments. The trait indicates whether a type needs stack pinning when
//! passed to Ruby methods.
//!
//! # Pinning Strategy
//!
//! - **Immediate values** (Fixnum, Symbol, bool, etc.) don't need pinning as they're
//!   encoded directly in the VALUE and don't require GC protection.
//! - **Heap values** (String, Array, Hash, etc.) need pinning to ensure the GC can
//!   find them on the stack.

use crate::types::{
    Fixnum, Float, Integer, Qfalse, Qnil, Qtrue, RArray, RBignum, RClass, RFloat, RHash, RModule,
    RString, Symbol,
};
use crate::value::Value;

#[cfg(target_pointer_width = "64")]
use crate::types::Flonum;

/// Marker trait for types that can be method arguments.
///
/// This trait indicates whether a type needs stack pinning when passed
/// to a Ruby method. The `NEEDS_PINNING` constant is used by the method
/// registration macros to generate appropriate wrapper code.
///
/// # Example
///
/// ```no_run
/// use solidus::method::MethodArg;
/// use solidus::types::{Fixnum, RString};
///
/// // Check if a type needs pinning
/// assert_eq!(Fixnum::NEEDS_PINNING, false);  // immediate value
/// assert_eq!(RString::NEEDS_PINNING, true);  // heap value
/// ```
pub trait MethodArg: Sized {
    /// Whether this type requires stack pinning.
    ///
    /// - `false` for immediate values (Fixnum, Symbol, bool, etc.)
    /// - `true` for heap-allocated Ruby objects
    const NEEDS_PINNING: bool;
}

// ============================================================================
// Immediate types - no pinning needed
// ============================================================================

// Rust integer types
impl MethodArg for i8 {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for i16 {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for i32 {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for i64 {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for isize {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for u8 {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for u16 {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for u32 {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for u64 {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for usize {
    const NEEDS_PINNING: bool = false;
}

// Rust float types
impl MethodArg for f32 {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for f64 {
    const NEEDS_PINNING: bool = false;
}

// Rust bool
impl MethodArg for bool {
    const NEEDS_PINNING: bool = false;
}

// Ruby immediate types
impl MethodArg for Fixnum {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for Symbol {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for Qnil {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for Qtrue {
    const NEEDS_PINNING: bool = false;
}
impl MethodArg for Qfalse {
    const NEEDS_PINNING: bool = false;
}

// Flonum is immediate on 64-bit platforms
#[cfg(target_pointer_width = "64")]
impl MethodArg for Flonum {
    const NEEDS_PINNING: bool = false;
}

// ============================================================================
// Heap types - pinning needed
// ============================================================================

impl MethodArg for RString {
    const NEEDS_PINNING: bool = true;
}
impl MethodArg for RArray {
    const NEEDS_PINNING: bool = true;
}
impl MethodArg for RHash {
    const NEEDS_PINNING: bool = true;
}
impl MethodArg for RClass {
    const NEEDS_PINNING: bool = true;
}
impl MethodArg for RModule {
    const NEEDS_PINNING: bool = true;
}
impl MethodArg for Value {
    const NEEDS_PINNING: bool = true;
}

// Integer can be either Fixnum (immediate) or Bignum (heap), so we pin it
impl MethodArg for Integer {
    const NEEDS_PINNING: bool = true;
}

// Float can be either Flonum (immediate on 64-bit) or RFloat (heap), so we pin it
impl MethodArg for Float {
    const NEEDS_PINNING: bool = true;
}

// Specific heap numeric types
impl MethodArg for RBignum {
    const NEEDS_PINNING: bool = true;
}
impl MethodArg for RFloat {
    const NEEDS_PINNING: bool = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_integer_types_no_pinning() {
        assert!(!i8::NEEDS_PINNING);
        assert!(!i16::NEEDS_PINNING);
        assert!(!i32::NEEDS_PINNING);
        assert!(!i64::NEEDS_PINNING);
        assert!(!isize::NEEDS_PINNING);
        assert!(!u8::NEEDS_PINNING);
        assert!(!u16::NEEDS_PINNING);
        assert!(!u32::NEEDS_PINNING);
        assert!(!u64::NEEDS_PINNING);
        assert!(!usize::NEEDS_PINNING);
    }

    #[test]
    fn test_rust_float_types_no_pinning() {
        assert!(!f32::NEEDS_PINNING);
        assert!(!f64::NEEDS_PINNING);
    }

    #[test]
    fn test_bool_no_pinning() {
        assert!(!bool::NEEDS_PINNING);
    }

    #[test]
    fn test_immediate_ruby_types_no_pinning() {
        assert!(!Fixnum::NEEDS_PINNING);
        assert!(!Symbol::NEEDS_PINNING);
        assert!(!Qnil::NEEDS_PINNING);
        assert!(!Qtrue::NEEDS_PINNING);
        assert!(!Qfalse::NEEDS_PINNING);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_flonum_no_pinning() {
        assert!(!Flonum::NEEDS_PINNING);
    }

    #[test]
    fn test_heap_ruby_types_need_pinning() {
        assert!(RString::NEEDS_PINNING);
        assert!(RArray::NEEDS_PINNING);
        assert!(RHash::NEEDS_PINNING);
        assert!(RClass::NEEDS_PINNING);
        assert!(RModule::NEEDS_PINNING);
        assert!(Value::NEEDS_PINNING);
    }

    #[test]
    fn test_numeric_types_pinning() {
        // Integer can be Fixnum or Bignum, so it needs pinning
        assert!(Integer::NEEDS_PINNING);
        // Float can be Flonum or RFloat, so it needs pinning
        assert!(Float::NEEDS_PINNING);
        // Specific heap types
        assert!(RBignum::NEEDS_PINNING);
        assert!(RFloat::NEEDS_PINNING);
    }

    #[test]
    fn test_pinning_strategy_documented() {
        // This test documents the pinning strategy:
        // - Immediate values (encoded in VALUE): no pinning
        // - Heap values (allocated objects): pinning required
        // - Polymorphic types (can be either): conservative, require pinning

        // Group 1: Always immediate
        assert!(!Fixnum::NEEDS_PINNING);
        assert!(!Symbol::NEEDS_PINNING);

        // Group 2: Always heap
        assert!(RString::NEEDS_PINNING);
        assert!(RArray::NEEDS_PINNING);
        assert!(RBignum::NEEDS_PINNING);

        // Group 3: Polymorphic - conservative approach
        assert!(Integer::NEEDS_PINNING); // Can be Fixnum or Bignum
        assert!(Float::NEEDS_PINNING); // Can be Flonum or RFloat
        assert!(Value::NEEDS_PINNING); // Can be anything
    }
}
