//! The base Value type wrapping Ruby's VALUE.

use std::fmt;

/// Ruby value types.
///
/// These correspond to Ruby's internal type tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ValueType {
    /// None/undefined
    None = rb_sys::ruby_value_type::RUBY_T_NONE as u32,
    /// Object instance
    Object = rb_sys::ruby_value_type::RUBY_T_OBJECT as u32,
    /// Class
    Class = rb_sys::ruby_value_type::RUBY_T_CLASS as u32,
    /// Module
    Module = rb_sys::ruby_value_type::RUBY_T_MODULE as u32,
    /// Float
    Float = rb_sys::ruby_value_type::RUBY_T_FLOAT as u32,
    /// String
    String = rb_sys::ruby_value_type::RUBY_T_STRING as u32,
    /// Regexp
    Regexp = rb_sys::ruby_value_type::RUBY_T_REGEXP as u32,
    /// Array
    Array = rb_sys::ruby_value_type::RUBY_T_ARRAY as u32,
    /// Hash
    Hash = rb_sys::ruby_value_type::RUBY_T_HASH as u32,
    /// Struct
    Struct = rb_sys::ruby_value_type::RUBY_T_STRUCT as u32,
    /// Bignum
    Bignum = rb_sys::ruby_value_type::RUBY_T_BIGNUM as u32,
    /// File
    File = rb_sys::ruby_value_type::RUBY_T_FILE as u32,
    /// Data (TypedData)
    Data = rb_sys::ruby_value_type::RUBY_T_DATA as u32,
    /// Match data
    Match = rb_sys::ruby_value_type::RUBY_T_MATCH as u32,
    /// Complex number
    Complex = rb_sys::ruby_value_type::RUBY_T_COMPLEX as u32,
    /// Rational number
    Rational = rb_sys::ruby_value_type::RUBY_T_RATIONAL as u32,
    /// Nil
    Nil = rb_sys::ruby_value_type::RUBY_T_NIL as u32,
    /// True
    True = rb_sys::ruby_value_type::RUBY_T_TRUE as u32,
    /// False
    False = rb_sys::ruby_value_type::RUBY_T_FALSE as u32,
    /// Symbol
    Symbol = rb_sys::ruby_value_type::RUBY_T_SYMBOL as u32,
    /// Fixnum (immediate integer)
    Fixnum = rb_sys::ruby_value_type::RUBY_T_FIXNUM as u32,
    /// Undefined
    Undef = rb_sys::ruby_value_type::RUBY_T_UNDEF as u32,
    /// Internal node
    Node = rb_sys::ruby_value_type::RUBY_T_NODE as u32,
    /// Internal iclass
    IClass = rb_sys::ruby_value_type::RUBY_T_ICLASS as u32,
    /// Zombie (freed but not yet reclaimed)
    Zombie = rb_sys::ruby_value_type::RUBY_T_ZOMBIE as u32,
    /// Moved (used by compacting GC)
    Moved = rb_sys::ruby_value_type::RUBY_T_MOVED as u32,
}

impl ValueType {
    /// Convert from a raw Ruby type tag.
    fn from_raw(raw: rb_sys::ruby_value_type) -> Self {
        match raw {
            rb_sys::ruby_value_type::RUBY_T_NONE => ValueType::None,
            rb_sys::ruby_value_type::RUBY_T_OBJECT => ValueType::Object,
            rb_sys::ruby_value_type::RUBY_T_CLASS => ValueType::Class,
            rb_sys::ruby_value_type::RUBY_T_MODULE => ValueType::Module,
            rb_sys::ruby_value_type::RUBY_T_FLOAT => ValueType::Float,
            rb_sys::ruby_value_type::RUBY_T_STRING => ValueType::String,
            rb_sys::ruby_value_type::RUBY_T_REGEXP => ValueType::Regexp,
            rb_sys::ruby_value_type::RUBY_T_ARRAY => ValueType::Array,
            rb_sys::ruby_value_type::RUBY_T_HASH => ValueType::Hash,
            rb_sys::ruby_value_type::RUBY_T_STRUCT => ValueType::Struct,
            rb_sys::ruby_value_type::RUBY_T_BIGNUM => ValueType::Bignum,
            rb_sys::ruby_value_type::RUBY_T_FILE => ValueType::File,
            rb_sys::ruby_value_type::RUBY_T_DATA => ValueType::Data,
            rb_sys::ruby_value_type::RUBY_T_MATCH => ValueType::Match,
            rb_sys::ruby_value_type::RUBY_T_COMPLEX => ValueType::Complex,
            rb_sys::ruby_value_type::RUBY_T_RATIONAL => ValueType::Rational,
            rb_sys::ruby_value_type::RUBY_T_NIL => ValueType::Nil,
            rb_sys::ruby_value_type::RUBY_T_TRUE => ValueType::True,
            rb_sys::ruby_value_type::RUBY_T_FALSE => ValueType::False,
            rb_sys::ruby_value_type::RUBY_T_SYMBOL => ValueType::Symbol,
            rb_sys::ruby_value_type::RUBY_T_FIXNUM => ValueType::Fixnum,
            rb_sys::ruby_value_type::RUBY_T_UNDEF => ValueType::Undef,
            rb_sys::ruby_value_type::RUBY_T_NODE => ValueType::Node,
            rb_sys::ruby_value_type::RUBY_T_ICLASS => ValueType::IClass,
            rb_sys::ruby_value_type::RUBY_T_ZOMBIE => ValueType::Zombie,
            rb_sys::ruby_value_type::RUBY_T_MOVED => ValueType::Moved,
            _ => ValueType::None,
        }
    }
}

