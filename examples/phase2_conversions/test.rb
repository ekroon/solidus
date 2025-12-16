#!/usr/bin/env ruby
# frozen_string_literal: true

# Test script for Phase 2 Stage 2: Immediate Types Example
#
# This stage implements immediate value types (nil, true, false, fixnum, symbol, flonum).
# Full integration testing from Ruby requires Phase 3 (method definition).
# This script verifies that the extension builds successfully.

puts "=== Phase 2 Stage 2: Immediate Types Example ==="
puts

# Build the extension
puts "Building phase2_conversions extension..."
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
             'libphase2_conversions.dylib'
           when /linux/
             'libphase2_conversions.so'
           when /mingw|mswin/
             'phase2_conversions.dll'
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
puts "This example demonstrates immediate types at the Rust level."
puts
puts "Implemented types:"
puts "• Qnil, Qtrue, Qfalse - Ruby's singleton values"
puts "• Fixnum - Small integers (immediate value)"
puts "• Symbol - Interned strings"
puts "• Flonum - Immediate floats (64-bit platforms only)"
puts "• Full conversions for Rust bool, i8-i64, u8-u64, f32, f64, &str"
puts
puts "Key features:"
puts "• Immediate values don't require GC protection or pinning"
puts "• Type-safe conversions with TryConvert and IntoValue traits"
puts "• Symbol interning works correctly"
puts "• Platform-aware Flonum support"
puts
puts "Run 'cargo test' in this directory to run the Rust unit tests."
puts "Full Ruby integration testing requires Phase 3 (method registration)."
puts
puts "=== Next Steps ==="
puts
puts "Stage 3 will implement numeric types (Bignum for large integers, full Float support)."
puts "Stage 4+ will add String, Array, Hash, Class, and Module types."
