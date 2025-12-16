#!/usr/bin/env ruby
# frozen_string_literal: true

# Test script for Phase 2 Stage 5: Array Type Example
#
# This stage implements the RArray type with iteration support.
# Full integration testing from Ruby requires Phase 3 (method definition).
# This script verifies that the extension builds successfully.

puts "=== Phase 2 Stage 5: Array Type Example ==="
puts

# Build the extension
puts "Building phase2_array extension..."
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
             'libphase2_array.dylib'
           when /linux/
             'libphase2_array.so'
           when /mingw|mswin/
             'phase2_array.dll'
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
puts "This example demonstrates Ruby Array type at the Rust level."
puts
puts "Implemented features:"
puts "• RArray creation with new() and with_capacity()"
puts "• Array properties: len(), is_empty()"
puts "• Element access: entry() with positive and negative indices"
puts "• Element modification: store() at any index"
puts "• Stack operations: push(), pop()"
puts "• Iteration: each() with closures"
puts "• Conversions: from_slice(), to_vec()"
puts "• Mixed-type arrays (heterogeneous)"
puts "• Type-safe typed arrays (homogeneous)"
puts "• Nested arrays (multi-dimensional)"
puts "• Error handling for type mismatches"
puts
puts "Key features:"
puts "• Arrays are heap-allocated and require GC protection"
puts "• Dynamic sizing - grows automatically"
puts "• Heterogeneous - can hold any Ruby value"
puts "• Negative indexing (Ruby style)"
puts "• Out-of-bounds access returns nil"
puts "• Store beyond length extends with nils"
puts "• Compile-time type safety with generics"
puts
puts "Array operations:"
puts "• new() - Create empty array"
puts "• with_capacity(n) - Pre-allocate space"
puts "• push(val) - Add to end"
puts "• pop() -> Option<Value> - Remove from end"
puts "• entry(i) -> Value - Get element (nil if out of bounds)"
puts "• store(i, val) - Set element (extends if needed)"
puts "• each(|val| ...) - Iterate with closure"
puts "• from_slice(&[T]) - Convert Rust slice"
puts "• to_vec<T>() - Convert to Rust Vec"
puts
puts "Example functions:"
puts "• example_array_new() - Empty array creation"
puts "• example_array_with_capacity() - Pre-allocated array"
puts "• example_array_push_pop() - Stack operations"
puts "• example_array_entry() - Element access with indices"
puts "• example_array_store() - Storing at indices"
puts "• example_array_each() - Iteration with closures"
puts "• example_array_from_slice() - Rust slice to array"
puts "• example_array_to_vec() - Array to Rust vector"
puts "• example_array_mixed_types() - Heterogeneous arrays"
puts "• example_typed_array() - Type-safe homogeneous arrays"
puts "• example_nested_arrays() - Multi-dimensional arrays"
puts "• example_array_error_handling() - Type mismatch handling"
puts
puts "Run 'cargo test' in this directory to run the Rust unit tests."
puts "Full Ruby integration testing requires Phase 3 (method registration)."
puts
puts "=== Next Steps ==="
puts
puts "Stage 6 will implement Hash type with key-value operations."
puts "Stage 7+ will add Class and Module types."
