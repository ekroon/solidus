//! Ruby Module type.

use crate::convert::{IntoValue, TryConvert};
use crate::error::{Error, ExceptionClass};
use crate::types::RClass;
use crate::value::{ReprValue, Value};

/// Ruby Module.
///
/// Ruby modules are containers for methods and constants. They can be included
/// in classes (mixins) or used as namespaces.
///
/// # Example
///
/// ```ignore
/// use solidus::types::RModule;
///
/// // Get the Enumerable module
/// let enumerable = RModule::from_name("Enumerable").unwrap();
/// assert_eq!(enumerable.name().unwrap(), "Enumerable");
/// ```
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct RModule(Value);

impl RModule {
    /// Get a module by name.
    ///
    /// Returns `None` if the module doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RModule;
    ///
    /// let enumerable = RModule::from_name("Enumerable").unwrap();
    /// assert_eq!(enumerable.name().unwrap(), "Enumerable");
    ///
    /// let missing = RModule::from_name("NonExistentModule");
    /// assert!(missing.is_none());
    /// ```
    pub fn from_name(name: &str) -> Option<Self> {
        // Convert to C string
        let c_name = std::ffi::CString::new(name).ok()?;

        // SAFETY: rb_path2class looks up a class/module by name and may raise an exception
        // If it raises, we catch it by checking rb_errinfo()
        let val = unsafe {
            // Save the current exception state
            let old_errinfo = rb_sys::rb_errinfo();

            // Try to get the module
            let result = rb_sys::rb_path2class(c_name.as_ptr());

            // Check if an exception was raised
            let new_errinfo = rb_sys::rb_errinfo();
            if new_errinfo != old_errinfo {
                // Exception was raised, clear it and return None
                rb_sys::rb_set_errinfo(rb_sys::Qnil.into());
                return None;
            }

            result
        };

        if val == rb_sys::Qnil.into() {
            None
        } else {
            // Verify it's a module, not a class
            let value = unsafe { Value::from_raw(val) };
            if value.rb_type() == crate::value::ValueType::Module {
                Some(RModule(value))
            } else {
                None
            }
        }
    }

    /// Get the name of this module.
    ///
    /// Returns `None` for anonymous modules.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RModule;
    ///
    /// let enumerable = RModule::from_name("Enumerable").unwrap();
    /// assert_eq!(enumerable.name().unwrap(), "Enumerable");
    /// ```
    pub fn name(&self) -> Option<String> {
        // SAFETY: self.0 is a valid Ruby module VALUE
        // rb_mod_name works for both classes and modules
        let val = unsafe { rb_sys::rb_mod_name(self.0.as_raw()) };

        // Check if it's nil (anonymous module)
        if val == rb_sys::Qnil.into() {
            return None;
        }

        // SAFETY: rb_mod_name returns a Ruby string or nil
        // We immediately copy the string, so it doesn't outlive the VALUE
        unsafe {
            let ptr = rb_sys::RSTRING_PTR(val);
            let len = rb_sys::RSTRING_LEN(val) as usize;
            let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
            String::from_utf8(bytes.to_vec()).ok()
        }
    }
}

impl ReprValue for RModule {
    #[inline]
    fn as_value(&self) -> Value {
        self.0.clone()
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        RModule(val)
    }
}

impl TryConvert for RModule {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if val.rb_type() == crate::value::ValueType::Module {
            // SAFETY: We've verified it's a Module
            Ok(unsafe { RModule::from_value_unchecked(val) })
        } else {
            Err(Error::type_error("expected Module"))
        }
    }
}

impl IntoValue for RModule {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

/// Trait for types that can define constants (both Class and Module).
///
/// Ruby classes and modules share common behavior for defining constants
/// and retrieving them. This trait provides that shared interface.
///
/// # Example
///
/// ```ignore
/// use solidus::types::{RClass, Module};
///
/// let string_class = RClass::from_name("String").unwrap();
/// string_class.define_const("MY_CONST", 42i64).unwrap();
/// let val = string_class.const_get("MY_CONST").unwrap();
/// ```
pub trait Module: ReprValue {
    /// Define a constant in this module/class.
    ///
    /// This sets a constant that can be accessed as `ModuleName::CONST_NAME`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::{RClass, Module};
    ///
    /// let string_class = RClass::from_name("String").unwrap();
    /// string_class.define_const("VERSION", "1.0.0").unwrap();
    /// ```
    fn define_const<T: IntoValue>(&self, name: &str, value: T) -> Result<(), Error> {
        // Convert name to C string
        let c_name = std::ffi::CString::new(name)
            .map_err(|_| Error::argument("constant name contains null byte"))?;

        let val = value.into_value();

        // SAFETY: self is a valid module/class, c_name is a valid C string, val is a valid VALUE
        unsafe {
            rb_sys::rb_const_set(
                self.as_value().as_raw(),
                rb_sys::rb_intern(c_name.as_ptr()),
                val.as_raw(),
            );
        }

        Ok(())
    }

