# Solidus

A safe Rust library for writing Ruby extensions with automatic stack pinning.

[![CI](https://github.com/ekroon/solidus/actions/workflows/ci.yml/badge.svg)](https://github.com/ekroon/solidus/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/solidus.svg)](https://crates.io/crates/solidus)
[![Documentation](https://docs.rs/solidus/badge.svg)](https://docs.rs/solidus)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

## The Problem

When writing Ruby extensions in Rust, Ruby values must stay on the stack so Ruby's garbage collector can find them. In other libraries like Magnus, accidentally moving values to the heap is undefined behavior:

```rust
// This is UB in Magnus - values moved to heap, invisible to Ruby GC
let values: Vec<Value> = vec![ruby.str_new("hello")];
```

This is error-prone, not enforced by the type system, and not visible at the API level. See [Magnus issue #101](https://github.com/matsadler/magnus/issues/101) for details.

## The Solution

Solidus uses Rust's type system to enforce stack locality at compile time through **pinned-from-creation**:

1. **All VALUE types are `!Copy`** - Cannot be accidentally copied to heap
2. **Creation returns `PinGuard<T>`** - Must explicitly choose stack or heap storage
3. **Compile-time enforcement** - Cannot forget to pin or box a value

```rust
use solidus::prelude::*;

// Creating a value returns a PinGuard
let guard = RString::new("hello");

// Option 1: Pin on stack (common case)
let pinned = guard.pin();
pin_on_stack!(s = pinned);
// s is Pin<&StackPinned<RString>>, cannot be moved to heap

// Option 2: Box for heap storage (for collections)
let guard = RArray::new();
let boxed = guard.into_box();     // Explicit GC registration
let mut values = vec![boxed];     // Safe! GC knows about it
```

## Key Features

- **Safety by default**: Ruby values must be pinned from creation - enforced at compile time
- **Clear API**: `PinGuard<T>` with `#[must_use]` makes requirements explicit
- **Zero-cost abstractions**: All safety checks are compile-time only
- **Immediate values optimized**: `Fixnum`, `Symbol`, `true`, `false`, `nil` remain `Copy`
- **Prevents Magnus-style UB**: Compiler enforces what Magnus only documents

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
