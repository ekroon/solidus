//! Phase 2 Stage 7: Class and Module Types Example
//!
//! This example demonstrates Ruby's Class and Module types with the Module trait.
//! Classes and modules are first-class objects in Ruby that define behavior and namespaces.
//!
//! This shows Stage 7 implementation: RClass and RModule types with shared Module trait.

use solidus::prelude::*;

/// Example 1: Getting built-in Ruby classes
///
/// Demonstrates accessing Ruby's standard built-in classes.
#[no_mangle]
pub extern "C" fn example_builtin_classes() -> rb_sys::VALUE {
    // Get Ruby's built-in String class
    let string_class = RClass::from_name("String").unwrap();
    assert_eq!(string_class.name().unwrap(), "String");

    // Get Array class
    let array_class = RClass::from_name("Array").unwrap();
    assert_eq!(array_class.name().unwrap(), "Array");

    // Get Hash class
    let hash_class = RClass::from_name("Hash").unwrap();
    assert_eq!(hash_class.name().unwrap(), "Hash");

    // Get Integer class
    let integer_class = RClass::from_name("Integer").unwrap();
    assert_eq!(integer_class.name().unwrap(), "Integer");

    // Get Float class
    let float_class = RClass::from_name("Float").unwrap();
    assert_eq!(float_class.name().unwrap(), "Float");

    // Get Symbol class
    let symbol_class = RClass::from_name("Symbol").unwrap();
    assert_eq!(symbol_class.name().unwrap(), "Symbol");

    // Return the String class as an example
    string_class.into_value().as_raw()
}

/// Example 2: Getting class names
///
/// Shows how to retrieve the name of a Ruby class.
#[no_mangle]
pub extern "C" fn example_class_names() -> rb_sys::VALUE {
    // Get various classes and check their names
    let classes = vec![
        ("String", "String"),
        ("Array", "Array"),
        ("Hash", "Hash"),
        ("Object", "Object"),
        ("BasicObject", "BasicObject"),
        ("File", "File"),
    ];

    for (class_name, expected_name) in classes {
        let class = RClass::from_name(class_name).unwrap();
        let name = class.name().unwrap();
        assert_eq!(name, expected_name);
    }

    // Return a success value
    Qtrue::new().into_value().as_raw()
}

/// Example 3: Walking the superclass chain
///
/// Demonstrates navigating Ruby's class hierarchy.
#[no_mangle]
pub extern "C" fn example_superclass_chain() -> rb_sys::VALUE {
    // Start with String class
    let string_class = RClass::from_name("String").unwrap();
    assert_eq!(string_class.name().unwrap(), "String");

    // Get its superclass (Object)
    let object_class = string_class.superclass().unwrap();
    assert_eq!(object_class.name().unwrap(), "Object");

    // Get Object's superclass (BasicObject)
    let basic_object = object_class.superclass().unwrap();
    assert_eq!(basic_object.name().unwrap(), "BasicObject");

    // BasicObject has no superclass
    assert!(basic_object.clone().superclass().is_none());

    // Return the BasicObject class
    basic_object.into_value().as_raw()
}

/// Example 4: Complete superclass chain iteration
///
/// Shows how to iterate through all superclasses of a class.
#[no_mangle]
pub extern "C" fn example_iterate_superclasses() -> rb_sys::VALUE {
    // Create an array to store the class hierarchy
    // SAFETY: Value is used immediately and returned to Ruby
    let result = unsafe { RArray::new() };

    // Start with Integer class
    let mut current_class = Some(RClass::from_name("Integer").unwrap());

    // Walk up the chain
    while let Some(class) = current_class {
        if let Some(name) = class.name() {
            // SAFETY: Value is used immediately
            result.push(unsafe { RString::new(&name) });
        }
        current_class = class.superclass();
    }

    // Result should be: ["Integer", "Numeric", "Object", "BasicObject"]
    assert!(result.len() >= 3); // At least Integer, Object, BasicObject

    result.into_value().as_raw()
}

/// Example 5: Getting Ruby modules
///
/// Demonstrates accessing Ruby's built-in modules.
#[no_mangle]
pub extern "C" fn example_builtin_modules() -> rb_sys::VALUE {
    // Get Enumerable module
    let enumerable = RModule::from_name("Enumerable").unwrap();
    assert_eq!(enumerable.name().unwrap(), "Enumerable");

    // Get Kernel module
    let kernel = RModule::from_name("Kernel").unwrap();
    assert_eq!(kernel.name().unwrap(), "Kernel");

    // Get Comparable module
    let comparable = RModule::from_name("Comparable").unwrap();
    assert_eq!(comparable.name().unwrap(), "Comparable");

    // Return the Enumerable module
    enumerable.into_value().as_raw()
}

