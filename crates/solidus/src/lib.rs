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
//! # Example
//!
//! ```ignore
//! use solidus::prelude::*;
//!
//! fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
//!     // `other` is guaranteed to be on the stack - enforced by the type system
//!     rb_self.concat(other.get())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
