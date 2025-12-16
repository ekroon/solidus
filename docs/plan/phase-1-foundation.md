# Phase 1: Foundation

## Objective

Implement core types that form the foundation of Solidus's safety guarantees.

## Dependencies

- Phase 0 complete

## Tasks

### 1.1 Value Type

The base wrapper around Ruby's `VALUE`.

```rust
// crates/solidus/src/value/value.rs

/// A Ruby VALUE wrapper.
/// 
/// This is a thin wrapper around the raw `VALUE` type from rb-sys.
/// It should not be stored on the heap - use `BoxValue<T>` for that.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Value(rb_sys::VALUE);

impl Value {
    /// Create a Value from a raw Ruby VALUE.
    /// 
    /// # Safety
    /// The VALUE must be valid.
    pub unsafe fn from_raw(raw: rb_sys::VALUE) -> Self;
    
    /// Get the raw VALUE.
    pub fn as_raw(self) -> rb_sys::VALUE;
    
    /// Check if this value is nil.
    pub fn is_nil(self) -> bool;
    
    /// Check if this value is truthy (not nil or false).
    pub fn is_truthy(self) -> bool;
    
    /// Get the Ruby type of this value.
    pub fn rb_type(self) -> ValueType;
}
```

- [ ] Implement `Value` struct
- [ ] Implement basic methods (is_nil, is_truthy, rb_type)
- [ ] Implement `Debug`, `PartialEq`, `Eq` traits
- [ ] Add unit tests

### 1.2 StackPinned Type

The `!Unpin` wrapper that enables stack pinning.

```rust
// crates/solidus/src/value/pinned.rs

/// A wrapper that prevents a value from being unpinned.
/// 
/// This type is `!Unpin` by design. It's used with `Pin<&StackPinned<T>>`
/// to ensure Ruby values remain at a fixed stack location during method calls.
#[repr(transparent)]
pub struct StackPinned<T> {
    value: T,
    _pin: PhantomPinned,
}

impl<T> StackPinned<T> {
    /// Create a new StackPinned wrapper.
    pub fn new(value: T) -> Self;
    
    /// Get a reference to the wrapped value.
    pub fn get(self: Pin<&Self>) -> &T;
    
    /// Get a mutable reference to the wrapped value.
    pub fn get_mut(self: Pin<&mut Self>) -> &mut T;
}

impl<T: ReprValue> StackPinned<T> {
    /// Convert to a heap-allocated BoxValue.
    /// 
    /// # Safety
    /// The pinned value must not be used after calling this method.
    pub unsafe fn into_box_value(self: Pin<&mut Self>) -> BoxValue<T>;
}
```

- [ ] Implement `StackPinned<T>` struct
- [ ] Implement `get`, `get_mut` methods
- [ ] Implement `into_box_value` method
- [ ] Create `pin_on_stack!` macro
- [ ] Add tests

### 1.3 BoxValue Type

Heap-allocated, GC-registered wrapper.

```rust
// crates/solidus/src/value/boxed.rs

/// A heap-allocated Ruby value that is protected from garbage collection.
/// 
/// Use this when you need to store Ruby values in Rust collections or
/// keep them alive across async boundaries.
pub struct BoxValue<T> {
    ptr: NonNull<T>,
}

impl<T: ReprValue> BoxValue<T> {
    /// Create a new BoxValue, registering with Ruby's GC.
    pub fn new(value: T) -> Self;
}

impl<T> Deref for BoxValue<T> {
    type Target = T;
    fn deref(&self) -> &T;
}

impl<T> Drop for BoxValue<T> {
    fn drop(&mut self);  // Unregisters from GC
}
```

- [ ] Implement `BoxValue<T>` struct
- [ ] Implement GC registration (`rb_gc_register_address`)
- [ ] Implement `Drop` with GC unregistration
- [ ] Implement `Deref`, `DerefMut`
- [ ] Add tests

### 1.4 ReprValue Trait

Trait for types that represent Ruby values.

```rust
// crates/solidus/src/value/traits.rs

/// Trait for types that wrap a Ruby VALUE.
pub trait ReprValue: Copy {
    /// Get this value as a base Value.
    fn as_value(self) -> Value;
    
    /// Create from a Value without type checking.
    /// 
    /// # Safety
    /// The value must actually be of this type.
    unsafe fn from_value_unchecked(val: Value) -> Self;
}
```

- [ ] Define `ReprValue` trait
- [ ] Implement for `Value`
- [ ] Add helper methods on the trait

### 1.5 Ruby Handle

The entry point for Ruby API access.

```rust
// crates/solidus/src/ruby.rs

/// Handle to the Ruby VM.
/// 
/// This type cannot be created directly - it's provided by the `#[solidus::init]`
/// macro or obtained via `Ruby::get()` when Ruby is known to be initialized.
pub struct Ruby {
    _private: (),
}

impl Ruby {
    /// Get a reference to Ruby.
    /// 
    /// # Safety
    /// Ruby must be initialized.
    pub unsafe fn get() -> &'static Self;
    
    // Constants
    pub fn qnil(&self) -> Value;
    pub fn qtrue(&self) -> Value;
    pub fn qfalse(&self) -> Value;
    
    // Class access
    pub fn class_object(&self) -> RClass;
    pub fn class_string(&self) -> RClass;
    // ... etc
}
```

- [ ] Implement `Ruby` struct
- [ ] Implement constant accessors (nil, true, false)
- [ ] Implement class accessors
- [ ] Add `define_class`, `define_module`, `define_global_function`

### 1.6 Error Type

Error handling for Ruby exceptions.

```rust
// crates/solidus/src/error.rs

/// A Ruby exception.
pub struct Error {
    value: Value,
}

impl Error {
    /// Create a new error with the given exception class and message.
    pub fn new<T: Into<String>>(class: ExceptionClass, message: T) -> Self;
    
    /// Create from a panic.
    pub(crate) fn from_panic(panic: Box<dyn Any + Send>) -> Self;
    
    /// Raise this error (diverges).
    pub fn raise(self) -> !;
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {}
```

- [ ] Implement `Error` struct
- [ ] Implement `new`, `raise` methods
- [ ] Implement `from_panic` for catch_unwind integration
- [ ] Implement std traits (`Error`, `Display`, `Debug`)

### 1.7 GC Module

Garbage collection utilities.

```rust
// crates/solidus/src/gc.rs

/// Register a VALUE location with the GC.
/// 
/// # Safety
/// The pointer must remain valid until `unregister_address` is called.
pub unsafe fn register_address(addr: *mut VALUE);

/// Unregister a VALUE location from the GC.
/// 
/// # Safety
/// The address must have been previously registered.
pub unsafe fn unregister_address(addr: *mut VALUE);

/// Mark a value during GC marking phase.
pub fn mark(value: Value);

/// Permanently prevent a value from being garbage collected.
pub fn register_mark_object(value: Value);
```

- [ ] Implement GC functions
- [ ] Document safety requirements
- [ ] Add tests

## Acceptance Criteria

- [ ] All core types compile and have basic tests
- [ ] `StackPinned<T>` is verifiably `!Unpin`
- [ ] `BoxValue<T>` correctly registers/unregisters with GC
- [ ] `pin_on_stack!` macro works correctly
- [ ] Error handling integrates with `Result<T, Error>`
