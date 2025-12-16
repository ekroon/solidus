# Phase 2 Stage 6: Hash Type Example

This example demonstrates Ruby's Hash type in Solidus with complete key-value operations, iteration, and conversions to/from Rust HashMap.

## Overview

Stage 6 of Phase 2 implements the **RHash** type, which wraps Ruby's Hash class. Hashes are dynamic key-value stores that can contain any Ruby values as keys and values. Like strings and arrays, hashes are heap-allocated objects that require GC protection when used in method signatures (Phase 3+).

### Key Features

- **Dynamic key-value storage** - Store any Ruby values as keys and values
- **Multiple key types** - String, Symbol, Integer, or any Ruby object
- **Efficient operations** - insert(), get(), delete() in O(1) expected time
- **Safe iteration** - Closure-based iteration (not Iterator trait)
- **Type-safe conversions** - Convert to/from Rust HashMap<K, V>
- **Nested hashes** - Hashes can contain other hashes as values
- **Option-based lookup** - get() returns Option<Value> for safety

## What This Example Demonstrates

1. **Hash Creation** - Creating empty hashes with new()
2. **Insertion** - Adding key-value pairs with insert()
3. **Lookup** - Getting values by key with get()
4. **Updates** - Updating existing keys (insert overwrites)
5. **Deletion** - Removing keys with delete()
6. **Iteration** - Looping over entries with each()
7. **String Keys** - Using string literals as keys
8. **Symbol Keys** - Using Ruby symbols as keys
9. **Integer Keys** - Using numbers as keys
10. **Mixed Keys** - Different key types in one hash
11. **HashMap Conversion** - From Rust HashMap to Ruby Hash
12. **HashMap Extraction** - From Ruby Hash to Rust HashMap
13. **Nested Hashes** - Hashes containing hashes
14. **Type Safety** - Compile-time type checking
15. **Round-trip Conversion** - Safe bidirectional conversion

## Ruby Hash Basics

Ruby hashes are **unordered** key-value collections (though Ruby 1.9+ preserves insertion order). Any Ruby object can be a key or value, and keys are compared using `eql?` and `hash` methods.

### Common Key Types

| Key Type | Example | Use Case |
|----------|---------|----------|
| String | `"name"` | General-purpose keys |
| Symbol | `:name` | Performance-critical keys (Ruby idiom) |
| Integer | `1`, `42` | Numeric indices, IDs |
| Mixed | Various | Dynamic structures |

### Hash Operations

```rust
// Create a new hash
let hash = RHash::new();

// Insert key-value pairs
hash.insert("name", "Alice");
hash.insert("age", 30i64);

// Get values
if let Some(val) = hash.get("name") {
    let name = String::try_convert(val)?;
}

// Delete keys
let removed = hash.delete("age");

// Iterate
hash.each(|key, val| {
    // Process each pair
    Ok(())
})?;

// Check size
assert_eq!(hash.len(), 1);
assert!(!hash.is_empty());
```

## Hash API

### Construction

| Method | Description |
|--------|-------------|
| `RHash::new()` | Create a new empty hash |
| `RHash::default()` | Same as new() |
| `RHash::from_hash_map(map)` | Create from Rust HashMap |

### Properties

| Method | Description |
|--------|-------------|
| `len(self) -> usize` | Get number of key-value pairs |
| `is_empty(self) -> bool` | Check if hash is empty |

### Key-Value Operations

| Method | Description |
|--------|-------------|
| `insert<K, V>(self, key: K, value: V)` | Insert or update a key-value pair |
| `get<K>(self, key: K) -> Option<Value>` | Get value by key (None if missing) |
| `delete<K>(self, key: K) -> Option<Value>` | Delete key and return value |

### Iteration

| Method | Description |
|--------|-------------|
| `each<F>(self, f: F) -> Result<(), Error>` | Iterate over key-value pairs |

### Conversions

| Method | Description |
|--------|-------------|
| `to_hash_map<K, V>(self) -> Result<HashMap<K, V>, Error>` | Convert to Rust HashMap |

## Key Types in Depth

### String Keys

The most common key type in Ruby hashes:

```rust
let hash = RHash::new();
hash.insert("name", "Alice");
hash.insert("email", "alice@example.com");

let name = hash.get("name").unwrap();
let email = hash.get("email").unwrap();
```

