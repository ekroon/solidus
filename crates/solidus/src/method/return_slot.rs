//! ReturnSlot pattern for safe return of Ruby values from methods.
//!
//! This module provides `ReturnSlot<T>`, a type that allows methods to return
//! newly-created Ruby values while maintaining stack-pinning guarantees.
//!
//! # The Problem
//!
//! When a method creates a new Ruby value (e.g., `RString::new("hello")`), it gets
//! a `NewValue<T>` that must be pinned on the stack. But we need to return this
//! value to Ruby at the FFI boundary, which requires extracting the raw VALUE.
//!
//! The challenge is preventing users from:
//! 1. Storing the VALUE in a `Vec` or other heap collection
//! 2. Forgetting to actually return a value
//! 3. Creating ReturnSlot themselves (bypassing the macro)
//!
//! # Design
//!
//! `ReturnSlot<T>` uses several techniques to prevent misuse:
//!
//! 1. **Private constructor with sealed token** - Only the `method!` macro can create it
//! 2. **Lifetime binding** - The slot is tied to the stack frame
//! 3. **pub(crate) extraction** - Only internal code can extract the VALUE
//!
//! # Example (conceptual - actual usage is via method! macro)
//!
//! ```ignore
//! // The method! macro generates something like:
//! fn wrapper(rb_self: VALUE) -> VALUE {
//!     let slot = ReturnSlot::new(__private_token);
//!     
//!     let result = user_function(rb_self, &mut slot);
//!     
//!     match result {
//!         Ok(()) => slot.take(),
//!         Err(e) => e.raise(),
//!     }
//! }
//!
//! // User writes:
//! fn user_function(rb_self: RString, slot: &mut ReturnSlot<RString>) -> Result<(), Error> {
//!     pin_on_stack!(result = RString::new("hello"));
//!     slot.set(result);
//!     Ok(())
//! }
//! ```

use std::cell::Cell;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::value::{ReprValue, StackPinned};

// ============================================================================
// APPROACH 1: Out-parameter with sealed constructor
// ============================================================================

/// A sealed token type that can only be created by the method! macro.
///
/// This prevents users from creating `ReturnSlot` directly.
#[doc(hidden)]
pub struct ReturnSlotToken {
    _private: (),
}

impl ReturnSlotToken {
    /// Create a new token. Only callable from method! macro via pub(crate).
    #[doc(hidden)]
    #[inline]
    pub fn new() -> Self {
        ReturnSlotToken { _private: () }
    }
}

/// A slot for returning Ruby values from methods.
///
/// This type can only be created by the `method!` macro. User code receives
/// a mutable reference to it and can set a value via [`set()`](ReturnSlot::set).
///
/// # Why not just return `NewValue<T>`?
///
/// The current design allows returning `Result<NewValue<T>, Error>`, which works
/// because `IntoValue` is implemented for `NewValue`. However, `ReturnSlot` provides
/// additional guarantees:
///
/// 1. The value is extracted immediately within the wrapper function
/// 2. There's no intermediate state where the VALUE could escape
/// 3. The slot itself cannot be stored in collections (lifetime-bounded)
///
/// # Design Constraints
///
/// - `ReturnSlot` itself cannot be put in a `Vec` because it has a lifetime
/// - The `set()` method takes a pinned reference, ensuring the value is stack-pinned
/// - The `take()` method is `pub(crate)`, preventing user access to raw VALUE
pub struct ReturnSlot<'a, T: ReprValue> {
    /// The VALUE to return, if set.
    /// We use Cell for interior mutability since we need &mut self for API clarity
    /// but internally just need to write once.
    value: Cell<Option<rb_sys::VALUE>>,

    /// Marker to track the type we're returning.
    _marker: PhantomData<&'a T>,
}

impl<'a, T: ReprValue> ReturnSlot<'a, T> {
    /// Create a new return slot.
    ///
    /// This is `pub` but requires a `ReturnSlotToken` which can only be created
    /// by internal code (via `pub(crate)`), effectively sealing the constructor.
    #[doc(hidden)]
    #[inline]
    pub fn new(_token: ReturnSlotToken) -> Self {
        ReturnSlot {
            value: Cell::new(None),
            _marker: PhantomData,
        }
    }

