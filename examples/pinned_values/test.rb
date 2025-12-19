#!/usr/bin/env ruby
# frozen_string_literal: true

# Pinned Values Example - Test Script
#
# This script demonstrates and tests the pinning concepts in Solidus.
# Run with: ruby test.rb (after building with cargo build)

require_relative 'target/debug/pinned_values'

puts "=" * 70
puts "Pinned Values Example - Demonstrating GC-Safe Ruby Extensions"
puts "=" * 70
puts

# ============================================================================
# Test 1: Stack Pinning with Single Argument
# ============================================================================

puts "Test 1: Stack Pinning - Process String"
puts "-" * 70

result = process_string("hello world")
puts "process_string('hello world') => #{result.inspect}"
raise "Expected 'Processed: HELLO WORLD'" unless result == "Processed: HELLO WORLD"

result = process_string("solidus")
puts "process_string('solidus') => #{result.inspect}"
raise "Expected 'Processed: SOLIDUS'" unless result == "Processed: SOLIDUS"

puts "  Stack pinning ensures the argument stays visible to Ruby's GC"
puts "  while the Rust function processes it."
puts
puts "PASSED"
puts

# ============================================================================
# Test 2: Multiple Pinned Arguments
# ============================================================================

puts "Test 2: Multiple Pinned Arguments - Concatenate"
puts "-" * 70

result = concat_strings("Hello, ", "World!")
puts "concat_strings('Hello, ', 'World!') => #{result.inspect}"
raise "Expected 'Hello, World!'" unless result == "Hello, World!"

result = concat_strings("foo", "bar")
puts "concat_strings('foo', 'bar') => #{result.inspect}"
raise "Expected 'foobar'" unless result == "foobar"

puts "  Each argument is independently pinned on the stack."
puts "  Multiple pinned values are all visible to the GC simultaneously."
puts
puts "PASSED"
puts

# ============================================================================
# Test 3: Instance Method with Pinned Argument
# ============================================================================

puts "Test 3: Instance Method - String#append_solidus"
puts "-" * 70

result = "Hello".append_solidus(" from Solidus!")
puts "'Hello'.append_solidus(' from Solidus!') => #{result.inspect}"
raise "Expected 'Hello from Solidus!'" unless result == "Hello from Solidus!"

result = "Rust".append_solidus(" + Ruby")
puts "'Rust'.append_solidus(' + Ruby') => #{result.inspect}"
raise "Expected 'Rust + Ruby'" unless result == "Rust + Ruby"

puts "  The method! macro handles 'self' specially, while additional"
puts "  arguments use Pin<&StackPinned<T>> for GC safety."
puts
puts "PASSED"
puts

# ============================================================================
# Test 4: BoxValue for Heap Storage
# ============================================================================

puts "Test 4: BoxValue - Heap Storage with GC Registration"
puts "-" * 70

result = box_string("test")
puts "box_string('test') => #{result.inspect}"
raise "Expected 'Boxed: test'" unless result == "Boxed: test"

result = box_string("heap allocated")
puts "box_string('heap allocated') => #{result.inspect}"
raise "Expected 'Boxed: heap allocated'" unless result == "Boxed: heap allocated"

puts "  BoxValue<T> uses rb_gc_register_address() to tell Ruby's GC"
puts "  about heap-stored values. The GC will now scan these locations."
puts
puts "PASSED"
puts

# ============================================================================
# Test 5: Vec<BoxValue<T>> - Collection of Ruby Values
# ============================================================================

puts "Test 5: String Collector - Vec<BoxValue<RString>>"
puts "-" * 70

# Clear any previous state
collector_clear()
puts "collector_clear() - starting fresh"

# Add strings to collection
count = collect_string("apple")
puts "collect_string('apple') => count: #{count}"
raise "Expected count 1" unless count == 1

count = collect_string("banana")
puts "collect_string('banana') => count: #{count}"
raise "Expected count 2" unless count == 2

count = collect_string("cherry")
puts "collect_string('cherry') => count: #{count}"
raise "Expected count 3" unless count == 3

# Get count
count = collector_count()
puts "collector_count() => #{count}"
raise "Expected count 3" unless count == 3

# Join all strings
joined = collector_join(", ")
puts "collector_join(', ') => #{joined.inspect}"
raise "Expected 'apple, banana, cherry'" unless joined == "apple, banana, cherry"

# Convert to Ruby array
arr = collector_to_array()
puts "collector_to_array() => #{arr.inspect}"
raise "Expected ['apple', 'banana', 'cherry']" unless arr == ["apple", "banana", "cherry"]

# Clear and verify
cleared = collector_clear()
puts "collector_clear() => cleared #{cleared} items"
raise "Expected cleared 3" unless cleared == 3

count = collector_count()
puts "collector_count() after clear => #{count}"
raise "Expected count 0" unless count == 0

puts
puts "  Vec<BoxValue<RString>> safely stores Ruby strings on the heap."
puts "  Each BoxValue is registered with the GC, preventing collection."
puts
puts "PASSED"
puts

# ============================================================================
# Test 6: Stack Pinning Demo
# ============================================================================

puts "Test 6: Demo - Multiple Stack-Pinned Values"
puts "-" * 70

result = demo_stack_pinning()
puts "demo_stack_pinning() => #{result.inspect}"
raise "Expected 'Stack pinning keeps values safe!'" unless result == "Stack pinning keeps values safe!"

puts
puts "  Multiple values pinned on the stack are all visible to GC."
puts "  Even if GC runs mid-function, all values remain protected."
puts
puts "PASSED"
puts

# ============================================================================
# Test 7: Heap Boxing Demo
# ============================================================================

puts "Test 7: Demo - Heap Boxing with Vec"
puts "-" * 70

result = demo_heap_boxing()
puts "demo_heap_boxing() => #{result.inspect}"
raise "Expected array of strings" unless result == ["These", "are", "heap", "stored", "safely!"]

puts
puts "  Values stored in Vec<BoxValue<T>> remain valid because each"
puts "  BoxValue is registered with rb_gc_register_address()."
puts "  When the Vec is dropped, rb_gc_unregister_address() is called."
puts
puts "PASSED"
puts

# ============================================================================
# Summary
# ============================================================================

puts "=" * 70
puts "ALL TESTS PASSED!"
puts "=" * 70
puts
puts "Summary of Pinning Concepts Demonstrated:"
puts
puts "  1. STACK PINNING (Pin<&StackPinned<T>>)"
puts "     - Fast: no heap allocation"
puts "     - Safe: values on stack are visible to GC"
puts "     - Use: function arguments, local processing"
puts
puts "  2. HEAP BOXING (BoxValue<T>)"
puts "     - Uses rb_gc_register_address() for GC visibility"
puts "     - Safe: values on heap are registered with GC"
puts "     - Use: Vec, HashMap, struct fields, caching"
puts
puts "  3. THE PROBLEM SOLIDUS SOLVES"
puts "     - Ruby GC only scans the C stack for VALUE references"
puts "     - Values moved to heap without registration can be collected"
puts "     - This causes use-after-free bugs that are hard to debug"
puts
puts "  4. COMPILE-TIME SAFETY"
puts "     - All VALUE types are !Copy (can't accidentally move to heap)"
puts "     - Creation returns PinGuard (must pin or box explicitly)"
puts "     - The compiler enforces what Magnus only documents"
puts
