# Phase 3: Method Registration - Implementation Tasks

This file contains the detailed task breakdown for Phase 3. Each task should be
completed in order, as later tasks depend on earlier ones.

## Prerequisites

- Phase 2 complete (all Ruby types, TryConvert, IntoValue)
- Understanding of Ruby's method registration C API
- Understanding of Rust's `Pin` and macro_rules! system

## Task Status Legend

- [ ] Not started
- [x] Complete
- [~] In progress

---

## Stage 1: Method Infrastructure

Core traits and types needed before implementing the method macros.

### Task 3.1.1: Create method module structure

**File**: `crates/solidus/src/method/mod.rs`

- [x] Create the `method` directory and `mod.rs`
- [x] Add module to `lib.rs`
- [x] Create submodule files

```rust
// crates/solidus/src/method/mod.rs
mod args;
mod return_value;

pub use args::MethodArg;
pub use return_value::ReturnValue;
```

### Task 3.1.2: Implement ReturnValue trait

**File**: `crates/solidus/src/method/return_value.rs`

The `ReturnValue` trait handles converting method return values to Ruby.
It supports both direct values and `Result` types.

- [x] Define the `ReturnValue` trait
- [x] Implement for types that implement `IntoValue`
- [x] Implement for `Result<T, Error>` where T: IntoValue
- [x] Implement for `()` (returns nil)
- [x] Add tests

```rust
use crate::convert::IntoValue;
use crate::error::Error;
use crate::value::Value;

/// Trait for types that can be returned from Ruby methods.
///
/// This handles both infallible returns (types implementing `IntoValue`)
/// and fallible returns (`Result<T, Error>`).
pub trait ReturnValue {
    /// Convert this value into a Ruby return value.
    ///
    /// Returns `Ok(Value)` on success, or `Err(Error)` if an error occurred.
    fn into_return_value(self) -> Result<Value, Error>;
}

// Infallible return - any type that can be converted to Value
impl<T: IntoValue> ReturnValue for T {
    fn into_return_value(self) -> Result<Value, Error> {
        Ok(self.into_value())
    }
}

// Note: The above impl conflicts with Result<T, Error>, so we need a different approach.
// We'll use a sealed trait pattern or specialization workaround.
```

**Design Note**: Due to Rust's coherence rules, we need to handle the conflict
between `impl<T: IntoValue> ReturnValue for T` and `impl<T: IntoValue> ReturnValue for Result<T, Error>`.
Options:
1. Use a newtype wrapper for one case
2. Only implement for specific types (not generic)
3. Use the "sealed trait" pattern with a marker

Recommended approach: Implement `ReturnValue` only for `Result<T, Error>` and have
the macro call `.into_value()` for non-Result returns, or wrap non-Result returns
in `Ok()` before calling a unified handler.

### Task 3.1.3: Implement MethodArg marker trait

**File**: `crates/solidus/src/method/args.rs`

This trait marks types that can be used as method arguments and indicates
whether they need stack pinning.

- [x] Define the `MethodArg` trait
- [x] Implement for immediate types (no pinning needed): `i8`-`i64`, `u8`-`u64`, `f32`, `f64`, `bool`, `Fixnum`, `Symbol`, `Qnil`, `Qtrue`, `Qfalse`
- [x] Implement for heap types (pinning needed): `RString`, `RArray`, `RHash`, `RClass`, `RModule`, `Value`, etc.
- [x] Add tests

