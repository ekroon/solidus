# Phase 3: Method Registration

## Objective

Implement the `method!` and `function!` macros that automatically handle stack pinning
of Ruby value arguments.

## Dependencies

- Phase 1 complete
- Phase 2 mostly complete (at least RString, basic types)

## Core Concepts

### Method Wrapper Generation

The `method!` macro generates an `extern "C"` function that:

1. Receives raw `VALUE` arguments from Ruby
2. Converts each argument to its Rust type
3. **Stack-pins heap values** using `Pin::new_unchecked`
4. Calls the user's function
5. Converts the return value back to Ruby
6. Handles panics and errors

### Argument Classification

Arguments are classified at compile-time:

| Rust Type | Classification | Pinning |
|-----------|---------------|---------|
| `i64`, `u64`, etc. | Immediate | No |
| `bool` | Immediate | No |
| `Fixnum` | Immediate | No |
| `Symbol` | Immediate | No |
| `RString` | Heap | **Yes** |
| `RArray` | Heap | **Yes** |
| `Pin<&StackPinned<T>>` | Explicit pinned | Yes |
| `&T` where T: ReprValue | Reference | Yes (implicit) |

### Self Parameter

The `self` parameter is always passed by value (not pinned) since:
- It's on the stack in the wrapper function
- It can't be moved during the method call
- Pinning would be redundant

## Tasks

### 3.1 Method Traits

```rust
// crates/solidus/src/method/mod.rs

/// Return type for Ruby methods.
pub trait ReturnValue {
    fn into_return_value(self) -> Result<Value, Error>;
}

impl<T: IntoValue> ReturnValue for T { ... }
impl<T: IntoValue> ReturnValue for Result<T, Error> { ... }

/// Marker trait for types that can be method arguments.
pub trait MethodArg {
    /// Whether this type requires pinning.
    const NEEDS_PINNING: bool;
}
```

- [ ] Define `ReturnValue` trait
- [ ] Define `MethodArg` trait
- [ ] Implement for all relevant types

### 3.2 Method Macro (Arity 0-3)

Start with low arities to prove the concept:

```rust
// Usage:
class.define_method("length", method!(MyClass::length, 0))?;
class.define_method("concat", method!(concat, 1))?;
class.define_method("insert", method!(insert, 2))?;

// Generated wrapper for arity 1:
unsafe extern "C" fn wrapper(rb_self: VALUE, arg0: VALUE) -> VALUE {
    let result = std::panic::catch_unwind(|| {
        let self_converted = RString::try_convert(Value::from_raw(rb_self))?;
        let arg0_converted = RString::try_convert(Value::from_raw(arg0))?;
        
        // Stack pin the argument
        let mut pinned = StackPinned::new(arg0_converted);
        let pin = Pin::new_unchecked(&mut pinned);
        
        concat(self_converted, pin).into_return_value()
    });
    
    match result {
        Ok(Ok(v)) => v.as_raw(),
        Ok(Err(e)) => e.raise(),
        Err(panic) => Error::from_panic(panic).raise(),
    }
}
```

- [ ] Implement `method!` for arity 0
- [ ] Implement `method!` for arity 1
- [ ] Implement `method!` for arity 2
- [ ] Implement `method!` for arity 3
- [ ] Add comprehensive tests

### 3.3 Method Macro (Arity 4-15)

Extend to full arity range:

- [ ] Use `seq_macro` or similar to reduce repetition
- [ ] Implement arities 4-15
- [ ] Test edge cases

### 3.4 Function Macro

For methods without `self`:

```rust
// Usage:
ruby.define_global_function("greet", function!(greet, 1))?;

fn greet(name: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
    let name_str = name.get().to_string()?;
    RString::new(&format!("Hello, {}!", name_str))
}
```

- [ ] Implement `function!` macro
- [ ] Test with various arities

### 3.5 Mixed Argument Support

Support methods with both pinned and non-pinned arguments:

```rust
fn example(
    rb_self: RString,
    count: i64,                           // Not pinned (immediate)
    other: Pin<&StackPinned<RString>>,    // Pinned
) -> Result<RString, Error>
```

The macro needs to:
1. Detect which arguments need pinning
2. Generate appropriate conversion code for each

- [ ] Implement argument classification
- [ ] Generate correct pinning code per argument
- [ ] Test mixed argument combinations

### 3.6 Keyword Arguments (Optional)

Consider support for Ruby keyword arguments:

```rust
fn example(
    rb_self: RString,
    kwargs: KwArgs,
) -> Result<RString, Error>
```

- [ ] Research Ruby kwargs API
- [ ] Design Rust API
- [ ] Implement if time permits

### 3.7 Block Arguments

Support for methods that accept blocks:

```rust
fn each_line(
    rb_self: RString,
    block: Proc,
) -> Result<RString, Error> {
    for line in rb_self.lines()? {
        block.call((line,))?;
    }
    Ok(rb_self)
}
```

- [ ] Implement `Proc` type (if not done in phase 2)
- [ ] Add block detection to method macro
- [ ] Test block handling

### 3.8 Init Macro

The `#[solidus::init]` attribute macro:

```rust
#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("MyClass", ruby.class_object())?;
    class.define_method("foo", method!(foo, 1))?;
    Ok(())
}
```

Generates:

