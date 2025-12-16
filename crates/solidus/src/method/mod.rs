//! Method registration infrastructure.
//!
//! This module provides traits, types, and macros for registering Rust functions as Ruby methods.
//! The core design is based on automatic stack pinning of heap-allocated Ruby values to
//! prevent them from being moved to the heap where the GC cannot track them.
//!
//! # Overview
//!
//! The method registration system consists of:
//!
//! - [`MethodArg`] - Marker trait for types that can be method arguments
//! - [`ReturnValue`] - Trait for types that can be returned from methods
//! - `method!` - Macro for wrapping Rust functions as Ruby methods
//!
//! # Example
//!
//! ```ignore
//! use solidus::prelude::*;
//!
//! // Define a method
//! fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
//!     // `other` is pinned on the stack, safe from GC
//!     let other_str = other.get();
//!     // ... implement concat logic
//!     Ok(rb_self)
//! }
//!
//! // Register the method
//! // class.define_method("concat", method!(concat, 1), 1)?;
//! ```

mod args;
mod return_value;

pub use args::MethodArg;
pub use return_value::ReturnValue;

/// Generate an extern "C" wrapper for a Ruby method.
///
/// This macro creates a wrapper function that can be passed to Ruby's method
/// registration functions (like `rb_define_method` from the Ruby C API). The wrapper handles:
///
/// - Panic catching via `std::panic::catch_unwind`
/// - Type conversion of arguments via `TryConvert`
/// - Stack pinning of heap-allocated arguments
/// - Error propagation (converts `Err` to Ruby exceptions)
/// - Return value conversion via `IntoValue`
///
/// # Arity
///
/// The macro requires specifying the arity (number of arguments excluding self).
/// Use `method!(function_name, arity)` where arity is 0-15.
///
/// # Example
///
/// ```ignore
/// use solidus::prelude::*;
///
/// // Arity 0 - just self
/// fn length(rb_self: RString) -> Result<i64, Error> {
///     Ok(rb_self.len() as i64)
/// }
///
/// // Arity 1 - self + one argument  
/// fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
///     // other is automatically pinned by the wrapper
///     Ok(rb_self)
/// }
///
/// // Register with Ruby
/// // class.define_method("length", method!(length, 0), 0)?;
/// // class.define_method("concat", method!(concat, 1), 1)?;
/// ```
#[macro_export]
macro_rules! method {
    // Arity 0 - self only
    ($func:path, 0) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn wrapper(rb_self: $crate::rb_sys::VALUE) -> $crate::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                let self_value = unsafe { $crate::Value::from_raw(rb_self) };
                let self_converted = $crate::convert::TryConvert::try_convert(self_value)?;

                let result = $func(self_converted);

                use $crate::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => $crate::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(wrapper as usize) }
    }};

    // Arity 1 - self + 1 argument
    ($func:path, 1) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn wrapper(
            rb_self: $crate::rb_sys::VALUE,
            arg0: $crate::rb_sys::VALUE,
        ) -> $crate::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                let self_value = unsafe { $crate::Value::from_raw(rb_self) };
                let self_converted = $crate::convert::TryConvert::try_convert(self_value)?;

                let arg0_value = unsafe { $crate::Value::from_raw(arg0) };
                let arg0_converted = $crate::convert::TryConvert::try_convert(arg0_value)?;
                $crate::pin_on_stack!(arg0_pinned = arg0_converted);

                let result = $func(self_converted, arg0_pinned);

                use $crate::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => $crate::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(wrapper as usize) }
    }};

    // Arity 2 - self + 2 arguments
    ($func:path, 2) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn wrapper(
            rb_self: $crate::rb_sys::VALUE,
            arg0: $crate::rb_sys::VALUE,
            arg1: $crate::rb_sys::VALUE,
        ) -> $crate::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                let self_value = unsafe { $crate::Value::from_raw(rb_self) };
                let self_converted = $crate::convert::TryConvert::try_convert(self_value)?;

                let arg0_value = unsafe { $crate::Value::from_raw(arg0) };
                let arg0_converted = $crate::convert::TryConvert::try_convert(arg0_value)?;
                $crate::pin_on_stack!(arg0_pinned = arg0_converted);

                let arg1_value = unsafe { $crate::Value::from_raw(arg1) };
                let arg1_converted = $crate::convert::TryConvert::try_convert(arg1_value)?;
                $crate::pin_on_stack!(arg1_pinned = arg1_converted);

                let result = $func(self_converted, arg0_pinned, arg1_pinned);

                use $crate::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => $crate::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(wrapper as usize) }
    }};

    // Arity 3 - self + 3 arguments
    ($func:path, 3) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn wrapper(
            rb_self: $crate::rb_sys::VALUE,
            arg0: $crate::rb_sys::VALUE,
            arg1: $crate::rb_sys::VALUE,
            arg2: $crate::rb_sys::VALUE,
        ) -> $crate::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                let self_value = unsafe { $crate::Value::from_raw(rb_self) };
                let self_converted = $crate::convert::TryConvert::try_convert(self_value)?;

                let arg0_value = unsafe { $crate::Value::from_raw(arg0) };
                let arg0_converted = $crate::convert::TryConvert::try_convert(arg0_value)?;
                $crate::pin_on_stack!(arg0_pinned = arg0_converted);

                let arg1_value = unsafe { $crate::Value::from_raw(arg1) };
                let arg1_converted = $crate::convert::TryConvert::try_convert(arg1_value)?;
                $crate::pin_on_stack!(arg1_pinned = arg1_converted);

                let arg2_value = unsafe { $crate::Value::from_raw(arg2) };
                let arg2_converted = $crate::convert::TryConvert::try_convert(arg2_value)?;
                $crate::pin_on_stack!(arg2_pinned = arg2_converted);

                let result = $func(self_converted, arg0_pinned, arg1_pinned, arg2_pinned);

                use $crate::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => $crate::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(wrapper as usize) }
    }};

    // Arity 4 - self + 4 arguments
    ($func:path, 4) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn wrapper(
            rb_self: $crate::rb_sys::VALUE,
            arg0: $crate::rb_sys::VALUE,
            arg1: $crate::rb_sys::VALUE,
            arg2: $crate::rb_sys::VALUE,
            arg3: $crate::rb_sys::VALUE,
        ) -> $crate::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                let self_value = unsafe { $crate::Value::from_raw(rb_self) };
                let self_converted = $crate::convert::TryConvert::try_convert(self_value)?;

                let arg0_value = unsafe { $crate::Value::from_raw(arg0) };
                let arg0_converted = $crate::convert::TryConvert::try_convert(arg0_value)?;
                $crate::pin_on_stack!(arg0_pinned = arg0_converted);

                let arg1_value = unsafe { $crate::Value::from_raw(arg1) };
                let arg1_converted = $crate::convert::TryConvert::try_convert(arg1_value)?;
                $crate::pin_on_stack!(arg1_pinned = arg1_converted);

                let arg2_value = unsafe { $crate::Value::from_raw(arg2) };
                let arg2_converted = $crate::convert::TryConvert::try_convert(arg2_value)?;
                $crate::pin_on_stack!(arg2_pinned = arg2_converted);

                let arg3_value = unsafe { $crate::Value::from_raw(arg3) };
                let arg3_converted = $crate::convert::TryConvert::try_convert(arg3_value)?;
                $crate::pin_on_stack!(arg3_pinned = arg3_converted);

                let result = $func(
                    self_converted,
                    arg0_pinned,
                    arg1_pinned,
                    arg2_pinned,
                    arg3_pinned,
                );

                use $crate::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => $crate::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(wrapper as usize) }
    }};

    // Arities 5-15: Follow the same pattern as 0-4
    // These can be added by duplicating the arity 4 pattern and adding more arguments
    // For now, we provide a helpful error message
    ($func:path, $arity:literal) => {
        compile_error!(concat!(
            "method! arity ",
            stringify!($arity),
            " not yet implemented. ",
            "Currently supported arities: 0-4. ",
            "To add arity ",
            stringify!($arity),
            ", extend the method! macro in ",
            "crates/solidus/src/method/mod.rs following the pattern used for arities 0-4."
        ))
    };
}

