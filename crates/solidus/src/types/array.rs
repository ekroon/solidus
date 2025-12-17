//! Ruby Array type.

use crate::convert::{IntoValue, TryConvert};
use crate::error::Error;
use crate::value::{PinGuard, ReprValue, Value};

/// Ruby Array (heap allocated).
///
/// Ruby arrays are dynamic, heterogeneous arrays that can contain any Ruby values.
/// These are heap-allocated objects that require GC protection.
///
/// # Example
///
/// ```ignore
/// use solidus::types::RArray;
///
/// let arr = RArray::new();
/// arr.push(42i64);
/// arr.push("hello");
/// assert_eq!(arr.len(), 2);
/// ```
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct RArray(Value);

impl RArray {
    /// Create a new empty Ruby array.
    ///
    /// Returns a `PinGuard<RArray>` that must be pinned on the stack
    /// or boxed on the heap for GC safety.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    /// use solidus::pin_on_stack;
    ///
    /// let guard = RArray::new();
    /// pin_on_stack!(arr = guard);
    /// assert_eq!(arr.get().len(), 0);
    /// assert!(arr.get().is_empty());
    /// ```
    pub fn new() -> PinGuard<Self> {
        // SAFETY: rb_ary_new creates a new Ruby array
        let val = unsafe { rb_sys::rb_ary_new() };
        // SAFETY: rb_ary_new returns a valid VALUE
        PinGuard::new(RArray(unsafe { Value::from_raw(val) }))
    }

    /// Create a new Ruby array with the specified capacity.
    ///
    /// This pre-allocates space for `capacity` elements, which can improve
    /// performance when you know how many elements you'll add.
    ///
    /// Returns a `PinGuard<RArray>` that must be pinned on the stack
    /// or boxed on the heap for GC safety.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    /// use solidus::pin_on_stack;
    ///
    /// let guard = RArray::with_capacity(100);
    /// pin_on_stack!(arr = guard);
    /// assert_eq!(arr.get().len(), 0);
    /// ```
    pub fn with_capacity(capacity: usize) -> PinGuard<Self> {
        // SAFETY: rb_ary_new_capa creates a new Ruby array with the given capacity
        let val = unsafe { rb_sys::rb_ary_new_capa(capacity as _) };
        // SAFETY: rb_ary_new_capa returns a valid VALUE
        PinGuard::new(RArray(unsafe { Value::from_raw(val) }))
    }

