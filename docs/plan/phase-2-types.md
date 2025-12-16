# Phase 2: Ruby Types

## Objective

Implement wrapper types for Ruby's built-in classes.

## Dependencies

- Phase 1 complete

## Type Categories

### Immediate Values (No Pinning Required)

These types don't need GC protection and can be passed directly:

| Rust Type | Ruby Type | Notes |
|-----------|-----------|-------|
| `Fixnum` | Small Integer | Fits in VALUE |
| `Symbol` | Symbol | Interned, never collected |
| `Qtrue` | true | Singleton |
| `Qfalse` | false | Singleton |
| `Qnil` | nil | Singleton |

### Heap Values (Pinning Required)

These types are heap-allocated in Ruby and need pinning:

| Rust Type | Ruby Type | Notes |
|-----------|-----------|-------|
| `RString` | String | |
| `RArray` | Array | |
| `RHash` | Hash | |
| `RBignum` | Large Integer | |
| `RFloat` | Float | Large floats |
| `RClass` | Class | |
| `RModule` | Module | |
| `RRegexp` | Regexp | |
| `RStruct` | Struct | |
| `Proc` | Proc | |

### Union Types

| Rust Type | Contains | Notes |
|-----------|----------|-------|
| `Integer` | `Fixnum` or `RBignum` | |
| `Float` | `Flonum` or `RFloat` | |

## Tasks

### 2.1 Immediate Types

- [ ] `Fixnum` - small integers
- [ ] `Symbol` - interned symbols  
- [ ] `Flonum` - small floats (on platforms that support it)
- [ ] Singleton accessors for nil, true, false

### 2.2 String Type

```rust
// crates/solidus/src/types/string.rs

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RString(Value);

impl RString {
    pub fn new(s: &str) -> Result<Self, Error>;
    pub fn from_slice(bytes: &[u8]) -> Result<Self, Error>;
    
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    
    /// Get string contents.
    /// 
    /// # Safety
    /// The returned slice is only valid while no Ruby code runs.
    pub unsafe fn as_slice(&self) -> &[u8];
    
    pub fn to_string(&self) -> Result<String, Error>;
}
```

- [ ] Implement `RString`
- [ ] Implement `ReprValue`, `TryConvert`, `IntoValue`
- [ ] Add encoding support
- [ ] Add tests

### 2.3 Array Type

```rust
// crates/solidus/src/types/array.rs

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RArray(Value);

impl RArray {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
    pub fn from_slice<T: IntoValue>(slice: &[T]) -> Self;
    
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    
    pub fn push<T: IntoValue>(&self, value: T);
    pub fn pop(&self) -> Option<Value>;
    
    pub fn entry<T: TryConvert>(&self, index: isize) -> Result<T, Error>;
    pub fn store<T: IntoValue>(&self, index: isize, value: T);
    
    pub fn each<F>(&self, f: F) -> Result<(), Error>
    where
        F: FnMut(Value) -> Result<(), Error>;
}
```

- [ ] Implement `RArray`
- [ ] Implement iteration
- [ ] Add tests

### 2.4 Hash Type

```rust
// crates/solidus/src/types/hash.rs

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RHash(Value);

impl RHash {
    pub fn new() -> Self;
    
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    
    pub fn get<K: IntoValue, V: TryConvert>(&self, key: K) -> Result<Option<V>, Error>;
    pub fn insert<K: IntoValue, V: IntoValue>(&self, key: K, value: V);
    pub fn delete<K: IntoValue>(&self, key: K) -> Option<Value>;
    
    pub fn each<F>(&self, f: F) -> Result<(), Error>
    where
        F: FnMut(Value, Value) -> Result<(), Error>;
}
```

- [ ] Implement `RHash`
- [ ] Implement iteration
- [ ] Add tests

### 2.5 Integer Type

```rust
// crates/solidus/src/types/integer.rs

/// Small integer that fits in a VALUE.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Fixnum(Value);

/// Large integer (heap allocated).
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RBignum(Value);

/// Any Ruby integer.
#[derive(Clone, Copy)]
pub enum Integer {
    Fixnum(Fixnum),
    Bignum(RBignum),
}

impl Integer {
    pub fn from_i64(n: i64) -> Self;
    pub fn from_u64(n: u64) -> Self;
    
    pub fn to_i64(&self) -> Result<i64, Error>;
    pub fn to_u64(&self) -> Result<u64, Error>;
}
```

- [ ] Implement `Fixnum`
- [ ] Implement `RBignum`
- [ ] Implement `Integer` union type
- [ ] Add conversions for all integer sizes
- [ ] Add tests

### 2.6 Float Type

- [ ] Implement `Flonum` (immediate float, if platform supports)
- [ ] Implement `RFloat` (heap float)
- [ ] Implement `Float` union type
- [ ] Add tests

### 2.7 Class and Module Types

```rust
// crates/solidus/src/types/class.rs

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RClass(Value);

impl RClass {
    pub fn new(superclass: RClass) -> Result<Self, Error>;
    pub fn name(&self) -> Option<String>;
    pub fn superclass(&self) -> Option<RClass>;
    
    pub fn define_method<M>(&self, name: &str, method: M) -> Result<(), Error>
    where
        M: Method;
    
    pub fn define_singleton_method<M>(&self, name: &str, method: M) -> Result<(), Error>
    where
        M: Method;
        
    pub fn new_instance<A: ArgList>(&self, args: A) -> Result<Value, Error>;
}
```

- [ ] Implement `RClass`
- [ ] Implement `RModule`
- [ ] Implement `Module` trait for shared functionality
- [ ] Add tests

### 2.8 Other Types

- [ ] `Symbol` - already immediate, but add methods
- [ ] `RRegexp` - regular expressions
- [ ] `RStruct` - Ruby structs
- [ ] `Proc` - blocks/procs/lambdas
- [ ] `Range` - ranges

## Conversion Traits

### 2.9 TryConvert Trait

```rust
// crates/solidus/src/convert/try_convert.rs

/// Convert a Ruby Value to a Rust type.
pub trait TryConvert: Sized {
    fn try_convert(val: Value) -> Result<Self, Error>;
}

// Implementations for:
// - All Ruby wrapper types
// - Rust primitives (i8-i64, u8-u64, f32, f64, bool)
// - String, &str, PathBuf
// - Option<T>
// - Vec<T>, HashMap<K,V>
```

- [ ] Define trait
- [ ] Implement for all Ruby types
- [ ] Implement for Rust primitives
- [ ] Implement for collections
- [ ] Add tests

### 2.10 IntoValue Trait

```rust
// crates/solidus/src/convert/into_value.rs

/// Convert a Rust type to a Ruby Value.
pub trait IntoValue {
    fn into_value(self, ruby: &Ruby) -> Value;
}
```

- [ ] Define trait
- [ ] Implement for all Ruby types
- [ ] Implement for Rust primitives
- [ ] Implement for collections
- [ ] Add tests

## Acceptance Criteria

- [ ] All major Ruby types have Rust wrappers
- [ ] `TryConvert` and `IntoValue` work for common types
- [ ] Immediate values can be used without pinning
- [ ] Heap values require pinning in method signatures
- [ ] Comprehensive test coverage
