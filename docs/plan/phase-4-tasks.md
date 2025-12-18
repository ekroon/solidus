# Phase 4: TypedData - Implementation Tasks

This file contains the detailed task breakdown for Phase 4. Each task should be
completed in order, as later tasks depend on earlier ones.

## Prerequisites

- Phase 3 complete (method!, function!, #[init], define_method)
- Understanding of Ruby's TypedData C API
- Understanding of GC marking and compaction

## Task Status Legend

- [ ] Not started
- [x] Complete
- [~] In progress

---

## Stage 1: Module Structure

Create the typed_data module with proper organization.

### Task 4.1.1: Create typed_data module structure

**File**: `crates/solidus/src/typed_data/mod.rs`

- [ ] Create the `typed_data` directory
- [ ] Create `mod.rs` with submodule declarations
- [ ] Add module to `lib.rs`
- [ ] Re-export public items

```rust
// crates/solidus/src/typed_data/mod.rs
//! TypedData support for wrapping Rust types as Ruby objects.
//!
//! This module provides the infrastructure for wrapping arbitrary Rust types
//! as Ruby objects with proper garbage collection integration.
//!
//! # Example
//!
//! ```ignore
//! use solidus::prelude::*;
//!
//! #[solidus::wrap(class = "Point")]
//! struct Point {
//!     x: f64,
//!     y: f64,
//! }
//!
//! impl Point {
//!     fn new(x: f64, y: f64) -> Self {
//!         Self { x, y }
//!     }
//! }
//! ```

mod data_type;
mod marker;
mod traits;
mod wrap;

pub use data_type::{DataType, DataTypeBuilder};
pub use marker::{Compactor, Marker};
pub use traits::{DataTypeFunctions, TypedData};
pub use wrap::{get, get_mut, wrap};
```

**File**: `crates/solidus/src/lib.rs` (extend)

- [ ] Add `pub mod typed_data;`
- [ ] Add to prelude: `TypedData`, `DataTypeFunctions`, `wrap`, `get`, `get_mut`

**Acceptance**: Module compiles with empty submodules

---

## Stage 2: Core Traits

Define the traits that Rust types must implement to be wrapped as Ruby objects.

### Task 4.2.1: Implement TypedData trait

**File**: `crates/solidus/src/typed_data/traits.rs`

- [ ] Define the `TypedData` trait
- [ ] Require `Sized + Send` bounds
- [ ] Add `class_name()` method
- [ ] Add `data_type()` method returning static reference
- [ ] Add documentation with examples

```rust
// crates/solidus/src/typed_data/traits.rs

use super::DataType;

/// Trait for Rust types that can be wrapped in Ruby objects.
///
/// This trait is typically implemented via the `#[solidus::wrap]` attribute macro,
/// but can also be implemented manually for more control.
///
/// # Example (manual implementation)
///
/// ```ignore
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
pub trait TypedData: Sized + Send {
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
```

### Task 4.2.2: Implement DataTypeFunctions trait

**File**: `crates/solidus/src/typed_data/traits.rs` (extend)

- [ ] Define `DataTypeFunctions` trait
- [ ] Add `mark(&self, marker: &Marker)` with default empty impl
- [ ] Add `compact(&mut self, compactor: &Compactor)` with default empty impl
- [ ] Add `size(&self) -> usize` with default `size_of::<Self>()` impl
- [ ] Document when each function is called

```rust
use super::{Compactor, Marker};

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
/// ```ignore
/// use solidus::prelude::*;
/// use solidus::typed_data::{DataTypeFunctions, Marker, Compactor};
///
/// #[solidus::wrap(class = "Container", mark)]
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
```

**Acceptance**: `cargo test -p solidus typed_data::traits` passes (placeholder tests)

---

## Stage 3: DataType Struct

Create the `DataType` struct that wraps Ruby's `rb_data_type_t`.

### Task 4.3.1: Implement DataType struct

**File**: `crates/solidus/src/typed_data/data_type.rs`

- [ ] Create `DataType` struct wrapping `rb_data_type_t`
- [ ] Implement methods to access the inner type
- [ ] Add `as_raw()` method for FFI

```rust
// crates/solidus/src/typed_data/data_type.rs

use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr;

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
```

### Task 4.3.2: Implement DataTypeBuilder

**File**: `crates/solidus/src/typed_data/data_type.rs` (extend)

- [ ] Create `DataTypeBuilder<T>` generic struct
- [ ] Implement `new(name: &'static str)` constructor
- [ ] Implement `free_immediately()` option
- [ ] Implement `mark()` option (requires `DataTypeFunctions`)
- [ ] Implement `compact()` option (requires `DataTypeFunctions`)
- [ ] Implement `size()` option (requires `DataTypeFunctions`)
- [ ] Implement `build()` method that creates `DataType`
- [ ] Generate correct callback trampolines

```rust
use std::ffi::CString;
use std::marker::PhantomData;

use super::traits::{DataTypeFunctions, TypedData};

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

    /// Build the `DataType`.
    ///
    /// # Panics
    ///
    /// Panics if the name contains interior null bytes.
    pub fn build(self) -> DataType {
        // Create null-terminated name
        // We leak this because rb_data_type_t needs a static lifetime
        let name_cstr = CString::new(self.name)
            .expect("DataType name must not contain null bytes");
        let name_ptr = name_cstr.into_raw();

        // Build flags
        let mut flags: rb_sys::VALUE = 0;
        if self.free_immediately {
            // RUBY_TYPED_FREE_IMMEDIATELY flag
            flags |= rb_sys::ruby_typed_free_flag::RUBY_TYPED_FREE_IMMEDIATELY as rb_sys::VALUE;
        }

        // Create the rb_data_type_t
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
unsafe extern "C" fn size_callback<T: DataTypeFunctions>(data: *const c_void) -> usize {
    if !data.is_null() {
        let value = unsafe { &*(data as *const T) };
        value.size()
    } else {
        0
    }
}
```

**Note**: The callbacks use `extern "C"` functions that Ruby can call. The type parameter
`T` is monomorphized at compile time, creating separate functions for each wrapped type.

### Task 4.3.3: Handle flag constants

**File**: `crates/solidus/src/typed_data/data_type.rs` (extend)

- [ ] Check if `RUBY_TYPED_FREE_IMMEDIATELY` is available in rb-sys
- [ ] Add fallback constant if not available
- [ ] Document the flag's meaning

```rust
// Check rb-sys for the constant, define if missing
#[allow(dead_code)]
const RUBY_TYPED_FREE_IMMEDIATELY: rb_sys::VALUE = 1;

// Note: rb-sys may expose this as:
// rb_sys::ruby_typed_free_flag::RUBY_TYPED_FREE_IMMEDIATELY
```

**Acceptance**: `DataTypeBuilder::<Point>::new("Point").build()` compiles

---

## Stage 4: Wrap/Get Functions

Implement the core functions for wrapping and unwrapping Rust values.

### Task 4.4.1: Implement wrap function

**File**: `crates/solidus/src/typed_data/wrap.rs`

- [ ] Create `wrap<T>(ruby: &Ruby, class: RClass, value: T) -> Result<Value, Error>`
- [ ] Allocate value on heap with `Box::into_raw`
- [ ] Call `rb_data_typed_object_wrap`
- [ ] Return the wrapped value
- [ ] Add comprehensive documentation

```rust
// crates/solidus/src/typed_data/wrap.rs

use crate::error::Error;
use crate::ruby::Ruby;
use crate::types::RClass;
use crate::value::Value;

use super::TypedData;

/// Wrap a Rust value in a Ruby object.
///
/// This creates a new Ruby object that wraps the given Rust value. The Rust value
/// is moved to the heap and will be freed when the Ruby object is garbage collected.
///
/// # Arguments
///
/// * `ruby` - Reference to the Ruby runtime
/// * `class` - The Ruby class for the wrapped object (must be a subclass of Object)
/// * `value` - The Rust value to wrap
///
/// # Returns
///
/// A Ruby `Value` representing the wrapped object.
///
/// # Example
///
/// ```ignore
/// use solidus::prelude::*;
/// use solidus::typed_data::{wrap, TypedData};
///
/// #[solidus::wrap(class = "Point")]
/// struct Point { x: f64, y: f64 }
///
/// fn create_point(ruby: &Ruby, class: &RClass, x: f64, y: f64) -> Result<Value, Error> {
///     let point = Point { x, y };
///     wrap(ruby, class, point)
/// }
/// ```
///
/// # Safety
///
/// The wrapped value will be freed when the Ruby object is collected. Do not
/// attempt to access the value after the Ruby object has been collected.
pub fn wrap<T: TypedData>(_ruby: &Ruby, class: &RClass, value: T) -> Result<Value, Error> {
    // Allocate on heap
    let boxed = Box::new(value);
    let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;

    // Get the data type descriptor
    let data_type = T::data_type();

    // Create the Ruby object
    // SAFETY: class is a valid Ruby class, ptr is a valid heap pointer,
    // data_type describes T correctly
    let raw = unsafe {
        rb_sys::rb_data_typed_object_wrap(
            class.as_value().as_raw(),
            ptr,
            data_type.as_raw(),
        )
    };

    Ok(Value::from_raw(raw))
}
```

### Task 4.4.2: Implement get function

**File**: `crates/solidus/src/typed_data/wrap.rs` (extend)

- [ ] Create `get<T>(value: &Value) -> Result<&T, Error>`
- [ ] Use `rb_check_typeddata` for safe extraction with type checking
- [ ] Return appropriate error if type doesn't match
- [ ] Add documentation

```rust
/// Get a reference to the wrapped Rust value.
///
/// This extracts a reference to the Rust value wrapped in a Ruby object.
/// The reference is valid as long as the Ruby object is not collected.
///
/// # Arguments
///
/// * `value` - A Ruby Value that wraps a `T`
///
/// # Returns
///
/// A reference to the wrapped value, or an error if the value is not a
/// wrapped `T`.
///
/// # Example
///
/// ```ignore
/// use solidus::typed_data::get;
///
/// fn point_x(rb_self: Value) -> Result<f64, Error> {
///     let point: &Point = get(&rb_self)?;
///     Ok(point.x)
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The value is not a TypedData object
/// - The value wraps a different type than `T`
pub fn get<T: TypedData>(value: &Value) -> Result<&T, Error> {
    let data_type = T::data_type();

    // SAFETY: rb_check_typeddata returns NULL if type doesn't match,
    // otherwise returns the data pointer
    let ptr = unsafe {
        rb_sys::rb_check_typeddata(value.as_raw(), data_type.as_raw())
    };

    if ptr.is_null() {
        return Err(Error::type_error(format!(
            "expected {}, got {}",
            T::class_name(),
            value.class_name().unwrap_or_else(|_| "unknown".to_string())
        )));
    }

    // SAFETY: rb_check_typeddata verified this is a T
    let reference = unsafe { &*(ptr as *const T) };
    Ok(reference)
}
```

### Task 4.4.3: Implement get_mut function

**File**: `crates/solidus/src/typed_data/wrap.rs` (extend)

- [ ] Create `get_mut<T>(value: &Value) -> Result<&mut T, Error>`
- [ ] Document that this is unsafe if multiple references exist
- [ ] Recommend `RefCell` pattern for safe mutation
- [ ] Add documentation with examples

```rust
/// Get a mutable reference to the wrapped Rust value.
///
/// This extracts a mutable reference to the Rust value wrapped in a Ruby object.
///
/// # Safety Warning
///
/// This function does NOT provide any aliasing guarantees. If you call this
/// while another reference to the same data exists, you will have undefined
/// behavior. For safe mutation, use `RefCell<T>` inside your wrapped type:
///
/// ```ignore
/// use std::cell::RefCell;
///
/// #[solidus::wrap(class = "Counter")]
/// struct Counter(RefCell<i64>);
///
/// impl Counter {
///     fn increment(&self) -> i64 {
///         let mut val = self.0.borrow_mut();
///         *val += 1;
///         *val
///     }
/// }
/// ```
///
/// # Arguments
///
/// * `value` - A Ruby Value that wraps a `T`
///
/// # Returns
///
/// A mutable reference to the wrapped value, or an error if the value is not
/// a wrapped `T`.
///
/// # Errors
///
/// Returns an error if:
/// - The value is not a TypedData object
/// - The value wraps a different type than `T`
pub fn get_mut<T: TypedData>(value: &Value) -> Result<&mut T, Error> {
    let data_type = T::data_type();

    let ptr = unsafe {
        rb_sys::rb_check_typeddata(value.as_raw(), data_type.as_raw())
    };

    if ptr.is_null() {
        return Err(Error::type_error(format!(
            "expected {}, got {}",
            T::class_name(),
            value.class_name().unwrap_or_else(|_| "unknown".to_string())
        )));
    }

    // SAFETY: rb_check_typeddata verified this is a T
    // WARNING: Caller must ensure no aliasing
    let reference = unsafe { &mut *(ptr as *mut T) };
    Ok(reference)
}
```

### Task 4.4.4: Add helper method to Value

**File**: `crates/solidus/src/value/inner.rs` (extend)

- [ ] Add `class_name(&self) -> Result<String, Error>` method to Value
- [ ] Use `rb_obj_classname` or similar Ruby API

```rust
impl Value {
    /// Get the class name of this value.
    ///
    /// Returns the name of the Ruby class for this value.
    pub fn class_name(&self) -> Result<String, Error> {
        // SAFETY: rb_obj_classname is safe for any VALUE
        let name_ptr = unsafe { rb_sys::rb_obj_classname(self.as_raw()) };
        if name_ptr.is_null() {
            return Err(Error::runtime_error("could not get class name"));
        }
        let name = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
        Ok(name.to_string_lossy().into_owned())
    }
}
```

**Acceptance**: Can wrap and unwrap a simple struct

---

## Stage 5: GC Helpers

Implement helpers for marking and compacting Ruby values in TypedData.

### Task 4.5.1: Implement Marker helper

**File**: `crates/solidus/src/typed_data/marker.rs`

- [ ] Create `Marker` struct (zero-sized or minimal)
- [ ] Implement `mark<T: ReprValue>(&self, value: &T)` method
- [ ] Use existing `gc::mark` function internally
- [ ] Add documentation with examples

```rust
// crates/solidus/src/typed_data/marker.rs

use crate::gc;
use crate::value::traits::ReprValue;
use crate::value::BoxValue;

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
```

### Task 4.5.2: Implement Compactor helper

**File**: `crates/solidus/src/typed_data/marker.rs` (extend, or separate file)

- [ ] Create `Compactor` struct
- [ ] Implement `location<T: ReprValue>(&self, value: &T) -> Value` method
- [ ] Use `rb_gc_location` to get the new location
- [ ] Add documentation

```rust
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
        let new_raw = unsafe { rb_sys::rb_gc_location(raw) };
        crate::value::Value::from_raw(new_raw)
    }

    /// Get the new location of a raw VALUE after compaction.
    ///
    /// Lower-level version for types that store raw VALUEs.
    #[inline]
    pub fn location_raw(&self, value: rb_sys::VALUE) -> rb_sys::VALUE {
        unsafe { rb_sys::rb_gc_location(value) }
    }
}
```

**Acceptance**: `Marker` and `Compactor` compile and can be used in callbacks

---

## Stage 6: #[wrap] Attribute Macro

Implement the proc-macro that generates `TypedData` implementations.

### Task 4.6.1: Parse wrap attribute arguments

**File**: `crates/solidus-macros/src/lib.rs` (extend)

- [ ] Parse `class = "Name"` argument (required)
- [ ] Parse `free_immediately` flag (optional, default true)
- [ ] Parse `mark` flag (optional)
- [ ] Parse `compact` flag (optional)
- [ ] Parse `size` flag (optional)
- [ ] Add error handling for invalid arguments

```rust
// Helper struct to hold parsed wrap attributes
struct WrapArgs {
    class_name: String,
    free_immediately: bool,
    mark: bool,
    compact: bool,
    size: bool,
}

fn parse_wrap_args(attr: TokenStream) -> Result<WrapArgs, syn::Error> {
    // Parse as attribute arguments
    // Example: class = "Point", mark, size
    
    let mut class_name = None;
    let mut free_immediately = true;
    let mut mark = false;
    let mut compact = false;
    let mut size = false;

    // Parse comma-separated items
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("class") {
            let value: syn::LitStr = meta.value()?.parse()?;
            class_name = Some(value.value());
            Ok(())
        } else if meta.path.is_ident("free_immediately") {
            free_immediately = true;
            Ok(())
        } else if meta.path.is_ident("mark") {
            mark = true;
            Ok(())
        } else if meta.path.is_ident("compact") {
            compact = true;
            Ok(())
        } else if meta.path.is_ident("size") {
            size = true;
            Ok(())
        } else {
            Err(meta.error("unknown wrap attribute"))
        }
    });

    syn::parse::Parser::parse(parser, attr)?;

    let class_name = class_name.ok_or_else(|| {
        syn::Error::new(proc_macro2::Span::call_site(), "missing required `class` attribute")
    })?;

    Ok(WrapArgs {
        class_name,
        free_immediately,
        mark,
        compact,
        size,
    })
}
```

### Task 4.6.2: Generate TypedData implementation

**File**: `crates/solidus-macros/src/lib.rs` (extend)

- [ ] Generate `impl TypedData for T`
- [ ] Generate static `DataType` using `OnceLock`
- [ ] Use `DataTypeBuilder` with appropriate options
- [ ] Preserve the original struct definition

```rust
/// Marks a struct as wrappable in a Ruby object.
///
/// This attribute macro generates an implementation of the `TypedData` trait
/// for the annotated struct, allowing it to be wrapped as a Ruby object.
///
/// # Arguments
///
/// * `class = "Name"` - (Required) The Ruby class name for this type
/// * `free_immediately` - Free memory immediately when collected (default: true)
/// * `mark` - Enable GC marking (requires `DataTypeFunctions` impl)
/// * `compact` - Enable GC compaction (requires `DataTypeFunctions` impl)  
/// * `size` - Enable size reporting (requires `DataTypeFunctions` impl)
///
/// # Example
///
/// ```ignore
/// use solidus::prelude::*;
///
/// #[solidus::wrap(class = "Point")]
/// struct Point {
///     x: f64,
///     y: f64,
/// }
///
/// // For types with Ruby values, add marking:
/// #[solidus::wrap(class = "Container", mark)]
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
#[proc_macro_attribute]
pub fn wrap(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = match parse_wrap_args(attr) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };

    let input = syn::parse_macro_input!(item as syn::ItemStruct);
    let struct_name = &input.ident;
    let class_name = &args.class_name;

    // Build the DataTypeBuilder chain
    let mut builder_chain = quote::quote! {
        solidus::typed_data::DataTypeBuilder::<#struct_name>::new(#class_name)
    };

    if args.free_immediately {
        builder_chain = quote::quote! { #builder_chain.free_immediately() };
    }
    if args.mark {
        builder_chain = quote::quote! { #builder_chain.mark() };
    }
    if args.compact {
        builder_chain = quote::quote! { #builder_chain.compact() };
    }
    if args.size {
        builder_chain = quote::quote! { #builder_chain.size() };
    }

    let expanded = quote::quote! {
        #input

        impl solidus::typed_data::TypedData for #struct_name {
            fn class_name() -> &'static str {
                #class_name
            }

            fn data_type() -> &'static solidus::typed_data::DataType {
                static DATA_TYPE: std::sync::OnceLock<solidus::typed_data::DataType> = 
                    std::sync::OnceLock::new();
                DATA_TYPE.get_or_init(|| {
                    #builder_chain.build()
                })
            }
        }
    };

    expanded.into()
}
```

### Task 4.6.3: Re-export wrap macro from solidus

**File**: `crates/solidus/src/lib.rs` (extend)

- [ ] Re-export `#[wrap]` from solidus-macros
- [ ] Add to prelude
- [ ] Update documentation

