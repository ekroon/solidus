# Solidus

A safe Rust library for writing Ruby extensions with automatic stack pinning.

[![CI](https://github.com/ekroon/solidus/actions/workflows/ci.yml/badge.svg)](https://github.com/ekroon/solidus/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/solidus.svg)](https://crates.io/crates/solidus)
[![Documentation](https://docs.rs/solidus/badge.svg)](https://docs.rs/solidus)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

## The Problem

When writing Ruby extensions in Rust, Ruby values must stay on the stack so Ruby's garbage collector can find them. In other libraries like Magnus, accidentally moving values to the heap is undefined behavior:

```rust
// This is UB - values moved to heap, invisible to Ruby GC
let values: Vec<Value> = vec![ruby.str_new("hello")];
```

This is error-prone, not enforced by the type system, and not visible at the API level.

## The Solution

Solidus uses Rust's `Pin` type to enforce stack locality at compile time:

```rust
use solidus::prelude::*;

// Method arguments are automatically stack-pinned
fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
    // `other` cannot be moved to heap - enforced by type system
    rb_self.concat(other.get())
}

// Explicit opt-in for heap allocation
fn store(arr: Pin<&StackPinned<RArray>>) -> Result<(), Error> {
    let boxed: BoxValue<RString> = BoxValue::new(arr.get().entry(0)?);  // GC-registered
    // boxed can safely be stored in Vec, HashMap, etc.
    Ok(())
}
```

## Features

- **Safety by default**: Ruby values are stack-pinned via `Pin<&StackPinned<T>>`. Users cannot accidentally move values to the heap.
- **Explicit heap allocation**: When heap storage is needed, use `BoxValue<T>`, which registers with Ruby's GC.
- **Zero-cost abstractions**: Pinning has no runtime overhead - it's purely a compile-time guarantee.
- **Immediate values bypass pinning**: `Fixnum`, `Symbol`, `true`, `false`, `nil` don't need GC protection.

## Requirements

- Rust 1.85+ (Edition 2024)
- Ruby 3.4+

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
solidus = "0.1"
```

## License

This project is licensed under the MIT License - see [LICENSE-MIT](LICENSE-MIT) for details.
