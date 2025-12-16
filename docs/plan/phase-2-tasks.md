# Phase 2: Ruby Types - Implementation Tasks

This file contains the detailed task breakdown for Phase 2. Each task should be
completed in order, as later tasks depend on earlier ones.

## Prerequisites

- Phase 1 complete (Value, StackPinned, BoxValue, ReprValue, Error)
- Understanding of Ruby's VALUE representation and type tags

## Task Status Legend

- [ ] Not started
- [x] Complete
- [~] In progress

---

## Stage 1: Conversion Traits

These traits are the foundation for all type conversions and must be implemented first.

### Task 2.1.1: Create convert module structure

**File**: `crates/solidus/src/convert/mod.rs`

- [ ] Create the `convert` directory and `mod.rs`
- [ ] Add module to `lib.rs`
- [ ] Re-export traits in prelude

```rust
// crates/solidus/src/convert/mod.rs
mod into_value;
mod try_convert;

pub use into_value::IntoValue;
pub use try_convert::TryConvert;
```

### Task 2.1.2: Implement TryConvert trait

**File**: `crates/solidus/src/convert/try_convert.rs`

- [ ] Define the `TryConvert` trait
- [ ] Implement for `Value` (identity conversion)
- [ ] Add tests

```rust
/// Convert a Ruby Value to a Rust type.
pub trait TryConvert: Sized {
    /// Attempt to convert a Ruby Value to Self.
    fn try_convert(val: Value) -> Result<Self, Error>;
}

// Identity implementation
impl TryConvert for Value {
    fn try_convert(val: Value) -> Result<Self, Error> {
        Ok(val)
    }
}
```

### Task 2.1.3: Implement IntoValue trait

**File**: `crates/solidus/src/convert/into_value.rs`

- [ ] Define the `IntoValue` trait
- [ ] Implement for `Value` (identity conversion)
- [ ] Add tests

```rust
/// Convert a Rust type to a Ruby Value.
pub trait IntoValue {
    /// Convert self into a Ruby Value.
    fn into_value(self) -> Value;
}

// Identity implementation
impl IntoValue for Value {
    fn into_value(self) -> Value {
        self
    }
}
```

**Acceptance**: `cargo test -p solidus convert` passes

---

## Stage 2: Immediate Types

Immediate values don't require GC protection and can be passed directly.

### Task 2.2.1: Create types module structure

**File**: `crates/solidus/src/types/mod.rs`

- [ ] Create the `types` directory and `mod.rs`
- [ ] Add module to `lib.rs`
- [ ] Re-export types in prelude

### Task 2.2.2: Implement Qnil, Qtrue, Qfalse wrappers

**File**: `crates/solidus/src/types/immediate.rs`

- [ ] Create `Qnil` unit struct with singleton accessor
- [ ] Create `Qtrue` unit struct with singleton accessor  
- [ ] Create `Qfalse` unit struct with singleton accessor
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue` for each
- [ ] Add tests

```rust
/// Ruby nil value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Qnil;

impl Qnil {
    /// Get the nil value.
    #[inline]
    pub fn new() -> Self {
        Qnil
    }
}

impl ReprValue for Qnil {
    fn as_value(self) -> Value {
        Value::nil()
    }
    
    unsafe fn from_value_unchecked(_val: Value) -> Self {
        Qnil
    }
}
```

### Task 2.2.3: Implement Fixnum

**File**: `crates/solidus/src/types/integer.rs`

- [ ] Create `Fixnum` wrapper for small integers
- [ ] Implement `from_i64`, `to_i64` methods
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Implement conversions for i8, i16, i32, i64, isize
- [ ] Implement conversions for u8, u16, u32 (u64 may overflow)
- [ ] Add tests

```rust
/// Small integer that fits in a VALUE (immediate value).
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Fixnum(Value);

impl Fixnum {
    /// Create a Fixnum from an i64.
    /// Returns None if the value doesn't fit in a Fixnum.
    pub fn from_i64(n: i64) -> Option<Self>;
    
