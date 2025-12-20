# Solidus

A safe Rust library for writing Ruby extensions with compile-time GC safety.

[![CI](https://github.com/ekroon/solidus/actions/workflows/ci.yml/badge.svg)](https://github.com/ekroon/solidus/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/solidus.svg)](https://crates.io/crates/solidus)
[![Documentation](https://docs.rs/solidus/badge.svg)](https://docs.rs/solidus)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

## The Problem

When writing Ruby extensions in Rust, Ruby values must stay visible to the garbage collector. Ruby's GC uses conservative stack scanning - it only looks at the C stack to find live VALUE references. In other libraries like Magnus, accidentally moving values to the heap causes undefined behavior:

```rust
// In Magnus, this compiles but is UB - values on heap, invisible to Ruby GC
let values: Vec<RString> = vec![ruby.str_new("hello")];
```

This is error-prone, not enforced by the type system, and can lead to use-after-free bugs. See [Magnus issue #101](https://github.com/matsadler/magnus/issues/101) for details.

## The Solution

Solidus enforces safety at compile time through three mechanisms:

1. **All VALUE types are `!Copy`** - Cannot be accidentally duplicated to heap
2. **Context for stack allocation** - Methods receive a `Context` with GC-visible stack slots
3. **Explicit heap registration** - `BoxValue<T>` for when heap storage is needed

```rust
use solidus::prelude::*;
use std::pin::Pin;

// Methods receive &Context as first parameter (injected by macros)
// Create values using ctx.new_*() - they live in stack slots visible to GC
fn greet<'ctx>(
    ctx: &'ctx Context,
    name: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let name_str = name.get().to_string()?;
    ctx.new_string(&format!("Hello, {}!", name_str)).map_err(Into::into)
}

// For heap storage (Vec, HashMap), use BoxValue via *_boxed() methods
let boxed = RString::new_boxed("hello");  // GC-registered
let mut values: Vec<BoxValue<RString>> = vec![boxed];  // Safe!
```

## Quick Start

### Using Context (Recommended)

The `Context` type provides safe stack-allocated slots for Ruby values. Methods defined with `method!` or `function!` declarative macros automatically inject a `&Context` as the first parameter:

```rust
use solidus::prelude::*;
use std::pin::Pin;

/// Greet someone by name
fn greet<'ctx>(
    ctx: &'ctx Context,
    name: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let name_str = name.get().to_string()?;
    ctx.new_string(&format!("Hello, {}!", name_str)).map_err(Into::into)
}

/// Initialize the extension
#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    ruby.define_global_function("greet", solidus::function!(greet, 1), 1)?;
    Ok(())
}
```

### Using BoxValue (Simple Cases)

For simple functions that just return a value, `BoxValue<T>` is convenient. When using the `#[solidus_macros::function]` or `#[solidus_macros::method]` attribute macros, the Context parameter is handled automatically:

```rust
use solidus::prelude::*;

/// Return a greeting - BoxValue handles GC registration automatically
#[solidus_macros::function]
fn hello() -> Result<BoxValue<RString>, Error> {
    Ok(RString::new_boxed("Hello from Solidus!"))
}

#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    ruby.define_global_function("hello", __solidus_function_hello::wrapper(), __solidus_function_hello::ARITY)?;
    Ok(())
}
```

Then in Ruby:

```ruby
require 'my_extension'
puts greet("World")  # => "Hello, World!"
puts hello()         # => "Hello from Solidus!"
```

## Understanding Context

`Context` is central to Solidus's safety model. It provides stack-allocated slots where Ruby values can live safely during a method call.

**Key points:**
- `method!` and `function!` declarative macros inject `&Context` as the first parameter
- Attribute macros (`#[solidus_macros::method]`, `#[solidus_macros::function]`) handle Context automatically without requiring an explicit parameter
- `Context` provides 8 VALUE slots by default, customizable via const generic: `Context<'a, 16>`
- Use `ctx.new_string()`, `ctx.new_array()`, `ctx.new_hash()` to create values
- Values have lifetime `'ctx` - they cannot outlive the method call
- For heap storage (collections), use `BoxValue<T>` via `*_boxed()` methods

```rust
fn process<'ctx>(ctx: &'ctx Context) -> Result<Pin<&'ctx StackPinned<RArray>>, Error> {
    let arr = ctx.new_array()?;
    let s1 = ctx.new_string("hello")?;
    let s2 = ctx.new_string("world")?;
    // All three values are in Context's stack slots - GC can see them
    arr.get().push(s1.get().as_value());
    arr.get().push(s2.get().as_value());
    Ok(arr)
}
```

For more details, see [Pinning](docs/guide/pinning.md) and [BoxValue](docs/guide/boxvalue.md).

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
    // Get Ruby configuration and set up linking
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

- **Safety by default**: Ruby values are created in GC-visible locations - enforced at compile time
- **Clear API**: `Context` for stack values, `BoxValue<T>` for heap - explicit and safe
- **Zero-cost abstractions**: All safety checks are compile-time only
- **Immediate values optimized**: `Fixnum`, `Symbol`, `true`, `false`, `nil` remain `Copy`
- **Prevents Magnus-style UB**: Compiler enforces what Magnus only documents

## API Overview

### Core Types

| Type | Description |
|------|-------------|
| `Value` | Base wrapper around Ruby's VALUE (`!Copy`) |
| `Context` | Stack-allocated slots for creating values in methods |
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

| Macro | Type | Purpose |
|-------|------|---------|
| `#[solidus::init]` | Attribute | Generates extension entry point |
| `solidus::method!(fn, arity)` | Declarative | Creates wrapper for instance methods (injects Context) |
| `solidus::function!(fn, arity)` | Declarative | Creates wrapper for functions/class methods (injects Context) |
| `#[solidus_macros::method]` | Attribute | Alternative for methods (handles Context automatically) |
| `#[solidus_macros::function]` | Attribute | Alternative for functions (handles Context automatically) |

### Modules

- `solidus::prelude` - Common imports for convenience
- `solidus::context` - Context for creating values in methods
- `solidus::types` - Ruby type wrappers
- `solidus::convert` - Type conversion traits (`TryConvert`, `IntoValue`)
- `solidus::typed_data` - Wrap Rust structs as Ruby objects
- `solidus::error` - Error and exception handling

## Comparison with Magnus

| Aspect | Magnus | Solidus |
|--------|--------|---------|
| Heap safety | Documentation only | Compile-time enforced |
| VALUE types | `Copy` | `!Copy` |
| Value creation | Returns value directly | Via `Context` or `*_boxed()` methods |
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