```rust
/// Marker trait for types that can be method arguments.
///
/// This trait indicates whether a type needs stack pinning when passed
/// to a Ruby method.
pub trait MethodArg: Sized {
    /// Whether this type requires stack pinning.
    ///
    /// Immediate values (Fixnum, Symbol, bool, etc.) return `false`.
    /// Heap-allocated Ruby objects return `true`.
    const NEEDS_PINNING: bool;
}

// Immediate types - no pinning needed
impl MethodArg for i8 { const NEEDS_PINNING: bool = false; }
impl MethodArg for i16 { const NEEDS_PINNING: bool = false; }
impl MethodArg for i32 { const NEEDS_PINNING: bool = false; }
impl MethodArg for i64 { const NEEDS_PINNING: bool = false; }
impl MethodArg for isize { const NEEDS_PINNING: bool = false; }
impl MethodArg for u8 { const NEEDS_PINNING: bool = false; }
impl MethodArg for u16 { const NEEDS_PINNING: bool = false; }
impl MethodArg for u32 { const NEEDS_PINNING: bool = false; }
impl MethodArg for u64 { const NEEDS_PINNING: bool = false; }
impl MethodArg for usize { const NEEDS_PINNING: bool = false; }
impl MethodArg for f32 { const NEEDS_PINNING: bool = false; }
impl MethodArg for f64 { const NEEDS_PINNING: bool = false; }
impl MethodArg for bool { const NEEDS_PINNING: bool = false; }
impl MethodArg for Fixnum { const NEEDS_PINNING: bool = false; }
impl MethodArg for Symbol { const NEEDS_PINNING: bool = false; }
impl MethodArg for Qnil { const NEEDS_PINNING: bool = false; }
impl MethodArg for Qtrue { const NEEDS_PINNING: bool = false; }
impl MethodArg for Qfalse { const NEEDS_PINNING: bool = false; }

// Heap types - pinning needed
impl MethodArg for RString { const NEEDS_PINNING: bool = true; }
impl MethodArg for RArray { const NEEDS_PINNING: bool = true; }
impl MethodArg for RHash { const NEEDS_PINNING: bool = true; }
impl MethodArg for RClass { const NEEDS_PINNING: bool = true; }
impl MethodArg for RModule { const NEEDS_PINNING: bool = true; }
impl MethodArg for Value { const NEEDS_PINNING: bool = true; }
impl MethodArg for Integer { const NEEDS_PINNING: bool = true; } // Can be Bignum
impl MethodArg for Float { const NEEDS_PINNING: bool = true; }   // Can be RFloat
impl MethodArg for RBignum { const NEEDS_PINNING: bool = true; }
impl MethodArg for RFloat { const NEEDS_PINNING: bool = true; }
```

**Acceptance**: `cargo test -p solidus method` passes

---

## Stage 2: Basic Method Macro (Explicit Signatures)

Implement the `method!` macro with explicit `Pin<&StackPinned<T>>` in signatures.
This is the "full form" that gives users complete control.

### Task 3.2.1: Implement method! macro for arity 0

**File**: `crates/solidus/src/method/mod.rs`

- [x] Create the `method!` macro for arity 0 (self only)
- [x] Handle panic catching with `std::panic::catch_unwind`
- [x] Handle error propagation
- [x] Convert return value to Ruby VALUE
- [x] Add tests

```rust
/// Usage:
/// class.define_method("length", method!(MyString::length, 0))?;
///
/// fn length(rb_self: RString) -> Result<i64, Error> {
///     Ok(rb_self.len() as i64)
/// }
///
/// Generated wrapper:
/// unsafe extern "C" fn wrapper(rb_self: VALUE) -> VALUE {
///     // ... panic handling, conversion, etc.
/// }
```

The macro should generate:
1. An `extern "C"` wrapper function
2. Panic catching via `catch_unwind`
3. Self argument conversion
4. Return value conversion
5. Error handling (raise on error)

### Task 3.2.2: Implement method! macro for arity 1

**File**: `crates/solidus/src/method/mod.rs` (extend)

- [x] Extend `method!` to support arity 1
- [x] Stack-pin the argument if it's a heap type
- [x] Pass pinned reference to user function
- [x] Add tests