    /// Get the value as i64.
    pub fn to_i64(self) -> i64;
}
```

**Note**: Use `rb_sys::FIXNUM_P` to check if a VALUE is a fixnum,
`rb_sys::RB_FIX2LONG` to extract, `rb_sys::RB_LONG2FIX` to create.

### Task 2.2.4: Implement Symbol

**File**: `crates/solidus/src/types/symbol.rs`

- [ ] Create `Symbol` wrapper
- [ ] Implement `new` from string (interns the symbol)
- [ ] Implement `name` to get symbol string
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Add tests

```rust
/// Ruby Symbol (interned string, immediate value).
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Symbol(Value);

impl Symbol {
    /// Create or get an existing symbol from a string.
    pub fn new(name: &str) -> Self;
    
    /// Get the symbol's name.
    pub fn name(self) -> Result<String, Error>;
}
```

**Note**: Use `rb_sys::rb_intern` to create, `rb_sys::rb_sym2str` + string APIs to get name.

### Task 2.2.5: Implement Flonum (conditional)

**File**: `crates/solidus/src/types/float.rs`

- [ ] Create `Flonum` wrapper for immediate floats (64-bit platforms only)
- [ ] Implement `from_f64`, `to_f64` methods
- [ ] Use conditional compilation for platforms without flonum
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Add tests

```rust
/// Immediate float value (only on 64-bit platforms).
#[cfg(target_pointer_width = "64")]
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Flonum(Value);
```

**Note**: Use `rb_sys::FLONUM_P` to check, `rb_sys::RB_FLOAT_VALUE` to extract.

**Acceptance**: `cargo test -p solidus types::immediate` passes

---

## Stage 3: Numeric Types (Heap)

### Task 2.3.1: Implement RBignum

**File**: `crates/solidus/src/types/integer.rs` (extend)

- [ ] Create `RBignum` wrapper for large integers
- [ ] Implement conversion to/from i64, u64 (with range checking)
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Add tests

```rust
/// Large integer (heap allocated).
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RBignum(Value);
```

**Note**: Use `rb_sys::RB_TYPE_P` with `RUBY_T_BIGNUM`, `rb_sys::rb_big2ll` / `rb_sys::rb_ll2big`.

### Task 2.3.2: Implement Integer union type

**File**: `crates/solidus/src/types/integer.rs` (extend)

- [ ] Create `Integer` enum containing `Fixnum` or `RBignum`
- [ ] Implement unified conversion methods
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Add tests

```rust
/// Any Ruby integer (Fixnum or Bignum).
#[derive(Clone, Copy)]
pub enum Integer {
    Fixnum(Fixnum),
    Bignum(RBignum),
}

impl Integer {
    pub fn from_i64(n: i64) -> Self;
    pub fn from_u64(n: u64) -> Self;
    pub fn to_i64(self) -> Result<i64, Error>;
    pub fn to_u64(self) -> Result<u64, Error>;
}
```

### Task 2.3.3: Implement RFloat

**File**: `crates/solidus/src/types/float.rs` (extend)

- [ ] Create `RFloat` wrapper for heap floats
- [ ] Implement conversion to/from f64
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Add tests

```rust
/// Heap-allocated float.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RFloat(Value);
```

### Task 2.3.4: Implement Float union type

**File**: `crates/solidus/src/types/float.rs` (extend)

- [ ] Create `Float` enum containing `Flonum` or `RFloat`
- [ ] Implement unified conversion methods
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Implement `TryConvert`/`IntoValue` for f32, f64
- [ ] Add tests

```rust
/// Any Ruby float.
#[derive(Clone, Copy)]
pub enum Float {
    #[cfg(target_pointer_width = "64")]
    Flonum(Flonum),
    RFloat(RFloat),
}
```

**Acceptance**: `cargo test -p solidus types::integer types::float` passes

---

## Stage 4: String Type

### Task 2.4.1: Implement RString basics

**File**: `crates/solidus/src/types/string.rs`

- [ ] Create `RString` wrapper
- [ ] Implement `new(s: &str)` constructor
- [ ] Implement `from_slice(bytes: &[u8])` constructor
- [ ] Implement `len()`, `is_empty()`
- [ ] Implement `ReprValue`
- [ ] Add tests

```rust
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RString(Value);

