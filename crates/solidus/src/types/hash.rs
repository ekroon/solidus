//! Ruby Hash type.

use std::collections::HashMap;

use crate::convert::{IntoValue, TryConvert};
use crate::error::Error;
use crate::value::{BoxValue, ReprValue, Value};

/// Ruby Hash (heap allocated).
///
/// Ruby hashes are dynamic key-value stores that can contain any Ruby values
/// as keys and values. These are heap-allocated objects that require GC protection.
///
/// Values should be created via `Context::new_hash()` for stack-pinned hashes
/// within methods, or `RHash::new_boxed()` for heap-allocated hashes.
///
/// # Example
///
/// ```no_run
/// use solidus::types::RHash;
///
/// // For heap storage, use new_boxed()
/// let mut hash = RHash::new_boxed();
/// hash.insert("key", "value");
/// assert_eq!(hash.len(), 1);
///
/// // For stack-pinned hashes in methods, use Context::new_hash()
/// ```
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct RHash(Value);

impl RHash {
    /// Create a new empty Ruby hash.
    ///
    /// # Safety
    ///
    /// The caller must ensure the returned value is:
    /// - Pinned on the stack with `pin_on_stack!`, OR
    /// - Immediately boxed with `.into_box()`, OR
    /// - Immediately returned to Ruby
    ///
    /// Failing to do so may result in the value being collected by Ruby's GC.
    ///
    /// For safe alternatives, use:
    /// - `RHash::new_boxed()` for heap storage
    /// - `Context::new_hash()` for stack-pinned hashes in methods
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RHash;
    ///
    /// // SAFETY: Value is immediately returned to Ruby
    /// let hash = unsafe { RHash::new() };
    /// ```
    pub unsafe fn new() -> Self {
        // SAFETY: rb_hash_new creates a new Ruby hash
        let val = unsafe { rb_sys::rb_hash_new() };
        // SAFETY: rb_hash_new returns a valid VALUE
        RHash(unsafe { Value::from_raw(val) })
    }

    /// Internal: Create a new empty Ruby hash.
    ///
    /// Users should use `Context::new_hash()` or `RHash::new_boxed()` instead.
    #[doc(hidden)]
    pub(crate) unsafe fn new_internal() -> Self {
        // SAFETY: rb_hash_new creates a new Ruby hash
        let val = unsafe { rb_sys::rb_hash_new() };
        // SAFETY: rb_hash_new returns a valid VALUE
        RHash(unsafe { Value::from_raw(val) })
    }

    /// Create a new empty Ruby hash, boxed for heap storage.
    ///
    /// This is safe because the value is immediately registered with Ruby's GC.
    /// Use `Context::new_hash()` for stack-pinned hashes within methods.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RHash;
    ///
    /// let boxed = RHash::new_boxed();
    /// assert_eq!(boxed.len(), 0);
    /// assert!(boxed.is_empty());
    /// ```
    pub fn new_boxed() -> BoxValue<Self> {
        // SAFETY: We immediately box and register with GC
        unsafe { BoxValue::new(Self::new_internal()) }
    }