    /// Get a constant from this module/class.
    ///
    /// Returns an error if the constant doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::{RClass, Module};
    ///
    /// let file_class = RClass::from_name("File").unwrap();
    /// let separator = file_class.const_get("SEPARATOR").unwrap();
    /// ```
    fn const_get(&self, name: &str) -> Result<Value, Error> {
        // Convert name to C string
        let c_name = std::ffi::CString::new(name)
            .map_err(|_| Error::argument("constant name contains null byte"))?;

        // Get the symbol ID for the constant name
        let id = unsafe { rb_sys::rb_intern(c_name.as_ptr()) };

        // Check if the constant is defined first
        let defined = unsafe { rb_sys::rb_const_defined(self.as_value().as_raw(), id) != 0 };

        if !defined {
            return Err(Error::new(
                ExceptionClass::NameError,
                format!("uninitialized constant {}", name),
            ));
        }

        // SAFETY: We've verified the constant exists, so rb_const_get should succeed
        let val = unsafe { rb_sys::rb_const_get(self.as_value().as_raw(), id) };

        Ok(unsafe { Value::from_raw(val) })
    }

    /// Define an instance method on this class/module.
    ///
    /// This registers a Rust function as a Ruby method that can be called on instances
    /// of this class or objects that include this module.
    ///
    /// # Arguments
    ///
    /// * `name` - The method name (Ruby-style method names like "foo" or "foo_bar")
    /// * `func` - A function pointer generated by the `method!` macro
    /// * `arity` - The number of arguments (-1 for variadic, 0-15 for fixed)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::{method, Ruby, Module};
    /// use solidus::types::RClass;
    ///
    /// fn my_method(rb_self: RString) -> Result<RString, Error> {
    ///     Ok(rb_self)
    /// }
    ///
    /// let ruby = unsafe { Ruby::get() };
    /// let class = ruby.define_class("MyClass", ruby.class_object());
    /// let rclass = RClass::try_convert(class)?;
    /// rclass.define_method("my_method", method!(my_method, 0), 0)?;
    /// ```
    fn define_method(
        self,
        name: &str,
        func: unsafe extern "C" fn() -> rb_sys::VALUE,
        arity: i32,
    ) -> Result<(), Error> {
        // Convert name to C string
        let c_name = std::ffi::CString::new(name)
            .map_err(|_| Error::argument("method name contains null byte"))?;

        // SAFETY: self is a valid module/class, c_name is a valid C string
        // rb_define_method registers the function pointer with Ruby
        // The function pointer must remain valid for the lifetime of the Ruby VM
        unsafe {
            rb_sys::rb_define_method(self.as_value().as_raw(), c_name.as_ptr(), Some(func), arity);
        }

        Ok(())
    }

    /// Define a singleton method on this class/module.
    ///
    /// Singleton methods are also known as "class methods" when defined on a class.
    /// They can be called directly on the class/module rather than on instances.
    ///
    /// # Arguments
    ///
    /// * `name` - The method name
    /// * `func` - A function pointer generated by the `method!` or `function!` macro
    /// * `arity` - The number of arguments (-1 for variadic, 0-15 for fixed)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::{function, Ruby, Module};
    /// use solidus::types::RClass;
    ///
    /// fn class_method() -> Result<i64, Error> {
    ///     Ok(42)
    /// }
    ///
    /// let ruby = unsafe { Ruby::get() };
    /// let class = ruby.define_class("MyClass", ruby.class_object());
    /// let rclass = RClass::try_convert(class)?;
    /// rclass.define_singleton_method("class_method", function!(class_method, 0), 0)?;
    /// ```
    fn define_singleton_method(
        self,
        name: &str,
        func: unsafe extern "C" fn() -> rb_sys::VALUE,
        arity: i32,
    ) -> Result<(), Error> {
        // Convert name to C string
        let c_name = std::ffi::CString::new(name)
            .map_err(|_| Error::argument("method name contains null byte"))?;

        // SAFETY: self is a valid module/class, c_name is a valid C string
        // rb_define_singleton_method registers the function pointer with Ruby
        unsafe {
            rb_sys::rb_define_singleton_method(
                self.as_value().as_raw(),
                c_name.as_ptr(),
                Some(func),
                arity,
            );
        }

        Ok(())
    }