```rust
// In lib.rs
pub use solidus_macros::wrap;

// In prelude
pub use crate::wrap;
```

**Acceptance**: `#[solidus::wrap(class = "Point")] struct Point { x: f64, y: f64 }` compiles

---

## Stage 7: Integration and Examples

Create examples and finalize the implementation.

### Task 4.7.1: Create phase4_typed_data example

**Directory**: `examples/phase4_typed_data/`

- [ ] Create `Cargo.toml` with dependencies
- [ ] Create `build.rs` using rb-sys-env
- [ ] Create `src/lib.rs` with Point example
- [ ] Create `test.rb` to exercise from Ruby
- [ ] Create `README.md` documenting the example

**File**: `examples/phase4_typed_data/Cargo.toml`

```toml
[package]
name = "phase4_typed_data"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
solidus = { path = "../../crates/solidus" }

[build-dependencies]
rb-sys-env = "0.1"
```

**File**: `examples/phase4_typed_data/src/lib.rs`

```rust
use solidus::prelude::*;
use solidus::typed_data::{wrap, get};

#[solidus::wrap(class = "Point")]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }

    fn distance(&self, other: &Point) -> f64 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }
}

// Wrapper functions for Ruby
fn point_new(x: f64, y: f64) -> Result<Value, Error> {
    let ruby = Ruby::get();
    let class = ruby.class_object(); // Should get Point class
    let point = Point::new(x, y);
    wrap(&ruby, &class, point)
}

fn point_x(rb_self: Value) -> Result<f64, Error> {
    let point: &Point = get(&rb_self)?;
    Ok(point.x())
}

fn point_y(rb_self: Value) -> Result<f64, Error> {
    let point: &Point = get(&rb_self)?;
    Ok(point.y())
}

fn point_distance(rb_self: Value, other: Value) -> Result<f64, Error> {
    let point: &Point = get(&rb_self)?;
    let other_point: &Point = get(&other)?;
    Ok(point.distance(other_point))
}

#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("Point", ruby.class_object())?;
    class.define_singleton_method("new", function!(point_new, 2))?;
    class.define_method("x", method!(point_x, 0))?;
    class.define_method("y", method!(point_y, 0))?;
    class.define_method("distance", method!(point_distance, 1))?;
    Ok(())
}
```