**Pros:** Intuitive, flexible, human-readable
**Cons:** Slightly slower than symbols, creates new objects

### Symbol Keys

Ruby symbols are interned strings (only one copy exists per symbol):

```rust
let hash = RHash::new();
let name_key = Symbol::new("name");
let age_key = Symbol::new("age");

hash.insert(name_key, "Bob");
hash.insert(age_key, 25i64);

let name = hash.get(name_key).unwrap();
```

**Pros:** Memory-efficient, faster equality checks, Ruby idiom
**Cons:** Never garbage collected (use with finite set of keys)

### Integer Keys

Use integers when you need numeric indices:

```rust
let hash = RHash::new();
hash.insert(1i64, "first");
hash.insert(2i64, "second");
hash.insert(100i64, "hundredth");

let val = hash.get(2i64).unwrap();
```

**Pros:** Fast, compact, natural for IDs
**Cons:** Different semantics from Arrays (not ordered, sparse)

### Mixed Key Types

Ruby allows different key types in the same hash:

```rust
let hash = RHash::new();
hash.insert("string_key", 1i64);
hash.insert(Symbol::new("symbol_key"), 2i64);
hash.insert(100i64, 3i64);

// Each key type is distinct
assert_eq!(hash.len(), 3);
```

**Important:** Keys are compared by value and type. `"key"` and `Symbol::new("key")` are different keys!

## Iteration

### Why Not Iterator Trait?

We don't implement Rust's `Iterator` trait because it would be unsafe. Between iterator calls, Ruby code could run (if the closure calls back into Ruby), potentially triggering GC which could modify or move the hash. By using a closure with `each()`, we maintain control over when Ruby code can execute.

### Iteration Examples

```rust
// Sum all values
let mut sum = 0i64;
hash.each(|_key, val| {
    let n = i64::try_convert(val)?;
    sum += n;
    Ok(())
})?;

// Collect all keys
let mut keys = Vec::new();
hash.each(|key, _val| {
    let s = RString::try_convert(key)?;
    keys.push(s.to_string()?);
    Ok(())
})?;

// Filter values
let mut filtered = RHash::new();
hash.each(|key, val| {
    let n = i64::try_convert(val)?;
    if n > 10 {
        filtered.insert(key, val);
    }
    Ok(())
})?;
```

### Early Termination

Return an `Err` to stop iteration:

```rust
hash.each(|key, val| {
    if some_condition {
        return Err(Error::type_error("stop"));
    }
    Ok(())
})?;
```

## HashMap Conversions

### Ruby Hash → Rust HashMap

Convert a Ruby hash to a Rust HashMap with typed keys and values:

```rust
let hash = RHash::new();
hash.insert("x", 10i64);
hash.insert("y", 20i64);

// Specify key and value types
let map: HashMap<String, i64> = hash.to_hash_map()?;

assert_eq!(map.get("x"), Some(&10));
assert_eq!(map.get("y"), Some(&20));
```

**Type Conversion:** Both keys and values are converted using `TryConvert`. If any element fails to convert, the entire operation returns an error.

### Rust HashMap → Ruby Hash

Convert a Rust HashMap to a Ruby hash:

```rust
let mut map = HashMap::new();
map.insert("red", 255i64);
map.insert("green", 128i64);
map.insert("blue", 0i64);

let hash = RHash::from_hash_map(map);

assert_eq!(hash.len(), 3);
let red = i64::try_convert(hash.get("red").unwrap())?;
```

**Type Conversion:** Keys and values are converted using `IntoValue`. This always succeeds for types that implement `IntoValue`.

### Round-trip Conversion

You can safely convert back and forth:

```rust
// Start with HashMap
let mut original = HashMap::new();
original.insert("key", 42i64);

// To Ruby
let ruby_hash = RHash::from_hash_map(original.clone());

// Back to Rust
let roundtrip: HashMap<String, i64> = ruby_hash.to_hash_map()?;

assert_eq!(roundtrip, original);
```

## Nested Hashes

Hashes can contain other hashes as values:

