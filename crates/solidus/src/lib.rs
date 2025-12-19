//! Solidus - A safe Rust library for writing Ruby extensions
//!
//! Solidus provides compile-time enforcement that Ruby values stay on the stack or are
//! explicitly registered with the garbage collector. This prevents undefined behavior that
//! can occur when Ruby values are accidentally moved to the heap.
//!
//! # The Problem
//!
//! Ruby's garbage collector scans the C stack to find live VALUE references. If a VALUE
//! is moved to the heap (e.g., into a `Vec` or `Box`), the GC cannot see it and may
//! collect the underlying Ruby object, leading to use-after-free bugs.
//!
//! Other libraries like Magnus rely on documentation to prevent this, but the type system
//! doesn't enforce it. See [Magnus issue #101](https://github.com/matsadler/magnus/issues/101).
//!
//! # The Solution: Pinned-From-Creation
//!
//! Solidus enforces safety at compile time through three mechanisms:
//!
//! 1. **All VALUE types are `!Copy`** - Prevents accidental duplication to heap
//! 2. **Creation returns `NewValue<T>`** - Forces explicit choice of stack or heap storage
//! 3. **Methods use `&self`** - Prevents moves of `!Copy` types
//!
//! ## Creating Values
//!
//! There are two main ways to create Ruby values:
//!
//! 1. **Inside methods**: Use the `Context` parameter provided by the `method!`/`function!` macros
//! 2. **For collections**: Use `*_boxed()` methods which immediately register with Ruby's GC
//!
//! ```no_run
//! use solidus::prelude::*;
//!
//! // For heap storage (collections), use the *_boxed() methods
//! let boxed = RArray::new_boxed();  // Safe - immediately GC-registered
//! let mut values = vec![boxed];  // Safe! GC knows about it
//!
//! // Inside methods, use Context (see Method Registration below)
//! ```
//!
//! # Core Types
//!
//! - [`Value`] - Base wrapper around Ruby's VALUE
//! - [`NewValue<T>`] - Guard that enforces pinning or boxing of new values
//! - [`StackPinned<T>`](value::StackPinned) - `!Unpin` wrapper for stack-pinned values
//! - [`BoxValue<T>`] - Heap-allocated, GC-registered wrapper
//! - [`Ruby`] - Handle to the Ruby VM
//! - [`Error`] - Ruby exception wrapper
//!
//! # Method Registration
//!
//! Define Rust functions as Ruby methods using `method!` or `function!` macros.
//! Methods receive a `&Context` as their first parameter for creating new Ruby values:
//!
//! ```no_run
//! use solidus::prelude::*;
//! use solidus::method;
//!
//! // Method that creates a new string using Context
//! fn concat<'a>(ctx: &'a Context, rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<Pin<&'a StackPinned<RString>>, Error> {
//!     let self_str = rb_self.to_string()?;
//!     let other_str = other.get().to_string()?;
//!     Ok(ctx.new_string(&format!("{}{}", self_str, other_str))?)
//! }
//!
//! // Initialize the extension
//! #[solidus::init]
//! fn init(ruby: &Ruby) -> Result<(), Error> {
//!     let class_val = ruby.define_class("MyString", ruby.class_object());
//!     let class = RClass::try_convert(class_val)?;
//!     class.define_method("concat", method!(concat, 1), 1)?;
//!     Ok(())
//! }
//! ```
//!
//! # Type Safety Guarantees
//!
//! - **Cannot store VALUES in Vec/HashMap without explicit `BoxValue`** - Compile error
//! - **Cannot forget to pin new values** - `#[must_use]` warning
//! - **Cannot copy VALUES to heap** - All heap types are `!Copy`
//! - **Immediate values remain `Copy`** - Fixnum, Symbol, nil, true, false are optimized

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
// The #[init] and #[wrap] attribute macros don't conflict, so they're re-exported.
pub use solidus_macros::{init, wrap};

// Modules
pub mod context;
pub mod convert;
pub mod error;
pub mod gc;
pub mod method;
pub mod ruby;
pub mod typed_data;
pub mod types;
pub mod value;

// Re-exports for convenience
pub use context::Context;
pub use error::{AllocationError, Error, ExceptionClass};
pub use ruby::Ruby;
pub use value::{BoxValue, NewValue, ReprValue, StackPinned, Value, ValueType};

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

    pub use crate::context::Context;
    pub use crate::convert::{IntoValue, TryConvert};
    pub use crate::error::{AllocationError, Error, ExceptionClass};
    pub use crate::init;
    pub use crate::method::{ReturnWitness, WitnessedReturn};
    pub use crate::pin_on_stack;
    pub use crate::ruby::Ruby;
    pub use crate::typed_data::{
        Compactor, DataType, DataTypeFunctions, Marker, TypedData, get, get_mut, wrap,
    };
    pub use crate::types::{
        Encoding, Fixnum, Float, Integer, Module, Qfalse, Qnil, Qtrue, RArray, RBignum, RClass,
        RFloat, RHash, RModule, RString, Symbol,
    };
    pub use crate::value::{BoxValue, NewValue, ReprValue, StackPinned, Value, ValueType};

    #[cfg(target_pointer_width = "64")]
    pub use crate::types::Flonum;
}
