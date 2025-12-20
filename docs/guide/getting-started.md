# Getting Started with Solidus

This guide walks you through creating your first Ruby extension with Solidus.

## Prerequisites

- Rust 1.85+ (Edition 2024)
- Ruby 3.4+
- A working Ruby development environment with headers

## Project Setup

### 1. Create a New Rust Library

```bash
cargo new --lib my_extension
cd my_extension
```

### 2. Configure Cargo.toml

Update your `Cargo.toml` to build a dynamic library and include the required dependencies:

```toml
[package]
name = "my_extension"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
solidus = "0.1"
rb-sys = "0.9"

[build-dependencies]
rb-sys-build = "0.9"
```

Key points:
- `crate-type = ["cdylib"]` produces a dynamic library Ruby can load
- `rb-sys` provides the low-level Ruby C API bindings
- `rb-sys-build` configures linking to Ruby in your build script

### 3. Create build.rs

Create a `build.rs` file in your project root to set up Ruby linking:

```rust
fn main() {
    let rb_config = rb_sys_build::RbConfig::current();
    rb_config.print_cargo_args();
}
```

## Creating a Minimal Extension

### 1. Write Your Extension Code

Replace the contents of `src/lib.rs` with:

```rust
use solidus::prelude::*;
use std::pin::Pin;

/// A simple function that returns a greeting.
fn hello<'ctx>(ctx: &'ctx Context) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    ctx.new_string("Hello from Solidus!").map_err(Into::into)
}

/// Initialize the extension and register our function.
#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    ruby.define_global_function("hello", solidus::function!(hello, 0), 0)?;
    Ok(())
}
```

Let's break this down:

- `use solidus::prelude::*` imports commonly used types and traits
- `#[solidus::init]` generates the `Init_my_extension` entry point Ruby expects
- `solidus::function!` creates the wrapper that bridges Rust and Ruby
- The third argument to `define_global_function` is the arity (number of arguments)

### 2. Build the Extension

```bash
cargo build
```

This creates a dynamic library in `target/debug/`:
- macOS: `libmy_extension.dylib`
- Linux: `libmy_extension.so`
- Windows: `my_extension.dll`

### 3. Create a Ruby-loadable Symlink

Ruby expects extensions with a `.bundle` (macOS) or `.so` (Linux) extension:

```bash
# macOS
cd target/debug
ln -sf libmy_extension.dylib my_extension.bundle

# Linux
cd target/debug
ln -sf libmy_extension.so my_extension.so
```

### 4. Test in Ruby

Create a `test.rb` file:

```ruby
#!/usr/bin/env ruby
require_relative 'target/debug/my_extension'

puts hello()  # => "Hello from Solidus!"
```

Run it:

```bash
ruby test.rb
```

## Defining Functions with Arguments

Let's add functions that take arguments. Solidus enforces stack pinning for
Ruby values to ensure GC safety.

```rust
use solidus::prelude::*;
use std::pin::Pin;

/// Greets a person by name.
/// Arguments use Pin<&StackPinned<T>> for GC safety.
fn greet<'ctx>(
    ctx: &'ctx Context,
    name: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let name_str = name.get().to_string()?;
    ctx.new_string(&format!("Hello, {}!", name_str))
        .map_err(Into::into)
}

/// Adds two numbers (passed as strings).
fn add(
    _ctx: &Context,
    a: Pin<&StackPinned<RString>>,
    b: Pin<&StackPinned<RString>>,
) -> Result<i64, Error> {
    let num_a = a.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("first argument must be a number"))?;
    let num_b = b.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("second argument must be a number"))?;
    Ok(num_a + num_b)
}

#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Register global functions using function! macro
    // Arguments: function name, arity
    ruby.define_global_function("greet", solidus::function!(greet, 1), 1)?;
    ruby.define_global_function("add", solidus::function!(add, 2), 2)?;
    Ok(())
}
```

Key points:
- `Pin<&StackPinned<RString>>` is the signature for Ruby VALUE arguments
- Use `.get()` to access the inner value from a pinned reference
- The `function!` macro handles all the pinning automatically
- Return types can be pinned Ruby values (`Pin<&'ctx StackPinned<T>>`) or Rust primitives (`i64`)
- All functions that return Ruby values must include `ctx: &'ctx Context` parameter

## Defining Classes and Methods

