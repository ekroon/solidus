#!/usr/bin/env ruby
# frozen_string_literal: true

# Test script for Phase 2 Stage 1: Conversion Traits Example
#
# Note: This stage only implements the conversion trait infrastructure.
# Full integration testing from Ruby requires Phase 3 (method definition).
# This script simply verifies that the extension builds successfully.

puts "=== Phase 2 Stage 1: Conversion Traits Example ==="
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
puts "This example demonstrates the TryConvert and IntoValue traits at the Rust level."
puts
puts "Key points:"
puts "• Stage 1 implements only identity conversions (Value -> Value)"
puts "• The exported C functions are present but cannot be meaningfully tested"
puts "  from Ruby without proper method registration (Phase 3)"
puts "• Run 'cargo test' in this directory to run the Rust unit tests"
puts "• Later stages will add implementations for specific Ruby types"
puts "  (integers, strings, arrays, etc.)"
puts
puts "=== Next Steps ==="
puts
puts "Stage 2 will implement immediate types (Fixnum, Symbol, etc.) which will"
puts "enable real conversions between Rust and Ruby types."