**File**: `examples/phase4_typed_data/test.rb`

```ruby
require_relative 'target/release/libphase4_typed_data'

# Test Point creation
p1 = Point.new(0.0, 0.0)
p2 = Point.new(3.0, 4.0)

puts "Point 1: (#{p1.x}, #{p1.y})"
puts "Point 2: (#{p2.x}, #{p2.y})"
puts "Distance: #{p1.distance(p2)}"

# Should print 5.0 (3-4-5 triangle)
raise "Expected 5.0" unless p1.distance(p2) == 5.0

puts "All tests passed!"
```

### Task 4.7.2: Create RefCell mutability example

**File**: `examples/phase4_typed_data/src/lib.rs` (extend or separate)

- [ ] Add Counter example with RefCell
- [ ] Show safe mutation pattern
- [ ] Test from Ruby

```rust
use std::cell::RefCell;

#[solidus::wrap(class = "Counter")]
struct Counter {
    value: RefCell<i64>,
}

impl Counter {
    fn new(initial: i64) -> Self {
        Self { value: RefCell::new(initial) }
    }

    fn get(&self) -> i64 {
        *self.value.borrow()
    }

    fn increment(&self) -> i64 {
        let mut val = self.value.borrow_mut();
        *val += 1;
        *val
    }
}
```