    /// Get the number of elements in the array.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    ///
    /// let arr = RArray::new();
    /// arr.push(1);
    /// arr.push(2);
    /// assert_eq!(arr.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        // SAFETY: self.0 is a valid Ruby array VALUE
        unsafe { rb_sys::RARRAY_LEN(self.0.as_raw()) as usize }
    }

    /// Check if the array is empty.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    ///
    /// let arr = RArray::new();
    /// assert!(arr.is_empty());
    ///
    /// arr.push(1);
    /// assert!(!arr.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Push a value onto the end of the array.
    ///
    /// This modifies the array in place.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    ///
    /// let arr = RArray::new();
    /// arr.push(42i64);
    /// arr.push("hello");
    /// assert_eq!(arr.len(), 2);
    /// ```
    pub fn push<T: IntoValue>(&self, value: T) {
        let val = value.into_value();
        // SAFETY: self.0 is a valid Ruby array, val is a valid VALUE
        unsafe {
            rb_sys::rb_ary_push(self.0.as_raw(), val.as_raw());
        }
    }

    /// Remove and return the last element of the array.
    ///
    /// Returns `None` if the array is empty.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    ///
    /// let arr = RArray::new();
    /// arr.push(1);
    /// arr.push(2);
    ///
    /// let val = arr.pop().unwrap();
    /// assert_eq!(arr.len(), 1);
    /// ```
    pub fn pop(&self) -> Option<Value> {
        if self.is_empty() {
            return None;
        }

        // SAFETY: self.0 is a valid Ruby array
        let val = unsafe { rb_sys::rb_ary_pop(self.0.as_raw()) };
        // SAFETY: rb_ary_pop returns a valid VALUE (or nil if empty)
        let value = unsafe { Value::from_raw(val) };

        // rb_ary_pop returns nil if the array was empty
        if value.is_nil() { None } else { Some(value) }
    }

    /// Get the element at the specified index.
    ///
    /// Returns `nil` if the index is out of bounds. Negative indices count
    /// from the end of the array (-1 is the last element).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    ///
    /// let arr = RArray::new();
    /// arr.push(10);
    /// arr.push(20);
    /// arr.push(30);
    ///
    /// let val = arr.entry(1);
    /// let val_neg = arr.entry(-1); // Last element
    /// ```
    pub fn entry(&self, index: isize) -> Value {
        // SAFETY: self.0 is a valid Ruby array, rb_ary_entry handles bounds checking
        let val = unsafe { rb_sys::rb_ary_entry(self.0.as_raw(), index as _) };
        // SAFETY: rb_ary_entry returns a valid VALUE (nil if out of bounds)
        unsafe { Value::from_raw(val) }
    }

    /// Store a value at the specified index.
    ///
    /// If the index is out of bounds, the array will be extended with `nil` values.
    /// Negative indices count from the end of the array (-1 is the last element).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    ///
    /// let arr = RArray::new();
    /// arr.store(0, 42);
    /// arr.store(1, "hello");
    /// arr.store(-1, "world"); // Replaces last element
    /// ```
    pub fn store<T: IntoValue>(&self, index: isize, value: T) {
        let val = value.into_value();
        // SAFETY: self.0 is a valid Ruby array, val is a valid VALUE
        unsafe {
            rb_sys::rb_ary_store(self.0.as_raw(), index as _, val.as_raw());
        }
    }

    /// Iterate over the array elements.
    ///
    /// The closure is called for each element in the array. If the closure
    /// returns an error, iteration stops and the error is returned.
    ///
    /// # Why not Iterator?
    ///
    /// We don't implement Rust's `Iterator` trait because it would be unsafe.
    /// Between iterator calls, Ruby code could run (if the closure calls back
    /// into Ruby), potentially triggering GC which could modify or move the array.
    /// By using a closure, we maintain control over when Ruby code can execute.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    ///
    /// let arr = RArray::new();
    /// arr.push(1);
    /// arr.push(2);
    /// arr.push(3);
    ///
    /// let mut sum = 0i64;
    /// arr.each(|val| {
    ///     let n = i64::try_convert(val)?;
    ///     sum += n;
    ///     Ok(())
    /// })?;
    /// assert_eq!(sum, 6);
    /// ```
    pub fn each<F>(&self, mut f: F) -> Result<(), Error>
    where
        F: FnMut(Value) -> Result<(), Error>,
    {
        let len = self.len();
        for i in 0..len {
            let val = self.entry(i as isize);
            f(val)?;
        }
        Ok(())
    }

    /// Create a Ruby array from a Rust slice.
    ///
    /// Returns a `PinGuard<RArray>` that must be pinned on the stack
    /// or boxed on the heap for GC safety.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    /// use solidus::pin_on_stack;
    ///
    /// let guard = RArray::from_slice(&[1, 2, 3, 4, 5]);
    /// pin_on_stack!(arr = guard);
    /// assert_eq!(arr.get().len(), 5);
    /// ```
    pub fn from_slice<T: IntoValue + Copy>(slice: &[T]) -> PinGuard<Self> {
        let guard = RArray::with_capacity(slice.len());
        // SAFETY: We need to unwrap the guard to use the array, then re-wrap it
        let arr = unsafe { guard.into_inner() };
        for &item in slice {
            arr.push(item);
        }
        PinGuard::new(arr)
    }

    /// Convert this array to a Rust Vec.
    ///
    /// Each element is converted using `TryConvert`. If any element fails
    /// to convert, an error is returned.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solidus::types::RArray;
    ///
    /// let arr = RArray::new();
    /// arr.push(1);
    /// arr.push(2);
    /// arr.push(3);
    ///
    /// let vec: Vec<i64> = arr.to_vec()?;
    /// assert_eq!(vec, vec![1, 2, 3]);
    /// ```
    pub fn to_vec<T: TryConvert>(&self) -> Result<Vec<T>, Error> {
        let len = self.len();
        let mut vec = Vec::with_capacity(len);
        for i in 0..len {
            let val = self.entry(i as isize);
            vec.push(T::try_convert(val)?);
        }
        Ok(vec)
    }
}

impl Default for RArray {
    fn default() -> Self {
        // SAFETY: We unwrap the PinGuard to return Self
        unsafe { RArray::new().into_inner() }
    }
}

impl ReprValue for RArray {
    #[inline]
    fn as_value(&self) -> Value {
        self.0.clone()
    }

