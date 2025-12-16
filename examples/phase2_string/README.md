# Phase 2 Stage 4: String Type Example

This example demonstrates Ruby's String type in Solidus with full encoding support, UTF-8 handling, and binary data operations.

## Overview

Stage 4 of Phase 2 implements the **RString** type, which wraps Ruby's String class. Unlike immediate values (Stage 2), strings are heap-allocated objects that require GC protection when used in method signatures (Phase 3+).

### Key Features

- **UTF-8 strings** - Safe handling of Unicode text
- **Binary data** - Support for non-UTF-8 byte sequences
- **Encoding support** - Get, set, and convert between encodings
- **Null bytes** - Ruby strings can contain null bytes (unlike C strings)
- **Type safety** - Compile-time guarantees for string operations
- **Safe conversions** - Validated UTF-8 conversion to Rust String
- **Always-safe bytes** - to_bytes() works with any byte sequence

## What This Example Demonstrates

1. **String Creation** - From &str and byte slices
2. **Empty Strings** - Proper empty string handling
3. **Byte Slices** - Creating strings from raw byte data
4. **UTF-8 Strings** - Unicode character support
5. **Binary Data** - Non-UTF-8 byte sequences
6. **Encoding Info** - Getting encoding details
7. **Encoding Conversion** - Converting between encodings
8. **Rust ↔ Ruby Conversions** - Bidirectional type conversion
9. **Null Bytes** - Strings with embedded nulls
10. **Round-trip Conversion** - Safe UTF-8 round trips
11. **Encoding Lookup** - Finding encodings by name
12. **String Concatenation** - Type-safe string operations

## Ruby String Encoding

Ruby strings are not just character sequences - they're **byte sequences with an associated encoding**. The encoding determines how those bytes are interpreted as characters.

### Common Encodings

| Encoding | Description | Use Case |
|----------|-------------|----------|
| UTF-8 | Unicode (variable-width) | Most text, internationalization |
| ASCII-8BIT | Binary data | Raw bytes, no character interpretation |
| US-ASCII | 7-bit ASCII only | Simple English text, backwards compatibility |

### Encoding in Solidus

```rust
// Get string encoding
let s = RString::new("hello");
let enc = s.encoding();
println!("Encoding: {}", enc.name());

// Standard encodings
let utf8 = Encoding::utf8();          // UTF-8
let binary = Encoding::ascii_8bit();  // ASCII-8BIT (binary)
let ascii = Encoding::us_ascii();     // US-ASCII

// Look up encoding by name
if let Some(enc) = Encoding::find("ISO-8859-1") {
    let converted = s.encode(enc).unwrap();
}
```

## Safe vs Unsafe String Access

### Safe: Copying Data

```rust
// Safe: Copies data to owned Rust types
let rust_string = rstring.to_string().unwrap(); // String (validates UTF-8)
let bytes = rstring.to_bytes();                 // Vec<u8> (always works)
```

These methods copy the string data, so they're safe to use at any time. The original Ruby string can be modified or garbage collected without affecting your Rust data.

### Unsafe: Borrowing Data

```rust
// Unsafe: Borrows Ruby's internal buffer
unsafe {
    let slice = rstring.as_slice(); // &'static [u8]
    // Only valid while:
    // 1. No Ruby code runs that could modify the string
    // 2. No GC compaction occurs (Ruby 2.7+)
    // 3. String value stays alive
}
```

This is much faster (no copy) but requires careful lifetime management. **Only use this when performance is critical and you understand the constraints.**

## UTF-8 vs Binary Strings

### UTF-8 Strings

```rust
// Create UTF-8 string
let s = RString::new("Hello 世界 🌍");

// Safe conversion to Rust String
let rust_str = s.to_string().unwrap(); // Ok: valid UTF-8

// UTF-8 characters can be multiple bytes
assert!(s.len() > 10); // "Hello 世界 🌍" is ~17 bytes
```

### Binary Strings

