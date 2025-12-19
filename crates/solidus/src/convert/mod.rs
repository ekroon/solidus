//! Type conversion traits for Ruby values.
//!
//! This module provides two core traits for converting between Ruby and Rust types:
//!
//! - [`TryConvert`] - Convert from Ruby [`crate::value::Value`] to Rust types (fallible)
//! - [`IntoValue`] - Convert from Rust types to Ruby [`crate::value::Value`] (infallible)
//!
//! # Example
//!
//! ```no_run
//! use solidus::prelude::*;
//! use solidus::convert::{TryConvert, IntoValue};
//!
//! fn example(val: Value) -> Result<Value, Error> {
//!     // Convert Ruby value to Rust i64
//!     let n: i64 = i64::try_convert(val)?;
//!     
//!     // Convert Rust i64 back to Ruby value
//!     let result = (n * 2).into_value();
//!     Ok(result)
//! }
//! ```

mod into_value;
mod try_convert;

pub use into_value::IntoValue;
pub use try_convert::{TryConvert, TryConvertOwned};