    #[inline]
    unsafe fn from_value_unchecked(val: Value) -> Self {
        RArray(val)
    }
}

impl TryConvert for RArray {
    fn try_convert(val: Value) -> Result<Self, Error> {
        if val.rb_type() == crate::value::ValueType::Array {
            // SAFETY: We've verified it's an Array
            Ok(unsafe { RArray::from_value_unchecked(val) })
        } else {
            Err(Error::type_error("expected Array"))
        }
    }
}

impl IntoValue for RArray {
    #[inline]
    fn into_value(self) -> Value {
        self.as_value()
    }
}

// Conversions for Rust Vec

impl<T: TryConvert> TryConvert for Vec<T> {
    fn try_convert(val: Value) -> Result<Self, Error> {
        RArray::try_convert(val)?.to_vec()
    }
}

impl<T: IntoValue + Copy> IntoValue for Vec<T> {
    fn into_value(self) -> Value {
        let guard = RArray::from_slice(&self);
        // SAFETY: We immediately convert to Value
        unsafe { guard.into_inner().into_value() }
    }
}

// Also implement for slices
impl<T: IntoValue + Copy> IntoValue for &[T] {
    fn into_value(self) -> Value {
        let guard = RArray::from_slice(self);
        // SAFETY: We immediately convert to Value
        unsafe { guard.into_inner().into_value() }
    }
}

