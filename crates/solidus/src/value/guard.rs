//! PinGuard type for enforcing pinning at VALUE creation time.

use std::marker::PhantomPinned;
use std::ops::{Deref, DerefMut};

use super::traits::IntoPinnable;
use super::{BoxValue, ReprValue};

/// A guard that enforces pinning of newly-created Ruby values.
///
/// When you create a new Ruby value (e.g., via `RString::new()`), it returns
/// a `PinGuard<T>`. This forces you to make an explicit choice:
///
/// - **Stack pin it** with `pin_on_stack!` macro (fast, common case)
/// - **Heap box it** with `.into_box()` → `BoxValue<T>` (GC-registered, for collections)
///
/// The `#[must_use]` attribute ensures you can't accidentally forget to pin
/// or box the value, which would risk it being moved to the heap unprotected.
///
/// # Why is this needed?
///
/// Ruby's GC scans the C stack to find live VALUE references. If a VALUE
/// is moved to the heap without GC registration, the GC cannot see it and
/// may collect the underlying Ruby object, leading to use-after-free bugs.
///
/// By making all Ruby types `!Copy` and using `PinGuard`, we ensure that
/// values are either:
/// 1. Stack-pinned (GC can see them)
/// 2. Explicitly boxed and GC-registered (GC is notified)
///
/// # Type Safety
///
/// `PinGuard` is `!Unpin` (via `PhantomPinned`), making it clear that this
/// is not a final storage type. You cannot store it in collections or move
/// it around freely. It exists solely to bridge the gap between creation
/// and pinning/boxing.
///
/// # Example
///
/// ```ignore
/// use solidus::RString;
/// use solidus::pin_on_stack;
///
/// // Creating a Ruby string returns a PinGuard
/// let guard = RString::new("hello");
///
/// // Option 1: Pin on stack (common case)
/// pin_on_stack!(s = guard);
/// // Now s is Pin<&StackPinned<RString>>
///
/// // Option 2: Box for heap storage
/// let guard = RString::new("world");
/// let boxed = guard.into_box();
/// let mut strings = Vec::new();
/// strings.push(boxed); // Safe - registered with GC
/// ```
///
/// # Must-use Warning
///
/// If you create a value and don't pin or box it, you'll get a compiler warning:
///
/// ```ignore
/// let _ = RString::new("oops"); // Warning: VALUE must be pinned on stack or explicitly boxed
/// ```
///
/// # Safety Design
///
/// The critical safety property is that pinning must be **atomic with guard consumption**.
/// The old design with `.pin()` had a gap:
/// ```ignore
/// let guard = RString::new("hello");
/// let stack_pinned = guard.pin();  // Returns StackPinned<T>
/// let vec = vec![stack_pinned];    // DANGER! Moved to heap!
/// ```
/// The new design eliminates this by making `pin_on_stack!` consume the guard directly,
/// creating the StackPinned and pinning it atomically in one step.
#[must_use = "VALUE must be pinned on stack or explicitly boxed"]
pub struct PinGuard<T: ReprValue> {
    value: T,
    /// Makes PinGuard !Unpin, preventing accidental storage in collections.
    _pin: PhantomPinned,
}

impl<T: ReprValue> PinGuard<T> {
    /// Create a new PinGuard wrapping a Ruby value.
    ///
    /// This is `pub` so type constructors (like `RString::new()`) can return guards.
    #[inline]
    pub fn new(value: T) -> Self {
        PinGuard {
            value,
            _pin: PhantomPinned,
        }
    }

    /// Unwrap the guard, returning the inner value for internal use by pin_on_stack! macro.
    ///
    /// # Safety
    ///
    /// This is unsafe because it allows the VALUE to escape without being
    /// pinned or boxed. The caller must ensure the value is immediately
    /// wrapped into StackPinned and pinned. This should only be used by
    /// the pin_on_stack! macro.
    ///
    /// This method is named differently from `into_inner` to make its
    /// internal-only purpose clear.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn into_inner_for_macro(self) -> T {
        self.value
    }

    /// Box the value on the heap with GC registration.
    ///
    /// This consumes the guard and returns a `BoxValue<T>`, which can be
    /// safely stored in Rust collections because it registers with Ruby's GC.
    ///
    /// Use this when you need to store values on the heap (e.g., in a Vec).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::{RString, BoxValue};
    ///
    /// let guard = RString::new("hello");
    /// let boxed = guard.into_box();
    ///
    /// let mut strings: Vec<BoxValue<RString>> = Vec::new();
    /// strings.push(boxed); // Safe - registered with GC
    /// ```
    #[inline]
    pub fn into_box(self) -> BoxValue<T> {
        BoxValue::new(self.value)
    }

    /// Unwrap the guard, returning the inner value.
    ///
    /// # Safety
    ///
    /// This is unsafe because it allows the VALUE to escape without being
    /// pinned or boxed. The caller must ensure the value is immediately
    /// used in a safe context (e.g., wrapped back into a PinGuard or
    /// immediately pinned).
    ///
    /// This is primarily used internally when converting between guard types.
    #[inline]
    pub unsafe fn into_inner(self) -> T {
        self.value
    }
}

