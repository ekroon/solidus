# Phase 3 Attribute Macros Example

This example demonstrates the `#[solidus_macros::method]` and `#[solidus_macros::function]`
attribute macros that provide **implicit pinning** for method arguments.

## Key Feature: Implicit Pinning

The main innovation of these attribute macros is that you can write simple function
signatures without manually wrapping types in `Pin<&StackPinned<T>>`:

```rust
// Simple signature with implicit pinning (recommended)
#[solidus_macros::method]
fn concat(rb_self: RString, other: RString) -> Result<RString, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.to_string()?;
    Ok(RString::new(&format!("{}{}", self_str, other_str)))
}

// Explicit pinning (still supported for backward compatibility)
#[solidus_macros::method]
fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    Ok(RString::new(&format!("{}{}", self_str, other_str)))
}
```

## Comparison with phase3_methods

| Feature | `phase3_methods` | `phase3_attr_macros` |
|---------|------------------|----------------------|
| Macro style | `method!(fn_name, arity)` | `#[solidus_macros::method]` |
| Argument signature | Explicit `Pin<&StackPinned<T>>` | Implicit `T` (simple) |
| Generated module | N/A | `__solidus_method_<name>` |
| ARITY constant | Passed separately | `__solidus_method_<name>::ARITY` |

## Building

```bash
cd examples/phase3_attr_macros
cargo build
```

This creates `target/debug/libphase3_attr_macros.dylib` (or `.so` on Linux).

## Testing

```bash
# Create symlink for Ruby to load
cd target/debug
ln -sf libphase3_attr_macros.dylib phase3_attr_macros.bundle
cd ../..

# Run the test script
ruby test.rb
```

Expected output:
```
======================================================================
Phase 3 Attribute Macros - Implicit Pinning Test
======================================================================

Testing Global Functions (Implicit Pinning)
----------------------------------------------------------------------
attr_get_greeting() => "Hello from attribute macros!"
attr_greet('World') => "Hello, World!"
attr_join_strings('Hello', 'World') => "Hello World"
All global function tests passed!

...

======================================================================
ALL TESTS PASSED!
======================================================================
```

## How It Works

### Generated Code Structure

For each function annotated with `#[solidus_macros::method]` or `#[solidus_macros::function]`,
the macro generates a hidden module:

```rust
// For: #[solidus_macros::function]
//      fn greet(name: RString) -> Result<RString, Error>

#[doc(hidden)]
pub mod __solidus_function_greet {
    pub const ARITY: i32 = 1;
    
    pub fn wrapper() -> unsafe extern "C" fn() -> solidus::rb_sys::VALUE {
        // Generated wrapper that handles:
        // - Panic catching
        // - Type conversion
        // - Stack pinning
        // - Error propagation
    }
}
```

### Registration Pattern

```rust
// Using the generated module for registration
ruby.define_global_function(
    "greet",
    __solidus_function_greet::wrapper(),
    __solidus_function_greet::ARITY,
)?;
```

### Copy Requirement

When using implicit pinning (simple type signatures), the argument types must implement
`Copy`. This is enforced at compile time. All solidus Ruby value types (`RString`,
`RArray`, `Value`, etc.) implement `Copy` since they are just wrappers around a
pointer-sized `VALUE`.

## Features Demonstrated

1. **Global functions** with implicit pinning (arities 0-2)
2. **Instance methods** with implicit pinning (arities 0-2)
3. **Module functions** with implicit pinning
4. **Explicit pinning** for backward compatibility
5. **Mixed signatures** combining implicit and explicit pinning

## Supported Arities

Currently, the attribute macros support arities 0-2. For higher arities, use the
`method!` and `function!` declarative macros from `phase3_methods`.

## Code Organization

```
phase3_attr_macros/
├── Cargo.toml          # Extension gem configuration
├── build.rs            # Ruby extension build script
├── src/
│   └── lib.rs          # Main extension code with attribute macros
├── test.rb             # Ruby test script
└── README.md           # This file
```

## Related Documentation

- [Phase 3 Tasks](../../docs/plan/phase-3-tasks.md) - Implementation details
- [phase3_methods example](../phase3_methods/) - Alternative using declarative macros
- [solidus-macros crate](../../crates/solidus-macros/) - Macro implementation

## License

MIT
