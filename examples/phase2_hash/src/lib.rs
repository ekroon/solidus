//! Phase 2 Stage 6: Hash Type Example
//!
//! This example demonstrates Ruby's Hash type with key-value operations.
//! Hashes are heap-allocated objects that require GC protection.
//!
//! This shows Stage 6 implementation: RHash type with insertion, lookup,
//! deletion, iteration, and conversions to/from Rust HashMap.

use solidus::prelude::*;
use std::collections::HashMap;

/// Example 1: Creating empty hashes
///
/// Demonstrates basic hash creation and property checking.
#[no_mangle]
pub extern "C" fn example_hash_new() -> rb_sys::VALUE {
    // Create a new empty hash
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };

    // Check properties
    assert_eq!(hash.len(), 0);
    assert!(hash.is_empty());

    hash.into_value().as_raw()
}

/// Example 2: Inserting key-value pairs
///
/// Shows how to insert elements into a hash.
#[no_mangle]
pub extern "C" fn example_hash_insert() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };

    // Insert string key with integer value
    hash.insert("name", "Alice");
    hash.insert("age", 30i64);
    hash.insert("active", true);

    // Check the size
    assert_eq!(hash.len(), 3);
    assert!(!hash.is_empty());

    hash.into_value().as_raw()
}

/// Example 3: Getting values by key
///
/// Demonstrates retrieving values from a hash.
#[no_mangle]
pub extern "C" fn example_hash_get() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };

    // Insert some data
    hash.insert("x", 100i64);
    hash.insert("y", 200i64);

    // Get values by key
    if let Some(val) = hash.get("x") {
        let x = i64::try_convert(val).unwrap();
        assert_eq!(x, 100);
    }

    if let Some(val) = hash.get("y") {
        let y = i64::try_convert(val).unwrap();
        assert_eq!(y, 200);
    }

    // Missing key returns None
    assert!(hash.get("z").is_none());

    hash.into_value().as_raw()
}

/// Example 4: Updating existing keys
///
/// Shows that insert updates the value if the key exists.
#[no_mangle]
pub extern "C" fn example_hash_update() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };

    // Insert initial value
    hash.insert("counter", 1i64);
    assert_eq!(hash.len(), 1);

    // Update the same key
    hash.insert("counter", 2i64);
    assert_eq!(hash.len(), 1); // Still just one key

    // Verify the value was updated
    let val = hash.get("counter").unwrap();
    let counter = i64::try_convert(val).unwrap();
    assert_eq!(counter, 2);

    hash.into_value().as_raw()
}

/// Example 5: Deleting keys
///
/// Demonstrates removing key-value pairs from a hash.
#[no_mangle]
pub extern "C" fn example_hash_delete() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };

    // Insert data
    hash.insert("keep", 1i64);
    hash.insert("remove", 2i64);
    assert_eq!(hash.len(), 2);

    // Delete a key and get its value
    if let Some(val) = hash.delete("remove") {
        let removed = i64::try_convert(val).unwrap();
        assert_eq!(removed, 2);
    }

    // Hash now has one element
    assert_eq!(hash.len(), 1);
    assert!(hash.get("remove").is_none());
    assert!(hash.get("keep").is_some());

    // Deleting non-existent key returns None
    assert!(hash.delete("missing").is_none());

    hash.into_value().as_raw()
}

/// Example 6: Iterating over hash entries
///
/// Shows how to iterate over all key-value pairs.
#[no_mangle]
pub extern "C" fn example_hash_iteration() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };

    // Insert some data
    hash.insert("a", 1i64);
    hash.insert("b", 2i64);
    hash.insert("c", 3i64);

    // Iterate and sum values
    let mut sum = 0i64;
    hash.each(|_key, val| {
        let n = i64::try_convert(val)?;
        sum += n;
        Ok(())
    })
    .unwrap();

    assert_eq!(sum, 6);

    hash.into_value().as_raw()
}

