//! Ruby String type.

use std::ffi::CStr;

use crate::convert::{IntoValue, TryConvert};
use crate::error::Error;
use crate::value::{BoxValue, NewValue, ReprValue, Value};

/// Ruby String (heap allocated).
///
/// Ruby strings are mutable byte sequences with an associated encoding.
/// These are heap-allocated objects that require GC protection.
///
/// This type is `!Copy` to prevent accidental heap storage. Values should be
/// created via `Context::new_string()` for stack-pinned strings within methods,
/// or `RString::new_boxed()` for heap-allocated strings.
///
/// # Example
///
/// ```no_run
/// use solidus::types::RString;
///
/// // For heap storage, use new_boxed()
/// let boxed = RString::new_boxed("hello");
/// assert_eq!(boxed.len(), 5);
///
/// // For stack-pinned strings in methods, use Context::new_string()
/// ```
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct RString(Value);

impl RString {
    /// Create a new Ruby string from a Rust string slice.
    ///
    /// Returns a `NewValue<RString>` guard that must be either:
    /// - Pinned on the stack with `pin_on_stack!`, OR
    /// - Boxed for heap storage with `.into_box()`, OR
    /// - Immediately returned to Ruby
    ///
    /// # Safety
    ///
    /// The caller must ensure the returned guard is handled appropriately.
    /// Failing to pin or box the value may result in it being collected by Ruby's GC.
    ///
    /// For safe alternatives, use:
    /// - `RString::new_boxed()` for heap storage
    /// - `Context::new_string()` for stack-pinned strings in methods
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RString;
    /// use solidus::convert::IntoValue;
    /// use solidus::pin_on_stack;
    ///
    /// // SAFETY: Value is immediately pinned
    /// pin_on_stack!(s = unsafe { RString::new("hello") });
    ///
    /// // Or: SAFETY: Value is immediately returned to Ruby
    /// let guard = unsafe { RString::new("hello") };
    /// let raw = guard.into_value().as_raw();
    /// ```
    pub unsafe fn new(s: &str) -> NewValue<Self> {
        // SAFETY: Caller ensures the returned value is properly handled
        NewValue::new(unsafe { Self::from_slice_raw(s.as_bytes()) })
    }

    /// Create a new Ruby string, boxed for heap storage.
    ///
    /// This is safe because the value is immediately registered with Ruby's GC.
    /// Use `Context::new_string()` for stack-pinned strings within methods.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RString;
    ///
    /// let boxed = RString::new_boxed("hello world");
    /// assert_eq!(boxed.len(), 11);
    /// ```
    pub fn new_boxed(s: &str) -> BoxValue<Self> {
        // SAFETY: We immediately box and register with GC
        unsafe { BoxValue::new(Self::from_slice_raw(s.as_bytes())) }
    }

    /// Create a new Ruby string from a byte slice.
    ///
    /// Returns a `NewValue<RString>` guard that must be either:
    /// - Pinned on the stack with `pin_on_stack!`, OR
    /// - Boxed for heap storage with `.into_box()`, OR
    /// - Immediately returned to Ruby
    ///
    /// # Safety
    ///
    /// The caller must ensure the returned guard is handled appropriately.
    /// Failing to pin or box the value may result in it being collected by Ruby's GC.
    ///
    /// For a safe alternative, use `RString::from_slice_boxed()`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RString;
    /// use solidus::pin_on_stack;
    ///
    /// // SAFETY: Value is immediately pinned
    /// pin_on_stack!(s = unsafe { RString::from_slice(b"hello\x00world") });
    /// ```
    pub unsafe fn from_slice(bytes: &[u8]) -> NewValue<Self> {
        // SAFETY: Caller ensures the returned value is properly handled
        NewValue::new(unsafe { Self::from_slice_raw(bytes) })
    }

    /// Internal: Create a new Ruby string from a byte slice without guard.
    ///
    /// This is unsafe and returns the raw RString for internal use only.
    /// Users should use `from_slice()` or `from_slice_boxed()` instead.
    #[doc(hidden)]
    unsafe fn from_slice_raw(bytes: &[u8]) -> Self {
        // SAFETY: rb_str_new creates a new Ruby string with the given bytes
        let val = unsafe {
            rb_sys::rb_str_new(
                bytes.as_ptr() as *const std::os::raw::c_char,
                bytes.len() as _,
            )
        };
        // SAFETY: rb_str_new returns a valid VALUE
        RString(unsafe { Value::from_raw(val) })
    }

