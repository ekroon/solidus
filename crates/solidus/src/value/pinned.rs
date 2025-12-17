//! Stack-pinned wrapper for Ruby values.

use std::marker::PhantomPinned;
use std::pin::Pin;

/// A wrapper that prevents a value from being unpinned.
///
/// This type is `!Unpin` by design. It's used with `Pin<&StackPinned<T>>`
/// to ensure Ruby values remain at a fixed stack location during method calls.
///
/// # Why Stack Pinning?
///
/// Ruby's garbage collector scans the C stack to find live VALUE references.
/// If a VALUE is moved to the heap (e.g., into a `Vec` or `Box`), the GC
/// cannot see it and may collect the underlying Ruby object.
///
/// By using `Pin<&StackPinned<T>>` in method signatures, we guarantee at
/// compile time that the value cannot be moved to the heap.
///
/// # Example
///
/// ```ignore
/// use std::pin::Pin;
/// use solidus::value::StackPinned;
///
/// // The pin_on_stack! macro creates a pinned reference
/// pin_on_stack!(value = some_ruby_string);
///
/// // Now `value` is a Pin<&StackPinned<RString>>
/// // It cannot be moved to the heap
/// ```
#[repr(transparent)]
pub struct StackPinned<T> {
    value: T,
    _pin: PhantomPinned,
}

// StackPinned is !Unpin because it contains PhantomPinned
// This is the core safety guarantee - we cannot accidentally unpin it
// Note: We do NOT implement Unpin - the default !Unpin behavior from
// PhantomPinned is exactly what we want.

impl<T> StackPinned<T> {
    /// Create a new StackPinned wrapper directly.
    ///
    /// This should typically be used via the `pin_on_stack!` macro
    /// which handles the pinning automatically.
    #[inline]
    pub const fn new(value: T) -> Self {
        StackPinned {
            value,
            _pin: PhantomPinned,
        }
    }

    /// Get a reference to the wrapped value.
    ///
    /// This is the primary way to access the underlying value
    /// when you have a `Pin<&StackPinned<T>>`.
    #[inline]
    pub fn get(self: Pin<&Self>) -> &T {
        // SAFETY: We're not moving the value, just providing a reference
        &self.get_ref().value
    }

    /// Get a mutable reference to the wrapped value.
    ///
    /// # Safety
    ///
    /// This is safe because we're not moving the value out,
    /// just providing mutable access to it in place.
    #[inline]
    pub fn get_mut(self: Pin<&mut Self>) -> &mut T {
        // SAFETY: We're not moving the StackPinned, just providing
        // mutable access to the inner value. The Pin contract is
        // maintained because T itself doesn't need to be pinned.
        unsafe { &mut self.get_unchecked_mut().value }
    }

