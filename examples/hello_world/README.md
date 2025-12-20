# Hello World - Minimal Solidus Example

This is the simplest possible Solidus Ruby extension, demonstrating the basic project structure and a single global function.

## What This Example Shows

- Basic project structure for a Solidus extension
- Using `#[solidus::init]` to initialize the extension
- Defining a global function callable from Ruby

## Building

```bash
cargo build --manifest-path examples/hello_world/Cargo.toml
```

## Running

```bash
ruby examples/hello_world/test.rb
```

## Code Overview

```rust
use solidus::prelude::*;

#[solidus_macros::function]
fn hello<'ctx>(ctx: &'ctx Context) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    ctx.new_string("Hello from Solidus!").map_err(Into::into)
}

#[solidus_macros::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    ruby.define_global_function(
        "hello",
        __solidus_function_hello::wrapper(),
        __solidus_function_hello::ARITY,
    )?;
    Ok(())
}
```

Then from Ruby:

```ruby
require 'hello_world'
puts hello()  # => "Hello from Solidus!"
```
