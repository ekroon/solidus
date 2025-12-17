//! Procedural macros for the solidus Ruby extension library.
//!
//! This crate provides the following macros:
//!
//! - `#[init]` - Mark a function as the Ruby extension entry point
//! - `#[method]` - Generate wrappers for Ruby instance methods
//! - `#[function]` - Generate wrappers for Ruby global/module functions
//! - `#[wrap]` - Derive TypedData implementation for Rust types (planned)
//!
//! These macros are re-exported by the main `solidus` crate and should not be
//! used directly.
//!
//! # Automatic Pinning and Parameter Types
//!
//! The `#[method]` and `#[function]` macros automatically pin method arguments on the
//! stack for GC safety. However, you must use the correct parameter type based on what
//! you're passing:
//!
//! ## Ruby VALUE Types (RString, RArray, etc.)
//!
//! For Ruby VALUE types, you **must** use `Pin<&StackPinned<T>>` in your function
//! signature to ensure type safety:
//!
//! ```ignore
//! #[solidus::method]
//! fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
//!     // Access the inner value with .get()
//!     let other_str = other.get().to_string()?;
//!     // ...
//! }
//! ```
//!
//! All Ruby VALUE types are `!Copy` to prevent unsafe heap storage. The
//! `Pin<&StackPinned<T>>` signature ensures values remain pinned on the stack.
//!
//! ## Rust Primitives (i64, bool, String, etc.)
//!
//! For Rust primitive types that implement `TryConvert`, use the type directly:
//!
//! ```ignore
//! #[solidus::method]
//! fn repeat(rb_self: RString, count: i64) -> Result<PinGuard<RString>, Error> {
//!     // Use `count` directly as i64
//! }
//! ```
//!
//! The macro automatically converts Ruby VALUEs to Rust primitives when possible.

#![warn(missing_docs)]
#![warn(clippy::all)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, FnArg, GenericArgument, ItemFn, Lit, Meta, Pat, PathArguments, Token, Type,
    parse_macro_input, punctuated::Punctuated,
};

/// Result type for macro operations.
type MacroResult<T> = Result<T, syn::Error>;

/// Information about a parsed parameter.
struct ParamInfo {
    /// Whether the type is already `Pin<&StackPinned<T>>`
    is_explicit_pinned: bool,
    /// The inner type (T if `Pin<&StackPinned<T>>`, or the original type)
    inner_type: Type,
}

/// Check if a type is `Pin<&StackPinned<T>>` and extract the inner type T.
///
/// Returns `Some(T)` if the type matches `Pin<&StackPinned<T>>`, `None` otherwise.
fn extract_pinned_inner_type(ty: &Type) -> Option<Type> {
    // Match: Pin<&StackPinned<T>> or std::pin::Pin<&StackPinned<T>> or ::std::pin::Pin<...>
    let Type::Path(type_path) = ty else {
        return None;
    };

    // Get the last segment (should be "Pin")
    let last_seg = type_path.path.segments.last()?;
    if last_seg.ident != "Pin" {
        return None;
    }

    // Get the generic arguments of Pin<...>
    let PathArguments::AngleBracketed(pin_args) = &last_seg.arguments else {
        return None;
    };

    // Pin should have exactly one type argument
    let GenericArgument::Type(pin_inner) = pin_args.args.first()? else {
        return None;
    };

    // The inner type should be &StackPinned<T>
    let Type::Reference(ref_type) = pin_inner else {
        return None;
    };

    // Get the referenced type (should be StackPinned<T>)
    let Type::Path(stack_pinned_path) = ref_type.elem.as_ref() else {
        return None;
    };

    // Get the last segment (should be "StackPinned")
    let stack_pinned_seg = stack_pinned_path.path.segments.last()?;
    if stack_pinned_seg.ident != "StackPinned" {
        return None;
    }

    // Get the generic arguments of StackPinned<T>
    let PathArguments::AngleBracketed(sp_args) = &stack_pinned_seg.arguments else {
        return None;
    };

    // Extract T from StackPinned<T>
    let GenericArgument::Type(inner_type) = sp_args.args.first()? else {
        return None;
    };

    Some(inner_type.clone())
}