```rust
#[no_mangle]
pub extern "C" fn Init_extension_name() {
    unsafe {
        let ruby = Ruby::get();
        if let Err(e) = init(ruby) {
            e.raise();
        }
    }
}
```

- [ ] Implement in `solidus-macros` crate
- [ ] Detect extension name from crate name
- [ ] Handle errors properly
- [ ] Add tests

## Acceptance Criteria

- [ ] `method!` works for arities 0-15
- [ ] `function!` works for arities 0-15
- [ ] Mixed pinned/non-pinned arguments work
- [ ] `#[solidus::init]` generates correct init function
- [ ] Panic handling works correctly
- [ ] Error propagation works correctly
- [ ] All tests pass
- [ ] Implicit pinning works without relying on Copy (ADR-007)

## Implicit Pinning Redesign (ADR-007)

**Status**: Design in progress, implementation pending.

The current `#[solidus_macros::method]` and `#[solidus_macros::function]` attribute
macros rely on VALUE types being `Copy` to provide implicit pinning. This is unsafe
because copies can escape to the heap.

### Current (Unsafe) Approach

```rust
#[solidus_macros::method]
fn concat(rb_self: RString, other: RString) -> Result<RString, Error> {
    // `other` appears to be pinned, but since RString is Copy,
    // user could store `other` in a Vec here - unsafe!
}
```

The macro pins the original VALUE, then **copies** it to pass to the user function.
The user receives an unprotected copy.

### Goal: Implicit Pinning Without Copy

We want to keep the ergonomic API but make it safe:

```rust
#[solidus_macros::method]
fn concat(rb_self: RString, other: RString) -> Result<RString, Error> {
    // This should be safe even though signature looks simple
}
```

### Design Options

#### Option A: Pass by Reference

The macro passes `&T` instead of `T`:

```rust
// User writes:
fn concat(rb_self: &RString, other: &RString) -> Result<RString, Error>

// Macro generates wrapper that:
// 1. Pins values on stack
// 2. Passes references to pinned values
```

**Pros**: Simple, idiomatic Rust
**Cons**: User must use `&T` syntax, can still call `.clone()` if Clone is implemented

#### Option B: Pass Lifetime-Bound Wrapper

Create a newtype that can't escape:

```rust
/// A reference to a VALUE that is tied to the current stack frame
pub struct ValueRef<'a, T>(&'a StackPinned<T>);

// User writes:
fn concat(rb_self: ValueRef<'_, RString>, other: ValueRef<'_, RString>) 
    -> Result<RString, Error>
```

**Pros**: Type makes borrowing explicit, can't implement Clone
**Cons**: More verbose than plain `RString`

#### Option C: Interior Reference Pattern

The macro generates code that passes a short-lived borrow:

```rust
// User writes (unchanged from current):
fn concat(rb_self: RString, other: RString) -> Result<RString, Error>

// Macro generates:
unsafe extern "C" fn wrapper(rb_self: VALUE, arg0: VALUE) -> VALUE {
    // Pin values
    let mut self_pinned = StackPinned::new(RString::from_raw(rb_self));
    let mut arg0_pinned = StackPinned::new(RString::from_raw(arg0));
    
    // Create non-Copy handles that borrow from pinned storage
    let self_ref = RString::borrow_from(&self_pinned);
    let arg0_ref = RString::borrow_from(&arg0_pinned);
    
    concat(self_ref, arg0_ref)
}
```

This requires `RString` to have a lifetime parameter or use internal borrows.

**Pros**: Keeps simple `RString` signature
**Cons**: Complex implementation, may need lifetime parameters on types

#### Option D: Rely on !Copy Alone

Simply making types `!Copy` may be sufficient:

```rust
// User writes:
fn concat(rb_self: RString, other: RString) -> Result<RString, Error>

// Since RString is !Copy, moving it into a Vec would move it out of the function.
// The macro can pass by value if the user function consumes the value.
```

**Pros**: Simplest implementation
**Cons**: User can still call `.clone()` if Clone is implemented (but that's expensive and obvious)

### Recommended Approach

**Option A (Pass by Reference)** is recommended:

1. Simplest to implement
2. Idiomatic Rust - passing `&T` is natural
3. Users understand that `&T` is borrowed
4. If types also implement `!Clone` (or have expensive Clone), escape is prevented

The attribute macro would transform:

```rust
#[solidus_macros::method]
fn concat(rb_self: &RString, other: &RString) -> Result<RString, Error> {
    // Safe! Cannot store `other` anywhere - it's a reference
}
```

Into:

```rust
fn concat(rb_self: &RString, other: &RString) -> Result<RString, Error> { ... }

unsafe extern "C" fn concat_wrapper(rb_self: VALUE, arg0: VALUE) -> VALUE {
    let mut self_pinned = StackPinned::new(RString::from_raw(rb_self));
    let mut arg0_pinned = StackPinned::new(RString::from_raw(arg0));
    
    let result = concat(
        self_pinned.get_ref(),
        arg0_pinned.get_ref(),
    );
    // ... handle result
}
```

### Implementation Plan

See [phase-3-tasks.md](phase-3-tasks.md#stage-3-ergonomic-method-macro-implicit-pinning)
for updated tasks.

### Migration Path

1. Update Phase 2 types to be `!Copy` (see phase-2-tasks.md Stage 10)
2. Update attribute macros to pass `&T` instead of `T`
3. Update existing examples and tests
4. Document the new pattern
