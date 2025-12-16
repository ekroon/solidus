//! Method registration infrastructure.
//!
//! This module provides traits and types for registering Rust functions as Ruby methods.
//! The core design is based on automatic stack pinning of heap-allocated Ruby values to
//! prevent them from being moved to the heap where the GC cannot track them.
//!
//! # Overview
//!
//! The method registration system consists of:
//!
//! - [`MethodArg`] - Marker trait for types that can be method arguments
//! - [`ReturnValue`] - Trait for types that can be returned from methods
//!
//! # Example
//!
//! ```ignore
//! use solidus::prelude::*;
//!
//! // Method with mixed immediate and heap arguments
//! fn insert(rb_self: RArray, index: i64, value: RString) -> Result<RArray, Error> {
//!     // `index` is immediate (no pinning needed)
//!     // `value` is heap-allocated (automatically pinned by wrapper)
//!     rb_self.insert(index, value)
//! }
//! ```

mod args;
mod return_value;

pub use args::MethodArg;
pub use return_value::ReturnValue;
