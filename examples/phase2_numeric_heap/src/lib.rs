//! Phase 2 Stage 3: Numeric Types (Heap) Example
//!
//! This example demonstrates heap-allocated numeric types and automatic selection
//! between immediate and heap representations.
//!
//! **Key types:**
//! - `RBignum` - Large integers that don't fit in Fixnum (heap-allocated)
//! - `Integer` - Union type that automatically selects Fixnum or Bignum
//! - `RFloat` - Heap-allocated floats
//! - `Float` - Union type that automatically selects Flonum or RFloat
//!
//! **Important concepts:**
//! - Ruby uses immediate values (Fixnum, Flonum) when possible for efficiency
//! - Large numbers require heap allocation (Bignum, RFloat)
//! - The union types (Integer, Float) handle this complexity automatically
//! - Range checking prevents overflow/underflow
//!
//! This shows Stage 3 implementation: heap numeric types with automatic selection.

use solidus::prelude::*;

/// Example 1: Creating large integers that require Bignum
///
/// Demonstrates when Ruby must use heap allocation for integers.
#[no_mangle]
pub extern "C" fn example_large_integers() -> rb_sys::VALUE {
    // Small integers use Fixnum (immediate)
    let small = Integer::from_i64(42);
    assert!(matches!(small, Integer::Fixnum(_)));
    
    // Very large u64 values typically require Bignum
    // (2^63 and above are too large for Fixnum on most platforms)
    let large = (1u64 << 63) + 12345;
    let big_int = Integer::from_u64(large);
    
    // Verify it round-trips correctly
    assert_eq!(big_int.to_u64().unwrap(), large);
    
    big_int.into_value().as_raw()
}

/// Example 2: Working with RBignum directly
///
/// Shows explicit Bignum creation and conversion.
#[no_mangle]
pub extern "C" fn example_bignum_explicit() -> rb_sys::VALUE {
    // Create a very large number
    let large_u64 = u64::MAX;
    
    // Create an Integer (may be Bignum or Fixnum depending on platform)
    let int = Integer::from_u64(large_u64);
    
    // If it's a Bignum, we can work with it
    match int {
        Integer::Bignum(bignum) => {
            // Convert back to u64
            let back = bignum.to_u64().unwrap();
            assert_eq!(back, large_u64);
            bignum.into_value().as_raw()
        }
        Integer::Fixnum(fixnum) => {
            // On some platforms even large values might fit
            // Just return it
            fixnum.into_value().as_raw()
        }
    }
}

/// Example 3: Integer union type handles complexity
///
/// The Integer enum automatically chooses the right representation.
#[no_mangle]
pub extern "C" fn example_integer_auto_selection(n: i64) -> rb_sys::VALUE {
    // Integer::from_i64 automatically picks Fixnum or Bignum
    let int = Integer::from_i64(n);
    
    // We don't need to worry about which variant it is
    // Conversion methods work on both
    match int.to_i64() {
        Ok(value) => {
            // Successfully converted back
            assert_eq!(value, n);
            int.into_value().as_raw()
        }
        Err(_) => {
            // Out of range (shouldn't happen for i64 input)
            Qnil::new().into_value().as_raw()
        }
    }
}

/// Example 4: Range checking with unsigned integers
///
/// Demonstrates conversion with range validation.
#[no_mangle]
pub extern "C" fn example_u64_range_check(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };
    
    // Try to convert to u64
    match u64::try_convert(value) {
        Ok(n) => {
            // Successfully converted
            // Double it and convert back
            let doubled = n.saturating_mul(2);
            doubled.into_value().as_raw()
        }
        Err(_) => {
            // Not an integer, or negative, or out of range
            Qnil::new().into_value().as_raw()
        }
    }
}

/// Example 5: Arithmetic with large integers
///
/// Shows working with values that may overflow Fixnum range.
#[no_mangle]
pub extern "C" fn example_large_arithmetic() -> rb_sys::VALUE {
    // Start with a large number near the edge of Fixnum range
    let base = 1i64 << 60; // 2^60
    
    // Create Integer from it
    let int1 = Integer::from_i64(base);
    let int2 = Integer::from_i64(base);
    
    // In real code, you'd use Ruby's arithmetic operations
    // Here we just demonstrate the types handle large values
    let result1 = int1.to_i64().unwrap();
    let _result2 = int2.to_i64().unwrap();
    
    // Multiply (may overflow i64, but demonstrates the concept)
    if let Some(product) = result1.checked_mul(2) {
        Integer::from_i64(product).into_value().as_raw()
    } else {
        // Overflow - return original
        int1.into_value().as_raw()
    }
}

/// Example 6: Working with heap-allocated floats (RFloat)
///
/// Demonstrates heap float creation and conversion.
#[no_mangle]
pub extern "C" fn example_rfloat() -> rb_sys::VALUE {
    // Create a heap-allocated float
    let pi = std::f64::consts::PI;
    let float = RFloat::from_f64(pi);
    
    // Get the value back
    let value = float.to_f64();
    assert!((value - pi).abs() < 0.0000001);
    
    float.into_value().as_raw()
}

/// Example 7: Float union type automatic selection
///
/// On 64-bit platforms, small floats may be Flonum (immediate).
/// The Float enum handles this automatically.
#[no_mangle]
pub extern "C" fn example_float_auto_selection(f: f64) -> rb_sys::VALUE {
    // Float::from_f64 automatically picks Flonum or RFloat
    let float = Float::from_f64(f);
    
    // We can work with it without knowing which variant
    let back = float.to_f64();
    assert!((back - f).abs() < 0.0000001);
    
    float.into_value().as_raw()
}

