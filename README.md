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
2. **Creation returns `NewValue<T>`** - Must explicitly choose stack or heap storage
3. **Compile-time enforcement** - Cannot forget to pin or box a value

```rust
use solidus::prelude::*;

// Creating a value returns a NewValue
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

## Quick Start

Here's a complete example showing how to create a Ruby extension with Solidus:

```rust
use solidus::prelude::*;
use std::pin::Pin;

/// A function that greets someone by name
fn greet(name: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let name_str = name.get().to_string()?;
    Ok(RString::new(&format!("Hello, {}!", name_str)))
}

/// Initialize the extension - called when Ruby loads the library
#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Register a global function callable from Ruby
    ruby.define_global_function("greet", solidus::function!(greet, 1), 1)?;
    Ok(())
}
```

Then in Ruby:

```ruby
require 'my_extension'
puts greet("World")  # => "Hello, World!"
```

## Installation

### Requirements

- Rust 1.85+ (Edition 2024)
- Ruby 3.4+ with development headers

### Setting Up a New Extension

1. Create a new Rust library:

```bash
cargo new --lib my_extension
cd my_extension
```

2. Configure `Cargo.toml`:

```toml
[package]
name = "my_extension"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
solidus = "0.1"
rb-sys = "0.9"

[build-dependencies]
rb-sys-build = "0.9"
```

3. Create `build.rs`:

```rust
fn main() {
    let rb_config = rb_sys_build::RbConfig::current();
    rb_config.print_cargo_args();
}
```

4. Write your extension in `src/lib.rs` (see Quick Start above)

5. Build and test:

```bash
cargo build
# Create symlink for Ruby to find
ln -sf target/debug/libmy_extension.dylib target/debug/my_extension.bundle  # macOS
ln -sf target/debug/libmy_extension.so target/debug/my_extension.so          # Linux

ruby -e "require_relative 'target/debug/my_extension'; puts greet('World')"
```

## Key Features

- **Safety by default**: Ruby values must be pinned from creation - enforced at compile time
- **Clear API**: `NewValue<T>` with `#[must_use]` makes requirements explicit
- **Zero-cost abstractions**: All safety checks are compile-time only
- **Immediate values optimized**: `Fixnum`, `Symbol`, `true`, `false`, `nil` remain `Copy`
- **Prevents Magnus-style UB**: Compiler enforces what Magnus only documents

## API Overview

### Core Types

| Type | Description |
|------|-------------|
| `Value` | Base wrapper around Ruby's VALUE (`!Copy`) |
| `NewValue<T>` | Guard enforcing pinning or boxing of new values |
| `StackPinned<T>` | `!Unpin` wrapper for stack-pinned values |
| `BoxValue<T>` | Heap-allocated, GC-registered wrapper |
| `Ruby` | Handle to the Ruby VM |
| `Error` | Ruby exception wrapper |

### Ruby Types

| Type | Ruby Equivalent |
|------|-----------------|
| `RString` | String |
| `RArray` | Array |
| `RHash` | Hash |
| `RClass` | Class |
| `RModule` | Module |
| `Integer`, `Fixnum`, `RBignum` | Integer |
| `Float`, `RFloat`, `Flonum` | Float |
| `Symbol` | Symbol |
| `Qnil`, `Qtrue`, `Qfalse` | nil, true, false |

### Key Macros

| Macro | Purpose |
|-------|---------|
| `#[solidus::init]` | Generates extension entry point |
| `solidus::method!(fn, arity)` | Creates wrapper for instance methods |
| `solidus::function!(fn, arity)` | Creates wrapper for functions/class methods |
| `pin_on_stack!(var = guard)` | Pins a value on the stack |

### Modules

- `solidus::prelude` - Common imports for convenience
- `solidus::types` - Ruby type wrappers
- `solidus::convert` - Type conversion traits (`TryConvert`, `IntoValue`)
- `solidus::typed_data` - Wrap Rust structs as Ruby objects
- `solidus::error` - Error and exception handling

## Comparison with Magnus

| Aspect | Magnus | Solidus |
|--------|--------|---------|
| Heap safety | Documentation only | Compile-time enforced |
| VALUE types | `Copy` | `!Copy` |
| Value creation | Returns value directly | Returns `NewValue<T>` |
| Heap storage | Unsafe, UB if forgotten | Explicit `BoxValue<T>` with GC registration |
| Immediate values | `Copy` | `Copy` (same) |
| Runtime overhead | None | None (compile-time only) |

**When to choose Solidus:**
- You want compile-time guarantees against GC-related undefined behavior
- You're building production systems where safety is critical
- You want explicit control over heap vs stack allocation

**When Magnus might be preferred:**
- You have existing Magnus code and the migration cost is high
- You prefer a more established ecosystem

## Documentation

- [Getting Started](docs/guide/getting-started.md) - Full setup and first extension
- [Pinning](docs/guide/pinning.md) - Understanding stack pinning and GC safety
- [BoxValue](docs/guide/boxvalue.md) - Safe heap storage for Ruby values
- [Ruby Types](docs/guide/types.md) - Working with RString, RArray, RHash, etc.
- [Methods](docs/guide/methods.md) - Defining Ruby methods in Rust
- [TypedData](docs/guide/typed-data.md) - Wrapping Rust structs as Ruby objects
- [Error Handling](docs/guide/error-handling.md) - Working with Ruby exceptions

See also the [examples/](examples/) directory for complete working extensions.

## Contributing

Contributions are welcome! Here's how to get started:

### Development Setup

```bash
git clone https://github.com/ekroon/solidus.git
cd solidus

# Build
cargo build --workspace

# Run tests (no Ruby required)
cargo test --workspace

# Run tests with Ruby integration
cargo test --workspace --features link-ruby

# Lint
cargo fmt --check && cargo clippy --workspace
```

### Guidelines

1. **Format code**: Run `cargo fmt --all` before committing
2. **Run clippy**: Ensure `cargo clippy --workspace -- -D warnings` passes
3. **Add tests**: New features should include tests
4. **Update docs**: Keep documentation in sync with code changes
5. **Follow style**: See [AGENTS.md](AGENTS.md) for code style guidelines

### Pre-commit Hook

Install the pre-commit hook to automatically check formatting:

```bash
cp pre-commit.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
```

### Reporting Issues

Please report bugs and feature requests on the [GitHub Issues](https://github.com/ekroon/solidus/issues) page.

## License

This project is licensed under the MIT License - see [LICENSE-MIT](LICENSE-MIT) for details.