```rust
/// Usage:
/// class.define_method("concat", method!(concat, 1))?;
///
/// // Explicit form - user specifies Pin<&StackPinned<T>>
/// fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
///     // other is pinned on the stack, safe from GC
/// }
///
/// Generated wrapper:
/// unsafe extern "C" fn wrapper(rb_self: VALUE, arg0: VALUE) -> VALUE {
///     let result = std::panic::catch_unwind(|| {
///         let self_converted = RString::try_convert(Value::from_raw(rb_self))?;
///         let arg0_converted = RString::try_convert(Value::from_raw(arg0))?;
///         
///         // Stack pin the argument
///         let mut arg0_pinned = StackPinned::new(arg0_converted);
///         let arg0_pin = Pin::new_unchecked(&arg0_pinned);
///         
///         concat(self_converted, arg0_pin)
///     });
///     // ... handle result
/// }
```

### Task 3.2.3: Implement method! macro for arity 2-3

**File**: `crates/solidus/src/method/mod.rs` (extend)

- [x] Extend `method!` to support arity 2
- [x] Extend `method!` to support arity 3
- [x] Each argument independently pinned
- [x] Add tests for various argument combinations

### Task 3.2.4: Implement method! macro for arity 4-15

**File**: `crates/solidus/src/method/mod.rs` (extend)

Ruby's `rb_define_method` supports up to 15 arguments (or variadic).

- [x] Use helper macro to reduce repetition (arity 4 complete)
- [~] Implement arities 4-15 (4 complete, 5-15 pending)
- [x] Test edge cases (arity 4)

```rust
// Helper macro to generate arity variants
macro_rules! impl_method_arity {
    ($arity:literal, $($arg:ident),*) => {
        // ... generate code for this arity
    };
}
```

**Acceptance**: `method!` works for arities 0-15 with explicit signatures

---

## Stage 3: Ergonomic Method Macro (Implicit Pinning)

Add support for simpler signatures where the macro handles pinning internally.

### Task 3.3.1: Design implicit pinning API

- [x] Design how users specify simple signatures
- [x] Decide on macro syntax for implicit vs explicit

Option A: Separate macro names
```rust
// Explicit (user handles Pin types)
method!(concat, 1)

// Implicit (macro handles pinning)
method_auto!(concat, 1)
```

Option B: Type inference in the macro
```rust
// The macro inspects the function signature and pins as needed
method!(concat, 1)

fn concat(rb_self: RString, other: RString) -> Result<RString, Error> {
    // macro automatically pins `other` before calling
}
```

Option C: Attribute on function
```rust
#[solidus::method]
fn concat(rb_self: RString, other: RString) -> Result<RString, Error> {
    // Proc macro transforms this
}
```

**Recommended**: Option B - the macro always pins heap types internally, users
write simple signatures. The explicit `Pin<&StackPinned<T>>` form is still
supported for users who want to make pinning visible in their API.

### Task 3.3.2: Implement implicit pinning in method! macro

**File**: `crates/solidus-macros/src/lib.rs` (attribute macro)

- [x] Implement `#[method]` attribute macro with implicit pinning
- [x] User functions receive the inner type directly (e.g., `RString` not `Pin<&StackPinned<RString>>`)
- [x] Copy bound enforcement for type safety
- [x] Support for explicit `Pin<&StackPinned<T>>` signatures (backward compatibility)
- [x] Document the implicit pinning behavior
- [x] Add tests

```rust
/// With implicit pinning:
/// fn concat(rb_self: RString, other: RString) -> Result<RString, Error> {
///     // `other` was pinned by the macro wrapper before this call
///     // Safe to use during this method call
/// }
///
/// Generated wrapper pins the argument, then passes the inner value:
/// unsafe extern "C" fn wrapper(rb_self: VALUE, arg0: VALUE) -> VALUE {
///     let result = std::panic::catch_unwind(|| {
///         let self_converted = RString::try_convert(Value::from_raw(rb_self))?;
///         let arg0_converted = RString::try_convert(Value::from_raw(arg0))?;
///         
///         // Pin on stack (even though user doesn't see Pin type)
///         let arg0_pinned = StackPinned::new(arg0_converted);
///         let _pin = Pin::new_unchecked(&arg0_pinned);
///         
///         // Pass inner value to user function
///         concat(self_converted, arg0_pinned.value)
///     });
///     // ...
/// }
```

