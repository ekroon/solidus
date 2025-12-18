//! TypedData and DataTypeFunctions traits (placeholder)

use super::DataType;

/// Placeholder TypedData trait
pub trait TypedData: Sized + Send {
    /// The Ruby class name for this type.
    fn class_name() -> &'static str;

    /// The DataType descriptor for this type.
    fn data_type() -> &'static DataType;
}

/// Placeholder DataTypeFunctions trait
pub trait DataTypeFunctions: TypedData {}