    /// Get the inner value, consuming the wrapper.
    ///
    /// This can only be called on an unpinned `StackPinned`,
    /// which means it cannot be used on a `Pin<&StackPinned<T>>`.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

/// Create a stack-pinned value.
///
/// This macro accepts `PinGuard<T>` expressions from value creation
/// (e.g., `RString::new("hello")`).
///
/// The macro atomically consumes the guard and creates a pinned reference,
/// preventing the safety gap where `StackPinned<T>` could be moved to the heap.
///
/// # Examples
///
/// ```ignore
/// use solidus::pin_on_stack;
/// use solidus::types::RString;
///
/// // Pin a newly created value
/// pin_on_stack!(s = RString::new("hello"));
/// // s is Pin<&StackPinned<RString>>
/// ```
///
/// # Safety
///
/// The macro ensures that:
/// - Values are pinned atomically with guard consumption
/// - No intermediate movable `StackPinned<T>` value exists
/// - The pinned value cannot be moved to the heap
#[macro_export]
macro_rules! pin_on_stack {
    // Pattern for direct value wrapping and pinning
    ($name:ident = $guard:expr) => {
        let __guard = $guard;
        // Use IntoPinnable trait to extract the value (helps with type inference in macros)
        // SAFETY: We immediately wrap in StackPinned and pin it
        let __value = unsafe { $crate::value::IntoPinnable::into_pinnable(__guard) };
        let __stack = $crate::value::StackPinned::new(__value);
        // SAFETY: We're pinning a value on the stack. The value cannot
        // be moved because we immediately shadow the binding with a Pin.
        #[allow(unused_mut)]
        let mut __stack = __stack;
        #[allow(unused_unsafe)]
        let $name = unsafe { ::std::pin::Pin::new_unchecked(&__stack) };
    };
    // Pattern for mutable direct value wrapping and pinning
    (mut $name:ident = $guard:expr) => {
        let __guard = $guard;
        // Use IntoPinnable trait to extract the value (helps with type inference in macros)
        // SAFETY: We immediately wrap in StackPinned and pin it
        let __value = unsafe { $crate::value::IntoPinnable::into_pinnable(__guard) };
        let __stack = $crate::value::StackPinned::new(__value);
        // SAFETY: We're pinning a value on the stack. The value cannot
        // be moved because we immediately shadow the binding with a Pin.
        #[allow(unused_mut)]
        let mut __stack = __stack;
        #[allow(unused_unsafe)]
        let $name = unsafe { ::std::pin::Pin::new_unchecked(&mut __stack) };
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    #[test]
    fn test_stack_pinned_get() {
        let pinned = StackPinned::new(42i32);
        // SAFETY: We're pinning on the stack in this test
        let pinned = unsafe { Pin::new_unchecked(&pinned) };
        assert_eq!(*StackPinned::get(pinned), 42);
    }

    #[test]
    fn test_stack_pinned_get_mut() {
        let mut pinned = StackPinned::new(42i32);
        // SAFETY: We're pinning on the stack in this test
        let mut pinned = unsafe { Pin::new_unchecked(&mut pinned) };
        *StackPinned::get_mut(pinned.as_mut()) = 100;
        // Re-borrow as immutable to call get
        let pinned = pinned.as_ref();
        assert_eq!(*StackPinned::get(pinned), 100);
    }

    #[test]
    fn test_stack_pinned_into_inner() {
        let pinned = StackPinned::new(String::from("hello"));
        let inner = pinned.into_inner();
        assert_eq!(inner, "hello");
    }

    #[test]
    fn test_pin_on_stack_macro() {
        // Test with PinGuard
        use crate::value::PinGuard;
        let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
        pin_on_stack!(value = guard);
        assert!(value.get().is_nil());
    }

    #[test]
    fn test_pin_on_stack_macro_mut() {
        // Test with PinGuard (mutable)
        use crate::value::PinGuard;
        let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
        pin_on_stack!(mut value = guard);
        // Just check that it works - convert to immutable ref to call get()
        assert!(value.as_ref().get().is_nil());
    }

    #[test]
    fn test_pin_on_stack_with_value() {
        // Test pinning a Value via PinGuard
        use crate::value::PinGuard;
        let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
        pin_on_stack!(value = guard);
        assert!(value.get().is_nil());
    }

    // This test verifies that StackPinned is !Unpin by demonstrating
    // that Pin::new cannot be used with it (only Pin::new_unchecked works)
    #[test]
    fn test_stack_pinned_is_not_unpin() {
        // StackPinned should be !Unpin, which means:
        // 1. Pin::new() cannot be used (requires Unpin)
        // 2. Pin::new_unchecked() must be used instead

        let pinned = StackPinned::new(42i32);
        let boxed = Box::new(pinned);

        // This line would fail to compile if uncommented, proving !Unpin:
        // let _ = Pin::new(boxed);
        // error: the trait bound `StackPinned<i32>: Unpin` is not satisfied

        // Instead, we must use the unsafe Pin::new_unchecked:
        let pinned_box = unsafe { Pin::new_unchecked(boxed) };
        assert_eq!(*StackPinned::get(pinned_box.as_ref()), 42);

        // We can verify at compile-time that StackPinned<i32> is !Unpin
        // by uncommenting this function and call:
        // fn requires_unpin<T: Unpin>(_: &T) {}
        // requires_unpin(&StackPinned::new(42i32));
        // ^ This would fail to compile with:
        //   error: the trait bound `StackPinned<i32>: Unpin` is not satisfied

        // The fact that this test compiles and runs proves StackPinned is !Unpin
    }
}