### Task 4.7.3: Create GC marking example

**File**: Create example showing types with Ruby values

- [ ] Add Container example with `Vec<BoxValue<Value>>`
- [ ] Implement `DataTypeFunctions::mark`
- [ ] Test GC doesn't collect contained values

```rust
use solidus::BoxValue;
use solidus::typed_data::{DataTypeFunctions, Marker};

#[solidus::wrap(class = "Container", mark)]
struct Container {
    items: Vec<BoxValue<Value>>,
}

impl DataTypeFunctions for Container {
    fn mark(&self, marker: &Marker) {
        for item in &self.items {
            marker.mark_boxed(item);
        }
    }
}
```

### Task 4.7.4: Update lib.rs exports

**File**: `crates/solidus/src/lib.rs`

- [ ] Export `typed_data` module
- [ ] Update prelude with key types
- [ ] Add module-level documentation

```rust
pub mod typed_data;

// In prelude
pub use crate::typed_data::{
    DataType, DataTypeFunctions, TypedData,
    get, get_mut, wrap,
    Marker, Compactor,
};
```

### Task 4.7.5: Add tests

**File**: `crates/solidus/tests/typed_data_tests.rs` or inline

- [ ] Test TypedData trait implementation
- [ ] Test wrap/get/get_mut functions
- [ ] Test with Ruby (feature-gated)
- [ ] Test GC marking (feature-gated)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct TestPoint {
        x: f64,
        y: f64,
    }

    impl TypedData for TestPoint {
        fn class_name() -> &'static str { "TestPoint" }
        fn data_type() -> &'static DataType {
            static DT: std::sync::OnceLock<DataType> = std::sync::OnceLock::new();
            DT.get_or_init(|| DataTypeBuilder::<TestPoint>::new("TestPoint").build())
        }
    }

    #[test]
    fn test_data_type_creation() {
        let dt = TestPoint::data_type();
        assert_eq!(dt.name().to_str().unwrap(), "TestPoint");
    }
}