/// Parse a function parameter and extract its name and type information.
fn parse_param(param: &FnArg) -> MacroResult<ParamInfo> {
    let FnArg::Typed(pat_type) = param else {
        return Err(syn::Error::new_spanned(
            param,
            "expected typed parameter, not self",
        ));
    };

    let Pat::Ident(_pat_ident) = pat_type.pat.as_ref() else {
        return Err(syn::Error::new_spanned(
            &pat_type.pat,
            "expected identifier pattern",
        ));
    };

    let ty = (*pat_type.ty).clone();
    let (is_explicit_pinned, inner_type) = if let Some(inner) = extract_pinned_inner_type(&ty) {
        (true, inner)
    } else {
        (false, ty.clone())
    };

    Ok(ParamInfo {
        is_explicit_pinned,
        inner_type,
    })
}

/// Marks a function as the Ruby extension entry point.
///
/// The function must have the signature `fn(&Ruby) -> Result<(), Error>`.
///
/// By default, the generated initialization function will be named `Init_<crate_name>`,
/// where `<crate_name>` is the name of your crate with dashes converted to underscores.
///
/// You can override the extension name using the `name` parameter:
///
/// ```ignore
/// #[solidus::init(name = "my_extension")]
/// fn init(ruby: &Ruby) -> Result<(), Error> {
///     Ok(())
/// }
/// ```
///
/// # Examples
///
/// Basic usage with default naming:
///
/// ```ignore
/// use solidus::prelude::*;
///
/// #[solidus::init]
/// fn init(ruby: &Ruby) -> Result<(), Error> {
///     let class = ruby.define_class("MyClass", ruby.class_object());
///     // Define methods on class...
///     Ok(())
/// }
/// ```
///
/// Custom extension name:
///
/// ```ignore
/// use solidus::prelude::*;
///
/// #[solidus::init(name = "my_custom_name")]
/// fn init(ruby: &Ruby) -> Result<(), Error> {
///     Ok(())
/// }
/// ```
///
/// # Validation
///
/// The macro performs several validations:
///
/// - Extension names must be valid Ruby identifiers (start with letter or underscore,
///   contain only letters, digits, and underscores)
/// - Unknown attributes are rejected with helpful error messages
/// - The `name` attribute must be a string literal
///
/// This will generate:
///
/// ```ignore
/// #[no_mangle]
/// pub unsafe extern "C" fn Init_my_crate_name() {
///     solidus::Ruby::mark_ruby_thread();
///     let ruby = solidus::Ruby::get();
///     if let Err(e) = init(ruby) {
///         e.raise();
///     }
/// }
/// ```
///
/// # Safety
///
/// This macro generates unsafe code because it creates an unsafe extern "C" function
/// that interfaces with Ruby's C API. The generated code is safe to use as a Ruby
/// extension entry point.
///
/// ## Rust 2024 Compatibility
///
/// In Rust 2024 edition, calling unsafe extern "C" functions requires an unsafe block.
/// This is intentional - the generated `Init_*` function is only meant to be called by
/// Ruby's VM when loading the extension, not by user code. The unsafety is handled
/// internally with appropriate SAFETY comments documenting the invariants.
///
/// Users of this macro do not need to write any unsafe code themselves - the macro
/// handles all unsafe operations internally.
#[proc_macro_attribute]
pub fn init(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments (syn v2 uses Punctuated<Meta, Token![,]>)
    let attr_args = parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);

    // Parse the function
    let input_fn = parse_macro_input!(item as ItemFn);

    // Generate the expanded code, or return a compile error
    match init_impl(&attr_args, input_fn) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