    /// Get the number of key-value pairs in the hash.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RHash;
    ///
    /// let mut hash = RHash::new_boxed();
    /// hash.insert("a", 1);
    /// hash.insert("b", 2);
    /// assert_eq!(hash.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        // SAFETY: self.0 is a valid Ruby hash VALUE
        unsafe { rb_sys::rb_hash_size_num(self.0.as_raw()) as usize }
    }

    /// Check if the hash is empty.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RHash;
    ///
    /// let mut hash = RHash::new_boxed();
    /// assert!(hash.is_empty());
    ///
    /// hash.insert("key", "value");
    /// assert!(!hash.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the value associated with a key.
    ///
    /// Returns `None` if the key is not found.
    ///
    /// # Note
    ///
    /// This method returns `None` for both missing keys and keys with `nil` values.
    /// If you need to distinguish between these cases, use Ruby's `Hash#key?` method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use solidus::types::RHash;
    /// use solidus::convert::TryConvert;
    ///
    /// let mut hash = RHash::new_boxed();
    /// hash.insert("key", 42i64);
    ///
    /// let val = hash.get("key").unwrap();
    /// assert_eq!(i64::try_convert(val)?, 42);
    ///
    /// assert!(hash.get("missing").is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get<K: IntoValue>(&self, key: K) -> Option<Value> {
        let key_val = key.into_value();
        // SAFETY: self.0 is a valid Ruby hash, key_val is a valid VALUE
        let val = unsafe { rb_sys::rb_hash_lookup(self.0.as_raw(), key_val.as_raw()) };
        // SAFETY: rb_hash_lookup returns a valid VALUE or Qnil
        let value = unsafe { Value::from_raw(val) };

        // rb_hash_lookup returns Qnil if the key doesn't exist
        // Note: This means we can't distinguish between a nil value and missing key
        if value.is_nil() { None } else { Some(value) }
    }

    /// Insert or update a key-value pair.
    ///
    /// This modifies the hash in place. If the key already exists, its value
    /// is updated. Otherwise, a new key-value pair is added.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RHash;
    ///
    /// let mut hash = RHash::new_boxed();
    /// hash.insert("name", "Alice");
    /// hash.insert("age", 30i64);
    /// assert_eq!(hash.len(), 2);
    ///
    /// hash.insert("age", 31i64); // Update existing key
    /// assert_eq!(hash.len(), 2);
    /// ```
    pub fn insert<K: IntoValue, V: IntoValue>(&self, key: K, value: V) {
        let key_val = key.into_value();
        let val = value.into_value();
        // SAFETY: self.0 is a valid Ruby hash, both VALUES are valid
        unsafe {
            rb_sys::rb_hash_aset(self.0.as_raw(), key_val.as_raw(), val.as_raw());
        }
    }

    /// Delete a key-value pair and return the value.
    ///
    /// Returns `None` if the key doesn't exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use solidus::types::RHash;
    /// use solidus::convert::TryConvert;
    ///
    /// let mut hash = RHash::new_boxed();
    /// hash.insert("key", 42i64);
    ///
    /// let val = hash.delete("key").unwrap();
    /// assert_eq!(i64::try_convert(val)?, 42);
    /// assert_eq!(hash.len(), 0);
    ///
    /// assert!(hash.delete("key").is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn delete<K: IntoValue>(&self, key: K) -> Option<Value> {
        let key_val = key.into_value();
        // SAFETY: self.0 is a valid Ruby hash, key_val is a valid VALUE
        let val = unsafe { rb_sys::rb_hash_delete(self.0.as_raw(), key_val.as_raw()) };
        // SAFETY: rb_hash_delete returns a valid VALUE or Qnil
        let value = unsafe { Value::from_raw(val) };

        // rb_hash_delete returns Qnil if the key doesn't exist
        if value.is_nil() { None } else { Some(value) }
    }

    /// Iterate over the hash key-value pairs.
    ///
    /// The closure is called for each (key, value) pair in the hash. If the
    /// closure returns an error, iteration stops and the error is returned.
    ///
    /// # Why not Iterator?
    ///
    /// We don't implement Rust's `Iterator` trait because it would be unsafe.
    /// Between iterator calls, Ruby code could run (if the closure calls back
    /// into Ruby), potentially triggering GC which could modify or move the hash.
    /// By using a closure, we maintain control over when Ruby code can execute.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use solidus::types::RHash;
    /// use solidus::convert::TryConvert;
    ///
    /// let mut hash = RHash::new_boxed();
    /// hash.insert("a", 1i64);
    /// hash.insert("b", 2i64);
    ///
    /// let mut sum = 0i64;
    /// hash.each(|key, val| {
    ///     let n = i64::try_convert(val)?;
    ///     sum += n;
    ///     Ok(())
    /// })?;
    /// assert_eq!(sum, 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn each<F>(&self, mut f: F) -> Result<(), Error>
    where
        F: FnMut(Value, Value) -> Result<(), Error>,
    {
        use crate::types::RArray;

        // Create a temporary array to collect key-value pairs
        // This is safer than using rb_hash_foreach which requires complex FFI callbacks
        // SAFETY: We immediately use the array and don't let it escape
        let pairs = unsafe { RArray::new() };

        // Use rb_hash_foreach to collect all pairs
        unsafe extern "C" fn collect_pair(
            key: rb_sys::VALUE,
            val: rb_sys::VALUE,
            arg: rb_sys::VALUE,
        ) -> i32 {
            // SAFETY: We're in an unsafe block within an extern "C" function called from Ruby
            unsafe {
                let pairs_array = Value::from_raw(arg);
                let pair = rb_sys::rb_ary_new();
                rb_sys::rb_ary_push(pair, key);
                rb_sys::rb_ary_push(pair, val);
                rb_sys::rb_ary_push(pairs_array.as_raw(), pair);
            }
            0 // ST_CONTINUE
        }

        // SAFETY: self.0 is a valid hash, pairs is a valid array, collect_pair follows the callback contract
        unsafe {
            rb_sys::rb_hash_foreach(
                self.0.as_raw(),
                Some(collect_pair),
                pairs.as_value().as_raw(),
            );
        }

        // Now iterate over the collected pairs
        let pairs_arr = RArray::try_convert(pairs.as_value())?;
        for i in 0..pairs_arr.len() {
            let pair_val = pairs_arr.entry(i as isize);
            let pair = RArray::try_convert(pair_val)?;
            let key = pair.entry(0);
            let val = pair.entry(1);
            f(key, val)?;
        }

        Ok(())
    }

    /// Convert this hash to a Rust HashMap.
    ///
    /// Both keys and values are converted using `TryConvert`. If any element
    /// fails to convert, an error is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use solidus::types::RHash;
    /// use std::collections::HashMap;
    ///
    /// let mut hash = RHash::new_boxed();
    /// hash.insert("a", 1i64);
    /// hash.insert("b", 2i64);
    ///
    /// let map: HashMap<String, i64> = hash.to_hash_map()?;
    /// assert_eq!(map.get("a"), Some(&1));
    /// assert_eq!(map.get("b"), Some(&2));
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_hash_map<K, V>(&self) -> Result<HashMap<K, V>, Error>
    where
        K: TryConvert + Eq + std::hash::Hash,
        V: TryConvert,
    {
        let mut map = HashMap::with_capacity(self.len());

        self.each(|key, val| {
            let k = K::try_convert(key)?;
            let v = V::try_convert(val)?;
            map.insert(k, v);
            Ok(())
        })?;

        Ok(map)
    }

    /// Create a Ruby hash from a Rust HashMap.
    ///
    /// # Safety
    ///
    /// The caller must ensure the returned value is:
    /// - Pinned on the stack with `pin_on_stack!`, OR
    /// - Immediately boxed with `.into_box()`, OR
    /// - Immediately returned to Ruby
    ///
    /// For a safe alternative, use `RHash::from_hash_map_boxed()`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RHash;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("a", 1i64);
    /// map.insert("b", 2i64);
    ///
    /// // SAFETY: Value is immediately returned to Ruby
    /// let hash = unsafe { RHash::from_hash_map(map) };
    /// ```
    pub unsafe fn from_hash_map<K, V>(map: HashMap<K, V>) -> Self
    where
        K: IntoValue,
        V: IntoValue,
    {
        // SAFETY: Caller ensures the returned value is properly handled
        let hash = unsafe { Self::new() };
        for (k, v) in map {
            hash.insert(k, v);
        }
        hash
    }

    /// Internal: Create a Ruby hash from a Rust HashMap.
    #[doc(hidden)]
    pub(crate) unsafe fn from_hash_map_internal<K, V>(map: HashMap<K, V>) -> Self
    where
        K: IntoValue,
        V: IntoValue,
    {
        // SAFETY: Caller ensures the returned value is properly handled
        let hash = unsafe { Self::new_internal() };
        for (k, v) in map {
            hash.insert(k, v);
        }
        hash
    }

    /// Create a Ruby hash from a Rust HashMap, boxed for heap storage.
    ///
    /// This is safe because the value is immediately registered with Ruby's GC.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use solidus::types::RHash;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("a", 1i64);
    /// map.insert("b", 2i64);
    ///
    /// let boxed = RHash::from_hash_map_boxed(map);
    /// assert_eq!(boxed.len(), 2);
    /// ```
    pub fn from_hash_map_boxed<K, V>(map: HashMap<K, V>) -> BoxValue<Self>
    where
        K: IntoValue,
        V: IntoValue,
    {
        // SAFETY: We immediately box and register with GC
        unsafe { BoxValue::new(Self::from_hash_map_internal(map)) }
    }
}

