//! Marker and Compactor for GC operations

use crate::gc;
use crate::value::{BoxValue, ReprValue};

/// Helper for marking Ruby values during garbage collection.
///
/// This struct is passed to `DataTypeFunctions::mark` and provides methods
/// for marking Ruby values that your wrapped type contains.
///
/// # Example
///
/// ```ignore
/// use solidus::typed_data::{DataTypeFunctions, Marker};
/// use solidus::BoxValue;
/// use solidus::Value;
///
/// struct Container {
///     items: Vec<BoxValue<Value>>,
/// }
///
/// impl DataTypeFunctions for Container {
///     fn mark(&self, marker: &Marker) {
///         for item in &self.items {
///             marker.mark(item);
///         }
///     }
/// }
/// ```
pub struct Marker {
    // Zero-sized type - just a namespace for the mark method
    _private: (),
}

impl Marker {
    /// Create a new Marker.
    ///
    /// This is called internally by the GC callbacks.
    #[inline]
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }

    /// Mark a Ruby value as reachable.
    ///
    /// Call this for any Ruby values your type contains to prevent them
    /// from being garbage collected.
    #[inline]
    pub fn mark<T: ReprValue>(&self, value: &T) {
        gc::mark(value.as_value());
    }

    /// Mark a BoxValue as reachable.
    ///
    /// Convenience method for marking BoxValue instances.
    #[inline]
    pub fn mark_boxed<T: ReprValue>(&self, value: &BoxValue<T>) {
        gc::mark(value.as_value());
    }
}

/// Helper for updating Ruby value references after GC compaction.
///
/// This struct is passed to `DataTypeFunctions::compact` and provides methods
/// for getting the new location of Ruby values that may have moved during
/// garbage collection compaction.
///
/// # Example
///
/// ```ignore
/// use solidus::typed_data::{DataTypeFunctions, Compactor};
/// use solidus::Value;
///
/// struct Container {
///     // Raw VALUE stored (unusual, but possible)
///     cached_value: rb_sys::VALUE,
/// }
///
/// impl DataTypeFunctions for Container {
///     fn compact(&mut self, compactor: &Compactor) {
///         // Update the cached value to its new location
///         self.cached_value = compactor.location_raw(self.cached_value);
///     }
/// }
/// ```
///
/// # Note
///
/// Most types that use `BoxValue<T>` don't need to implement `compact` because
/// `BoxValue` stores values by address registration, not by raw VALUE.
pub struct Compactor {
    _private: (),
}

impl Compactor {
    /// Create a new Compactor.
    ///
    /// This is called internally by the GC callbacks.
    #[inline]
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }

    /// Get the new location of a Ruby value after compaction.
    ///
    /// If the value was moved during GC compaction, this returns the new location.
    /// If it wasn't moved, it returns the original value.
    #[inline]
    pub fn location<T: ReprValue>(&self, value: &T) -> crate::value::Value {
        let raw = value.as_value().as_raw();
        // SAFETY: rb_gc_location is safe to call during compaction phase,
        // and the returned VALUE is valid
        let new_raw = unsafe { rb_sys::rb_gc_location(raw) };
        // SAFETY: new_raw is a valid VALUE returned from rb_gc_location
        unsafe { crate::value::Value::from_raw(new_raw) }
    }

    /// Get the new location of a raw VALUE after compaction.
    ///
    /// Lower-level version for types that store raw VALUEs.
    #[inline]
    pub fn location_raw(&self, value: rb_sys::VALUE) -> rb_sys::VALUE {
        // SAFETY: rb_gc_location is safe to call during compaction phase
        unsafe { rb_sys::rb_gc_location(value) }
    }
}
