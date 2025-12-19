//! Context for creating Ruby values within method calls.
//!
//! The `Context` type provides stack-allocated storage for Ruby VALUES,
//! ensuring they are visible to Ruby's GC during method execution.

use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::pin::Pin;

use crate::error::AllocationError;
use crate::types::{RArray, RHash, RString};
use crate::value::{BoxValue, ReprValue, StackPinned, Value};

/// Context for creating Ruby values within a method call.
///
/// `Context` provides stack-allocated storage for Ruby VALUES, ensuring they
/// are visible to Ruby's GC. Values created via `ctx.new_xxx()` are pinned
/// in the Context's storage and cannot escape to the heap unsafely.
///
/// # Capacity
///
/// By default, Context provides 8 VALUE slots. If you need more, the capacity
/// can be customized via const generics. If slots are exhausted, `new_xxx()`
/// methods return `Err(AllocationError)`.
///
/// # Interior Mutability
///
/// Context uses interior mutability (`&self` methods) to allow creating
/// multiple values and using them together:
///
/// ```ignore
/// fn example<'ctx>(ctx: &'ctx Context) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
///     let s1 = ctx.new_string("hello")?;
///     let s2 = ctx.new_string("world")?;
///     // Can use s1 and s2 together
///     Ok(s1)
/// }
/// ```
///
/// # Lifetime Safety
///
/// Values borrowed from Context have lifetime `'ctx` tied to the Context itself.
/// This prevents values from outliving the method call's stack frame.
pub struct Context<'a, const N: usize = 8> {
    /// Storage for VALUES. Each slot holds a raw rb_sys::VALUE.
    slots: UnsafeCell<[MaybeUninit<rb_sys::VALUE>; N]>,

    /// Number of slots currently in use.
    used: Cell<usize>,

    /// Marker for lifetime and !Send/!Sync.
    _marker: PhantomData<&'a mut ()>,
}

impl<'a, const N: usize> Context<'a, N> {
    /// Create a new Context.
    ///
    /// This is `#[doc(hidden)]` because only the `method!`/`function!` macros
    /// should create Contexts.
    #[doc(hidden)]
    #[inline]
    pub fn new() -> Self {
        Context {
            // SAFETY: MaybeUninit doesn't require initialization
            slots: UnsafeCell::new(unsafe { MaybeUninit::uninit().assume_init() }),
            used: Cell::new(0),
            _marker: PhantomData,
        }
    }

    /// Returns the number of available slots.
    #[inline]
    pub fn available(&self) -> usize {
        N - self.used.get()
    }

    /// Returns the total capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    /// Allocate a slot and store a VALUE, returning a pinned reference.
    ///
    /// # Safety
    ///
    /// The caller must ensure `value` is a valid Ruby VALUE that will remain
    /// valid for the lifetime of the Context.
    #[inline]
    unsafe fn alloc_slot<T: ReprValue>(
        &'a self,
        value: T,
    ) -> Result<Pin<&'a StackPinned<T>>, AllocationError> {
        let idx = self.used.get();
        if idx >= N {
            return Err(AllocationError);
        }

        // Store the raw VALUE
        // SAFETY: We have exclusive access via interior mutability and idx < N
        let slots = unsafe { &mut *self.slots.get() };
        slots[idx].write(value.as_raw());
        self.used.set(idx + 1);

