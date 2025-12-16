//! Core value types for Solidus.
//!
//! This module contains the fundamental types for working with Ruby values:
//!
//! - [`Value`] - Base wrapper around Ruby's VALUE
//! - [`StackPinned`] - `!Unpin` wrapper for stack pinning
//! - [`BoxValue`] - Heap-allocated, GC-registered wrapper
//! - [`ReprValue`] - Trait for types that represent Ruby values

mod boxed;
mod inner;
mod pinned;
mod traits;

pub use boxed::BoxValue;
pub use inner::{Value, ValueType};
pub use pinned::StackPinned;
pub use traits::ReprValue;