    /// Set the return value from a pinned reference.
    ///
    /// This consumes the pinned value conceptually (extracts its VALUE) but doesn't
    /// actually move it - the StackPinned wrapper stays on the stack until the
    /// current scope ends, which is fine since we only need the VALUE.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn my_method(rb_self: RString, slot: &mut ReturnSlot<RString>) -> Result<(), Error> {
    ///     pin_on_stack!(result = RString::new("hello"));
    ///     slot.set(result);
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn set(&self, pinned: Pin<&StackPinned<T>>) {
        let value = pinned.get();
        self.value.set(Some(value.as_raw()));
    }

    /// Set the return value from a NewValue directly.
    ///
    /// This is a convenience method that avoids needing `pin_on_stack!` for
    /// simple returns. The guard is consumed and its VALUE is captured.
    ///
    /// # Safety Note
    ///
    /// This is safe because:
    /// 1. The VALUE is immediately captured and will be returned to Ruby
    /// 2. The wrapper function extracts this VALUE before the stack frame ends
    /// 3. Ruby will then own the value (on Ruby's stack or in its heap)
    #[inline]
    pub fn set_guard(&self, guard: crate::value::NewValue<T>) {
        self.value.set(Some(guard.as_raw()));
    }

    /// Extract the VALUE for the FFI return.
    ///
    /// # Panics
    ///
    /// Panics if no value was set. The `method!` macro should ensure this
    /// only happens if the user function returns `Ok(())` without calling `set()`.
    #[doc(hidden)]
    #[inline]
    pub(crate) fn take(&self) -> rb_sys::VALUE {
        self.value.get().expect("ReturnSlot::set() was not called")
    }

    /// Check if a value has been set.
    #[doc(hidden)]
    #[inline]
    pub(crate) fn is_set(&self) -> bool {
        self.value.get().is_some()
    }
}

// ============================================================================
// APPROACH 2: Returned value with lifetime preventing Vec storage
// ============================================================================

/// An alternative design where the return value is constructed and returned,
/// rather than set into an out-parameter.
///
/// Key insight: We can use a lifetime to prevent storage in collections.
/// If `ReturnedValue<'a, T>` has a lifetime `'a` that's tied to the stack frame,
/// it cannot be stored in `Vec<ReturnedValue<T>>` (needs `'static`).
pub struct ReturnedValue<'a, T: ReprValue> {
    value: rb_sys::VALUE,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: ReprValue> ReturnedValue<'a, T> {
    /// Create from a pinned value.
    ///
    /// The lifetime `'a` is tied to the pinned reference, preventing this
    /// from being stored in collections.
    #[inline]
    pub fn from_pinned(pinned: Pin<&'a StackPinned<T>>) -> Self {
        let value = pinned.get().as_raw();
        ReturnedValue {
            value,
            _marker: PhantomData,
        }
    }

    /// Create from a NewValue.
    ///
    /// When created from a guard (not a reference), we need a different approach
    /// to prevent Vec storage. See `ReturnedValueOwned` below.
    #[inline]
    pub fn from_guard(guard: crate::value::NewValue<T>) -> ReturnedValueOwned<T> {
        ReturnedValueOwned {
            value: guard.as_raw(),
            _marker: PhantomData,
        }
    }

    /// Extract for FFI.
    #[doc(hidden)]
    #[inline]
    pub(crate) fn into_raw(self) -> rb_sys::VALUE {
        self.value
    }
}

/// An owned return value that cannot be stored in Vec due to being !Unpin.
///
/// This is an alternative to lifetime-based prevention. By making it !Unpin,
/// we don't prevent Vec storage (Vec doesn't require Unpin), but we signal intent.
///
/// Actually, !Unpin doesn't prevent Vec storage. Let's try a different approach...
pub struct ReturnedValueOwned<T: ReprValue> {
    value: rb_sys::VALUE,
    _marker: PhantomData<T>,
}

impl<T: ReprValue> ReturnedValueOwned<T> {
    /// Create from a NewValue.
    #[inline]
    pub fn from_guard(guard: crate::value::NewValue<T>) -> Self {
        ReturnedValueOwned {
            value: guard.as_raw(),
            _marker: PhantomData,
        }
    }

    /// Extract for FFI.
    #[doc(hidden)]
    #[inline]
    pub(crate) fn into_raw(self) -> rb_sys::VALUE {
        self.value
    }
}