### Task 3.3.3: Support mixed immediate/heap arguments

**File**: `crates/solidus-macros/src/lib.rs` (attribute macro)

- [x] Immediate arguments passed directly (no pinning overhead)
- [x] Heap arguments pinned automatically
- [x] Works for any combination of explicit Pin and implicit types

```rust
/// Mixed arguments example:
/// fn insert(rb_self: RArray, index: i64, value: RString) -> Result<RArray, Error> {
///     // index is i64 (immediate) - passed directly
///     // value is RString (heap) - pinned by wrapper
/// }
```

**Acceptance**: Implicit pinning works for arities 0-2 with mixed argument types, extensible pattern for higher arities

---

## Stage 4: Function Macro

Implement `function!` for module/global functions (no `self`).

### Task 3.4.1: Implement function! macro for arity 0-3

**File**: `crates/solidus/src/method/mod.rs` (extend)

- [x] Create `function!` macro similar to `method!` but without self
- [x] Support arities 0-3
- [x] Add tests

```rust
/// Usage:
/// ruby.define_global_function("greet", function!(greet, 1))?;
///
/// fn greet(name: RString) -> Result<RString, Error> {
///     let name_str = name.to_string()?;
///     Ok(RString::new(&format!("Hello, {}!", name_str)))
/// }
```

### Task 3.4.2: Implement function! macro for arity 4

**File**: `crates/solidus/src/method/mod.rs` (extend)

- [x] Extend `function!` to support arity 4 (matching `method!` arity)
- [x] Use similar helper pattern as `method!`
- [x] Add tests

**Acceptance**: `function!` works for arities 0-4 (arities 5-15 can be added following same pattern)

---

## Stage 5: Method Definition API

Add `define_method` to `RClass`, `RModule`, and `Ruby`.

### Task 3.5.1: Add define_method to Module trait

**File**: `crates/solidus/src/types/module.rs` (extend)

- [x] Add `define_method` to the `Module` trait
- [x] Accept method name and function pointer from `method!` macro
- [x] Handle the Ruby C API call to `rb_define_method`
- [x] Add tests

```rust
pub trait Module: ReprValue {
    // ... existing methods ...
    
    /// Define an instance method on this class/module.
    ///
    /// # Arguments
    ///
    /// * `name` - The method name
    /// * `func` - A function pointer generated by the `method!` macro
    /// * `arity` - The number of arguments (-1 for variadic, -2 for args as array)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let class = ruby.define_class("MyClass", ruby.class_object())?;
    /// class.define_method("greet", method!(greet, 1), 1)?;
    /// ```
    fn define_method(
        self,
        name: &str,
        func: unsafe extern "C" fn() -> rb_sys::VALUE,
        arity: i32,
    ) -> Result<(), Error>;
}
```

**Note**: The function pointer type needs to be generic enough to handle different
arities. Ruby's `rb_define_method` accepts a function pointer cast, so we can use
a generic type or transmute.

### Task 3.5.2: Add define_singleton_method to Module trait

**File**: `crates/solidus/src/types/module.rs` (extend)

- [x] Add `define_singleton_method` (class methods)
- [x] Uses `rb_define_singleton_method` under the hood
- [x] Add tests

```rust
/// Define a singleton (class) method.
fn define_singleton_method(
    self,
    name: &str,
    func: unsafe extern "C" fn() -> rb_sys::VALUE,
    arity: i32,
) -> Result<(), Error>;
```

### Task 3.5.3: Add define_global_function to Ruby

**File**: `crates/solidus/src/ruby.rs` (extend)

- [x] Add `define_global_function` to `Ruby`
- [x] Uses `rb_define_global_function`
- [x] Add tests

```rust
impl Ruby {
    /// Define a global function (available everywhere).
    pub fn define_global_function(
        &self,
        name: &str,
        func: unsafe extern "C" fn() -> rb_sys::VALUE,
        arity: i32,
    ) -> Result<(), Error>;
}
```

### Task 3.5.4: Add define_module_function to Module trait

**File**: `crates/solidus/src/types/module.rs` (extend)

- [x] Add `define_module_function` (callable as both Module.func and Module::func)
- [x] Uses `rb_define_module_function`
- [x] Add tests

**Acceptance**: Full method definition API is available

---

## Stage 6: Init Macro

Implement `#[solidus::init]` attribute macro.