/// Implementation of the init macro.
fn init_impl(
    attr_args: &Punctuated<Meta, Token![,]>,
    input_fn: ItemFn,
) -> MacroResult<TokenStream> {
    // Extract and validate the custom name if provided
    let custom_name = extract_name_from_attrs(attr_args)?;

    // Generate the Init_ function name
    let init_fn_name = if let Some(name) = custom_name {
        validate_name(&name)?;
        format!("Init_{}", name)
    } else {
        // Get crate name from CARGO_PKG_NAME environment variable
        let crate_name = std::env::var("CARGO_PKG_NAME")
            .unwrap_or_else(|_| "extension".to_string())
            .replace('-', "_");
        format!("Init_{}", crate_name)
    };

    let init_fn_ident = syn::Ident::new(&init_fn_name, proc_macro2::Span::call_site());
    let user_fn_name = &input_fn.sig.ident;

    // Generate the output
    let expanded = quote! {
        // Keep the original function
        #input_fn

        /// Ruby extension entry point.
        ///
        /// This function is automatically called by Ruby when the extension is loaded.
        #[no_mangle]
        #[allow(unsafe_op_in_unsafe_fn)]
        pub unsafe extern "C" fn #init_fn_ident() {
            // SAFETY: This function is the Ruby extension entry point and is called by Ruby.
            // We mark the Ruby thread and get the Ruby handle, which are safe operations
            // when called from Ruby's Init_ function.

            // Wrap everything in catch_unwind to handle panics gracefully
            let result = std::panic::catch_unwind(|| {
                // Mark this thread as the Ruby thread
                solidus::Ruby::mark_ruby_thread();

                // Get the Ruby handle
                let ruby = solidus::Ruby::get();

                // Call the user's init function and raise on error
                if let Err(e) = #user_fn_name(ruby) {
                    e.raise();
                }
            });

            // If a panic occurred, abort the process
            if result.is_err() {
                eprintln!("FATAL: Panic occurred during Ruby extension initialization");
                std::process::abort();
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

/// Extract the `name` parameter from attribute arguments.
///
/// Returns an error if:
/// - Unknown attributes are provided
/// - The `name` attribute value is not a string literal
fn extract_name_from_attrs(args: &Punctuated<Meta, Token![,]>) -> MacroResult<Option<String>> {
    let mut name = None;

    for arg in args {
        match arg {
            Meta::NameValue(nv) => {
                if nv.path.is_ident("name") {
                    if let Expr::Lit(expr_lit) = &nv.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            name = Some(lit_str.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "name attribute must be a string literal",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "name attribute must be a string literal",
                        ));
                    }
                } else {
                    return Err(syn::Error::new_spanned(
                        &nv.path,
                        format!(
                            "unknown attribute '{}', expected 'name'",
                            nv.path
                                .get_ident()
                                .map(|i| i.to_string())
                                .unwrap_or_default()
                        ),
                    ));
                }
            }
            Meta::Path(path) => {
                return Err(syn::Error::new_spanned(
                    path,
                    format!(
                        "unknown attribute '{}', expected 'name = \"...\"'",
                        path.get_ident().map(|i| i.to_string()).unwrap_or_default()
                    ),
                ));
            }
            Meta::List(list) => {
                return Err(syn::Error::new_spanned(
                    &list.path,
                    format!(
                        "unknown attribute '{}', expected 'name = \"...\"'",
                        list.path
                            .get_ident()
                            .map(|i| i.to_string())
                            .unwrap_or_default()
                    ),
                ));
            }
        }
    }

    Ok(name)
}

/// Validate that a name is a valid Ruby identifier.
///
/// Ruby identifiers must:
/// - Start with a letter or underscore
/// - Contain only letters, digits, and underscores
/// - Not be empty
fn validate_name(name: &str) -> MacroResult<()> {
    if name.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "extension name cannot be empty",
        ));
    }

    let mut chars = name.chars();

    // First character must be letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        Some(c) => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "extension name '{}' must start with a letter or underscore, found '{}'",
                    name, c
                ),
            ));
        }
        None => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "extension name cannot be empty",
            ));
        }
    }

    // Remaining characters must be letters, digits, or underscores
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "extension name '{}' contains invalid character '{}', only letters, digits, and underscores are allowed",
                    name, c
                ),
            ));
        }
    }

    Ok(())
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

