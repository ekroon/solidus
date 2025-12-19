//! Error handling for Ruby exceptions.

use std::any::Any;
use std::ffi::CString;
use std::fmt;

use crate::value::Value;

/// Common Ruby exception classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionClass {
    /// StandardError - base class for most exceptions
    StandardError,
    /// RuntimeError - generic runtime error
    RuntimeError,
    /// TypeError - type mismatch error
    TypeError,
    /// ArgumentError - wrong number or type of arguments
    ArgumentError,
    /// NoMemoryError - memory allocation failed
    NoMemoryError,
    /// RangeError - value out of range
    RangeError,
    /// IndexError - index out of bounds
    IndexError,
    /// KeyError - key not found
    KeyError,
    /// NameError - undefined name
    NameError,
    /// NoMethodError - method not found
    NoMethodError,
    /// IOError - I/O operation failed
    IOError,
    /// SystemCallError - system call failed
    SystemCallError,
    /// NotImplementedError - feature not implemented
    NotImplementedError,
    /// FrozenError - object is frozen
    FrozenError,
    /// StopIteration - iteration has ended
    StopIteration,
}

impl ExceptionClass {
    /// Get the Ruby exception class VALUE.
    ///
    /// # Safety
    ///
    /// Ruby must be initialized before calling this.
    pub fn as_value(self) -> Value {
        // SAFETY: These constants are always valid after Ruby init
        unsafe {
            Value::from_raw(match self {
                ExceptionClass::StandardError => rb_sys::rb_eStandardError,
                ExceptionClass::RuntimeError => rb_sys::rb_eRuntimeError,
                ExceptionClass::TypeError => rb_sys::rb_eTypeError,
                ExceptionClass::ArgumentError => rb_sys::rb_eArgError,
                ExceptionClass::NoMemoryError => rb_sys::rb_eNoMemError,
                ExceptionClass::RangeError => rb_sys::rb_eRangeError,
                ExceptionClass::IndexError => rb_sys::rb_eIndexError,
                ExceptionClass::KeyError => rb_sys::rb_eKeyError,
                ExceptionClass::NameError => rb_sys::rb_eNameError,
                ExceptionClass::NoMethodError => rb_sys::rb_eNoMethodError,
                ExceptionClass::IOError => rb_sys::rb_eIOError,
                ExceptionClass::SystemCallError => rb_sys::rb_eSystemCallError,
                ExceptionClass::NotImplementedError => rb_sys::rb_eNotImpError,
                ExceptionClass::FrozenError => rb_sys::rb_eFrozenError,
                ExceptionClass::StopIteration => rb_sys::rb_eStopIteration,
            })
        }
    }
}

/// The class for a Ruby exception - either a built-in or custom class.
#[derive(Debug, Clone)]
enum ErrorClass {
    /// A built-in exception class (resolved lazily)
    BuiltIn(ExceptionClass),
    /// A custom exception class (already resolved)
    Custom(Value),
}

impl ErrorClass {
    /// Get the exception class as a VALUE.
    fn as_value(&self) -> Value {
        match self {
            ErrorClass::BuiltIn(class) => class.as_value(),
            ErrorClass::Custom(value) => value.clone(),
        }
    }
}

/// A Ruby exception.
///
/// This type represents a Ruby exception that can be raised or returned
/// as an error from Rust functions.
///
/// # Example
///
/// ```no_run
/// use solidus::{Error, ExceptionClass};
///
/// fn validate_age(age: i64) -> Result<(), Error> {
///     if age < 0 {
///         return Err(Error::new(ExceptionClass::ArgumentError, "age cannot be negative"));
///     }
///     Ok(())
/// }
/// ```
pub struct Error {
    /// The exception class to use (lazily resolved)
    class: ErrorClass,
    /// The error message
    message: String,
}

impl Error {
    /// Create a new error with the given exception class and message.
    pub fn new<T: Into<String>>(class: ExceptionClass, message: T) -> Self {
        Error {
            class: ErrorClass::BuiltIn(class),
            message: message.into(),
        }
    }

    /// Create an error with a custom exception class.
    pub fn with_class<T: Into<String>>(class: Value, message: T) -> Self {
        Error {
            class: ErrorClass::Custom(class),
            message: message.into(),
        }
    }

    /// Create a RuntimeError with the given message.
    pub fn runtime<T: Into<String>>(message: T) -> Self {
        Error::new(ExceptionClass::RuntimeError, message)
    }

    /// Create a TypeError with the given message.
    pub fn type_error<T: Into<String>>(message: T) -> Self {
        Error::new(ExceptionClass::TypeError, message)
    }

    /// Create an ArgumentError with the given message.
    pub fn argument<T: Into<String>>(message: T) -> Self {
        Error::new(ExceptionClass::ArgumentError, message)
    }