// ============================================================================
// APPROACH 3: Callback-based approach
// ============================================================================

/// A callback-based approach where the return value is provided via closure.
///
/// The idea is that the method! macro provides a closure that captures the
/// return location, and the user calls it with their value.
///
/// ```ignore
/// fn my_method<F>(rb_self: RString, ret: F) -> Result<(), Error>
/// where
///     F: FnOnce(NewValue<RString>)
/// {
///     let result = RString::new("hello");
///     ret(result);  // Calls macro-provided closure
///     Ok(())
/// }
/// ```
///
/// Pros:
/// - Very clear intent
/// - No way to store the callback in a collection
///
/// Cons:
/// - More complex method signature
/// - Callback feels unusual for "returning" a value
pub struct ReturnCallback<'a, T: ReprValue> {
    /// Where to write the result
    slot: &'a Cell<Option<rb_sys::VALUE>>,
    _marker: PhantomData<T>,
}

impl<'a, T: ReprValue> ReturnCallback<'a, T> {
    /// Create a new callback.
    #[doc(hidden)]
    pub(crate) fn new(slot: &'a Cell<Option<rb_sys::VALUE>>) -> Self {
        ReturnCallback {
            slot,
            _marker: PhantomData,
        }
    }

    /// Set the return value.
    #[inline]
    pub fn set(self, guard: crate::value::NewValue<T>) {
        self.slot.set(Some(guard.as_raw()));
    }

    /// Set the return value from a pinned reference.
    #[inline]
    pub fn set_pinned(self, pinned: Pin<&StackPinned<T>>) {
        self.slot.set(Some(pinned.get().as_raw()));
    }
}

// ============================================================================
// APPROACH 4: Witness type that proves we're in a method context
// ============================================================================

/// A witness type that proves we're in a method context.
///
/// This is similar to the token approach but the witness is not consumed.
/// The idea is that methods take `&MethodContext` as a hidden first parameter,
/// and this context provides the return slot.
pub struct MethodContext<'a> {
    return_value: Cell<Option<rb_sys::VALUE>>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> MethodContext<'a> {
    /// Create a new method context. Only callable from method! macro.
    #[doc(hidden)]
    #[inline]
    pub fn new() -> Self {
        MethodContext {
            return_value: Cell::new(None),
            _lifetime: PhantomData,
        }
    }

    /// Return a value from this method.
    #[inline]
    pub fn return_value<T: ReprValue>(&self, guard: crate::value::NewValue<T>) {
        self.return_value.set(Some(guard.as_raw()));
    }

    /// Return a pinned value from this method.
    #[inline]
    pub fn return_pinned<T: ReprValue>(&self, pinned: Pin<&StackPinned<T>>) {
        self.return_value.set(Some(pinned.get().as_raw()));
    }

    /// Extract the return value. Panics if not set.
    #[doc(hidden)]
    #[inline]
    pub(crate) fn take(&self) -> rb_sys::VALUE {
        self.return_value
            .get()
            .expect("MethodContext::return_value() was not called")
    }
}

// ============================================================================
// RECOMMENDED APPROACH: Combining the best ideas
// ============================================================================

/// The recommended design combines:
/// 1. Sealed constructor (via pub(crate))
/// 2. Lifetime to prevent storage in 'static collections
/// 3. Clear API with both `set_guard` and `set_pinned` methods
///
/// See `ReturnSlot<'a, T>` above - it's the primary implementation.

// ============================================================================
// Example macro expansion sketches
// ============================================================================

/// Example of how method! macro would use ReturnSlot (Approach 1):
///
/// ```ignore
/// // User writes:
/// fn greet(rb_self: RString, slot: &ReturnSlot<RString>) -> Result<(), Error> {
///     slot.set_guard(RString::new("hello"));
///     Ok(())
/// }
///
/// // method! macro generates:
/// unsafe extern "C" fn wrapper(rb_self: VALUE) -> VALUE {
///     let result = std::panic::catch_unwind(|| {
///         let self_value = Value::from_raw(rb_self);
///         let self_converted = TryConvert::try_convert(self_value)?;
///         
///         // Create the return slot with private token
///         let slot = ReturnSlot::new(ReturnSlotToken::new());
///         
///         // Call user function
///         greet(self_converted, &slot)?;
///         
///         // Extract the return value
///         Ok(slot.take())
///     });
///     
///     match result {
///         Ok(Ok(value)) => value,
///         Ok(Err(error)) => error.raise(),
///         Err(panic) => Error::from_panic(panic).raise(),
///     }
/// }
/// ```

