# Phase 2 Stage 7: Class and Module Types Example

This example demonstrates Solidus's implementation of Ruby's Class and Module types, showcasing the type-safe wrappers around Ruby's class and module system.

## Overview

Ruby classes and modules are first-class objects that define behavior and namespaces. This example shows how Solidus provides type-safe access to these objects while maintaining Ruby's dynamic capabilities.

## Key Features Demonstrated

### 1. **RClass Type**
- Getting built-in Ruby classes by name
- Retrieving class names
- Navigating the superclass chain
- Walking complete class hierarchies
- Type-safe conversions from Ruby VALUES

### 2. **RModule Type**
- Getting built-in Ruby modules by name
- Retrieving module names
- Type-safe conversions from Ruby VALUES
- Distinguishing modules from classes

### 3. **Module Trait**
- Shared behavior between RClass and RModule
- Polymorphic constant operations
- Generic functions that work with both types
- Unified interface for classes and modules

### 4. **Constants**
- Defining constants on classes
- Defining constants on modules
- Retrieving constants with error handling
- Accessing built-in Ruby constants

### 5. **Class Hierarchy**
- Walking superclass chains
- Navigating from any class to BasicObject
- Working with nested classes (e.g., File::Stat)
- Understanding Ruby's class relationships

### 6. **Type Safety**
- Compile-time distinction between classes and modules
- Type conversion errors for invalid types
- Safe error handling for missing constants
- Copy semantics for efficient value passing

## Examples Included

The example provides 15 comprehensive demonstrations:

1. **`example_builtin_classes`** - Access Ruby's standard classes (String, Array, Hash, etc.)
2. **`example_class_names`** - Retrieve names from multiple classes
3. **`example_superclass_chain`** - Navigate from String to BasicObject
4. **`example_iterate_superclasses`** - Walk complete inheritance chains
5. **`example_builtin_modules`** - Access standard modules (Enumerable, Kernel, Comparable)
6. **`example_define_const_on_class`** - Define multiple types of constants on classes
7. **`example_define_const_on_module`** - Define constants on modules
8. **`example_builtin_constants`** - Access Ruby's built-in constants like File::SEPARATOR
9. **`example_module_trait_polymorphism`** - Use generic functions with Module trait
10. **`example_value_to_class`** - Convert Ruby VALUES to typed RClass
11. **`example_value_to_module`** - Convert Ruby VALUES to typed RModule
12. **`example_const_error_handling`** - Handle missing constants gracefully
13. **`example_type_safety`** - Demonstrate compile-time type guarantees
14. **`example_complex_hierarchy`** - Work with nested class structures
15. **`example_class_copy_semantics`** - Show efficient value passing

## Building

```bash
# Build the example
cargo build --release

# Or build in debug mode for development
cargo build
```

## Running

```bash
# Run the test script
ruby test.rb
```

The test script will:
1. Load the compiled extension
2. Execute all 15 example functions
3. Verify the results
4. Display the class/module relationships discovered

## Code Walkthrough

### Getting Built-in Classes

```rust
// Get Ruby's String class
let string_class = RClass::from_name("String").unwrap();
assert_eq!(string_class.name().unwrap(), "String");

// Get other built-in classes
let array_class = RClass::from_name("Array").unwrap();
let hash_class = RClass::from_name("Hash").unwrap();
```

### Walking the Superclass Chain

```rust
// Start with String class
let string_class = RClass::from_name("String").unwrap();

// Get its superclass (Object)
let object_class = string_class.superclass().unwrap();
assert_eq!(object_class.name().unwrap(), "Object");

// Get Object's superclass (BasicObject)
let basic_object = object_class.superclass().unwrap();
assert_eq!(basic_object.name().unwrap(), "BasicObject");

// BasicObject has no superclass
assert!(basic_object.superclass().is_none());
```

### Working with Modules

```rust
// Get Enumerable module
let enumerable = RModule::from_name("Enumerable").unwrap();
assert_eq!(enumerable.name().unwrap(), "Enumerable");

// Note: String is a class, so this returns None
let not_a_module = RModule::from_name("String");
assert!(not_a_module.is_none());
```

### Using the Module Trait

```rust
// Generic function that works with both classes and modules
fn define_constant<T: Module>(container: T, name: &str, value: i64) -> Result<(), Error> {
    container.define_const(name, value)
}

// Works with RClass
let string_class = RClass::from_name("String").unwrap();
define_constant(string_class, "MY_CONST", 100).unwrap();

// Works with RModule
let enumerable = RModule::from_name("Enumerable").unwrap();
define_constant(enumerable, "MY_CONST", 200).unwrap();
```

### Defining and Getting Constants

```rust
// Define constants on a class
let string_class = RClass::from_name("String").unwrap();
string_class.define_const("VERSION", "1.0.0").unwrap();
string_class.define_const("MAX_SIZE", 1024i64).unwrap();

// Get them back
let version = string_class.const_get("VERSION").unwrap();
let version_str = RString::try_convert(version).unwrap();
assert_eq!(version_str.to_string().unwrap(), "1.0.0");

// Handle missing constants
let result = string_class.const_get("NONEXISTENT");
assert!(result.is_err());
```

### Type Safety

