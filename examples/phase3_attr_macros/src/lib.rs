//! Phase 3 Attribute Macros Example
//!
//! This example demonstrates the `#[solidus_macros::method]` and `#[solidus_macros::function]`
//! attribute macros that provide **automatic pinning** for method arguments.
//!
//! # Key Features Demonstrated
//!
//! - **Automatic pinning**: Write signatures with `Pin<&StackPinned<T>>` and the macro's
//!   generated wrapper handles the stack pinning for you. You never manually pin values.
//! - **Compile-time safety**: VALUE types are `!Copy` after ADR-007, preventing accidental
//!   heap storage that would break GC safety.
//! - **Clean signatures**: Your function receives properly pinned references, ready to use.
//!
//! # Comparison with phase3_methods
//!
//! Both `phase3_methods` and `phase3_attr_macros` use the same `Pin<&StackPinned<T>>` signatures.
//! The difference is in how you write the code:
//!
//! - `phase3_methods`: Uses `method!` and `function!` declarative macros
//! - `phase3_attr_macros`: Uses `#[method]` and `#[function]` attribute macros
//!
//! Both provide the same automatic pinning behavior in the generated wrapper code.
//!
//! # How Pinning Prevents Unsafe Heap Storage
//!
//! After ADR-007, all VALUE types (RString, RArray, etc.) are `!Copy`. This means you
//! cannot accidentally move them to the heap, which would break GC safety.
//!
//! ## What the `Pin<&StackPinned<T>>` Signature Does
//!
//! When you write a function signature with `Pin<&StackPinned<RString>>`, the macro's
//! generated wrapper:
//!
//! 1. Receives raw VALUE arguments from Ruby
//! 2. Converts them to typed values (e.g., RString)
//! 3. Wraps them in StackPinned<T> on the wrapper's stack frame
//! 4. Pins them and passes Pin<&StackPinned<T>> to your function
//!
//! You never manually pin values - the macro does it for you.
//!
//! ## Reading Values from Pinned References
//!
//! Access the inner value using `.get()`:
//!
//! ```ignore
//! #[solidus_macros::method]
//! fn example(rb_self: RString, arg: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
//!     // Access the inner value with .get()
//!     let s = arg.get().to_string()?;
//!     Ok(RString::new(&format!("Got: {}", s)))
//! }
//! ```
//!
//! ## Why This Matters for GC Safety
//!
//! Ruby's garbage collector scans the C stack to find VALUE references. By enforcing
//! `!Copy` on VALUE types and requiring `Pin<&StackPinned<T>>` signatures, Solidus
//! ensures at compile time that:
//!
//! 1. VALUES cannot be moved to the heap (they're `!Copy`)
//! 2. The macro wrapper keeps VALUES pinned on its stack frame
//! 3. Ruby's GC can always find these VALUES during collection
//!
//! This prevents the undefined behavior that Magnus allows, where you can accidentally
//! store VALUEs in heap collections like `Vec<RString>` without GC protection.

use solidus::prelude::*;
use std::pin::Pin;

// ============================================================================
// Function Examples - All use Pin<&StackPinned<T>> for heap arguments
// ============================================================================

/// Global function with no arguments.
#[solidus_macros::function]
fn get_greeting() -> Result<PinGuard<RString>, Error> {
    Ok(RString::new("Hello from attribute macros!"))
}

/// Global function with automatic pinning via macro wrapper.
/// The macro handles converting Ruby VALUEs to RStrings and pinning them.
#[solidus_macros::function]
fn greet(name: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
    let name_str = name.get().to_string()?;
    Ok(RString::new(&format!("Hello, {}!", name_str)))
}

/// Global function with two arguments, both automatically pinned.
#[solidus_macros::function]
fn join_strings(first: Pin<&StackPinned<RString>>, second: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
    let a = first.get().to_string()?;
    let b = second.get().to_string()?;
    Ok(RString::new(&format!("{} {}", a, b)))
}

/// Instance method with no extra arguments (just self).
/// Self doesn't need pinning - Ruby guarantees the receiver is live during the call.
#[solidus_macros::method]
fn length(rb_self: RString) -> Result<i64, Error> {
    let s = rb_self.to_string()?;
    Ok(s.len() as i64)
}

/// Instance method with automatic pinning for the argument.
#[solidus_macros::method]
fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    Ok(RString::new(&format!("{}{}", self_str, other_str)))
}

/// Instance method with two automatically pinned arguments.
#[solidus_macros::method]
fn surround(rb_self: RString, prefix: Pin<&StackPinned<RString>>, suffix: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
    let p = prefix.get().to_string()?;
    let s = rb_self.to_string()?;
    let x = suffix.get().to_string()?;
    Ok(RString::new(&format!("{}{}{}", p, s, x)))
}

// ============================================================================
// All Signatures Use Pin<&StackPinned<T>> - The Macro Handles Pinning
// ============================================================================

/// Instance method - same Pin<&StackPinned<T>> signature style.
#[solidus_macros::method]
fn concat_explicit(
    rb_self: RString,
    other: Pin<&StackPinned<RString>>,
) -> Result<PinGuard<RString>, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    Ok(RString::new(&format!("{}{}", self_str, other_str)))
}

/// Global function - same Pin<&StackPinned<T>> signature style.
#[solidus_macros::function]
fn uppercase_explicit(s: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
    let input = s.get().to_string()?;
    Ok(RString::new(&input.to_uppercase()))
}

