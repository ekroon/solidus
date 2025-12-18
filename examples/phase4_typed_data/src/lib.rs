use solidus::prelude::*;
use solidus::typed_data::{get, get_mut, wrap, DataTypeFunctions, Marker};
use std::cell::RefCell;
use std::sync::OnceLock;

// ============================================================================
// Task 4.7.1: Basic Point example
// ============================================================================

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

    fn distance(&self, other: &Point) -> f64 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }
}

// Global storage for class references
static POINT_CLASS: OnceLock<RClass> = OnceLock::new();
static COUNTER_CLASS: OnceLock<RClass> = OnceLock::new();
static CONTAINER_CLASS: OnceLock<RClass> = OnceLock::new();

// Wrapper functions for Ruby
fn point_new(
    x: Pin<&StackPinned<Float>>,
    y: Pin<&StackPinned<Float>>,
) -> Result<Value, Error> {
    let ruby = unsafe { Ruby::get() };
    let class = POINT_CLASS
        .get()
        .ok_or_else(|| Error::runtime("Point class not initialized"))?;
    let x_val = x.get().to_f64();
    let y_val = y.get().to_f64();
    let point = Point::new(x_val, y_val);
    wrap(ruby, class, point)
}

fn point_x(rb_self: Value) -> Result<f64, Error> {
    let point: &Point = get(&rb_self)?;
    Ok(point.x())
}

fn point_y(rb_self: Value) -> Result<f64, Error> {
    let point: &Point = get(&rb_self)?;
    Ok(point.y())
}

fn point_distance(rb_self: Value, other: Pin<&StackPinned<Value>>) -> Result<f64, Error> {
    let point: &Point = get(&rb_self)?;
    let other_point: &Point = get(other.get())?;
    Ok(point.distance(other_point))
}

// ============================================================================
// Task 4.7.2: Counter example with RefCell for safe mutation
// ============================================================================

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

// Wrapper functions for Ruby
fn counter_new(initial: Pin<&StackPinned<Integer>>) -> Result<Value, Error> {
    let ruby = unsafe { Ruby::get() };
    let class = COUNTER_CLASS
        .get()
        .ok_or_else(|| Error::runtime("Counter class not initialized"))?;
    let initial_val = initial.get().to_i64()?;
    let counter = Counter::new(initial_val);
    wrap(ruby, class, counter)
}

fn counter_get(rb_self: Value) -> Result<i64, Error> {
    let counter: &Counter = get(&rb_self)?;
    Ok(counter.get())
}

fn counter_increment(rb_self: Value) -> Result<i64, Error> {
    let counter: &Counter = get(&rb_self)?;
    Ok(counter.increment())
}

// ============================================================================
// Task 4.7.3: Container example with GC marking
// ============================================================================

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

// Wrapper functions for Ruby
fn container_new() -> Result<Value, Error> {
    let ruby = unsafe { Ruby::get() };
    let class = CONTAINER_CLASS
        .get()
        .ok_or_else(|| Error::runtime("Container class not initialized"))?;
    let container = Container::new();
    wrap(ruby, class, container)
}

fn container_push(rb_self: Value, value: Pin<&StackPinned<Value>>) -> Result<Value, Error> {
    let container: &mut Container = get_mut(&rb_self)?;
    let boxed = BoxValue::new(value.get().as_value());
    container.push(boxed);
    Ok(rb_self)
}

fn container_len(rb_self: Value) -> Result<usize, Error> {
    let container: &Container = get(&rb_self)?;
    Ok(container.len())
}

fn container_get(rb_self: Value, index: Pin<&StackPinned<Integer>>) -> Result<Value, Error> {
    let container: &Container = get(&rb_self)?;
    let idx = index.get().to_u64()? as usize;
    match container.get(idx) {
        Some(boxed) => Ok(boxed.as_value()),
        None => Err(Error::runtime("Index out of bounds")),
    }
}

// ============================================================================
// Init function
// ============================================================================

fn init(ruby: &Ruby) -> Result<(), Error> {
    // Define Point class
    let point_class_val = ruby.define_class("Point", ruby.class_object());
    let point_class = RClass::try_convert(point_class_val)?;
    
    // Define all methods on the class (must clone since define_method consumes self)
    point_class.clone().define_singleton_method("new", solidus::function!(point_new, 2), 2)?;
    point_class.clone().define_method("x", solidus::method!(point_x, 0), 0)?;
    point_class.clone().define_method("y", solidus::method!(point_y, 0), 0)?;
    point_class.clone().define_method("distance", solidus::method!(point_distance, 1), 1)?;
    
    // Then store in OnceLock
    POINT_CLASS
        .set(point_class)
        .map_err(|_| Error::runtime("Point class already initialized"))?;

    // Define Counter class
    let counter_class_val = ruby.define_class("Counter", ruby.class_object());
    let counter_class = RClass::try_convert(counter_class_val)?;
    
    // Define all methods on the class (must clone since define_method consumes self)
    counter_class.clone().define_singleton_method("new", solidus::function!(counter_new, 1), 1)?;
    counter_class.clone().define_method("get", solidus::method!(counter_get, 0), 0)?;
    counter_class.clone().define_method("increment", solidus::method!(counter_increment, 0), 0)?;
    
    // Then store in OnceLock
    COUNTER_CLASS
        .set(counter_class)
        .map_err(|_| Error::runtime("Counter class already initialized"))?;

    // Define Container class
    let container_class_val = ruby.define_class("Container", ruby.class_object());
    let container_class = RClass::try_convert(container_class_val)?;
    
    // Define all methods on the class (must clone since define_method consumes self)
    container_class.clone().define_singleton_method("new", solidus::function!(container_new, 0), 0)?;
    container_class.clone().define_method("push", solidus::method!(container_push, 1), 1)?;
    container_class.clone().define_method("len", solidus::method!(container_len, 0), 0)?;
    container_class.clone().define_method("get", solidus::method!(container_get, 1), 1)?;
    
    // Then store in OnceLock
    CONTAINER_CLASS
        .set(container_class)
        .map_err(|_| Error::runtime("Container class already initialized"))?;

    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn Init_phase4_typed_data() {
    // Mark this thread as the Ruby thread
    Ruby::mark_ruby_thread();

    // Get the Ruby handle
    let ruby = Ruby::get();

    // Call the init function and raise on error
    if let Err(e) = init(ruby) {
        e.raise();
    }
}