impl ReprValue for RHash {
    #[inline]
    fn as_value(&self) -> Value {
        self.0.clone()
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        RHash(val)
    }
}

impl TryConvert for RHash {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if val.rb_type() == crate::value::ValueType::Hash {
            // SAFETY: We've verified it's a Hash
            Ok(unsafe { RHash::from_value_unchecked(val) })
        } else {
            Err(Error::type_error("expected Hash"))
        }
    }
}

impl IntoValue for RHash {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

// Conversions for Rust HashMap

impl<K, V> TryConvert for HashMap<K, V>
where
    K: TryConvert + Eq + std::hash::Hash,
    V: TryConvert,
{
    fn try_convert(val: Value) -> Result<Self, Error> {
        let hash = RHash::try_convert(val)?;
        hash.to_hash_map()
    }
}

impl<K, V> IntoValue for HashMap<K, V>
where
    K: IntoValue,
    V: IntoValue,
{
    fn into_value(self) -> Value {
        // Use the boxed version for safety
        RHash::from_hash_map_boxed(self).as_value()
    }
}

#[cfg(all(test, any(feature = "embed", feature = "link-ruby")))]
mod tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;

    #[ruby_test]
    fn test_rhash_new_boxed() {
        let hash = RHash::new_boxed();
        assert_eq!(hash.len(), 0);
        assert!(hash.is_empty());
    }

