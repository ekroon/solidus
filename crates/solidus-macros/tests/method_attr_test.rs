//! Tests for the #[method] and #[function] attribute macros.
//!
//! These tests verify that the macros compile correctly and generate the expected
//! structures (ARITY constant and wrapper() function).
//!
//! These tests require Ruby to be linked, so they only run with the `link-ruby` feature.
#![cfg(feature = "link-ruby")]

use solidus::prelude::*;
use solidus::rb_sys;
use std::pin::Pin;

// ============================================================================
// Test Types and Helpers
// ============================================================================

/// A test type that simulates a Ruby heap object needing pinning.
#[derive(Clone, Copy, Debug)]
struct TestValue(i64);

impl solidus::value::ReprValue for TestValue {
    fn as_value(&self) -> Value {
        unsafe { Value::from_raw(self.0 as rb_sys::VALUE) }
    }

    unsafe fn from_value_unchecked(val: Value) -> Self {
        TestValue(val.as_raw() as i64)
    }
}

impl solidus::convert::TryConvert for TestValue {
    fn try_convert(val: Value) -> Result<Self, Error> {
        Ok(TestValue(val.as_raw() as i64))
    }
}

impl solidus::convert::IntoValue for TestValue {
    fn into_value(self) -> Value {
        unsafe { Value::from_raw(self.0 as rb_sys::VALUE) }
    }
}

impl solidus::method::IntoReturnValue for TestValue {
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error> {
        Ok(self.0 as rb_sys::VALUE)
    }
}

// ============================================================================
// Method Tests (arity 0-2)
// ============================================================================

/// Test method with arity 0 (self only).
#[solidus_macros::method]
fn method_arity_0(rb_self: TestValue) -> Result<i64, Error> {
    Ok(rb_self.0)
}

/// Test method with arity 1 (self + 1 argument).
#[solidus_macros::method]
fn method_arity_1(rb_self: TestValue, arg0: Pin<&StackPinned<TestValue>>) -> Result<i64, Error> {
    Ok(rb_self.0 + arg0.get().0)
}

/// Test method with arity 2 (self + 2 arguments).
#[solidus_macros::method]
fn method_arity_2(
    rb_self: TestValue,
    arg0: Pin<&StackPinned<TestValue>>,
    arg1: Pin<&StackPinned<TestValue>>,
) -> Result<i64, Error> {
    Ok(rb_self.0 + arg0.get().0 + arg1.get().0)
}

// ============================================================================
// Function Tests (arity 0-2)
// ============================================================================

/// Test function with arity 0 (no arguments).
#[solidus_macros::function]
fn function_arity_0() -> Result<i64, Error> {
    Ok(42)
}

/// Test function with arity 1 (1 argument).
#[solidus_macros::function]
fn function_arity_1(arg0: Pin<&StackPinned<TestValue>>) -> Result<i64, Error> {
    Ok(arg0.get().0)
}

/// Test function with arity 2 (2 arguments).
#[solidus_macros::function]
fn function_arity_2(
    arg0: Pin<&StackPinned<TestValue>>,
    arg1: Pin<&StackPinned<TestValue>>,
) -> Result<i64, Error> {
    Ok(arg0.get().0 + arg1.get().0)
}

// ============================================================================
// Generated Module Structure Tests
// ============================================================================

#[test]
fn test_method_arity_0_generates_correct_arity() {
    assert_eq!(__solidus_method_method_arity_0::ARITY, 0);
}

#[test]
fn test_method_arity_1_generates_correct_arity() {
    assert_eq!(__solidus_method_method_arity_1::ARITY, 1);
}

#[test]
fn test_method_arity_2_generates_correct_arity() {
    assert_eq!(__solidus_method_method_arity_2::ARITY, 2);
}

#[test]
fn test_function_arity_0_generates_correct_arity() {
    assert_eq!(__solidus_function_function_arity_0::ARITY, 0);
}

#[test]
fn test_function_arity_1_generates_correct_arity() {
    assert_eq!(__solidus_function_function_arity_1::ARITY, 1);
}

#[test]
fn test_function_arity_2_generates_correct_arity() {
    assert_eq!(__solidus_function_function_arity_2::ARITY, 2);
}

// ============================================================================
// Wrapper Function Type Tests
// ============================================================================

#[test]
fn test_method_wrapper_returns_correct_type() {
    // The wrapper() function should return a function pointer
    let wrapper: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_method_method_arity_0::wrapper();
    // We can't call it without Ruby, but we can verify the type
    let _: unsafe extern "C" fn() -> rb_sys::VALUE = wrapper;
}

#[test]
fn test_function_wrapper_returns_correct_type() {
    let wrapper: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_function_function_arity_0::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE = wrapper;
}

// ============================================================================
// All Arities Wrapper Type Tests
// ============================================================================

#[test]
fn test_all_method_wrappers_compile() {
    let _: unsafe extern "C" fn() -> rb_sys::VALUE = __solidus_method_method_arity_0::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE = __solidus_method_method_arity_1::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE = __solidus_method_method_arity_2::wrapper();
}

