# Phase 2 Stage 5: Array Type Example

This example demonstrates Ruby's Array type in Solidus with full iteration support, element access, and type-safe operations.

## Overview

Stage 5 of Phase 2 implements the **RArray** type, which wraps Ruby's Array class. Ruby arrays are dynamic, heterogeneous arrays that can contain any Ruby values. Unlike immediate values (Stage 2), arrays are heap-allocated objects that require GC protection when used in method signatures (Phase 3+).

### Key Features

- **Dynamic arrays** - Grow automatically as elements are added
- **Heterogeneous** - Can hold any mix of Ruby value types
- **Stack operations** - Push and pop for LIFO behavior
- **Flexible indexing** - Positive and negative indices (Ruby style)
- **Safe iteration** - Closure-based iteration with GC safety
- **Type-safe conversions** - Convert to/from Rust Vec with type checking
- **Nested arrays** - Support for multi-dimensional arrays
- **Bounds handling** - Out-of-bounds returns nil, store extends array

## What This Example Demonstrates

1. **Array Creation** - new(), with_capacity()
2. **Array Properties** - len(), is_empty()
3. **Stack Operations** - push(), pop()
4. **Element Access** - entry() with positive/negative indices
5. **Element Modification** - store() at any index
6. **Iteration** - each() with closures
7. **From Rust** - from_slice() to create from Rust data
8. **To Rust** - to_vec() to convert to Rust Vec
9. **Mixed Types** - Arrays with different value types
10. **Typed Arrays** - Type-safe homogeneous arrays
11. **Nested Arrays** - Multi-dimensional array structures
12. **Error Handling** - Type mismatches and bounds checking

## Ruby Array Basics

Ruby arrays are fundamentally different from Rust vectors:

| Feature | Ruby Array | Rust Vec |
|---------|-----------|----------|
| Element types | Heterogeneous (any type) | Homogeneous (single type) |
| Allocation | Heap (GC managed) | Heap (manual or RAII) |
| Indexing | Positive and negative | Positive only |
| Out of bounds | Returns nil | Panics or returns None |
| Extending | Automatic with nils | Manual with push/resize |

### Array Indexing

Ruby arrays support both positive and negative indices:

```rust
let arr = RArray::from_slice(&[10, 20, 30, 40, 50]);

// Positive indices (from start)
arr.entry(0);   // 10 (first element)
arr.entry(2);   // 30 (third element)
arr.entry(4);   // 50 (last element)

// Negative indices (from end)
arr.entry(-1);  // 50 (last element)
arr.entry(-2);  // 40 (second to last)
arr.entry(-5);  // 10 (first element)

// Out of bounds returns nil
arr.entry(100); // nil
arr.entry(-100); // nil
```

### Array Extension

Storing beyond the array length automatically extends it with `nil` values:

```rust
let arr = RArray::new();
arr.store(0, 1);   // [1]
arr.store(5, 99);  // [1, nil, nil, nil, nil, 99]
```

## RArray API Overview

### Construction

```rust
// Create empty array
let arr = RArray::new();

// Create with pre-allocated capacity (optimization)
let arr = RArray::with_capacity(100);

// Create from Rust slice
let arr = RArray::from_slice(&[1, 2, 3, 4, 5]);
```

### Properties

```rust
let len = arr.len();        // Number of elements
let empty = arr.is_empty(); // true if length is 0
```

### Stack Operations

```rust
// Add to end
arr.push(42);
arr.push("hello");

// Remove from end
if let Some(val) = arr.pop() {
    // Process value
}
```

### Element Access

```rust
// Get element by index (returns Value, nil if out of bounds)
let val = arr.entry(0);     // First element
let val = arr.entry(-1);    // Last element

// Store element at index (extends with nil if needed)
arr.store(0, 42);           // Replace first element
arr.store(-1, 99);          // Replace last element
arr.store(10, 123);         // Extends array to length 11
```

### Iteration

```rust
// Iterate with closure
arr.each(|val| {
    // Process each element
    // Return Ok(()) to continue or Err to stop
    Ok(())
})?;

// Sum all integers
let mut sum = 0i64;
arr.each(|val| {
    let n = i64::try_convert(val)?;
    sum += n;
    Ok(())
})?;
```

### Conversions