/// Example 7: Using Symbol keys
///
/// Demonstrates hashes with symbol keys (common in Ruby).
#[no_mangle]
pub extern "C" fn example_hash_symbol_keys() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };

    // Use symbols as keys
    let name_key = Symbol::new("name");
    let age_key = Symbol::new("age");

    hash.insert(name_key.clone(), "Bob");
    hash.insert(age_key.clone(), 25i64);

    // Retrieve using symbols
    if let Some(val) = hash.get(name_key) {
        let name = RString::try_convert(val).unwrap();
        assert_eq!(name.to_string().unwrap(), "Bob");
    }

    if let Some(val) = hash.get(age_key) {
        let age = i64::try_convert(val).unwrap();
        assert_eq!(age, 25);
    }

    hash.into_value().as_raw()
}

/// Example 8: Using Integer keys
///
/// Shows that hashes can use integers as keys.
#[no_mangle]
pub extern "C" fn example_hash_integer_keys() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };

    // Use integers as keys
    hash.insert(1i64, "first");
    hash.insert(2i64, "second");
    hash.insert(3i64, "third");

    assert_eq!(hash.len(), 3);

    // Retrieve by integer key
    if let Some(val) = hash.get(2i64) {
        let s = RString::try_convert(val).unwrap();
        assert_eq!(s.to_string().unwrap(), "second");
    }

    hash.into_value().as_raw()
}

/// Example 9: Mixed key types
///
/// Demonstrates that hashes can have different key types.
#[no_mangle]
pub extern "C" fn example_hash_mixed_keys() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };

    // Mix different key types
    hash.insert("string_key", 1i64);
    hash.insert(Symbol::new("symbol_key"), 2i64);
    hash.insert(100i64, 3i64);

    assert_eq!(hash.len(), 3);

    // Each key type is distinct
    assert!(hash.get("string_key").is_some());
    assert!(hash.get(Symbol::new("symbol_key")).is_some());
    assert!(hash.get(100i64).is_some());

    hash.into_value().as_raw()
}

/// Example 10: Converting from Rust HashMap
///
/// Shows how to create Ruby hashes from Rust HashMaps.
#[no_mangle]
pub extern "C" fn example_hash_from_hashmap() -> rb_sys::VALUE {
    // Create a Rust HashMap
    let mut map = HashMap::new();
    map.insert("red", 255i64);
    map.insert("green", 128i64);
    map.insert("blue", 0i64);

    // Convert to Ruby hash
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::from_hash_map(map) };

    assert_eq!(hash.len(), 3);

    // Verify the values
    let red = i64::try_convert(hash.get("red").unwrap()).unwrap();
    assert_eq!(red, 255);

    hash.into_value().as_raw()
}

/// Example 11: Converting to Rust HashMap
///
/// Demonstrates converting Ruby hashes to Rust HashMaps.
#[no_mangle]
pub extern "C" fn example_hash_to_hashmap() -> rb_sys::VALUE {
    // Create Ruby hash
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };
    hash.insert("width", 1920i64);
    hash.insert("height", 1080i64);

    // Convert to Rust HashMap
    let map: HashMap<String, i64> = hash.to_hash_map().unwrap();

    assert_eq!(map.len(), 2);
    assert_eq!(map.get("width"), Some(&1920));
    assert_eq!(map.get("height"), Some(&1080));

    hash.into_value().as_raw()
}

/// Example 12: Nested hashes
///
/// Shows that hashes can contain other hashes as values.
#[no_mangle]
pub extern "C" fn example_hash_nested() -> rb_sys::VALUE {
    // Create inner hash
    // SAFETY: Value is used immediately
    let inner = unsafe { RHash::new() };
    inner.insert("city", "Portland");
    inner.insert("state", "Oregon");

    // Create outer hash
    // SAFETY: Value is used immediately and returned to Ruby
    let outer = unsafe { RHash::new() };
    outer.insert("name", "Alice");
    outer.insert("location", inner);

    assert_eq!(outer.len(), 2);

    // Retrieve nested hash
    if let Some(val) = outer.get("location") {
        let location = RHash::try_convert(val).unwrap();
        assert_eq!(location.len(), 2);

        if let Some(city_val) = location.get("city") {
            let city = RString::try_convert(city_val).unwrap();
            assert_eq!(city.to_string().unwrap(), "Portland");
        }
    }

    outer.into_value().as_raw()
}

