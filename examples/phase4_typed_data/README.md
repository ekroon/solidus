# Phase 4 Typed Data Example

This example demonstrates wrapping Rust types as Ruby objects using the TypedData API in Solidus.

## Examples Included

### Task 4.7.1: Basic Point

Shows how to:
- Use `#[solidus::wrap(class = "Point")]` to make a struct wrappable
- Implement methods on the Rust type
- Wrap instances with the `wrap()` function
- Unwrap instances with the `get()` function
- Pass wrapped objects between Ruby and Rust
- **Use primitive arguments** (`f64`) directly in function signatures

### Task 4.7.2: Counter with RefCell

Shows how to:
- Use `RefCell<T>` for interior mutability
- Safely mutate wrapped data from Ruby method calls
- Follow Rust's borrowing rules even when called from Ruby
- **Use primitive arguments** (`i64`) directly in function signatures

### Task 4.7.3: Container with GC Marking

Shows how to:
- Store Ruby values inside wrapped Rust types using `Vec<BoxValue<Value>>`
- Implement `DataTypeFunctions` to mark contained Ruby values for GC
- Use `#[solidus::wrap(class = "Container", mark)]` to enable GC marking
- Prevent Ruby values from being garbage collected while stored in Rust
- **Mix primitive arguments** (`i64` for index) with Ruby VALUE types (`Pin<&StackPinned<Value>>`)

## Parameter Type Rules

The `#[solidus_macros::method]` and `#[solidus_macros::function]` macros handle different
parameter types appropriately for GC safety:

### Self Parameter (`rb_self`)

The receiver (`rb_self`) does **not** need pinning because Ruby's method dispatch guarantees
the receiver is live for the duration of the method call. Use the type directly:

```rust
#[solidus_macros::method]
fn point_x(rb_self: Value) -> Result<f64, Error> { ... }
```

### Method Arguments

Method arguments may need pinning depending on their type:

1. **Rust primitives** (`i64`, `f64`, `bool`, `String`, etc.) - Use the type directly:
   ```rust
   #[solidus_macros::method]
   fn container_get(rb_self: Value, index: i64) -> Result<Value, Error> { ... }
   ```

2. **Ruby VALUE types** (`Value`, `RString`, `RArray`, etc.) - Use `Pin<&StackPinned<T>>`:
   ```rust
   #[solidus_macros::method]
   fn container_push(rb_self: Value, value: Pin<&StackPinned<Value>>) -> Result<Value, Error> { ... }
   ```

Arguments need pinning because they may be computed values that are the only reference
to a Ruby object. Without pinning, the GC could collect them during the method body.

### Function Parameters

Functions (no `rb_self`) follow the same rules for their parameters:

```rust
// Primitive arguments - no pinning needed
#[solidus_macros::function]
fn point_new(x: f64, y: f64) -> Result<Value, Error> { ... }

// Ruby VALUE arguments - use Pin<&StackPinned<T>>
#[solidus_macros::function]
fn process(value: Pin<&StackPinned<Value>>) -> Result<Value, Error> { ... }
```

The macros automatically handle conversion from Ruby VALUEs to Rust primitives when
the type implements `TryConvert`.

## Building

```bash
cargo build
```

## Running

```bash
ruby test.rb
```

## API Demonstrated

### Core TypedData Functions

- `wrap(ruby: &Ruby, class: &RClass, value: T) -> Result<Value, Error>` - Wrap a Rust value
- `get<T>(value: &Value) -> Result<&T, Error>` - Get immutable reference to wrapped data
- `get_mut<T>(value: &Value) -> Result<&mut T, Error>` - Get mutable reference to wrapped data

### TypedData Trait

The `#[solidus::wrap]` attribute macro implements this trait automatically:

```rust
pub trait TypedData {
    fn class_name() -> &'static str;
    fn data_type() -> &'static DataType;
}
```

### DataTypeFunctions Trait

Implement this to customize GC behavior:

```rust
pub trait DataTypeFunctions {
    fn mark(&self, marker: &Marker) { }
    fn compact(&self, compactor: &Compactor) { }
    fn size(&self) -> usize { std::mem::size_of::<Self>() }
}
```

##Safety

TypedData provides memory safety by:

1. **Automatic memory management**: Wrapped values are freed when the Ruby object is GC'd
2. **Type checking**: `get()` and `get_mut()` verify the type at runtime
3. **Borrow checking**: RefCell provides runtime borrow checking for mutation
4. **GC integration**: DataTypeFunctions::mark ensures referenced Ruby values aren't collected
