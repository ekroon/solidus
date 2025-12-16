# Solidus Guide

This guide will help you get started with Solidus for writing Ruby extensions in Rust.

## Table of Contents

1. [Getting Started](getting-started.md) - Installation and first extension
2. [Core Concepts](core-concepts.md) - Understanding stack pinning and safety
3. [Ruby Types](ruby-types.md) - Working with RString, RArray, RHash, etc.
4. [Methods and Functions](methods.md) - Defining Ruby methods in Rust
5. [TypedData](typed-data.md) - Wrapping Rust structs as Ruby objects
6. [Error Handling](error-handling.md) - Working with Ruby exceptions

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
    fn greet(&self, name: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
        let name_str = name.get().to_string()?;
        RString::new(&format!("Hello, {}!", name_str))
    }
}
```

## Further Reading

- [API Documentation](https://docs.rs/solidus)
- [Examples](../../examples/)
- [Implementation Plan](../plan/)