Here's a more complete example with a Ruby class:

```rust
use solidus::prelude::*;
use std::pin::Pin;

/// Instance method: returns the string's length.
/// The `rb_self` parameter is the Ruby receiver (self).
fn string_length(_ctx: &Context, rb_self: RString) -> Result<i64, Error> {
    let s = rb_self.to_string()?;
    Ok(s.len() as i64)
}

/// Instance method: concatenates another string.
/// Arguments use Pin<&StackPinned<T>> for GC safety.
fn string_concat<'ctx>(
    ctx: &'ctx Context,
    rb_self: RString,
    other: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    ctx.new_string(&format!("{}{}", self_str, other_str))
        .map_err(Into::into)
}

/// Class method: creates a greeting.
fn create_greeting<'ctx>(ctx: &'ctx Context) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    ctx.new_string("Hello!").map_err(Into::into)
}

#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Define a class that inherits from String
    let my_string_class = ruby.define_class("MyString", ruby.class_string());
    let my_string = RClass::try_convert(my_string_class)?;

    // Define instance methods using method! macro
    // Arguments: function name, arity (number of args excluding self)
    my_string.clone().define_method(
        "length_in_bytes",
        solidus::method!(string_length, 0),
        0,
    )?;
    my_string.clone().define_method(
        "concat_with",
        solidus::method!(string_concat, 1),
        1,
    )?;

    // Define a class method (singleton method) using function! macro
    my_string.define_singleton_method(
        "greeting",
        solidus::function!(create_greeting, 0),
        0,
    )?;

    Ok(())
}
```

Test it in Ruby:

```ruby
require_relative 'target/debug/my_extension'

s = MyString.new("hello")
puts s.length_in_bytes  # => 5
puts s.concat_with(" world")  # => "hello world"
puts MyString.greeting  # => "Hello!"
```

## Using Attribute Macros

For cleaner code, you can use the attribute macros from `solidus-macros`:

```toml
# Add to Cargo.toml
[dependencies]
solidus = "0.1"
solidus-macros = "0.1"
rb-sys = "0.9"
```

```rust
use solidus::prelude::*;
use std::pin::Pin;

#[solidus_macros::function]
fn greet<'ctx>(
    ctx: &'ctx Context,
    name: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let name_str = name.get().to_string()?;
    ctx.new_string(&format!("Hello, {}!", name_str))
        .map_err(Into::into)
}

#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // The macro generates __solidus_function_greet with wrapper() and ARITY
    ruby.define_global_function(
        "greet",
        __solidus_function_greet::wrapper(),
        __solidus_function_greet::ARITY,
    )?;
    Ok(())
}
```

## Error Handling

Solidus propagates errors as Ruby exceptions:

```rust
use solidus::prelude::*;
use std::pin::Pin;

fn might_fail<'ctx>(
    ctx: &'ctx Context,
    should_fail: bool,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    if should_fail {
        Err(Error::runtime("Something went wrong!"))
    } else {
        ctx.new_string("Success!").map_err(Into::into)
    }
}
```

Error types available:
- `Error::runtime(msg)` - RuntimeError
- `Error::argument(msg)` - ArgumentError  
- `Error::type_error(msg)` - TypeError

## Understanding Stack Pinning

Solidus enforces that Ruby values stay visible to the garbage collector. This is
why you see `Pin<&StackPinned<T>>` in function signatures.

**The problem**: Ruby's GC scans the C stack to find live objects. If you move a
Ruby value to the heap (e.g., into a `Vec`), the GC can't find it and may free
the underlying object.

**The solution**: Solidus makes all Ruby VALUE types `!Copy`, so they can't be
accidentally moved to the heap. The macro-generated wrappers handle pinning
values on the stack automatically.

For more details, see [Pinning](pinning.md).

## Next Steps

- [Pinning](pinning.md) - Why Ruby values need pinning and how Solidus enforces it
- [BoxValue](boxvalue.md) - Storing Ruby values on the heap safely
- [Ruby Types](types.md) - Working with RString, RArray, RHash
- [Methods and Functions](methods.md) - Advanced method registration
- [TypedData](typed-data.md) - Wrapping Rust structs as Ruby objects
- [Error Handling](error-handling.md) - Working with Ruby exceptions
- [Examples](../../examples/) - Complete working examples