/// Generate an extern "C" wrapper for a Ruby function.
///
/// This macro is similar to `method!` but for module/global functions that don't
/// have a `self` parameter. It creates a wrapper function that can be passed to
/// Ruby's function registration APIs (like `rb_define_global_function` and
/// `rb_define_module_function`).
///
/// The wrapper handles:
///
/// - Panic catching via `std::panic::catch_unwind`
/// - Type conversion of arguments via `TryConvert`
/// - Stack pinning of heap-allocated arguments
/// - Error propagation (converts `Err` to Ruby exceptions)
/// - Return value conversion via `IntoValue`
///
/// # Arity
///
/// The macro requires specifying the arity (number of arguments).
/// Use `function!(function_name, arity)` where arity is 0-4.
///
/// # Example
///
/// ```ignore
/// use solidus::prelude::*;
///
/// // Arity 0 - no arguments
/// fn greet() -> Result<RString, Error> {
///     Ok(RString::new("Hello, World!"))
/// }
///
/// // Arity 1 - one argument
/// fn greet_name(name: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
///     // name is automatically pinned by the wrapper
///     Ok(RString::new(&format!("Hello, {}!", name.get().to_string()?)))
/// }
///
/// // Register with Ruby
/// // ruby.define_global_function("greet", function!(greet, 0), 0)?;
/// // ruby.define_global_function("greet_name", function!(greet_name, 1), 1)?;
/// ```
#[macro_export]
macro_rules! function {
    // Arity 0 - no arguments
    ($func:path, 0) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn wrapper() -> $crate::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                let result = $func();

                use $crate::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => $crate::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(wrapper as usize) }
    }};

    // Arity 1 - 1 argument
    ($func:path, 1) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn wrapper(arg0: $crate::rb_sys::VALUE) -> $crate::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                let arg0_value = unsafe { $crate::Value::from_raw(arg0) };
                let arg0_converted = $crate::convert::TryConvert::try_convert(arg0_value)?;
                $crate::pin_on_stack!(arg0_pinned = arg0_converted);

                let result = $func(arg0_pinned);

                use $crate::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => $crate::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(wrapper as usize) }
    }};

    // Arity 2 - 2 arguments
    ($func:path, 2) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn wrapper(
            arg0: $crate::rb_sys::VALUE,
            arg1: $crate::rb_sys::VALUE,
        ) -> $crate::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                let arg0_value = unsafe { $crate::Value::from_raw(arg0) };
                let arg0_converted = $crate::convert::TryConvert::try_convert(arg0_value)?;
                $crate::pin_on_stack!(arg0_pinned = arg0_converted);

                let arg1_value = unsafe { $crate::Value::from_raw(arg1) };
                let arg1_converted = $crate::convert::TryConvert::try_convert(arg1_value)?;
                $crate::pin_on_stack!(arg1_pinned = arg1_converted);

                let result = $func(arg0_pinned, arg1_pinned);

                use $crate::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => $crate::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(wrapper as usize) }
    }};

    // Arity 3 - 3 arguments
    ($func:path, 3) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn wrapper(
            arg0: $crate::rb_sys::VALUE,
            arg1: $crate::rb_sys::VALUE,
            arg2: $crate::rb_sys::VALUE,
        ) -> $crate::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                let arg0_value = unsafe { $crate::Value::from_raw(arg0) };
                let arg0_converted = $crate::convert::TryConvert::try_convert(arg0_value)?;
                $crate::pin_on_stack!(arg0_pinned = arg0_converted);

                let arg1_value = unsafe { $crate::Value::from_raw(arg1) };
                let arg1_converted = $crate::convert::TryConvert::try_convert(arg1_value)?;
                $crate::pin_on_stack!(arg1_pinned = arg1_converted);

                let arg2_value = unsafe { $crate::Value::from_raw(arg2) };
                let arg2_converted = $crate::convert::TryConvert::try_convert(arg2_value)?;
                $crate::pin_on_stack!(arg2_pinned = arg2_converted);

                let result = $func(arg0_pinned, arg1_pinned, arg2_pinned);

                use $crate::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => $crate::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(wrapper as usize) }
    }};

    // Arity 4 - 4 arguments
    ($func:path, 4) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn wrapper(
            arg0: $crate::rb_sys::VALUE,
            arg1: $crate::rb_sys::VALUE,
            arg2: $crate::rb_sys::VALUE,
            arg3: $crate::rb_sys::VALUE,
        ) -> $crate::rb_sys::VALUE {
            let result = ::std::panic::catch_unwind(|| {
                let arg0_value = unsafe { $crate::Value::from_raw(arg0) };
                let arg0_converted = $crate::convert::TryConvert::try_convert(arg0_value)?;
                $crate::pin_on_stack!(arg0_pinned = arg0_converted);

                let arg1_value = unsafe { $crate::Value::from_raw(arg1) };
                let arg1_converted = $crate::convert::TryConvert::try_convert(arg1_value)?;
                $crate::pin_on_stack!(arg1_pinned = arg1_converted);

                let arg2_value = unsafe { $crate::Value::from_raw(arg2) };
                let arg2_converted = $crate::convert::TryConvert::try_convert(arg2_value)?;
                $crate::pin_on_stack!(arg2_pinned = arg2_converted);

                let arg3_value = unsafe { $crate::Value::from_raw(arg3) };
                let arg3_converted = $crate::convert::TryConvert::try_convert(arg3_value)?;
                $crate::pin_on_stack!(arg3_pinned = arg3_converted);

                let result = $func(arg0_pinned, arg1_pinned, arg2_pinned, arg3_pinned);

                use $crate::method::ReturnValue;
                result.into_return_value()
            });

            match result {
                Ok(Ok(value)) => value.as_raw(),
                Ok(Err(error)) => error.raise(),
                Err(panic) => $crate::Error::from_panic(panic).raise(),
            }
        }

        unsafe { ::std::mem::transmute(wrapper as usize) }
    }};

    // Arities 5-15: Follow the same pattern as 0-4
    // These can be added by duplicating the arity 4 pattern and adding more arguments
    // For now, we provide a helpful error message
    ($func:path, $arity:literal) => {
        compile_error!(concat!(
            "function! arity ",
            stringify!($arity),
            " not yet implemented. ",
            "Currently supported arities: 0-4. ",
            "To add arity ",
            stringify!($arity),
            ", extend the function! macro in ",
            "crates/solidus/src/method/mod.rs following the pattern used for arities 0-4."
        ))
    };
}