    /// Create a new Ruby string from bytes, boxed for heap storage.
    ///
    /// This is safe because the value is immediately registered with Ruby's GC.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RString;
    ///
    /// let bytes = b"hello\x00world";
    /// let boxed = RString::from_slice_boxed(bytes);
    /// assert_eq!(boxed.len(), 11);
    /// ```
    pub fn from_slice_boxed(bytes: &[u8]) -> BoxValue<Self> {
        // SAFETY: We immediately box and register with GC
        unsafe { BoxValue::new(Self::from_slice_raw(bytes)) }
    }

    /// Get the length of the string in bytes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RString;
    ///
    /// let s = RString::new_boxed("hello");
    /// assert_eq!(s.len(), 5);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        // SAFETY: self.0 is a valid Ruby string VALUE
        unsafe { rb_sys::RSTRING_LEN(self.0.as_raw()) as usize }
    }

    /// Check if the string is empty.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RString;
    ///
    /// let s = RString::new_boxed("");
    /// assert!(s.is_empty());
    ///
    /// let s2 = RString::new_boxed("hello");
    /// assert!(!s2.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get string contents as a byte slice.
    ///
    /// # Safety
    ///
    /// The returned slice is only valid while:
    /// - No Ruby code runs that could modify the string
    /// - No Ruby code runs that could trigger a GC compaction (Ruby 2.7+)
    /// - The string value is not moved or deallocated
    ///
    /// The caller must ensure the string remains valid and unmodified
    /// for the lifetime of the returned slice.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RString;
    ///
    /// let s = RString::new_boxed("hello");
    /// unsafe {
    ///     let bytes = s.as_slice();
    ///     assert_eq!(bytes, b"hello");
    /// }
    /// ```
    pub unsafe fn as_slice(&self) -> &'static [u8] {
        // SAFETY: Caller ensures string is valid and unmodified
        unsafe {
            let ptr = rb_sys::RSTRING_PTR(self.0.as_raw());
            let len = rb_sys::RSTRING_LEN(self.0.as_raw()) as usize;
            std::slice::from_raw_parts(ptr as *const u8, len)
        }
    }

    /// Copy string contents to a Rust String.
    ///
    /// Returns an error if the string contains invalid UTF-8.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RString;
    ///
    /// let s = RString::new_boxed("hello");
    /// assert_eq!(s.to_string().unwrap(), "hello");
    /// ```
    pub fn to_string(&self) -> Result<String, Error> {
        // SAFETY: We immediately copy the bytes, so they don't outlive the string
        let bytes = unsafe { self.as_slice() };
        String::from_utf8(bytes.to_vec()).map_err(|e| {
            Error::new(
                crate::ExceptionClass::TypeError,
                format!("invalid UTF-8 in Ruby string: {}", e),
            )
        })
    }

    /// Copy string contents to a byte vector.
    ///
    /// This always succeeds and works with any byte sequence.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RString;
    ///
    /// let s = RString::new_boxed("hello");
    /// assert_eq!(s.to_bytes(), b"hello");
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        // SAFETY: We immediately copy the bytes, so they don't outlive the string
        unsafe { self.as_slice().to_vec() }
    }

    /// Get the encoding of this string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RString;
    ///
    /// let s = RString::new_boxed("hello");
    /// let enc = s.encoding();
    /// ```
    pub fn encoding(&self) -> Encoding {
        // SAFETY: self.0 is a valid Ruby string VALUE
        let enc_ptr = unsafe { rb_sys::rb_enc_get(self.0.as_raw()) };
        Encoding { ptr: enc_ptr }
    }

    /// Encode this string to a different encoding.
    ///
    /// Returns a new string with the specified encoding.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::{RString, Encoding};
    ///
    /// let s = RString::new_boxed("hello");
    /// let utf8 = Encoding::utf8();
    /// let encoded = s.encode(utf8).unwrap();
    /// ```
    pub fn encode(&self, encoding: Encoding) -> Result<RString, Error> {
        // SAFETY: self.0 is a valid Ruby string, encoding.ptr is a valid encoding
        let val = unsafe {
            let enc_value = rb_sys::rb_enc_from_encoding(encoding.ptr);
            rb_sys::rb_str_encode(self.0.as_raw(), enc_value, 0, rb_sys::Qnil.into())
        };

        // Check if an exception was raised
        // For now, we'll just wrap the result
        Ok(RString(unsafe { Value::from_raw(val) }))
    }
}