```rust
// Create binary string (not valid UTF-8)
let bytes = b"\xFF\xFE invalid UTF-8 \x80\x81";
let s = RString::from_slice(bytes);

// to_string() fails for invalid UTF-8
assert!(s.to_string().is_err());

// to_bytes() always works
let bytes_back = s.to_bytes();
assert_eq!(bytes_back, bytes);
```

## Conversion Patterns

### Rust String → Ruby String

```rust
// From &str
let s = RString::new("hello");

// From String
let rust_string = String::from("hello");
let ruby_value = rust_string.into_value();
```

### Ruby String → Rust String

```rust
// Safe: Validates UTF-8 and copies
match rstring.to_string() {
    Ok(s) => println!("Valid UTF-8: {}", s),
    Err(_) => println!("Not valid UTF-8"),
}

// Always safe: Get raw bytes
let bytes = rstring.to_bytes();
```

### Type Conversion

```rust
// TryConvert for Value → String
let value: Value = /* ... */;
if let Ok(s) = String::try_convert(value) {
    // Successfully converted Ruby string to Rust String
    println!("{}", s);
}

// IntoValue for String → Value
let rust_string = String::from("hello");
let value = rust_string.into_value();
```

## Code Examples

### Creating Strings

```rust
// From &str
let s1 = RString::new("Hello, world!");

// From byte slice
let bytes = b"Binary \x00 data";
let s2 = RString::from_slice(bytes);

// Empty string
let s3 = RString::new("");
assert!(s3.is_empty());
```

### String Properties

```rust
let s = RString::new("Hello");

// Length in bytes
assert_eq!(s.len(), 5);

// Check if empty
assert!(!s.is_empty());

// Get encoding
let enc = s.encoding();
println!("Encoding: {}", enc.name());
```

### Working with Content

```rust
let s = RString::new("Hello 世界");

// Convert to Rust String (validates UTF-8)
let rust_str = s.to_string().unwrap();
assert_eq!(rust_str, "Hello 世界");

// Get raw bytes
let bytes = s.to_bytes();
assert!(bytes.len() > 5); // UTF-8 encoding
```

### Handling Binary Data

```rust
// Binary data with invalid UTF-8
let bytes = b"\xFF\xFE\x00\x80";
let s = RString::from_slice(bytes);

// to_string() fails
assert!(s.to_string().is_err());

// to_bytes() always works
let recovered = s.to_bytes();
assert_eq!(recovered, bytes);
```

### String with Null Bytes

```rust
// Unlike C strings, Ruby strings can contain nulls
let bytes = b"before\x00after";
let s = RString::from_slice(bytes);

assert_eq!(s.len(), 12); // Includes the null byte
let back = s.to_bytes();
assert_eq!(back[6], 0); // Null at position 6
```

### Encoding Conversion

```rust
let s = RString::new("Hello");

// Convert to UTF-8
let utf8 = Encoding::utf8();
let utf8_str = s.encode(utf8).unwrap();

// Convert to binary
let binary = Encoding::ascii_8bit();
let binary_str = s.encode(binary).unwrap();

// Look up encoding by name
if let Some(enc) = Encoding::find("UTF-8") {
    assert_eq!(enc.name(), "UTF-8");
}
```

## Building and Running

Build the example:

```bash
cargo build --release --manifest-path examples/phase2_string/Cargo.toml
```

Run the Ruby test script:

```bash
ruby examples/phase2_string/test.rb
```

Or run just the Rust tests:

```bash
cargo test --manifest-path examples/phase2_string/Cargo.toml
```

## Code Structure

- `src/lib.rs` - Example functions demonstrating string operations
- `test.rb` - Ruby script that builds and tests the extension
- `Cargo.toml` - Build configuration
- `build.rs` - Ruby integration build script

## RString API Summary

### Construction

| Method | Description |
|--------|-------------|
| `RString::new(s: &str)` | Create from Rust string slice |
| `RString::from_slice(bytes: &[u8])` | Create from byte slice |

### Properties

| Method | Description |
|--------|-------------|
| `len(self) -> usize` | Get byte length |
| `is_empty(self) -> bool` | Check if empty |
| `encoding(self) -> Encoding` | Get string encoding |

### Content Access

