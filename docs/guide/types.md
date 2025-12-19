# Working with Ruby Types

This guide covers how to work with Ruby types in Solidus, including the distinction
between immediate and heap types, type conversions, and practical usage patterns.

## Type Categories

Ruby values fall into two categories based on how they're stored:

### Immediate Values

Immediate values are encoded directly in the VALUE pointer itself. They don't
require heap allocation or GC protection, making them safe to pass around freely.

| Type | Description | Rust Type |
|------|-------------|-----------|
| `nil` | Ruby's null value | `Qnil` |
| `true` | Boolean true | `Qtrue` |
| `false` | Boolean false | `Qfalse` |
| Fixnum | Small integers (~±2^62) | `Fixnum` |
| Symbol | Interned strings | `Symbol` |
| Flonum | Small floats (64-bit only) | `Flonum` |

Immediate types are `Copy` in Rust because they don't reference heap memory:

```rust
use solidus::types::{Fixnum, Symbol, Qnil};

let num = Fixnum::from_i64(42).unwrap();
let sym = Symbol::new("hello");
let nil = Qnil::new();

// These can be freely copied
let num2 = num;
let sym2 = sym;
```

### Heap Values

Heap values are allocated on Ruby's heap and managed by the garbage collector.
They must be either stack-pinned or explicitly boxed to prevent GC issues.

| Type | Description | Rust Type |
|------|-------------|-----------|
| String | Mutable byte sequences | `RString` |
| Array | Dynamic arrays | `RArray` |
| Hash | Key-value maps | `RHash` |
| Bignum | Large integers | `RBignum` |
| Float | Heap-allocated floats | `RFloat` |
| Class | Ruby class objects | `RClass` |
| Module | Ruby module objects | `RModule` |

Heap types return `PinGuard<T>` from constructors, enforcing proper handling:

```rust
use solidus::types::RString;
use solidus::pin_on_stack;

// Creating a string returns PinGuard<RString>
let guard = RString::new("hello");

// Pin on stack for local use
pin_on_stack!(s = guard);
println!("Length: {}", s.get().len());
```

## Immediate Types

### Boolean Values (Qnil, Qtrue, Qfalse)

Ruby's singleton boolean values:

```rust
use solidus::types::{Qnil, Qtrue, Qfalse};
use solidus::convert::IntoValue;

// Create singletons
let nil = Qnil::new();
let t = Qtrue::new();
let f = Qfalse::new();

// Convert Rust bool to Ruby
let ruby_true = true.into_value();   // Qtrue
let ruby_false = false.into_value(); // Qfalse
```

Ruby's truthiness differs from Rust: only `nil` and `false` are falsy:

```rust
use solidus::convert::TryConvert;

// Ruby truthiness: only nil and false are falsy
let is_truthy = bool::try_convert(some_value)?;
```

### Fixnum (Small Integers)

Fixnum represents small integers that fit in the VALUE encoding:

```rust
use solidus::types::Fixnum;

// Create a Fixnum (returns None if too large)
let num = Fixnum::from_i64(42).expect("fits in Fixnum");
let value = num.to_i64();  // 42

// Very large values won't fit
assert!(Fixnum::from_i64(i64::MAX).is_none());
```

For general integer handling, use `Integer` which handles both Fixnum and Bignum:

```rust
use solidus::types::Integer;

let small = Integer::from_i64(42);      // Fixnum internally
let large = Integer::from_u64(u64::MAX); // Bignum internally

// Both work the same way
let n = small.to_i64()?;
```

### Symbol

Symbols are interned strings, commonly used as hash keys:

```rust
use solidus::types::Symbol;

let sym1 = Symbol::new("hello");
let sym2 = Symbol::new("hello");

// Same string = same symbol (interned)
assert_eq!(sym1.as_value(), sym2.as_value());

// Get the symbol name
let name = sym1.name()?;  // "hello"
```

### Flonum (64-bit only)

On 64-bit platforms, small floats can be immediate values:

```rust
#[cfg(target_pointer_width = "64")]
use solidus::types::Flonum;

// Some floats fit in immediate encoding
if let Some(flonum) = Flonum::from_f64(1.5) {
    let value = flonum.to_f64();  // 1.5
}
```

For general float handling, use `Float` which handles both Flonum and RFloat:

```rust
use solidus::types::Float;

let f = Float::from_f64(3.14159);
let value = f.to_f64();
```

## Heap Types

### RString

Ruby strings are mutable byte sequences with encoding support:

