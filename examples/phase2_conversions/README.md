# Phase 2 Stage 2: Immediate Types Example

This example demonstrates Ruby's immediate types in Solidus: nil, true, false, fixnum, symbol, and flonum.

## Overview

Stage 2 of Phase 2 implements type-safe wrappers for Ruby's immediate values:

- **`Qnil`**, **`Qtrue`**, **`Qfalse`** - Ruby's singleton values
- **`Fixnum`** - Small integers encoded directly in the VALUE
- **`Symbol`** - Interned strings
- **`Flonum`** - Immediate floats (64-bit platforms only)

Immediate values are special because they're encoded directly in the VALUE pointer rather than being heap-allocated. This means they don't require GC protection or stack pinning.

## What This Example Demonstrates

1. **Boolean Singletons** - Working with nil, true, and false
2. **Rust Bool Conversion** - Converting Rust booleans to Ruby
3. **Ruby Truthiness Rules** - Only nil and false are falsy
4. **Fixnum Operations** - Creating and converting small integers
5. **Integer Type Conversions** - Converting between various Rust integer types
6. **Symbol Creation** - Creating and interning symbols
7. **Symbol from &str** - Direct string-to-symbol conversion
8. **Float Conversions** - Working with f32 and f64
9. **Flonum (64-bit)** - Immediate floats on 64-bit platforms
10. **Type Safety** - Immediate values don't need pinning

## Key Features

### No Pinning Required

Unlike heap-allocated Ruby objects, immediate values can be passed directly:

```rust
fn process_immediate_values(count: i64, name: Symbol, enabled: bool) -> Value {
    // No Pin<&StackPinned<T>> needed - these are all immediate values!
    if enabled {
        Symbol::new(&format!("{}_{}", name.name().unwrap(), count)).into_value()
    } else {
        Qnil::new().into_value()
    }
}
```

### Type-Safe Conversions

Each immediate type has proper `TryConvert` and `IntoValue` implementations:

```rust
// Create a Fixnum
let num = Fixnum::from_i64(42).expect("fits in Fixnum");
assert_eq!(num.to_i64(), 42);

// Create a Symbol
let sym = Symbol::new("hello");
assert_eq!(sym.name().unwrap(), "hello");

// Ruby truthiness
let is_truthy = bool::try_convert(value).unwrap();
```

### Symbol Interning

Symbols are automatically interned:

```rust
let sym1 = Symbol::new("test");
let sym2 = Symbol::new("test");
assert_eq!(sym1.as_value(), sym2.as_value()); // Same symbol!
```

## Building and Running

Build the example:

```bash
cargo build --release --manifest-path examples/phase2_conversions/Cargo.toml
```

Run the Ruby test script:

```bash
ruby examples/phase2_conversions/test.rb
```

Or run just the Rust tests:

```bash
cargo test --manifest-path examples/phase2_conversions/Cargo.toml
```

## Code Structure

- `src/lib.rs` - Example functions demonstrating immediate types
- `test.rb` - Ruby script that loads and tests the extension
- `Cargo.toml` - Build configuration

## Immediate Value Types

### Qnil, Qtrue, Qfalse

Zero-sized types representing Ruby's singletons:

```rust
let nil_val = Qnil::new();
let true_val = Qtrue::new();
let false_val = Qfalse::new();

// Rust bool maps to Ruby true/false
let ruby_bool = true.into_value(); // Qtrue
```

### Fixnum

Small integers that fit directly in a VALUE:

```rust
let num = Fixnum::from_i64(42).expect("fits in Fixnum");
let doubled = (num.to_i64() * 2).into_value();
```

### Symbol

Interned strings used for identifiers:

```rust
let sym = Symbol::new("method_name");
println!("Symbol: {}", sym.name().unwrap());

// Direct conversion from &str
let sym2 = "another_symbol".into_value();
```

### Flonum (64-bit only)

On 64-bit platforms, small floats can be immediate:

```rust
#[cfg(target_pointer_width = "64")]
{
    let flonum = Flonum::from_f64(1.5).expect("fits in Flonum");
    assert_eq!(flonum.to_f64(), 1.5);
}

// General float conversion (works on all platforms)
let float = 3.14f64.into_value();
let back = f64::try_convert(float).unwrap();
```

## Rust Type Conversions

The following Rust types have automatic conversions:

| Rust Type | Ruby Type | Notes |
|-----------|-----------|-------|
| `bool` | `TrueClass`/`FalseClass` | Ruby truthiness rules apply |
| `i8`, `i16`, `i32`, `i64`, `isize` | `Fixnum` | Panics if too large (Bignum not yet implemented) |
| `u8`, `u16`, `u32` | `Fixnum` | Always fits |
| `u64`, `usize` | `Fixnum` | Panics if too large |
| `f32`, `f64` | `Float` | May be Flonum or heap-allocated |
| `&str` | `Symbol` | Direct symbol creation |

## Next Steps

Future stages will add:

- **Stage 3**: Numeric types (Bignum for large integers)
- **Stage 4**: String type with encoding support  
- **Stage 5**: Array type with iteration
- **Stage 6**: Hash type
- **Stage 7**: Class and Module types

## Related Documentation

- [Phase 2 Tasks](../../docs/plan/phase-2-tasks.md) - Detailed implementation plan
- [Phase 2 Types](../../docs/plan/phase-2-types.md) - Type system design
- [Solidus README](../../README.md) - Project overview
