//! Phase 2 Stage 2: Immediate Types Example
//!
//! This example demonstrates Ruby's immediate types (nil, true, false, fixnum, symbol, flonum).
//! Immediate values are encoded directly in the VALUE and don't require GC protection or pinning.
//!
//! This shows Stage 2 implementation: immediate value wrappers with type-safe conversions.

use solidus::prelude::*;

/// Example 1: Working with nil, true, and false
///
/// Demonstrates the Qnil, Qtrue, and Qfalse singleton types.
#[no_mangle]
pub extern "C" fn example_booleans() -> rb_sys::VALUE {
    // Create Ruby singletons
    let nil_val = Qnil::new();
    let true_val = Qtrue::new();
    let false_val = Qfalse::new();
    
    // Convert to Value
    assert!(nil_val.as_value().is_nil());
    assert!(true_val.as_value().is_true());
    assert!(false_val.as_value().is_false());
    
    // Try converting Values back to specific types
    assert!(Qnil::try_convert(nil_val.as_value()).is_ok());
    assert!(Qtrue::try_convert(true_val.as_value()).is_ok());
    assert!(Qfalse::try_convert(false_val.as_value()).is_ok());
    
    // Cross-type conversion should fail
    assert!(Qtrue::try_convert(nil_val.as_value()).is_err());
    assert!(Qfalse::try_convert(true_val.as_value()).is_err());
    
    true_val.into_value().as_raw()
}

/// Example 2: Rust bool to Ruby conversions
///
/// Shows how Rust booleans map to Ruby true/false.
#[no_mangle]
pub extern "C" fn example_rust_bool(rust_bool: bool) -> rb_sys::VALUE {
    // Rust bool -> Ruby true/false
    let ruby_value = rust_bool.into_value();
    
    // Ruby follows its truthiness rules
    if rust_bool {
        assert!(ruby_value.is_true());
    } else {
        assert!(ruby_value.is_false());
    }
    
    ruby_value.as_raw()
}

/// Example 3: Ruby truthiness
///
/// Demonstrates Ruby's truthiness rules: only nil and false are falsy.
#[no_mangle]
pub extern "C" fn example_truthiness(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };
    
    // Convert Ruby value to Rust bool using Ruby's truthiness rules
    let is_truthy = bool::try_convert(value).unwrap();
    
    is_truthy.into_value().as_raw()
}

/// Example 4: Working with Fixnum (small integers)
///
/// Demonstrates fixnum creation and conversion.
#[no_mangle]
pub extern "C" fn example_fixnum(n: i64) -> rb_sys::VALUE {
    // Create a Fixnum (panics if too large for current implementation)
    let fixnum = Fixnum::from_i64(n).expect("value should fit in Fixnum");
    
    // Get the value back
    assert_eq!(fixnum.to_i64(), n);
    
    // Convert to Value
    let value = fixnum.into_value();
    assert!(!value.is_nil());
    
    value.as_raw()
}

/// Example 5: Integer type conversions
///
/// Shows conversion between Ruby integers and various Rust integer types.
#[no_mangle]
pub extern "C" fn example_integer_conversions(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };
    
    // Try converting to different integer types
    if let Ok(n) = i32::try_convert(value) {
        // Successfully converted to i32
        let doubled = n * 2;
        return doubled.into_value().as_raw();
    }
    
    // If it doesn't fit in i32, try i64
    if let Ok(n) = i64::try_convert(value) {
        let doubled = n * 2;
        return doubled.into_value().as_raw();
    }
    
    // Not an integer
    Qnil::new().into_value().as_raw()
}

/// Example 6: Working with Symbols
///
/// Demonstrates symbol creation and interning.
#[no_mangle]
pub extern "C" fn example_symbols() -> rb_sys::VALUE {
    // Create symbols
    let sym1 = Symbol::new("hello");
    let sym2 = Symbol::new("hello");
    let sym3 = Symbol::new("world");
    
    // Symbols are interned - same string = same symbol
    assert_eq!(sym1.as_value(), sym2.as_value());
    assert_ne!(sym1.as_value(), sym3.as_value());
    
    // Get symbol name
    assert_eq!(sym1.name().unwrap(), "hello");
    assert_eq!(sym3.name().unwrap(), "world");
    
    sym1.into_value().as_raw()
}

/// Example 7: Symbol from &str
///
/// Shows direct conversion from Rust string slices to Ruby symbols.
#[no_mangle]
pub extern "C" fn example_str_to_symbol() -> rb_sys::VALUE {
    // &str can be directly converted to a Ruby symbol
    let sym_value = "test_symbol".into_value();
    
    // Verify it's a symbol
    let sym = Symbol::try_convert(sym_value).unwrap();
    assert_eq!(sym.name().unwrap(), "test_symbol");
    
    sym_value.as_raw()
}

/// Example 8: Working with floats
///
/// Demonstrates float conversions (f32, f64).
#[no_mangle]
pub extern "C" fn example_floats(f: f64) -> rb_sys::VALUE {
    // Convert f64 to Ruby
    let ruby_float = f.into_value();
    
    // Convert back
    let back_to_rust = f64::try_convert(ruby_float).unwrap();
    assert!((back_to_rust - f).abs() < 0.00001);
    
    // f32 also works
    let f32_val = 2.5f32.into_value();
    let back = f32::try_convert(f32_val).unwrap();
    assert!((back - 2.5f32).abs() < 0.00001);
    
    ruby_float.as_raw()
}

#[cfg(target_pointer_width = "64")]
/// Example 9: Flonum (immediate floats on 64-bit platforms)
///
/// On 64-bit platforms, small floats can be immediate values.
#[no_mangle]
pub extern "C" fn example_flonum_64bit() -> rb_sys::VALUE {
    // Try to create a Flonum
    if let Some(flonum) = Flonum::from_f64(1.5) {
        let value = flonum.to_f64();
        assert!((value - 1.5).abs() < 0.00001);
        flonum.into_value().as_raw()
    } else {
        // Some floats require heap allocation
        1.5f64.into_value().as_raw()
    }
}

/// Example 10: Type-safe immediate value handling
///
/// Shows how immediate values don't need pinning in function signatures.
fn process_immediate_values(count: i64, name: Symbol, enabled: bool) -> Value {
    // All of these are immediate values, so they can be passed directly
    // without Pin<&StackPinned<T>> wrappers
    
    if enabled {
        Symbol::new(&format!("{}_{}", name.name().unwrap(), count)).into_value()
    } else {
        Qnil::new().into_value()
    }
}

#[no_mangle]
pub extern "C" fn example_immediate_function() -> rb_sys::VALUE {
    let result = process_immediate_values(
        42,
        Symbol::new("test"),
        true
    );
    
    result.as_raw()
}

/// Initialize the extension
#[no_mangle]
pub extern "C" fn Init_phase2_conversions() {
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
        
        // Verify immediate types are Copy
        fn assert_copy<T: Copy>() {}
        assert_copy::<Qnil>();
        assert_copy::<Qtrue>();
        assert_copy::<Qfalse>();
        assert_copy::<Fixnum>();
        assert_copy::<Symbol>();
        
        #[cfg(target_pointer_width = "64")]
        assert_copy::<Flonum>();
    }
}