/// A Ruby VALUE wrapper.
///
/// This is a thin wrapper around the raw `VALUE` type from rb-sys.
/// It should not be stored on the heap - use [`BoxValue<T>`](crate::BoxValue) for that.
///
/// # Safety
///
/// `Value` is `Copy` and can be freely passed around, but care must be taken
/// to ensure the underlying Ruby object is not garbage collected while in use.
/// In method signatures, use `Pin<&StackPinned<T>>` to guarantee stack pinning.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Value(rb_sys::VALUE);

impl Value {
    /// Create a Value from a raw Ruby VALUE.
    ///
    /// # Safety
    ///
    /// The VALUE must be valid (either a proper Ruby object reference or
    /// an immediate value like nil, true, false, fixnum, or symbol).
    #[inline]
    pub const unsafe fn from_raw(raw: rb_sys::VALUE) -> Self {
        Value(raw)
    }

    /// Get the raw VALUE.
    #[inline]
    pub const fn as_raw(self) -> rb_sys::VALUE {
        self.0
    }

    /// Check if this value is nil.
    #[inline]
    pub fn is_nil(self) -> bool {
        rb_sys::NIL_P(self.0)
    }

    /// Check if this value is truthy (not nil or false).
    #[inline]
    pub fn is_truthy(self) -> bool {
        rb_sys::TEST(self.0)
    }

    /// Check if this value is false.
    #[inline]
    pub fn is_false(self) -> bool {
        self.0 == Into::<rb_sys::VALUE>::into(rb_sys::Qfalse)
    }

    /// Check if this value is true.
    #[inline]
    pub fn is_true(self) -> bool {
        self.0 == Into::<rb_sys::VALUE>::into(rb_sys::Qtrue)
    }

    /// Check if this value is undefined.
    #[inline]
    pub fn is_undef(self) -> bool {
        self.0 == Into::<rb_sys::VALUE>::into(rb_sys::Qundef)
    }

    /// Check if this is an immediate value (doesn't require GC protection).
    ///
    /// Immediate values include: nil, true, false, fixnums, and symbols.
    /// These values are encoded directly in the VALUE and don't point to
    /// heap-allocated Ruby objects.
    #[inline]
    pub fn is_immediate(self) -> bool {
        rb_sys::IMMEDIATE_P(self.0) || self.is_nil() || self.is_true() || self.is_false()
    }

    /// Get the Ruby type of this value.
    #[inline]
    pub fn rb_type(self) -> ValueType {
        // SAFETY: RB_TYPE handles all cases safely
        let raw_type = unsafe { rb_sys::RB_TYPE(self.0) };
        ValueType::from_raw(raw_type)
    }

    /// Get the nil value.
    #[inline]
    pub fn nil() -> Self {
        // SAFETY: Qnil is always valid
        unsafe { Value::from_raw(rb_sys::Qnil.into()) }
    }

    /// Get the true value.
    #[inline]
    pub fn r#true() -> Self {
        // SAFETY: Qtrue is always valid
        unsafe { Value::from_raw(rb_sys::Qtrue.into()) }
    }

    /// Get the false value.
    #[inline]
    pub fn r#false() -> Self {
        // SAFETY: Qfalse is always valid
        unsafe { Value::from_raw(rb_sys::Qfalse.into()) }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Value")
            .field("raw", &format_args!("{:#x}", self.0))
            .field("type", &self.rb_type())
            .finish()
    }
}

impl PartialEq for Value {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Value {}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests verify compile-time behavior only.
    // Tests requiring Ruby API calls need the `embed` feature.

    #[test]
    fn test_value_size() {
        // Value should be the same size as VALUE
        assert_eq!(
            std::mem::size_of::<Value>(),
            std::mem::size_of::<rb_sys::VALUE>()
        );
    }

    #[test]
    fn test_value_alignment() {
        // Value should have the same alignment as VALUE
        assert_eq!(
            std::mem::align_of::<Value>(),
            std::mem::align_of::<rb_sys::VALUE>()
        );
    }

    #[test]
    fn test_value_is_copy() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<Value>();
    }

    #[test]
    fn test_value_type_variants() {
        // Verify all value types are distinct
        assert_ne!(ValueType::None as u32, ValueType::Object as u32);
        assert_ne!(ValueType::String as u32, ValueType::Array as u32);
        assert_ne!(ValueType::Nil as u32, ValueType::True as u32);
        assert_ne!(ValueType::True as u32, ValueType::False as u32);
    }
}