#[cfg(test)]
mod tests {
    use crate::convert::{IntoValue, TryConvert};
    use crate::error::Error;
    use crate::value::{ReprValue, StackPinned, Value};
    use std::pin::Pin;

    // Helper type that needs pinning
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct TestType(i64);

    impl ReprValue for TestType {
        fn as_value(self) -> Value {
            unsafe { Value::from_raw(self.0 as rb_sys::VALUE) }
        }

        unsafe fn from_value_unchecked(val: Value) -> Self {
            TestType(val.as_raw() as i64)
        }
    }

    impl TryConvert for TestType {
        fn try_convert(val: Value) -> Result<Self, Error> {
            Ok(TestType(val.as_raw() as i64))
        }
    }

    impl IntoValue for TestType {
        fn into_value(self) -> Value {
            unsafe { Value::from_raw(self.0 as rb_sys::VALUE) }
        }
    }

    // Test method with arity 0
    fn test_arity_0(_self: TestType) -> Result<i64, Error> {
        Ok(42)
    }

    // Test method with arity 1
    fn test_arity_1(_self: TestType, arg: Pin<&StackPinned<TestType>>) -> Result<i64, Error> {
        Ok(arg.get().0)
    }

    // Test method with arity 2
    fn test_arity_2(
        _self: TestType,
        arg0: Pin<&StackPinned<TestType>>,
        arg1: Pin<&StackPinned<TestType>>,
    ) -> Result<i64, Error> {
        Ok(arg0.get().0 + arg1.get().0)
    }