    /// Define a module function.
    ///
    /// Module functions are callable as both `Module.func` (singleton method) and
    /// `Module::func` when the module is included. This is equivalent to defining
    /// both an instance method and a module method.
    ///
    /// # Arguments
    ///
    /// * `name` - The function name
    /// * `func` - A function pointer generated by the `method!` or `function!` macro
    /// * `arity` - The number of arguments (-1 for variadic, 0-15 for fixed)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::{function, Ruby, Module};
    /// use solidus::types::RModule;
    ///
    /// fn my_function() -> Result<i64, Error> {
    ///     Ok(42)
    /// }
    ///
    /// let ruby = unsafe { Ruby::get() };
    /// let module = ruby.define_module("MyModule");
    /// let rmodule = RModule::try_convert(module)?;
    /// rmodule.define_module_function("my_function", function!(my_function, 0), 0)?;
    /// ```
    fn define_module_function(
        self,
        name: &str,
        func: unsafe extern "C" fn() -> rb_sys::VALUE,
        arity: i32,
    ) -> Result<(), Error> {
        // Convert name to C string
        let c_name = std::ffi::CString::new(name)
            .map_err(|_| Error::argument("method name contains null byte"))?;

        // SAFETY: self is a valid module, c_name is a valid C string
        // rb_define_module_function registers the function as both instance and singleton
        unsafe {
            rb_sys::rb_define_module_function(
                self.as_value().as_raw(),
                c_name.as_ptr(),
                Some(func),
                arity,
            );
        }

        Ok(())
    }
}

// Implement Module trait for RClass
impl Module for RClass {}

// Implement Module trait for RModule
impl Module for RModule {}

#[cfg(all(test, any(feature = "embed", feature = "link-ruby")))]
mod tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;

    #[ruby_test]
    fn test_rmodule_from_name() {
        let enumerable = RModule::from_name("Enumerable").unwrap();
        assert_eq!(enumerable.name().unwrap(), "Enumerable");
    }

    #[ruby_test]
    #[ignore] // TODO: Fix exception handling in from_name
    fn test_rmodule_from_name_missing() {
        let missing = RModule::from_name("NonExistentModule123");
        assert!(missing.is_none());
    }

    #[ruby_test]
    fn test_rmodule_from_name_rejects_class() {
        // String is a class, not a module, so it should return None
        let string = RModule::from_name("String");
        assert!(string.is_none());
    }

    #[ruby_test]
    fn test_rmodule_name() {
        let enumerable = RModule::from_name("Enumerable").unwrap();
        assert_eq!(enumerable.name().unwrap(), "Enumerable");

        let kernel = RModule::from_name("Kernel").unwrap();
        assert_eq!(kernel.name().unwrap(), "Kernel");

        let comparable = RModule::from_name("Comparable").unwrap();
        assert_eq!(comparable.name().unwrap(), "Comparable");
    }

    #[ruby_test]
    fn test_rmodule_try_convert() {
        let enumerable = RModule::from_name("Enumerable").unwrap();
        let val = enumerable.into_value();

        let converted = RModule::try_convert(val).unwrap();
        assert_eq!(converted.name().unwrap(), "Enumerable");
    }

    #[ruby_test]
    fn test_rmodule_try_convert_wrong_type() {
        let val = 42i64.into_value();
        assert!(RModule::try_convert(val).is_err());
    }

    #[ruby_test]
    fn test_rmodule_try_convert_rejects_class() {
        let string_class = RClass::from_name("String").unwrap();
        let val = string_class.into_value();

        // Should fail because it's a class, not a module
        assert!(RModule::try_convert(val).is_err());
    }

    #[ruby_test]
    fn test_module_trait_define_const_on_class() {
        let string_class = RClass::from_name("String").unwrap();

        // Define a constant
        string_class.define_const("TEST_CONST_1", 12345i64).unwrap();

        // Get it back
        let val = string_class.const_get("TEST_CONST_1").unwrap();
        assert_eq!(i64::try_convert(val).unwrap(), 12345);
    }

    #[ruby_test]
    fn test_module_trait_define_const_on_module() {
        let enumerable = RModule::from_name("Enumerable").unwrap();

        // Define a constant
        enumerable
            .define_const("TEST_CONST_2", "test value")
            .unwrap();

        // Get it back
        let val = enumerable.const_get("TEST_CONST_2").unwrap();
        let s = crate::types::RString::try_convert(val).unwrap();
        assert_eq!(s.to_string().unwrap(), "test value");
    }

    #[ruby_test]
    fn test_module_trait_const_get_missing() {
        let string_class = RClass::from_name("String").unwrap();

        let result = string_class.const_get("NONEXISTENT_CONST_XYZ");
        assert!(result.is_err());
    }