    #[ruby_test]
    fn test_rhash_insert_and_get() {
        let hash = RHash::new_boxed();
        hash.insert("key", 42i64);

        assert_eq!(hash.len(), 1);
        assert!(!hash.is_empty());

        let val = hash.get("key").unwrap();
        assert_eq!(i64::try_convert(val).unwrap(), 42);
    }

    #[ruby_test]
    fn test_rhash_get_missing() {
        let hash = RHash::new_boxed();
        assert!(hash.get("missing").is_none());
    }

    #[ruby_test]
    fn test_rhash_insert_update() {
        let hash = RHash::new_boxed();
        hash.insert("key", 1i64);
        assert_eq!(hash.len(), 1);

        hash.insert("key", 2i64);
        assert_eq!(hash.len(), 1); // Should still be 1

        let val = hash.get("key").unwrap();
        assert_eq!(i64::try_convert(val).unwrap(), 2);
    }

    #[ruby_test]
    fn test_rhash_multiple_keys() {
        let hash = RHash::new_boxed();
        hash.insert("a", 1i64);
        hash.insert("b", 2i64);
        hash.insert("c", 3i64);

        assert_eq!(hash.len(), 3);

        assert_eq!(i64::try_convert(hash.get("a").unwrap()).unwrap(), 1);
        assert_eq!(i64::try_convert(hash.get("b").unwrap()).unwrap(), 2);
        assert_eq!(i64::try_convert(hash.get("c").unwrap()).unwrap(), 3);
    }

    #[ruby_test]
    fn test_rhash_delete() {
        let hash = RHash::new_boxed();
        hash.insert("key", 42i64);

        let val = hash.delete("key").unwrap();
        assert_eq!(i64::try_convert(val).unwrap(), 42);
        assert_eq!(hash.len(), 0);
    }

    #[ruby_test]
    fn test_rhash_delete_missing() {
        let hash = RHash::new_boxed();
        assert!(hash.delete("missing").is_none());
    }

    #[ruby_test]
    fn test_rhash_delete_twice() {
        let hash = RHash::new_boxed();
        hash.insert("key", 42i64);

        hash.delete("key");
        assert!(hash.delete("key").is_none());
    }

    #[ruby_test]
    fn test_rhash_each() {
        let hash = RHash::new_boxed();
        hash.insert("a", 1i64);
        hash.insert("b", 2i64);
        hash.insert("c", 3i64);

        let mut sum = 0i64;
        hash.each(|_key, val| {
            let n = i64::try_convert(val)?;
            sum += n;
            Ok(())
        })
        .unwrap();

        assert_eq!(sum, 6);
    }