| Method | Description |
|--------|-------------|
| `to_string(self) -> Result<String, Error>` | Copy to Rust String (validates UTF-8) |
| `to_bytes(self) -> Vec<u8>` | Copy to byte vector (always works) |
| `unsafe as_slice(self) -> &'static [u8]` | Borrow internal buffer (unsafe) |

### Encoding Operations

| Method | Description |
|--------|-------------|
| `encode(self, enc: Encoding) -> Result<RString, Error>` | Convert to encoding |

## Encoding API Summary

### Standard Encodings

| Method | Description |
|--------|-------------|
| `Encoding::utf8()` | Get UTF-8 encoding |
| `Encoding::ascii_8bit()` | Get ASCII-8BIT (binary) encoding |
| `Encoding::us_ascii()` | Get US-ASCII encoding |

### Encoding Operations

| Method | Description |
|--------|-------------|
| `Encoding::find(name: &str) -> Option<Encoding>` | Look up encoding by name |
| `name(self) -> &'static str` | Get encoding name |

## Key Design Points

### Why Separate to_string() and to_bytes()?

- **to_string()** - Type-safe UTF-8 validation. Returns `Result` because not all Ruby strings are valid UTF-8.
- **to_bytes()** - Always succeeds. Use when working with binary data or when UTF-8 validation isn't needed.

### Why is as_slice() Unsafe?

Ruby strings are mutable and subject to garbage collection. The returned slice borrows Ruby's internal buffer, which can be invalidated if:

1. Ruby code modifies the string
2. GC compaction moves the string (Ruby 2.7+)
3. The string is garbage collected

Use `to_string()` or `to_bytes()` instead unless you need maximum performance and can guarantee the string won't be modified.

### Encoding Edge Cases

- Default encoding depends on Ruby version and environment
- String created with `new()` usually gets UTF-8 encoding
- String created with `from_slice()` usually gets ASCII-8BIT (binary)
- Always check encoding if it matters for your use case

## Comparison with Magnus

| Feature | Solidus | Magnus |
|---------|---------|--------|
| String creation | `RString::new()` | `RString::new()` |
| UTF-8 validation | `to_string() -> Result` | `to_str()? + panic` |
| Binary data | `to_bytes() -> Vec<u8>` | `as_slice()` (unsafe) |
| Encoding support | Full `Encoding` type | Limited |
| Null bytes | Full support | Full support |
| Type safety | Compile-time checks | Runtime checks |

## Safety Guarantees

1. **UTF-8 Validation** - `to_string()` validates UTF-8 and returns `Result`
2. **Copy Safety** - `to_string()` and `to_bytes()` copy data, safe from GC
3. **Type Checking** - `TryConvert` ensures type safety at boundaries
4. **Encoding Awareness** - Explicit encoding operations prevent encoding bugs

## Common Pitfalls

### ❌ Assuming all Ruby strings are UTF-8

```rust
// Wrong: Panics if string contains invalid UTF-8
let s = rstring.to_string().unwrap();
```

```rust
// Right: Handle encoding issues
match rstring.to_string() {
    Ok(s) => process_text(s),
    Err(_) => process_binary(rstring.to_bytes()),
}
```

### ❌ Using as_slice() without understanding safety

```rust
// Wrong: Slice may be invalidated
let slice = unsafe { rstring.as_slice() };
do_ruby_stuff(); // Could invalidate slice!
process_slice(slice); // Use-after-free!
```

```rust
// Right: Copy the data if you need to call Ruby
let bytes = rstring.to_bytes();
do_ruby_stuff(); // Safe, we own the data
process_bytes(&bytes);
```

## Next Steps

Future stages will add:

- **Stage 5**: Array type with iteration and element access
- **Stage 6**: Hash type with key-value operations
- **Stage 7**: Class and Module types for object-oriented programming

## Related Documentation

- [Phase 2 Tasks](../../docs/plan/phase-2-tasks.md) - Detailed implementation plan
- [Phase 2 Types](../../docs/plan/phase-2-types.md) - Type system design
- [RString Implementation](../../crates/solidus/src/types/string.rs) - Source code
- [Solidus README](../../README.md) - Project overview