impl ReprValue for RString {
    #[inline]
    fn as_value(&self) -> Value {
        self.0.clone()
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        RString(val)
    }
}

impl TryConvert for RString {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if val.rb_type() == crate::value::ValueType::String {
            // SAFETY: We've verified it's a String
            Ok(unsafe { RString::from_value_unchecked(val) })
        } else {
            Err(Error::type_error("expected String"))
        }
    }
}

impl IntoValue for RString {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

// Conversions for Rust String types

impl TryConvert for String {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let rstring = RString::try_convert(val)?;
        rstring.to_string()
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        // Use the boxed version for safety
        RString::new_boxed(&self).as_value()
    }
}

// Convert string slices to Ruby strings
impl IntoValue for &str {
    fn into_value(self) -> Value {
        // Use the boxed version for safety
        RString::new_boxed(self).as_value()
    }
}

/// Ruby string encoding.
///
/// This type represents a Ruby encoding object (rb_encoding).
/// Ruby strings have an associated encoding that determines how
/// bytes are interpreted as characters.
///
/// Encodings are immediate values in Ruby, so this type can be safely copied.
///
/// # Example
///
/// ```no_run
/// use solidus::types::{RString, Encoding};
///
/// let enc = Encoding::utf8();
/// let s = RString::new_boxed("hello");
/// let encoded = s.encode(enc).unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct Encoding {
    ptr: *mut rb_sys::rb_encoding,
}

impl Encoding {
    /// Get the UTF-8 encoding.
    ///
    /// UTF-8 is Ruby's most common string encoding.
    pub fn utf8() -> Self {
        // SAFETY: rb_utf8_encoding returns a valid encoding pointer
        let ptr = unsafe { rb_sys::rb_utf8_encoding() };
        Encoding { ptr }
    }

    /// Get the ASCII-8BIT (binary) encoding.
    ///
    /// This encoding treats strings as raw byte sequences.
    pub fn ascii_8bit() -> Self {
        // SAFETY: rb_ascii8bit_encoding returns a valid encoding pointer
        let ptr = unsafe { rb_sys::rb_ascii8bit_encoding() };
        Encoding { ptr }
    }

    /// Get the US-ASCII encoding.
    ///
    /// This encoding only allows bytes 0-127.
    pub fn us_ascii() -> Self {
        // SAFETY: rb_usascii_encoding returns a valid encoding pointer
        let ptr = unsafe { rb_sys::rb_usascii_encoding() };
        Encoding { ptr }
    }

    /// Find an encoding by name.
    ///
    /// Returns None if the encoding is not found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::Encoding;
    ///
    /// let enc = Encoding::find("UTF-8").unwrap();
    /// let enc2 = Encoding::find("ISO-8859-1");
    /// ```
    pub fn find(name: &str) -> Option<Self> {
        // Convert to C string
        let c_name = std::ffi::CString::new(name).ok()?;

        // SAFETY: rb_enc_find_index returns -1 if not found
        let index = unsafe { rb_sys::rb_enc_find_index(c_name.as_ptr()) };

        if index < 0 {
            None
        } else {
            // SAFETY: index is valid if >= 0
            let ptr = unsafe { rb_sys::rb_enc_from_index(index) };
            Some(Encoding { ptr })
        }
    }

    /// Get the name of this encoding.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::Encoding;
    ///
    /// let enc = Encoding::utf8();
    /// assert_eq!(enc.name(), "UTF-8");
    /// ```
    pub fn name(&self) -> &'static str {
        // SAFETY: The encoding pointer has a name field that points to a static string
        unsafe {
            let ptr = (*self.ptr).name;
            let c_str = CStr::from_ptr(ptr);
            c_str.to_str().unwrap_or("unknown")
        }
    }
}

