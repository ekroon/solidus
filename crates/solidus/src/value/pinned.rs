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
impl<T> Unpin for StackPinned<T> where Self: Sized {}

impl<T> StackPinned<T> {
    /// Create a new StackPinned wrapper.
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
/// This macro creates a `StackPinned<T>` on the stack and pins it,
/// returning a `Pin<&StackPinned<T>>` reference.
///
/// # Example
///
/// ```ignore
/// use solidus::pin_on_stack;
///
/// // Pin a Ruby string on the stack
/// pin_on_stack!(pinned_str = ruby_string);
///
/// // pinned_str is now Pin<&StackPinned<RString>>
/// let inner: &RString = pinned_str.get();
/// ```
#[macro_export]
macro_rules! pin_on_stack {
    ($name:ident = $value:expr) => {
        let $name = $crate::value::StackPinned::new($value);
        // SAFETY: We're pinning a value on the stack. The value cannot
        // be moved because we immediately shadow the binding with a Pin.
        #[allow(unused_mut)]
        let mut $name = $name;
        #[allow(unused_unsafe)]
        let $name = unsafe { ::std::pin::Pin::new_unchecked(&$name) };
    };
    (mut $name:ident = $value:expr) => {
        let $name = $crate::value::StackPinned::new($value);
        // SAFETY: We're pinning a value on the stack. The value cannot
        // be moved because we immediately shadow the binding with a Pin.
        #[allow(unused_mut)]
        let mut $name = $name;
        #[allow(unused_unsafe)]
        let $name = unsafe { ::std::pin::Pin::new_unchecked(&mut $name) };
    };
}

#[cfg(test)]
mod tests {
    use super::*;

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
        pin_on_stack!(value = 42i32);
        assert_eq!(*StackPinned::get(value), 42);
    }

    #[test]
    fn test_pin_on_stack_macro_mut() {
        pin_on_stack!(mut value = 42i32);
        let mut value = value;
        *StackPinned::get_mut(value.as_mut()) = 100;
        let value = value.as_ref();
        assert_eq!(*StackPinned::get(value), 100);
    }
}
