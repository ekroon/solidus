//! Phase 3 Attribute Macros Example
//!
//! This example demonstrates the `#[solidus_macros::method]` and `#[solidus_macros::function]`
//! attribute macros that provide **implicit pinning** for method arguments.
//!
//! # Key Features Demonstrated
//!
//! - **Implicit pinning**: Write simple signatures like `fn foo(arg: RString)` instead of
//!   `fn foo(arg: Pin<&StackPinned<RString>>)`. The macro handles stack pinning automatically.
//! - **Explicit pinning**: Backward-compatible with explicit `Pin<&StackPinned<T>>` signatures.
//! - **Mixed signatures**: Can mix implicit and explicit pinning in the same function.
//! - **Copy bound enforcement**: Implicit pinning requires `T: Copy` (enforced at compile time).
//!
//! # Comparison with phase3_methods
//!
//! The `phase3_methods` example uses the `method!` and `function!` declarative macros,
//! which require explicit `Pin<&StackPinned<T>>` for all heap-allocated arguments.
//!
//! This example uses the attribute macros which allow simpler signatures via implicit pinning.
//!
//! # How Pinning Prevents Unsafe Heap Storage
//!
//! When you use explicit `Pin<&StackPinned<T>>` signatures, the Rust compiler prevents
//! you from moving the pinned value to the heap (e.g., into a `Vec`). This is crucial
//! for GC safety - Ruby values must stay on the stack where Ruby's GC can find them.
//!
//! ## What Works: Reading the Value
//!
//! You can safely read and copy the inner value from a pinned reference:
//!
//! ```ignore
//! #[solidus_macros::method]
//! fn safe_read(rb_self: RString, arg: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
//!     // This is safe - we copy the VALUE (which is just a pointer-sized integer)
//!     let value: RString = *arg.get();
//!     Ok(value)
//! }
//! ```
//!
//! ## What the Compiler Prevents: Moving Pinned References to Heap
//!
//! The following code would NOT compile because you cannot move a pinned reference
//! into a `Vec` (which lives on the heap):
//!
//! ```compile_fail
//! use solidus::prelude::*;
//! use std::pin::Pin;
//!
//! // This function tries to store pinned references in a Vec - this won't compile!
//! fn try_store_pinned_refs(args: Vec<Pin<&StackPinned<RString>>>) {
//!     // ERROR: Cannot collect pinned references into a Vec
//!     // The Pin guarantees the values stay on the stack
//! }
//!
//! #[solidus_macros::method]
//! fn bad_collect(
//!     rb_self: RString,
//!     arg: Pin<&StackPinned<RString>>,
//! ) -> Result<RString, Error> {
//!     // This won't compile - can't store the pinned reference in a Vec
//!     let mut heap_vec: Vec<Pin<&StackPinned<RString>>> = Vec::new();
//!     heap_vec.push(arg);  // ERROR: cannot move out of `arg`
//!     Ok(rb_self)
//! }
//! ```
//!
//! ## Why This Matters
//!
//! Ruby's garbage collector scans the C stack to find VALUE references. If a VALUE
//! is moved to the Rust heap (e.g., in a `Vec<RString>`), Ruby's GC won't find it
//! and may collect the underlying Ruby object while Rust still holds a reference.
//!
//! The `StackPinned<T>` wrapper combined with `Pin` provides compile-time guarantees
//! that VALUES stay on the stack where Ruby's GC can protect them.
//!
//! ## Implicit Pinning: The Best of Both Worlds
//!
//! With implicit pinning, you write simple signatures but get the same safety:
//!
//! ```ignore
//! #[solidus_macros::function]
//! fn greet(name: RString) -> Result<RString, Error> {
//!     // `name` was automatically pinned on the stack by the macro wrapper.
//!     // The user function receives a Copy of the VALUE, which is safe because:
//!     // 1. The original VALUE is pinned on the wrapper's stack frame
//!     // 2. RString is Copy (it's just a VALUE, a pointer-sized integer)
//!     // 3. Ruby's GC will find the pinned VALUE during any GC pause
//!     Ok(RString::new(&format!("Hello, {}!", name.to_string()?)))
//! }
//! ```

use solidus::prelude::*;
use std::pin::Pin;

// ============================================================================
// Implicit Pinning Examples - Simple, ergonomic signatures
// ============================================================================

/// Global function with no arguments.
#[solidus_macros::function]
fn get_greeting() -> Result<RString, Error> {
    Ok(RString::new("Hello from attribute macros!"))
}

/// Global function with implicit pinning.
/// Notice: `name` is just `RString`, not `Pin<&StackPinned<RString>>`.
#[solidus_macros::function]
fn greet(name: RString) -> Result<RString, Error> {
    let name_str = name.to_string()?;
    Ok(RString::new(&format!("Hello, {}!", name_str)))
}

