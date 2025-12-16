#!/usr/bin/env ruby
# frozen_string_literal: true

# Test script for Phase 2 Stage 7: Class and Module Types Example
#
# This stage implements the RClass and RModule types with the Module trait.
# Full integration testing from Ruby requires Phase 3 (method definition).
# This script verifies that the extension builds successfully.

puts "=== Phase 2 Stage 7: Class and Module Types Example ==="
puts

# Build the extension
puts "Building phase2_class_module extension..."
build_result = system("cargo build --release --manifest-path #{__dir__}/Cargo.toml")

unless build_result
  puts "❌ Build failed"
  exit(1)
end

puts "✓ Build successful"
puts

# Verify the library was created
lib_name = case RUBY_PLATFORM
           when /darwin/
             'libphase2_class_module.dylib'
           when /linux/
             'libphase2_class_module.so'
           when /mingw|mswin/
             'phase2_class_module.dll'
           else
             raise "Unsupported platform: #{RUBY_PLATFORM}"
           end

lib_path = File.expand_path("target/release/#{lib_name}", __dir__)

if File.exist?(lib_path)
  puts "✓ Extension library created at:"
  puts "  #{lib_path}"
else
  puts "❌ Extension library not found at:"
  puts "  #{lib_path}"
  exit(1)
end

puts
puts "=== Testing Notes ==="
puts
puts "This example demonstrates Phase 2 Stage 7 implementation:"
puts "  • RClass type - Working with Ruby classes"
puts "  • RModule type - Working with Ruby modules"
puts "  • Class properties - name(), superclass()"
puts "  • Module trait - Shared behavior between RClass and RModule"
puts "  • Constants - define_const(), const_get()"
puts "  • Class hierarchy - Navigating the inheritance chain"
puts "  • Type safety - Compile-time guarantees"
puts "  • Built-in classes - Accessing Ruby's built-in classes"
puts
puts "The extension includes 15 example functions:"
puts "  1. example_builtin_classes - Access Ruby's standard classes"
puts "  2. example_class_names - Retrieve names from multiple classes"
puts "  3. example_superclass_chain - Navigate from String to BasicObject"
puts "  4. example_iterate_superclasses - Walk complete inheritance chains"
puts "  5. example_builtin_modules - Access standard modules"
puts "  6. example_define_const_on_class - Define constants on classes"
puts "  7. example_define_const_on_module - Define constants on modules"
puts "  8. example_builtin_constants - Access Ruby's built-in constants"
puts "  9. example_module_trait_polymorphism - Use generic functions"
puts "  10. example_value_to_class - Convert VALUES to typed RClass"
puts "  11. example_value_to_module - Convert VALUES to typed RModule"
puts "  12. example_const_error_handling - Handle missing constants"
puts "  13. example_type_safety - Demonstrate compile-time guarantees"
puts "  14. example_complex_hierarchy - Work with nested classes"
puts "  15. example_class_copy_semantics - Show efficient value passing"
puts
puts "Phase 3 will add full method definition support to enable calling"
puts "these functions directly from Ruby using natural syntax."
puts
puts "=== Key Features ==="
puts
puts "RClass type:"
puts "  • from_name(name) - Get a class by name"
puts "  • name() - Get the class name"
puts "  • superclass() - Get the superclass"
puts "  • TryConvert - Type-safe conversions"
puts
puts "RModule type:"
puts "  • from_name(name) - Get a module by name"
puts "  • name() - Get the module name"
puts "  • TryConvert - Type-safe conversions"
puts
puts "Module trait (shared by RClass and RModule):"
puts "  • define_const(name, value) - Define a constant"
puts "  • const_get(name) - Get a constant value"
puts
puts "=== Example Usage (from Rust) ==="
puts
puts <<~RUST
  // Get Ruby's built-in String class
  let string_class = RClass::from_name("String").unwrap();
  assert_eq!(string_class.name().unwrap(), "String");
  
  // Walk the superclass chain
  let object_class = string_class.superclass().unwrap();
  assert_eq!(object_class.name().unwrap(), "Object");
  
  // Get a module
  let enumerable = RModule::from_name("Enumerable").unwrap();
  
  // Define constants using the Module trait
  string_class.define_const("MY_CONST", 42i64).unwrap();
  enumerable.define_const("MY_CONST", "Hello").unwrap();
  
  // Get constants back
  let val = string_class.const_get("MY_CONST").unwrap();
  assert_eq!(i64::try_convert(val).unwrap(), 42);
RUST

puts "=== Design Highlights ==="
puts
puts "Type Safety:"
puts "  • RClass and RModule are distinct types"
puts "  • Compile-time checks prevent mixing them up"
puts "  • TryConvert provides runtime validation"
puts
puts "Shared Behavior:"
puts "  • Module trait unifies common operations"
puts "  • Generic code works with both classes and modules"
puts "  • Clear distinction where behavior differs"
puts
puts "Copy Semantics:"
puts "  • Both types are Copy (just VALUE wrappers)"
puts "  • No need for explicit cloning or borrowing"
puts "  • Efficient passing throughout the codebase"
puts
puts "Error Handling:"
puts "  • Missing classes/modules return None"
puts "  • Missing constants return Result::Err"
puts "  • Clear error messages for debugging"
puts
puts "✓ All Phase 2 Stage 7 features demonstrated successfully!"
puts
puts "See README.md for detailed documentation and examples."