```rust
// Rust slice → Ruby array
let slice = &[1, 2, 3, 4, 5];
let arr = RArray::from_slice(slice);

// Ruby array → Rust Vec (type-safe)
let vec: Vec<i64> = arr.to_vec()?; // All elements must be i64

// Rust Vec → Ruby array (via IntoValue)
let vec = vec![1, 2, 3];
let val = vec.into_value();
```

## Code Examples

### Creating Arrays

```rust
// Empty array
let arr = RArray::new();
assert_eq!(arr.len(), 0);
assert!(arr.is_empty());

// Pre-allocated capacity
let arr = RArray::with_capacity(100);
for i in 0..100 {
    arr.push(i);
}

// From Rust slice
let numbers = &[1i64, 2, 3, 4, 5];
let arr = RArray::from_slice(numbers);
assert_eq!(arr.len(), 5);
```

### Stack Operations

```rust
let arr = RArray::new();

// Push elements
arr.push(10);
arr.push(20);
arr.push(30);
assert_eq!(arr.len(), 3);

// Pop elements
let val = arr.pop().unwrap();
assert_eq!(i64::try_convert(val).unwrap(), 30);
assert_eq!(arr.len(), 2);

// Pop from empty returns None
let empty = RArray::new();
assert!(empty.pop().is_none());
```

### Element Access

```rust
let arr = RArray::from_slice(&[100, 200, 300, 400, 500]);

// Positive indices
let val = arr.entry(0);
assert_eq!(i64::try_convert(val).unwrap(), 100);

let val = arr.entry(2);
assert_eq!(i64::try_convert(val).unwrap(), 300);

// Negative indices
let val = arr.entry(-1);
assert_eq!(i64::try_convert(val).unwrap(), 500);

let val = arr.entry(-2);
assert_eq!(i64::try_convert(val).unwrap(), 400);

// Out of bounds
let val = arr.entry(100);
assert!(val.is_nil());
```

### Storing Elements

```rust
let arr = RArray::new();

// Store at index 0
arr.store(0, 42);
assert_eq!(arr.len(), 1);

// Replace existing element
arr.store(0, 99);
let val = arr.entry(0);
assert_eq!(i64::try_convert(val).unwrap(), 99);

// Store beyond length extends with nils
arr.store(5, 123);
assert_eq!(arr.len(), 6);

// Elements 1-4 are nil
for i in 1..5 {
    assert!(arr.entry(i).is_nil());
}
```

### Iterating Over Arrays

```rust
let arr = RArray::from_slice(&[1, 2, 3, 4, 5]);

// Sum all elements
let mut sum = 0i64;
arr.each(|val| {
    let n = i64::try_convert(val)?;
    sum += n;
    Ok(())
})?;
assert_eq!(sum, 15);

// Count elements
let mut count = 0;
arr.each(|_| {
    count += 1;
    Ok(())
})?;
assert_eq!(count, 5);

// Early termination on error
arr.each(|val| {
    if some_condition {
        return Err(Error::type_error("stopped"));
    }
    Ok(())
})?;
```

### Type-Safe Conversions

```rust
// Ruby array → Rust Vec
let arr = RArray::from_slice(&[1i64, 2, 3, 4, 5]);
let vec: Vec<i64> = arr.to_vec()?;
assert_eq!(vec, vec![1, 2, 3, 4, 5]);

// Rust Vec → Ruby array
let vec = vec![10i64, 20, 30];
let val = vec.into_value();
let arr = RArray::try_convert(val)?;

// Rust slice → Ruby array
let slice: &[i64] = &[1, 2, 3];
let arr = RArray::from_slice(slice);
```

### Mixed-Type Arrays

```rust
// Ruby arrays can hold different types
let arr = RArray::new();
arr.push(42i64);
arr.push(RString::new("hello"));
arr.push(true);
arr.push(3.14f64);

// Access each with type checking
let val0 = arr.entry(0);
assert_eq!(i64::try_convert(val0)?, 42);

let val1 = arr.entry(1);
let s = RString::try_convert(val1)?;
assert_eq!(s.to_string()?, "hello");

let val2 = arr.entry(2);
assert_eq!(bool::try_convert(val2)?, true);

let val3 = arr.entry(3);
assert_eq!(f64::try_convert(val3)?, 3.14);
```

### Nested Arrays