/// Example 8: Float conversions with type checking
///
/// Shows conversion from Ruby values to Rust float types.
#[no_mangle]
pub extern "C" fn example_float_conversion(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };
    
    // Try to convert to f64
    match f64::try_convert(value) {
        Ok(f) => {
            // Successfully converted
            // Square it and return
            let squared = f * f;
            squared.into_value().as_raw()
        }
        Err(_) => {
            // Not a float
            Qnil::new().into_value().as_raw()
        }
    }
}

/// Example 9: Demonstrating integer overflow handling
///
/// Shows how Integer handles values that don't fit in smaller types.
#[no_mangle]
pub extern "C" fn example_integer_overflow() -> rb_sys::VALUE {
    // Create a large integer
    let large = Integer::from_i64(i64::MAX);
    
    // Try to convert to smaller types with range checking
    // i32 conversion will fail for large values
    let val = large.into_value();
    
    // This would return an error for i64::MAX
    if i32::try_convert(val).is_err() {
        // As expected, doesn't fit in i32
        // Return the original large value
        return large.into_value().as_raw();
    }
    
    // If it somehow fit, return that
    val.as_raw()
}

/// Example 10: Demonstrating Float variants on different platforms
///
/// On 64-bit platforms, shows the difference between Flonum and RFloat.
#[no_mangle]
pub extern "C" fn example_float_variants() -> rb_sys::VALUE {
    // Small float - may be Flonum on 64-bit
    let small = Float::from_f64(1.5);
    
    #[cfg(target_pointer_width = "64")]
    {
        // Try to create a Flonum directly
        if let Some(flonum) = Flonum::from_f64(1.5) {
            // It's a Flonum (immediate value)
            let value = flonum.to_f64();
            assert!((value - 1.5).abs() < 0.001);
            return flonum.into_value().as_raw();
        }
    }
    
    // Either not 64-bit, or Ruby chose to heap-allocate
    // Return the Float union type
    small.into_value().as_raw()
}

/// Example 11: Round-trip conversion demonstrating fidelity
///
/// Shows that numeric conversions preserve values correctly.
#[no_mangle]
pub extern "C" fn example_numeric_round_trip() -> rb_sys::VALUE {
    // Test integer round-trip
    let original_int = 123456789i64;
    let int_val = original_int.into_value();
    let back_int = i64::try_convert(int_val).unwrap();
    assert_eq!(back_int, original_int);
    
    // Test float round-trip
    let original_float = std::f64::consts::PI;
    let float_val = original_float.into_value();
    let back_float = f64::try_convert(float_val).unwrap();
    assert!((back_float - original_float).abs() < 0.0000001);
    
    // Return a hash showing both values (requires Phase 2 Stage 6)
    // For now, just return true
    Qtrue::new().into_value().as_raw()
}

/// Example 12: Negative number handling
///
/// Shows that negative numbers work correctly with both types.
#[no_mangle]
pub extern "C" fn example_negative_numbers() -> rb_sys::VALUE {
    // Negative integer
    let neg_int = Integer::from_i64(-9876543210);
    assert_eq!(neg_int.to_i64().unwrap(), -9876543210);
    
    // Trying to convert negative to unsigned should fail
    assert!(neg_int.to_u64().is_err());
    
    // Negative float
    let neg_float = Float::from_f64(-123.456);
    assert!((neg_float.to_f64() + 123.456).abs() < 0.001);
    
    neg_int.into_value().as_raw()
}

/// Initialize the extension
#[no_mangle]
pub extern "C" fn Init_phase2_numeric_heap() {
    // Note: Full method definition requires Phase 3
    // For now, this is just a placeholder that Ruby will call when loading the extension
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_checks() {
        // These tests verify compile-time behavior only
        // Tests requiring Ruby API calls need the Ruby runtime
        
        // Verify numeric types are Copy
        fn assert_copy<T: Copy>() {}
        assert_copy::<Integer>();
        assert_copy::<Float>();
        assert_copy::<RBignum>();
        assert_copy::<RFloat>();
        
        #[cfg(target_pointer_width = "64")]
        assert_copy::<Flonum>();
    }
    
    #[test]
    fn test_type_sizes() {
        // Verify that wrapper types are the same size as Value
        use std::mem::size_of;
        
        assert_eq!(size_of::<RBignum>(), size_of::<Value>());
        assert_eq!(size_of::<RFloat>(), size_of::<Value>());
        
        #[cfg(target_pointer_width = "64")]
        assert_eq!(size_of::<Flonum>(), size_of::<Value>());
    }
    
    #[test]
    fn test_enum_variants_compile() {
        // Test that we can pattern match on Integer at compile time
        fn match_integer(int: Integer) -> &'static str {
            match int {
                Integer::Fixnum(_) => "fixnum",
                Integer::Bignum(_) => "bignum",
            }
        }
        
        // Test that we can pattern match on Float at compile time
        fn match_float(float: Float) -> &'static str {
            match float {
                #[cfg(target_pointer_width = "64")]
                Float::Flonum(_) => "flonum",
                Float::RFloat(_) => "rfloat",
            }
        }
        
        // Just verify the functions compile, don't call them (would require Ruby)
        let _ = match_integer;
        let _ = match_float;
    }
}
