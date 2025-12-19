# Error Handling in Solidus

This guide covers error handling patterns for Ruby extensions written with Solidus.

## The Error Type

Solidus provides the `Error` type for representing Ruby exceptions in Rust. It consists of:

- An **exception class** (e.g., `TypeError`, `ArgumentError`)
- An **error message** (a human-readable string)

```rust
use solidus::{Error, ExceptionClass};

// Create an error with a specific exception class
let error = Error::new(ExceptionClass::ArgumentError, "invalid argument");

// Access the error message
assert_eq!(error.message(), "invalid argument");
```

The `Error` type implements:
- `std::error::Error` - for compatibility with Rust's error handling ecosystem
- `std::fmt::Display` - displays the error message
- `std::fmt::Debug` - debug representation

## Creating Errors

### Convenience Constructors

Solidus provides convenience methods for common exception types:

```rust
use solidus::Error;

// RuntimeError - generic runtime errors
let err = Error::runtime("something went wrong");

// TypeError - type mismatch errors
let err = Error::type_error("expected String, got Integer");

// ArgumentError - wrong arguments
let err = Error::argument("expected at least 2 arguments");

// RangeError - value out of range
let err = Error::range_error("value must be between 0 and 100");
```

### Using ExceptionClass

For other exception types or more control, use `Error::new` with an `ExceptionClass`:

```rust
use solidus::{Error, ExceptionClass};

// NameError - undefined name
let err = Error::new(ExceptionClass::NameError, "undefined local variable 'foo'");

// IndexError - index out of bounds
let err = Error::new(ExceptionClass::IndexError, "index 5 out of bounds for length 3");

// KeyError - key not found
let err = Error::new(ExceptionClass::KeyError, "key not found: :missing");

// IOError - I/O operation failed
let err = Error::new(ExceptionClass::IOError, "failed to read file");

// FrozenError - object is frozen
let err = Error::new(ExceptionClass::FrozenError, "can't modify frozen String");
```

### Available Exception Classes

The `ExceptionClass` enum includes all common Ruby exception types:

| Exception Class | Use Case |
|-----------------|----------|
| `StandardError` | Base class for most exceptions |
| `RuntimeError` | Generic runtime error |
| `TypeError` | Type mismatch (wrong class) |
| `ArgumentError` | Wrong number or type of arguments |
| `RangeError` | Numeric value out of range |
| `IndexError` | Index out of bounds |
| `KeyError` | Key not found (Hash, etc.) |
| `NameError` | Undefined name |
| `NoMethodError` | Method not found |
| `IOError` | I/O operation failed |
| `SystemCallError` | System call failed |
| `NotImplementedError` | Feature not implemented |
| `FrozenError` | Attempt to modify frozen object |
| `NoMemoryError` | Memory allocation failed |
| `StopIteration` | Iteration has ended |

### Custom Exception Classes

For custom Ruby exception classes, use `Error::with_class`:

```rust
use solidus::{Error, Value};

fn raise_custom_error(exception_class: Value) -> Error {
    Error::with_class(exception_class, "custom error occurred")
}
```

## The ? Operator and Error Propagation

Solidus methods return `Result<T, Error>`, making error propagation natural with `?`:

```rust
use solidus::prelude::*;
use std::pin::Pin;

fn process_string(s: Pin<&StackPinned<RString>>) -> Result<i64, Error> {
    // Convert to Rust string - propagates TypeError if conversion fails
    let rust_string = s.get().to_string()?;
    
    // Parse as integer - we convert the parse error to ArgumentError
    let number: i64 = rust_string.parse()
        .map_err(|_| Error::argument("expected a numeric string"))?;
    
    Ok(number * 2)
}
```

The `?` operator:
1. Returns early with the error if the `Result` is `Err`
2. Unwraps the value if the `Result` is `Ok`
3. Works seamlessly because `Error` implements `std::error::Error`

## Converting Rust Errors to Ruby Exceptions

### Using map_err

Convert Rust errors to Solidus errors with `map_err`:

```rust
use solidus::Error;
use std::fs;

fn read_config(path: &str) -> Result<String, Error> {
    fs::read_to_string(path)
        .map_err(|e| Error::new(
            solidus::ExceptionClass::IOError,
            format!("failed to read {}: {}", path, e)
        ))
}
```

### Common Patterns