/// Marks a function as a Ruby instance method.
///
/// This attribute macro generates an extern "C" wrapper function and a companion module
/// containing the arity constant and a `wrapper()` function that returns the function pointer.
///
/// The generated wrapper handles:
/// - Panic catching via `std::panic::catch_unwind`
/// - Type conversion of `self` via `TryConvert`
/// - Type conversion and stack pinning of arguments
/// - Error propagation (converts `Err` to Ruby exceptions)
/// - Return value conversion via `ReturnValue`
///
/// # Arguments
///
/// The first parameter must be `self` (the Ruby receiver), and subsequent parameters
/// are the method arguments.
///
/// # Automatic Pinning and Parameter Types
///
/// The macro **automatically pins all arguments on the stack** for GC safety. This
/// happens in the generated wrapper code, not in your function body.
///
/// You must choose the correct parameter type based on what you're working with:
///
/// ## Ruby VALUE Types (RString, RArray, etc.)
///
/// For Ruby VALUE types, use `Pin<&StackPinned<T>>` in your function signature:
///
/// ```ignore
/// #[solidus::method]
/// fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
///     let self_str = rb_self.to_string()?;
///     let other_str = other.get().to_string()?;  // Access with .get()
///     RString::new(&format!("{}{}", self_str, other_str))
/// }
/// ```
///
/// All Ruby VALUE types are `!Copy` to prevent unsafe heap storage. The
/// `Pin<&StackPinned<T>>` signature ensures the value remains pinned on the stack.
///
/// ## Rust Primitives (i64, bool, String, etc.)
///
/// For Rust primitive types, use the type directly:
///
/// ```ignore
/// #[solidus::method]
/// fn repeat(rb_self: RString, count: i64) -> Result<PinGuard<RString>, Error> {
///     let s = rb_self.to_string()?;
///     RString::new(&s.repeat(count as usize))
/// }
/// ```
///
/// The macro automatically converts Ruby VALUEs to Rust primitives when the type
/// implements `TryConvert`.
///
/// # Generated Code
///
/// For a method like:
///
/// ```ignore
/// #[solidus::method]
/// fn greet(rb_self: RString) -> Result<PinGuard<RString>, Error> {
///     Ok(rb_self.into())
/// }
/// ```
///
/// The macro generates:
///
/// ```ignore
/// fn greet(rb_self: RString) -> Result<PinGuard<RString>, Error> {
///     Ok(rb_self.into())
/// }
///
/// #[doc(hidden)]
/// #[allow(non_camel_case_types)]
/// pub mod __solidus_method_greet {
///     use super::*;
///     
///     pub const ARITY: i32 = 0;
///     
///     pub fn wrapper() -> unsafe extern "C" fn() -> solidus::rb_sys::VALUE {
///         // ... wrapper implementation
///     }
/// }
/// ```
///
/// # Example
///
/// ```ignore
/// use solidus::prelude::*;
///
/// // Ruby VALUE types use Pin<&StackPinned<T>>
/// #[solidus::method]
/// fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
///     let self_str = rb_self.to_string()?;
///     let other_str = other.get().to_string()?;
///     RString::new(&format!("{}{}", self_str, other_str))
/// }
///
/// // Rust primitives use the type directly
/// #[solidus::method]
/// fn repeat(rb_self: RString, count: i64) -> Result<PinGuard<RString>, Error> {
///     let s = rb_self.to_string()?;
///     RString::new(&s.repeat(count as usize))
/// }
///
/// // Register with Ruby using the generated module:
/// // class.define_method("concat", __solidus_method_concat::wrapper(), __solidus_method_concat::ARITY)?;
/// ```
///
/// # Supported Arities
///
/// Currently supports arities 0-2 (self + 0-2 arguments).
///
/// # Safety
///
/// The generated wrapper function is marked `unsafe extern "C"` because it interfaces
/// directly with Ruby's C API. The wrapper handles all safety concerns internally.
#[proc_macro_attribute]
pub fn method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    match method_impl(input_fn) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

