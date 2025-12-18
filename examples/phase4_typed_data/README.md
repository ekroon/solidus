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

## Primitive Argument Support

This example demonstrates the `#[solidus_macros::method]` and `#[solidus_macros::function]`
attribute macros with support for Rust primitive types as direct arguments:

```rust
// Primitive f64 arguments - no wrapping needed
#[solidus_macros::function]
fn point_new(x: f64, y: f64) -> Result<Value, Error> { ... }

// Primitive i64 argument
#[solidus_macros::function]
fn counter_new(initial: i64) -> Result<Value, Error> { ... }

// ALL Ruby VALUE types (including self) use Pin<&StackPinned<T>> for GC safety
#[solidus_macros::method]
fn point_x(rb_self: Pin<&StackPinned<Value>>) -> Result<f64, Error> {
    let point: &Point = get(rb_self.get())?;
    Ok(point.x())
}

// Mixed: pinned self with primitive i64 index
#[solidus_macros::method]
fn container_get(rb_self: Pin<&StackPinned<Value>>, index: i64) -> Result<Value, Error> {
    let container: &Container = get(rb_self.get())?;
    // ...
}

// Both self and value arguments use Pin<&StackPinned<T>>
#[solidus_macros::method]
fn container_push(rb_self: Pin<&StackPinned<Value>>, value: Pin<&StackPinned<Value>>) -> Result<Value, Error> {
    let container: &mut Container = get_mut(rb_self.get())?;
    // ...
}
```

### Why Self Needs Pinning

The self parameter (like all Ruby VALUE types) needs `Pin<&StackPinned<Value>>` because:

1. **GC Safety**: If we stored the self VALUE in a Vec or on the heap and then lost the 
   stack reference on the Ruby side, it might get garbage collected.
2. **Consistency**: The pinning requirement ensures users must use `BoxValue` if they want 
   heap storage, which properly registers with Ruby's GC.
3. **Compile-time Safety**: The type system prevents accidental heap storage of VALUEs.

### Pinning Rules

- **If `TryConvert` creates a new Rust value** (copy/clone like `i64`, `f64`, `String`): No pinning needed
- **If `TryConvert` wraps a Ruby VALUE** (like `Value`, `RString`, `RArray`): Pinning IS required
- **Self parameter**: Always pinned for Ruby VALUE types

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
