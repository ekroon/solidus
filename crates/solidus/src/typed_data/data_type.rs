use std::ffi::CStr;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::{c_ulong, c_void};
use std::ptr;

use super::traits::{DataTypeFunctions, TypedData};

/// Describes a Rust type to Ruby's TypedData system.
///
/// This struct wraps Ruby's `rb_data_type_t` and provides the callbacks
/// needed for proper garbage collection integration.
///
/// Use `DataTypeBuilder` to construct instances.
#[repr(transparent)]
pub struct DataType {
    inner: rb_sys::rb_data_type_t,
}

impl DataType {
    /// Get a pointer to the underlying `rb_data_type_t`.
    ///
    /// # Safety
    ///
    /// The returned pointer is valid for the lifetime of this `DataType`.
    #[inline]
    pub fn as_raw(&self) -> *const rb_sys::rb_data_type_t {
        &self.inner
    }

    /// Get the name of this data type.
    pub fn name(&self) -> &CStr {
        // SAFETY: wrap_struct_name is always a valid C string from a static &str
        unsafe { CStr::from_ptr(self.inner.wrap_struct_name) }
    }
}

// SAFETY: DataType contains only function pointers and static strings,
// which are safe to share across threads
unsafe impl Send for DataType {}
unsafe impl Sync for DataType {}

/// Builder for creating `DataType` instances.
///
/// # Example
///
/// ```ignore
/// use solidus::typed_data::{DataType, DataTypeBuilder};
///
/// struct Point { x: f64, y: f64 }
///
/// let data_type: DataType = DataTypeBuilder::<Point>::new("Point")
///     .free_immediately()
///     .build();
/// ```
pub struct DataTypeBuilder<T> {
    name: &'static str,
    free_immediately: bool,
    mark: bool,
    compact: bool,
    size: bool,
    _phantom: PhantomData<T>,
}

impl<T: TypedData> DataTypeBuilder<T> {
    /// Create a new builder with the given type name.
    ///
    /// The name is used for diagnostics and error messages.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            free_immediately: true, // Default to immediate freeing
            mark: false,
            compact: false,
            size: false,
            _phantom: PhantomData,
        }
    }

    /// Free the wrapped data immediately when the Ruby object is collected.
    ///
    /// This is the default behavior. The alternative (not calling this) defers
    /// freeing to the end of the GC cycle, which is rarely needed.
    pub const fn free_immediately(mut self) -> Self {
        self.free_immediately = true;
        self
    }

    /// Build the `DataType`.
    ///
    /// This method works for both `TypedData` and `DataTypeFunctions` types.
    /// For advanced GC callbacks (mark, compact, size), the callbacks are
    /// conditionally included based on the flags set via the builder methods.
    ///
    /// Note: The mark, compact, and size flags are silently ignored if `T`
    /// does not implement `DataTypeFunctions`.
    ///
    /// # Panics
    ///
    /// Panics if the name contains interior null bytes.
    pub fn build(self) -> DataType {
        // Create null-terminated name.
        // INTENTIONAL MEMORY LEAK: We deliberately leak this CString because
        // rb_data_type_t requires a pointer with 'static lifetime. Ruby will
        // reference this string for the entire lifetime of the program.
        // This is a standard pattern in Ruby C extensions.
        let name_cstr = CString::new(self.name).expect("DataType name must not contain null bytes");
        let name_ptr = name_cstr.into_raw();

        // Build flags
        let mut flags: rb_sys::VALUE = 0;
        if self.free_immediately {
            // RUBY_TYPED_FREE_IMMEDIATELY = 1
            // This flag tells Ruby to free the wrapped data immediately when
            // the Ruby object is collected, rather than deferring to the end
            // of the GC cycle. This is typically the desired behavior.
            flags |= RUBY_TYPED_FREE_IMMEDIATELY;
        }

        // Create the rb_data_type_t.
        // For TypedData-only types, mark/compact/size flags are ignored.
        let inner = rb_sys::rb_data_type_t {
            wrap_struct_name: name_ptr,
            function: rb_sys::rb_data_type_struct__bindgen_ty_1 {
                dmark: None,
                dfree: Some(free_callback::<T>),
                dsize: None,
                dcompact: None,
                reserved: [ptr::null_mut()],
            },
            parent: ptr::null(),
            data: ptr::null_mut(),
            flags,
        };

        DataType { inner }
    }
}

// Implementation for types with DataTypeFunctions support.
// This provides additional builder methods and an optimized build path.
impl<T: DataTypeFunctions> DataTypeBuilder<T> {
    /// Enable GC marking for this type.
    ///
    /// Requires that `T` implements `DataTypeFunctions`.
    /// The `mark` method will be called during GC to mark any Ruby values.
    pub const fn mark(mut self) -> Self {
        self.mark = true;
        self
    }

