//! Procedural macros for the solidus Ruby extension library.
//!
//! This crate provides the following macros:
//!
//! - `#[init]` - Mark a function as the Ruby extension entry point
//! - `#[wrap]` - Derive TypedData implementation for Rust types
//!
//! These macros are re-exported by the main `solidus` crate and should not be
//! used directly.

#![warn(missing_docs)]
#![warn(clippy::all)]

use proc_macro::TokenStream;

/// Marks a function as the Ruby extension entry point.
///
/// The function must have the signature `fn(Ruby) -> Result<(), Error>`.
///
/// # Example
///
/// ```ignore
/// use solidus::prelude::*;
///
/// #[init]
/// fn init(ruby: Ruby) -> Result<(), Error> {
///     // Define classes and methods here
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: Implement in Phase 3
    item
}

/// Derives TypedData implementation for a Rust struct.
///
/// This allows the struct to be wrapped as a Ruby object with proper
/// garbage collection integration.
///
/// # Example
///
/// ```ignore
/// use solidus::prelude::*;
///
/// #[wrap]
/// struct Point {
///     x: f64,
///     y: f64,
/// }
/// ```
#[proc_macro_attribute]
pub fn wrap(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: Implement in Phase 4
    item
}
