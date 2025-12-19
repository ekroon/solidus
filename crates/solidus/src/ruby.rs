//! Ruby VM handle and entry point.

use std::cell::Cell;

use crate::value::Value;

// Thread-local marker that we use to check if we're on the Ruby thread
thread_local! {
    static IS_RUBY_THREAD: Cell<bool> = const { Cell::new(false) };
}

/// Handle to the Ruby VM.
///
/// This type cannot be created directly - it's provided by the `#[solidus::init]`
/// macro or obtained via [`Ruby::get()`] when Ruby is known to be initialized.
///
/// `Ruby` provides access to Ruby's built-in classes, constants, and module
/// definition APIs. It acts as proof that Ruby is initialized.
///
/// # Thread Safety
///
/// Ruby's C API is not thread-safe. The `Ruby` handle can only be used from
/// the thread where Ruby was initialized (typically the main thread).
///
/// # Example
///
/// ```no_run
/// use solidus::prelude::*;
///
/// #[solidus::init]
/// fn init(ruby: &Ruby) -> Result<(), Error> {
///     let object_class = ruby.class_object();
///     // ...
///     Ok(())
/// }
/// ```
pub struct Ruby {
    // Private field to prevent construction outside this module
    _private: (),
}

impl Ruby {
    /// Get a reference to Ruby.
    ///
    /// This provides access to Ruby's API. It can only be called when Ruby
    /// is known to be initialized.
    ///
    /// # Safety
    ///
    /// - Ruby must be initialized (i.e., `ruby_setup()` has been called).
    /// - Must be called from the Ruby main thread.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::Ruby;
    ///
    /// // In an extern "C" function called from Ruby:
    /// let ruby = unsafe { Ruby::get() };
    /// let nil = ruby.qnil();
    /// ```
    #[inline]
    pub unsafe fn get() -> &'static Self {
        // SAFETY: Ruby is a ZST wrapper, we just need a reference.
        // The static is used to give a stable address.
        &RUBY_INSTANCE
    }

    /// Mark the current thread as the Ruby thread.
    ///
    /// This should be called during initialization.
    ///
    /// # Safety
    ///
    /// This must only be called from the actual Ruby main thread.
    #[doc(hidden)]
    pub unsafe fn mark_ruby_thread() {
        IS_RUBY_THREAD.with(|cell| cell.set(true));
    }

    // =========================================================================
    // Constants
    // =========================================================================

    /// Get the nil value.
    #[inline]
    pub fn qnil(&self) -> Value {
        Value::nil()
    }

    /// Get the true value.
    #[inline]
    pub fn qtrue(&self) -> Value {
        Value::r#true()
    }

    /// Get the false value.
    #[inline]
    pub fn qfalse(&self) -> Value {
        Value::r#false()
    }

    // =========================================================================
    // Class accessors
    // =========================================================================

    /// Get the Object class.
    #[inline]
    pub fn class_object(&self) -> Value {
        // SAFETY: rb_cObject is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cObject) }
    }

    /// Get the Class class.
    #[inline]
    pub fn class_class(&self) -> Value {
        // SAFETY: rb_cClass is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cClass) }
    }

    /// Get the Module class.
    #[inline]
    pub fn class_module(&self) -> Value {
        // SAFETY: rb_cModule is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cModule) }
    }

    /// Get the String class.
    #[inline]
    pub fn class_string(&self) -> Value {
        // SAFETY: rb_cString is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cString) }
    }

    /// Get the Array class.
    #[inline]
    pub fn class_array(&self) -> Value {
        // SAFETY: rb_cArray is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cArray) }
    }

    /// Get the Hash class.
    #[inline]
    pub fn class_hash(&self) -> Value {
        // SAFETY: rb_cHash is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cHash) }
    }

    /// Get the Integer class.
    #[inline]
    pub fn class_integer(&self) -> Value {
        // SAFETY: rb_cInteger is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cInteger) }
    }

    /// Get the Float class.
    #[inline]
    pub fn class_float(&self) -> Value {
        // SAFETY: rb_cFloat is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cFloat) }
    }

    /// Get the Symbol class.
    #[inline]
    pub fn class_symbol(&self) -> Value {
        // SAFETY: rb_cSymbol is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cSymbol) }
    }

    /// Get the TrueClass class.
    #[inline]
    pub fn class_true(&self) -> Value {
        // SAFETY: rb_cTrueClass is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cTrueClass) }
    }

    /// Get the FalseClass class.
    #[inline]
    pub fn class_false(&self) -> Value {
        // SAFETY: rb_cFalseClass is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cFalseClass) }
    }

    /// Get the NilClass class.
    #[inline]
    pub fn class_nil(&self) -> Value {
        // SAFETY: rb_cNilClass is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_cNilClass) }
    }

    // =========================================================================
    // Exception classes
    // =========================================================================

    /// Get the StandardError exception class.
    #[inline]
    pub fn exception_standard_error(&self) -> Value {
        // SAFETY: rb_eStandardError is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_eStandardError) }
    }

    /// Get the RuntimeError exception class.
    #[inline]
    pub fn exception_runtime_error(&self) -> Value {
        // SAFETY: rb_eRuntimeError is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_eRuntimeError) }
    }

    /// Get the TypeError exception class.
    #[inline]
    pub fn exception_type_error(&self) -> Value {
        // SAFETY: rb_eTypeError is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_eTypeError) }
    }

    /// Get the ArgumentError exception class.
    #[inline]
    pub fn exception_argument_error(&self) -> Value {
        // SAFETY: rb_eArgError is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_eArgError) }
    }

    /// Get the NoMemoryError exception class.
    #[inline]
    pub fn exception_no_memory_error(&self) -> Value {
        // SAFETY: rb_eNoMemError is always valid after Ruby init
        unsafe { Value::from_raw(rb_sys::rb_eNoMemError) }
    }

    // =========================================================================
    // Module/Class definition
    // =========================================================================

    /// Define a new top-level class.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the class (must be a valid Ruby constant name)
    /// * `superclass` - The superclass (use `ruby.class_object()` for Object)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::Ruby;
    ///
    /// let ruby = unsafe { Ruby::get() };
    /// let my_class = ruby.define_class("MyClass", ruby.class_object());
    /// ```
    pub fn define_class(&self, name: &str, superclass: Value) -> Value {
        let c_name = std::ffi::CString::new(name).expect("class name contains null byte");
        // SAFETY: We ensure the name is a valid C string
        unsafe {
            Value::from_raw(rb_sys::rb_define_class(
                c_name.as_ptr(),
                superclass.as_raw(),
            ))
        }
    }

    /// Define a new top-level module.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module (must be a valid Ruby constant name)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::Ruby;
    ///
    /// let ruby = unsafe { Ruby::get() };
    /// let my_module = ruby.define_module("MyModule");
    /// ```
    pub fn define_module(&self, name: &str) -> Value {
        let c_name = std::ffi::CString::new(name).expect("module name contains null byte");
        // SAFETY: We ensure the name is a valid C string
        unsafe { Value::from_raw(rb_sys::rb_define_module(c_name.as_ptr())) }
    }

    /// Define a class under another module/class.
    ///
    /// # Arguments
    ///
    /// * `outer` - The containing module or class
    /// * `name` - The name of the class
    /// * `superclass` - The superclass
    pub fn define_class_under(&self, outer: Value, name: &str, superclass: Value) -> Value {
        let c_name = std::ffi::CString::new(name).expect("class name contains null byte");
        // SAFETY: We ensure the name is a valid C string
        unsafe {
            Value::from_raw(rb_sys::rb_define_class_under(
                outer.as_raw(),
                c_name.as_ptr(),
                superclass.as_raw(),
            ))
        }
    }

    /// Define a module under another module/class.
    ///
    /// # Arguments
    ///
    /// * `outer` - The containing module or class
    /// * `name` - The name of the module
    pub fn define_module_under(&self, outer: Value, name: &str) -> Value {
        let c_name = std::ffi::CString::new(name).expect("module name contains null byte");
        // SAFETY: We ensure the name is a valid C string
        unsafe {
            Value::from_raw(rb_sys::rb_define_module_under(
                outer.as_raw(),
                c_name.as_ptr(),
            ))
        }
    }

    /// Define a global function.
    ///
    /// Global functions are available everywhere in Ruby without needing to qualify
    /// them with a receiver. They become private methods of `Kernel` and are mixed
    /// into all objects.
    ///
    /// # Arguments
    ///
    /// * `name` - The function name
    /// * `func` - A function pointer generated by the `function!` macro
    /// * `arity` - The number of arguments (-1 for variadic, 0-15 for fixed)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::{function, Ruby, Error};
    /// use solidus::Context;
    /// use solidus::value::StackPinned;
    /// use solidus::types::RString;
    /// use std::pin::Pin;
    ///
    /// fn greet<'a>(ctx: &'a Context) -> Result<Pin<&'a StackPinned<RString>>, Error> {
    ///     Ok(ctx.new_string("Hello, World!")?)
    /// }
    ///
    /// let ruby = unsafe { Ruby::get() };
    /// ruby.define_global_function("greet", function!(greet, 0), 0).unwrap();
    /// // Now `greet` can be called from Ruby without qualification
    /// ```
    pub fn define_global_function(
        &self,
        name: &str,
        func: unsafe extern "C" fn() -> rb_sys::VALUE,
        arity: i32,
    ) -> Result<(), crate::error::Error> {
        use crate::error::Error;

        // Convert name to C string
        let c_name = std::ffi::CString::new(name)
            .map_err(|_| Error::argument("function name contains null byte"))?;

        // SAFETY: c_name is a valid C string
        // rb_define_global_function registers the function pointer with Ruby's Kernel module
        unsafe {
            rb_sys::rb_define_global_function(c_name.as_ptr(), Some(func), arity);
        }

        Ok(())
    }
}

