#!/usr/bin/env ruby
# frozen_string_literal: true

# Hello World - Minimal Solidus Example Test
#
# This script builds and tests the simplest possible Solidus extension.

require 'fileutils'

puts "=== Hello World - Minimal Solidus Example ==="
puts

# Build the extension
puts "Building hello_world extension..."
build_result = system("cargo build --manifest-path #{__dir__}/Cargo.toml")

unless build_result
  puts "Build failed!"
  exit(1)
end

puts "Build successful!"
puts

# Create a symlink so Ruby can load the extension
# On macOS, Ruby expects .bundle files, but Rust creates .dylib files
lib_name = case RUBY_PLATFORM
           when /darwin/
             'libhello_world.dylib'
           when /linux/
             'libhello_world.so'
           when /mingw|mswin/
             'hello_world.dll'
           else
             raise "Unsupported platform: #{RUBY_PLATFORM}"
           end

bundle_name = 'hello_world.bundle'
lib_path = File.join(__dir__, 'target', 'debug', lib_name)
bundle_path = File.join(__dir__, 'target', 'debug', bundle_name)

# Create symlink if it doesn't exist or is stale
if !File.exist?(bundle_path) || File.mtime(lib_path) > File.mtime(bundle_path)
  FileUtils.rm_f(bundle_path)
  FileUtils.ln_s(lib_name, bundle_path)
  puts "Created symlink: #{bundle_name} -> #{lib_name}"
end

# Now require the extension
$LOAD_PATH.unshift(File.join(__dir__, 'target', 'debug'))
require 'hello_world'

puts "Testing hello() function..."

result = hello()
puts "hello() => #{result.inspect}"

if result == "Hello from Solidus!"
  puts
  puts "SUCCESS!"
else
  puts
  puts "FAILED: Expected 'Hello from Solidus!', got #{result.inspect}"
  exit(1)
end
