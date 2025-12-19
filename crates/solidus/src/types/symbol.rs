//! Ruby Symbol type (immediate value).

use std::ffi::CString;

use crate::convert::{IntoValue, TryConvert};
use crate::error::Error;
use crate::value::{ReprValue, Value, ValueType};

/// Ruby Symbol (interned string, immediate value).
///
/// Symbols are Ruby's interned strings - immutable strings that are stored
/// once and reused. They're commonly used for hash keys and method names.
///
/// Symbol is an immediate value and does not require GC protection or pinning.
///
/// # Example
///
/// ```no_run
/// use solidus::types::Symbol;
///
/// let sym = Symbol::new("hello");
/// assert_eq!(sym.name().unwrap(), "hello");
/// ```
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Symbol(Value);

impl Symbol {
    /// Create or get an existing symbol from a string.
    ///
    /// Symbols are interned, so calling this with the same string multiple times
    /// will return the same Symbol.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::Symbol;
    /// use solidus::value::ReprValue;
    ///
    /// let sym1 = Symbol::new("foo");
    /// let sym2 = Symbol::new("foo");
    /// assert_eq!(sym1.as_value(), sym2.as_value());
    /// ```
    pub fn new(name: &str) -> Self {
        let c_name = CString::new(name).expect("symbol name contained null byte");
        // SAFETY: rb_intern interns the string and returns a symbol ID,
        // then rb_id2sym converts it to a Symbol VALUE
        let val = unsafe {
            let id = rb_sys::rb_intern(c_name.as_ptr());
            Value::from_raw(rb_sys::rb_id2sym(id))
        };
        Symbol(val)
    }

    /// Get the symbol's name as a String.
    ///
    /// # Errors
    ///
    /// Returns an error if the symbol name contains invalid UTF-8.
    pub fn name(&self) -> Result<String, Error> {
        // SAFETY: rb_sym2str converts symbol to string
        let str_val = unsafe { Value::from_raw(rb_sys::rb_sym2str(self.0.as_raw())) };

        // Get the string pointer and length
        // SAFETY: We know str_val is a String from rb_sym2str
        unsafe {
            let ptr = rb_sys::RSTRING_PTR(str_val.as_raw()) as *const u8;
            let len = rb_sys::RSTRING_LEN(str_val.as_raw()) as usize;
            let slice = std::slice::from_raw_parts(ptr, len);

            String::from_utf8(slice.to_vec()).map_err(|_| {
                Error::new(
                    crate::ExceptionClass::ArgumentError,
                    "symbol name is not valid UTF-8",
                )
            })
        }
    }
}

impl ReprValue for Symbol {
    #[inline]
    fn as_value(&self) -> Value {
        self.0.clone()
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        Symbol(val)
    }
}

impl TryConvert for Symbol {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if val.rb_type() == ValueType::Symbol {
            // SAFETY: We've verified it's a Symbol
            Ok(unsafe { Symbol::from_value_unchecked(val) })
        } else {
            Err(Error::type_error("expected Symbol"))
        }
    }
}

impl IntoValue for Symbol {
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
    fn test_symbol_new() {
        let sym = Symbol::new("test");
        assert_eq!(sym.as_value().rb_type(), ValueType::Symbol);
    }

    #[ruby_test]
    fn test_symbol_identity() {
        let sym1 = Symbol::new("foo");
        let sym2 = Symbol::new("foo");
        assert_eq!(sym1.as_value(), sym2.as_value());
    }

    #[ruby_test]
    fn test_symbol_different() {
        let sym1 = Symbol::new("foo");
        let sym2 = Symbol::new("bar");
        assert_ne!(sym1.as_value(), sym2.as_value());
    }
}