/// Example 6: Defining constants on classes
///
/// Shows how to define constants on Ruby classes using the Module trait.
#[no_mangle]
pub extern "C" fn example_define_const_on_class() -> rb_sys::VALUE {
    // Get the String class
    let string_class = RClass::from_name("String").unwrap();

    // Define various types of constants
    string_class
        .define_const("SOLIDUS_VERSION", "0.1.0")
        .unwrap();
    string_class
        .define_const("SOLIDUS_MAX_SIZE", 1024i64)
        .unwrap();
    string_class.define_const("SOLIDUS_ENABLED", true).unwrap();

    // Get them back to verify
    let version = string_class.const_get("SOLIDUS_VERSION").unwrap();
    let version_str = RString::try_convert(version).unwrap();
    assert_eq!(version_str.to_string().unwrap(), "0.1.0");

    let max_size = string_class.const_get("SOLIDUS_MAX_SIZE").unwrap();
    assert_eq!(i64::try_convert(max_size).unwrap(), 1024);

    let enabled = string_class.const_get("SOLIDUS_ENABLED").unwrap();
    assert!(bool::try_convert(enabled).unwrap());

    // Return true
    Qtrue::new().into_value().as_raw()
}

/// Example 7: Defining constants on modules
///
/// Shows how to define constants on Ruby modules using the Module trait.
#[no_mangle]
pub extern "C" fn example_define_const_on_module() -> rb_sys::VALUE {
    // Get the Enumerable module
    let enumerable = RModule::from_name("Enumerable").unwrap();

    // Define constants on the module
    enumerable.define_const("SOLIDUS_TEST_INT", 42i64).unwrap();
    enumerable
        .define_const("SOLIDUS_TEST_STR", "Hello from Solidus")
        .unwrap();

    // Get them back to verify
    let test_int = enumerable.const_get("SOLIDUS_TEST_INT").unwrap();
    assert_eq!(i64::try_convert(test_int).unwrap(), 42);

    let test_str = enumerable.const_get("SOLIDUS_TEST_STR").unwrap();
    let s = RString::try_convert(test_str).unwrap();
    assert_eq!(s.to_string().unwrap(), "Hello from Solidus");

    // Return true
    Qtrue::new().into_value().as_raw()
}

/// Example 8: Getting built-in constants
///
/// Demonstrates retrieving Ruby's built-in constants from classes.
#[no_mangle]
pub extern "C" fn example_builtin_constants() -> rb_sys::VALUE {
    // Get File class
    let file_class = RClass::from_name("File").unwrap();

    // File::SEPARATOR is a built-in constant
    let separator = file_class.const_get("SEPARATOR").unwrap();
    let sep_str = RString::try_convert(separator.clone()).unwrap();

    // On Unix-like systems it's "/", on Windows it's "\\"
    let sep = sep_str.to_string().unwrap();
    assert!(!sep.is_empty());

    // Return the separator
    separator.as_raw()
}

/// Example 9: Module trait polymorphism
///
/// Shows how the Module trait provides shared behavior for both classes and modules.
fn define_constant_generic<T: Module>(container: T, name: &str, value: i64) -> Result<(), Error> {
    // This function works with both RClass and RModule
    container.define_const(name, value)
}

#[no_mangle]
pub extern "C" fn example_module_trait_polymorphism() -> rb_sys::VALUE {
    // Use the same function for both class and module
    let string_class = RClass::from_name("String").unwrap();
    define_constant_generic(string_class.clone(), "POLY_TEST_CLASS", 100).unwrap();

    let enumerable = RModule::from_name("Enumerable").unwrap();
    define_constant_generic(enumerable.clone(), "POLY_TEST_MODULE", 200).unwrap();

    // Verify both constants were set
    let class_const = string_class.const_get("POLY_TEST_CLASS").unwrap();
    assert_eq!(i64::try_convert(class_const).unwrap(), 100);

    let module_const = enumerable.const_get("POLY_TEST_MODULE").unwrap();
    assert_eq!(i64::try_convert(module_const).unwrap(), 200);

    Qtrue::new().into_value().as_raw()
}

/// Example 10: Type conversions - Value to RClass
///
/// Demonstrates converting Ruby VALUES to typed RClass.
#[no_mangle]
pub extern "C" fn example_value_to_class(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };

    // Try to convert to RClass
    match RClass::try_convert(value) {
        Ok(class) => {
            // Successfully converted to class
            if let Some(name) = class.name() {
                // Return a string with the class name
                // SAFETY: Value is immediately returned to Ruby
                let result = unsafe { RString::new(&format!("Got class: {}", name)) };
                result.into_value().as_raw()
            } else {
                // Anonymous class
                // SAFETY: Value is immediately returned to Ruby
                let result = unsafe { RString::new("Got anonymous class") };
                result.into_value().as_raw()
            }
        }
        Err(_) => {
            // Not a class
            // SAFETY: Value is immediately returned to Ruby
            let result = unsafe { RString::new("Not a class") };
            result.into_value().as_raw()
        }
    }
}

