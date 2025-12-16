//! Solidus - A safe Rust library for writing Ruby extensions
//!
//! Solidus provides automatic stack pinning of Ruby values, eliminating the need for
//! users to manually ensure values stay on the stack. This prevents undefined behavior
//! that can occur when Ruby values are accidentally moved to the heap.
//!
//! # Safety Model
//!
//! Ruby's garbage collector scans the C stack to find live VALUE references. If a VALUE
//! is moved to the heap (e.g., into a `Vec` or `Box`), the GC cannot see it and may
//! collect the underlying Ruby object, leading to use-after-free bugs.
//!
//! Solidus solves this with compile-time enforcement:
//!
//! - Method arguments use `Pin<&StackPinned<T>>` which cannot be moved to the heap
//! - Explicit `BoxValue<T>` for heap storage registers values with Ruby's GC
//! - Immediate values (Fixnum, Symbol, true, false, nil) bypass pinning as they
//!   don't need GC protection
//!
//! # Core Types
//!
//! - [`Value`] - Base wrapper around Ruby's VALUE
//! - [`StackPinned<T>`](value::StackPinned) - `!Unpin` wrapper for stack pinning
//! - [`BoxValue<T>`] - Heap-allocated, GC-registered wrapper
//! - [`Ruby`] - Handle to the Ruby VM
//! - [`Error`] - Ruby exception wrapper
//!
//! # Method Registration
//!
//! Solidus provides `method!` and `function!` macros for registering Rust functions as Ruby methods:
//!
//! ```ignore
//! use solidus::prelude::*;
//!
//! // Define a Ruby method
//! fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
//!     // `other` is guaranteed to be on the stack - enforced by the type system
//!     let other_str = other.get().to_string()?;
//!     let self_str = rb_self.to_string()?;
//!     RString::new(&format!("{}{}", self_str, other_str))
//! }
//!
//! // Initialize the extension
//! #[solidus::init]
//! fn init(ruby: &Ruby) -> Result<(), Error> {
//!     let class = ruby.define_class("MyString", ruby.class_object())?;
//!     class.define_method("concat", method!(concat, 1), 1)?;
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Re-export rb-sys for low-level access
pub use rb_sys;

// Re-export proc-macros
// Note: #[method] and #[function] attribute macros are available via solidus_macros
// since the names conflict with the declarative method! and function! macros.
// Users should use:
//   - solidus::method!(func, arity) for the declarative macro
//   - #[solidus_macros::method] for the attribute macro
// The #[init] attribute macro doesn't conflict, so it's re-exported.
pub use solidus_macros::init;

// Modules
pub mod convert;
pub mod error;
pub mod gc;
pub mod method;
pub mod ruby;
pub mod types;
pub mod value;

// Re-exports for convenience
pub use error::{Error, ExceptionClass};
pub use ruby::Ruby;
pub use value::{BoxValue, ReprValue, StackPinned, Value, ValueType};

// Re-export all types
pub use types::{
    Encoding, Fixnum, Float, Integer, Module, Qfalse, Qnil, Qtrue, RArray, RBignum, RClass, RFloat,
    RHash, RModule, RString, Symbol,
};

#[cfg(target_pointer_width = "64")]
pub use types::Flonum;

/// Prelude module for convenient imports.
///
/// Use `use solidus::prelude::*;` to import commonly used types and traits.
pub mod prelude {
    pub use std::pin::Pin;

    pub use crate::convert::{IntoValue, TryConvert};
    pub use crate::error::{Error, ExceptionClass};
    pub use crate::init;
    pub use crate::pin_on_stack;
    pub use crate::ruby::Ruby;
    pub use crate::types::{
        Encoding, Fixnum, Float, Integer, Module, Qfalse, Qnil, Qtrue, RArray, RBignum, RClass,
        RFloat, RHash, RModule, RString, Symbol,
    };
    pub use crate::value::{BoxValue, ReprValue, StackPinned, Value, ValueType};

    #[cfg(target_pointer_width = "64")]
    pub use crate::types::Flonum;
}
