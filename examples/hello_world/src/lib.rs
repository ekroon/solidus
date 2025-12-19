//! Hello World - Minimal Solidus Example
//!
//! This is the simplest possible Solidus Ruby extension, demonstrating:
//! - Basic project structure
//! - The `#[solidus::init]` macro for extension initialization
//! - A single global function definition

use solidus::prelude::*;

/// A simple function that returns a greeting.
#[solidus_macros::function]
fn hello() -> Result<NewValue<RString>, Error> {
    Ok(RString::new("Hello from Solidus!"))
}

/// Initialize the Ruby extension.
///
/// This function is called by Ruby when the extension is loaded.
#[solidus_macros::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Define a global function that can be called from Ruby
    ruby.define_global_function(
        "hello",
        __solidus_function_hello::wrapper(),
        __solidus_function_hello::ARITY,
    )?;

    Ok(())
}
