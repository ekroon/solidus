# Method and Function Macro Examples

This document provides examples of using the `method!` and `function!` macros.

## method! - Instance Methods

The `method!` macro is used to wrap Rust functions as Ruby instance methods.
The first parameter is always `ctx: &'ctx Context`, followed by `rb_self` (the receiver).

```rust
use solidus::prelude::*;
use std::pin::Pin;

// Arity 0: just self (no new values created, no Context needed)
fn length(_ctx: &Context, rb_self: RString) -> Result<i64, Error> {
    Ok(rb_self.len() as i64)
}

// Arity 1: self + one argument (creates new value, needs Context)
fn concat<'ctx>(
    ctx: &'ctx Context,
    rb_self: RString,
    other: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    // other is automatically pinned on the stack by the wrapper
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    let result = format!("{}{}", self_str, other_str);
    ctx.new_string(&result).map_err(Into::into)
}

// Arity 2: self + two arguments (mutates existing value, no Context needed)
fn insert(
    _ctx: &Context,
    rb_self: RArray,
    index: Pin<&StackPinned<Integer>>,
    value: Pin<&StackPinned<Value>>,
) -> Result<RArray, Error> {
    // Both arguments are pinned
    // In real implementation, would call rb_ary_store or similar
    Ok(rb_self)
}

// Register methods with Ruby
fn register_methods(class: RClass) -> Result<(), Error> {
    // class.define_method("length", method!(length, 0), 0)?;
    // class.define_method("concat", method!(concat, 1), 1)?;
    // class.define_method("insert", method!(insert, 2), 2)?;
    Ok(())
}
```

## function! - Module/Global Functions

The `function!` macro is used for module functions and global functions.
Unlike `method!`, there is no `rb_self` parameter.

```rust
use solidus::prelude::*;
use std::pin::Pin;

// Arity 0: no arguments
fn hello_world<'ctx>(ctx: &'ctx Context) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    ctx.new_string("Hello, World!").map_err(Into::into)
}

// Arity 1: one argument
fn greet<'ctx>(
    ctx: &'ctx Context,
    name: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let name_str = name.get().to_string()?;
    ctx.new_string(&format!("Hello, {}!", name_str))
        .map_err(Into::into)
}

// Arity 2: two arguments (returns immediate value, Context unused)
fn add(
    _ctx: &Context,
    a: Pin<&StackPinned<Integer>>,
    b: Pin<&StackPinned<Integer>>,
) -> Result<Integer, Error> {
    let a_val = a.get().to_i64()?;
    let b_val = b.get().to_i64()?;
    Ok(Integer::from(a_val + b_val))
}

// Register functions with Ruby
fn register_functions(ruby: &Ruby) -> Result<(), Error> {
    // ruby.define_global_function("hello_world", function!(hello_world, 0), 0)?;
    // ruby.define_global_function("greet", function!(greet, 1), 1)?;
    // ruby.define_global_function("add", function!(add, 2), 2)?;
    Ok(())
}
```

## Context Parameter

Both `method!` and `function!` require a `ctx: &'ctx Context` parameter as the first parameter:

- For **instance methods**: `ctx` is first, followed by `rb_self`, then other arguments
- For **module/global functions**: `ctx` is first, then other arguments

The Context provides stack-allocated storage for creating new Ruby values. If your function:
- **Creates new Ruby values**: Use `ctx.new_string()`, `ctx.new_array()`, etc. and return `Pin<&'ctx StackPinned<T>>`
- **Only returns existing values or primitives**: You still receive `ctx` but may prefix it with `_ctx` if unused

## Key Differences

| Aspect | method! | function! |
|--------|---------|-----------|
| First parameter | Always `ctx: &'ctx Context`, then `rb_self` (receiver) | Always `ctx: &'ctx Context` |
| Use case | Instance methods | Module/global functions |
| Ruby API | `rb_define_method` | `rb_define_global_function`, `rb_define_module_function` |

## Supported Arities

Both macros currently support arities 0-4:

- `method!` or `function!` with arity 0-4
- Arities 5-15 can be added following the same pattern

## Panic Handling

Both macros automatically catch panics and convert them to Ruby exceptions:

```rust
fn might_panic(_ctx: &Context, rb_self: Value) -> Result<Value, Error> {
    if some_condition {
        panic!("Something went wrong!");  // Caught and converted to Ruby exception
    }
    Ok(rb_self)
}
```

## Error Propagation

Both macros handle `Result<T, Error>` return types and propagate errors to Ruby:

```rust
fn might_fail(_ctx: &Context, rb_self: Value) -> Result<Value, Error> {
    Err(Error::runtime_error("Operation failed"))  // Raised as Ruby exception
}
```

## Stack Pinning

Both macros automatically pin heap-allocated Ruby values on the stack to prevent
GC issues. The pinning is handled transparently by the wrapper:

```rust
// The wrapper generates code like:
// let arg_converted = RString::try_convert(arg_value)?;
// pin_on_stack!(arg_pinned = arg_converted);
// my_function(ctx, rb_self, arg_pinned);  // arg_pinned is Pin<&StackPinned<RString>>
```
