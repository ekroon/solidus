#!/usr/bin/env ruby
# frozen_string_literal: true

# Test script for Phase 2 Stage 3: Numeric Types (Heap) Example
#
# This stage implements heap-allocated numeric types (Bignum, RFloat) and union types
# (Integer, Float) that automatically select the appropriate representation.
#
# Full integration testing from Ruby requires Phase 3 (method definition).
# This script verifies that the extension builds successfully.

puts "=== Phase 2 Stage 3: Numeric Types (Heap) Example ==="
puts

# Build the extension
puts "Building phase2_numeric_heap extension..."
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
             'libphase2_numeric_heap.dylib'
           when /linux/
             'libphase2_numeric_heap.so'
           when /mingw|mswin/
             'phase2_numeric_heap.dll'
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
puts "This example demonstrates heap-allocated numeric types at the Rust level."
puts
puts "Implemented types:"
puts "• RBignum - Large integers that don't fit in Fixnum (heap-allocated)"
puts "• Integer - Union type that automatically selects Fixnum or Bignum"
puts "• RFloat - Heap-allocated floats"
puts "• Float - Union type that automatically selects Flonum or RFloat"
puts
puts "Key features:"
puts "• Automatic selection between immediate and heap representations"
puts "• Integer enum handles both small (Fixnum) and large (Bignum) values"
puts "• Float enum handles both immediate (Flonum) and heap (RFloat) values"
puts "• Range checking for conversions (e.g., negative integers to u64)"
puts "• Platform-aware: Flonum only on 64-bit platforms"
puts
puts "When Ruby uses heap vs immediate:"
puts "• Fixnum: Small integers (roughly ±2^62 on 64-bit platforms)"
puts "• Bignum: Large integers that exceed Fixnum range"
puts "• Flonum: Small floats on 64-bit platforms (immediate value)"
puts "• RFloat: All floats on 32-bit platforms, some floats on 64-bit"
puts
puts "How the union types work:"
puts "• Integer::from_i64/from_u64 automatically creates Fixnum or Bignum"
puts "• Float::from_f64 automatically creates Flonum or RFloat"
puts "• Conversion methods (to_i64, to_f64) work on both variants"
puts "• Users don't need to know which variant is used"
puts
puts "Run 'cargo test' in this directory to run the Rust unit tests."
puts "Full Ruby integration testing requires Phase 3 (method registration)."
puts
puts "=== Example Functions ==="
puts
puts "The extension exports these example functions:"
puts "• example_large_integers - Creating integers that need Bignum"
puts "• example_bignum_explicit - Working with RBignum directly"
puts "• example_integer_auto_selection - Integer enum in action"
puts "• example_u64_range_check - Range checking for unsigned integers"
puts "• example_large_arithmetic - Arithmetic with large values"
puts "• example_rfloat - Creating heap-allocated floats"
puts "• example_float_auto_selection - Float enum in action"
puts "• example_float_conversion - Converting Ruby floats to Rust"
puts "• example_integer_overflow - Handling overflow to smaller types"
puts "• example_float_variants - Flonum vs RFloat on 64-bit"
puts "• example_numeric_round_trip - Verifying conversion fidelity"
puts "• example_negative_numbers - Negative integer and float handling"
puts
puts "=== Next Steps ==="
puts
puts "Stage 4 will implement String type with encoding support."
puts "Stage 5+ will add Array, Hash, Class, and Module types."