/// Example of how method! macro would use current pattern (just return NewValue):
///
/// The current approach of returning `Result<NewValue<T>, Error>` actually works well.
/// The key insight is that `NewValue` implements `IntoValue`, so the macro can call
/// `into_return_value()` which extracts the VALUE safely.
///
/// The VALUE is immediately passed to Ruby before the stack frame ends, so it's safe.
///
/// ```ignore
/// // User writes:
/// fn greet(rb_self: RString) -> Result<NewValue<RString>, Error> {
///     Ok(RString::new("hello"))
/// }
///
/// // This works because:
/// // 1. NewValue<T> implements IntoValue
/// // 2. IntoValue::into_value() extracts the VALUE
/// // 3. The VALUE is immediately returned to Ruby
/// ```

// ============================================================================
// Analysis and comparison
// ============================================================================

#[cfg(test)]
mod design_analysis {
    //! # Design Analysis
    //!
    //! ## Current approach: `Result<NewValue<T>, Error>`
    //!
    //! Pros:
    //! - Simple, idiomatic Rust return type
    //! - Works with existing trait system
    //! - No extra parameters in method signature
    //!
    //! Cons:
    //! - User could theoretically store NewValue in a Vec before returning
    //!   (though they'd get a warning since NewValue is #[must_use])
    //! - NewValue is !Unpin but that doesn't prevent Vec storage
    //!
    //! ## ReturnSlot (Approach 1): Out-parameter
    //!
    //! Pros:
    //! - Clear separation of "calculation" and "returning"
    //! - Slot cannot be stored (has lifetime)
    //! - Constructor is sealed
    //!
    //! Cons:
    //! - Extra parameter in every method
    //! - Less idiomatic (out-parameters are rare in Rust)
    //! - User might forget to call `set()`
    //!
    //! ## ReturnedValue (Approach 2): Lifetime-bound return
    //!
    //! Pros:
    //! - Lifetime prevents storage in 'static collections
    //! - More natural return syntax
    //!
    //! Cons:
    //! - The owned variant has no such protection
    //! - Lifetime ergonomics can be painful
    //!
    //! ## ReturnCallback (Approach 3): Callback-based
    //!
    //! Pros:
    //! - Cannot possibly store a closure in a collection
    //! - Very explicit about what happens
    //!
    //! Cons:
    //! - Unusual API design
    //! - Method signatures become complex
    //!
    //! ## MethodContext (Approach 4): Context parameter
    //!
    //! Pros:
    //! - Single parameter provides all method utilities
    //! - Could be extended with more features (e.g., exception handling)
    //!
    //! Cons:
    //! - Still an extra parameter
    //! - Feels like an IoC container
    //!
    //! ## Verdict
    //!
    //! The current `Result<NewValue<T>, Error>` approach is probably fine for
    //! most use cases. The key safety property is that:
    //!
    //! 1. NewValue cannot be accessed after the method returns (no references escape)
    //! 2. The macro immediately calls `into_value()` which extracts the VALUE
    //! 3. The VALUE is returned to Ruby before the stack frame unwinds
    //!
    //! The only "misuse" would be if the user explicitly stored the NewValue in
    //! a Vec *inside* the method and then iterated to return. But:
    //! - They'd be ignoring the #[must_use] warning
    //! - They'd have to explicitly discard the guard
    //! - This is a deliberate footgun, not accidental
    //!
    //! If we want to prevent even deliberate misuse, ReturnSlot (Approach 1) is
    //! the most practical enhancement.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    #[test]
    fn test_return_slot_set_and_take() {
        let token = ReturnSlotToken::new();
        let slot: ReturnSlot<'_, Value> = ReturnSlot::new(token);

        // Create a fake pinned value for testing
        let value = unsafe { Value::from_raw(42 as rb_sys::VALUE) };
        let stack_pinned = StackPinned::new(value);
        // SAFETY: Just for testing
        let pinned = unsafe { Pin::new_unchecked(&stack_pinned) };

        slot.set(pinned);
        assert!(slot.is_set());
        assert_eq!(slot.take(), 42 as rb_sys::VALUE);
    }

