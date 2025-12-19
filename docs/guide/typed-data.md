# TypedData: Wrapping Rust Types as Ruby Objects

TypedData is Ruby's mechanism for wrapping arbitrary C (or Rust) data structures
as Ruby objects with proper garbage collection integration. Solidus provides a
safe, ergonomic API for creating these wrapped types.

## When to Use TypedData

Use TypedData when you want to:

- Expose Rust structs to Ruby as first-class objects
- Store Rust data that lives beyond a single method call
- Create Ruby classes backed by efficient Rust implementations
- Maintain state in Rust while providing a Ruby API

TypedData is the foundation for building Ruby gems with Rust implementations.

## Basic Usage with `#[solidus::wrap]`

The simplest way to wrap a Rust type is with the `#[solidus::wrap]` attribute macro:

```rust
use solidus::prelude::*;

#[solidus::wrap(class = "Point")]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}
```

The `#[solidus::wrap]` macro generates the `TypedData` trait implementation,
which tells Ruby how to handle instances of your type.

### Macro Options

The `#[solidus::wrap]` attribute accepts several options:

| Option | Description |
|--------|-------------|
| `class = "Name"` | (Required) The Ruby class name |
| `free_immediately` | Free memory when object is collected (default: true) |
| `mark` | Enable GC marking (for types containing Ruby values) |
| `compact` | Enable GC compaction support |
| `size` | Enable memory size reporting |

## Wrapping and Unwrapping Values

### Creating Wrapped Objects

Use `wrap()` to create a Ruby object containing your Rust value:

```rust
use solidus::prelude::*;
use solidus::typed_data::wrap;

fn create_point(ruby: &Ruby, class: &RClass, x: f64, y: f64) -> Result<Value, Error> {
    let point = Point::new(x, y);
    wrap(ruby, class, point)
}
```

### Accessing the Wrapped Data

Use `get()` for immutable access:

```rust
use solidus::typed_data::get;

fn point_x(rb_self: &Value) -> Result<f64, Error> {
    let point: &Point = get(rb_self)?;
    Ok(point.x())
}
```

Use `get_mut()` when you need mutable access:

```rust
use solidus::typed_data::get_mut;

fn set_x(rb_self: &Value, new_x: f64) -> Result<(), Error> {
    let point: &mut Point = get_mut(rb_self)?;
    point.x = new_x;
    Ok(())
}
```

**Warning:** `get_mut()` does not provide aliasing protection within Rust code.
For safe interior mutability, use `RefCell` (see next section).

## Mutable Types with RefCell

For types that need safe mutation, wrap mutable fields in `RefCell`:

```rust
use solidus::prelude::*;
use std::cell::RefCell;

#[solidus::wrap(class = "Counter")]
struct Counter {
    value: RefCell<i64>,
}

impl Counter {
    fn new(initial: i64) -> Self {
        Self {
            value: RefCell::new(initial),
        }
    }

    fn get(&self) -> i64 {
        *self.value.borrow()
    }

    fn increment(&self) -> i64 {
        let mut val = self.value.borrow_mut();
        *val += 1;
        *val
    }
}
```

This pattern allows mutation through `&self` references, which is safe because:

1. Ruby's GVL ensures single-threaded access to Ruby objects
2. `RefCell` provides runtime borrow checking within your Rust code

## Types Containing Ruby Values

When your wrapped type stores Ruby values, you must tell the garbage collector
about them so they don't get collected prematurely. Use `BoxValue<T>` to store
Ruby values and implement `DataTypeFunctions` for GC marking.

```rust
use solidus::prelude::*;
use solidus::typed_data::{DataTypeFunctions, Marker};

#[solidus::wrap(class = "Container", mark)]
struct Container {
    items: Vec<BoxValue<Value>>,
}

impl Container {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn push(&mut self, value: BoxValue<Value>) {
        self.items.push(value);
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn get(&self, index: usize) -> Option<&BoxValue<Value>> {
        self.items.get(index)
    }
}

impl DataTypeFunctions for Container {
    fn mark(&self, marker: &Marker) {
        for item in &self.items {
            marker.mark_boxed(item);
        }
    }
}
```

Key points:

- Use `mark` in the `#[wrap]` attribute to enable GC marking
- Store Ruby values in `BoxValue<T>` for heap storage
- Implement `DataTypeFunctions::mark()` to mark all contained Ruby values
- The `Marker` provides `mark()` and `mark_boxed()` methods

## The DataTypeFunctions Trait

`DataTypeFunctions` provides hooks into Ruby's garbage collector:

```rust
pub trait DataTypeFunctions: TypedData {
    /// Mark any Ruby values this type contains.
    fn mark(&self, marker: &Marker) {}

    /// Update references after GC compaction.
    fn compact(&mut self, compactor: &Compactor) {}

    /// Report memory size for GC statistics.
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
```