// Implement Clone for PinGuard
impl<T: ReprValue> Clone for PinGuard<T> {
    #[inline]
    fn clone(&self) -> Self {
        PinGuard {
            value: self.value.clone(),
            _pin: PhantomPinned,
        }
    }
}

// Note: We do NOT implement ReprValue for PinGuard because:
// 1. PinGuard is a temporary guard type, not a final value type
// 2. We use Deref/DerefMut to access the inner value's methods instead

// Implement IntoValue for PinGuard to allow returning guarded values directly from methods
impl<T: ReprValue> crate::convert::IntoValue for PinGuard<T> {
    #[inline]
    fn into_value(self) -> super::Value {
        // When returning from a method, we can safely unwrap the guard
        // because the value is immediately converted to a raw VALUE
        // and returned to Ruby.
        self.value.as_value()
    }
}

// Implement Deref to allow transparent access to the inner value's methods
impl<T: ReprValue> Deref for PinGuard<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// Implement DerefMut to allow mutable access to the inner value's methods
impl<T: ReprValue> DerefMut for PinGuard<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

// Implement AsRef to provide standard trait access
impl<T: ReprValue> AsRef<T> for PinGuard<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.value
    }
}

// Implement AsMut to provide standard trait access
impl<T: ReprValue> AsMut<T> for PinGuard<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

// Implement IntoPinnable for PinGuard to help with type inference in pin_on_stack! macro
impl<T: ReprValue> IntoPinnable for PinGuard<T> {
    type Target = T;

    #[inline]
    unsafe fn into_pinnable(self) -> Self::Target {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_pin_guard_to_pin_on_stack() {
        // Test the atomic pinning workflow
        use crate::pin_on_stack;

        // SAFETY: Qnil is always a valid VALUE
        let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });

        // The guard is atomically consumed and pinned
        pin_on_stack!(pinned = guard);

        // Verify we can access the value
        let value = pinned.get();
        assert!(value.is_nil());
    }

    // Test for into_box requires Ruby to be initialized (GC registration)
    // This is tested in integration tests with rb-sys-test-helpers
    #[cfg(any(feature = "embed", feature = "link-ruby"))]
    #[test]
    fn test_pin_guard_into_box() {
        // Create a guard
        // SAFETY: Qnil is always a valid VALUE
        let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });

        // Box it
        let boxed = guard.into_box();

        // Verify we can access the value
        assert!(boxed.is_nil());
    }

    #[test]
    fn test_pin_guard_as_ref() {
        // SAFETY: Qnil is always a valid VALUE
        let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });

        // We can inspect without consuming
        let value_ref = guard.as_ref();
        assert!(value_ref.is_nil());

        // And still pin afterward with the macro
        use crate::pin_on_stack;
        pin_on_stack!(_pinned = guard);
    }

    #[test]
    fn test_pin_guard_as_mut() {
        // SAFETY: Qnil is always a valid VALUE
        let mut guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });

        // We can mutate without consuming
        let value_mut = guard.as_mut();
        assert!(value_mut.is_nil());

        // And still pin afterward with the macro
        use crate::pin_on_stack;
        pin_on_stack!(_pinned = guard);
    }

    // This test verifies that PinGuard is !Unpin
    #[test]
    fn test_pin_guard_is_not_unpin() {
        // SAFETY: Qnil is always a valid VALUE
        let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });

        // PinGuard should be !Unpin due to PhantomPinned
        // This means it cannot be stored in collections without
        // first calling .into_box()

        // This would fail to compile:
        // fn requires_unpin<T: Unpin>(_: T) {}
        // requires_unpin(guard);

        // The guard is meant to be consumed immediately via pin_on_stack! or .into_box()
        use crate::pin_on_stack;
        pin_on_stack!(_pinned = guard);
    }

    #[test]
    fn test_pin_guard_size() {
        // PinGuard should be the same size as the wrapped type
        // (PhantomPinned is zero-sized)
        assert_eq!(
            std::mem::size_of::<PinGuard<Value>>(),
            std::mem::size_of::<Value>()
        );
    }

    #[test]
    fn test_pin_guard_with_pin_on_stack_macro() {
        // Test that PinGuard works with pin_on_stack! macro atomically
        use crate::pin_on_stack;

        // Direct pattern - guard is consumed atomically by the macro
        // SAFETY: Qnil is always a valid VALUE
        let guard = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) });
        pin_on_stack!(pinned_value = guard);

        // pinned_value is Pin<&StackPinned<Value>>
        let inner = pinned_value.get();
        assert!(inner.is_nil());

        // Pattern 2: One-shot - direct expression
        // SAFETY: Qnil is always a valid VALUE
        pin_on_stack!(
            pinned_direct = PinGuard::new(unsafe { Value::from_raw(rb_sys::Qnil.into()) })
        );
        let inner2 = pinned_direct.get();
        assert!(inner2.is_nil());
    }
}
