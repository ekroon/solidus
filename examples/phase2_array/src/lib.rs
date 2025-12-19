//! Phase 2 Stage 5: Array Type Example
//!
//! This example demonstrates Ruby's Array type with iteration support.
//! Arrays are heap-allocated objects that require GC protection.
//!
//! This shows Stage 5 implementation: RArray type with dynamic arrays and type-safe operations.

use solidus::prelude::*;

/// Example 1: Creating empty arrays
///
/// Demonstrates basic array creation.
#[no_mangle]
pub extern "C" fn example_array_new() -> rb_sys::VALUE {
    // Create a new empty array
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::new() };

    // Check basic properties
    assert_eq!(arr.len(), 0);
    assert!(arr.is_empty());

    arr.into_value().as_raw()
}

/// Example 2: Creating arrays with capacity
///
/// Shows pre-allocation for better performance.
#[no_mangle]
pub extern "C" fn example_array_with_capacity() -> rb_sys::VALUE {
    // Create an array with pre-allocated capacity
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::with_capacity(100) };

    // Still empty, but has space for 100 elements
    assert_eq!(arr.len(), 0);
    assert!(arr.is_empty());

    // Adding elements won't trigger reallocation
    for i in 0..100 {
        arr.push(i as i64);
    }

    assert_eq!(arr.len(), 100);

    arr.into_value().as_raw()
}

/// Example 3: Pushing and popping elements
///
/// Demonstrates stack operations on arrays.
#[no_mangle]
pub extern "C" fn example_array_push_pop() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::new() };

    // Push some elements
    arr.push(10i64);
    arr.push(20i64);
    arr.push(30i64);

    assert_eq!(arr.len(), 3);

    // Pop the last element
    let last = arr.pop().unwrap();
    assert_eq!(i64::try_convert(last).unwrap(), 30);
    assert_eq!(arr.len(), 2);

    // Pop another
    let second = arr.pop().unwrap();
    assert_eq!(i64::try_convert(second).unwrap(), 20);
    assert_eq!(arr.len(), 1);

    // One element remains
    let first = arr.pop().unwrap();
    assert_eq!(i64::try_convert(first).unwrap(), 10);
    assert!(arr.is_empty());

    // Popping from empty array returns None
    assert!(arr.pop().is_none());

    arr.into_value().as_raw()
}

/// Example 4: Element access by index
///
/// Shows positive and negative indexing.
#[no_mangle]
pub extern "C" fn example_array_entry() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::new() };
    arr.push(100i64);
    arr.push(200i64);
    arr.push(300i64);
    arr.push(400i64);
    arr.push(500i64);

    // Positive indices
    let val0 = arr.entry(0);
    assert_eq!(i64::try_convert(val0).unwrap(), 100);

    let val2 = arr.entry(2);
    assert_eq!(i64::try_convert(val2).unwrap(), 300);

    let val4 = arr.entry(4);
    assert_eq!(i64::try_convert(val4).unwrap(), 500);

    // Negative indices (count from end)
    let val_last = arr.entry(-1);
    assert_eq!(i64::try_convert(val_last).unwrap(), 500);

    let val_second_last = arr.entry(-2);
    assert_eq!(i64::try_convert(val_second_last).unwrap(), 400);

    // Out of bounds returns nil
    let val_oob = arr.entry(100);
    assert!(val_oob.is_nil());

    let val_neg_oob = arr.entry(-100);
    assert!(val_neg_oob.is_nil());

    arr.into_value().as_raw()
}