```rust
use solidus::types::{RString, Encoding};
use solidus::pin_on_stack;

// Create from &str
pin_on_stack!(s = RString::new("Hello, world!"));

// Basic operations
let len = s.get().len();           // 13 (bytes)
let is_empty = s.get().is_empty(); // false

// Convert to Rust String (copies data)
let rust_string = s.get().to_string()?;  // "Hello, world!"

// Get raw bytes
let bytes = s.get().to_bytes();  // Vec<u8>
```

Creating strings from binary data:

```rust
// Binary data with null bytes
let bytes = b"binary\x00data";
pin_on_stack!(s = RString::from_slice(bytes));

// Length includes null byte
assert_eq!(s.get().len(), 11);

// to_string() fails for invalid UTF-8, to_bytes() always works
let bytes_back = s.get().to_bytes();
```

Working with encodings:

```rust
use solidus::types::Encoding;

// Get standard encodings
let utf8 = Encoding::utf8();
let binary = Encoding::ascii_8bit();
let ascii = Encoding::us_ascii();

// Find encoding by name
let latin1 = Encoding::find("ISO-8859-1").unwrap();

// Check string encoding
let enc = s.get().encoding();
println!("Encoding: {}", enc.name());

// Convert encoding
let encoded = s.get().encode(utf8)?;
```

### RArray

Ruby arrays are dynamic, heterogeneous collections:

```rust
use solidus::types::RArray;
use solidus::pin_on_stack;
use solidus::convert::TryConvert;

// Create empty array
pin_on_stack!(arr = RArray::new());

// Create with capacity (performance optimization)
pin_on_stack!(arr = RArray::with_capacity(100));

// Push elements
arr.get().push(42i64);
arr.get().push("hello");
arr.get().push(true);

// Access by index
let first = arr.get().entry(0);      // 42
let last = arr.get().entry(-1);      // true (negative = from end)
let oob = arr.get().entry(100);      // nil (out of bounds)

// Store at index
arr.get().store(0, 99i64);           // Replace first
arr.get().store(10, "gap");          // Extends with nils

// Pop last element
if let Some(val) = arr.get().pop() {
    let b = bool::try_convert(val)?;
}
```

Iterating over arrays:

```rust
// Use each() with a closure (safer than Iterator)
let mut sum = 0i64;
arr.get().each(|val| {
    let n = i64::try_convert(val)?;
    sum += n;
    Ok(())
})?;
```

Converting to/from Rust collections:

```rust
// From slice
pin_on_stack!(arr = RArray::from_slice(&[1i64, 2, 3, 4, 5]));

// To Vec (with type conversion)
let vec: Vec<i64> = arr.get().to_vec()?;

// Vec<T> also implements IntoValue/TryConvert
let ruby_arr = vec![1i64, 2, 3].into_value();
let rust_vec: Vec<i64> = Vec::try_convert(ruby_arr)?;
```

### RHash

Ruby hashes are key-value stores supporting any Ruby type as keys:

```rust
use solidus::types::{RHash, Symbol};
use solidus::pin_on_stack;
use solidus::convert::TryConvert;

// Create empty hash
pin_on_stack!(hash = RHash::new());

// Insert with string keys
hash.get().insert("name", "Alice");
hash.get().insert("age", 30i64);

// Insert with symbol keys (common in Ruby)
hash.get().insert(Symbol::new("active"), true);

// Insert with integer keys
hash.get().insert(1i64, "first");

// Get values
if let Some(val) = hash.get().get("name") {
    let name = String::try_convert(val)?;
}

// Delete and return value
if let Some(val) = hash.get().delete("age") {
    let age = i64::try_convert(val)?;
}

// Check size
let len = hash.get().len();
let empty = hash.get().is_empty();
```

Iterating over hashes:

```rust
// Iterate key-value pairs
let mut sum = 0i64;
hash.get().each(|key, val| {
    if let Ok(n) = i64::try_convert(val) {
        sum += n;
    }
    Ok(())
})?;
```

Converting to/from Rust HashMap:

```rust
use std::collections::HashMap;

// From HashMap
let mut map = HashMap::new();
map.insert("red", 255i64);
map.insert("green", 128i64);
pin_on_stack!(hash = RHash::from_hash_map(map));

// To HashMap
let rust_map: HashMap<String, i64> = hash.get().to_hash_map()?;

// HashMap<K, V> also implements IntoValue/TryConvert
let ruby_hash = rust_map.into_value();
```

### RClass and RModule

Classes and modules are first-class objects:

