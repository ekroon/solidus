//! Wrap and unwrap Rust values as Ruby objects.

use crate::ReprValue;
use crate::error::Error;
use crate::ruby::Ruby;
use crate::types::RClass;
use crate::value::Value;

use super::TypedData;

/// Wrap a Rust value in a Ruby object.
///
/// This creates a new Ruby object that wraps the given Rust value. The Rust value
/// is moved to the heap and will be freed when the Ruby object is garbage collected.
///
/// # Arguments
///
/// * `ruby` - Reference to the Ruby runtime
/// * `class` - The Ruby class for the wrapped object (must be a subclass of Object)
/// * `value` - The Rust value to wrap
///
/// # Returns
///
/// A Ruby `Value` representing the wrapped object.
///
/// # Example
///
/// ```ignore
/// use solidus::prelude::*;
/// use solidus::typed_data::{wrap, TypedData};
///
/// #[solidus::wrap(class = "Point")]
/// struct Point { x: f64, y: f64 }
///
/// fn create_point(ruby: &Ruby, class: &RClass, x: f64, y: f64) -> Result<Value, Error> {
///     let point = Point { x, y };
///     wrap(ruby, class, point)
/// }
/// ```
///
/// # Safety
///
/// The wrapped value will be freed when the Ruby object is collected. Do not
/// attempt to access the value after the Ruby object has been collected.
pub fn wrap<T: TypedData>(_ruby: &Ruby, class: &RClass, value: T) -> Result<Value, Error> {
    // Allocate on heap
    let boxed = Box::new(value);
    let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;

    // Get the data type descriptor
    let data_type = T::data_type();

    // Create the Ruby object
    // SAFETY: class is a valid Ruby class, ptr is a valid heap pointer,
    // data_type describes T correctly
    let raw = unsafe {
        rb_sys::rb_data_typed_object_wrap(class.as_value().as_raw(), ptr, data_type.as_raw())
    };

    // SAFETY: rb_data_typed_object_wrap returns a valid Ruby VALUE
    Ok(unsafe { Value::from_raw(raw) })
}

/// Get a reference to the wrapped Rust value.
///
/// This extracts a reference to the Rust value wrapped in a Ruby object.
/// The reference is valid as long as the Ruby object is not collected.
///
/// # Arguments
///
/// * `value` - A Ruby Value that wraps a `T`
///
/// # Returns
///
/// A reference to the wrapped value, or an error if the value is not a
/// wrapped `T`.
///
/// # Example
///
/// ```ignore
/// use solidus::typed_data::get;
///
/// fn point_x(rb_self: Value) -> Result<f64, Error> {
///     let point: &Point = get(&rb_self)?;
///     Ok(point.x)
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The value is not a TypedData object
/// - The value wraps a different type than `T`
pub fn get<T: TypedData>(value: &Value) -> Result<&T, Error> {
    let data_type = T::data_type();

    // SAFETY: rb_check_typeddata returns NULL if type doesn't match,
    // otherwise returns the data pointer
    let ptr = unsafe { rb_sys::rb_check_typeddata(value.as_raw(), data_type.as_raw()) };

    if ptr.is_null() {
        return Err(Error::type_error(format!(
            "expected {}, got {}",
            T::class_name(),
            value.class_name().unwrap_or_else(|_| "unknown".to_string())
        )));
    }

    // SAFETY: rb_check_typeddata verified this is a T
    let reference = unsafe { &*(ptr as *const T) };
    Ok(reference)
}

/// Get a mutable reference to the wrapped Rust value.
///
/// This extracts a mutable reference to the Rust value wrapped in a Ruby object.
///
/// # Why is this not marked `unsafe`?
///
/// While this function returns a mutable reference that could theoretically alias
/// with other references, Ruby's GVL (Global VM Lock) ensures single-threaded access
/// to Ruby objects. Since Ruby code cannot run in parallel, aliasing cannot occur in
/// practice. However, users must still ensure they don't create aliasing within their
/// own Rust code.
///
/// # Safety Warning
///
/// This function does NOT provide any aliasing guarantees within your Rust code.
/// If you call this while another reference to the same data exists in your Rust code,
/// you will have undefined behavior. For safe mutation, use `RefCell<T>` inside your
/// wrapped type:
///
/// ```ignore
/// use std::cell::RefCell;
///
/// #[solidus::wrap(class = "Counter")]
/// struct Counter(RefCell<i64>);
///
/// impl Counter {
///     fn increment(&self) -> i64 {
///         let mut val = self.0.borrow_mut();
///         *val += 1;
///         *val
///     }
/// }
/// ```
///
/// # Arguments
///
/// * `value` - A Ruby Value that wraps a `T`
///
/// # Returns
///
/// A mutable reference to the wrapped value, or an error if the value is not
/// a wrapped `T`.
///
/// # Errors
///
/// Returns an error if:
/// - The value is not a TypedData object
/// - The value wraps a different type than `T`
#[allow(clippy::mut_from_ref)]
pub fn get_mut<T: TypedData>(value: &Value) -> Result<&mut T, Error> {
    let data_type = T::data_type();

    let ptr = unsafe { rb_sys::rb_check_typeddata(value.as_raw(), data_type.as_raw()) };

    if ptr.is_null() {
        return Err(Error::type_error(format!(
            "expected {}, got {}",
            T::class_name(),
            value.class_name().unwrap_or_else(|_| "unknown".to_string())
        )));
    }

    // SAFETY: rb_check_typeddata verified this is a T
    // WARNING: Caller must ensure no aliasing
    let reference = unsafe { &mut *(ptr as *mut T) };
    Ok(reference)
}