```rust
// Create 2D array (array of arrays)
let row1 = RArray::from_slice(&[1, 2, 3]);
let row2 = RArray::from_slice(&[4, 5, 6]);
let row3 = RArray::from_slice(&[7, 8, 9]);

let matrix = RArray::new();
matrix.push(row1);
matrix.push(row2);
matrix.push(row3);

// Access nested elements
let first_row = RArray::try_convert(matrix.entry(0))?;
let val = first_row.entry(0);
assert_eq!(i64::try_convert(val)?, 1);

// Access element at [1][2]
let second_row = RArray::try_convert(matrix.entry(1))?;
let val = second_row.entry(2);
assert_eq!(i64::try_convert(val)?, 6);
```

### Error Handling

```rust
// Array with mixed types
let arr = RArray::new();
arr.push(1i64);
arr.push(2i64);
arr.push(RString::new("not a number"));
arr.push(4i64);

// to_vec() fails with mixed types
let result: Result<Vec<i64>, Error> = arr.to_vec();
assert!(result.is_err());

// Selectively convert matching elements
let mut numbers = Vec::new();
arr.each(|val| {
    if let Ok(n) = i64::try_convert(val) {
        numbers.push(n);
    }
    Ok(())
})?;
assert_eq!(numbers, vec![1, 2, 4]); // String skipped
```

## Why Closure-Based Iteration?

You might notice that `RArray` doesn't implement Rust's `Iterator` trait. This is intentional for safety reasons:

### The Problem with Iterator

```rust
// This would be UNSAFE with Iterator trait
let arr = RArray::new();
arr.push(1);
arr.push(2);

let mut iter = arr.iter(); // Hypothetical unsafe iterator
let first = iter.next();

// If the closure calls Ruby code, GC could run
// GC could modify or move the array
// Our iterator would have a dangling pointer!
ruby_function_call(); 

let second = iter.next(); // UNSAFE: Array might have moved
```

### The Safe Solution: Closures

```rust
// Safe: We control when Ruby code can run
arr.each(|val| {
    // Process element
    // If this calls Ruby code, we handle it safely
    Ok(())
})?;
```

By using closures, we maintain control over the array's lifetime and ensure GC safety.

### Converting to Iterator (If Needed)

If you need iterator methods, convert to Vec first:

```rust
let arr = RArray::new();
arr.push(1i64);
arr.push(2i64);
arr.push(3i64);

// Convert to Vec (copies data, safe from GC)
let vec: Vec<i64> = arr.to_vec()?;

// Now we can use iterator methods
let sum: i64 = vec.iter().sum();
let doubled: Vec<i64> = vec.iter().map(|x| x * 2).collect();
```

## Building and Running

Build the example:

```bash
cargo build --release --manifest-path examples/phase2_array/Cargo.toml
```

Run the Ruby test script:

```bash
ruby examples/phase2_array/test.rb
```

Or run just the Rust tests:

```bash
cargo test --manifest-path examples/phase2_array/Cargo.toml
```

## Code Structure

- `src/lib.rs` - 12 example functions demonstrating array operations
- `test.rb` - Ruby script that builds and tests the extension
- `Cargo.toml` - Build configuration
- `build.rs` - Ruby integration build script

## RArray API Summary

### Construction

| Method | Description |
|--------|-------------|
| `RArray::new()` | Create empty array |
| `RArray::with_capacity(n)` | Create with pre-allocated capacity |
| `RArray::from_slice(&[T])` | Create from Rust slice |

### Properties

| Method | Description |
|--------|-------------|
| `len(self) -> usize` | Get number of elements |
| `is_empty(self) -> bool` | Check if empty |

### Stack Operations

| Method | Description |
|--------|-------------|
| `push(self, val: T)` | Add element to end |
| `pop(self) -> Option<Value>` | Remove and return last element |

### Element Access

| Method | Description |
|--------|-------------|
| `entry(self, index: isize) -> Value` | Get element at index (nil if out of bounds) |
| `store(self, index: isize, val: T)` | Set element at index (extends if needed) |

### Iteration

| Method | Description |
|--------|-------------|
| `each<F>(self, f: F) -> Result<(), Error>` | Iterate with closure |

### Conversions

| Method | Description |
|--------|-------------|
| `to_vec<T: TryConvert>(self) -> Result<Vec<T>, Error>` | Convert to Rust Vec |

