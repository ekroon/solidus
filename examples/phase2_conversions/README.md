# Phase 2 Stage 1: Conversion Traits Example

This example demonstrates the `TryConvert` and `IntoValue` traits that form the foundation for converting between Ruby and Rust types in Solidus.

## Overview

Stage 1 of Phase 2 implements the core conversion trait infrastructure:

- **`IntoValue`** - Converts Rust types to Ruby `Value` (infallible)
- **`TryConvert`** - Converts Ruby `Value` to Rust types (fallible)

At this stage, only identity conversions for `Value` are implemented. Later stages will add implementations for specific types like strings, integers, arrays, etc.

## What This Example Demonstrates

1. **Identity Conversion with IntoValue** - Converting a `Value` back to `Value`
2. **Identity Conversion with TryConvert** - Converting a `Value` to `Value` with error handling
3. **Working with Nil Values** - Converting nil through both traits
4. **Generic Functions with IntoValue** - Writing generic functions that accept any `IntoValue` type
5. **Generic Functions with TryConvert** - Writing generic functions that convert to any `TryConvert` type
6. **Chained Conversions** - Composing conversions together

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

- `src/lib.rs` - Example functions demonstrating the conversion traits
- `test.rb` - Ruby script that loads and tests the extension
- `Cargo.toml` - Build configuration

## Key Concepts

### IntoValue Trait

```rust
pub trait IntoValue {
    fn into_value(self) -> Value;
}
```

Used to convert Rust types into Ruby `Value`. This is infallible - the conversion always succeeds.

### TryConvert Trait

```rust
pub trait TryConvert: Sized {
    fn try_convert(val: Value) -> Result<Self, Error>;
}
```

Used to convert Ruby `Value` into Rust types. This is fallible - the conversion can fail if the Ruby value is not compatible with the target type.

### Identity Conversions

The base implementation for both traits is the identity conversion for `Value`:

```rust
impl IntoValue for Value {
    fn into_value(self) -> Value {
        self
    }
}

impl TryConvert for Value {
    fn try_convert(val: Value) -> Result<Self, Error> {
        Ok(val)
    }
}
```

These identity conversions serve as the foundation. Later stages will add implementations for specific types.

## Future Enhancements

As Phase 2 progresses through subsequent stages, additional implementations will be added:

- **Stage 2**: Immediate types (Fixnum, Symbol, Qnil, Qtrue, Qfalse, Flonum)
- **Stage 3**: Numeric types (Integer, Float, Bignum)
- **Stage 4**: String type with encoding support
- **Stage 5**: Array type with iteration
- **Stage 6**: Hash type
- **Stage 7**: Class and Module types

Each of these types will implement both `TryConvert` and `IntoValue` for seamless conversion between Ruby and Rust.

## Related Documentation

- [Phase 2 Tasks](../../docs/plan/phase-2-tasks.md) - Detailed implementation plan
- [Phase 2 Types](../../docs/plan/phase-2-types.md) - Type system design
- [Solidus README](../../README.md) - Project overview
