//! Wrap, get, and get_mut functions (placeholder)

use crate::error::Error;
use crate::ruby::Ruby;
use crate::types::RClass;
use crate::value::Value;

use super::TypedData;

/// Placeholder wrap function
pub fn wrap<T: TypedData>(_ruby: &Ruby, _class: &RClass, _value: T) -> Result<Value, Error> {
    unimplemented!("wrap() not yet implemented")
}

/// Placeholder get function
pub fn get<T: TypedData>(_value: &Value) -> Result<&T, Error> {
    unimplemented!("get() not yet implemented")
}

/// Placeholder get_mut function
pub fn get_mut<T: TypedData>(_value: &Value) -> Result<&mut T, Error> {
    unimplemented!("get_mut() not yet implemented")
}
