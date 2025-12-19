# Phase 8: Context-Based Value Creation

## Overview

Replace `WitnessedReturn` with a `Context` type that provides safe, stack-allocated storage for Ruby values created within method/function calls.

## Goals

1. `Context` is passed as first argument to wrapped methods/functions
2. Stack-allocates space for Ruby VALUE pointers (8 slots default)
3. `new_xxx()` methods return `Result<Pin<&StackPinned<T>>, AllocationError>`
4. `new_xxx_boxed()` methods return `BoxValue<T>` (always succeeds)
5. Uses interior mutability (`&self`) for creating multiple values
6. Remove unsafe `RString::new()` etc. from public API

## Design Decisions

1. **Return type**: `Result<..., AllocationError>` for ergonomic `?` usage
2. **Explicit return**: Users return `Pin<&StackPinned<T>>` which wrapper unwraps
3. **Storage**: Raw `rb_sys::VALUE` array (all same size, type-erased)
4. **Arguments**: Still auto-pinned by wrapper, separate from Context
5. **No automatic heap fallback**: Return error on exhaustion
6. **No direct NewValue creation**: Users must use Context

## Implementation Stages

### Stage 1: Core Infrastructure
- [ ] Add `AllocationError` to `error.rs`
- [ ] Create `context.rs` with `Context` struct
- [ ] Implement `new_string`, `new_array`, `new_hash` methods
- [ ] Ensure `#[repr(transparent)]` on `StackPinned`

### Stage 2: Return Value Handling
- [ ] Create new `IntoReturnValue` trait
- [ ] Implement for `Pin<&StackPinned<T>>`, `BoxValue<T>`, primitives
- [ ] Implement for `Result<T, Error>`

### Stage 3: Macro Updates
- [ ] Update `method!` macro to create Context and pass to user function
- [ ] Update `function!` macro similarly

### Stage 4: Type Migration
- [ ] Make `RString::new()` internal/remove
- [ ] Make `RArray::new()` internal/remove
- [ ] Make `RHash::new()` internal/remove
- [ ] Keep `*_boxed()` methods public

### Stage 5: Example Updates
- [ ] Update `examples/phase3_methods/src/lib.rs`
- [ ] Update other examples as needed

### Stage 6: Cleanup
- [ ] Remove `return_slot.rs` (WitnessedReturn)
- [ ] Update documentation
- [ ] Clean up `NewValue` usage

## API Design

### Context Struct

```rust
pub struct Context<'a, const N: usize = 8> {
    slots: UnsafeCell<[MaybeUninit<rb_sys::VALUE>; N]>,
    used: Cell<usize>,
    _marker: PhantomData<&'a mut ()>,
}

impl<'a, const N: usize> Context<'a, N> {
    pub fn new_string(&'a self, s: &str) -> Result<Pin<&'a StackPinned<RString>>, AllocationError>;
    pub fn new_string_boxed(&self, s: &str) -> BoxValue<RString>;
    pub fn new_array(&'a self) -> Result<Pin<&'a StackPinned<RArray>>, AllocationError>;
    pub fn new_array_boxed(&self) -> BoxValue<RArray>;
    pub fn new_hash(&'a self) -> Result<Pin<&'a StackPinned<RHash>>, AllocationError>;
    pub fn new_hash_boxed(&self) -> BoxValue<RHash>;
    pub fn pin_value<T: ReprValue>(&'a self, value: T) -> Result<Pin<&'a StackPinned<T>>, AllocationError>;
}
```

### IntoReturnValue Trait

```rust
pub trait IntoReturnValue {
    fn into_return_value(self) -> Result<rb_sys::VALUE, Error>;
}

// Implemented for:
// - Pin<&StackPinned<T>>
// - BoxValue<T>
// - i64, i32, usize, bool, ()
// - Value
// - Result<T, Error> where T: IntoReturnValue
```

### Method Signatures (Before/After)

**Before:**
```rust
fn greet(rb_self: RString) -> Result<NewValue<RString>, Error> {
    Ok(unsafe { RString::new(&format!("Hello, {}!", rb_self.to_string()?)) })
}
```

**After:**
```rust
fn greet<'ctx>(
    ctx: &'ctx Context,
    rb_self: RString
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    ctx.new_string(&format!("Hello, {}!", rb_self.to_string()?))
        .map_err(Into::into)
}
```

## Files to Create/Modify

### New Files
- `crates/solidus/src/context.rs`

### Modified Files
- `crates/solidus/src/error.rs` - Add `AllocationError`
- `crates/solidus/src/method/mod.rs` - Update macros
- `crates/solidus/src/method/return_value.rs` - New `IntoReturnValue`
- `crates/solidus/src/lib.rs` - Export Context
- `crates/solidus/src/value/pinned.rs` - Ensure repr(transparent)
- `crates/solidus/src/types/string.rs` - Remove public new()
- `crates/solidus/src/types/array.rs` - Remove public new()
- `crates/solidus/src/types/hash.rs` - Remove public new()
- `examples/phase3_methods/src/lib.rs` - Update to use Context

### Files to Remove
- `crates/solidus/src/method/return_slot.rs`

## Future Work

1. Const generics for custom capacity: `method!(greet, 0, capacity = 16)`
2. Automatic heap fallback option
3. Attribute macro updates in `solidus-macros`
