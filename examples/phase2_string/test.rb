#!/usr/bin/env ruby
# frozen_string_literal: true

# Test script for Phase 2 Stage 4: String Type Example
#
# This stage implements the RString type with encoding support.
# Full integration testing from Ruby requires Phase 3 (method definition).
# This script verifies that the extension builds successfully.

puts "=== Phase 2 Stage 4: String Type Example ==="
puts

# Build the extension
puts "Building phase2_string extension..."
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
             'libphase2_string.dylib'
           when /linux/
             'libphase2_string.so'
           when /mingw|mswin/
             'phase2_string.dll'
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
puts "This example demonstrates Ruby String type at the Rust level."
puts
puts "Implemented features:"
puts "• RString creation from &str and byte slices"
puts "• String properties: len(), is_empty()"
puts "• Content access: to_string(), to_bytes()"
puts "• Type-safe conversions between Rust and Ruby strings"
puts "• Encoding support with the Encoding type"
puts "• UTF-8 and binary (non-UTF-8) data handling"
puts "• String encoding information and conversion"
puts "• Null byte handling (unlike C strings)"
puts
puts "Key features:"
puts "• RString is heap-allocated and requires GC protection"
puts "• Safe conversion to Rust String (validates UTF-8)"
puts "• Always-safe conversion to Vec<u8> (works with any bytes)"
puts "• Encoding-aware string operations"
puts "• Compile-time type safety"
puts
puts "Encodings demonstrated:"
puts "• UTF-8 - Unicode text encoding"
puts "• ASCII-8BIT - Binary byte sequences"
puts "• US-ASCII - 7-bit ASCII only"
puts "• Dynamic encoding lookup by name"
puts
puts "Example functions:"
puts "• example_string_from_str() - Create from &str"
puts "• example_empty_string() - Empty string handling"
puts "• example_string_from_bytes() - Create from byte slice"
puts "• example_utf8_string() - UTF-8 Unicode support"
puts "• example_binary_string() - Non-UTF-8 binary data"
puts "• example_string_encoding() - Get encoding info"
puts "• example_encoding_conversion() - Convert encodings"
puts "• example_string_conversions() - Rust ↔ Ruby conversions"
puts "• example_string_with_nulls() - Null byte handling"
puts "• example_string_roundtrip() - Safe round-trip conversion"
puts "• example_find_encoding() - Look up encodings by name"
puts "• example_string_concatenation() - Type-safe operations"
puts
puts "Run 'cargo test' in this directory to run the Rust unit tests."
puts "Full Ruby integration testing requires Phase 3 (method registration)."
puts
puts "=== Next Steps ==="
puts
puts "Stage 5 will implement Array type with iteration support."
puts "Stage 6+ will add Hash, Class, and Module types."