    // Test method with arity 3
    fn test_arity_3(
        _self: TestType,
        arg0: Pin<&StackPinned<TestType>>,
        arg1: Pin<&StackPinned<TestType>>,
        arg2: Pin<&StackPinned<TestType>>,
    ) -> Result<i64, Error> {
        Ok(arg0.get().0 + arg1.get().0 + arg2.get().0)
    }

    // Test method with arity 4
    fn test_arity_4(
        _self: TestType,
        arg0: Pin<&StackPinned<TestType>>,
        arg1: Pin<&StackPinned<TestType>>,
        arg2: Pin<&StackPinned<TestType>>,
        arg3: Pin<&StackPinned<TestType>>,
    ) -> Result<i64, Error> {
        Ok(arg0.get().0 + arg1.get().0 + arg2.get().0 + arg3.get().0)
    }

    #[test]
    fn test_method_macro_arity_0_compiles() {
        // This test just verifies that the macro expands without errors
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = method!(test_arity_0, 0);
    }

    #[test]
    fn test_method_macro_arity_1_compiles() {
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = method!(test_arity_1, 1);
    }

    #[test]
    fn test_method_macro_arity_2_compiles() {
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = method!(test_arity_2, 2);
    }

    #[test]
    fn test_method_macro_arity_3_compiles() {
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = method!(test_arity_3, 3);
    }

