# Phase 2 Stage 3: Numeric Types (Heap) Example

This example demonstrates **heap-allocated numeric types** in Solidus and how Ruby automatically chooses between immediate and heap representations for efficiency.

## What Stage 3 Implements

Stage 3 adds support for numeric types that require heap allocation:

1. **RBignum** - Large integers that don't fit in Fixnum
2. **Integer** - Union type that automatically selects Fixnum or Bignum
3. **RFloat** - Heap-allocated floating-point numbers
4. **Float** - Union type that automatically selects Flonum (64-bit only) or RFloat

## Immediate vs Heap Numeric Values

Ruby uses two strategies for storing numeric values:

### Immediate Values (No Heap Allocation)

**Fixnum** - Small integers encoded directly in the VALUE:
- Range: Roughly ±2^62 on 64-bit platforms (about ±4.6 × 10^18)
- No GC protection needed
- Fast operations

**Flonum** (64-bit platforms only) - Small floats encoded directly in the VALUE:
- Not all floats can be Flonum - Ruby decides based on precision
- No GC protection needed
- Fast operations

### Heap-Allocated Values (Require GC Protection)

**Bignum** - Large integers that exceed Fixnum range:
- Any integer too large for Fixnum (e.g., i64::MAX on most platforms)
- Arbitrary precision
- Requires GC protection