### When to Implement Each Method

| Method | When to Implement |
|--------|------------------|
| `mark` | Your type contains `BoxValue<T>` or raw Ruby VALUEs |
| `compact` | Your type stores raw `rb_sys::VALUE` that may move during compaction |
| `size` | Your type allocates memory beyond `size_of::<Self>()` (e.g., `Vec`, `String`) |

### Example with Size Reporting

```rust
impl DataTypeFunctions for Container {
    fn mark(&self, marker: &Marker) {
        for item in &self.items {
            marker.mark_boxed(item);
        }
    }

    fn size(&self) -> usize {
        std::mem::size_of::<Self>() +
            self.items.capacity() * std::mem::size_of::<BoxValue<Value>>()
    }
}
```

## Defining Methods on Wrapped Types

Combine TypedData with the method macros to define a complete Ruby class:

```rust
use solidus::prelude::*;
use solidus::typed_data::{get, wrap};
use std::pin::Pin;
use std::sync::OnceLock;

#[solidus::wrap(class = "Point")]
struct Point {
    x: f64,
    y: f64,
}

// Store the class for use in constructors
static POINT_CLASS: OnceLock<RClass> = OnceLock::new();

// Constructor function
#[solidus::function]
fn point_new(x: f64, y: f64) -> Result<Value, Error> {
    let ruby = unsafe { Ruby::get() };
    let class = POINT_CLASS
        .get()
        .ok_or_else(|| Error::runtime("Point class not initialized"))?;
    wrap(ruby, class, Point { x, y })
}

// Instance methods
#[solidus::method]
fn point_x(rb_self: Pin<&StackPinned<Value>>) -> Result<f64, Error> {
    let point: &Point = get(rb_self.get())?;
    Ok(point.x)
}

#[solidus::method]
fn point_y(rb_self: Pin<&StackPinned<Value>>) -> Result<f64, Error> {
    let point: &Point = get(rb_self.get())?;
    Ok(point.y)
}

#[solidus::method]
fn point_distance(
    rb_self: Pin<&StackPinned<Value>>,
    other: Pin<&StackPinned<Value>>
) -> Result<f64, Error> {
    let p1: &Point = get(rb_self.get())?;
    let p2: &Point = get(other.get())?;
    Ok(((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt())
}

// Registration
#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class_val = ruby.define_class("Point", ruby.class_object());
    let class = RClass::try_convert(class_val)?;

    // Register methods using generated modules
    class.clone().define_singleton_method(
        "new",
        __solidus_function_point_new::wrapper(),
        __solidus_function_point_new::ARITY,
    )?;
    class.clone().define_method(
        "x",
        __solidus_method_point_x::wrapper(),
        __solidus_method_point_x::ARITY,
    )?;
    class.clone().define_method(
        "y",
        __solidus_method_point_y::wrapper(),
        __solidus_method_point_y::ARITY,
    )?;
    class.clone().define_method(
        "distance",
        __solidus_method_point_distance::wrapper(),
        __solidus_method_point_distance::ARITY,
    )?;

    POINT_CLASS.set(class).map_err(|_| Error::runtime("Already initialized"))?;

    Ok(())
}
```

## Manual TypedData Implementation

For advanced use cases, you can implement `TypedData` manually:

```rust
use solidus::typed_data::{DataType, DataTypeBuilder, TypedData};
use std::sync::OnceLock;

struct Point {
    x: f64,
    y: f64,
}

impl TypedData for Point {
    fn class_name() -> &'static str {
        "Point"
    }

    fn data_type() -> &'static DataType {
        static DATA_TYPE: OnceLock<DataType> = OnceLock::new();
        DATA_TYPE.get_or_init(|| {
            DataTypeBuilder::<Point>::new("Point")
                .free_immediately()
                .build()
        })
    }
}
```

### DataTypeBuilder Methods

| Method | Description |
|--------|-------------|
| `new(name)` | Create builder with type name |
| `free_immediately()` | Free when collected (default) |
| `mark()` | Enable mark callback (requires `DataTypeFunctions`) |
| `compact()` | Enable compact callback (requires `DataTypeFunctions`) |
| `size()` | Enable size callback (requires `DataTypeFunctions`) |
| `build()` | Build without GC callbacks |
| `build_with_callbacks()` | Build with enabled GC callbacks |

## Summary

TypedData in Solidus provides:

1. **Simple wrapping** with `#[solidus::wrap]`
2. **Safe access** via `get()` and `get_mut()`
3. **Interior mutability** with `RefCell` for mutable types
4. **GC integration** via `DataTypeFunctions` for types containing Ruby values
5. **Method definition** combining with `#[solidus::method]` and `#[solidus::function]`

For complete working examples, see `examples/phase4_typed_data/` in the repository.