```rust
use solidus::types::{RClass, RModule, Module};

// Get built-in classes
let string_class = RClass::from_name("String").unwrap();
let array_class = RClass::from_name("Array").unwrap();

// Get class name
let name = string_class.name().unwrap();  // "String"

// Get superclass
let object_class = string_class.superclass().unwrap();

// Get modules
let enumerable = RModule::from_name("Enumerable").unwrap();
let kernel = RModule::from_name("Kernel").unwrap();

// Define constants (Module trait)
string_class.define_const("MY_VERSION", "1.0.0")?;
let version = string_class.const_get("MY_VERSION")?;
```

## Type Conversions

Solidus provides two traits for type conversion:

### TryConvert: Ruby to Rust

Convert Ruby values to Rust types (may fail):

```rust
use solidus::convert::TryConvert;
use solidus::value::Value;

fn process(val: Value) -> Result<(), Error> {
    // Convert to specific types
    let n: i64 = i64::try_convert(val)?;
    let s: String = String::try_convert(val)?;
    let arr: Vec<i64> = Vec::try_convert(val)?;
    
    // Convert to Ruby wrapper types
    let rstr = RString::try_convert(val)?;
    let rarr = RArray::try_convert(val)?;
    
    Ok(())
}
```

### IntoValue: Rust to Ruby

Convert Rust types to Ruby values (always succeeds):

```rust
use solidus::convert::IntoValue;

// Primitives
let int_val = 42i64.into_value();
let float_val = 3.14f64.into_value();
let bool_val = true.into_value();
let str_val = "hello".into_value();

// Collections
let vec_val = vec![1i64, 2, 3].into_value();
let map_val = HashMap::from([("a", 1)]).into_value();

// Ruby types
let rstr = RString::new("hello");
let val = rstr.into_value();
```

### Supported Conversions

| Rust Type | Ruby Type | Notes |
|-----------|-----------|-------|
| `i8`-`i64` | Integer | Range checked for smaller types |
| `u8`-`u64` | Integer | Range checked |
| `f32`, `f64` | Float | |
| `bool` | true/false | |
| `String`, `&str` | String | UTF-8 encoding |
| `Vec<T>` | Array | Where T: TryConvert/IntoValue |
| `HashMap<K, V>` | Hash | Where K, V: TryConvert/IntoValue |
| `()` | nil | Unit type returns nil |

## Numeric Types

Ruby has a unified `Integer` class but different internal representations:

### Integer Handling

```rust
use solidus::types::{Integer, Fixnum, RBignum};

// Use Integer for general handling
let int = Integer::from_i64(42);
let big = Integer::from_u64(u64::MAX);

// Convert to Rust types
let n: i64 = int.to_i64()?;
let m: u64 = big.to_u64()?;

// Integer conversions handle both Fixnum and Bignum
let val = some_ruby_value;
let n = i64::try_convert(val)?;  // Works for any Integer
```

### Float Handling

```rust
use solidus::types::{Float, RFloat};

// Use Float for general handling
let f = Float::from_f64(3.14159);
let value = f.to_f64();

// f64 conversions work with any Float
let n = f64::try_convert(some_value)?;
let ruby_float = 2.5f64.into_value();
```

## Best Practices

### 1. Use Appropriate Types

```rust
// For small integers, Fixnum is efficient
if let Some(num) = Fixnum::from_i64(count) {
    // Use num
}

// For general integers, use Integer or i64
let n: i64 = i64::try_convert(val)?;

// For hash keys, prefer Symbol
hash.insert(Symbol::new("key"), value);
```

### 2. Handle Type Errors

```rust
// Always handle conversion failures
match i64::try_convert(val) {
    Ok(n) => process_number(n),
    Err(e) => handle_error(e),
}

// Or use ? for propagation
let n: i64 = i64::try_convert(val)?;
```

### 3. Minimize Conversions

```rust
// Bad: Multiple conversions
let s = rstring.to_string()?;
let len = s.len();

// Good: Use Ruby API directly
let len = rstring.len();
```

### 4. Pin Heap Values Correctly

```rust
// Good: Pin for local use
pin_on_stack!(s = RString::new("hello"));
let len = s.get().len();

// Good: Convert immediately if just passing to Ruby
let val = RString::new("hello").into_value();
```

## Further Reading

- [Pinning](pinning.md) - Why Ruby values need pinning and how Solidus enforces it
- [BoxValue](boxvalue.md) - Storing Ruby values on the heap safely
- [Methods and Functions](methods.md) - Using types in method signatures
- [Examples](../../examples/) - Complete working examples