```rust
// Create nested structure
let address = RHash::new();
address.insert("city", "Portland");
address.insert("state", "Oregon");
address.insert("zip", 97201i64);

let person = RHash::new();
person.insert("name", "Alice");
person.insert("address", address);

// Access nested data
let addr_val = person.get("address").unwrap();
let addr = RHash::try_convert(addr_val)?;
let city_val = addr.get("city").unwrap();
let city = RString::try_convert(city_val)?;
```

This enables building complex data structures entirely in Ruby.

## Error Handling

### Missing Keys

`get()` returns `Option<Value>`, so missing keys are handled gracefully:

```rust
match hash.get("missing_key") {
    Some(val) => {
        // Key exists, process value
        process_value(val);
    }
    None => {
        // Key doesn't exist, handle accordingly
        use_default_value();
    }
}
```

### Type Mismatches

Type conversion can fail if the value isn't the expected type:

```rust
// Insert a string
hash.insert("key", "not a number");

// Try to convert to integer
match hash.get("key") {
    Some(val) => {
        match i64::try_convert(val) {
            Ok(n) => println!("Got integer: {}", n),
            Err(_) => println!("Value is not an integer"),
        }
    }
    None => println!("Key not found"),
}
```

### Iteration Errors

Errors during iteration stop the iteration and propagate:

```rust
let result = hash.each(|key, val| {
    let n = i64::try_convert(val)?; // Fails if not an integer
    process_number(n);
    Ok(())
});

match result {
    Ok(()) => println!("All values processed"),
    Err(e) => println!("Error during iteration: {:?}", e),
}
```

## Type Safety

### Compile-time Guarantees

The type system prevents common mistakes:

```rust
// ✓ This compiles - valid key types
hash.insert("string", 1i64);
hash.insert(Symbol::new("symbol"), 2i64);
hash.insert(42i64, 3i64);

// ✓ This compiles - any IntoValue works
hash.insert("key", "string value");
hash.insert("key", 123i64);
hash.insert("key", true);
hash.insert("key", other_hash);

// ❌ This won't compile - not IntoValue
// hash.insert("key", some_rust_struct);
```

### Runtime Type Checking

Use `TryConvert` to safely handle dynamic types:

```rust
fn process_hash_value(hash: RHash, key: &str) -> Result<String, Error> {
    let val = hash.get(key)
        .ok_or_else(|| Error::type_error("key not found"))?;
    
    // Try to convert to string
    let s = RString::try_convert(val)?;
    s.to_string()
}
```

## Performance Considerations

### Hash Complexity

- `insert()` - O(1) expected, O(n) worst case
- `get()` - O(1) expected, O(n) worst case
- `delete()` - O(1) expected, O(n) worst case
- `len()` - O(1)
- `each()` - O(n)

### Key Type Performance

| Key Type | Equality Check | Hash Computation | Memory |
|----------|----------------|------------------|---------|
| Integer | Very fast | Very fast | 8 bytes |
| Symbol | Fast (pointer) | Fast (cached) | 8 bytes |
| String | Slow (content) | Slow (content) | Variable |

**Recommendation:** Use symbols for performance-critical code with a fixed set of keys.

### Conversion Costs

- `from_hash_map()` - O(n) insertions
- `to_hash_map()` - O(n) conversions and iterations
- `each()` with closure - O(n) but avoids intermediate allocations

## Building and Running

Build the example:

```bash
cargo build --release --manifest-path examples/phase2_hash/Cargo.toml
```

Run the Ruby test script:

```bash
ruby examples/phase2_hash/test.rb
```

Run just the Rust tests:

```bash
cargo test --manifest-path examples/phase2_hash/Cargo.toml
```

## Code Structure

- `src/lib.rs` - 15 example functions demonstrating hash operations
- `test.rb` - Ruby script that builds and verifies the extension
- `Cargo.toml` - Build configuration with dependencies
- `build.rs` - Ruby integration build script
- `.gitignore` - Ignore build artifacts

## Example Functions Reference

| Function | Demonstrates |
|----------|-------------|
| `example_hash_new()` | Creating empty hashes |
| `example_hash_insert()` | Inserting key-value pairs |
| `example_hash_get()` | Getting values by key |
| `example_hash_update()` | Updating existing keys |
| `example_hash_delete()` | Deleting keys |
| `example_hash_iteration()` | Iterating with each() |
| `example_hash_symbol_keys()` | Using Symbol keys |
| `example_hash_integer_keys()` | Using Integer keys |
| `example_hash_mixed_keys()` | Mixed key types |
| `example_hash_from_hashmap()` | Converting from HashMap |
| `example_hash_to_hashmap()` | Converting to HashMap |
| `example_hash_nested()` | Nested hash structures |
| `example_hash_type_safe()` | Type-safe operations |
| `example_hash_roundtrip()` | Round-trip conversion |
| `example_hash_collect_keys()` | Collecting keys |

