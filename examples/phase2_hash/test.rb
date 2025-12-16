#!/usr/bin/env ruby
# frozen_string_literal: true

# Test script for Phase 2 Stage 6: Hash Type Example
#
# This stage implements the RHash type with key-value operations.
# Full integration testing from Ruby requires Phase 3 (method definition).
# This script verifies that the extension builds successfully.

puts "=== Phase 2 Stage 6: Hash Type Example ==="
puts

# Build the extension
puts "Building phase2_hash extension..."
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
             'libphase2_hash.dylib'
           when /linux/
             'libphase2_hash.so'
           when /mingw|mswin/
             'phase2_hash.dll'
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
puts "This example demonstrates Ruby Hash type at the Rust level."
puts
puts "Implemented features:"
puts "• RHash creation with new()"
puts "• Hash properties: len(), is_empty()"
puts "• Key-value operations: insert(), get(), delete()"
puts "• Iteration with each() and closures"
puts "• Type-safe conversions between Rust HashMap and Ruby Hash"
puts "• Support for multiple key types (String, Symbol, Integer)"
puts "• Nested hashes (hashes as values)"
puts "• Error handling for missing keys (returns Option)"
puts
puts "Key features:"
puts "• RHash is heap-allocated and requires GC protection"
puts "• Keys can be any Ruby value (strings, symbols, integers, etc.)"
puts "• Values can be any Ruby value"
puts "• Compile-time type safety with TryConvert and IntoValue"
puts "• Safe iteration with closures (not Iterator trait)"
puts
puts "Key types demonstrated:"
puts "• String keys - hash.insert(\"name\", \"Alice\")"
puts "• Symbol keys - hash.insert(Symbol::new(\"status\"), \"active\")"
puts "• Integer keys - hash.insert(1i64, \"first\")"
puts "• Mixed key types in the same hash"
puts
puts "Example functions:"
puts "• example_hash_new() - Create empty hash"
puts "• example_hash_insert() - Insert key-value pairs"
puts "• example_hash_get() - Get values by key"
puts "• example_hash_update() - Update existing keys"
puts "• example_hash_delete() - Delete keys"
puts "• example_hash_iteration() - Iterate with each()"
puts "• example_hash_symbol_keys() - Use Symbol keys"
puts "• example_hash_integer_keys() - Use Integer keys"
puts "• example_hash_mixed_keys() - Mix different key types"
puts "• example_hash_from_hashmap() - Convert from Rust HashMap"
puts "• example_hash_to_hashmap() - Convert to Rust HashMap"
puts "• example_hash_nested() - Nested hash structures"
puts "• example_hash_type_safe() - Type-safe operations"
puts "• example_hash_roundtrip() - Round-trip conversion"
puts "• example_hash_collect_keys() - Collect keys during iteration"
puts
puts "Run 'cargo test' in this directory to run the Rust unit tests."
puts "Full Ruby integration testing requires Phase 3 (method registration)."
puts
puts "=== Next Steps ==="
puts
puts "Stage 7+ will implement Class and Module types."
puts "Phase 3 will add method definition and class registration."
