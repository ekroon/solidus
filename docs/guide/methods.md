# Defining Ruby Methods in Rust

This guide covers how to define Ruby methods and functions in Rust using Solidus.

## Table of Contents

1. [Overview](#overview)
2. [The `method!` Macro](#the-method-macro)
3. [The `function!` Macro](#the-function-macro)
4. [Attribute Macros](#attribute-macros)
5. [Method Arities](#method-arities)
6. [Argument Types and Pinning](#argument-types-and-pinning)
7. [Return Types and Error Handling](#return-types-and-error-handling)
8. [Registering Methods](#registering-methods)
9. [The `#[solidus::init]` Macro](#the-solidusinit-macro)

## Overview

Solidus provides two approaches for defining Ruby methods:

1. **Declarative macros** (`method!` and `function!`) - Generate wrapper functions inline
2. **Attribute macros** (`#[solidus::method]` and `#[solidus::function]`) - Annotate existing functions

Both approaches handle:

- Panic catching via `std::panic::catch_unwind`
- Type conversion of arguments via `TryConvert`
- Automatic stack pinning of heap-allocated arguments
- Error propagation (converts `Err` to Ruby exceptions)
- Return value conversion via `IntoValue`

## The `method!` Macro

The `method!` macro wraps a Rust function as a Ruby instance method. Instance methods
receive `self` (the Ruby receiver) as the first parameter.

### Basic Syntax

```rust
method!(function_name, arity)
```

### Example: Arity 0 (self only)

```rust
use solidus::prelude::*;

fn length(rb_self: RString) -> Result<i64, Error> {
    Ok(rb_self.len() as i64)
}

// Register the method
let class = ruby.define_class("MyString", ruby.class_string());
let rclass = RClass::try_convert(class)?;
rclass.define_method("length", method!(length, 0), 0)?;
```

### Example: Arity 1 (self + one argument)

```rust
use solidus::prelude::*;
use std::pin::Pin;

fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&format!("{}{}", self_str, other_str)) })
}

// Register the method
rclass.define_method("concat", method!(concat, 1), 1)?;
```

### Example: Arity 2 (self + two arguments)

```rust
fn multiply_three(
    rb_self: RString,
    arg1: Pin<&StackPinned<RString>>,
    arg2: Pin<&StackPinned<RString>>,
) -> Result<i64, Error> {
    let a = rb_self.to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("first argument must be a number"))?;
    let b = arg1.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("second argument must be a number"))?;
    let c = arg2.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("third argument must be a number"))?;
    Ok(a * b * c)
}

rclass.define_method("multiply_three", method!(multiply_three, 2), 2)?;
```

## The `function!` Macro

The `function!` macro wraps a Rust function as a Ruby function (no `self` parameter).
Use this for global functions, module functions, and class methods (singleton methods).

### Basic Syntax

```rust
function!(function_name, arity)
```

### Example: Arity 0 (no arguments)

```rust
use solidus::prelude::*;

fn greet() -> Result<NewValue<RString>, Error> {
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new("Hello, World!") })
}

// Register as a global function
ruby.define_global_function("greet", function!(greet, 0), 0)?;
```

### Example: Arity 1

```rust
use solidus::prelude::*;
use std::pin::Pin;

fn to_upper(s: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let input = s.get().to_string()?;
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&input.to_uppercase()) })
}

// Register as a module function
rmodule.define_module_function("to_upper", function!(to_upper, 1), 1)?;
```

### Example: Arity 2 (self + two arguments)

```rust
fn multiply_three(
    rb_self: RString,
    arg1: Pin<&StackPinned<RString>>,
    arg2: Pin<&StackPinned<RString>>,
) -> Result<i64, Error> {
    let a = rb_self.to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("first argument must be a number"))?;
    let b = arg1.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("second argument must be a number"))?;
    let c = arg2.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("third argument must be a number"))?;
    Ok(a * b * c)
}

rclass.define_method("multiply_three", method!(multiply_three, 2), 2)?;
```

## The `function!` Macro

The `function!` macro wraps a Rust function as a Ruby function (no `self` parameter).
Use this for global functions, module functions, and class methods (singleton methods).

### Basic Syntax

```rust
function!(function_name, arity)
```

### Example: Arity 0 (no arguments)

```rust
use solidus::prelude::*;

fn greet() -> Result<NewValue<RString>, Error> {
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new("Hello, World!") })
}

// Register as a global function
ruby.define_global_function("greet", function!(greet, 0), 0)?;
```

### Example: Arity 1

```rust
use solidus::prelude::*;
use std::pin::Pin;

fn to_upper(s: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let input = s.get().to_string()?;
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&input.to_uppercase()) })
}

// Register as a module function
rmodule.define_module_function("to_upper", function!(to_upper, 1), 1)?;
```

### Example: Arity 2

```rust
fn add_numbers(
    a: Pin<&StackPinned<RString>>,
    b: Pin<&StackPinned<RString>>
) -> Result<i64, Error> {
    let num_a = a.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("first argument must be a number"))?;
    let num_b = b.get().to_string()?.parse::<i64>()
        .map_err(|_| Error::argument("second argument must be a number"))?;
    Ok(num_a + num_b)
}

ruby.define_global_function("add_numbers", function!(add_numbers, 2), 2)?;
```

## Attribute Macros

Attribute macros provide an alternative syntax that generates a companion module
with the wrapper function and arity constant.

### `#[solidus::method]`

```rust
use solidus::prelude::*;
use std::pin::Pin;

#[solidus::method]
fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&format!("{}{}", self_str, other_str)) })
}

// Register using the generated module
rclass.define_method(
    "concat",
    __solidus_method_concat::wrapper(),
    __solidus_method_concat::ARITY,
)?;
```

### `#[solidus::function]`

```rust
use solidus::prelude::*;
use std::pin::Pin;

#[solidus::function]
fn greet(name: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let name_str = name.get().to_string()?;
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&format!("Hello, {}!", name_str)) })
}

// Register using the generated module
ruby.define_global_function(
    "greet",
    __solidus_function_greet::wrapper(),
    __solidus_function_greet::ARITY,
)?;
```

### Generated Module Structure

For a function named `foo`, the attribute macros generate:

```rust
#[doc(hidden)]
pub mod __solidus_method_foo {  // or __solidus_function_foo
    pub const ARITY: i32 = /* number of args */;
    
    pub fn wrapper() -> unsafe extern "C" fn() -> solidus::rb_sys::VALUE {
        // ... wrapper implementation
    }
}
```

## Method Arities

Arity refers to the number of arguments a method accepts (excluding `self` for instance methods).

| Arity | Instance Method (`method!`) | Function (`function!`) |
|-------|----------------------------|------------------------|
| 0     | Just self                  | No arguments           |
| 1     | self + 1 arg               | 1 argument             |
| 2     | self + 2 args              | 2 arguments            |
| 3     | self + 3 args              | 3 arguments            |
| 4     | self + 4 args              | 4 arguments            |

**Currently supported:** Arities 0-4 for both declarative macros. Attribute macros
support arities 0-2.

To add higher arities, extend the macro definitions in
`crates/solidus/src/method/mod.rs`.

## Argument Types and Pinning

### Why Pinning Matters

Ruby's garbage collector scans the C stack to find VALUE references. Solidus
enforces stack pinning at compile time to ensure Ruby VALUES remain visible to
the GC.

### Ruby VALUE Types

For Ruby VALUE types (RString, RArray, RHash, etc.), use `Pin<&StackPinned<T>>`:

```rust
fn example(arg: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    // Access the inner value with .get()
    let s = arg.get().to_string()?;
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&format!("Got: {}", s)) })
}
```

### The Self Parameter

For instance methods, `self` is passed directly as the Ruby type since Ruby
guarantees the receiver is live during the method call:

```rust
fn my_method(rb_self: RString) -> Result<i64, Error> {
    // rb_self is safe to use directly
    Ok(rb_self.len() as i64)
}
```

### Rust Primitive Types

Rust primitive types (i64, f64, bool, String) don't need pinning. The macro
automatically converts Ruby VALUES to these types via `TryConvert`:

```rust
#[solidus::method]
fn repeat(rb_self: RString, count: i64) -> Result<NewValue<RString>, Error> {
    let s = rb_self.to_string()?;
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&s.repeat(count as usize)) })
}
```

### Types That Don't Need Pinning

| Type | Reason |
|------|--------|
| `i8`, `i16`, `i32`, `i64`, `isize` | Rust primitives |
| `u8`, `u16`, `u32`, `u64`, `usize` | Rust primitives |
| `f32`, `f64` | Rust primitives |
| `bool` | Rust primitive |
| `String` | Rust owned string |
| `Fixnum` | Ruby immediate value |
| `Symbol` | Ruby immediate value |
| `Flonum` | Ruby immediate value (64-bit) |

### Types That Need Pinning

| Type | Reason |
|------|--------|
| `RString` | Heap-allocated Ruby object |
| `RArray` | Heap-allocated Ruby object |
| `RHash` | Heap-allocated Ruby object |
| `RClass` | Heap-allocated Ruby object |
| `RModule` | Heap-allocated Ruby object |
| `Value` | Can be any Ruby value |
| `Integer` | Can be Fixnum or Bignum |
| `Float` | Can be Flonum or RFloat |

## Return Types and Error Handling

### Return Type Requirements

Methods must return `Result<T, Error>` where `T` implements `IntoValue`:

```rust
// Return a Ruby string
fn example() -> Result<NewValue<RString>, Error> {
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new("hello") })
}

// Return a Rust integer (converts to Ruby Fixnum/Bignum)
fn example() -> Result<i64, Error> {
    Ok(42)
}

// Return a boolean
fn example() -> Result<bool, Error> {
    Ok(true)
}

// Return nil (unit type)
fn example() -> Result<(), Error> {
    Ok(())
}
```

### Error Handling

Return an `Error` to raise a Ruby exception:

```rust
fn validate(rb_self: RString) -> Result<bool, Error> {
    let s = rb_self.to_string()?;
    
    if s.is_empty() {
        return Err(Error::argument("string cannot be empty"));
    }
    
    if s.len() > 100 {
        return Err(Error::runtime("string too long"));
    }
    
    Ok(true)
}
```

### Error Types

| Function | Ruby Exception |
|----------|----------------|
| `Error::type_error(msg)` | TypeError |
| `Error::argument(msg)` | ArgumentError |
| `Error::runtime(msg)` | RuntimeError |
| `Error::new(class, msg)` | Custom exception class |

### Panic Handling

Panics are caught and converted to Ruby RuntimeError exceptions:

```rust
fn might_panic() -> Result<i64, Error> {
    panic!("Something went wrong!"); // Becomes Ruby RuntimeError
}
```

## Registering Methods

### Instance Methods

Use `define_method` on `RClass` or `RModule`:

```rust
let class = ruby.define_class("MyClass", ruby.class_object());
let rclass = RClass::try_convert(class)?;

rclass.define_method("my_method", method!(my_method, 1), 1)?;
```

### Class Methods (Singleton Methods)

Use `define_singleton_method`:

```rust
// Define a class method like MyClass.create
rclass.define_singleton_method("create", function!(create, 0), 0)?;
```

### Module Functions

Use `define_module_function` for functions callable as both `Module.func` and
via `include`:

```rust
let module = ruby.define_module("MyModule");
let rmodule = RModule::try_convert(module)?;

rmodule.define_module_function("utility", function!(utility, 1), 1)?;
```

### Global Functions

Use `define_global_function` for functions available everywhere:

```rust
ruby.define_global_function("greet", function!(greet, 0), 0)?;
```

### Summary Table

| Ruby Pattern | Solidus Registration |
|--------------|---------------------|
| `obj.method` | `rclass.define_method(...)` |
| `Class.method` | `rclass.define_singleton_method(...)` |
| `Module.method` | `rmodule.define_singleton_method(...)` |
| `Module.func` or via `include` | `rmodule.define_module_function(...)` |
| `global_func` | `ruby.define_global_function(...)` |

## The `#[solidus::init]` Macro

The `#[solidus::init]` macro marks a function as the Ruby extension entry point.

### Basic Usage

```rust
use solidus::prelude::*;

#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Define classes, modules, methods here
    let class = ruby.define_class("MyClass", ruby.class_object());
    
    Ok(())
}
```

### Custom Extension Name

By default, the generated `Init_` function uses your crate name. Override it with
the `name` parameter:

```rust
#[solidus::init(name = "my_custom_extension")]
fn init(ruby: &Ruby) -> Result<(), Error> {
    Ok(())
}
// Generates: Init_my_custom_extension
```

### Generated Code

The macro generates:

```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Init_your_crate_name() {
    std::panic::catch_unwind(|| {
        solidus::Ruby::mark_ruby_thread();
        let ruby = solidus::Ruby::get();
        if let Err(e) = init(ruby) {
            e.raise();
        }
    });
}
```

### Complete Example

```rust
use solidus::prelude::*;
use std::pin::Pin;

// Instance method
fn greet(rb_self: RString) -> Result<NewValue<RString>, Error> {
    let name = rb_self.to_string()?;
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&format!("Hello, {}!", name)) })
}

// Class method
fn create_default() -> Result<NewValue<RString>, Error> {
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new("default") })
}

// Global function
fn hello() -> Result<NewValue<RString>, Error> {
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new("Hello from Solidus!") })
}

#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Define a class with methods
    let class = ruby.define_class("Greeter", ruby.class_string());
    let rclass = RClass::try_convert(class)?;
    
    rclass.clone().define_method("greet", method!(greet, 0), 0)?;
    rclass.define_singleton_method("create_default", function!(create_default, 0), 0)?;
    
    // Define a global function
    ruby.define_global_function("hello", function!(hello, 0), 0)?;
    
    Ok(())
}
```

## Further Reading

- [Examples: phase3_methods](../../examples/phase3_methods/) - Comprehensive method examples
- [Examples: phase3_attr_macros](../../examples/phase3_attr_macros/) - Attribute macro examples
- [Pinning](pinning.md) - Why Ruby values need pinning and how Solidus enforces it
- [Ruby Types](types.md) - Working with RString, RArray, etc.
- [Error Handling](error-handling.md) - Working with Ruby exceptions
