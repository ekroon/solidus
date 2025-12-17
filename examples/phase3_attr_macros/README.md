# Phase 3 Attribute Macros Example

This example demonstrates the `#[solidus_macros::method]` and `#[solidus_macros::function]`
attribute macros that provide **automatic pinning** for method arguments.

## Key Feature: Automatic Pinning

The attribute macros generate wrapper code that automatically handles stack pinning of
Ruby VALUEs. You write function signatures with `Pin<&StackPinned<T>>`, and the macro's
wrapper does the pinning work for you:

```rust
// Your function signature uses Pin<&StackPinned<T>>
#[solidus_macros::method]
fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    Ok(RString::new(&format!("{}{}", self_str, other_str)))
}
```

The macro generates a wrapper that:
1. Receives raw VALUE arguments from Ruby
2. Converts them to typed values (e.g., RString)
3. Wraps them in StackPinned<T> on the wrapper's stack
4. Pins them and passes Pin<&StackPinned<T>> to your function

You never manually pin values - the macro handles it.

## Comparison with phase3_methods

Both `phase3_methods` and `phase3_attr_macros` use the same `Pin<&StackPinned<T>>`
signatures for heap-allocated Ruby values. The difference is in syntax:

| Feature | `phase3_methods` | `phase3_attr_macros` |
|---------|------------------|----------------------|
| Macro style | `method!(fn_name, arity)` | `#[solidus_macros::method]` |
| Argument signature | `Pin<&StackPinned<T>>` | `Pin<&StackPinned<T>>` |
| Generated module | N/A | `__solidus_method_<name>` |
| ARITY constant | Passed separately | `__solidus_method_<name>::ARITY` |

Both provide automatic pinning via the generated wrapper code.

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

### VALUE Types are !Copy (ADR-007)

After ADR-007, all Ruby VALUE types (RString, RArray, etc.) are `!Copy`. This prevents
accidentally moving them to the heap, which would break GC safety since Ruby's GC only
scans the C stack for VALUEs.

### Generated Code Structure

For each function annotated with `#[solidus_macros::method]` or `#[solidus_macros::function]`,
the macro generates a hidden module:

```rust
// For: #[solidus_macros::function]
//      fn greet(name: Pin<&StackPinned<RString>>) -> Result<PinGuard<RString>, Error>

#[doc(hidden)]
pub mod __solidus_function_greet {
    pub const ARITY: i32 = 1;
    
    pub fn wrapper() -> unsafe extern "C" fn() -> solidus::rb_sys::VALUE {
        // Generated wrapper that handles:
        // - Panic catching
        // - Type conversion from raw VALUEs
        // - Stack pinning (wraps values in StackPinned<T>)
        // - Calling your function with Pin<&StackPinned<T>>
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

## Features Demonstrated

1. **Global functions** with automatic pinning (arities 0-2)
2. **Instance methods** with automatic pinning (arities 0-2)
3. **Module functions** with automatic pinning
4. **Mixed argument types** (pinned heap values and immediate values)

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
