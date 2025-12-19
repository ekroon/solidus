//! Phase 3 Methods Example
//!
//! This example demonstrates the complete method registration system in Solidus,
//! showing all the different ways to register Rust functions as Ruby methods.
//!
//! Features demonstrated:
//! - Instance methods (using method! macro)
//! - Class methods (using function! macro with define_singleton_method)
//! - Module functions (using function! macro with define_module_function)
//! - Global functions (using function! macro with define_global_function)
//! - Various arities (0-3 arguments)
//! - Different return types (strings, integers)
//! - Error handling and propagation

use solidus::prelude::*;
use std::pin::Pin;

// ============================================================================
// Calculator Class - Demonstrates instance methods
// ============================================================================

/// Instance method with arity 0 - just self
/// Returns a greeting message
fn greet(rb_self: RString) -> Result<NewValue<RString>, Error> {
    let name = rb_self.to_string()?;
    Ok(RString::new(&format!("Hello, {}!", name)))
}

/// Instance method with arity 1 - self + one argument
/// Adds a number to another number (passed as string, converted to int)
fn add(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<i64, Error> {
    let a = rb_self.to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("first argument must be a number"))?;
    let b = other.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("second argument must be a number"))?;
    Ok(a + b)
}

/// Instance method with arity 2 - self + two arguments
/// Multiplies three numbers together
fn multiply_three(
    rb_self: RString,
    arg1: Pin<&StackPinned<RString>>,
    arg2: Pin<&StackPinned<RString>>,
) -> Result<i64, Error> {
    let a = rb_self.to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("first argument must be a number"))?;
    let b = arg1.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("second argument must be a number"))?;
    let c = arg2.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("third argument must be a number"))?;
    Ok(a * b * c)
}

/// Instance method that demonstrates error handling
/// Returns an error if called
fn always_fails(_rb_self: RString) -> Result<NewValue<RString>, Error> {
    Err(Error::runtime("This method always fails!"))
}

// ============================================================================
// StringUtils Module - Demonstrates module functions
// ============================================================================

/// Module function with arity 0
/// Returns a constant string
fn get_version() -> Result<NewValue<RString>, Error> {
    Ok(RString::new("1.0.0"))
}

/// Module function with arity 1
/// Converts a string to uppercase
fn to_upper(s: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let input = s.get().to_string()?;
    Ok(RString::new(&input.to_uppercase()))
}

/// Module function with arity 2
/// Joins two strings with a separator
fn join_with(
    s1: Pin<&StackPinned<RString>>,
    s2: Pin<&StackPinned<RString>>,
) -> Result<NewValue<RString>, Error> {
    let str1 = s1.get().to_string()?;
    let str2 = s2.get().to_string()?;
    Ok(RString::new(&format!("{} - {}", str1, str2)))
}

// ============================================================================
// Math Module - Demonstrates class methods on a module
// ============================================================================

/// Class method (singleton method) with arity 0
/// Returns a constant string representation of PI
fn pi() -> Result<NewValue<RString>, Error> {
    Ok(RString::new("3.14159"))
}

/// Class method with arity 1
/// Doubles a number (passed as string)
fn double(n: Pin<&StackPinned<RString>>) -> Result<i64, Error> {
    let num = n.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("argument must be a number"))?;
    Ok(num * 2)
}

/// Class method with arity 2
/// Calculates power (base^exponent, both as strings)
fn power(base: Pin<&StackPinned<RString>>, exponent: Pin<&StackPinned<RString>>) -> Result<i64, Error> {
    let b = base.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("base must be a number"))?;
    let e = exponent.get().to_string()?.parse::<u32>()
        .map_err(|_| Error::argument("exponent must be a positive number"))?;
    Ok(b.pow(e))
}

// ============================================================================
// Calculator Class (Continued) - Class methods
// ============================================================================

/// Class method that creates a new Calculator with a default name
fn create_default() -> Result<NewValue<RString>, Error> {
    Ok(RString::new("Calculator"))
}

/// Class method with arity 1
/// Creates a Calculator with a custom name
fn create_with_name(name: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let n = name.get().to_string()?;
    Ok(RString::new(&format!("Calculator: {}", n)))
}

// ============================================================================
// Global Functions - Available everywhere
// ============================================================================

/// Global function with arity 0
/// Returns a greeting
fn hello() -> Result<NewValue<RString>, Error> {
    Ok(RString::new("Hello from Solidus!"))
}

/// Global function with arity 1
/// Repeats a string n times
fn repeat_string(s: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let input = s.get().to_string()?;
    Ok(RString::new(&input.repeat(3)))
}

