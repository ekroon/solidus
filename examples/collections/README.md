# Collections Example

This example demonstrates working with Ruby collections (RArray and RHash) in Solidus, including iteration, building, and converting between Rust and Ruby types.

## Overview

Collections are fundamental to any programming task. This example shows how to:

1. **Iterate** over arrays and hashes using `each()`
2. **Build** collections incrementally with `push()` and `insert()`
3. **Convert** between Rust and Ruby collection types

## What This Example Demonstrates

### Part 1: Working with Arrays

| Function | Pattern | Description |
|----------|---------|-------------|
| `build_array()` | Building | Create empty array, populate with `push()` |
| `iterate_array_sum()` | Iteration | Sum elements using `each()` closure |
| `filter_array_even()` | Filtering | Build new array with matching elements |
| `vec_to_array()` | Rust→Ruby | Convert `Vec<i64>` to `RArray` |
| `array_to_vec()` | Ruby→Rust | Convert `RArray` to `Vec<i64>` |
| `map_array_double()` | Transform | Create array with transformed elements |

### Part 2: Working with Hashes

| Function | Pattern | Description |
|----------|---------|-------------|
| `build_hash()` | Building | Create empty hash, populate with `insert()` |
| `iterate_hash_entries()` | Iteration | Iterate key-value pairs with `each()` |
| `hashmap_to_rhash()` | Rust→Ruby | Convert `HashMap` to `RHash` |
| `rhash_to_hashmap()` | Ruby→Rust | Convert `RHash` to `HashMap` |
| `filter_hash_by_value()` | Filtering | Build new hash with matching entries |

### Part 3: Combining Arrays and Hashes

| Function | Pattern | Description |
|----------|---------|-------------|
| `array_of_hashes()` | Nesting | Store hashes in an array |
| `hash_with_array_values()` | Nesting | Store arrays as hash values |
| `group_by_length()` | Grouping | Group array items by computed key |
| `flatten_hash_arrays()` | Flattening | Combine nested arrays into one |

### Part 4: Round-trip Conversions

| Function | Pattern | Description |
|----------|---------|-------------|
| `roundtrip_vec()` | Conversion | `Vec` → `RArray` → `Vec` |
| `roundtrip_hashmap()` | Conversion | `HashMap` → `RHash` → `HashMap` |

## Collection Patterns

### Iterating with each()

Both `RArray` and `RHash` use closure-based iteration for safety:

```rust
// Array iteration
let arr = RArray::from_slice(&[1i64, 2, 3, 4, 5]);
let mut sum = 0i64;
arr.each(|val| {
    let n = i64::try_convert(val)?;
    sum += n;
    Ok(())
})?;

// Hash iteration
let hash = RHash::new();
hash.insert("a", 10i64);
hash.insert("b", 20i64);
hash.each(|key, val| {
    let k = RString::try_convert(key)?;
    let v = i64::try_convert(val)?;
    println!("{}: {}", k.to_string()?, v);
    Ok(())
})?;
```

### Building Collections

```rust
// Building an array
let arr = RArray::new();
arr.push(10i64);
arr.push(20i64);
arr.push(30i64);

// Building a hash
let hash = RHash::new();
hash.insert("name", "Alice");
hash.insert("age", 30i64);
hash.insert("active", true);
```

### Filtering Collections

```rust
// Filter array to new array
let arr = RArray::from_slice(&[1i64, 2, 3, 4, 5, 6]);
let evens = RArray::new();
arr.each(|val| {
    let n = i64::try_convert(val)?;
    if n % 2 == 0 {
        evens.push(n);
    }
    Ok(())
})?;

// Filter hash to new hash  
let hash = RHash::new();
hash.insert("small", 5i64);
hash.insert("large", 25i64);
let filtered = RHash::new();
hash.each(|key, val| {
    let n = i64::try_convert(val)?;
    if n > 10 {
        filtered.insert(key, val);
    }
    Ok(())
})?;
```

### Rust ↔ Ruby Conversions

```rust
// Vec<T> → RArray
let vec = vec![1i64, 2, 3, 4, 5];
let arr = RArray::from_slice(&vec);

// RArray → Vec<T>
let vec: Vec<i64> = arr.to_vec()?;

// HashMap<K, V> → RHash
let mut map = HashMap::new();
map.insert("key", 42i64);
let hash = RHash::from_hash_map(map);

// RHash → HashMap<K, V>
let map: HashMap<String, i64> = hash.to_hash_map()?;
```

### Nesting Collections

```rust
// Array of hashes (common for record sets)
let users = RArray::new();
let user1 = RHash::new();
user1.insert("name", "Alice");
user1.insert("age", 30i64);
users.push(user1);

// Hash with array values (grouping)
let groups = RHash::new();
groups.insert("admins", RArray::from_slice(&["alice", "bob"]));
groups.insert("users", RArray::from_slice(&["charlie", "dave"]));
```