impl RString {
    pub fn new(s: &str) -> Self;
    pub fn from_slice(bytes: &[u8]) -> Self;
    pub fn len(self) -> usize;
    pub fn is_empty(self) -> bool;
}
```

**Note**: Use `rb_sys::rb_str_new`, `rb_sys::RSTRING_LEN`.

### Task 2.4.2: Implement RString content access

**File**: `crates/solidus/src/types/string.rs` (extend)

- [ ] Implement `as_slice(&self) -> &[u8]` (unsafe, lifetime concerns)
- [ ] Implement `to_string(&self) -> Result<String, Error>`
- [ ] Implement `to_bytes(&self) -> Vec<u8>`
- [ ] Document safety requirements for `as_slice`
- [ ] Add tests

```rust
impl RString {
    /// Get string contents as a byte slice.
    /// 
    /// # Safety
    /// The returned slice is only valid while no Ruby code runs that could
    /// modify or move the string.
    pub unsafe fn as_slice(self) -> &[u8];
    
    /// Copy string contents to a Rust String.
    pub fn to_string(self) -> Result<String, Error>;
}
```

### Task 2.4.3: Implement RString conversions

**File**: `crates/solidus/src/types/string.rs` (extend)

- [ ] Implement `TryConvert` for `RString`
- [ ] Implement `IntoValue` for `RString`
- [ ] Implement `TryConvert` for `String`
- [ ] Implement `IntoValue` for `String`, `&str`
- [ ] Add tests

### Task 2.4.4: Implement RString encoding support

**File**: `crates/solidus/src/types/string.rs` (extend)

- [ ] Add `encoding(self) -> Encoding` method
- [ ] Add `encode(self, encoding: Encoding) -> Result<RString, Error>`
- [ ] Create basic `Encoding` type (can be expanded later)
- [ ] Add tests

**Acceptance**: `cargo test -p solidus types::string` passes

---

## Stage 5: Array Type

### Task 2.5.1: Implement RArray basics

**File**: `crates/solidus/src/types/array.rs`

- [ ] Create `RArray` wrapper
- [ ] Implement `new()` constructor
- [ ] Implement `with_capacity(n: usize)` constructor
- [ ] Implement `len()`, `is_empty()`
- [ ] Implement `ReprValue`
- [ ] Add tests

```rust
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RArray(Value);

impl RArray {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
    pub fn len(self) -> usize;
    pub fn is_empty(self) -> bool;
}
```

**Note**: Use `rb_sys::rb_ary_new`, `rb_sys::rb_ary_new_capa`, `rb_sys::RARRAY_LEN`.

### Task 2.5.2: Implement RArray element access

**File**: `crates/solidus/src/types/array.rs` (extend)

- [ ] Implement `push<T: IntoValue>(self, value: T)`
- [ ] Implement `pop(self) -> Option<Value>`
- [ ] Implement `entry(self, index: isize) -> Value`
- [ ] Implement `store<T: IntoValue>(self, index: isize, value: T)`
- [ ] Add tests

```rust
impl RArray {
    pub fn push<T: IntoValue>(self, value: T);
    pub fn pop(self) -> Option<Value>;
    pub fn entry(self, index: isize) -> Value;
    pub fn store<T: IntoValue>(self, index: isize, value: T);
}
```

**Note**: Use `rb_sys::rb_ary_push`, `rb_sys::rb_ary_pop`, `rb_sys::rb_ary_entry`, `rb_sys::rb_ary_store`.

### Task 2.5.3: Implement RArray iteration

**File**: `crates/solidus/src/types/array.rs` (extend)

- [ ] Implement `each<F>(self, f: F)` with closure
- [ ] Consider if Rust `Iterator` is safe (probably not due to GC)
- [ ] Add tests

```rust
impl RArray {
    pub fn each<F>(self, f: F) -> Result<(), Error>
    where
        F: FnMut(Value) -> Result<(), Error>;
}
```

### Task 2.5.4: Implement RArray conversions

**File**: `crates/solidus/src/types/array.rs` (extend)

- [ ] Implement `TryConvert` for `RArray`
- [ ] Implement `IntoValue` for `RArray`
- [ ] Implement `from_slice<T: IntoValue>(slice: &[T]) -> Self`
- [ ] Implement `TryConvert` for `Vec<T>` where T: TryConvert
- [ ] Implement `IntoValue` for `Vec<T>` where T: IntoValue
- [ ] Add tests

**Acceptance**: `cargo test -p solidus types::array` passes

---

## Stage 6: Hash Type

### Task 2.6.1: Implement RHash basics

**File**: `crates/solidus/src/types/hash.rs`

- [ ] Create `RHash` wrapper
- [ ] Implement `new()` constructor
- [ ] Implement `len()`, `is_empty()`
- [ ] Implement `ReprValue`
- [ ] Add tests

```rust
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RHash(Value);