**RFloat** - Heap-allocated floats:
- All floats on 32-bit platforms
- Some floats on 64-bit platforms (when they can't be Flonum)
- Full f64 precision
- Requires GC protection

## How the Union Types Work

The union types (`Integer` and `Float`) hide the complexity of immediate vs heap allocation:

### Integer Enum

```rust
pub enum Integer {
    Fixnum(Fixnum),  // Small integers (immediate)
    Bignum(RBignum), // Large integers (heap)
}
```

**Usage:**
```rust
// Automatically selects Fixnum or Bignum
let small = Integer::from_i64(42);           // → Fixnum
let large = Integer::from_u64(u64::MAX);     // → Bignum (on most platforms)

// Conversion methods work on both variants
let value = small.to_i64().unwrap();         // Always works
let value2 = large.to_u64().unwrap();        // Works if in range
```

**Benefits:**
- Users don't need to know which variant is used
- Automatic selection based on value size
- Unified API for all integers
- Range checking on conversions

### Float Enum

```rust
pub enum Float {
    #[cfg(target_pointer_width = "64")]
    Flonum(Flonum),  // Immediate float (64-bit only)
    RFloat(RFloat),  // Heap-allocated float
}
```

**Usage:**
```rust
// Automatically selects Flonum or RFloat
let f = Float::from_f64(1.5);

// Platform-independent access
let value = f.to_f64();  // Works on both variants

// Conversion to/from Rust types
let rust_f64 = 3.14f64;
let ruby_float = rust_f64.into_value();
```

**Benefits:**
- Platform-independent (Flonum only exists on 64-bit)
- Transparent handling of Ruby's float representation choice
- Full f64 precision

## Code Examples

### Example 1: Creating Large Integers

```rust
#[no_mangle]
pub extern "C" fn example_large_integers() -> rb_sys::VALUE {
    // Small integers use Fixnum (immediate)
    let small = Integer::from_i64(42);
    assert!(matches!(small, Integer::Fixnum(_)));
    
    // Very large u64 values typically require Bignum
    let large = (1u64 << 63) + 12345;
    let big_int = Integer::from_u64(large);
    
    // Verify it round-trips correctly
    assert_eq!(big_int.to_u64().unwrap(), large);
    
    big_int.into_value().as_raw()
}
```

### Example 2: Automatic Integer Selection

```rust
#[no_mangle]
pub extern "C" fn example_integer_auto_selection(n: i64) -> rb_sys::VALUE {
    // Integer::from_i64 automatically picks Fixnum or Bignum
    let int = Integer::from_i64(n);
    
    // We don't need to worry about which variant it is
    match int.to_i64() {
        Ok(value) => {
            assert_eq!(value, n);
            int.into_value().as_raw()
        }
        Err(_) => Qnil::new().into_value().as_raw()
    }
}
```

### Example 3: Range Checking with Unsigned Integers

```rust
#[no_mangle]
pub extern "C" fn example_u64_range_check(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };
    
    // Try to convert to u64
    match u64::try_convert(value) {
        Ok(n) => {
            // Successfully converted (positive integer in range)
            let doubled = n.saturating_mul(2);
            doubled.into_value().as_raw()
        }
        Err(_) => {
            // Not an integer, negative, or out of range
            Qnil::new().into_value().as_raw()
        }
    }
}
```

### Example 4: Working with Heap Floats

```rust
#[no_mangle]
pub extern "C" fn example_rfloat() -> rb_sys::VALUE {
    // Create a heap-allocated float
    let float = RFloat::from_f64(3.141592653589793);
    
    // Get the value back
    let value = float.to_f64();
    assert!((value - 3.141592653589793).abs() < 0.0000001);
    
    float.into_value().as_raw()
}
```

### Example 5: Float Automatic Selection

```rust
#[no_mangle]
pub extern "C" fn example_float_auto_selection(f: f64) -> rb_sys::VALUE {
    // Float::from_f64 automatically picks Flonum or RFloat
    let float = Float::from_f64(f);
    
    // We can work with it without knowing which variant
    let back = float.to_f64();
    assert!((back - f).abs() < 0.0000001);
    
    float.into_value().as_raw()
}
```

### Example 6: Negative Number Handling

```rust
#[no_mangle]
pub extern "C" fn example_negative_numbers() -> rb_sys::VALUE {
    // Negative integer
    let neg_int = Integer::from_i64(-9876543210);
    assert_eq!(neg_int.to_i64().unwrap(), -9876543210);
    
    // Converting negative to unsigned should fail
    assert!(neg_int.to_u64().is_err());
    
    // Negative float
    let neg_float = Float::from_f64(-123.456);
    assert!((neg_float.to_f64() + 123.456).abs() < 0.001);
    
    neg_int.into_value().as_raw()
}
```

## Key Implementation Details

### Type Safety

All numeric types implement the core Solidus traits:

- `ReprValue` - Convert to/from `Value`
- `TryConvert` - Convert from `Value` with type checking
- `IntoValue` - Convert to `Value`

### Range Checking

Conversions to smaller integer types include range checking:

```rust
// i64 to i32 - checks range
let n = i64::MAX;
let result = i32::try_convert(n.into_value());
assert!(result.is_err()); // Out of range

// Negative to unsigned - checks sign
let neg = -42i64;
let result = u64::try_convert(neg.into_value());
assert!(result.is_err()); // Negative value
```

### Overflow Handling

The union types handle overflow gracefully:

```rust
// Large value that might overflow i64
let large = Integer::from_u64(u64::MAX);

// Conversion may fail if out of range
match large.to_i64() {
    Ok(n) => println!("Fits in i64: {}", n),
    Err(_) => println!("Too large for i64"),
}
```

### Platform Awareness

Flonum support is conditional on platform:

```rust
#[cfg(target_pointer_width = "64")]
{
    // Flonum is available on 64-bit platforms
    if let Some(flonum) = Flonum::from_f64(1.5) {
        // Use immediate float
    }
}

// Float enum works on all platforms
let f = Float::from_f64(1.5);  // Always works
```

## Build and Run

### Build the Extension

```bash
cargo build --release
```

### Run the Test Script

```bash
ruby test.rb
```

This will build the extension and verify it was created successfully.

### Run Rust Unit Tests

```bash
cargo test
```

This runs compile-time checks and basic tests that don't require the Ruby runtime.

## Testing Notes

This example demonstrates heap numeric types at the Rust level. Full integration testing from Ruby requires Phase 3 (method registration and calling).

The test script verifies:
- Extension builds successfully
- Library file is created
- Basic compile-time type checks pass

## Relationship to Other Stages

- **Stage 1** (Conversion Traits): Provides `TryConvert` and `IntoValue` used by numeric types
- **Stage 2** (Immediate Types): Provides `Fixnum` and `Flonum` (for small values)
- **Stage 3** (This stage): Adds `RBignum`, `RFloat`, and union types
- **Stage 4+**: Will use numeric types for indexing, lengths, etc.

## Advanced Topics

### When Does Ruby Use Bignum?

The exact threshold depends on the platform:

- **64-bit platforms**: Fixnum range is approximately ±2^62
  - Values outside this range become Bignum
  - Example: `1 << 62` might be Fixnum or Bignum depending on exact value

- **32-bit platforms**: Fixnum range is approximately ±2^30
  - Much smaller range, so Bignum is more common

### When Does Ruby Use RFloat vs Flonum?

On 64-bit platforms, Ruby decides based on the float's representation:

- Small floats with limited precision → may be Flonum
- Floats requiring full precision → RFloat
- The decision is internal to Ruby

On 32-bit platforms, all floats are RFloat.

### Arithmetic Operations

This example shows type creation and conversion. Actual arithmetic operations would use Ruby's built-in operators:

```rust
// This example is conceptual - actual implementation requires Phase 3
fn add_integers(a: Integer, b: Integer) -> Integer {
    // In real code, you'd call Ruby's + operator
    // which handles overflow to Bignum automatically
}
```

## Comparison with Magnus

Magnus requires manual handling of Fixnum vs Bignum:

```rust
// Magnus approach
let val = magnus::Integer::from_i64(value)?;
// User must be aware of Fixnum/Bignum distinction
```

Solidus hides this complexity:

```rust
// Solidus approach
let val = Integer::from_i64(value);
// Automatically handles Fixnum/Bignum
```

## Next Steps

After Stage 3, the following stages implement:

- **Stage 4**: String type with encoding support
- **Stage 5**: Array type with element access and iteration
- **Stage 6**: Hash type with key-value operations
- **Stage 7**: Class and Module types

## Files in This Example

- `src/lib.rs` - 12 example functions demonstrating heap numeric types
- `test.rb` - Ruby script to build and verify the extension
- `Cargo.toml` - Build configuration
- `build.rs` - Ruby build configuration
- `.gitignore` - Git ignore patterns
- `README.md` - This file

## References

- [Phase 2 Types Plan](../../docs/plan/phase-2-types.md)
- [Phase 2 Implementation Tasks](../../docs/plan/phase-2-tasks.md)
- [Ruby C API Documentation](https://ruby-doc.org/core/doc/extension_rdoc.html)
- [Solidus Project README](../../README.md)