    /// Enable GC compaction support for this type.
    ///
    /// Requires that `T` implements `DataTypeFunctions`.
    /// The `compact` method will be called to update references after compaction.
    pub const fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Enable size reporting for GC statistics.
    ///
    /// Requires that `T` implements `DataTypeFunctions`.
    /// The `size` method will be called to report memory usage.
    pub const fn size(mut self) -> Self {
        self.size = true;
        self
    }

    /// Build the `DataType` with optional advanced GC callbacks.
    ///
    /// This specialized implementation is used when `T` implements `DataTypeFunctions`.
    /// It conditionally includes GC callbacks (mark, compact, size) based on the
    /// flags set via the builder methods.
    ///
    /// # Panics
    ///
    /// Panics if the name contains interior null bytes.
    pub fn build_with_callbacks(self) -> DataType {
        // Create null-terminated name.
        // INTENTIONAL MEMORY LEAK: We deliberately leak this CString because
        // rb_data_type_t requires a pointer with 'static lifetime. Ruby will
        // reference this string for the entire lifetime of the program.
        // This is a standard pattern in Ruby C extensions.
        let name_cstr = CString::new(self.name).expect("DataType name must not contain null bytes");
        let name_ptr = name_cstr.into_raw();

        // Build flags
        let mut flags: rb_sys::VALUE = 0;
        if self.free_immediately {
            // RUBY_TYPED_FREE_IMMEDIATELY = 1
            // This flag tells Ruby to free the wrapped data immediately when
            // the Ruby object is collected, rather than deferring to the end
            // of the GC cycle. This is typically the desired behavior.
            flags |= RUBY_TYPED_FREE_IMMEDIATELY;
        }

        // Create the rb_data_type_t with conditional callbacks.
        // The callbacks are included only if the corresponding flag is set.
        let inner = rb_sys::rb_data_type_t {
            wrap_struct_name: name_ptr,
            function: rb_sys::rb_data_type_struct__bindgen_ty_1 {
                dmark: if self.mark {
                    Some(mark_callback::<T>)
                } else {
                    None
                },
                dfree: Some(free_callback::<T>),
                dsize: if self.size {
                    Some(size_callback::<T>)
                } else {
                    None
                },
                dcompact: if self.compact {
                    Some(compact_callback::<T>)
                } else {
                    None
                },
                reserved: [ptr::null_mut()],
            },
            parent: ptr::null(),
            data: ptr::null_mut(),
            flags,
        };

        DataType { inner }
    }
}

/// Callback for freeing wrapped data.
///
/// # Safety
///
/// This is called by Ruby's GC. The `data` pointer must be a valid pointer
/// to a `T` that was allocated by `Box::into_raw`.
unsafe extern "C" fn free_callback<T>(data: *mut c_void) {
    if !data.is_null() {
        // SAFETY: data was created by Box::into_raw in wrap()
        let _ = unsafe { Box::from_raw(data as *mut T) };
    }
}

/// Callback for marking contained Ruby values.
///
/// # Safety
///
/// This is called by Ruby's GC. The `data` pointer must be a valid pointer to a `T`.
unsafe extern "C" fn mark_callback<T: DataTypeFunctions>(data: *mut c_void) {
    if !data.is_null() {
        let value = unsafe { &*(data as *const T) };
        let marker = super::Marker::new();
        value.mark(&marker);
    }
}

/// Callback for updating references after GC compaction.
///
/// # Safety
///
/// This is called by Ruby's GC. The `data` pointer must be a valid pointer to a `T`.
unsafe extern "C" fn compact_callback<T: DataTypeFunctions>(data: *mut c_void) {
    if !data.is_null() {
        let value = unsafe { &mut *(data as *mut T) };
        let compactor = super::Compactor::new();
        value.compact(&compactor);
    }
}

/// Callback for reporting memory size.
///
/// # Safety
///
/// This is called by Ruby's GC. The `data` pointer must be a valid pointer to a `T`.
unsafe extern "C" fn size_callback<T: DataTypeFunctions>(data: *const c_void) -> c_ulong {
    if !data.is_null() {
        let value = unsafe { &*(data as *const T) };
        value.size() as c_ulong
    } else {
        0
    }
}

// RUBY_TYPED_FREE_IMMEDIATELY constant value = 1
// This flag tells Ruby to free the wrapped data immediately when the Ruby
// object is collected, rather than deferring to the end of the GC cycle.
// This is typically the desired behavior for most TypedData wrappers as it
// ensures timely resource cleanup (file handles, network connections, etc.).
#[allow(dead_code)]
const RUBY_TYPED_FREE_IMMEDIATELY: rb_sys::VALUE = 1;

// Note: rb-sys may expose this as:
// rb_sys::ruby_typed_free_flag::RUBY_TYPED_FREE_IMMEDIATELY
