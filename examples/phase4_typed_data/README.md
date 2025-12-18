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

### Task 4.7.2: Counter with RefCell

Shows how to:
- Use `RefCell<T>` for interior mutability
- Safely mutate wrapped data from Ruby method calls
- Follow Rust's borrowing rules even when called from Ruby

### Task 4.7.3: Container with GC Marking

Shows how to:
- Store Ruby values inside wrapped Rust types using `Vec<BoxValue<Value>>`
- Implement `DataTypeFunctions` to mark contained Ruby values for GC
- Use `#[solidus::wrap(class = "Container", mark)]` to enable GC marking
- Prevent Ruby values from being garbage collected while stored in Rust

## Note on Compilation

**This example currently does not compile** because the `function!` and `method!` macros
don't yet support Rust primitive types (f64, i64, usize) as direct arguments. The macros
currently expect all arguments to be Ruby VALUE types wrapped in `Pin<&StackPinned<T>>`.

To make this compile, the function signatures would need to use:
- `Pin<&StackPinned<Float>>` instead of `f64`
- `Pin<&StackPinned<Integer>>` instead of `i64`/`usize`

This limitation will be addressed in future macro improvements. The TypedData API itself
(wrap, get, get_mut, DataTypeFunctions) is fully functional.

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