### Grouping Pattern

A common pattern is grouping array elements by a computed key:

```rust
let words = RArray::from_slice(&["a", "to", "cat", "word"]);
let grouped = RHash::new();

words.each(|val| {
    let s = RString::try_convert(val)?;
    let len = s.to_string()?.len() as i64;
    
    // Get or create the group array
    let group = match grouped.get(len) {
        Some(existing) => RArray::try_convert(existing)?,
        None => {
            let new_group = RArray::new();
            grouped.insert(len, new_group);
            new_group
        }
    };
    
    group.push(val);
    Ok(())
})?;
// Result: {1 => ["a"], 2 => ["to"], 3 => ["cat"], 4 => ["word"]}
```

## Why Closure-Based Iteration?

We use closures instead of implementing Rust's `Iterator` trait for safety:

```rust
// UNSAFE with Iterator trait:
let mut iter = arr.iter();  // Hypothetical
let first = iter.next();
ruby_function();  // GC could run here and invalidate the iterator!
let second = iter.next();  // Dangling pointer!

// SAFE with closures:
arr.each(|val| {
    // We control when Ruby code can run
    Ok(())
})?;
```

If you need Rust iterator methods, convert to `Vec` first:

```rust
let vec: Vec<i64> = arr.to_vec()?;
let sum: i64 = vec.iter().sum();
let doubled: Vec<i64> = vec.iter().map(|x| x * 2).collect();
```

## Type Safety

All conversions are type-safe:

```rust
// This will fail if array contains non-integers
let result: Result<Vec<i64>, Error> = arr.to_vec();

// Handle mixed types by filtering
let mut numbers = Vec::new();
arr.each(|val| {
    if let Ok(n) = i64::try_convert(val) {
        numbers.push(n);
    }
    Ok(())
})?;
```

## Performance Tips

1. **Pre-allocate capacity** when size is known:
   ```rust
   let arr = RArray::with_capacity(1000);
   ```

2. **Use `each()` over `to_vec()`** when you don't need the Vec:
   ```rust
   // More efficient - no intermediate allocation
   let mut sum = 0i64;
   arr.each(|val| {
       sum += i64::try_convert(val)?;
       Ok(())
   })?;
   ```

3. **Use symbols for hash keys** in performance-critical code:
   ```rust
   // Symbols are interned - faster equality checks
   hash.insert(Symbol::new("key"), value);
   ```

## Building and Running

Build the example:

```bash
cargo build --release --manifest-path examples/collections/Cargo.toml
```

Run the Ruby test script:

```bash
ruby examples/collections/test.rb
```

Run Rust tests:

```bash
cargo test --manifest-path examples/collections/Cargo.toml
```

## Code Structure

- `src/lib.rs` - 17 example functions demonstrating collection patterns
- `test.rb` - Ruby script that builds and verifies the extension
- `Cargo.toml` - Build configuration
- `build.rs` - Ruby integration build script
- `.gitignore` - Ignore build artifacts

## API Summary

### RArray

| Method | Description |
|--------|-------------|
| `RArray::new()` | Create empty array |
| `RArray::with_capacity(n)` | Create with pre-allocated capacity |
| `RArray::from_slice(&[T])` | Create from Rust slice |
| `push(val)` | Add element to end |
| `entry(index)` | Get element by index |
| `each(\|val\| ...)` | Iterate with closure |
| `to_vec::<T>()` | Convert to Rust `Vec<T>` |
| `len()` | Get number of elements |

### RHash

| Method | Description |
|--------|-------------|
| `RHash::new()` | Create empty hash |
| `RHash::from_hash_map(map)` | Create from Rust HashMap |
| `insert(key, val)` | Insert or update key-value pair |
| `get(key)` | Get value by key (`Option<Value>`) |
| `delete(key)` | Remove key and return value |
| `each(\|key, val\| ...)` | Iterate with closure |
| `to_hash_map::<K, V>()` | Convert to Rust `HashMap<K, V>` |
| `len()` | Get number of entries |

## Related Examples

- [phase2_array](../phase2_array/) - Detailed RArray API coverage
- [phase2_hash](../phase2_hash/) - Detailed RHash API coverage
- [phase2_conversions](../phase2_conversions/) - Type conversion patterns

## Related Documentation

- [Phase 2 Tasks](../../docs/plan/phase-2-tasks.md) - Implementation plan
- [RArray Source](../../crates/solidus/src/types/array.rs)
- [RHash Source](../../crates/solidus/src/types/hash.rs)
