//! Phase 2 Stage 4: String Type Example
//!
//! This example demonstrates Ruby's String type with encoding support.
//! Strings are heap-allocated objects that require GC protection.
//!
//! This shows Stage 4 implementation: RString type with UTF-8 and binary data handling.

use solidus::prelude::*;

/// Example 1: Creating strings from &str
///
/// Demonstrates basic string creation from Rust string slices.
#[no_mangle]
pub extern "C" fn example_string_from_str() -> rb_sys::VALUE {
    // Create a Ruby string from a &str - returns NewValue<RString>
    // SAFETY: Value is used immediately and returned to Ruby
    let s = unsafe { RString::new("Hello, Solidus!") };

    // Check basic properties using as_ref() for temporary access
    assert_eq!(s.as_ref().len(), 15);
    assert!(!s.as_ref().is_empty());

    // Convert back to Rust String
    let rust_string = s.as_ref().to_string().unwrap();
    assert_eq!(rust_string, "Hello, Solidus!");

    // Convert NewValue to VALUE for return
    s.into_value().as_raw()
}

/// Example 2: Empty string handling
///
/// Demonstrates working with empty strings.
#[no_mangle]
pub extern "C" fn example_empty_string() -> rb_sys::VALUE {
    // Create an empty string
    // SAFETY: Value is used immediately and returned to Ruby
    let s = unsafe { RString::new("") };

    // Verify it's empty
    assert_eq!(s.as_ref().len(), 0);
    assert!(s.as_ref().is_empty());

    // Convert to Rust String
    let rust_string = s.as_ref().to_string().unwrap();
    assert_eq!(rust_string, "");

    s.into_value().as_raw()
}

/// Example 3: Creating strings from byte slices
///
/// Shows how to create strings from raw byte data.
#[no_mangle]
pub extern "C" fn example_string_from_bytes() -> rb_sys::VALUE {
    // Create a string from a byte slice
    let bytes = b"Binary \x00 data with null bytes";
    // SAFETY: Value is used immediately and returned to Ruby
    let s = unsafe { RString::from_slice(bytes) };

    // Length includes the null byte
    assert_eq!(s.as_ref().len(), 32);

    // Get bytes back
    let bytes_back = s.as_ref().to_bytes();
    assert_eq!(bytes_back, bytes);

    s.into_value().as_raw()
}

/// Example 4: UTF-8 string handling
///
/// Demonstrates working with UTF-8 encoded strings.
#[no_mangle]
pub extern "C" fn example_utf8_string() -> rb_sys::VALUE {
    // Create a string with Unicode characters
    // SAFETY: Value is used immediately and returned to Ruby
    let s = unsafe { RString::new("Hello 世界 🌍") };

    // Get the byte length (UTF-8 encoded)
    let byte_len = s.as_ref().len();
    assert!(byte_len > 10); // Unicode chars take multiple bytes

    // Convert to Rust String preserving UTF-8
    let rust_string = s.as_ref().to_string().unwrap();
    assert_eq!(rust_string, "Hello 世界 🌍");

    s.into_value().as_raw()
}

/// Example 5: Binary (non-UTF-8) data
///
/// Shows handling of binary data that isn't valid UTF-8.
#[no_mangle]
pub extern "C" fn example_binary_string() -> rb_sys::VALUE {
    // Create a string with invalid UTF-8 bytes
    let bytes = b"\xFF\xFE invalid UTF-8 \x80\x81";
    // SAFETY: Value is used immediately and returned to Ruby
    let s = unsafe { RString::from_slice(bytes) };

    assert_eq!(s.as_ref().len(), 21);

    // to_string() will fail for invalid UTF-8
    assert!(s.as_ref().to_string().is_err());

    // to_bytes() always works
    let bytes_back = s.as_ref().to_bytes();
    assert_eq!(bytes_back, bytes);

    s.into_value().as_raw()
}

/// Example 6: String encoding information
///
/// Demonstrates getting encoding information from strings.
#[no_mangle]
pub extern "C" fn example_string_encoding() -> rb_sys::VALUE {
    // Create a UTF-8 string
    // SAFETY: Value is used immediately and returned to Ruby
    let s = unsafe { RString::new("Hello") };

    // Get the encoding
    let enc = s.as_ref().encoding();
    let _enc_name = enc.name();

    // Default encoding is usually UTF-8
    // (depends on Ruby version and environment)

    // We can also get standard encodings
    let utf8_enc = Encoding::utf8();
    assert_eq!(utf8_enc.name(), "UTF-8");

    let ascii_enc = Encoding::ascii_8bit();
    assert_eq!(ascii_enc.name(), "ASCII-8BIT");

    let us_ascii = Encoding::us_ascii();
    assert_eq!(us_ascii.name(), "US-ASCII");

    s.into_value().as_raw()
}

/// Example 7: Encoding conversion
///
/// Shows how to convert strings between different encodings.
#[no_mangle]
pub extern "C" fn example_encoding_conversion() -> rb_sys::VALUE {
    // Create a string
    // SAFETY: Value is used immediately and returned to Ruby
    let s = unsafe { RString::new("Hello") };

    // Convert to UTF-8
    let utf8_enc = Encoding::utf8();
    let utf8_str = s.as_ref().encode(utf8_enc).unwrap();

    // Verify it's still the same content
    assert_eq!(utf8_str.to_string().unwrap(), "Hello");

    // Convert to ASCII-8BIT (binary)
    let binary_enc = Encoding::ascii_8bit();
    let binary_str = s.as_ref().encode(binary_enc).unwrap();

    assert_eq!(binary_str.to_bytes(), b"Hello");

    utf8_str.into_value().as_raw()
}

