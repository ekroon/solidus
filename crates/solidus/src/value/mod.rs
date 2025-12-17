//! Core value types for Solidus.
//!
//! This module contains the fundamental types for working with Ruby values:
//!
//! - [`Value`] - Base wrapper around Ruby's VALUE
//! - [`StackPinned`] - `!Unpin` wrapper for stack pinning
//! - [`PinGuard`] - Guard that enforces pinning at creation time
//! - [`BoxValue`] - Heap-allocated, GC-registered wrapper
//! - [`ReprValue`] - Trait for types that represent Ruby values

mod boxed;
mod guard;
mod inner;
mod pinned;
mod traits;

pub use boxed::BoxValue;
pub use guard::PinGuard;
pub use inner::{Value, ValueType};
pub use pinned::StackPinned;
pub use traits::{IntoPinnable, ReprValue};
