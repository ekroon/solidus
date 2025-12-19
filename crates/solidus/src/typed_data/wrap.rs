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
/// ```no_run
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
/// ```no_run
/// use solidus::prelude::*;
/// use solidus::typed_data::{get, DataType, DataTypeBuilder, TypedData};
///
/// struct Point { x: f64, y: f64 }
///
/// impl TypedData for Point {
///     fn class_name() -> &'static str { "Point" }
///     fn data_type() -> &'static DataType {
///         static DT: std::sync::OnceLock<DataType> = std::sync::OnceLock::new();
///         DT.get_or_init(|| DataTypeBuilder::<Point>::new("Point").build())
///     }
/// }
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
/// ```no_run
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

#[cfg(all(test, any(feature = "link-ruby", feature = "embed")))]
mod ruby_tests {
    use super::*;
    use crate::convert::TryConvert;
    use crate::typed_data::{DataType, DataTypeBuilder, TypedData};
    use rb_sys_test_helpers::ruby_test;

    struct TestPoint {
        x: f64,
        y: f64,
    }

    impl TypedData for TestPoint {
        fn class_name() -> &'static str {
            "TestPoint"
        }
        fn data_type() -> &'static DataType {
            static DT: std::sync::OnceLock<DataType> = std::sync::OnceLock::new();
            DT.get_or_init(|| DataTypeBuilder::<TestPoint>::new("TestPoint").build())
        }
    }

    #[ruby_test]
    fn test_wrap_and_get() {
        // SAFETY: Ruby is initialized by rb_sys_test_helpers
        let ruby = unsafe { Ruby::get() };
        let object_class_val = ruby.class_object();
        let object_class = RClass::try_convert(object_class_val).unwrap();

        let point = TestPoint { x: 1.0, y: 2.0 };
        let wrapped = wrap(&ruby, &object_class, point).unwrap();

        let retrieved: &TestPoint = get(&wrapped).unwrap();
        assert_eq!(retrieved.x, 1.0);
        assert_eq!(retrieved.y, 2.0);
    }

    #[ruby_test]
    fn test_wrap_and_get_mut() {
        // SAFETY: Ruby is initialized by rb_sys_test_helpers
        let ruby = unsafe { Ruby::get() };
        let object_class_val = ruby.class_object();
        let object_class = RClass::try_convert(object_class_val).unwrap();

        let point = TestPoint { x: 1.0, y: 2.0 };
        let wrapped = wrap(&ruby, &object_class, point).unwrap();

        let retrieved: &mut TestPoint = get_mut(&wrapped).unwrap();
        assert_eq!(retrieved.x, 1.0);
        assert_eq!(retrieved.y, 2.0);

        // Mutate the value
        retrieved.x = 3.0;
        retrieved.y = 4.0;

        // Verify mutation
        let retrieved2: &TestPoint = get(&wrapped).unwrap();
        assert_eq!(retrieved2.x, 3.0);
        assert_eq!(retrieved2.y, 4.0);
    }

    #[ruby_test]
    fn test_multiple_wraps() {
        // SAFETY: Ruby is initialized by rb_sys_test_helpers
        let ruby = unsafe { Ruby::get() };
        let object_class_val = ruby.class_object();
        let object_class = RClass::try_convert(object_class_val).unwrap();

        // Create and wrap multiple points
        let point1 = TestPoint { x: 1.0, y: 2.0 };
        let wrapped1 = wrap(&ruby, &object_class, point1).unwrap();

        let point2 = TestPoint { x: 3.0, y: 4.0 };
        let wrapped2 = wrap(&ruby, &object_class, point2).unwrap();

        // Retrieve and verify both
        let retrieved1: &TestPoint = get(&wrapped1).unwrap();
        assert_eq!(retrieved1.x, 1.0);
        assert_eq!(retrieved1.y, 2.0);

        let retrieved2: &TestPoint = get(&wrapped2).unwrap();
        assert_eq!(retrieved2.x, 3.0);
        assert_eq!(retrieved2.y, 4.0);
    }
}
