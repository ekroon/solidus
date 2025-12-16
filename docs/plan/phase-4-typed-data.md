# Phase 4: TypedData

## Objective

Allow Rust types to be wrapped as Ruby objects, enabling methods to be defined on them.

## Dependencies

- Phase 3 complete

## Core Concepts

### TypedData in Ruby

Ruby's `TypedData` API allows C extensions to wrap native data in Ruby objects with:
- Custom marking (for GC)
- Custom freeing
- Custom compaction (for GC compaction)
- Size reporting

### Solidus Approach

```rust
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
    
    fn distance(&self, other: &Point) -> f64 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }
}

#[solidus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("Point", ruby.class_object())?;
    class.define_singleton_method("new", function!(Point::new, 2))?;
    class.define_method("x", method!(Point::x, 0))?;
    class.define_method("distance", method!(Point::distance, 1))?;
    Ok(())
}
```

## Tasks

### 4.1 TypedData Trait

```rust
// crates/solidus/src/typed_data/traits.rs

/// Trait for Rust types that can be wrapped in Ruby objects.
pub trait TypedData: Sized + Send {
    /// The Ruby class name.
    fn class_name() -> &'static str;
    
    /// The DataType descriptor.
    fn data_type() -> &'static DataType;
}

/// Optional trait for types that contain Ruby values.
pub trait DataTypeFunctions: TypedData {
    /// Mark any Ruby values this type contains.
    fn mark(&self, marker: &Marker) {}
    
    /// Update any Ruby values after GC compaction.
    fn compact(&self, compactor: &Compactor) {}
    
    /// Report the size of this value for GC statistics.
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
```

- [ ] Define `TypedData` trait
- [ ] Define `DataTypeFunctions` trait
- [ ] Implement marker/compactor helpers

### 4.2 DataType Struct

```rust
// crates/solidus/src/typed_data/data_type.rs

/// Describes a Rust type to Ruby's TypedData system.
pub struct DataType {
    name: &'static str,
    free: unsafe extern "C" fn(*mut c_void),
    mark: Option<unsafe extern "C" fn(*mut c_void)>,
    compact: Option<unsafe extern "C" fn(*mut c_void)>,
    size: Option<unsafe extern "C" fn(*const c_void) -> usize>,
}

impl DataType {
    pub const fn builder<T: TypedData>(name: &'static str) -> DataTypeBuilder<T>;
}
```

- [ ] Implement `DataType` struct
- [ ] Implement `DataTypeBuilder`
- [ ] Generate correct rb_data_type_t

### 4.3 Wrap Macro

```rust
// In solidus-macros

#[solidus::wrap(class = "Point", free_immediately)]
struct Point { ... }

// Generates:
impl TypedData for Point {
    fn class_name() -> &'static str { "Point" }
    
    fn data_type() -> &'static DataType {
        static DATA_TYPE: DataType = DataType::builder::<Point>("Point")
            .free_immediately()
            .build();
        &DATA_TYPE
    }
}
```

Macro options:
- `class = "Name"` - Ruby class name (required)
- `free_immediately` - Free when Ruby object is collected
- `mark` - Enable GC marking
- `size` - Enable size reporting

- [ ] Implement `#[wrap]` macro
- [ ] Support all options
- [ ] Generate correct trait implementations

### 4.4 Object Wrapping

```rust
// crates/solidus/src/typed_data/mod.rs

/// Wrap a Rust value in a Ruby object.
pub fn wrap<T: TypedData>(ruby: &Ruby, value: T) -> Result<Value, Error>;

/// Get a reference to the wrapped Rust value.
pub fn get<T: TypedData>(value: Value) -> Result<&T, Error>;

/// Get a mutable reference to the wrapped Rust value.
pub fn get_mut<T: TypedData>(value: Value) -> Result<&mut T, Error>;
```

- [ ] Implement `wrap` function
- [ ] Implement `get` / `get_mut` functions
- [ ] Handle type checking

### 4.5 Mutability with RefCell

For mutable wrapped types:

```rust
#[solidus::wrap(class = "Counter")]
struct Counter(RefCell<i64>);

impl Counter {
    fn new() -> Self {
        Self(RefCell::new(0))
    }
    
    fn increment(&self) -> i64 {
        let mut val = self.0.borrow_mut();
        *val += 1;
        *val
    }
}
```

- [ ] Document `RefCell` pattern
- [ ] Add example
- [ ] Test runtime borrow checking

### 4.6 Types Containing Ruby Values

For wrapped types that hold Ruby values:

```rust
#[solidus::wrap(class = "Container", mark)]
struct Container {
    items: Vec<BoxValue<Value>>,
}

impl DataTypeFunctions for Container {
    fn mark(&self, marker: &Marker) {
        for item in &self.items {
            marker.mark(**item);
        }
    }
}
```

- [ ] Implement `Marker` helper
- [ ] Implement `Compactor` helper
- [ ] Document the pattern
- [ ] Add tests

### 4.7 Derive Macro

Optional derive for default implementations:

```rust
#[derive(TypedData)]
#[solidus(class = "Point")]
struct Point {
    x: f64,
    y: f64,
}
```

- [ ] Implement derive macro (if time permits)
- [ ] Generate default DataTypeFunctions

## Acceptance Criteria

- [ ] `#[wrap]` macro generates correct trait implementations
- [ ] Rust types can be wrapped and unwrapped
- [ ] GC marking works for types with Ruby values
- [ ] RefCell pattern documented and tested
- [ ] Memory is correctly freed
- [ ] Type checking prevents wrong type access
