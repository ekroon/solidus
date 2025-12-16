//! Ruby type wrappers.
//!
//! This module provides type-safe wrappers around Ruby's built-in types.
//! Each type implements the core traits from the [`convert`](crate::convert) module.
//!
//! # Immediate vs Heap Types
//!
//! Ruby has two categories of values:
//!
//! - **Immediate values**: Encoded directly in the VALUE (Fixnum, Symbol, true, false, nil).
//!   These don't require GC protection and can be passed without pinning.
//! - **Heap values**: Allocated on the Ruby heap (String, Array, Hash, etc.).
//!   These require GC protection and must be stack-pinned or heap-boxed.
//!
//! # Example
//!
//! ```ignore
//! use solidus::prelude::*;
//! use solidus::types::{Fixnum, RString};
//!
//! fn example() -> Result<(), Error> {
//!     // Immediate values can be passed directly
//!     let num = Fixnum::from_i64(42).unwrap();
//!     
//!     // Heap values need pinning in method signatures
//!     let s = RString::new("hello");
//!     
//!     Ok(())
//! }
//! ```

mod array;
mod class;
mod float;
mod hash;
mod immediate;
mod integer;
mod module;
mod string;
mod symbol;

pub use array::RArray;
pub use class::RClass;
pub use hash::RHash;
pub use immediate::{Qfalse, Qnil, Qtrue};
pub use integer::{Fixnum, Integer, RBignum};
pub use module::{Module, RModule};
pub use string::{Encoding, RString};
pub use symbol::Symbol;

// Flonum is only available on 64-bit platforms
#[cfg(target_pointer_width = "64")]
pub use float::Flonum;

// RFloat and Float are always available
pub use float::{Float, RFloat};