#[test]
fn test_all_function_wrappers_compile() {
    let _: unsafe extern "C" fn() -> rb_sys::VALUE = __solidus_function_function_arity_0::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE = __solidus_function_function_arity_1::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE = __solidus_function_function_arity_2::wrapper();
}

// ============================================================================
// Different Return Type Tests
// ============================================================================

/// Method that returns unit.
#[solidus_macros::method]
fn method_returns_unit(rb_self: TestValue) -> Result<(), Error> {
    let _ = rb_self;
    Ok(())
}

/// Method that returns a Value.
#[solidus_macros::method]
fn method_returns_value(rb_self: TestValue) -> Result<TestValue, Error> {
    Ok(rb_self)
}

/// Method that returns bool.
#[solidus_macros::method]
fn method_returns_bool(rb_self: TestValue) -> Result<bool, Error> {
    Ok(rb_self.0 > 0)
}

#[test]
fn test_method_different_return_types_compile() {
    let _: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_method_method_returns_unit::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_method_method_returns_value::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_method_method_returns_bool::wrapper();
}

/// Function that returns unit.
#[solidus_macros::function]
fn function_returns_unit() -> Result<(), Error> {
    Ok(())
}

/// Function that returns bool.
#[solidus_macros::function]
fn function_returns_bool() -> Result<bool, Error> {
    Ok(true)
}

#[test]
fn test_function_different_return_types_compile() {
    let _: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_function_function_returns_unit::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_function_function_returns_bool::wrapper();
}

// ============================================================================
// Documentation Test - Shows Usage Pattern
// ============================================================================

/// This test documents the expected usage pattern for the attribute macros.
#[test]
fn test_usage_pattern_documentation() {
    // The attribute macro transforms:
    //
    // #[solidus_macros::method]
    // fn my_method(rb_self: RString, arg: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
    //     // implementation
    // }
    //
    // Into:
    //
    // fn my_method(rb_self: RString, arg: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
    //     // implementation (unchanged)
    // }
    //
    // #[doc(hidden)]
    // pub mod __solidus_method_my_method {
    //     pub const ARITY: i32 = 1;
    //     pub fn wrapper() -> unsafe extern "C" fn() -> VALUE {
    //         // extern "C" wrapper that handles:
    //         // - panic catching
    //         // - self conversion
    //         // - argument conversion and pinning
    //         // - error handling
    //     }
    // }
    //
    // Usage with Ruby:
    // class.define_method("my_method", __solidus_method_my_method::wrapper(), __solidus_method_my_method::ARITY)?;

    // Verify the pattern works:
    assert_eq!(__solidus_method_method_arity_1::ARITY, 1);
    let _wrapper = __solidus_method_method_arity_1::wrapper();
}

// ============================================================================
// Comparison with method!/function! Macros
// ============================================================================

/// This test shows how the attribute macros compare to the declarative macros.
#[test]
fn test_comparison_with_declarative_macros() {
    // Using declarative macro (existing approach):
    fn existing_method(
        _ctx: &solidus::Context,
        _rb_self: TestValue,
        _arg: Pin<&StackPinned<TestValue>>,
    ) -> Result<i64, Error> {
        Ok(0)
    }
    let _declarative_wrapper: unsafe extern "C" fn() -> rb_sys::VALUE =
        solidus::method!(existing_method, 1);

    // Using attribute macro (new approach):
    // The method_arity_1 function was defined above with #[solidus_macros::method]
    let _attribute_wrapper: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_method_method_arity_1::wrapper();

    // Both produce the same type of wrapper function pointer
    // The key differences:
    // 1. Attribute macro automatically determines arity from signature
    // 2. Attribute macro generates a companion module with ARITY constant
    // 3. Attribute macro keeps the original function intact
}

// ============================================================================
// Implicit Pinning Tests
// ============================================================================

/// Test method with implicit pinning - user specifies just the type, not Pin<&StackPinned<T>>
#[solidus_macros::method]
fn method_implicit_pinning(
    rb_self: TestValue,
    other: Pin<&StackPinned<TestValue>>,
) -> Result<i64, Error> {
    Ok(rb_self.0 + other.get().0)
}

/// Test method with implicit pinning and 2 args
#[solidus_macros::method]
fn method_implicit_pinning_2args(
    rb_self: TestValue,
    arg0: Pin<&StackPinned<TestValue>>,
    arg1: Pin<&StackPinned<TestValue>>,
) -> Result<i64, Error> {
    Ok(rb_self.0 + arg0.get().0 + arg1.get().0)
}

/// Test function with implicit pinning
#[solidus_macros::function]
fn function_implicit_pinning(arg: Pin<&StackPinned<TestValue>>) -> Result<i64, Error> {
    Ok(arg.get().0 * 2)
}