/// Example 13: Type-safe hash operations
///
/// Shows compile-time guarantees for hash operations.
fn build_user_hash(name: &str, age: i64, active: bool) -> Result<NewValue<RHash>, Error> {
    // SAFETY: Value is returned as part of a Result, caller handles it
    let hash = unsafe { RHash::new() };
    hash.insert("name", name);
    hash.insert("age", age);
    hash.insert("active", active);
    Ok(NewValue::new(hash))
}

#[no_mangle]
pub extern "C" fn example_hash_type_safe() -> rb_sys::VALUE {
    match build_user_hash("Charlie", 35, true) {
        Ok(hash) => {
            assert_eq!(hash.len(), 3);

            // Verify types
            let name = RString::try_convert(hash.get("name").unwrap()).unwrap();
            assert_eq!(name.to_string().unwrap(), "Charlie");

            let age = i64::try_convert(hash.get("age").unwrap()).unwrap();
            assert_eq!(age, 35);

            let active = bool::try_convert(hash.get("active").unwrap()).unwrap();
            assert!(active);

            hash.into_value().as_raw()
        }
        Err(_) => Qnil::new().into_value().as_raw(),
    }
}

/// Example 14: Round-trip HashMap conversion
///
/// Demonstrates safe round-trip conversion between Rust and Ruby.
#[no_mangle]
pub extern "C" fn example_hash_roundtrip() -> rb_sys::VALUE {
    // Start with a Rust HashMap
    let mut original = HashMap::new();
    original.insert("apple", 5i64);
    original.insert("banana", 3i64);
    original.insert("orange", 7i64);

    // Convert to Ruby hash
    // SAFETY: Value is used immediately and returned to Ruby
    let ruby_hash = unsafe { RHash::from_hash_map(original.clone()) };

    // Convert back to Rust HashMap
    let roundtrip: HashMap<String, i64> = ruby_hash.to_hash_map().unwrap();

    // Should be identical
    assert_eq!(roundtrip.len(), original.len());
    assert_eq!(roundtrip.get("apple"), original.get("apple"));
    assert_eq!(roundtrip.get("banana"), original.get("banana"));
    assert_eq!(roundtrip.get("orange"), original.get("orange"));

    ruby_hash.into_value().as_raw()
}

/// Example 15: Collecting iteration results
///
/// Shows how to collect keys and values during iteration.
#[no_mangle]
pub extern "C" fn example_hash_collect_keys() -> rb_sys::VALUE {
    // SAFETY: Value is used immediately and returned to Ruby
    let hash = unsafe { RHash::new() };
    hash.insert("alpha", 1i64);
    hash.insert("beta", 2i64);
    hash.insert("gamma", 3i64);

    // Collect all keys
    let mut keys = Vec::new();
    hash.each(|key, _val| {
        let s = RString::try_convert(key)?;
        keys.push(s.to_string()?);
        Ok(())
    })
    .unwrap();

    // Sort for deterministic comparison (hash order is not guaranteed)
    keys.sort();
    assert_eq!(keys, vec!["alpha", "beta", "gamma"]);

    hash.into_value().as_raw()
}

/// Initialize the extension
#[no_mangle]
pub extern "C" fn Init_phase2_hash() {
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

        // Verify RHash is Copy (it's just a VALUE wrapper)
        fn assert_copy<T: Copy>() {}
        assert_copy::<RHash>();
    }

    #[test]
    fn test_hash_types() {
        // Verify type properties without calling Ruby API

        // RHash should be transparent wrapper
        assert_eq!(std::mem::size_of::<RHash>(), std::mem::size_of::<Value>());
    }
}