/// Example 5: Storing elements at indices
///
/// Demonstrates modifying array elements.
#[no_mangle]
pub extern "C" fn example_array_store() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::new() };

    // Store at index 0
    arr.store(0, 42i64);
    assert_eq!(arr.len(), 1);

    let val = arr.entry(0);
    assert_eq!(i64::try_convert(val).unwrap(), 42);

    // Replace existing element
    arr.store(0, 99i64);
    let val = arr.entry(0);
    assert_eq!(i64::try_convert(val).unwrap(), 99);

    // Store beyond current length extends array with nils
    arr.store(5, 123i64);
    assert_eq!(arr.len(), 6);

    // Elements 1-4 are nil
    for i in 1..5 {
        assert!(arr.entry(i).is_nil());
    }

    let val5 = arr.entry(5);
    assert_eq!(i64::try_convert(val5).unwrap(), 123);

    // Negative indices work too
    arr.store(-1, 456i64);
    let val_last = arr.entry(-1);
    assert_eq!(i64::try_convert(val_last).unwrap(), 456);

    arr.into_value().as_raw()
}

/// Example 6: Iterating with each()
///
/// Shows closure-based iteration over array elements.
#[no_mangle]
pub extern "C" fn example_array_each() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::new() };

    // Add some numbers
    for i in 1..=10 {
        arr.push(i as i64);
    }

    // Sum all elements using each()
    let mut sum = 0i64;
    arr.each(|val| {
        let n = i64::try_convert(val)?;
        sum += n;
        Ok(())
    })
    .unwrap();

    assert_eq!(sum, 55); // 1+2+3+...+10 = 55

    // Count elements
    let mut count = 0;
    arr.each(|_| {
        count += 1;
        Ok(())
    })
    .unwrap();

    assert_eq!(count, 10);

    arr.into_value().as_raw()
}

/// Example 7: Creating arrays from slices
///
/// Demonstrates converting Rust slices to Ruby arrays.
#[no_mangle]
pub extern "C" fn example_array_from_slice() -> rb_sys::VALUE {
    // Create array from integer slice
    let numbers = &[1i64, 2, 3, 4, 5];
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::from_slice(numbers) };

    assert_eq!(arr.len(), 5);

    // Verify contents
    for (i, &expected) in numbers.iter().enumerate() {
        let val = arr.entry(i as isize);
        let actual = i64::try_convert(val).unwrap();
        assert_eq!(actual, expected);
    }

    arr.into_value().as_raw()
}

/// Example 8: Converting arrays to Vec
///
/// Shows type-safe conversion from Ruby arrays to Rust vectors.
#[no_mangle]
pub extern "C" fn example_array_to_vec() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::new() };
    arr.push(10i64);
    arr.push(20i64);
    arr.push(30i64);
    arr.push(40i64);
    arr.push(50i64);

    // Convert to Rust Vec<i64>
    let vec: Vec<i64> = arr.to_vec().unwrap();

    assert_eq!(vec.len(), 5);
    assert_eq!(vec, vec![10, 20, 30, 40, 50]);

    // We can now use Rust iterator methods
    let sum: i64 = vec.iter().sum();
    assert_eq!(sum, 150);

    let doubled: Vec<i64> = vec.iter().map(|x| x * 2).collect();
    assert_eq!(doubled, vec![20, 40, 60, 80, 100]);

    arr.into_value().as_raw()
}

/// Example 9: Mixed-type arrays
///
/// Demonstrates heterogeneous arrays with different Ruby types.
#[no_mangle]
pub extern "C" fn example_array_mixed_types() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::new() };

    // Ruby arrays can hold different types
    arr.push(42i64);
    // SAFETY: Value is used immediately
    arr.push(unsafe { RString::new("hello") });
    arr.push(true);
    arr.push(2.5f64);

    assert_eq!(arr.len(), 4);

    // Access each element with its type
    let val0 = arr.entry(0);
    assert_eq!(i64::try_convert(val0).unwrap(), 42);

    let val1 = arr.entry(1);
    let s = RString::try_convert(val1).unwrap();
    assert_eq!(s.to_string().unwrap(), "hello");

    let val2 = arr.entry(2);
    assert!(bool::try_convert(val2).unwrap());

    let val3 = arr.entry(3);
    let float_val = f64::try_convert(val3).unwrap();
    assert!((float_val - 2.5).abs() < f64::EPSILON);

    arr.into_value().as_raw()
}

