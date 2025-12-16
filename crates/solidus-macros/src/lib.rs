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
use quote::quote;
use syn::{Expr, ItemFn, Lit, Meta, Token, parse_macro_input, punctuated::Punctuated};

/// Result type for macro operations.
type MacroResult<T> = Result<T, syn::Error>;

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