// Static instance used by Ruby::get()
static RUBY_INSTANCE: Ruby = Ruby { _private: () };

#[cfg(test)]
mod tests {
    use super::*;

    // Ruby handle tests that don't require Ruby initialization.

    #[test]
    fn test_ruby_is_zst() {
        // Ruby should be a zero-sized type
        assert_eq!(std::mem::size_of::<Ruby>(), 0);
    }

    #[test]
    fn test_ruby_get_returns_static_ref() {
        // SAFETY: We're only checking that get() works, not calling Ruby APIs
        let ruby1 = unsafe { Ruby::get() };
        let ruby2 = unsafe { Ruby::get() };
        // Both should be the same static reference
        assert!(std::ptr::eq(ruby1, ruby2));
    }
}

#[cfg(all(test, any(feature = "embed", feature = "link-ruby")))]
mod ruby_tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;

    use crate::error::Error;
    use crate::function;
    use crate::types::RString;
    use crate::value::StackPinned;
    use std::pin::Pin;

    // Test functions for define_global_function
    fn test_global_func_arity_0() -> Result<i64, Error> {
        Ok(42)
    }

    fn test_global_func_arity_1(_arg: Pin<&StackPinned<RString>>) -> Result<i64, Error> {
        Ok(100)
    }

    fn test_global_func_arity_2(
        _arg0: Pin<&StackPinned<RString>>,
        _arg1: Pin<&StackPinned<RString>>,
    ) -> Result<i64, Error> {
        Ok(200)
    }

    #[ruby_test]
    fn test_define_global_function_arity_0() {
        let ruby = unsafe { Ruby::get() };

        // Define a global function - if this doesn't crash, it worked
        ruby.define_global_function(
            "solidus_test_global_0",
            function!(test_global_func_arity_0, 0),
            0,
        )
        .unwrap();
    }

    #[ruby_test]
    fn test_define_global_function_arity_1() {
        let ruby = unsafe { Ruby::get() };

        // Define a global function with 1 argument - if this doesn't crash, it worked
        ruby.define_global_function(
            "solidus_test_global_1",
            function!(test_global_func_arity_1, 1),
            1,
        )
        .unwrap();
    }

    #[ruby_test]
    fn test_define_global_function_arity_2() {
        let ruby = unsafe { Ruby::get() };

        // Define a global function with 2 arguments - if this doesn't crash, it worked
        ruby.define_global_function(
            "solidus_test_global_2",
            function!(test_global_func_arity_2, 2),
            2,
        )
        .unwrap();
    }

    #[ruby_test]
    fn test_define_global_function_error_on_null_byte() {
        let ruby = unsafe { Ruby::get() };

        let result =
            ruby.define_global_function("test\0func", function!(test_global_func_arity_0, 0), 0);

        assert!(result.is_err());
    }
}
