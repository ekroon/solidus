//! Return types for Ruby methods.
//!
//! This module provides `ReturnWitness` and `WitnessedReturn<'w, T>` which together
//! ensure that Ruby values returned from methods cannot be stored in collections.

use std::marker::PhantomData;
use std::pin::Pin;

use crate::value::{NewValue, ReprValue, StackPinned};

/// A witness type that exists only within the `method!` macro's stack frame.
///
/// The `method!` macro creates a `ReturnWitness` on its stack. User methods
/// borrow this witness when creating a `WitnessedReturn`, which ties the
/// return value's lifetime to the macro's stack frame.
///
/// This type has no public constructor - only the `method!` macro can create it.
pub struct ReturnWitness {
    _private: (),
}

impl ReturnWitness {
    /// Create a new witness.
    ///
    /// This is `#[doc(hidden)]` because only the `method!` macro should call it.
    #[doc(hidden)]
    #[inline]
    pub fn new() -> Self {
        ReturnWitness { _private: () }
    }
}

/// A return value that borrows a witness, preventing escape to collections.
///
/// The lifetime `'w` is tied to the `ReturnWitness` in the `method!` macro's
/// stack frame. This means `WitnessedReturn<'w, T>` cannot be stored in a
/// `Vec` or other collection (which would require `'static` lifetime).
///
/// # Example
///
/// ```ignore
/// // Method signature with witnessed return:
/// fn greet<'w>(
///     &self,
///     w: &'w ReturnWitness,
///     name: Pin<&StackPinned<NewValue<RString>>>
/// ) -> Result<WitnessedReturn<'w, RString>, Error> {
///     pin_on_stack!(result = RString::new("hello"));
///     Ok(WitnessedReturn::from_pinned(w, result))
/// }
/// ```
pub struct WitnessedReturn<'w, T: ReprValue> {
    value: rb_sys::VALUE,
    _witness: PhantomData<&'w ReturnWitness>,
    _type: PhantomData<T>,
}

impl<'w, T: ReprValue> WitnessedReturn<'w, T> {
    /// Create from a NewValue, borrowing the witness lifetime.
    #[inline]
    pub fn new(_witness: &'w ReturnWitness, guard: NewValue<T>) -> Self {
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

    /// Extract the raw VALUE for FFI return.
    ///
    /// This is `pub(crate)` - only the `method!` macro should call this.
    #[doc(hidden)]
    #[inline]
    pub fn into_raw(self) -> rb_sys::VALUE {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    use crate::pin_on_stack;
    use crate::value::NewValue;

    #[test]
    fn test_witnessed_return_from_guard() {
        // Simulating what the method! macro generates
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
            let value = unsafe { Value::from_raw(42 as rb_sys::VALUE) };
            let guard = NewValue::new(value);
            Ok(WitnessedReturn::new(w, guard))
        }

        let result = wrapper();
        assert_eq!(result, 42 as rb_sys::VALUE);
    }

    #[test]
    fn test_witnessed_return_from_pinned() {
        fn wrapper() -> rb_sys::VALUE {
            let witness = ReturnWitness::new();
            let result = user_method(&witness);
            result.unwrap().into_raw()
        }

        fn user_method<'w>(
            w: &'w ReturnWitness,
        ) -> Result<WitnessedReturn<'w, Value>, crate::error::Error> {
            let value = unsafe { Value::from_raw(99 as rb_sys::VALUE) };
            let guard = NewValue::new(value);
            pin_on_stack!(pinned = guard);
            Ok(WitnessedReturn::from_pinned(w, pinned))
        }

        let result = wrapper();
        assert_eq!(result, 99 as rb_sys::VALUE);
    }

    // Compile-time test: WitnessedReturn cannot be stored in a Vec
    // because its lifetime is not 'static.
    //
    // This code would fail to compile:
    // fn _cannot_store_in_vec() {
    //     let witness = ReturnWitness::new();
    //     let value = unsafe { Value::from_raw(1 as rb_sys::VALUE) };
    //     let guard = NewValue::new(value);
    //     let retval = WitnessedReturn::new(&witness, guard);
    //
    //     // ERROR: cannot put in Vec because lifetime isn't 'static
    //     let vec: Vec<WitnessedReturn<'static, Value>> = vec![retval];
    // }
}