/// Example 11: Type conversions - Value to RModule
///
/// Demonstrates converting Ruby VALUES to typed RModule.
#[no_mangle]
pub extern "C" fn example_value_to_module(val: rb_sys::VALUE) -> rb_sys::VALUE {
    let value = unsafe { Value::from_raw(val) };

    // Try to convert to RModule
    match RModule::try_convert(value) {
        Ok(module) => {
            // Successfully converted to module
            if let Some(name) = module.name() {
                // Return a string with the module name
                // SAFETY: Value is immediately returned to Ruby
                let result = unsafe { RString::new(&format!("Got module: {}", name)) };
                result.into_value().as_raw()
            } else {
                // Anonymous module
                // SAFETY: Value is immediately returned to Ruby
                let result = unsafe { RString::new("Got anonymous module") };
                result.into_value().as_raw()
            }
        }
        Err(_) => {
            // Not a module
            // SAFETY: Value is immediately returned to Ruby
            let result = unsafe { RString::new("Not a module") };
            result.into_value().as_raw()
        }
    }
}

/// Example 12: Error handling for missing constants
///
/// Shows proper error handling when constants don't exist.
#[no_mangle]
pub extern "C" fn example_const_error_handling() -> rb_sys::VALUE {
    let string_class = RClass::from_name("String").unwrap();

    // Try to get a non-existent constant
    let result = string_class.const_get("NONEXISTENT_CONSTANT_XYZ");

    match result {
        Ok(_) => {
            // This shouldn't happen
            // SAFETY: Value is immediately returned to Ruby
            let msg = unsafe { RString::new("ERROR: Found nonexistent constant!") };
            msg.into_value().as_raw()
        }
        Err(err) => {
            // Expected error
            let msg = format!("Expected error: {}", err);
            // SAFETY: Value is immediately returned to Ruby
            let result = unsafe { RString::new(&msg) };
            result.into_value().as_raw()
        }
    }
}

/// Example 13: Distinguishing classes from modules
///
/// Shows how RClass and RModule have distinct types and won't accept the wrong value.
#[no_mangle]
pub extern "C" fn example_type_safety() -> rb_sys::VALUE {
    // String is a class, not a module
    let string_class = RClass::from_name("String").unwrap();
    let string_value = string_class.into_value();

    // This succeeds because it's a class
    assert!(RClass::try_convert(string_value.clone()).is_ok());

    // This fails because String is a class, not a module
    assert!(RModule::try_convert(string_value).is_err());

    // Enumerable is a module, not a class
    let enumerable = RModule::from_name("Enumerable").unwrap();
    let enum_value = enumerable.into_value();

    // This succeeds because it's a module
    assert!(RModule::try_convert(enum_value.clone()).is_ok());

    // This fails because Enumerable is a module, not a class
    assert!(RClass::try_convert(enum_value).is_err());

    Qtrue::new().into_value().as_raw()
}

/// Example 14: Working with nested class hierarchies
///
/// Demonstrates working with more complex class relationships.
#[no_mangle]
pub extern "C" fn example_complex_hierarchy() -> rb_sys::VALUE {
    // Create an array to store class information
    // SAFETY: Value is used immediately and returned to Ruby
    let result = unsafe { RArray::new() };

    // Get File::Stat class (nested class)
    // Note: We use :: syntax for nested classes
    if let Some(file_stat) = RClass::from_name("File::Stat") {
        if let Some(name) = file_stat.name() {
            // SAFETY: Value is used immediately
            result.push(unsafe { RString::new(&name) });
        }

        // Walk its superclass chain
        let mut current = file_stat.superclass();
        while let Some(class) = current {
            if let Some(name) = class.name() {
                // SAFETY: Value is used immediately
                result.push(unsafe { RString::new(&name) });
            }
            current = class.superclass();
        }
    }

    result.into_value().as_raw()
}

/// Example 15: Copying and working with class values
///
/// Shows that RClass and RModule are Clone types that can be duplicated.
#[no_mangle]
pub extern "C" fn example_class_copy_semantics() -> rb_sys::VALUE {
    // RClass is Clone, so we can clone it
    let string_class = RClass::from_name("String").unwrap();

    // These are all clones
    let class1 = string_class.clone();
    let class2 = string_class.clone();
    let class3 = string_class.clone();

    // All refer to the same Ruby class
    assert_eq!(class1.name().unwrap(), "String");
    assert_eq!(class2.name().unwrap(), "String");
    assert_eq!(class3.name().unwrap(), "String");

    // RModule is also Clone
    let enumerable = RModule::from_name("Enumerable").unwrap();
    let mod1 = enumerable.clone();
    let mod2 = enumerable.clone();

    assert_eq!(mod1.name().unwrap(), "Enumerable");
    assert_eq!(mod2.name().unwrap(), "Enumerable");

    Qtrue::new().into_value().as_raw()
}

/// Initialize the extension
#[no_mangle]
pub extern "C" fn Init_phase2_class_module() {
    // Note: Full method definition requires Phase 3
    // For now, this is just a placeholder that Ruby will call when loading the extension
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_checks() {
        // Verify RClass is Clone
        fn assert_clone<T: Clone>() {}
        assert_clone::<RClass>();

        // Verify RModule is Clone
        assert_clone::<RModule>();
    }

    #[test]
    fn test_type_sizes() {
        // RClass should be a transparent wrapper around Value
        assert_eq!(std::mem::size_of::<RClass>(), std::mem::size_of::<Value>());

        // RModule should be a transparent wrapper around Value
        assert_eq!(std::mem::size_of::<RModule>(), std::mem::size_of::<Value>());
    }
}