/// Example 10: Type-safe typed arrays
///
/// Shows working with homogeneous arrays of a specific type.
#[no_mangle]
pub extern "C" fn example_typed_array() -> rb_sys::VALUE {
    // Create an array of strings
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::new() };
    arr.push(unsafe { RString::new("apple") });
    arr.push(unsafe { RString::new("banana") });
    arr.push(unsafe { RString::new("cherry") });

    // Convert to Vec<String> with type checking
    let mut strings = Vec::new();
    arr.each(|val| {
        let rstr = RString::try_convert(val)?;
        strings.push(rstr.to_string()?);
        Ok(())
    })
    .unwrap();

    assert_eq!(strings, vec!["apple", "banana", "cherry"]);

    arr.into_value().as_raw()
}

/// Example 11: Nested arrays
///
/// Demonstrates multi-dimensional arrays.
#[no_mangle]
pub extern "C" fn example_nested_arrays() -> rb_sys::VALUE {
    // Create a 2D array (array of arrays)
    // SAFETY: Values are used immediately and returned to Ruby
    let row1 = unsafe { RArray::from_slice(&[1i64, 2, 3]) };
    let row2 = unsafe { RArray::from_slice(&[4i64, 5, 6]) };
    let row3 = unsafe { RArray::from_slice(&[7i64, 8, 9]) };

    let matrix = unsafe { RArray::new() };
    matrix.push(row1);
    matrix.push(row2);
    matrix.push(row3);

    assert_eq!(matrix.len(), 3);

    // Access elements in the nested array
    let first_row = RArray::try_convert(matrix.entry(0)).unwrap();
    assert_eq!(first_row.len(), 3);

    let val = first_row.entry(0);
    assert_eq!(i64::try_convert(val).unwrap(), 1);

    // Access element at [1][2] (second row, third column)
    let second_row = RArray::try_convert(matrix.entry(1)).unwrap();
    let val_1_2 = second_row.entry(2);
    assert_eq!(i64::try_convert(val_1_2).unwrap(), 6);

    matrix.into_value().as_raw()
}

/// Example 12: Error handling with type mismatches
///
/// Shows proper error handling when converting array elements.
#[no_mangle]
pub extern "C" fn example_array_error_handling() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let arr = unsafe { RArray::new() };
    arr.push(1i64);
    arr.push(2i64);
    arr.push(unsafe { RString::new("not a number") }); // Type mismatch
    arr.push(4i64);

    // Try to convert to Vec<i64> - will fail at the string
    let result: Result<Vec<i64>, Error> = arr.to_vec();

    // Should fail because of the string element
    assert!(result.is_err());

    // Can handle mixed types by using Value
    let mut values = Vec::new();
    arr.each(|val| {
        values.push(val);
        Ok(())
    })
    .unwrap();

    assert_eq!(values.len(), 4);

    // Can selectively convert elements that match the type
    let mut numbers = Vec::new();
    arr.each(|val| {
        if let Ok(n) = i64::try_convert(val) {
            numbers.push(n);
        }
        Ok(())
    })
    .unwrap();

    assert_eq!(numbers, vec![1, 2, 4]); // String skipped

    arr.into_value().as_raw()
}

/// Initialize the extension
#[no_mangle]
pub extern "C" fn Init_phase2_array() {
    // Note: Full method definition requires Phase 3
    // For now, this is just a placeholder that Ruby will call when loading the extension
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_checks() {
        // These tests verify compile-time behavior only
        // Tests requiring Ruby API calls need the Ruby runtime

        // Verify RArray is Copy (it's just a VALUE wrapper)
        fn assert_copy<T: Copy>() {}
        assert_copy::<RArray>();
    }

    #[test]
    fn test_array_types() {
        // Verify type properties without calling Ruby API

        // RArray should be transparent wrapper around Value
        assert_eq!(std::mem::size_of::<RArray>(), std::mem::size_of::<Value>());
    }
}
