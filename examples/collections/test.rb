#!/usr/bin/env ruby
# frozen_string_literal: true

# Test script for Collections Example
#
# This example demonstrates working with Ruby collections (RArray and RHash).
# Full integration testing from Ruby requires Phase 3 (method definition).
# This script verifies that the extension builds successfully.

puts "=== Collections Example ==="
puts

# Build the extension
puts "Building collections extension..."
build_result = system("cargo build --release --manifest-path #{__dir__}/Cargo.toml")

unless build_result
  puts "Build failed"
  exit(1)
end

puts "Build successful"
puts

# Verify the library was created
lib_name = case RUBY_PLATFORM
           when /darwin/
             'libcollections.dylib'
           when /linux/
             'libcollections.so'
           when /mingw|mswin/
             'collections.dll'
           else
             raise "Unsupported platform: #{RUBY_PLATFORM}"
           end

lib_path = File.expand_path("target/release/#{lib_name}", __dir__)

if File.exist?(lib_path)
  puts "Extension library created at:"
  puts "  #{lib_path}"
else
  puts "Extension library not found at:"
  puts "  #{lib_path}"
  exit(1)
end

puts
puts "=== Collection Patterns Demonstrated ==="
puts
puts "PART 1: Working with Arrays"
puts "  * build_array()         - Create and populate with push()"
puts "  * iterate_array_sum()   - Sum elements using each()"
puts "  * filter_array_even()   - Filter to new array"
puts "  * vec_to_array()        - Convert Vec<i64> -> RArray"
puts "  * array_to_vec()        - Convert RArray -> Vec<i64>"
puts "  * map_array_double()    - Transform elements (map pattern)"
puts
puts "PART 2: Working with Hashes"
puts "  * build_hash()          - Create and populate with insert()"
puts "  * iterate_hash_entries()- Iterate key-value pairs"
puts "  * hashmap_to_rhash()    - Convert HashMap -> RHash"
puts "  * rhash_to_hashmap()    - Convert RHash -> HashMap"
puts "  * filter_hash_by_value()- Filter to new hash"
puts
puts "PART 3: Combining Arrays and Hashes"
puts "  * array_of_hashes()     - Store hashes in an array"
puts "  * hash_with_array_values() - Store arrays as hash values"
puts "  * group_by_length()     - Group array items by computed key"
puts "  * flatten_hash_arrays() - Flatten nested arrays"
puts
puts "PART 4: Round-trip Conversions"
puts "  * roundtrip_vec()       - Vec -> RArray -> Vec"
puts "  * roundtrip_hashmap()   - HashMap -> RHash -> HashMap"
puts
puts "=== Key Concepts ==="
puts
puts "Iteration:"
puts "  * Use each() with closures for safe iteration"
puts "  * Return Ok(()) to continue, Err to stop early"
puts "  * Closures receive Values that need type conversion"
puts
puts "Building Collections:"
puts "  * RArray::new(), RArray::with_capacity(n)"
puts "  * RHash::new()"
puts "  * Use push() for arrays, insert() for hashes"
puts
puts "Rust <-> Ruby Conversions:"
puts "  * Vec<T>: from_slice(&[T]) / to_vec::<T>()"
puts "  * HashMap<K,V>: from_hash_map(map) / to_hash_map::<K,V>()"
puts "  * Conversions are type-safe with TryConvert"
puts
puts "Run 'cargo test' in this directory to run the Rust unit tests."
puts "Full Ruby integration testing requires Phase 3 (method registration)."
puts
puts "=== Success ==="
