//! Marker and Compactor for GC operations

/// Helper for marking Ruby values during garbage collection.
///
/// This struct is passed to `DataTypeFunctions::mark` and provides methods
/// for marking Ruby values that should not be garbage collected.
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

    // Note: mark methods will be implemented in Stage 5 (Task 4.5.1)
}

/// Helper for updating Ruby value references after GC compaction.
///
/// This struct is passed to `DataTypeFunctions::compact` and provides methods
/// for getting the new location of Ruby values that may have moved during
/// garbage collection compaction.
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

    // Note: location methods will be implemented in Stage 5 (Task 4.5.2)
}