impl RHash {
    pub fn new() -> Self;
    pub fn len(self) -> usize;
    pub fn is_empty(self) -> bool;
}
```

### Task 2.6.2: Implement RHash operations

**File**: `crates/solidus/src/types/hash.rs` (extend)

- [ ] Implement `get<K: IntoValue>(self, key: K) -> Option<Value>`
- [ ] Implement `insert<K: IntoValue, V: IntoValue>(self, key: K, value: V)`
- [ ] Implement `delete<K: IntoValue>(self, key: K) -> Option<Value>`
- [ ] Add tests

### Task 2.6.3: Implement RHash iteration

**File**: `crates/solidus/src/types/hash.rs` (extend)

- [ ] Implement `each<F>(self, f: F)` with closure
- [ ] Add tests

```rust
impl RHash {
    pub fn each<F>(self, f: F) -> Result<(), Error>
    where
        F: FnMut(Value, Value) -> Result<(), Error>;
}
```

### Task 2.6.4: Implement RHash conversions

**File**: `crates/solidus/src/types/hash.rs` (extend)

- [ ] Implement `TryConvert` for `RHash`
- [ ] Implement `IntoValue` for `RHash`
- [ ] Implement `TryConvert` for `HashMap<K, V>`
- [ ] Implement `IntoValue` for `HashMap<K, V>`
- [ ] Add tests

**Acceptance**: `cargo test -p solidus types::hash` passes

---

## Stage 7: Class and Module Types

### Task 2.7.1: Implement RClass

**File**: `crates/solidus/src/types/class.rs`

- [ ] Create `RClass` wrapper
- [ ] Implement `from_value` with type checking
- [ ] Implement `name(self) -> Option<String>`
- [ ] Implement `superclass(self) -> Option<RClass>`
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Add tests

```rust
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RClass(Value);

impl RClass {
    pub fn name(self) -> Option<String>;
    pub fn superclass(self) -> Option<RClass>;
}
```

### Task 2.7.2: Implement RModule

**File**: `crates/solidus/src/types/module.rs`

- [ ] Create `RModule` wrapper
- [ ] Implement `from_value` with type checking
- [ ] Implement `name(self) -> Option<String>`
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Add tests

### Task 2.7.3: Implement shared Module trait

**File**: `crates/solidus/src/types/module.rs` (extend)

- [ ] Create `Module` trait for shared behavior
- [ ] Implement for both `RClass` and `RModule`
- [ ] Include `define_const`, `const_get` methods
- [ ] Note: `define_method` deferred to Phase 3

```rust
/// Trait for types that can define methods and constants.
pub trait Module: ReprValue {
    fn define_const<T: IntoValue>(self, name: &str, value: T) -> Result<(), Error>;
    fn const_get(self, name: &str) -> Result<Value, Error>;
}
```

**Acceptance**: `cargo test -p solidus types::class types::module` passes

---

## Stage 8: Additional Types (Optional for Phase 2)

These can be deferred to later phases if needed.

### Task 2.8.1: Implement RRegexp

**File**: `crates/solidus/src/types/regexp.rs`

- [ ] Create `RRegexp` wrapper
- [ ] Implement basic construction and matching
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`

