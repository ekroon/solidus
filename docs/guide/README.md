# Solidus Guide

This guide will help you get started with Solidus for writing Ruby extensions in Rust.

## Table of Contents

1. [Getting Started](getting-started.md) - Installation and first extension
2. [Pinning](pinning.md) - Why Ruby values need pinning and how Solidus enforces it
3. [BoxValue](boxvalue.md) - Storing Ruby values on the heap safely
4. [Ruby Types](types.md) - Working with RString, RArray, RHash, etc.
5. [Methods and Functions](methods.md) - Defining Ruby methods in Rust
6. [TypedData](typed-data.md) - Wrapping Rust structs as Ruby objects
7. [Error Handling](error-handling.md) - Working with Ruby exceptions

## Quick Example

```rust
use solidus::prelude::*;

#[init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("MyClass", ruby.class_object())?;
    
    class.define_method("greet", method!(MyClass::greet, 1))?;
    
    Ok(())
}

impl MyClass {
    fn greet<'ctx>(
        ctx: &'ctx Context,
        _rb_self: &Self,
        name: Pin<&StackPinned<RString>>,
    ) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
        let name_str = name.get().to_string()?;
        ctx.new_string(&format!("Hello, {}!", name_str))
            .map_err(Into::into)
    }
}
```

## Further Reading

- [API Documentation](https://docs.rs/solidus)
- [Examples](../../examples/)
- [Implementation Plan](../plan/)