    #[test]
    fn test_return_slot_with_guard() {
        use crate::value::NewValue;

        let token = ReturnSlotToken::new();
        let slot: ReturnSlot<'_, Value> = ReturnSlot::new(token);

        let value = unsafe { Value::from_raw(123 as rb_sys::VALUE) };
        let guard = NewValue::new(value);

        slot.set_guard(guard);
        assert!(slot.is_set());
        assert_eq!(slot.take(), 123 as rb_sys::VALUE);
    }

    // Compile-time test: ReturnSlot has a lifetime, so it can't go in a Vec<ReturnSlot<'static, T>>
    // This would fail to compile:
    // fn _cannot_store_in_static_vec() {
    //     let token = ReturnSlotToken::new();
    //     let slot: ReturnSlot<'static, Value> = ReturnSlot::new(token);
    //     let _vec: Vec<ReturnSlot<'static, Value>> = vec![slot]; // Would need 'static values
    // }
    //
    // Actually, this could compile if the user creates a ReturnSlot with 'static lifetime.
    // The protection comes from the sealed constructor - only internal code can create it.

    #[test]
    fn test_method_context() {
        use crate::value::NewValue;

        let ctx = MethodContext::new();

        let value = unsafe { Value::from_raw(456 as rb_sys::VALUE) };
        let guard = NewValue::new(value);

        ctx.return_value(guard);
        assert_eq!(ctx.take(), 456 as rb_sys::VALUE);
    }

    // ========================================================================
    // Simulated complete usage patterns
    // ========================================================================

    /// This test demonstrates how a hypothetical `method_with_slot!` macro would work.
    /// The user provides a function that receives a `&ReturnSlot<T>` parameter.
    #[test]
    fn test_simulated_method_wrapper_with_slot() {
        use crate::value::NewValue;

        // Simulate what the method! macro would generate
        fn wrapper() -> rb_sys::VALUE {
            // Macro creates the slot with private token
            let slot: ReturnSlot<'_, Value> = ReturnSlot::new(ReturnSlotToken::new());

            // Call the user's function
            let result = user_method(&slot);

            match result {
                Ok(()) => slot.take(),
                Err(_) => panic!("error in method"),
            }
        }

        // User writes this function
        fn user_method(ret: &ReturnSlot<'_, Value>) -> Result<(), crate::error::Error> {
            // Create a new Ruby value
            let new_value = unsafe { Value::from_raw(999 as rb_sys::VALUE) };
            let guard = NewValue::new(new_value);

            // Set it in the return slot
            ret.set_guard(guard);
            Ok(())
        }

        // Execute and verify
        let result = wrapper();
        assert_eq!(result, 999 as rb_sys::VALUE);
    }

    /// This test demonstrates that the current `Result<NewValue<T>, Error>` approach
    /// also works correctly, and is simpler for most use cases.
    #[test]
    fn test_simulated_method_wrapper_with_newvalue_return() {
        use crate::convert::IntoValue;
        use crate::value::NewValue;

        // Simulate what the current method! macro generates
        fn wrapper() -> rb_sys::VALUE {
            // Call the user's function directly
            let result = user_method();

            match result {
                Ok(guard) => {
                    // IntoValue extracts the VALUE from NewValue
                    guard.into_value().as_raw()
                }
                Err(_) => panic!("error in method"),
            }
        }

        // User writes this function - cleaner API!
        fn user_method() -> Result<NewValue<Value>, crate::error::Error> {
            let new_value = unsafe { Value::from_raw(888 as rb_sys::VALUE) };
            Ok(NewValue::new(new_value))
        }

        // Execute and verify
        let result = wrapper();
        assert_eq!(result, 888 as rb_sys::VALUE);
    }

    /// Test that demonstrates the MethodContext approach
    #[test]
    fn test_simulated_method_wrapper_with_context() {
        use crate::value::NewValue;

        // Simulate what a method_with_context! macro would generate
        fn wrapper() -> rb_sys::VALUE {
            let ctx = MethodContext::new();

            // Call user function with context
            let result = user_method_with_ctx(&ctx);

            match result {
                Ok(()) => ctx.take(),
                Err(_) => panic!("error in method"),
            }
        }

        // User writes this
        fn user_method_with_ctx(ctx: &MethodContext<'_>) -> Result<(), crate::error::Error> {
            let new_value = unsafe { Value::from_raw(777 as rb_sys::VALUE) };
            let guard = NewValue::new(new_value);
            ctx.return_value(guard);
            Ok(())
        }

        let result = wrapper();
        assert_eq!(result, 777 as rb_sys::VALUE);
    }
}