### Task 2.8.2: Implement RStruct

**File**: `crates/solidus/src/types/rstruct.rs`

- [ ] Create `RStruct` wrapper
- [ ] Implement member access
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`

### Task 2.8.3: Implement Proc

**File**: `crates/solidus/src/types/proc.rs`

- [ ] Create `Proc` wrapper
- [ ] Implement `call` method
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`

### Task 2.8.4: Implement Range

**File**: `crates/solidus/src/types/range.rs`

- [ ] Create `Range` wrapper
- [ ] Implement `begin`, `end`, `exclude_end?`
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`

---

## Final Integration

### Task 2.9.1: Update lib.rs exports

- [ ] Re-export all types from `crates/solidus/src/lib.rs`
- [ ] Update prelude with commonly used types
- [ ] Ensure `TryConvert` and `IntoValue` are in prelude

### Task 2.9.2: Update documentation

- [ ] Add doc comments to all public items
- [ ] Add module-level documentation
- [ ] Add examples to key methods
- [ ] Run `cargo doc` and verify

### Task 2.9.3: Final testing

- [ ] Run full test suite: `cargo test --workspace`
- [ ] Run with Ruby: `cargo test --workspace --features embed`
- [ ] Run clippy: `cargo clippy --workspace`
- [ ] Verify all acceptance criteria from phase-2-types.md

**Acceptance**: All Phase 2 acceptance criteria met

---

## Stage 10: Pinned-From-Creation Changes (ADR-007)

This stage implements the design changes required by ADR-007 to prevent VALUE
heap escape. See [decisions.md](decisions.md#adr-007-values-must-be-pinned-from-creation).

### Task 2.10.1: Remove Copy from VALUE wrapper types

**Files**: All type files in `crates/solidus/src/types/`

- [ ] Remove `Copy` from `Value` in `value/mod.rs`
- [ ] Remove `Copy` from `RString` in `types/string.rs`
- [ ] Remove `Copy` from `RArray` in `types/array.rs`
- [ ] Remove `Copy` from `RHash` in `types/hash.rs`
- [ ] Remove `Copy` from `RBignum` in `types/integer.rs`
- [ ] Remove `Copy` from `Integer` enum in `types/integer.rs`
- [ ] Remove `Copy` from `RFloat` in `types/float.rs`
- [ ] Remove `Copy` from `Float` enum in `types/float.rs`
- [ ] Remove `Copy` from `RClass` in `types/class.rs`
- [ ] Remove `Copy` from `RModule` in `types/module.rs`
- [ ] Fix all compilation errors that result from removing `Copy`
- [ ] Update tests

**Note**: Keep `Copy` on immediate types: `Fixnum`, `Symbol`, `Qnil`, `Qtrue`, `Qfalse`, `Flonum`.

### Task 2.10.2: Design and implement pinned creation API

**File**: New file `crates/solidus/src/value/creation.rs` or extend existing files

Choose one of these approaches:

**Option A: Macro-based creation (recommended)**
```rust
/// Create a new RString, pinning it on the stack
/// pin_on_stack!(s = RString::new("hello")?);
/// 
/// The macro expands to:
/// let __tmp = RString::new("hello")?;
/// let mut __pinned = StackPinned::new(__tmp);
/// let s = Pin::new(&mut __pinned);
```

- [ ] Update `pin_on_stack!` macro to support creation expressions
- [ ] Add `pin_on_stack!` examples to all creation methods
- [ ] Document the pattern

**Option B: Creation returns PinGuard**
```rust
/// A guard that holds an unpinned VALUE and must be consumed by pinning
pub struct PinGuard<T: ReprValue>(T);

impl<T: ReprValue> PinGuard<T> {
    /// Pin this guard on the stack
    /// SAFETY: Must be called immediately, value must not escape
    pub unsafe fn pin(self) -> T { self.0 }
}