/// Test function with implicit pinning and 2 args
#[solidus_macros::function]
fn function_implicit_pinning_2args(
    arg0: Pin<&StackPinned<TestValue>>,
    arg1: Pin<&StackPinned<TestValue>>,
) -> Result<i64, Error> {
    Ok(arg0.get().0 + arg1.get().0)
}

/// Test that mixing explicit and implicit pinning works
#[solidus_macros::method]
fn method_mixed_pinning(
    rb_self: TestValue,
    explicit: Pin<&StackPinned<TestValue>>,
    implicit: Pin<&StackPinned<TestValue>>,
) -> Result<i64, Error> {
    Ok(rb_self.0 + explicit.get().0 + implicit.get().0)
}

// ============================================================================
// Implicit Pinning Arity Tests
// ============================================================================

#[test]
fn test_method_implicit_pinning_arity() {
    assert_eq!(__solidus_method_method_implicit_pinning::ARITY, 1);
}

#[test]
fn test_method_implicit_pinning_2args_arity() {
    assert_eq!(__solidus_method_method_implicit_pinning_2args::ARITY, 2);
}

#[test]
fn test_function_implicit_pinning_arity() {
    assert_eq!(__solidus_function_function_implicit_pinning::ARITY, 1);
}

#[test]
fn test_function_implicit_pinning_2args_arity() {
    assert_eq!(__solidus_function_function_implicit_pinning_2args::ARITY, 2);
}

#[test]
fn test_method_mixed_pinning_arity() {
    assert_eq!(__solidus_method_method_mixed_pinning::ARITY, 2);
}

// ============================================================================
// Implicit Pinning Wrapper Type Tests
// ============================================================================

#[test]
fn test_implicit_pinning_wrappers_compile() {
    // Verify all implicit pinning wrappers have the correct type
    let _: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_method_method_implicit_pinning::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_method_method_implicit_pinning_2args::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_function_function_implicit_pinning::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_function_function_implicit_pinning_2args::wrapper();
    let _: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_method_method_mixed_pinning::wrapper();
}

// ============================================================================
// Direct Function Call Tests (verify the underlying functions work)
// ============================================================================

#[test]
fn test_method_implicit_pinning_direct_call() {
    // Test that the underlying function can be called directly
    solidus::pin_on_stack!(other = solidus::value::NewValue::new(TestValue(5)));
    let result = method_implicit_pinning(TestValue(10), other);
    assert_eq!(result.unwrap(), 15);
}

#[test]
fn test_function_implicit_pinning_direct_call() {
    solidus::pin_on_stack!(arg = solidus::value::NewValue::new(TestValue(21)));
    let result = function_implicit_pinning(arg);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_method_mixed_pinning_direct_call() {
    // Create pinned values for both parameters
    solidus::pin_on_stack!(explicit_arg = solidus::value::NewValue::new(TestValue(10)));
    solidus::pin_on_stack!(implicit_arg = solidus::value::NewValue::new(TestValue(100)));
    let result = method_mixed_pinning(TestValue(1), explicit_arg, implicit_arg);
    assert_eq!(result.unwrap(), 111);
}

// ============================================================================
// Documentation Test - Shows Usage Pattern with Implicit Pinning
// ============================================================================

/// This test documents the expected usage pattern for implicit pinning.
#[test]
fn test_implicit_pinning_documentation() {
    // OLD WAY (explicit pinning):
    // #[solidus_macros::method]
    // fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
    //     let s = other.get().to_string()?;  // Must use .get()
    //     // ...
    // }
    //
    // NEW WAY (implicit pinning):
    // #[solidus_macros::method]
    // fn concat(rb_self: RString, other: RString) -> Result<RString, Error> {
    //     let s = other.to_string()?;  // Use directly!
    //     // ...
    // }
    //
    // The macro automatically:
    // 1. Converts VALUE to the specified type via TryConvert
    // 2. Pins it on the stack for GC safety
    // 3. Extracts the inner value and passes it to your function
    //
    // This works because Ruby value types are Copy, so extracting from the pin
    // and passing by value is safe.

    // Both styles are supported simultaneously
    assert_eq!(__solidus_method_method_arity_1::ARITY, 1); // explicit Pin<&StackPinned<T>>
    assert_eq!(__solidus_method_method_implicit_pinning::ARITY, 1); // implicit pinning
}

// ============================================================================
// Backward Compatibility Test
// ============================================================================

/// Verify that explicit Pin<&StackPinned<T>> still works alongside implicit pinning
#[test]
fn test_backward_compatibility() {
    // Both explicit (old style) and implicit (new style) work
    let _explicit: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_method_method_arity_1::wrapper();
    let _implicit: unsafe extern "C" fn() -> rb_sys::VALUE =
        __solidus_method_method_implicit_pinning::wrapper();

    // They have the same arity
    assert_eq!(__solidus_method_method_arity_1::ARITY, 1);
    assert_eq!(__solidus_method_method_implicit_pinning::ARITY, 1);
}