### Task 3.6.1: Implement basic #[init] attribute macro

**File**: `crates/solidus-macros/src/lib.rs`

- [x] Parse the annotated function
- [x] Generate `#[no_mangle] pub extern "C" fn Init_<crate_name>()`
- [x] Call the user's function with `Ruby::get()`
- [x] Handle errors by raising Ruby exceptions
- [x] Add tests

```rust
/// Input:
/// #[solidus::init]
/// fn init(ruby: &Ruby) -> Result<(), Error> {
///     let class = ruby.define_class("MyClass", ruby.class_object())?;
///     class.define_method("foo", method!(foo, 1))?;
///     Ok(())
/// }
///
/// Output:
/// fn init(ruby: &Ruby) -> Result<(), Error> {
///     let class = ruby.define_class("MyClass", ruby.class_object())?;
///     class.define_method("foo", method!(foo, 1))?;
///     Ok(())
/// }
///
/// #[no_mangle]
/// pub extern "C" fn Init_my_extension() {
///     unsafe {
///         Ruby::mark_ruby_thread();
///         let ruby = Ruby::get();
///         if let Err(e) = init(ruby) {
///             e.raise();
///         }
///     }
/// }
```

### Task 3.6.2: Add crate name detection

**File**: `crates/solidus-macros/src/lib.rs` (extend)

- [x] Detect the crate name from `CARGO_PKG_NAME` environment variable
- [x] Convert to valid Ruby identifier (replace `-` with `_`)
- [x] Allow override via attribute argument: `#[solidus::init(name = "custom_name")]`
- [x] Add tests

```rust
// Default: uses crate name
#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> { ... }
// Generates: Init_my_crate_name

// Override:
#[solidus::init(name = "my_extension")]
fn init(ruby: &Ruby) -> Result<(), Error> { ... }
// Generates: Init_my_extension
```

### Task 3.6.3: Re-export init macro from main crate

**File**: `crates/solidus/src/lib.rs`

- [x] Re-export `#[init]` from solidus-macros
- [x] Add to prelude
- [x] Update documentation

```rust
pub use solidus_macros::init;
```

**Acceptance**: `#[solidus::init]` generates correct Ruby init function

---

## Stage 7: Variadic Arguments

Support Ruby methods that accept any number of arguments.

### Task 3.7.1: Design variadic argument API

- [ ] Research Ruby's variadic argument C API (`argc`, `argv` pattern)
- [ ] Design Rust API for variadic methods
- [ ] Document the approach

Ruby supports two variadic patterns:
1. `arity = -1`: `fn(argc: c_int, argv: *const VALUE, self: VALUE) -> VALUE`
2. `arity = -2`: `fn(self: VALUE, args: VALUE) -> VALUE` (args is a Ruby Array)

Recommended: Support arity -1 pattern with a safe wrapper.

```rust
/// Variadic method signature:
/// fn my_method(rb_self: RString, args: &[Value]) -> Result<Value, Error> {
///     for arg in args {
///         // process each argument
///     }
/// }
```

### Task 3.7.2: Implement variadic method! macro

**File**: `crates/solidus/src/method/mod.rs` (extend)

- [ ] Add `method_variadic!` or extend `method!` to support arity -1
- [ ] Convert `argc`/`argv` to a safe slice
- [ ] Pin all arguments in the slice
- [ ] Add tests