#[cfg(all(test, any(feature = "link-ruby", feature = "embed")))]
mod ruby_tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;

    #[ruby_test]
    fn test_wrap_and_get() {
        let ruby = Ruby::get();
        let class = ruby.class_object();
        
        let point = TestPoint { x: 1.0, y: 2.0 };
        let wrapped = wrap(&ruby, &class, point).unwrap();
        
        let retrieved: &TestPoint = get(&wrapped).unwrap();
        assert_eq!(retrieved.x, 1.0);
        assert_eq!(retrieved.y, 2.0);
    }
}
```

### Task 4.7.6: Update documentation

- [ ] Add doc comments to all public items
- [ ] Add module-level documentation to `typed_data/mod.rs`
- [ ] Update README.md with TypedData example
- [ ] Update PROGRESS.md to mark Phase 4 complete
- [ ] Run `cargo doc` and verify

**Acceptance**: All Phase 4 acceptance criteria met

---

## Acceptance Criteria (Summary)

From `phase-4-typed-data.md`:

- [ ] `#[wrap]` macro generates correct trait implementations
- [ ] Rust types can be wrapped and unwrapped
- [ ] GC marking works for types with Ruby values
- [ ] RefCell pattern documented and tested
- [ ] Memory is correctly freed
- [ ] Type checking prevents wrong type access