/// Global function with two implicitly pinned arguments.
#[solidus_macros::function]
fn join_strings(first: RString, second: RString) -> Result<RString, Error> {
    let a = first.to_string()?;
    let b = second.to_string()?;
    Ok(RString::new(&format!("{} {}", a, b)))
}

/// Instance method with no extra arguments (just self).
/// Self doesn't need pinning - Ruby guarantees the receiver is live during the call.
#[solidus_macros::method]
fn length(rb_self: RString) -> Result<i64, Error> {
    let s = rb_self.to_string()?;
    Ok(s.len() as i64)
}

/// Instance method with implicit pinning for the argument.
#[solidus_macros::method]
fn concat(rb_self: RString, other: RString) -> Result<RString, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.to_string()?;
    Ok(RString::new(&format!("{}{}", self_str, other_str)))
}

/// Instance method with two implicitly pinned arguments.
#[solidus_macros::method]
fn surround(rb_self: RString, prefix: RString, suffix: RString) -> Result<RString, Error> {
    let p = prefix.to_string()?;
    let s = rb_self.to_string()?;
    let x = suffix.to_string()?;
    Ok(RString::new(&format!("{}{}{}", p, s, x)))
}

// ============================================================================
// Explicit Pinning Examples - For backward compatibility
// ============================================================================

/// Instance method using explicit `Pin<&StackPinned<T>>` signature.
/// This is the traditional style, still fully supported.
#[solidus_macros::method]
fn concat_explicit(
    rb_self: RString,
    other: Pin<&StackPinned<RString>>,
) -> Result<RString, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    Ok(RString::new(&format!("{}{}", self_str, other_str)))
}

/// Global function with explicit pinning.
#[solidus_macros::function]
fn uppercase_explicit(s: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
    let input = s.get().to_string()?;
    Ok(RString::new(&input.to_uppercase()))
}

// ============================================================================
// Mixed Pinning Examples - Combine implicit and explicit in one function
// ============================================================================

/// Function with mixed signatures: first arg explicit, second arg implicit.
#[solidus_macros::function]
fn format_mixed(
    explicit_arg: Pin<&StackPinned<RString>>,
    implicit_arg: RString,
) -> Result<RString, Error> {
    let a = explicit_arg.get().to_string()?;
    let b = implicit_arg.to_string()?;
    Ok(RString::new(&format!("[{}] -> [{}]", a, b)))
}

/// Method with mixed signatures.
#[solidus_macros::method]
fn combine_mixed(
    rb_self: RString,
    explicit_arg: Pin<&StackPinned<RString>>,
) -> Result<RString, Error> {
    let s = rb_self.to_string()?;
    let e = explicit_arg.get().to_string()?;
    Ok(RString::new(&format!("{}+{}", s, e)))
}

// ============================================================================
// Module Functions - Using attribute macros on module-level functions
// ============================================================================

/// Module function with implicit pinning.
#[solidus_macros::function]
fn to_upper(s: RString) -> Result<RString, Error> {
    let input = s.to_string()?;
    Ok(RString::new(&input.to_uppercase()))
}

/// Module function to reverse a string.
#[solidus_macros::function]
fn reverse(s: RString) -> Result<RString, Error> {
    let input = s.to_string()?;
    let reversed: String = input.chars().rev().collect();
    Ok(RString::new(&reversed))
}

/// Module function with two args.
#[solidus_macros::function]
fn repeat_join(text: RString, separator: RString) -> Result<RString, Error> {
    let t = text.to_string()?;
    let sep = separator.to_string()?;
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
    attr_string_rclass.define_method(
        "attr_length",
        __solidus_method_length::wrapper(),
        __solidus_method_length::ARITY,
    )?;
    attr_string_rclass.define_method(
        "attr_concat",
        __solidus_method_concat::wrapper(),
        __solidus_method_concat::ARITY,
    )?;
    attr_string_rclass.define_method(
        "attr_surround",
        __solidus_method_surround::wrapper(),
        __solidus_method_surround::ARITY,
    )?;
    attr_string_rclass.define_method(
        "attr_concat_explicit",
        __solidus_method_concat_explicit::wrapper(),
        __solidus_method_concat_explicit::ARITY,
    )?;
    attr_string_rclass.define_method(
        "attr_combine_mixed",
        __solidus_method_combine_mixed::wrapper(),
        __solidus_method_combine_mixed::ARITY,
    )?;

    // ========================================================================
    // AttrStringUtils Module - Module functions using attribute macros
    // ========================================================================

    let string_utils_module = ruby.define_module("AttrStringUtils");
    let string_utils_rmodule = RModule::try_convert(string_utils_module)?;

    string_utils_rmodule.define_module_function(
        "to_upper",
        __solidus_function_to_upper::wrapper(),
        __solidus_function_to_upper::ARITY,
    )?;
    string_utils_rmodule.define_module_function(
        "reverse",
        __solidus_function_reverse::wrapper(),
        __solidus_function_reverse::ARITY,
    )?;
    string_utils_rmodule.define_module_function(
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
