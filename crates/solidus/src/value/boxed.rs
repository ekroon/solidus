//! Heap-allocated, GC-registered Ruby value wrapper.

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use super::traits::ReprValue;
use crate::gc;

/// A heap-allocated Ruby value that is protected from garbage collection.
///
/// Use this when you need to store Ruby values in Rust collections or
/// keep them alive across async boundaries. Unlike stack-pinned values,
/// `BoxValue<T>` can be safely stored on the heap because it registers
/// itself with Ruby's GC.
///
/// # How it works
///
/// When created, `BoxValue` allocates the value on the heap and calls
/// `rb_gc_register_address` to tell Ruby's GC about it. When dropped,
/// it calls `rb_gc_unregister_address` to clean up.
///
/// # Performance
///
/// Creating a `BoxValue` involves a heap allocation and a GC registration,
/// so prefer stack-pinned values (`Pin<&StackPinned<T>>`) when possible.
/// Use `BoxValue` only when you genuinely need heap storage.
///
/// # Example
///
/// ```no_run
/// use solidus::BoxValue;
/// use solidus::types::RString;
/// use solidus::pin_on_stack;
///
/// // Store Ruby values in a Vec
/// let mut strings: Vec<BoxValue<RString>> = Vec::new();
/// pin_on_stack!(ruby_string = RString::new("hello"));
/// strings.push(BoxValue::new(ruby_string.get().clone()));
///
/// // The Ruby string is protected from GC as long as the BoxValue exists
/// ```
pub struct BoxValue<T: ReprValue> {
    /// Pointer to the heap-allocated value.
    /// The VALUE is stored at this location and registered with the GC.
    ptr: NonNull<T>,
}

impl<T: ReprValue> BoxValue<T> {
    /// Create a new BoxValue, registering with Ruby's GC.
    ///
    /// This allocates the value on the heap and registers its location
    /// with Ruby's garbage collector.
    pub fn new(value: T) -> Self {
        // Allocate on the heap
        let boxed = Box::new(value);
        let ptr = NonNull::from(Box::leak(boxed));

        // Register with Ruby's GC
        // SAFETY: The pointer is valid and will remain valid until drop
        unsafe {
            gc::register_address(ptr.as_ptr() as *mut rb_sys::VALUE);
        }

        BoxValue { ptr }
    }

    /// Get a clone of the inner value.
    ///
    /// Note: This returns a clone of the value.
    /// The BoxValue continues to protect the value from GC.
    #[inline]
    pub fn get(&self) -> T {
        // SAFETY: ptr is always valid
        unsafe { self.ptr.as_ref().clone() }
    }

    /// Consume the BoxValue and return the inner value.
    ///
    /// # Warning
    ///
    /// After calling this, the returned value is no longer protected from GC.
    /// You must ensure it stays on the stack or re-register it somehow.
    pub fn into_inner(self) -> T {
        let value = self.get();

        // Unregister and deallocate
        // SAFETY: The pointer was registered in new()
        unsafe {
            gc::unregister_address(self.ptr.as_ptr() as *mut rb_sys::VALUE);
            drop(Box::from_raw(self.ptr.as_ptr()));
        }

        // Don't run Drop
        std::mem::forget(self);

        value
    }
}

impl<T: ReprValue> Deref for BoxValue<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: ptr is always valid
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ReprValue> DerefMut for BoxValue<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: ptr is always valid and we have unique access
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: ReprValue> Drop for BoxValue<T> {
    fn drop(&mut self) {
        // SAFETY: The pointer was registered in new()
        unsafe {
            gc::unregister_address(self.ptr.as_ptr() as *mut rb_sys::VALUE);
            drop(Box::from_raw(self.ptr.as_ptr()));
        }
    }
}

impl<T: ReprValue + fmt::Debug> fmt::Debug for BoxValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BoxValue").field(&**self).finish()
    }
}

impl<T: ReprValue> Clone for BoxValue<T> {
    fn clone(&self) -> Self {
        BoxValue::new(self.get())
    }
}

// BoxValue is Send + Sync if T is, since we own the allocation
// and the GC registration is thread-safe
unsafe impl<T: ReprValue + Send> Send for BoxValue<T> {}
unsafe impl<T: ReprValue + Sync> Sync for BoxValue<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    // BoxValue tests require Ruby to be initialized for GC registration.
    // These basic structural tests work without Ruby.

    #[test]
    fn test_box_value_size() {
        // BoxValue should be the size of a pointer
        assert_eq!(
            std::mem::size_of::<BoxValue<Value>>(),
            std::mem::size_of::<*mut Value>()
        );
    }
}