```rust
// String is a class, not a module
let string_class = RClass::from_name("String").unwrap();
let value = string_class.into_value();

// This succeeds
assert!(RClass::try_convert(value).is_ok());

// This fails at runtime (would be a compile error if we tried directly)
assert!(RModule::try_convert(value).is_err());
```

## API Reference

### RClass

```rust
impl RClass {
    /// Get a class by name (e.g., "String", "Array")
    pub fn from_name(name: &str) -> Option<Self>;
    
    /// Get the name of this class
    pub fn name(self) -> Option<String>;
    
    /// Get the superclass (returns None for BasicObject)
    pub fn superclass(self) -> Option<RClass>;
}
```

### RModule

```rust
impl RModule {
    /// Get a module by name (e.g., "Enumerable", "Kernel")
    pub fn from_name(name: &str) -> Option<Self>;
    
    /// Get the name of this module
    pub fn name(self) -> Option<String>;
}
```

### Module Trait

```rust
pub trait Module: ReprValue {
    /// Define a constant in this module/class
    fn define_const<T: IntoValue>(self, name: &str, value: T) -> Result<(), Error>;
    
    /// Get a constant from this module/class
    fn const_get(self, name: &str) -> Result<Value, Error>;
}
```

Both `RClass` and `RModule` implement the `Module` trait.

## Design Principles

### 1. **Type Safety**
- `RClass` and `RModule` are distinct types
- Compile-time checks prevent using the wrong type
- `TryConvert` provides runtime validation when needed

### 2. **Copy Semantics**
- Both types are `Copy` (they're just VALUE wrappers)
- No need for explicit cloning or borrowing
- Efficient passing around the codebase

### 3. **Shared Behavior**
- Module trait unifies common operations
- Generic code works with both classes and modules
- Clear distinction where behavior differs

### 4. **Error Handling**
- Missing classes/modules return `None`
- Missing constants return `Result::Err`
- Clear error messages for debugging

### 5. **Navigation Support**
- Easy traversal of class hierarchies
- Support for nested classes (Class::NestedClass)
- Access to Ruby's complete type system

## Limitations

### Not Yet Implemented (Phase 3)

The following operations are deferred to Phase 3:

- `define_method` - Defining methods on classes/modules
- Method lookup and invocation
- Include/prepend for modules
- Creating new classes/modules at runtime

These are intentionally omitted from Phase 2 to focus on the core type system.

### Known Issues

1. **Exception Handling**: `from_name()` with non-existent classes may have issues
   with exception handling in some scenarios. Use only with known class/module names.

## Testing

The example includes both Rust and Ruby tests:

### Rust Tests

```bash
# Run compile-time checks
cargo test
```

These verify:
- Type sizes and properties
- Copy trait implementation
- Transparent wrapper guarantees

### Ruby Integration Tests

```bash
# Build and run Ruby tests
cargo build --release
ruby test.rb
```

These verify:
- All example functions work correctly
- Constants are properly defined
- Class hierarchies are correctly navigated
- Type conversions work as expected

## Learning Path

1. **Start with**: `example_builtin_classes` - Basic class access
2. **Then try**: `example_superclass_chain` - Class hierarchy
3. **Explore**: `example_builtin_modules` - Module access
4. **Learn**: `example_define_const_on_class` - Constant operations
5. **Master**: `example_module_trait_polymorphism` - Generic programming

## Common Patterns

### Checking if a Constant Exists

```rust
let class = RClass::from_name("String").unwrap();
match class.const_get("MY_CONST") {
    Ok(value) => {
        // Constant exists, use it
        let num = i64::try_convert(value)?;
        println!("MY_CONST = {}", num);
    }
    Err(_) => {
        // Constant doesn't exist, define it
        class.define_const("MY_CONST", 42i64)?;
    }
}
```

### Walking All Superclasses

```rust
let mut current = Some(RClass::from_name("Integer").unwrap());
let mut chain = Vec::new();

while let Some(class) = current {
    if let Some(name) = class.name() {
        chain.push(name);
    }
    current = class.superclass();
}

println!("Hierarchy: {}", chain.join(" -> "));
```

### Generic Constant Definition

```rust
fn setup_constants<T: Module>(container: T) -> Result<(), Error> {
    container.define_const("VERSION", "1.0.0")?;
    container.define_const("DEBUG", false)?;
    container.define_const("MAX_ITEMS", 100i64)?;
    Ok(())
}

// Works with both classes and modules
setup_constants(RClass::from_name("String").unwrap())?;
setup_constants(RModule::from_name("Enumerable").unwrap())?;
```

## Related Examples

- **phase2_conversions** - Type conversion fundamentals
- **phase2_string** - String type implementation
- **phase2_array** - Array type implementation
- **phase2_hash** - Hash type implementation

## Further Reading

- [Phase 2 Types Plan](../../docs/plan/phase-2-types.md) - Overall type system design
- [Phase 2 Tasks](../../docs/plan/phase-2-tasks.md) - Implementation stages
- [Ruby Class Documentation](https://ruby-doc.org/core/Class.html)
- [Ruby Module Documentation](https://ruby-doc.org/core/Module.html)

## License

MIT License - see LICENSE-MIT in the repository root.