        // Create a pinned reference to the stored value
        // SAFETY:
        // - The slot is valid for lifetime 'a (the Context's lifetime)
        // - StackPinned<T> is #[repr(transparent)] over T
        // - T (e.g., RString) is #[repr(transparent)] over Value
        // - Value is #[repr(transparent)] over rb_sys::VALUE
        // - So the memory layout is compatible
        let slot_ptr = slots[idx].as_ptr() as *const StackPinned<T>;
        Ok(unsafe { Pin::new_unchecked(&*slot_ptr) })
    }

    // ========================================================================
    // String creation
    // ========================================================================

    /// Create a new Ruby string, stored in Context's stack slots.
    ///
    /// Returns `Err(AllocationError)` if all slots are exhausted.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn greet<'ctx>(ctx: &'ctx Context, name: &str) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    ///     ctx.new_string(&format!("Hello, {}!", name)).map_err(Into::into)
    /// }
    /// ```
    pub fn new_string(&'a self, s: &str) -> Result<Pin<&'a StackPinned<RString>>, AllocationError> {
        // SAFETY: rb_str_new creates a valid VALUE
        let value = unsafe {
            let val = rb_sys::rb_str_new(s.as_ptr() as *const std::os::raw::c_char, s.len() as _);
            RString::from_value_unchecked(Value::from_raw(val))
        };
        // SAFETY: value is a valid Ruby string VALUE
        unsafe { self.alloc_slot(value) }
    }

    /// Create a new Ruby string from bytes, stored in Context's stack slots.
    ///
    /// The string will have binary encoding.
    pub fn new_string_from_slice(
        &'a self,
        bytes: &[u8],
    ) -> Result<Pin<&'a StackPinned<RString>>, AllocationError> {
        let value = unsafe {
            let val = rb_sys::rb_str_new(
                bytes.as_ptr() as *const std::os::raw::c_char,
                bytes.len() as _,
            );
            RString::from_value_unchecked(Value::from_raw(val))
        };
        unsafe { self.alloc_slot(value) }
    }

    /// Create a new Ruby string, boxed for heap storage.
    ///
    /// This always succeeds (uses heap allocation, not Context slots).
    /// Use this when you need to store the string in a collection.
    pub fn new_string_boxed(&self, s: &str) -> BoxValue<RString> {
        RString::new_boxed(s)
    }

    // ========================================================================
    // Array creation
    // ========================================================================

    /// Create a new empty Ruby array, stored in Context's stack slots.
    ///
    /// Returns `Err(AllocationError)` if all slots are exhausted.
    pub fn new_array(&'a self) -> Result<Pin<&'a StackPinned<RArray>>, AllocationError> {
        let value = unsafe {
            let val = rb_sys::rb_ary_new();
            RArray::from_value_unchecked(Value::from_raw(val))
        };
        unsafe { self.alloc_slot(value) }
    }

    /// Create a new Ruby array with pre-allocated capacity.
    pub fn new_array_with_capacity(
        &'a self,
        capacity: usize,
    ) -> Result<Pin<&'a StackPinned<RArray>>, AllocationError> {
        let value = unsafe {
            let val = rb_sys::rb_ary_new_capa(capacity as _);
            RArray::from_value_unchecked(Value::from_raw(val))
        };
        unsafe { self.alloc_slot(value) }
    }

    /// Create a new Ruby array, boxed for heap storage.
    ///
    /// This always succeeds (uses heap allocation, not Context slots).
    pub fn new_array_boxed(&self) -> BoxValue<RArray> {
        RArray::new_boxed()
    }

    // ========================================================================
    // Hash creation
    // ========================================================================

    /// Create a new empty Ruby hash, stored in Context's stack slots.
    ///
    /// Returns `Err(AllocationError)` if all slots are exhausted.
    pub fn new_hash(&'a self) -> Result<Pin<&'a StackPinned<RHash>>, AllocationError> {
        let value = unsafe {
            let val = rb_sys::rb_hash_new();
            RHash::from_value_unchecked(Value::from_raw(val))
        };
        unsafe { self.alloc_slot(value) }
    }

    /// Create a new Ruby hash, boxed for heap storage.
    ///
    /// This always succeeds (uses heap allocation, not Context slots).
    pub fn new_hash_boxed(&self) -> BoxValue<RHash> {
        RHash::new_boxed()
    }

    // ========================================================================
    // Generic value pinning
    // ========================================================================

    /// Store an existing value in Context's stack slots.
    ///
    /// This is useful when you have a VALUE that you want to return
    /// through the Context's lifetime system.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn process<'ctx>(ctx: &'ctx Context, input: RString) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    ///     // Do some processing that doesn't create a new value
    ///     ctx.pin_value(input).map_err(Into::into)
    /// }
    /// ```
    pub fn pin_value<T: ReprValue>(
        &'a self,
        value: T,
    ) -> Result<Pin<&'a StackPinned<T>>, AllocationError> {
        // SAFETY: Caller provides a valid Ruby value
        unsafe { self.alloc_slot(value) }
    }
}

impl<const N: usize> Default for Context<'_, N> {
    fn default() -> Self {
        Self::new()
    }
}

// Context is !Send and !Sync because:
// 1. It contains raw pointers to Ruby VALUEs
// 2. Ruby's GC is not thread-safe
// 3. It's tied to a specific stack frame
// The PhantomData<&'a mut ()> handles this automatically.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_capacity() {
        let ctx: Context<'_, 8> = Context::new();
        assert_eq!(ctx.capacity(), 8);
        assert_eq!(ctx.available(), 8);
    }

    #[test]
    fn test_context_custom_capacity() {
        let ctx: Context<'_, 16> = Context::new();
        assert_eq!(ctx.capacity(), 16);
        assert_eq!(ctx.available(), 16);
    }

    // Tests that call Ruby require the embed or link-ruby feature
    #[cfg(any(feature = "embed", feature = "link-ruby"))]
    mod ruby_tests {
        use super::*;
        use rb_sys_test_helpers::ruby_test;

        #[ruby_test]
        fn test_new_string() {
            let ctx: Context<'_> = Context::new();
            let s = ctx.new_string("hello").unwrap();
            assert_eq!(s.get().len(), 5);
            assert_eq!(ctx.available(), 7);
        }

        #[ruby_test]
        fn test_new_string_from_slice() {
            let ctx: Context<'_> = Context::new();
            let s = ctx.new_string_from_slice(b"hello\x00world").unwrap();
            assert_eq!(s.get().len(), 11);
        }

        #[ruby_test]
        fn test_new_array() {
            let ctx: Context<'_> = Context::new();
            let arr = ctx.new_array().unwrap();
            assert_eq!(arr.get().len(), 0);
            arr.get().push(42i64);
            assert_eq!(arr.get().len(), 1);
        }

        #[ruby_test]
        fn test_new_hash() {
            let ctx: Context<'_> = Context::new();
            let hash = ctx.new_hash().unwrap();
            assert!(hash.get().is_empty());
            hash.get().insert("key", 42i64);
            assert_eq!(hash.get().len(), 1);
        }

        #[ruby_test]
        fn test_multiple_values() {
            let ctx: Context<'_> = Context::new();
            let s1 = ctx.new_string("hello").unwrap();
            let s2 = ctx.new_string("world").unwrap();
            assert_eq!(s1.get().len(), 5);
            assert_eq!(s2.get().len(), 5);
            assert_eq!(ctx.available(), 6);
        }

        #[ruby_test]
        fn test_exhaustion() {
            let ctx: Context<'_, 2> = Context::new();
            let _s1 = ctx.new_string("a").unwrap();
            let _s2 = ctx.new_string("b").unwrap();
            let result = ctx.new_string("c");
            assert!(result.is_err());
        }

        #[ruby_test]
        fn test_boxed_always_succeeds() {
            let ctx: Context<'_, 0> = Context::new();
            // Even with 0 slots, boxed methods work
            let boxed = ctx.new_string_boxed("hello");
            assert_eq!(boxed.len(), 5);
        }
    }
}