/// Example 8: String conversions between Rust and Ruby
///
/// Demonstrates bidirectional conversions.
#[no_mangle]
pub extern "C" fn example_string_conversions(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };

    // Try to convert Ruby value to Rust String
    if let Ok(rust_string) = String::try_convert(value) {
        // Successfully converted - process it
        let upper = rust_string.to_uppercase();

        // Convert back to Ruby string - NewValue can be converted directly
        // SAFETY: Value is immediately returned to Ruby
        return unsafe { RString::new(&upper) }.into_value().as_raw();
    }

    // Not a string - return nil
    Qnil::new().as_value().as_raw()
}

/// Example 9: String with null bytes
///
/// Shows that Ruby strings can contain null bytes (unlike C strings).
#[no_mangle]
pub extern "C" fn example_string_with_nulls() -> rb_sys::VALUE {
    // Create a string with embedded null bytes
    let bytes = b"before\x00middle\x00after";
    // SAFETY: Value is used immediately and returned to Ruby
    let s = unsafe { RString::from_slice(bytes) };

    // Length includes null bytes
    assert_eq!(s.as_ref().len(), 19);

    // All bytes are preserved
    let bytes_back = s.as_ref().to_bytes();
    assert_eq!(bytes_back, bytes);
    assert_eq!(bytes_back[6], 0); // null byte at position 6
    assert_eq!(bytes_back[13], 0); // null byte at position 13

    s.into_value().as_raw()
}

/// Example 10: Round-trip string conversion
///
/// Demonstrates safe round-trip conversion between Rust and Ruby.
#[no_mangle]
pub extern "C" fn example_string_roundtrip() -> rb_sys::VALUE {
    // Start with a Rust string
    let original = "The quick brown fox 🦊 jumps over the lazy dog 🐕";

    // Convert to Ruby
    // SAFETY: Value is used immediately and returned to Ruby
    let ruby_str = unsafe { RString::new(original) };

    // Convert back to Rust
    let roundtrip = ruby_str.as_ref().to_string().unwrap();

    // Should be identical
    assert_eq!(roundtrip, original);
    assert_eq!(ruby_str.as_ref().len(), original.len());

    ruby_str.into_value().as_raw()
}

/// Example 11: Finding encodings by name
///
/// Shows how to look up encodings dynamically.
#[no_mangle]
pub extern "C" fn example_find_encoding() -> rb_sys::VALUE {
    // Find encodings by name
    if let Some(utf8) = Encoding::find("UTF-8") {
        assert_eq!(utf8.name(), "UTF-8");
    }

    if let Some(latin1) = Encoding::find("ISO-8859-1") {
        assert_eq!(latin1.name(), "ISO-8859-1");
    }

    // Non-existent encoding returns None
    assert!(Encoding::find("INVALID-ENCODING").is_none());

    // Return a UTF-8 string - can return NewValue directly
    // SAFETY: Value is immediately returned to Ruby
    unsafe { RString::new("Encoding lookup works!") }.into_value().as_raw()
}

/// Example 12: Type-safe string handling
///
/// Shows compile-time guarantees for string operations.
fn concatenate_strings(s1: &RString, s2: &RString) -> Result<NewValue<RString>, Error> {
    // Both strings are valid Ruby strings at compile time
    let str1 = s1.to_string()?;
    let str2 = s2.to_string()?;

    let result = format!("{} {}", str1, str2);
    // SAFETY: Value is immediately returned
    Ok(unsafe { RString::new(&result) })
}

#[no_mangle]
pub extern "C" fn example_string_concatenation() -> rb_sys::VALUE {
    // SAFETY: Values are used immediately
    let s1 = unsafe { RString::new("Hello") };
    let s2 = unsafe { RString::new("World") };

    match concatenate_strings(s1.as_ref(), s2.as_ref()) {
        Ok(result) => {
            assert_eq!(result.as_ref().to_string().unwrap(), "Hello World");
            result.into_value().as_raw()
        }
        Err(_) => Qnil::new().as_value().as_raw(),
    }
}

/// Initialize the extension
#[no_mangle]
pub extern "C" fn Init_phase2_string() {
    // Note: Full method definition requires Phase 3
    // For now, this is just a placeholder that Ruby will call when loading the extension
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_checks() {
        // These tests verify compile-time behavior only
        // Tests requiring Ruby API calls need the Ruby runtime

        // RString is no longer Copy - it's !Copy to enforce pinning
        // This test now verifies that RString is Clone but not Copy
        fn assert_clone<T: Clone>() {}
        assert_clone::<RString>();

        // Encoding is still Copy (it's immediate)
        fn assert_copy<T: Copy>() {}
        assert_copy::<Encoding>();
    }

    #[test]
    fn test_string_types() {
        // Verify type properties without calling Ruby API

        // RString should be transparent wrapper
        assert_eq!(std::mem::size_of::<RString>(), std::mem::size_of::<Value>());

        // Encoding should be a pointer size
        assert_eq!(std::mem::size_of::<Encoding>(), std::mem::size_of::<usize>());
    }
}