/// Global function with arity 2
/// Returns the sum of two integers (passed as strings)
fn add_numbers(a: Pin<&StackPinned<RString>>, b: Pin<&StackPinned<RString>>) -> Result<i64, Error> {
    let num_a = a.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("first argument must be a number"))?;
    let num_b = b.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("second argument must be a number"))?;
    Ok(num_a + num_b)
}

/// Global function with arity 3
/// Returns the average of three numbers (passed as strings)
fn average_three(
    a: Pin<&StackPinned<RString>>,
    b: Pin<&StackPinned<RString>>,
    c: Pin<&StackPinned<RString>>,
) -> Result<NewValue<RString>, Error> {
    let num_a = a.get().to_string()?.parse::<f64>()
        .map_err(|_| Error::argument("first argument must be a number"))?;
    let num_b = b.get().to_string()?.parse::<f64>()
        .map_err(|_| Error::argument("second argument must be a number"))?;
    let num_c = c.get().to_string()?.parse::<f64>()
        .map_err(|_| Error::argument("third argument must be a number"))?;
    let avg = (num_a + num_b + num_c) / 3.0;
    Ok(RString::new(&format!("{:.1}", avg)))
}

// ============================================================================
// Initialization - Register all classes, modules, and methods
// ============================================================================

fn init_solidus(ruby: &Ruby) -> Result<(), Error> {
    // ========================================================================
    // Define Calculator class and its methods
    // ========================================================================
    
    let calc_class = ruby.define_class("Calculator", ruby.class_object());
    let calc_rclass = RClass::try_convert(calc_class)?;
    
    // Instance methods using method! macro
    calc_rclass.clone().define_method("greet", solidus::method!(greet, 0), 0)?;
    calc_rclass.clone().define_method("add", solidus::method!(add, 1), 1)?;
    calc_rclass.clone().define_method("multiply_three", solidus::method!(multiply_three, 2), 2)?;
    calc_rclass.clone().define_method("always_fails", solidus::method!(always_fails, 0), 0)?;
    
    // Class methods using function! macro and define_singleton_method
    calc_rclass.clone().define_singleton_method(
        "create_default",
        solidus::function!(create_default, 0),
        0
    )?;
    calc_rclass.define_singleton_method(
        "create_with_name",
        solidus::function!(create_with_name, 1),
        1
    )?;
    
    // ========================================================================
    // Define StringUtils module and its module functions
    // ========================================================================
    
    let string_utils_module = ruby.define_module("StringUtils");
    let string_utils_rmodule = RModule::try_convert(string_utils_module)?;
    
    // Module functions using function! macro and define_module_function
    // These can be called as StringUtils.method_name or via include
    string_utils_rmodule.clone().define_module_function(
        "get_version",
        solidus::function!(get_version, 0),
        0
    )?;
    string_utils_rmodule.clone().define_module_function(
        "to_upper",
        solidus::function!(to_upper, 1),
        1
    )?;
    string_utils_rmodule.define_module_function(
        "join_with",
        solidus::function!(join_with, 2),
        2
    )?;
    
    // ========================================================================
    // Define Math module with class methods
    // ========================================================================
    
    let math_module = ruby.define_module("SolidusMath");
    let math_rmodule = RModule::try_convert(math_module)?;
    
    // Singleton methods on the module (class methods)
    math_rmodule.clone().define_singleton_method(
        "pi",
        solidus::function!(pi, 0),
        0
    )?;
    math_rmodule.clone().define_singleton_method(
        "double",
        solidus::function!(double, 1),
        1
    )?;
    math_rmodule.define_singleton_method(
        "power",
        solidus::function!(power, 2),
        2
    )?;
    
    // ========================================================================
    // Define global functions
    // ========================================================================
    
    ruby.define_global_function("hello", solidus::function!(hello, 0), 0)?;
    ruby.define_global_function("repeat_string", solidus::function!(repeat_string, 1), 1)?;
    ruby.define_global_function("add_numbers", solidus::function!(add_numbers, 2), 2)?;
    ruby.define_global_function("average_three", solidus::function!(average_three, 3), 3)?;
    
    Ok(())
}

// Ruby extension entry point
#[no_mangle]
pub unsafe extern "C" fn Init_phase3_methods() {
    // Mark this thread as the Ruby thread
    Ruby::mark_ruby_thread();
    
    // Get the Ruby handle
    let ruby = Ruby::get();
    
    // Call the init function and raise on error
    if let Err(e) = init_solidus(ruby) {
        e.raise();
    }
}