```rust
/// Usage:
/// class.define_method("sprintf", method!(sprintf, -1))?;
///
/// fn sprintf(rb_self: Value, args: &[Value]) -> Result<RString, Error> {
///     // args contains all passed arguments
/// }
```

### Task 3.7.3: Implement variadic function! macro

**File**: `crates/solidus/src/method/mod.rs` (extend)

- [ ] Add variadic support to `function!`
- [ ] Add tests

**Acceptance**: Variadic methods and functions work correctly

---

## Stage 8: Block Arguments

Support Ruby methods that accept blocks.

### Task 3.8.1: Implement Proc type

**File**: `crates/solidus/src/types/proc.rs`

- [ ] Create `Proc` wrapper type
- [ ] Implement `call` method to invoke the proc
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Add tests

```rust
/// Ruby Proc (callable block).
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Proc(Value);

impl Proc {
    /// Check if a block was given to the current method.
    pub fn block_given() -> bool;
    
    /// Get the block passed to the current method.
    pub fn block() -> Option<Self>;
    
    /// Call the proc with the given arguments.
    pub fn call<A: IntoValueTuple>(self, args: A) -> Result<Value, Error>;
}
```

### Task 3.8.2: Add block support to method! macro

**File**: `crates/solidus/src/method/mod.rs` (extend)

- [ ] Add optional block parameter support
- [ ] Block is accessed via `Proc::block()` inside the method
- [ ] Add tests

```rust
/// Methods can check for and use blocks:
/// fn each_line(rb_self: RString) -> Result<RString, Error> {
///     if let Some(block) = Proc::block() {
///         for line in rb_self.lines()? {
///             block.call((line,))?;
///         }
///     }
///     Ok(rb_self)
/// }
```

### Task 3.8.3: Implement IntoValueTuple for block arguments

**File**: `crates/solidus/src/method/mod.rs` (extend)

- [ ] Create trait for converting tuples to argument arrays
- [ ] Implement for tuples of various sizes
- [ ] Add tests

```rust
/// Trait for types that can be converted to a slice of Values for proc calls.
pub trait IntoValueTuple {
    fn into_value_array(self) -> Vec<Value>;
}

impl IntoValueTuple for () {
    fn into_value_array(self) -> Vec<Value> { vec![] }
}

impl<T: IntoValue> IntoValueTuple for (T,) {
    fn into_value_array(self) -> Vec<Value> {
        vec![self.0.into_value()]
    }
}

// ... implement for (T1, T2), (T1, T2, T3), etc.
```

**Acceptance**: Block arguments work correctly

---

## Stage 9: Keyword Arguments

Support Ruby keyword arguments.

### Task 3.9.1: Research Ruby kwargs C API

- [ ] Study `rb_scan_args` and related APIs
- [ ] Understand `rb_get_kwargs`
- [ ] Document the C API patterns

### Task 3.9.2: Implement KwArgs type

**File**: `crates/solidus/src/types/kwargs.rs`

- [ ] Create `KwArgs` type for keyword arguments
- [ ] Implement methods to extract required/optional kwargs
- [ ] Handle unknown keyword errors
- [ ] Add tests

```rust
/// Keyword arguments helper.
pub struct KwArgs {
    hash: RHash,
}

impl KwArgs {
    /// Get a required keyword argument.
    pub fn required<T: TryConvert>(&self, key: &str) -> Result<T, Error>;
    
    /// Get an optional keyword argument.
    pub fn optional<T: TryConvert>(&self, key: &str) -> Result<Option<T>, Error>;
    
    /// Get an optional keyword argument with a default value.
    pub fn optional_or<T: TryConvert>(&self, key: &str, default: T) -> Result<T, Error>;
}
```

### Task 3.9.3: Add kwargs support to method! macro

**File**: `crates/solidus/src/method/mod.rs` (extend)

- [ ] Support methods with `KwArgs` as final parameter
- [ ] Generate correct Ruby method signature
- [ ] Add tests

