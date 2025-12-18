use solidus::prelude::*;
use solidus::typed_data::{get, get_mut, wrap, DataTypeFunctions, Marker};
use std::cell::RefCell;
use std::pin::Pin;
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

// Wrapper functions for Ruby using attribute macros with primitive arguments
#[solidus_macros::function]
fn point_new(x: f64, y: f64) -> Result<Value, Error> {
    let ruby = unsafe { Ruby::get() };
    let class = POINT_CLASS
        .get()
        .ok_or_else(|| Error::runtime("Point class not initialized"))?;
    let point = Point::new(x, y);
    wrap(ruby, class, point)
}

#[solidus_macros::method]
fn point_x(rb_self: Value) -> Result<f64, Error> {
    let point: &Point = get(&rb_self)?;
    Ok(point.x())
}

#[solidus_macros::method]
fn point_y(rb_self: Value) -> Result<f64, Error> {
    let point: &Point = get(&rb_self)?;
    Ok(point.y())
}

#[solidus_macros::method]
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

// Wrapper functions for Ruby using attribute macros with primitive arguments
#[solidus_macros::function]
fn counter_new(initial: i64) -> Result<Value, Error> {
    let ruby = unsafe { Ruby::get() };
    let class = COUNTER_CLASS
        .get()
        .ok_or_else(|| Error::runtime("Counter class not initialized"))?;
    let counter = Counter::new(initial);
    wrap(ruby, class, counter)
}

#[solidus_macros::method]
fn counter_get(rb_self: Value) -> Result<i64, Error> {
    let counter: &Counter = get(&rb_self)?;
    Ok(counter.get())
}

#[solidus_macros::method]
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

// Wrapper functions for Ruby using attribute macros
#[solidus_macros::function]
fn container_new() -> Result<Value, Error> {
    let ruby = unsafe { Ruby::get() };
    let class = CONTAINER_CLASS
        .get()
        .ok_or_else(|| Error::runtime("Container class not initialized"))?;
    let container = Container::new();
    wrap(ruby, class, container)
}

#[solidus_macros::method]
fn container_push(rb_self: Value, value: Pin<&StackPinned<Value>>) -> Result<Value, Error> {
    let container: &mut Container = get_mut(&rb_self)?;
    let boxed = BoxValue::new(value.get().as_value());
    container.push(boxed);
    Ok(rb_self)
}

#[solidus_macros::method]
fn container_len(rb_self: Value) -> Result<usize, Error> {
    let container: &Container = get(&rb_self)?;
    Ok(container.len())
}

#[solidus_macros::method]
fn container_get(rb_self: Value, index: i64) -> Result<Value, Error> {
    let container: &Container = get(&rb_self)?;
    if index < 0 {
        return Err(Error::runtime("Index cannot be negative"));
    }
    match container.get(index as usize) {
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

    // Define all methods on the class using attribute macro generated modules
    point_class.clone().define_singleton_method(
        "new",
        __solidus_function_point_new::wrapper(),
        __solidus_function_point_new::ARITY,
    )?;
    point_class.clone().define_method(
        "x",
        __solidus_method_point_x::wrapper(),
        __solidus_method_point_x::ARITY,
    )?;
    point_class.clone().define_method(
        "y",
        __solidus_method_point_y::wrapper(),
        __solidus_method_point_y::ARITY,
    )?;
    point_class.clone().define_method(
        "distance",
        __solidus_method_point_distance::wrapper(),
        __solidus_method_point_distance::ARITY,
    )?;

    // Then store in OnceLock
    POINT_CLASS
        .set(point_class)
        .map_err(|_| Error::runtime("Point class already initialized"))?;

    // Define Counter class
    let counter_class_val = ruby.define_class("Counter", ruby.class_object());
    let counter_class = RClass::try_convert(counter_class_val)?;

    // Define all methods on the class using attribute macro generated modules
    counter_class.clone().define_singleton_method(
        "new",
        __solidus_function_counter_new::wrapper(),
        __solidus_function_counter_new::ARITY,
    )?;
    counter_class.clone().define_method(
        "get",
        __solidus_method_counter_get::wrapper(),
        __solidus_method_counter_get::ARITY,
    )?;
    counter_class.clone().define_method(
        "increment",
        __solidus_method_counter_increment::wrapper(),
        __solidus_method_counter_increment::ARITY,
    )?;

    // Then store in OnceLock
    COUNTER_CLASS
        .set(counter_class)
        .map_err(|_| Error::runtime("Counter class already initialized"))?;

    // Define Container class
    let container_class_val = ruby.define_class("Container", ruby.class_object());
    let container_class = RClass::try_convert(container_class_val)?;

    // Define all methods on the class using attribute macro generated modules
    container_class.clone().define_singleton_method(
        "new",
        __solidus_function_container_new::wrapper(),
        __solidus_function_container_new::ARITY,
    )?;
    container_class.clone().define_method(
        "push",
        __solidus_method_container_push::wrapper(),
        __solidus_method_container_push::ARITY,
    )?;
    container_class.clone().define_method(
        "len",
        __solidus_method_container_len::wrapper(),
        __solidus_method_container_len::ARITY,
    )?;
    container_class.clone().define_method(
        "get",
        __solidus_method_container_get::wrapper(),
        __solidus_method_container_get::ARITY,
    )?;

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
