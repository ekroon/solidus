//! Phase 2 Stage 1: Conversion Traits Example
//!
//! This example demonstrates the `TryConvert` and `IntoValue` traits that form
//! the foundation for converting between Ruby and Rust types.
//!
//! Since Stage 1 only implements the trait infrastructure with identity conversions
//! for `Value`, this example shows how the traits work at a fundamental level.
//! Later stages will add implementations for specific types like strings, integers,
//! arrays, etc.

use solidus::prelude::*;

/// Example 1: Identity conversion with IntoValue
/// 
/// Shows that a Value can be converted back to a Value (identity conversion).
#[no_mangle]
pub extern "C" fn example_identity_into_value(val: rb_sys::VALUE) -> rb_sys::VALUE {
    // Wrap the raw Ruby VALUE (unsafe as we trust Ruby to pass valid VALUEs)
    let value = unsafe { Value::from_raw(val) };
    
    // Convert it back using IntoValue (identity conversion)
    let result = value.into_value();
    
    result.as_raw()
}

/// Example 2: Identity conversion with TryConvert
///
/// Shows that a Value can be converted to a Value via TryConvert (always succeeds).
#[no_mangle]
pub extern "C" fn example_identity_try_convert(val: rb_sys::VALUE) -> rb_sys::VALUE {
    // Wrap the raw Ruby VALUE (unsafe as we trust Ruby to pass valid VALUEs)
    let value = unsafe { Value::from_raw(val) };
    
    // Try to convert it back to a Value
    match Value::try_convert(value) {
        Ok(result) => result.as_raw(),
        Err(e) => e.raise(),
    }
}

/// Example 3: Working with nil values
///
/// Demonstrates converting nil using both traits.
#[no_mangle]
pub extern "C" fn example_nil_conversions() -> rb_sys::VALUE {
    // Get a nil value
    let nil_value = Value::nil();
    
    // Convert using IntoValue
    let via_into = nil_value.into_value();
    
    // Convert using TryConvert
    let via_try = match Value::try_convert(nil_value) {
        Ok(v) => v,
        Err(e) => e.raise(),
    };
    
    // Both should be the same
    assert_eq!(via_into.as_raw(), via_try.as_raw());
    
    via_into.as_raw()
}

/// Example 4: Generic function using IntoValue
///
/// Shows how generic functions can accept any type that implements IntoValue.
fn convert_to_value<T: IntoValue>(item: T) -> Value {
    item.into_value()
}

#[no_mangle]
pub extern "C" fn example_generic_into_value(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };
    
    // Use the generic function
    let result = convert_to_value(value);
    
    result.as_raw()
}

/// Example 5: Generic function using TryConvert
///
/// Shows how generic functions can convert from Value to any type implementing TryConvert.
fn convert_from_value<T: TryConvert>(val: Value) -> Result<T, Error> {
    T::try_convert(val)
}

#[no_mangle]
pub extern "C" fn example_generic_try_convert(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };
    
    // Use the generic function to convert Value -> Value
    match convert_from_value::<Value>(value) {
        Ok(result) => result.as_raw(),
        Err(e) => e.raise(),
    }
}

/// Example 6: Chaining conversions
///
/// Demonstrates that conversions can be chained together.
#[no_mangle]
pub extern "C" fn example_chained_conversions(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };
    
    // Chain: Value -> IntoValue -> TryConvert -> Value
    let result = match Value::try_convert(value.into_value()) {
        Ok(v) => v,
        Err(e) => e.raise(),
    };
    
    result.as_raw()
}

/// Initialize the extension
#[no_mangle]
pub extern "C" fn Init_phase2_conversions() {
    // Note: Full method definition requires Phase 3
    // For now, this is just a placeholder that Ruby will call when loading the extension
}

// Note: Unit tests that call Ruby API functions (like Value::nil()) require
// the Ruby runtime to be initialized. These tests would need to use
// rb-sys-test-helpers with #[ruby_test] annotation.
//
// For now, the trait implementations are tested via the solidus crate tests.
// This example serves to demonstrate the pattern and verify compilation.