/// Marks a function as a Ruby module/global function.
///
/// This attribute macro is similar to `#[solidus::method]` but for functions that don't
/// have a `self` parameter. It generates an extern "C" wrapper function and a companion
/// module containing the arity constant and a `wrapper()` function.
///
/// # Arguments
///
/// All parameters are method arguments.
///
/// # Automatic Pinning and Parameter Types
///
/// Like `#[solidus::method]`, this macro **automatically pins all arguments on the stack**
/// for GC safety. This happens in the generated wrapper code, not in your function body.
///
/// You must choose the correct parameter type based on what you're working with:
///
/// ## Ruby VALUE Types (RString, RArray, etc.)
///
/// For Ruby VALUE types, use `Pin<&StackPinned<T>>` in your function signature:
///
/// ```ignore
/// #[solidus::function]
/// fn greet(name: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
///     RString::new(&format!("Hello, {}!", name.get().to_string()?))
/// }
/// ```
///
/// All Ruby VALUE types are `!Copy` to prevent unsafe heap storage. The
/// `Pin<&StackPinned<T>>` signature ensures the value remains pinned on the stack.
///
/// ## Rust Primitives (i64, bool, String, etc.)
///
/// For Rust primitive types, use the type directly:
///
/// ```ignore
/// #[solidus::function]
/// fn add(a: i64, b: i64) -> Result<i64, Error> {
///     Ok(a + b)
/// }
/// ```
///
/// The macro automatically converts Ruby VALUEs to Rust primitives when the type
/// implements `TryConvert`.
///
/// # Generated Code
///
/// For a function like:
///
/// ```ignore
/// #[solidus::function]
/// fn add(a: i64, b: i64) -> Result<i64, Error> {
///     Ok(a + b)
/// }
/// ```
///
/// The macro generates:
///
/// ```ignore
/// fn add(a: i64, b: i64) -> Result<i64, Error> {
///     Ok(a + b)
/// }
///
/// #[doc(hidden)]
/// #[allow(non_camel_case_types)]
/// pub mod __solidus_function_add {
///     use super::*;
///     
///     pub const ARITY: i32 = 2;
///     
///     pub fn wrapper() -> unsafe extern "C" fn() -> solidus::rb_sys::VALUE {
///         // ... wrapper implementation
///     }
/// }
/// ```
///
/// # Example
///
/// ```ignore
/// use solidus::prelude::*;
///
/// // Ruby VALUE types use Pin<&StackPinned<T>>
/// #[solidus::function]
/// fn greet(name: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
///     let name_str = name.get().to_string()?;
///     RString::new(&format!("Hello, {}!", name_str))
/// }
///
/// // Rust primitives use the type directly
/// #[solidus::function]
/// fn add(a: i64, b: i64) -> Result<i64, Error> {
///     Ok(a + b)
/// }
///
/// // Register with Ruby using the generated module:
/// // ruby.define_global_function("greet", __solidus_function_greet::wrapper(), __solidus_function_greet::ARITY)?;
/// ```
///
/// # Supported Arities
///
/// Currently supports arities 0-2.
///
/// # Safety
///
/// The generated wrapper function is marked `unsafe extern "C"` because it interfaces
/// directly with Ruby's C API. The wrapper handles all safety concerns internally.
#[proc_macro_attribute]
pub fn function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    match function_impl(input_fn) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

