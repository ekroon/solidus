//! Collections Example
//!
//! This example demonstrates working with Ruby collections (RArray and RHash)
//! including iteration, building, and converting between Rust and Ruby types.
//!
//! Key patterns covered:
//! - Iterating collections with `each()`
//! - Building collections with `push()` and `insert()`
//! - Converting `Vec<T>` <-> `RArray` and `HashMap<K,V>` <-> `RHash`

use std::collections::HashMap;

use solidus::prelude::*;

// =============================================================================
// PART 1: Working with Arrays
// =============================================================================

/// Example 1: Building an array from scratch
///
/// Demonstrates creating an empty array and populating it with push().
#[no_mangle]
pub extern "C" fn build_array() -> rb_sys::VALUE {
    let arr = RArray::new();

    // Add elements one by one
    arr.push(10i64);
    arr.push(20i64);
    arr.push(30i64);
    arr.push(40i64);
    arr.push(50i64);

    assert_eq!(arr.len(), 5);

    arr.into_value().as_raw()
}

/// Example 2: Iterating over an array with each()
///
/// Shows closure-based iteration to sum array elements.
#[no_mangle]
pub extern "C" fn iterate_array_sum() -> rb_sys::VALUE {
    let arr = RArray::from_slice(&[1i64, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    // Sum all elements using each()
    let mut sum = 0i64;
    arr.each(|val| {
        let n = i64::try_convert(val)?;
        sum += n;
        Ok(())
    })
    .unwrap();

    assert_eq!(sum, 55); // 1+2+...+10 = 55

    // Return the sum as a Ruby value
    sum.into_value().as_raw()
}

/// Example 3: Filtering array elements during iteration
///
/// Demonstrates building a new array with only elements matching a condition.
#[no_mangle]
pub extern "C" fn filter_array_even() -> rb_sys::VALUE {
    let arr = RArray::from_slice(&[1i64, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    // Create a new array with only even numbers
    let evens = RArray::new();
    arr.each(|val| {
        let n = i64::try_convert(val)?;
        if n % 2 == 0 {
            evens.push(n);
        }
        Ok(())
    })
    .unwrap();

    assert_eq!(evens.len(), 5);

    // Verify contents
    let vec: Vec<i64> = evens.to_vec().unwrap();
    assert_eq!(vec, vec![2, 4, 6, 8, 10]);

    evens.into_value().as_raw()
}

/// Example 4: Convert Vec<i64> to RArray
///
/// Shows how to convert a Rust vector to a Ruby array.
#[no_mangle]
pub extern "C" fn vec_to_array() -> rb_sys::VALUE {
    // Create a Rust vector
    let numbers: Vec<i64> = vec![100, 200, 300, 400, 500];

    // Convert to Ruby array using from_slice
    let arr = RArray::from_slice(&numbers);

    assert_eq!(arr.len(), 5);

    // Verify first and last elements
    let first = i64::try_convert(arr.entry(0)).unwrap();
    let last = i64::try_convert(arr.entry(-1)).unwrap();
    assert_eq!(first, 100);
    assert_eq!(last, 500);

    arr.into_value().as_raw()
}

/// Example 5: Convert RArray to Vec<i64>
///
/// Shows type-safe conversion from Ruby array to Rust vector.
#[no_mangle]
pub extern "C" fn array_to_vec() -> rb_sys::VALUE {
    let arr = RArray::new();
    arr.push(11i64);
    arr.push(22i64);
    arr.push(33i64);
    arr.push(44i64);
    arr.push(55i64);

    // Convert to Rust Vec<i64>
    let vec: Vec<i64> = arr.to_vec().unwrap();

    assert_eq!(vec.len(), 5);
    assert_eq!(vec, vec![11, 22, 33, 44, 55]);

    // Use Rust iterator methods on the converted vector
    let doubled: Vec<i64> = vec.iter().map(|x| x * 2).collect();
    let sum: i64 = doubled.iter().sum();
    assert_eq!(sum, 330); // (11+22+33+44+55) * 2 = 330

    // Return the original array
    arr.into_value().as_raw()
}

/// Example 6: Transform array elements (map-like operation)
///
/// Demonstrates creating a new array with transformed elements.
#[no_mangle]
pub extern "C" fn map_array_double() -> rb_sys::VALUE {
    let arr = RArray::from_slice(&[1i64, 2, 3, 4, 5]);

    // Create a new array with doubled values
    let doubled = RArray::with_capacity(arr.len());
    arr.each(|val| {
        let n = i64::try_convert(val)?;
        doubled.push(n * 2);
        Ok(())
    })
    .unwrap();

    assert_eq!(doubled.len(), 5);

    let vec: Vec<i64> = doubled.to_vec().unwrap();
    assert_eq!(vec, vec![2, 4, 6, 8, 10]);

    doubled.into_value().as_raw()
}

// =============================================================================
// PART 2: Working with Hashes
// =============================================================================

/// Example 7: Building a hash from scratch
///
/// Demonstrates creating an empty hash and populating it with insert().
#[no_mangle]
pub extern "C" fn build_hash() -> rb_sys::VALUE {
    let hash = RHash::new();

    // Add key-value pairs
    hash.insert("name", "Alice");
    hash.insert("language", "Rust");
    hash.insert("year", 2024i64);
    hash.insert("active", true);

    assert_eq!(hash.len(), 4);

    hash.into_value().as_raw()
}

/// Example 8: Iterating over hash entries with each()
///
/// Shows closure-based iteration over key-value pairs.
#[no_mangle]
pub extern "C" fn iterate_hash_entries() -> rb_sys::VALUE {
    let hash = RHash::new();
    hash.insert("a", 10i64);
    hash.insert("b", 20i64);
    hash.insert("c", 30i64);

    // Sum all values
    let mut sum = 0i64;
    hash.each(|_key, val| {
        let n = i64::try_convert(val)?;
        sum += n;
        Ok(())
    })
    .unwrap();

    assert_eq!(sum, 60);

    // Collect all keys
    let mut keys = Vec::new();
    hash.each(|key, _val| {
        let s = RString::try_convert(key)?;
        keys.push(s.to_string()?);
        Ok(())
    })
    .unwrap();

    // Sort for deterministic comparison (hash iteration order not guaranteed)
    keys.sort();
    assert_eq!(keys, vec!["a", "b", "c"]);

    hash.into_value().as_raw()
}

/// Example 9: Convert HashMap<String, i64> to RHash
///
/// Shows how to convert a Rust HashMap to a Ruby Hash.
#[no_mangle]
pub extern "C" fn hashmap_to_rhash() -> rb_sys::VALUE {
    // Create a Rust HashMap
    let mut map = HashMap::new();
    map.insert("width", 1920i64);
    map.insert("height", 1080i64);
    map.insert("fps", 60i64);

    // Convert to Ruby hash
    let hash = RHash::from_hash_map(map);

    assert_eq!(hash.len(), 3);

    // Verify values
    let width = i64::try_convert(hash.get("width").unwrap()).unwrap();
    let height = i64::try_convert(hash.get("height").unwrap()).unwrap();
    let fps = i64::try_convert(hash.get("fps").unwrap()).unwrap();

    assert_eq!(width, 1920);
    assert_eq!(height, 1080);
    assert_eq!(fps, 60);

    hash.into_value().as_raw()
}

/// Example 10: Convert RHash to HashMap<String, i64>
///
/// Shows type-safe conversion from Ruby Hash to Rust HashMap.
#[no_mangle]
pub extern "C" fn rhash_to_hashmap() -> rb_sys::VALUE {
    let hash = RHash::new();
    hash.insert("red", 255i64);
    hash.insert("green", 128i64);
    hash.insert("blue", 64i64);

    // Convert to Rust HashMap
    let map: HashMap<String, i64> = hash.to_hash_map().unwrap();

    assert_eq!(map.len(), 3);
    assert_eq!(map.get("red"), Some(&255));
    assert_eq!(map.get("green"), Some(&128));
    assert_eq!(map.get("blue"), Some(&64));

    // Use Rust HashMap methods
    let total: i64 = map.values().sum();
    assert_eq!(total, 447); // 255 + 128 + 64

    hash.into_value().as_raw()
}

/// Example 11: Filter hash entries during iteration
///
/// Demonstrates building a new hash with only entries matching a condition.
#[no_mangle]
pub extern "C" fn filter_hash_by_value() -> rb_sys::VALUE {
    let hash = RHash::new();
    hash.insert("small", 5i64);
    hash.insert("medium", 15i64);
    hash.insert("large", 25i64);
    hash.insert("tiny", 1i64);

    // Create a new hash with values > 10
    let filtered = RHash::new();
    hash.each(|key, val| {
        let n = i64::try_convert(val.clone())?;
        if n > 10 {
            filtered.insert(key, val);
        }
        Ok(())
    })
    .unwrap();

    assert_eq!(filtered.len(), 2);
    assert!(filtered.get("medium").is_some());
    assert!(filtered.get("large").is_some());
    assert!(filtered.get("small").is_none());
    assert!(filtered.get("tiny").is_none());

    filtered.into_value().as_raw()
}

// =============================================================================
// PART 3: Combining Arrays and Hashes
// =============================================================================

/// Example 12: Array of hashes
///
/// Demonstrates storing hashes in an array (common data structure pattern).
#[no_mangle]
pub extern "C" fn array_of_hashes() -> rb_sys::VALUE {
    let users = RArray::new();

    // Create user records as hashes
    let user1 = RHash::new();
    user1.insert("name", "Alice");
    user1.insert("age", 30i64);

    let user2 = RHash::new();
    user2.insert("name", "Bob");
    user2.insert("age", 25i64);

    let user3 = RHash::new();
    user3.insert("name", "Charlie");
    user3.insert("age", 35i64);

    users.push(user1);
    users.push(user2);
    users.push(user3);

    assert_eq!(users.len(), 3);

    // Access nested data
    let first_user = RHash::try_convert(users.entry(0)).unwrap();
    let name_val = first_user.get("name").unwrap();
    let name = RString::try_convert(name_val).unwrap();
    assert_eq!(name.to_string().unwrap(), "Alice");

    users.into_value().as_raw()
}

/// Example 13: Hash with array values
///
/// Demonstrates storing arrays as hash values.
#[no_mangle]
pub extern "C" fn hash_with_array_values() -> rb_sys::VALUE {
    let data = RHash::new();

    // Create arrays for each category
    let fruits = RArray::from_slice(&["apple", "banana", "cherry"]);
    let vegetables = RArray::from_slice(&["carrot", "broccoli", "spinach"]);
    let grains = RArray::from_slice(&["rice", "wheat"]);

    data.insert("fruits", fruits);
    data.insert("vegetables", vegetables);
    data.insert("grains", grains);

    assert_eq!(data.len(), 3);

    // Access nested array
    let fruits_val = data.get("fruits").unwrap();
    let fruits_arr = RArray::try_convert(fruits_val).unwrap();
    assert_eq!(fruits_arr.len(), 3);

    let first_fruit = RString::try_convert(fruits_arr.entry(0)).unwrap();
    assert_eq!(first_fruit.to_string().unwrap(), "apple");

    data.into_value().as_raw()
}

/// Example 14: Grouping array elements into a hash
///
/// Demonstrates a common pattern: grouping items by a computed key.
#[no_mangle]
pub extern "C" fn group_by_length() -> rb_sys::VALUE {
    let words = RArray::from_slice(&["a", "to", "the", "be", "cat", "word", "hello"]);

    // Group words by their length
    let grouped = RHash::new();

    words
        .each(|val| {
            let s = RString::try_convert(val.clone())?;
            let word = s.to_string()?;
            let len = word.len() as i64;

            // Get or create the array for this length
            let group = match grouped.get(len) {
                Some(existing) => RArray::try_convert(existing)?,
                None => {
                    // Use RArray::default() to get an RArray directly
                    // (default() internally creates and unwraps a PinGuard)
                    let new_group = RArray::default();
                    grouped.insert(len, new_group.clone());
                    new_group
                }
            };

            group.push(val);
            Ok(())
        })
        .unwrap();

    // Verify grouping
    let len_1 = RArray::try_convert(grouped.get(1i64).unwrap()).unwrap();
    let len_2 = RArray::try_convert(grouped.get(2i64).unwrap()).unwrap();
    let len_3 = RArray::try_convert(grouped.get(3i64).unwrap()).unwrap();

    assert_eq!(len_1.len(), 1); // "a"
    assert_eq!(len_2.len(), 2); // "to", "be"
    assert_eq!(len_3.len(), 2); // "the", "cat"

    grouped.into_value().as_raw()
}

/// Example 15: Flattening a hash with array values into a single array
///
/// Demonstrates combining nested arrays from hash values.
#[no_mangle]
pub extern "C" fn flatten_hash_arrays() -> rb_sys::VALUE {
    let data = RHash::new();
    data.insert("a", RArray::from_slice(&[1i64, 2]));
    data.insert("b", RArray::from_slice(&[3i64, 4, 5]));
    data.insert("c", RArray::from_slice(&[6i64]));

    // Flatten all arrays into one
    let flattened = RArray::new();
    data.each(|_key, val| {
        let arr = RArray::try_convert(val)?;
        arr.each(|item| {
            flattened.push(item);
            Ok(())
        })?;
        Ok(())
    })
    .unwrap();

    assert_eq!(flattened.len(), 6);

    // Convert to Vec and sort to verify all elements (hash order not guaranteed)
    let mut vec: Vec<i64> = flattened.to_vec().unwrap();
    vec.sort();
    assert_eq!(vec, vec![1, 2, 3, 4, 5, 6]);

    flattened.into_value().as_raw()
}

// =============================================================================
// PART 4: Round-trip Conversions
// =============================================================================

/// Example 16: Round-trip Vec<i64> conversion
///
/// Demonstrates safe round-trip conversion: Vec -> RArray -> Vec
#[no_mangle]
pub extern "C" fn roundtrip_vec() -> rb_sys::VALUE {
    // Start with Rust Vec
    let original = vec![10i64, 20, 30, 40, 50];

    // Convert to Ruby array
    let arr = RArray::from_slice(&original);

    // Convert back to Rust Vec
    let roundtrip: Vec<i64> = arr.to_vec().unwrap();

    // Should be identical
    assert_eq!(original, roundtrip);

    arr.into_value().as_raw()
}

/// Example 17: Round-trip HashMap conversion
///
/// Demonstrates safe round-trip conversion: HashMap -> RHash -> HashMap
#[no_mangle]
pub extern "C" fn roundtrip_hashmap() -> rb_sys::VALUE {
    // Start with Rust HashMap
    let mut original = HashMap::new();
    original.insert("one", 1i64);
    original.insert("two", 2i64);
    original.insert("three", 3i64);

    // Convert to Ruby hash
    let hash = RHash::from_hash_map(original.clone());

    // Convert back to Rust HashMap
    let roundtrip: HashMap<String, i64> = hash.to_hash_map().unwrap();

    // Should be identical (values)
    assert_eq!(roundtrip.len(), original.len());
    assert_eq!(roundtrip.get("one"), original.get("one"));
    assert_eq!(roundtrip.get("two"), original.get("two"));
    assert_eq!(roundtrip.get("three"), original.get("three"));

    hash.into_value().as_raw()
}

// =============================================================================
// Extension Initialization
// =============================================================================

/// Initialize the extension
#[no_mangle]
pub extern "C" fn Init_collections() {
    // Note: Full method definition requires Phase 3
    // For now, this is just a placeholder that Ruby will call when loading the extension
}

// =============================================================================
// Rust-only Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_checks() {
        // Verify RArray and RHash are Clone (not Copy - they're heap-allocated)
        fn assert_clone<T: Clone>() {}
        assert_clone::<RArray>();
        assert_clone::<RHash>();
    }

    #[test]
    fn test_type_sizes() {
        // Verify types are transparent wrappers around Value
        assert_eq!(std::mem::size_of::<RArray>(), std::mem::size_of::<Value>());
        assert_eq!(std::mem::size_of::<RHash>(), std::mem::size_of::<Value>());
    }
}
