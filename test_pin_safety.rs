// Test file to investigate PinGuard → StackPinned safety
// This file tests various scenarios to see if we can accidentally
// move StackPinned values to the heap without proper GC registration.

use solidus::value::{PinGuard, StackPinned, Value, ReprValue};
use solidus::pin_on_stack;

// Scenario 1: With the new API, we can't get a movable StackPinned
// The .pin() method has been removed to prevent this scenario
#[allow(dead_code)]
fn test_vec_storage() {
    // Create a guard
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    
    // The only way to use the guard is:
    // 1. pin_on_stack! - which creates Pin<&StackPinned>, not movable
    // 2. into_box() - which explicitly registers with GC
    
    // This scenario is now impossible - we can't get a movable StackPinned!
    pin_on_stack!(pinned = guard);
    let _inner = pinned.get();
}

// Scenario 2: With the new API, pin_on_stack! is the only way
#[allow(dead_code)]
fn test_without_macro() {
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    
    // The only safe way is pin_on_stack!, which atomically:
    // 1. Consumes the guard
    // 2. Creates StackPinned<T>
    // 3. Pins it immediately
    pin_on_stack!(pinned = guard);
    
    // Now we have Pin<&StackPinned<Value>>, which is safe
    let _inner = pinned.get();
}

// Scenario 3: With the new API, we can't create movable StackPinned values
// This scenario is now impossible - StackPinned can only exist behind Pin<&_>

#[allow(dead_code)]
fn test_struct_storage() {
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    
    // We can't get a movable StackPinned anymore
    // pin_on_stack! creates Pin<&StackPinned>, which can't be moved into structs
    pin_on_stack!(pinned = guard);
    
    // If we need heap storage, we must use into_box():
    let guard2 = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    let _boxed = guard2.into_box();  // Properly registered with GC
}

// Scenario 4: The proper workflow with new API
#[allow(dead_code)]
fn test_proper_workflow() {
    // Step 1: Create a Ruby value (returns PinGuard)
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    
    // Step 2: Use pin_on_stack! to atomically pin
    // This consumes the guard and creates Pin<&StackPinned<T>>
    pin_on_stack!(pinned_ref = guard);
    
    // Now pinned_ref is Pin<&StackPinned<Value>>
    // This is the safe type we want
    let inner: &Value = pinned_ref.get();
    
    drop(inner);
}

// Scenario 5: No more double wrapping - simplified API
#[allow(dead_code)]
fn test_double_wrapping() {
    // With the new API, we go directly from PinGuard to Pin<&StackPinned<T>>
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    
    // One-step pinning
    pin_on_stack!(pinned_value = guard);
    
    // pinned_value is Pin<&StackPinned<Value>>
    let inner = pinned_value.get();  // This is &Value
    
    drop(inner);
}

// Scenario 6: With the new API, we don't need .pin()
#[allow(dead_code)]
fn test_skip_pin() {
    // Create a guard
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    
    // pin_on_stack! now accepts PinGuard directly
    pin_on_stack!(pinned = guard);
    
    // This works because the macro calls into_inner_for_macro on the guard
    let _inner = pinned.get();
}

// Scenario 7: With the new API, we return PinGuard, not StackPinned
#[allow(dead_code)]
fn create_pin_guard() -> PinGuard<Value> {
    let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
    guard  // Returns PinGuard<Value>
}

#[allow(dead_code)]
fn test_return_pin_guard() {
    // Get a PinGuard from a function
    let guard = create_pin_guard();
    
    // Pin it on the stack in the current frame
    pin_on_stack!(pinned = guard);
    
    // This is safe - the pinning happens in the current stack frame
    let inner = pinned.get();
    drop(inner);
}

fn main() {
    println!("This file is for compile-time analysis, not execution");
}