## Key Design Points

### Why Pre-Allocation?

`with_capacity()` pre-allocates memory, which improves performance when you know the final size:

```rust
// Without capacity - multiple reallocations
let arr = RArray::new();
for i in 0..10000 {
    arr.push(i); // May trigger reallocation
}

// With capacity - single allocation
let arr = RArray::with_capacity(10000);
for i in 0..10000 {
    arr.push(i); // No reallocation needed
}
```

### Why Does pop() Return Option?

Unlike Ruby's `Array#pop` which returns `nil` for empty arrays, Rust's `pop()` returns `Option<Value>`:

```rust
// Ruby-style (what the C API does)
let val = arr.pop(); // Returns nil if empty

// Rust-style (what Solidus does)
match arr.pop() {
    Some(val) => { /* Array had elements */ }
    None => { /* Array was empty */ }
}
```

This is more idiomatic Rust and provides better type safety.

### Why Store Extends the Array?

This matches Ruby's behavior:

```ruby
arr = []
arr[5] = 42
# arr is now [nil, nil, nil, nil, nil, 42]
```

In Rust:

```rust
let arr = RArray::new();
arr.store(5, 42);
assert_eq!(arr.len(), 6);
// Elements 0-4 are nil, element 5 is 42
```

## Comparison with Magnus

| Feature | Solidus | Magnus |
|---------|---------|--------|
| Array creation | `RArray::new()` | `RArray::new()` |
| Capacity hint | `with_capacity(n)` | ❌ Not available |
| Element access | `entry(i) -> Value` | `entry(i).unwrap()` |
| Bounds checking | Returns nil | Panics on invalid access |
| Stack ops | `push()`, `pop()` | `push()`, `pop()` |
| Iteration | Closure-based `each()` | Unsafe `Iterator` impl |
| Conversions | Type-safe `to_vec<T>()` | Limited conversion support |
| Type safety | Compile-time checks | Runtime checks |

## Safety Guarantees

1. **No Dangling References** - `each()` uses closures to prevent iterator invalidation
2. **Bounds Safety** - `entry()` returns nil instead of panicking
3. **Type Safety** - `to_vec<T>()` validates all elements before converting
4. **GC Safety** - Arrays are properly protected from garbage collection
5. **Index Safety** - Negative indices work correctly

## Common Pitfalls

### ❌ Assuming to_vec() Always Succeeds

```rust
// Wrong: Panics if array contains non-integers
let vec: Vec<i64> = arr.to_vec().unwrap();
```

```rust
// Right: Handle type mismatches
match arr.to_vec::<i64>() {
    Ok(vec) => process_integers(vec),
    Err(_) => handle_mixed_types(arr),
}
```

### ❌ Forgetting Negative Indices

```rust
// Wrong: Only handles positive indices
for i in 0..arr.len() {
    let val = arr.entry(i);
}
```

```rust
// Right: Remember Ruby supports negative indices
let last = arr.entry(-1);
let second_last = arr.entry(-2);
```

### ❌ Not Checking for Nil After entry()

```rust
// Wrong: Assumes element exists
let val = arr.entry(5);
let n = i64::try_convert(val).unwrap(); // Panics if out of bounds
```

```rust
// Right: Check for nil
let val = arr.entry(5);
if !val.is_nil() {
    let n = i64::try_convert(val)?;
    // Process n
}
```

## Performance Notes

- `with_capacity()` eliminates reallocations when size is known
- `each()` is faster than `to_vec()` + iterator when you don't need the Vec
- `entry()` is O(1) for both positive and negative indices
- `store()` beyond length is O(n) where n is the extension size
- Ruby arrays are optimized for small sizes with inline storage

## Next Steps

Future stages will add:

- **Stage 6**: Hash type with key-value operations
- **Stage 7**: Class and Module types for object-oriented programming
- **Phase 3**: Method definition with automatic stack pinning

## Related Documentation

- [Phase 2 Tasks](../../docs/plan/phase-2-tasks.md) - Detailed implementation plan
- [Phase 2 Types](../../docs/plan/phase-2-types.md) - Type system design
- [RArray Implementation](../../crates/solidus/src/types/array.rs) - Source code
- [Solidus README](../../README.md) - Project overview