---

## Ruby C API Functions Reference

| Operation | Function |
|-----------|----------|
| Create TypedData object | `rb_data_typed_object_wrap(klass, datap, type)` |
| Check TypedData type | `rb_check_typeddata(obj, data_type)` |
| Get data pointer | `RTYPEDDATA_GET_DATA(obj)` |
| Check if TypedData | `RTYPEDDATA_P(obj)` |
| Mark for GC | `rb_gc_mark(value)` |
| Get location after compaction | `rb_gc_location(value)` |
| Get class name | `rb_obj_classname(obj)` |

### rb_data_type_t Structure

```c
struct rb_data_type_struct {
    const char *wrap_struct_name;  // Name for diagnostics
    struct {
        RUBY_DATA_FUNC dmark;      // GC marking callback
        RUBY_DATA_FUNC dfree;      // Deallocation callback  
        size_t (*dsize)(const void *); // Size reporting callback
        RUBY_DATA_FUNC dcompact;   // GC compaction callback
        void *reserved[1];
    } function;
    const rb_data_type_t *parent;  // Parent type (for inheritance)
    void *data;                    // User data
    VALUE flags;                   // Behavioral flags
};
```

### Flag Constants

| Flag | Meaning |
|------|---------|
| `RUBY_TYPED_FREE_IMMEDIATELY` | Free data immediately when object collected |
| `RUBY_TYPED_WB_PROTECTED` | Object uses write barriers (advanced) |
| `RUBY_TYPED_FROZEN_SHAREABLE` | Object is shareable when frozen (Ractor) |

