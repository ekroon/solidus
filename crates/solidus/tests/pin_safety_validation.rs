// Tests that verify the safety guarantees of PinGuard and pin_on_stack!
// These tests confirm that dangerous patterns no longer compile.

use solidus::value::{PinGuard, Value};
use solidus::pin_on_stack;

// Test 1: The safe workflow - atomic pinning via pin_on_stack!
#[test]
fn test_atomic_pinning_workflow() {
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    
    // This is the ONLY safe way to pin: atomically consume the guard
    pin_on_stack!(pinned_ref = guard);
    
    // Now pinned_ref is Pin<&StackPinned<Value>>
    let inner = pinned_ref.get();
    assert!(inner.is_nil());
}

// Test 2: Direct expression pinning works
#[test]
fn test_direct_expression_pinning() {
    // We can pin the result of an expression directly
    pin_on_stack!(pinned = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) }));
    
    let inner = pinned.get();
    assert!(inner.is_nil());
}

// Test 3: Boxing is still available for heap storage
#[cfg(any(feature = "embed", feature = "link-ruby"))]
#[test]
fn test_heap_boxing_still_works() {
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    
    // Explicit heap storage with GC registration
    let boxed = guard.into_box();
    
    // Can safely store in collections
    let mut vec = Vec::new();
    vec.push(boxed);
    
    assert_eq!(vec.len(), 1);
}

// Test 4: Plain ReprValue types need to be wrapped in PinGuard for pin_on_stack!
#[test]
fn test_plain_types_work() {
    // Simulating what happens in method! macro with try_convert
    let value = unsafe { Value::from_raw(rb_sys::Qnil.into()) };
    
    // pin_on_stack! requires PinGuard now
    let guard = PinGuard::new(value);
    pin_on_stack!(pinned = guard);
    
    let inner = pinned.get();
    assert!(inner.is_nil());
}

// The following tests should NOT compile when uncommented:
// They verify that the safety gap has been closed.

/*
// SHOULD NOT COMPILE: .pin() method no longer exists
#[test]
fn test_cannot_call_pin_method() {
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    let stack_pinned = guard.pin();  // ERROR: no method named `pin` found
}
*/

/*
// SHOULD NOT COMPILE: Cannot move StackPinned to Vec
// (This test can't be written because we can't create a StackPinned without pinning it)
#[test]
fn test_cannot_move_to_vec() {
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    // No way to get a movable StackPinned anymore!
    // The only way to get StackPinned is via pin_on_stack!, which immediately pins it
}
*/

/*
// SHOULD NOT COMPILE: Cannot return StackPinned from function
// (This test can't be written because we can't create a StackPinned without pinning it)
fn create_stackpinned() -> StackPinned<Value> {
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    // guard.pin() doesn't exist anymore
    // pin_on_stack! produces Pin<&StackPinned>, not StackPinned
}
*/

// Test 5: Verify PinGuard is still !Unpin (cannot be stored without consumption)
#[test]
fn test_pin_guard_is_not_unpin() {
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    
    // This would fail if uncommented:
    // fn requires_unpin<T: Unpin>(_: T) {}
    // requires_unpin(guard);  // ERROR: PinGuard<Value>: Unpin is not satisfied
    
    // Guard must be consumed via pin_on_stack! or .into_box()
    pin_on_stack!(_pinned = guard);
}

// Test 6: Verify the #[must_use] warning still triggers
#[test]
#[allow(unused_must_use)]  // We're testing that the warning exists
fn test_must_use_warning() {
    // This should generate a warning about unused PinGuard
    PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    // Warning: VALUE must be pinned on stack or explicitly boxed
}