/// Implementation of the #[method] attribute macro.
fn method_impl(input_fn: ItemFn) -> MacroResult<TokenStream> {
    let fn_name = &input_fn.sig.ident;
    let module_name = syn::Ident::new(
        &format!("__solidus_method_{}", fn_name),
        proc_macro2::Span::call_site(),
    );

    // Extract parameters
    let params: Vec<_> = input_fn.sig.inputs.iter().collect();

    // First parameter must be self (the Ruby receiver)
    if params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input_fn.sig,
            "method must have at least one parameter (self)",
        ));
    }

    // Parse all parameters
    let mut parsed_params = Vec::new();
    for param in &params {
        parsed_params.push(parse_param(param)?);
    }

    // Arity is number of parameters minus self
    let arity = (params.len() - 1) as i32;

    // Generate the wrapper based on parsed parameters
    let wrapper_fn = generate_method_wrapper_dynamic(fn_name, &parsed_params)?;

    let expanded = quote! {
        // Keep the original function
        #input_fn

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub mod #module_name {
            use super::*;

            /// The arity of this method (number of arguments excluding self).
            pub const ARITY: i32 = #arity;

            /// Returns the extern "C" wrapper function pointer for this method.
            ///
            /// This can be passed to `define_method` for Ruby method registration.
            pub fn wrapper() -> unsafe extern "C" fn() -> solidus::rb_sys::VALUE {
                #wrapper_fn
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

/// Implementation of the #[function] attribute macro.
fn function_impl(input_fn: ItemFn) -> MacroResult<TokenStream> {
    let fn_name = &input_fn.sig.ident;
    let module_name = syn::Ident::new(
        &format!("__solidus_function_{}", fn_name),
        proc_macro2::Span::call_site(),
    );

    // Extract parameters
    let params: Vec<_> = input_fn.sig.inputs.iter().collect();

    // Parse all parameters
    let mut parsed_params = Vec::new();
    for param in &params {
        parsed_params.push(parse_param(param)?);
    }

    // Arity is number of parameters (no self for functions)
    let arity = params.len() as i32;

    // Generate the wrapper based on parsed parameters
    let wrapper_fn = generate_function_wrapper_dynamic(fn_name, &parsed_params)?;

    let expanded = quote! {
        // Keep the original function
        #input_fn

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub mod #module_name {
            use super::*;

            /// The arity of this function (number of arguments).
            pub const ARITY: i32 = #arity;

            /// Returns the extern "C" wrapper function pointer for this function.
            ///
            /// This can be passed to `define_global_function` or `define_module_function`
            /// for Ruby function registration.
            pub fn wrapper() -> unsafe extern "C" fn() -> solidus::rb_sys::VALUE {
                #wrapper_fn
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

/// Generate the extern "C" wrapper for a method dynamically based on parsed parameters.
///
/// This function generates a wrapper that handles both explicit `Pin<&StackPinned<T>>`
/// parameters and implicit pinning for simple types.
fn generate_method_wrapper_dynamic(
    fn_name: &syn::Ident,
    params: &[ParamInfo],
) -> MacroResult<proc_macro2::TokenStream> {
    let arity = params.len().saturating_sub(1);
    if arity > 2 {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "#[solidus::method] currently supports arities 0-2, got {}. \
                 For higher arities, use the method! macro directly.",
                arity
            ),
        ));
    }

    // First param is self (no pinning needed).
    // DESIGN: The self parameter does not need pinning because:
    // 1. It comes directly from Ruby (rb_self) and Ruby guarantees the receiver is live
    //    for the duration of the method call.
    // 2. The method dispatch itself keeps the receiver on Ruby's stack, protecting it from GC.
    // 3. Arguments, however, may have been computed and could be the only reference to an
    //    object, so they need explicit pinning to prevent GC during the method body.
    let self_param = &params[0];
    let self_type = &self_param.inner_type;

    // Generate extern "C" parameter declarations
    let mut extern_params = vec![quote! { rb_self: solidus::rb_sys::VALUE }];
    for i in 0..arity {
        let arg_name = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
        extern_params.push(quote! { #arg_name: solidus::rb_sys::VALUE });
    }

    // Generate conversion code for each argument
    let mut conversion_stmts = Vec::new();
    let mut call_args = Vec::new();

    // Self conversion
    conversion_stmts.push(quote! {
        let self_value = unsafe { solidus::Value::from_raw(rb_self) };
        let self_converted: #self_type = solidus::convert::TryConvert::try_convert(self_value)?;
    });
    call_args.push(quote! { self_converted });

    // Argument conversions with pinning
    for (i, param) in params.iter().skip(1).enumerate() {
        let arg_value = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
        let arg_converted = syn::Ident::new(
            &format!("arg{}_converted", i),
            proc_macro2::Span::call_site(),
        );
        let arg_pinned =
            syn::Ident::new(&format!("arg{}_pinned", i), proc_macro2::Span::call_site());
        let inner_type = &param.inner_type;

        // Always convert and pin (the pinning is for GC safety)
        conversion_stmts.push(quote! {
            let #arg_value = unsafe { solidus::Value::from_raw(#arg_value) };
            let #arg_converted: #inner_type = solidus::convert::TryConvert::try_convert(#arg_value)?;
            solidus::pin_on_stack!(#arg_pinned = solidus::value::PinGuard::new(#arg_converted));
        });

        // Determine how to pass the argument to the user function
        if param.is_explicit_pinned {
            // User wants Pin<&StackPinned<T>>, pass the pinned reference directly
            call_args.push(quote! { #arg_pinned });
        } else {
            // User wants T directly - this only works for Rust primitives that implement TryConvert.
            // For Ruby VALUE types, users MUST use Pin<&StackPinned<T>>.
            // We pass the pinned value and let the compiler enforce the right signature.
            call_args.push(quote! { #arg_pinned });
        }
    }

    Ok(quote! {
        #[allow(unused_unsafe)]
        unsafe extern "C" fn __wrapper(
            #(#extern_params),*
        ) -> solidus::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                #(#conversion_stmts)*

                let result = #fn_name(#(#call_args),*);

                use solidus::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => solidus::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(__wrapper as usize) }
    })
}

/// Generate the extern "C" wrapper for a function dynamically based on parsed parameters.
///
/// This function generates a wrapper that handles both explicit `Pin<&StackPinned<T>>`
/// parameters and implicit pinning for simple types.
fn generate_function_wrapper_dynamic(
    fn_name: &syn::Ident,
    params: &[ParamInfo],
) -> MacroResult<proc_macro2::TokenStream> {
    let arity = params.len();
    if arity > 2 {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "#[solidus::function] currently supports arities 0-2, got {}. \
                 For higher arities, use the function! macro directly.",
                arity
            ),
        ));
    }

    // Generate extern "C" parameter declarations (always has _rb_self for Ruby)
    let mut extern_params = vec![quote! { _rb_self: solidus::rb_sys::VALUE }];
    for i in 0..arity {
        let arg_name = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
        extern_params.push(quote! { #arg_name: solidus::rb_sys::VALUE });
    }

    // Generate conversion code for each argument
    let mut conversion_stmts = Vec::new();
    let mut call_args = Vec::new();

    // Argument conversions with pinning
    for (i, param) in params.iter().enumerate() {
        let arg_value = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
        let arg_converted = syn::Ident::new(
            &format!("arg{}_converted", i),
            proc_macro2::Span::call_site(),
        );
        let arg_pinned =
            syn::Ident::new(&format!("arg{}_pinned", i), proc_macro2::Span::call_site());
        let inner_type = &param.inner_type;

        // Always convert and pin (the pinning is for GC safety)
        conversion_stmts.push(quote! {
            let #arg_value = unsafe { solidus::Value::from_raw(#arg_value) };
            let #arg_converted: #inner_type = solidus::convert::TryConvert::try_convert(#arg_value)?;
            solidus::pin_on_stack!(#arg_pinned = solidus::value::PinGuard::new(#arg_converted));
        });

        // Determine how to pass the argument to the user function
        if param.is_explicit_pinned {
            // User wants Pin<&StackPinned<T>>, pass the pinned reference directly
            call_args.push(quote! { #arg_pinned });
        } else {
            // User wants T directly - this only works for Rust primitives that implement TryConvert.
            // For Ruby VALUE types, users MUST use Pin<&StackPinned<T>>.
            // We pass the pinned value and let the compiler enforce the right signature.
            call_args.push(quote! { #arg_pinned });
        }
    }

    // Handle arity 0 case (no arguments to convert)
    let body = if params.is_empty() {
        quote! {
            let result = #fn_name();
        }
    } else {
        quote! {
            #(#conversion_stmts)*
            let result = #fn_name(#(#call_args),*);
        }
    };

    Ok(quote! {
        #[allow(unused_unsafe)]
        unsafe extern "C" fn __wrapper(
            #(#extern_params),*
        ) -> solidus::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                #body

                use solidus::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => solidus::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(__wrapper as usize) }
    })
}