---

## Notes

### Design Decisions

1. **Immediate freeing by default**: We default to `RUBY_TYPED_FREE_IMMEDIATELY` because
   it's simpler and matches user expectations. Deferred freeing is rarely needed.

2. **RefCell for mutation**: Rather than provide an unsafe `get_mut` that could cause
   aliasing issues, we document and encourage the `RefCell<T>` pattern for safe
   interior mutability.

3. **No inheritance in Phase 4**: TypedData inheritance (via the `parent` field) is
   deferred to a later phase to keep the initial implementation simpler.

4. **Separate class definition**: The `#[wrap]` macro only generates trait impls.
   Users still define the Ruby class in their `#[init]` function, keeping concerns
   separated and giving full control over class hierarchy.

5. **Static DataType**: Each wrapped type gets a static `DataType` instance created
   via `OnceLock`. This ensures the `rb_data_type_t` has a stable address that Ruby
   can reference for the lifetime of the process.

### Memory Layout

When you call `wrap(ruby, class, value)`:

1. The Rust value is moved to the heap via `Box::new`
2. `Box::into_raw` gives us a raw pointer
3. `rb_data_typed_object_wrap` creates a Ruby object pointing to that data
4. When Ruby GCs the object, our `dfree` callback is called
5. `dfree` reconstructs the `Box` from the raw pointer and drops it

This means:
- The Rust value lives on the Rust heap, not Ruby's heap
- Ruby only holds a pointer to our data
- We control the lifetime through the GC callbacks

### GC Interaction

For types containing Ruby values (`BoxValue<T>`), you must:

1. Add `mark` to the `#[wrap]` attribute
2. Implement `DataTypeFunctions::mark`
3. Call `marker.mark()` for each Ruby value

Without proper marking, contained Ruby values may be collected while your wrapper
still references them, causing use-after-free bugs.

### Thread Safety

- `TypedData` requires `Send` bound because Ruby objects can be accessed from
  any thread (in theory, though GVL limits actual parallelism)
- For shared mutable state, use `RefCell` (single-threaded) or `Mutex`/`RwLock`
  (thread-safe) inside your wrapped type
- The `DataType` itself is `Send + Sync` because it only contains function pointers
  and static strings