    #[test]
    fn test_method_macro_arity_4_compiles() {
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = method!(test_arity_4, 4);
    }

    // Test that the macro generates the correct function pointer type
    #[test]
    fn test_method_returns_function_pointer() {
        let wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = method!(test_arity_0, 0);
        // Verify it's the right type
        let _: unsafe extern "C" fn() -> rb_sys::VALUE = wrapper;
    }

    // ===============================================
    // Tests for function! macro
    // ===============================================

    // Test function with arity 0
    fn test_fn_arity_0() -> Result<i64, Error> {
        Ok(42)
    }

    // Test function with arity 1
    fn test_fn_arity_1(arg: Pin<&StackPinned<TestType>>) -> Result<i64, Error> {
        Ok(arg.get().0)
    }

    // Test function with arity 2
    fn test_fn_arity_2(
        arg0: Pin<&StackPinned<TestType>>,
        arg1: Pin<&StackPinned<TestType>>,
    ) -> Result<i64, Error> {
        Ok(arg0.get().0 + arg1.get().0)
    }

    // Test function with arity 3
    fn test_fn_arity_3(
        arg0: Pin<&StackPinned<TestType>>,
        arg1: Pin<&StackPinned<TestType>>,
        arg2: Pin<&StackPinned<TestType>>,
    ) -> Result<i64, Error> {
        Ok(arg0.get().0 + arg1.get().0 + arg2.get().0)
    }

    // Test function with arity 4
    fn test_fn_arity_4(
        arg0: Pin<&StackPinned<TestType>>,
        arg1: Pin<&StackPinned<TestType>>,
        arg2: Pin<&StackPinned<TestType>>,
        arg3: Pin<&StackPinned<TestType>>,
    ) -> Result<i64, Error> {
        Ok(arg0.get().0 + arg1.get().0 + arg2.get().0 + arg3.get().0)
    }

    #[test]
    fn test_function_macro_arity_0_compiles() {
        // This test just verifies that the macro expands without errors
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = function!(test_fn_arity_0, 0);
    }

    #[test]
    fn test_function_macro_arity_1_compiles() {
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = function!(test_fn_arity_1, 1);
    }

    #[test]
    fn test_function_macro_arity_2_compiles() {
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = function!(test_fn_arity_2, 2);
    }

    #[test]
    fn test_function_macro_arity_3_compiles() {
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = function!(test_fn_arity_3, 3);
    }

    #[test]
    fn test_function_macro_arity_4_compiles() {
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = function!(test_fn_arity_4, 4);
    }

    // Test that the function macro generates the correct function pointer type
    #[test]
    fn test_function_returns_function_pointer() {
        let wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = function!(test_fn_arity_0, 0);
        // Verify it's the right type
        let _: unsafe extern "C" fn() -> rb_sys::VALUE = wrapper;
    }

    // Test that function! works with different return types
    fn test_fn_returns_unit() -> Result<(), Error> {
        Ok(())
    }

    fn test_fn_returns_value(_arg: Pin<&StackPinned<TestType>>) -> Result<TestType, Error> {
        Ok(TestType(100))
    }

    #[test]
    fn test_function_with_unit_return() {
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = function!(test_fn_returns_unit, 0);
    }

    #[test]
    fn test_function_with_value_return() {
        let _wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = function!(test_fn_returns_value, 1);
    }

    // Test demonstrating difference between method! and function!
    // method! has self parameter, function! does not
    #[test]
    fn test_method_vs_function_signatures() {
        // Method: rb_self + 1 arg = 2 parameters total for the Rust function
        fn method_example(
            _rb_self: TestType,
            _arg: Pin<&StackPinned<TestType>>,
        ) -> Result<i64, Error> {
            Ok(0)
        }
        let _method_wrapper: unsafe extern "C" fn() -> rb_sys::VALUE = method!(method_example, 1);

        // Function: no rb_self, just 1 arg = 1 parameter total for the Rust function
        fn function_example(_arg: Pin<&StackPinned<TestType>>) -> Result<i64, Error> {
            Ok(0)
        }
        let _function_wrapper: unsafe extern "C" fn() -> rb_sys::VALUE =
            function!(function_example, 1);

        // Both compile successfully, demonstrating the key difference:
        // - method! expects rb_self as first parameter
        // - function! has no rb_self parameter
    }
}
