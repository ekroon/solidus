//! TypedData support for wrapping Rust types as Ruby objects.
//!
//! This module provides the infrastructure for wrapping arbitrary Rust types
//! as Ruby objects with proper garbage collection integration.
//!
//! # Example
//!
//! ```no_run
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