    #[ruby_test]
    fn test_module_trait_const_get_builtin() {
        let file_class = RClass::from_name("File").unwrap();

        // File::SEPARATOR is a built-in constant
        let separator = file_class.const_get("SEPARATOR").unwrap();
        let s = crate::types::RString::try_convert(separator).unwrap();
        // Should be "/" on Unix-like systems, "\" on Windows
        assert!(!s.to_string().unwrap().is_empty());
    }

    #[ruby_test]
    fn test_module_trait_define_const_overwrite() {
        let string_class = RClass::from_name("String").unwrap();

        // Define a constant
        string_class.define_const("TEST_OVERWRITE", 1i64).unwrap();

        // Overwrite it (Ruby will warn but allows it)
        string_class.define_const("TEST_OVERWRITE", 2i64).unwrap();

        // Get the new value
        let val = string_class.const_get("TEST_OVERWRITE").unwrap();
        assert_eq!(i64::try_convert(val).unwrap(), 2);
    }

    #[ruby_test]
    fn test_rmodule_multiple_builtin_modules() {
        let modules = vec!["Enumerable", "Kernel", "Comparable"];

        for module_name in modules {
            let module = RModule::from_name(module_name).unwrap();
            assert_eq!(module.name().unwrap(), module_name);
        }
    }

    // Tests for method definition API

    use crate::types::RString;
    use crate::{function, method};
    use std::pin::Pin;

    // Test method for define_method
    fn test_method_arity_0(rb_self: RString) -> Result<i64, Error> {
        let _ = rb_self;
        Ok(42)
    }

    fn test_method_arity_1(
        rb_self: RString,
        _arg: Pin<&crate::value::StackPinned<RString>>,
    ) -> Result<i64, Error> {
        let _ = rb_self;
        Ok(100)
    }

    // Test function for define_singleton_method and define_global_function
    fn test_function_arity_0() -> Result<i64, Error> {
        Ok(999)
    }

    fn test_function_arity_1(_arg: Pin<&crate::value::StackPinned<RString>>) -> Result<i64, Error> {
        Ok(777)
    }

    #[ruby_test]
    fn test_module_define_method() {
        use crate::Ruby;

        let ruby = unsafe { Ruby::get() };
        let class = ruby.define_class("TestDefineMethod", ruby.class_object());
        let rclass = RClass::try_convert(class).unwrap();

        // Define an instance method - if this doesn't crash, it worked
        rclass
            .define_method("test_method", method!(test_method_arity_0, 0), 0)
            .unwrap();

        // Success - method was registered without crashing
    }

    #[ruby_test]
    fn test_module_define_method_with_arg() {
        use crate::Ruby;

        let ruby = unsafe { Ruby::get() };
        let class = ruby.define_class("TestDefineMethodArg", ruby.class_object());
        let rclass = RClass::try_convert(class).unwrap();

        // Define an instance method with 1 argument - if this doesn't crash, it worked
        rclass
            .define_method("test_method_arg", method!(test_method_arity_1, 1), 1)
            .unwrap();
    }

    #[ruby_test]
    fn test_module_define_singleton_method() {
        use crate::Ruby;

        let ruby = unsafe { Ruby::get() };
        let class = ruby.define_class("TestDefineSingleton", ruby.class_object());
        let rclass = RClass::try_convert(class).unwrap();

        // Define a singleton method (class method) - if this doesn't crash, it worked
        rclass
            .define_singleton_method("test_class_method", function!(test_function_arity_0, 0), 0)
            .unwrap();
    }

    #[ruby_test]
    fn test_module_define_singleton_method_with_arg() {
        use crate::Ruby;

        let ruby = unsafe { Ruby::get() };
        let class = ruby.define_class("TestDefineSingletonArg", ruby.class_object());
        let rclass = RClass::try_convert(class).unwrap();

        // Define a singleton method with 1 argument - if this doesn't crash, it worked
        rclass
            .define_singleton_method(
                "test_class_method_arg",
                function!(test_function_arity_1, 1),
                1,
            )
            .unwrap();
    }

    #[ruby_test]
    fn test_module_define_module_function() {
        use crate::Ruby;

        let ruby = unsafe { Ruby::get() };
        let module = ruby.define_module("TestDefineModuleFunc");
        let rmodule = RModule::try_convert(module).unwrap();

        // Define a module function - if this doesn't crash, it worked
        rmodule
            .define_module_function("test_mod_func", function!(test_function_arity_0, 0), 0)
            .unwrap();
    }

    #[ruby_test]
    fn test_module_define_method_error_on_null_byte() {
        use crate::Ruby;

        let ruby = unsafe { Ruby::get() };
        let class = ruby.define_class("TestNullByte", ruby.class_object());
        let rclass = RClass::try_convert(class).unwrap();

        // Try to define a method with a null byte in the name
        let result = rclass.define_method("test\0method", method!(test_method_arity_0, 0), 0);

        assert!(result.is_err());
    }
}