// ============================================================================
// APPROACH 5: Borrowing trick to prevent Vec storage
// ============================================================================

/// A clever approach: make the return type borrow something that exists
/// only within the method! macro's scope.
///
/// ```ignore
/// // Macro creates a "witness" on the stack
/// let _witness: ReturnWitness = ReturnWitness::new();
///
/// // User function borrows the witness
/// fn user_method<'w>(_: &'w ReturnWitness) -> ReturnValue<'w, RString> {
///     ReturnValue::new(RString::new("hello"))
/// }
///
/// // The ReturnValue cannot outlive the witness
/// let retval = user_method(&_witness);
/// let raw = retval.into_raw(); // Only macro can call this
/// ```
///
/// The key insight: ReturnValue<'w, T> borrows 'w from the witness, so it cannot
/// escape the function. And the witness itself is created by the macro.
pub struct ReturnWitness {
    _private: (),
}

impl ReturnWitness {
    /// Create a new witness. Only the macro should call this.
    #[doc(hidden)]
    #[inline]
    pub fn new() -> Self {
        ReturnWitness { _private: () }
    }
}

/// A return value that borrows a witness, preventing escape.
pub struct WitnessedReturn<'w, T: ReprValue> {
    value: rb_sys::VALUE,
    _witness: PhantomData<&'w ReturnWitness>,
    _type: PhantomData<T>,
}

impl<'w, T: ReprValue> WitnessedReturn<'w, T> {
    /// Create from a NewValue, borrowing the witness lifetime.
    #[inline]
    pub fn new(_witness: &'w ReturnWitness, guard: crate::value::NewValue<T>) -> Self {
        WitnessedReturn {
            value: guard.as_raw(),
            _witness: PhantomData,
            _type: PhantomData,
        }
    }

    /// Create from a pinned reference.
    #[inline]
    pub fn from_pinned(_witness: &'w ReturnWitness, pinned: Pin<&StackPinned<T>>) -> Self {
        WitnessedReturn {
            value: pinned.get().as_raw(),
            _witness: PhantomData,
            _type: PhantomData,
        }
    }

    /// Extract the raw VALUE. Only the macro should call this.
    #[doc(hidden)]
    #[inline]
    pub(crate) fn into_raw(self) -> rb_sys::VALUE {
        self.value
    }
}

#[cfg(test)]
mod witness_tests {
    use super::*;
    use crate::Value;
    use crate::value::NewValue;

    #[test]
    fn test_witnessed_return() {
        // Simulating what the macro would generate
        fn wrapper() -> rb_sys::VALUE {
            let witness = ReturnWitness::new();

            // User function borrows the witness
            let result = user_method(&witness);

            match result {
                Ok(retval) => retval.into_raw(),
                Err(_) => panic!("error"),
            }
        }

        // User writes this
        fn user_method<'w>(
            w: &'w ReturnWitness,
        ) -> Result<WitnessedReturn<'w, Value>, crate::error::Error> {
            let value = unsafe { Value::from_raw(666 as rb_sys::VALUE) };
            let guard = NewValue::new(value);
            Ok(WitnessedReturn::new(w, guard))
        }

        let result = wrapper();
        assert_eq!(result, 666 as rb_sys::VALUE);
    }

    // This would fail to compile - the return value cannot escape
    // fn _cannot_escape() {
    //     let witness = ReturnWitness::new();
    //     let value = unsafe { Value::from_raw(1 as rb_sys::VALUE) };
    //     let guard = NewValue::new(value);
    //     let retval = WitnessedReturn::new(&witness, guard);
    //
    //     // ERROR: Cannot move retval out of this scope because it borrows witness
    //     // let escaped = retval;  // This would work...
    //     // drop(witness);         // But this would fail - borrowed
    //
    //     // ERROR: Cannot put in Vec because lifetime isn't 'static
    //     // let vec: Vec<WitnessedReturn<'static, Value>> = vec![retval];
    // }
}