## Common Patterns

### Configuration Hash

```rust
fn create_config() -> RHash {
    let config = RHash::new();
    config.insert("host", "localhost");
    config.insert("port", 3000i64);
    config.insert("debug", true);
    config
}
```

### Counting with Hash

```rust
let counts = RHash::new();
for item in items {
    let key = item.name();
    let current = counts.get(key)
        .and_then(|v| i64::try_convert(v).ok())
        .unwrap_or(0);
    counts.insert(key, current + 1);
}
```

### Grouping with Hash

```rust
let groups = RHash::new();
items.each(|_idx, item| {
    let category = get_category(item)?;
    let group = groups.get(category)
        .map(|v| RArray::try_convert(v))
        .transpose()?
        .unwrap_or_else(RArray::new);
    group.push(item);
    groups.insert(category, group);
    Ok(())
})?;
```

## Comparison with Magnus

| Feature | Solidus | Magnus |
|---------|---------|--------|
| Hash creation | `RHash::new()` | `RHash::new()` |
| Insert | `insert(k, v)` | `aset(k, v)` |
| Get | `get(k) -> Option` | `get(k) -> Result` |
| Delete | `delete(k) -> Option` | `delete(k)` |
| Iteration | `each(closure)` | `foreach()` |
| HashMap conversion | Full support | Limited |
| Type safety | Compile-time | Runtime |

## Safety Guarantees

1. **Option for Missing Keys** - `get()` returns `Option<Value>`, no panics
2. **Type Safety** - `TryConvert` ensures type checking at boundaries  
3. **Safe Iteration** - Closure-based, no invalid iterators
4. **GC Safety** - Hash is always valid (heap-allocated with GC tracking)

## Common Pitfalls

### ❌ Confusing missing keys with nil values

```rust
// Wrong: Can't distinguish missing key from nil value
if hash.get("key").is_none() {
    // Could be missing OR could be nil value
}
```

```rust
// Right: Use Ruby's has_key? if you need to distinguish
// (Requires Phase 3 method calling)
```

### ❌ Assuming iteration order

```rust
// Wrong: Expecting specific order
let keys = collect_keys(&hash);
assert_eq!(keys[0], "first_inserted"); // May fail!
```

```rust
// Right: Don't depend on order
let mut keys = collect_keys(&hash);
keys.sort(); // Sort if order matters
```

### ❌ Modifying hash during iteration

```rust
// Wrong: Modifying while iterating (undefined behavior)
hash.each(|key, val| {
    hash.delete(key); // DON'T DO THIS
    Ok(())
})?;
```

```rust
// Right: Collect keys first, then modify
let mut to_delete = Vec::new();
hash.each(|key, _val| {
    to_delete.push(key);
    Ok(())
})?;
for key in to_delete {
    hash.delete(key);
}
```

### ❌ Using strings instead of symbols

```rust
// Less efficient: New string objects
hash.insert("status", "active");
hash.insert("priority", "high");
hash.get("status");
```

```rust
// More efficient: Interned symbols
hash.insert(Symbol::new("status"), "active");
hash.insert(Symbol::new("priority"), "high");
hash.get(Symbol::new("status"));
```

## Next Steps

Future stages and phases will add:

- **Phase 3**: Method definition and registration
- **Phase 3**: Calling Ruby methods on hash values
- **Phase 4**: Custom Ruby classes with hash support
- **Phase 5**: Advanced features like hash default values

## Related Documentation

- [Phase 2 Tasks](../../docs/plan/phase-2-tasks.md) - Detailed implementation plan
- [Phase 2 Types](../../docs/plan/phase-2-types.md) - Type system design
- [RHash Implementation](../../crates/solidus/src/types/hash.rs) - Source code
- [Array Example](../phase2_array/) - Similar collection type
- [Solidus README](../../README.md) - Project overview