```rust
use solidus::Error;

// Parse errors -> ArgumentError
fn parse_int(s: &str) -> Result<i64, Error> {
    s.parse().map_err(|_| Error::argument(format!("'{}' is not a valid integer", s)))
}

// Option -> KeyError
fn get_required<T>(opt: Option<T>, key: &str) -> Result<T, Error> {
    opt.ok_or_else(|| Error::new(
        solidus::ExceptionClass::KeyError,
        format!("required key '{}' not found", key)
    ))
}

// Bounds checking -> RangeError
fn validate_range(value: i64, min: i64, max: i64) -> Result<i64, Error> {
    if value < min || value > max {
        return Err(Error::range_error(format!(
            "value {} is outside range {}..{}", value, min, max
        )));
    }
    Ok(value)
}
```

## How Errors Become Ruby Exceptions

When a Solidus method returns `Err(error)`, the generated wrapper code:

1. Catches the error from your Rust function
2. Calls `error.raise()` which invokes Ruby's `rb_raise`
3. The Ruby VM handles the exception using its normal exception mechanism

```rust
use solidus::prelude::*;

fn divide(a: i64, b: i64) -> Result<i64, Error> {
    if b == 0 {
        return Err(Error::argument("division by zero"));
    }
    Ok(a / b)
}

// When called from Ruby with b=0, raises:
// ArgumentError: division by zero
```

### Panic Handling

Solidus also catches Rust panics and converts them to Ruby exceptions:

```rust
fn might_panic() -> Result<i64, Error> {
    panic!("something went terribly wrong");
    // Converted to: RuntimeError: Rust panic: something went terribly wrong
}
```

This prevents panics from unwinding through Ruby's C code, which would cause undefined behavior.

## Best Practices for Error Messages

### Be Specific

```rust
// Bad: vague message
Err(Error::type_error("wrong type"))

// Good: tells user what was expected and received
Err(Error::type_error("expected String, got Integer"))
```

### Include Context

```rust
// Bad: no context
Err(Error::argument("invalid value"))

// Good: includes the problematic value
Err(Error::argument(format!("invalid port number: {}", port)))
```

### Match Ruby Conventions

Ruby error messages typically:
- Start with a lowercase letter
- Don't end with punctuation
- Are concise but informative

```rust
// Matches Ruby style
Err(Error::argument("wrong number of arguments (given 3, expected 1)"))
Err(Error::type_error("no implicit conversion of Integer into String"))
Err(Error::new(ExceptionClass::IndexError, "index 5 outside of array bounds"))
```

### Use the Right Exception Type

| Situation | Exception Type |
|-----------|----------------|
| Wrong argument type | `TypeError` |
| Wrong argument count or value | `ArgumentError` |
| Numeric overflow/underflow | `RangeError` |
| Array/String index out of bounds | `IndexError` |
| Hash key not found | `KeyError` |
| Undefined constant/variable | `NameError` |
| Method doesn't exist | `NoMethodError` |
| File/network errors | `IOError` |
| Modifying frozen object | `FrozenError` |
| Generic runtime error | `RuntimeError` |

## Complete Example

Here's a complete example showing various error handling patterns:

```rust
use solidus::prelude::*;
use std::pin::Pin;

/// Validates and processes a user age value.
fn validate_age(age: i64) -> Result<i64, Error> {
    if age < 0 {
        return Err(Error::argument("age cannot be negative"));
    }
    if age > 150 {
        return Err(Error::range_error(format!(
            "age {} is unreasonably large",
            age
        )));
    }
    Ok(age)
}

/// Parses a name string, ensuring it's not empty.
fn parse_name(name: Pin<&StackPinned<RString>>) -> Result<String, Error> {
    let name_str = name.get().to_string()?;  // Propagates TypeError on invalid UTF-8
    
    let trimmed = name_str.trim();
    if trimmed.is_empty() {
        return Err(Error::argument("name cannot be empty"));
    }
    
    Ok(trimmed.to_string())
}

/// Creates a greeting for a person, with full error handling.
fn create_greeting(
    name: Pin<&StackPinned<RString>>,
    age: i64,
) -> Result<NewValue<RString>, Error> {
    // Validate inputs - errors propagate with ?
    let validated_name = parse_name(name)?;
    let validated_age = validate_age(age)?;
    
    // Create the greeting
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&format!(
        "Hello, {}! You are {} years old.",
        validated_name,
        validated_age
    )) })
}
```

## See Also

- [Getting Started](getting-started.md) - Basic extension setup
- [Methods and Functions](methods.md) - Defining Ruby methods
- [API Documentation](https://docs.rs/solidus) - Full API reference