impl RString {
    pub fn new(s: &str) -> Result<PinGuard<RString>, Error>;
}
```

- [ ] Create `PinGuard<T>` type
- [ ] Update all creation functions to return `PinGuard<T>`
- [ ] Update `pin_on_stack!` to accept `PinGuard<T>`

### Task 2.10.3: Update BoxValue for !Copy types

**File**: `crates/solidus/src/value/boxed.rs`

- [ ] Ensure `BoxValue::new()` works with `!Copy` types
- [ ] Add `BoxValue::from_pinned()` to convert pinned refs to boxed
- [ ] Update documentation with heap storage patterns
- [ ] Add tests

```rust
impl<T: ReprValue> BoxValue<T> {
    /// Create a new BoxValue from a pinned reference
    /// This is the safe way to move a VALUE to the heap
    pub fn from_pinned(pinned: Pin<&StackPinned<T>>) -> Self;
}
```

### Task 2.10.4: Update TryConvert for !Copy types

**File**: `crates/solidus/src/convert/try_convert.rs`

- [ ] Ensure `TryConvert` works correctly with `!Copy` types
- [ ] Consider if `TryConvert` should return pinned types
- [ ] Update any implementations that assume `Copy`
- [ ] Add tests

### Task 2.10.5: Update IntoValue for !Copy types

**File**: `crates/solidus/src/convert/into_value.rs`

- [ ] Ensure `IntoValue` works correctly with `!Copy` types
- [ ] `IntoValue::into_value(self)` consumes self, which is fine for `!Copy`
- [ ] Add implementations for `&T` and `Pin<&StackPinned<T>>` if needed
- [ ] Add tests

### Task 2.10.6: Update all examples

**Directory**: `examples/`

- [ ] Update `phase2_string` example for !Copy RString
- [ ] Update `phase2_array` example for !Copy RArray
- [ ] Update `phase2_hash` example for !Copy RHash
- [ ] Update `phase2_numeric_heap` example for !Copy Integer/Float
- [ ] Update `phase2_class_module` example for !Copy RClass/RModule
- [ ] Update `phase2_conversions` example

### Task 2.10.7: Update documentation

- [ ] Update `phase-2-types.md` code examples (remove Copy from derives)
- [ ] Update doc comments on all affected types
- [ ] Add section on VALUE creation and pinning patterns
- [ ] Update README if needed

**Acceptance**: All VALUE types are `!Copy`, creation APIs enforce pinning,
`BoxValue<T>` is the only way to store VALUEs on the heap.

---

## Notes

### Ruby C API Functions Reference

| Operation | Function |
|-----------|----------|
| Check fixnum | `FIXNUM_P(v)` |
| Fixnum to long | `RB_FIX2LONG(v)` |
| Long to fixnum | `RB_LONG2FIX(n)` |
| Check type | `RB_TYPE_P(v, T_XXX)` |
| Get type | `RB_TYPE(v)` |
| New string | `rb_str_new(ptr, len)` |
| String length | `RSTRING_LEN(v)` |
| String pointer | `RSTRING_PTR(v)` |
| New array | `rb_ary_new()` |
| Array length | `RARRAY_LEN(v)` |
| Array push | `rb_ary_push(ary, val)` |
| New hash | `rb_hash_new()` |
| Hash get | `rb_hash_aref(hash, key)` |
| Hash set | `rb_hash_aset(hash, key, val)` |
| Intern symbol | `rb_intern(name)` |
| Symbol to string | `rb_sym2str(sym)` |

### Design Decisions

1. **Flonum is platform-specific**: Only exists on 64-bit platforms. The `Float` union
   type handles this transparently.

2. **Iteration uses closures, not iterators**: Rust `Iterator` would be unsafe because
   the GC could run between iterations. Closures ensure we control execution flow.

3. **`as_slice` is unsafe**: Returns a reference into Ruby's heap. Any Ruby code that
   runs could invalidate this reference.

4. **Method definition deferred**: `RClass::define_method` needs the `method!` macro
   from Phase 3 to be ergonomic.