// ============================================================================
// Mixed Argument Types - Pinned and Immediate Values
// ============================================================================

/// Function demonstrating multiple pinned arguments.
#[solidus_macros::function]
fn format_mixed(
    explicit_arg: Pin<&StackPinned<RString>>,
    implicit_arg: Pin<&StackPinned<RString>>,
) -> Result<PinGuard<RString>, Error> {
    let a = explicit_arg.get().to_string()?;
    let b = implicit_arg.get().to_string()?;
    Ok(RString::new(&format!("[{}] -> [{}]", a, b)))
}

/// Method demonstrating pinned argument with self.
#[solidus_macros::method]
fn combine_mixed(
    rb_self: RString,
    explicit_arg: Pin<&StackPinned<RString>>,
) -> Result<PinGuard<RString>, Error> {
    let s = rb_self.to_string()?;
    let e = explicit_arg.get().to_string()?;
    Ok(RString::new(&format!("{}+{}", s, e)))
}

// ============================================================================
// Module Functions - Using attribute macros on module-level functions
// ============================================================================

/// Module function with automatic pinning.
#[solidus_macros::function]
fn to_upper(s: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
    let input = s.get().to_string()?;
    Ok(RString::new(&input.to_uppercase()))
}

/// Module function to reverse a string.
#[solidus_macros::function]
fn reverse(s: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
    let input = s.get().to_string()?;
    let reversed: String = input.chars().rev().collect();
    Ok(RString::new(&reversed))
}

/// Module function with two args.
#[solidus_macros::function]
fn repeat_join(text: Pin<&StackPinned<RString>>, separator: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
    let t = text.get().to_string()?;
    let sep = separator.get().to_string()?;
    Ok(RString::new(&format!("{}{}{}{}{}", t, sep, t, sep, t)))
}

// ============================================================================
// Initialization - Register everything with Ruby
// ============================================================================

fn init_solidus(ruby: &Ruby) -> Result<(), Error> {
    // ========================================================================
    // Global Functions
    // ========================================================================

    // Using the generated module: __solidus_function_<name>::wrapper() and ::ARITY
    ruby.define_global_function(
        "attr_get_greeting",
        __solidus_function_get_greeting::wrapper(),
        __solidus_function_get_greeting::ARITY,
    )?;
    ruby.define_global_function(
        "attr_greet",
        __solidus_function_greet::wrapper(),
        __solidus_function_greet::ARITY,
    )?;
    ruby.define_global_function(
        "attr_join_strings",
        __solidus_function_join_strings::wrapper(),
        __solidus_function_join_strings::ARITY,
    )?;
    ruby.define_global_function(
        "attr_uppercase_explicit",
        __solidus_function_uppercase_explicit::wrapper(),
        __solidus_function_uppercase_explicit::ARITY,
    )?;
    ruby.define_global_function(
        "attr_format_mixed",
        __solidus_function_format_mixed::wrapper(),
        __solidus_function_format_mixed::ARITY,
    )?;

    // ========================================================================
    // AttrString Class - Instance methods using attribute macros
    // ========================================================================

    let attr_string_class = ruby.define_class("AttrString", ruby.class_string());
    let attr_string_rclass = RClass::try_convert(attr_string_class)?;

    // Register instance methods using the generated modules
    attr_string_rclass.clone().define_method(
        "attr_length",
        __solidus_method_length::wrapper(),
        __solidus_method_length::ARITY,
    )?;
    attr_string_rclass.clone().define_method(
        "attr_concat",
        __solidus_method_concat::wrapper(),
        __solidus_method_concat::ARITY,
    )?;
    attr_string_rclass.clone().define_method(
        "attr_surround",
        __solidus_method_surround::wrapper(),
        __solidus_method_surround::ARITY,
    )?;
    attr_string_rclass.clone().define_method(
        "attr_concat_explicit",
        __solidus_method_concat_explicit::wrapper(),
        __solidus_method_concat_explicit::ARITY,
    )?;
    attr_string_rclass.clone().define_method(
        "attr_combine_mixed",
        __solidus_method_combine_mixed::wrapper(),
        __solidus_method_combine_mixed::ARITY,
    )?;

    // ========================================================================
    // AttrStringUtils Module - Module functions using attribute macros
    // ========================================================================

    let string_utils_module = ruby.define_module("AttrStringUtils");
    let string_utils_rmodule = RModule::try_convert(string_utils_module)?;

    string_utils_rmodule.clone().define_module_function(
        "to_upper",
        __solidus_function_to_upper::wrapper(),
        __solidus_function_to_upper::ARITY,
    )?;
    string_utils_rmodule.clone().define_module_function(
        "reverse",
        __solidus_function_reverse::wrapper(),
        __solidus_function_reverse::ARITY,
    )?;
    string_utils_rmodule.clone().define_module_function(
        "repeat_join",
        __solidus_function_repeat_join::wrapper(),
        __solidus_function_repeat_join::ARITY,
    )?;

    Ok(())
}

// Ruby extension entry point
#[no_mangle]
pub unsafe extern "C" fn Init_phase3_attr_macros() {
    // Mark this thread as the Ruby thread
    Ruby::mark_ruby_thread();

    // Get the Ruby handle
    let ruby = Ruby::get();

    // Call the init function and raise on error
    if let Err(e) = init_solidus(ruby) {
        e.raise();
    }
}