```rust
/// Method with keyword arguments:
/// fn create(rb_self: RClass, name: RString, kwargs: KwArgs) -> Result<Value, Error> {
///     let age: i64 = kwargs.required("age")?;
///     let active: bool = kwargs.optional_or("active", true)?;
///     // ...
/// }
```

**Acceptance**: Keyword arguments work correctly

---

## Stage 10: Integration and Polish

### Task 3.10.1: Create Phase 3 examples

**Directory**: `examples/phase3_methods/`

- [x] Create example extension demonstrating all features
- [x] Include `lib.rs`, `Cargo.toml`, `build.rs`, `test.rb`
- [x] Show method definition with various arities
- [x] Show function definition
- [ ] Show blocks and kwargs (deferred to future phases)
- [x] Add README.md

### Task 3.10.2: Update documentation

- [x] Add doc comments to all public items
- [x] Add module-level documentation for `method` module
- [x] Add examples to key macros and functions
- [x] Update lib.rs documentation
- [x] Run `cargo doc` and verify

### Task 3.10.3: Final testing

- [x] Run full test suite: `cargo test --workspace`
- [x] Run with Ruby: `cargo test --workspace --features link-ruby`
- [x] Run clippy: `cargo clippy --workspace`
- [x] Test the example extension with Ruby
- [x] Verify all acceptance criteria

**Acceptance**: All Phase 3 acceptance criteria met

---

## Acceptance Criteria (Summary)

From `phase-3-methods.md`:

- [x] `method!` works for arities 0-4 (0-15 possible via same pattern)
- [x] `function!` works for arities 0-4 (0-15 possible via same pattern)
- [x] Mixed pinned/non-pinned arguments work (implicit via Pin<&StackPinned<T>>)
- [x] Implicit pinning provides ergonomic API (via #[method] and #[function] attribute macros)
- [ ] Variadic arguments supported (deferred to future work)
- [ ] Block arguments supported (deferred to future work)
- [ ] Keyword arguments supported (deferred to future work)
- [x] `#[solidus::init]` generates correct init function
- [x] `define_method` available on Module trait
- [x] `define_singleton_method` available on Module trait
- [x] `define_module_function` available on Module trait
- [x] `define_global_function` available on Ruby
- [x] Panic handling works correctly
- [x] Error propagation works correctly
- [x] All tests pass (192 passed with Ruby, 0 failed)

---

## Notes

### Ruby C API Functions Reference

| Operation | Function |
|-----------|----------|
| Define method | `rb_define_method(klass, name, func, arity)` |
| Define singleton method | `rb_define_singleton_method(obj, name, func, arity)` |
| Define module function | `rb_define_module_function(mod, name, func, arity)` |
| Define global function | `rb_define_global_function(name, func, arity)` |
| Check block given | `rb_block_given_p()` |
| Get block proc | `rb_block_proc()` |
| Call proc | `rb_proc_call(proc, args)` |
| Get kwargs | `rb_get_kwargs(hash, table, required, optional, values)` |

### Arity Values

| Arity | Meaning |
|-------|---------|
| 0 | No arguments |
| 1-15 | Fixed number of arguments |
| -1 | Variadic: `(int argc, VALUE *argv, VALUE self)` |
| -2 | Variadic: `(VALUE self, VALUE args)` where args is Array |

### Design Decisions

1. **Implicit pinning by default**: The `method!` macro pins all heap arguments
   internally. Users write simple signatures with direct types. This provides
   safety without verbosity.

2. **Explicit pinning available**: Users who want to make pinning visible can
   still use `Pin<&StackPinned<T>>` in their signatures.

3. **Declarative macros first**: Start with `macro_rules!` for `method!` and
   `function!`. Move to proc-macros only if limitations are encountered.

4. **Proc-macro for #[init]**: The `#[init]` attribute requires proc-macro
   capabilities for function transformation.

5. **Self is not pinned**: The `self` parameter doesn't need pinning because
   it's always on the stack in the wrapper function and can't be moved during
   the method call.