    #[ruby_test]
    fn test_rhash_each_empty() {
        let hash = RHash::new_boxed();
        let mut count = 0;
        hash.each(|_, _| {
            count += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(count, 0);
    }

    #[ruby_test]
    fn test_rhash_each_with_keys() {
        use crate::types::RString;

        let hash = RHash::new_boxed();
        hash.insert("a", 1i64);
        hash.insert("b", 2i64);

        let mut keys = Vec::new();
        hash.each(|key, _val| {
            let s = RString::try_convert(key)?;
            keys.push(s.to_string()?);
            Ok(())
        })
        .unwrap();

        // Sort because hash iteration order is not guaranteed
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[ruby_test]
    fn test_rhash_each_error() {
        let hash = RHash::new_boxed();
        hash.insert("a", 1i64);

        let result = hash.each(|_, _| Err(Error::type_error("test error")));
        assert!(result.is_err());
    }

    #[ruby_test]
    fn test_rhash_try_convert() {
        let hash = RHash::new_boxed();
        hash.insert("key", 42i64);

        let val = hash.as_value();
        let converted = RHash::try_convert(val).unwrap();
        assert_eq!(converted.len(), 1);
    }

    #[ruby_test]
    fn test_rhash_try_convert_wrong_type() {
        let val = 42i64.into_value();
        assert!(RHash::try_convert(val).is_err());
    }

    #[ruby_test]
    fn test_rhash_to_hash_map() {
        let hash = RHash::new_boxed();
        hash.insert("a", 1i64);
        hash.insert("b", 2i64);

        let map: HashMap<String, i64> = hash.to_hash_map().unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&2));
    }

    #[ruby_test]
    fn test_rhash_from_hash_map_boxed() {
        let mut map = HashMap::new();
        map.insert("a", 1i64);
        map.insert("b", 2i64);

        let hash = RHash::from_hash_map_boxed(map);
        assert_eq!(hash.len(), 2);

        assert_eq!(i64::try_convert(hash.get("a").unwrap()).unwrap(), 1);
        assert_eq!(i64::try_convert(hash.get("b").unwrap()).unwrap(), 2);
    }

    #[ruby_test]
    fn test_hash_map_conversion_roundtrip() {
        let mut map = HashMap::new();
        map.insert("x", 10i64);
        map.insert("y", 20i64);

        let val = map.clone().into_value();
        let converted: HashMap<String, i64> = HashMap::try_convert(val).unwrap();

        assert_eq!(converted.len(), 2);
        assert_eq!(converted.get("x"), Some(&10));
        assert_eq!(converted.get("y"), Some(&20));
    }

    #[ruby_test]
    fn test_rhash_mixed_types() {
        use crate::types::{RString, Symbol};

        let hash = RHash::new_boxed();
        hash.insert("string_key", 42i64);
        hash.insert(Symbol::new("symbol_key"), "value");
        hash.insert(123i64, true);

        assert_eq!(hash.len(), 3);

        let val1 = hash.get("string_key").unwrap();
        assert_eq!(i64::try_convert(val1).unwrap(), 42);

        let val2 = hash.get(Symbol::new("symbol_key")).unwrap();
        // Check if it's a symbol value
        if let Ok(sym) = Symbol::try_convert(val2.clone()) {
            // It's a symbol, not a string
            panic!("Got Symbol instead of String: {:?}", sym);
        }
        let s = RString::try_convert(val2).unwrap();
        assert_eq!(s.to_string().unwrap(), "value");

        let val3 = hash.get(123i64).unwrap();
        assert_eq!(bool::try_convert(val3).unwrap(), true);
    }

    #[ruby_test]
    fn test_rhash_nested() {
        let inner = RHash::new_boxed();
        inner.insert("inner_key", 99i64);

        let outer = RHash::new_boxed();
        outer.insert("outer_key", inner.as_value());

        assert_eq!(outer.len(), 1);

        let val = outer.get("outer_key").unwrap();
        let inner_hash = RHash::try_convert(val).unwrap();
        assert_eq!(inner_hash.len(), 1);

        let inner_val = inner_hash.get("inner_key").unwrap();
        assert_eq!(i64::try_convert(inner_val).unwrap(), 99);
    }

    #[ruby_test]
    fn test_rhash_with_integer_keys() {
        let hash = RHash::new_boxed();
        hash.insert(1i64, "one");
        hash.insert(2i64, "two");
        hash.insert(3i64, "three");

        assert_eq!(hash.len(), 3);

        use crate::types::RString;
        let val = hash.get(2i64).unwrap();
        let s = RString::try_convert(val).unwrap();
        assert_eq!(s.to_string().unwrap(), "two");
    }
}