// SAFETY: Encoding pointers are global and thread-safe
unsafe impl Send for Encoding {}
unsafe impl Sync for Encoding {}

#[cfg(all(test, any(feature = "embed", feature = "link-ruby")))]
mod tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;

    #[ruby_test]
    fn test_rstring_new_boxed() {
        let s = RString::new_boxed("hello");
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());
    }

    #[ruby_test]
    fn test_rstring_empty() {
        let s = RString::new_boxed("");
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
    }

    #[ruby_test]
    fn test_rstring_from_slice_boxed() {
        let bytes = b"hello\x00world";
        let s = RString::from_slice_boxed(bytes);
        assert_eq!(s.len(), 11);
    }

    #[ruby_test]
    fn test_rstring_to_string() {
        let s = RString::new_boxed("hello world");
        assert_eq!(s.to_string().unwrap(), "hello world");
    }

    #[ruby_test]
    fn test_rstring_to_bytes() {
        let s = RString::new_boxed("hello");
        assert_eq!(s.to_bytes(), b"hello");
    }

    #[ruby_test]
    fn test_rstring_as_slice() {
        let s = RString::new_boxed("hello");
        unsafe {
            let bytes = s.as_slice();
            assert_eq!(bytes, b"hello");
        }
    }

    #[ruby_test]
    fn test_rstring_try_convert() {
        let s = RString::new_boxed("test");
        let val = s.into_value();
        let s = RString::try_convert(val).unwrap();
        assert_eq!(s.to_string().unwrap(), "test");
    }

    #[ruby_test]
    fn test_rstring_try_convert_wrong_type() {
        let val = 42i64.into_value();
        assert!(RString::try_convert(val).is_err());
    }

    #[ruby_test]
    fn test_string_conversion() {
        let rust_string = String::from("hello");
        let val = rust_string.clone().into_value();
        let converted = String::try_convert(val).unwrap();
        assert_eq!(converted, rust_string);
    }

    #[ruby_test]
    fn test_encoding_utf8() {
        let enc = Encoding::utf8();
        assert_eq!(enc.name(), "UTF-8");
    }

    #[ruby_test]
    fn test_encoding_ascii_8bit() {
        let enc = Encoding::ascii_8bit();
        assert_eq!(enc.name(), "ASCII-8BIT");
    }

    #[ruby_test]
    fn test_encoding_us_ascii() {
        let enc = Encoding::us_ascii();
        assert_eq!(enc.name(), "US-ASCII");
    }

    #[ruby_test]
    fn test_encoding_find() {
        let enc = Encoding::find("UTF-8").unwrap();
        assert_eq!(enc.name(), "UTF-8");

        let enc2 = Encoding::find("ISO-8859-1").unwrap();
        assert_eq!(enc2.name(), "ISO-8859-1");
    }

    #[ruby_test]
    fn test_encoding_find_not_found() {
        let enc = Encoding::find("INVALID-ENCODING");
        assert!(enc.is_none());
    }

    #[ruby_test]
    fn test_rstring_encoding() {
        let s = RString::new_boxed("hello");
        let enc = s.encoding();
        // Default encoding depends on Ruby version and environment
        // Just verify we can get it
        let _name = enc.name();
    }

    #[ruby_test]
    fn test_rstring_encode() {
        let s = RString::new_boxed("hello");
        let utf8 = Encoding::utf8();
        let encoded = s.encode(utf8).unwrap();
        assert_eq!(encoded.to_string().unwrap(), "hello");
    }

    #[ruby_test]
    fn test_rstring_round_trip() {
        let original = "test string with émojis 🎉";
        let s = RString::new_boxed(original);
        assert_eq!(s.to_string().unwrap(), original);
    }

    #[ruby_test]
    fn test_rstring_with_null_bytes() {
        let bytes = b"hello\x00world\x00";
        let s = RString::from_slice_boxed(bytes);
        assert_eq!(s.len(), 12);
        assert_eq!(s.to_bytes(), bytes);
    }
}
