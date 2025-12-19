use super::{Compactor, DataType, Marker};

/// Trait for Rust types that can be wrapped in Ruby objects.
///
/// This trait is typically implemented via the `#[solidus::wrap]` attribute macro,
/// but can also be implemented manually for more control.
///
/// # Example (manual implementation)
///
/// ```no_run
/// use solidus::typed_data::{DataType, DataTypeBuilder, TypedData};
///
/// struct Point {
///     x: f64,
///     y: f64,
/// }
///
/// impl TypedData for Point {
///     fn class_name() -> &'static str {
///         "Point"
///     }
///
///     fn data_type() -> &'static DataType {
///         static DATA_TYPE: std::sync::OnceLock<DataType> = std::sync::OnceLock::new();
///         DATA_TYPE.get_or_init(|| {
///             DataTypeBuilder::<Point>::new("Point").build()
///         })
///     }
/// }
/// ```
///
/// # Safety
///
/// The `data_type()` method must return a reference to a `DataType` that correctly
/// describes this type's memory layout and GC requirements. Using the `#[wrap]` macro
/// ensures this is done correctly.
///
/// The `'static` bound is required because wrapped values are stored in Ruby objects
/// and can live indefinitely (until the Ruby object is garbage collected).
pub trait TypedData: Sized + Send + 'static {
    /// The Ruby class name for this type.
    ///
    /// This is used for error messages and debugging.
    fn class_name() -> &'static str;

    /// The DataType descriptor for this type.
    ///
    /// This must return a reference to a static `DataType` instance that describes
    /// how Ruby should handle instances of this type (marking, freeing, etc.).
    fn data_type() -> &'static DataType;
}

/// Optional trait for types that need custom GC behavior.
///
/// Implement this trait when your wrapped type contains Ruby values that need
/// to be marked during garbage collection, or when you want to report custom
/// memory sizes for GC statistics.
///
/// # When to implement
///
/// - **`mark`**: Your type contains `BoxValue<T>` or raw Ruby VALUEs
/// - **`compact`**: Your type contains Ruby values that may move during GC compaction
/// - **`size`**: Your type allocates additional memory beyond `size_of::<Self>()`
///
/// # Example
///
/// ```no_run
/// use solidus::prelude::*;
/// use solidus::typed_data::{DataType, DataTypeBuilder, DataTypeFunctions, Marker, Compactor};
///
/// struct Container {
///     items: Vec<BoxValue<Value>>,
/// }
///
/// impl TypedData for Container {
///     fn class_name() -> &'static str { "Container" }
///     fn data_type() -> &'static DataType {
///         static DT: std::sync::OnceLock<DataType> = std::sync::OnceLock::new();
///         DT.get_or_init(|| DataTypeBuilder::<Container>::new("Container").mark().size().build_with_callbacks())
///     }
/// }
///
/// impl DataTypeFunctions for Container {
///     fn mark(&self, marker: &Marker) {
///         for item in &self.items {
///             marker.mark(&**item);
///         }
///     }
///
///     fn size(&self) -> usize {
///         std::mem::size_of::<Self>() +
///             self.items.capacity() * std::mem::size_of::<BoxValue<Value>>()
///     }
/// }
/// ```
pub trait DataTypeFunctions: TypedData {
    /// Mark any Ruby values this type contains.
    ///
    /// Called during GC marking phase. Use `marker.mark(value)` to mark
    /// any Ruby values your type holds references to.
    ///
    /// Default implementation does nothing (appropriate for types without Ruby values).
    #[inline]
    fn mark(&self, _marker: &Marker) {}

    /// Update any Ruby values after GC compaction.
    ///
    /// Called during GC compaction. Use `compactor.location(value)` to get
    /// the new location of moved values and update your references.
    ///
    /// Default implementation does nothing (appropriate for types without Ruby values).
    #[inline]
    fn compact(&mut self, _compactor: &Compactor) {}

    /// Report the size of this value for GC statistics.
    ///
    /// Should return the total memory used by this instance, including any
    /// heap allocations owned by the instance.
    ///
    /// Default implementation returns `size_of::<Self>()`.
    #[inline]
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