#[cfg(all(test, any(feature = "embed", feature = "link-ruby")))]
mod tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;

    #[ruby_test]
    fn test_rarray_new() {
        let arr = RArray::new();
        assert_eq!(arr.len(), 0);
        assert!(arr.is_empty());
    }

    #[ruby_test]
    fn test_rarray_with_capacity() {
        let arr = RArray::with_capacity(100);
        assert_eq!(arr.len(), 0);
        assert!(arr.is_empty());
    }

    #[ruby_test]
    fn test_rarray_push() {
        let arr = RArray::new();
        arr.push(42i64);
        assert_eq!(arr.len(), 1);
        assert!(!arr.is_empty());

        arr.push(100i64);
        assert_eq!(arr.len(), 2);
    }

    #[ruby_test]
    fn test_rarray_pop() {
        let arr = RArray::new();
        arr.push(1i64);
        arr.push(2i64);
        arr.push(3i64);

        let val = arr.pop().unwrap();
        assert_eq!(i64::try_convert(val).unwrap(), 3);
        assert_eq!(arr.len(), 2);

        let val = arr.pop().unwrap();
        assert_eq!(i64::try_convert(val).unwrap(), 2);
        assert_eq!(arr.len(), 1);
    }

    #[ruby_test]
    fn test_rarray_pop_empty() {
        let arr = RArray::new();
        assert!(arr.pop().is_none());
    }

    #[ruby_test]
    fn test_rarray_entry() {
        let arr = RArray::new();
        arr.push(10i64);
        arr.push(20i64);
        arr.push(30i64);

        let val = arr.entry(0);
        assert_eq!(i64::try_convert(val).unwrap(), 10);

        let val = arr.entry(1);
        assert_eq!(i64::try_convert(val).unwrap(), 20);

        let val = arr.entry(2);
        assert_eq!(i64::try_convert(val).unwrap(), 30);
    }

    #[ruby_test]
    fn test_rarray_entry_negative() {
        let arr = RArray::new();
        arr.push(10i64);
        arr.push(20i64);
        arr.push(30i64);

        let val = arr.entry(-1);
        assert_eq!(i64::try_convert(val).unwrap(), 30);

        let val = arr.entry(-2);
        assert_eq!(i64::try_convert(val).unwrap(), 20);

        let val = arr.entry(-3);
        assert_eq!(i64::try_convert(val).unwrap(), 10);
    }

    #[ruby_test]
    fn test_rarray_entry_out_of_bounds() {
        let arr = RArray::new();
        arr.push(10i64);

        let val = arr.entry(5);
        assert!(val.is_nil());

        let val = arr.entry(-5);
        assert!(val.is_nil());
    }

    #[ruby_test]
    fn test_rarray_store() {
        let arr = RArray::new();
        arr.store(0, 42i64);
        assert_eq!(arr.len(), 1);

        let val = arr.entry(0);
        assert_eq!(i64::try_convert(val).unwrap(), 42);

        arr.store(0, 99i64);
        let val = arr.entry(0);
        assert_eq!(i64::try_convert(val).unwrap(), 99);
    }

    #[ruby_test]
    fn test_rarray_store_extends() {
        let arr = RArray::new();
        arr.store(5, 42i64);
        assert_eq!(arr.len(), 6);

        // Elements 0-4 should be nil
        for i in 0..5 {
            assert!(arr.entry(i).is_nil());
        }

        let val = arr.entry(5);
        assert_eq!(i64::try_convert(val).unwrap(), 42);
    }

    #[ruby_test]
    fn test_rarray_store_negative() {
        let arr = RArray::new();
        arr.push(1i64);
        arr.push(2i64);
        arr.push(3i64);

        arr.store(-1, 99i64);
        let val = arr.entry(-1);
        assert_eq!(i64::try_convert(val).unwrap(), 99);
    }

    #[ruby_test]
    fn test_rarray_each() {
        let arr = RArray::new();
        arr.push(1i64);
        arr.push(2i64);
        arr.push(3i64);

        let mut sum = 0i64;
        arr.each(|val| {
            let n = i64::try_convert(val)?;
            sum += n;
            Ok(())
        })
        .unwrap();

        assert_eq!(sum, 6);
    }

    #[ruby_test]
    fn test_rarray_each_empty() {
        let arr = RArray::new();
        let mut count = 0;
        arr.each(|_| {
            count += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(count, 0);
    }

    #[ruby_test]
    fn test_rarray_each_error() {
        let arr = RArray::new();
        arr.push(1i64);
        arr.push(2i64);
        arr.push(3i64);

        let result = arr.each(|_| Err(Error::type_error("test error")));
        assert!(result.is_err());
    }

    #[ruby_test]
    fn test_rarray_from_slice() {
        let slice = &[1i64, 2, 3, 4, 5];
        let arr = RArray::from_slice(slice);
        assert_eq!(arr.len(), 5);

        for (i, &expected) in slice.iter().enumerate() {
            let val = arr.entry(i as isize);
            assert_eq!(i64::try_convert(val).unwrap(), expected);
        }
    }

    #[ruby_test]
    fn test_rarray_to_vec() {
        let arr = RArray::new();
        arr.push(1i64);
        arr.push(2i64);
        arr.push(3i64);

        let vec: Vec<i64> = arr.to_vec().unwrap();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[ruby_test]
    fn test_rarray_try_convert() {
        let arr = RArray::new();
        arr.push(1i64);

        let val = arr.into_value();
        let converted = RArray::try_convert(val).unwrap();
        assert_eq!(converted.len(), 1);
    }

    #[ruby_test]
    fn test_rarray_try_convert_wrong_type() {
        let val = 42i64.into_value();
        assert!(RArray::try_convert(val).is_err());
    }

    #[ruby_test]
    fn test_vec_conversion() {
        let vec = vec![1i64, 2, 3, 4, 5];
        let val = vec.clone().into_value();

        let converted: Vec<i64> = Vec::try_convert(val).unwrap();
        assert_eq!(converted, vec);
    }

    #[ruby_test]
    fn test_slice_into_value() {
        let slice: &[i64] = &[1, 2, 3];
        let val = slice.into_value();

        let arr = RArray::try_convert(val).unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[ruby_test]
    fn test_rarray_mixed_types() {
        use crate::types::RString;

        let arr = RArray::new();
        arr.push(42i64);
        arr.push(RString::new("hello"));
        arr.push(true);

        assert_eq!(arr.len(), 3);

        let val0 = arr.entry(0);
        assert_eq!(i64::try_convert(val0).unwrap(), 42);

        let val1 = arr.entry(1);
        let s = RString::try_convert(val1).unwrap();
        assert_eq!(s.to_string().unwrap(), "hello");

        let val2 = arr.entry(2);
        assert_eq!(bool::try_convert(val2).unwrap(), true);
    }

    #[ruby_test]
    fn test_rarray_default() {
        let arr = RArray::default();
        assert_eq!(arr.len(), 0);
        assert!(arr.is_empty());
    }

    #[ruby_test]
    fn test_rarray_nested() {
        let inner = RArray::new();
        inner.push(1i64);
        inner.push(2i64);

        let outer = RArray::new();
        outer.push(inner);
        outer.push(3i64);

        assert_eq!(outer.len(), 2);

        let val = outer.entry(0);
        let inner_arr = RArray::try_convert(val).unwrap();
        assert_eq!(inner_arr.len(), 2);
    }
}