    /// Create a RangeError with the given message.
    pub fn range_error<T: Into<String>>(message: T) -> Self {
        Error::new(ExceptionClass::RangeError, message)
    }

    /// Create an error from a Rust panic.
    ///
    /// This is used internally to convert panics caught by `catch_unwind`
    /// into Ruby exceptions.
    pub fn from_panic(panic: Box<dyn Any + Send>) -> Self {
        let message = if let Some(s) = panic.downcast_ref::<&str>() {
            format!("Rust panic: {}", s)
        } else if let Some(s) = panic.downcast_ref::<String>() {
            format!("Rust panic: {}", s)
        } else {
            "Rust panic: unknown error".to_string()
        };

        Error::new(ExceptionClass::RuntimeError, message)
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the exception class as a VALUE.
    ///
    /// # Safety
    ///
    /// Ruby must be initialized before calling this.
    pub fn exception_class(&self) -> Value {
        self.class.as_value()
    }

    /// Raise this error as a Ruby exception (diverges).
    ///
    /// This function never returns - it raises a Ruby exception using
    /// `rb_raise` which performs a longjmp.
    ///
    /// # Safety
    ///
    /// This must only be called in a context where Ruby exceptions can
    /// be raised (i.e., during a Ruby method call). Raising outside of
    /// Ruby context will crash the process.
    pub fn raise(self) -> ! {
        let c_message = CString::new(self.message.as_str())
            .unwrap_or_else(|_| CString::new("error message contained null byte").unwrap());

        // SAFETY: rb_raise never returns, it longjmps to Ruby's exception handler
        unsafe {
            rb_sys::rb_raise(
                self.class.as_value().as_raw(),
                c_str!("%s"),
                c_message.as_ptr(),
            );
        }
    }

    /// Convert this error into a Ruby exception object.
    ///
    /// Unlike `raise()`, this doesn't raise the exception - it just
    /// creates the exception object. This is useful for storing
    /// exceptions to raise later.
    pub fn to_exception(&self) -> Value {
        let c_message = CString::new(self.message.as_str())
            .unwrap_or_else(|_| CString::new("error message contained null byte").unwrap());

        // SAFETY: rb_exc_new creates a new exception object
        unsafe {
            Value::from_raw(rb_sys::rb_exc_new_cstr(
                self.class.as_value().as_raw(),
                c_message.as_ptr(),
            ))
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("message", &self.message)
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

/// Error returned when Context cannot allocate more stack slots.
#[derive(Debug, Clone, Copy)]
pub struct AllocationError;

impl std::fmt::Display for AllocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context stack slots exhausted")
    }
}

impl std::error::Error for AllocationError {}

// Allow using ? to convert AllocationError to Error
impl From<AllocationError> for Error {
    fn from(_: AllocationError) -> Self {
        Error::runtime("Context stack slots exhausted")
    }
}

// Helper macro for C format strings
macro_rules! c_str {
    ($s:literal) => {
        concat!($s, "\0").as_ptr() as *const std::os::raw::c_char
    };
}
use c_str;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_new() {
        let error = Error::new(ExceptionClass::RuntimeError, "test error");
        assert_eq!(error.message(), "test error");
    }

    #[test]
    fn test_error_runtime() {
        let error = Error::runtime("runtime error");
        assert_eq!(error.message(), "runtime error");
    }

    #[test]
    fn test_error_type_error() {
        let error = Error::type_error("type error");
        assert_eq!(error.message(), "type error");
    }

    #[test]
    fn test_error_argument() {
        let error = Error::argument("argument error");
        assert_eq!(error.message(), "argument error");
    }

    #[test]
    fn test_error_display() {
        let error = Error::new(ExceptionClass::RuntimeError, "display test");
        assert_eq!(format!("{}", error), "display test");
    }

    #[test]
    fn test_error_debug() {
        let error = Error::new(ExceptionClass::RuntimeError, "debug test");
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Error"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_error_from_panic_str() {
        let panic: Box<dyn Any + Send> = Box::new("panic message");
        let error = Error::from_panic(panic);
        assert!(error.message().contains("panic message"));
    }

    #[test]
    fn test_error_from_panic_string() {
        let panic: Box<dyn Any + Send> = Box::new(String::from("panic string"));
        let error = Error::from_panic(panic);
        assert!(error.message().contains("panic string"));
    }

    #[test]
    fn test_error_from_panic_unknown() {
        let panic: Box<dyn Any + Send> = Box::new(42i32);
        let error = Error::from_panic(panic);
        assert!(error.message().contains("unknown"));
    }

    #[test]
    fn test_exception_class_enum() {
        // Just verify the enum variants exist
        let _ = ExceptionClass::StandardError;
        let _ = ExceptionClass::RuntimeError;
        let _ = ExceptionClass::TypeError;
        let _ = ExceptionClass::ArgumentError;
    }
}
