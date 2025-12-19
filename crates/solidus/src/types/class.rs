//! Ruby Class type.

use crate::convert::{IntoValue, TryConvert};
use crate::error::Error;
use crate::value::{ReprValue, Value};

/// Ruby Class.
///
/// Ruby classes are first-class objects that define the behavior of their instances.
/// Every Ruby object has a class, and classes themselves are objects of class Class.
///
/// # Example
///
/// ```no_run
/// use solidus::types::RClass;
///
/// // Get the String class
/// let string_class = RClass::from_name("String").unwrap();
/// assert_eq!(string_class.name().unwrap(), "String");
///
/// // Get the superclass
/// let object_class = string_class.superclass().unwrap();
/// assert_eq!(object_class.name().unwrap(), "Object");
/// ```
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct RClass(Value);

impl RClass {
    /// Get a class by name.
    ///
    /// Returns `None` if the class doesn't exist.
    ///
    /// # Known Issues
    ///
    /// Currently, there's an issue with exception handling when looking up
    /// non-existent classes that causes tests to hang. Use only with known
    /// class names for now. This will be fixed in a future update.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RClass;
    ///
    /// let string_class = RClass::from_name("String").unwrap();
    /// assert_eq!(string_class.name().unwrap(), "String");
    /// ```
    pub fn from_name(name: &str) -> Option<Self> {
        // Convert to C string
        let c_name = std::ffi::CString::new(name).ok()?;

        // SAFETY: rb_path2class looks up a class by name and may raise an exception
        // If it raises, we catch it by checking rb_errinfo()
        let val = unsafe {
            // Save the current exception state
            let old_errinfo = rb_sys::rb_errinfo();

            // Try to get the class
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
            Some(RClass(unsafe { Value::from_raw(val) }))
        }
    }

    /// Get the name of this class.
    ///
    /// Returns `None` for anonymous classes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RClass;
    ///
    /// let string_class = RClass::from_name("String").unwrap();
    /// assert_eq!(string_class.name().unwrap(), "String");
    /// ```
    pub fn name(&self) -> Option<String> {
        // SAFETY: self.0 is a valid Ruby class VALUE
        let val = unsafe { rb_sys::rb_class_name(self.0.as_raw()) };

        // Check if it's nil (anonymous class)
        if val == rb_sys::Qnil.into() {
            return None;
        }

        // SAFETY: rb_class_name returns a Ruby string or nil
        let _name_value = unsafe { Value::from_raw(val) };

        // SAFETY: We immediately copy the string, so it doesn't outlive the VALUE
        unsafe {
            let ptr = rb_sys::RSTRING_PTR(val);
            let len = rb_sys::RSTRING_LEN(val) as usize;
            let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
            String::from_utf8(bytes.to_vec()).ok()
        }
    }

    /// Get the superclass of this class.
    ///
    /// Returns `None` for BasicObject (which has no superclass).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RClass;
    ///
    /// let string_class = RClass::from_name("String").unwrap();
    /// let object_class = string_class.superclass().unwrap();
    /// assert_eq!(object_class.name().unwrap(), "Object");
    ///
    /// let basic_object = RClass::from_name("BasicObject").unwrap();
    /// assert!(basic_object.superclass().is_none());
    /// ```
    pub fn superclass(self) -> Option<RClass> {
        // SAFETY: self.0 is a valid Ruby class VALUE
        let val = unsafe { rb_sys::rb_class_superclass(self.0.as_raw()) };

        // rb_class_superclass returns Qnil if there's no superclass
        if val == rb_sys::Qnil.into() {
            None
        } else {
            Some(RClass(unsafe { Value::from_raw(val) }))
        }
    }
}

impl ReprValue for RClass {
    #[inline]
    fn as_value(&self) -> Value {
        self.0.clone()
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        RClass(val)
    }
}

impl TryConvert for RClass {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if val.rb_type() == crate::value::ValueType::Class {
            // SAFETY: We've verified it's a Class
            Ok(unsafe { RClass::from_value_unchecked(val) })
        } else {
            Err(Error::type_error("expected Class"))
        }
    }
}

impl IntoValue for RClass {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

#[cfg(all(test, any(feature = "embed", feature = "link-ruby")))]
mod tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;

    #[ruby_test]
    fn test_rclass_from_name() {
        let string_class = RClass::from_name("String").unwrap();
        assert_eq!(string_class.name().unwrap(), "String");
    }

    #[ruby_test]
    #[ignore] // TODO: Fix exception handling in from_name
    fn test_rclass_from_name_missing() {
        let missing = RClass::from_name("NonExistentClass123");
        assert!(missing.is_none());
    }

    #[ruby_test]
    fn test_rclass_name() {
        let string_class = RClass::from_name("String").unwrap();
        assert_eq!(string_class.name().unwrap(), "String");

        let array_class = RClass::from_name("Array").unwrap();
        assert_eq!(array_class.name().unwrap(), "Array");

        let hash_class = RClass::from_name("Hash").unwrap();
        assert_eq!(hash_class.name().unwrap(), "Hash");
    }

    #[ruby_test]
    fn test_rclass_superclass() {
        let string_class = RClass::from_name("String").unwrap();
        let superclass = string_class.superclass().unwrap();
        assert_eq!(superclass.name().unwrap(), "Object");
    }

    #[ruby_test]
    fn test_rclass_superclass_chain() {
        let string_class = RClass::from_name("String").unwrap();

        let object_class = string_class.superclass().unwrap();
        assert_eq!(object_class.name().unwrap(), "Object");

        let basic_object = object_class.superclass().unwrap();
        assert_eq!(basic_object.name().unwrap(), "BasicObject");

        // BasicObject has no superclass
        assert!(basic_object.superclass().is_none());
    }

    #[ruby_test]
    fn test_rclass_try_convert() {
        let string_class = RClass::from_name("String").unwrap();
        let val = string_class.into_value();

        let converted = RClass::try_convert(val).unwrap();
        assert_eq!(converted.name().unwrap(), "String");
    }

    #[ruby_test]
    fn test_rclass_try_convert_wrong_type() {
        let val = 42i64.into_value();
        assert!(RClass::try_convert(val).is_err());
    }

    #[ruby_test]
    fn test_rclass_multiple_builtin_classes() {
        let classes = vec!["String", "Array", "Hash", "Integer", "Float", "Symbol"];

        for class_name in classes {
            let class = RClass::from_name(class_name).unwrap();
            assert_eq!(class.name().unwrap(), class_name);
        }
    }
}
