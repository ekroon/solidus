#!/usr/bin/env ruby
# frozen_string_literal: true

# Phase 3 Methods - Comprehensive Test Script
#
# This script tests all the different method registration patterns
# demonstrated in the phase3_methods example.

require_relative 'target/debug/phase3_methods'

puts "=" * 70
puts "Phase 3 Methods - Comprehensive Method Registration Test"
puts "=" * 70
puts

# ============================================================================
# Test Global Functions
# ============================================================================

puts "Testing Global Functions"
puts "-" * 70

# Arity 0
result = hello()
puts "hello() => #{result.inspect}"
raise "Expected 'Hello from Solidus!'" unless result == "Hello from Solidus!"

# Arity 1
result = repeat_string("Hi!")
puts "repeat_string('Hi!') => #{result.inspect}"
raise "Expected 'Hi!Hi!Hi!'" unless result == "Hi!Hi!Hi!"

# Arity 2
result = add_numbers("10", "20")
puts "add_numbers('10', '20') => #{result.inspect}"
raise "Expected 30" unless result == 30

# Arity 3
result = average_three("10", "20", "30")
puts "average_three('10', '20', '30') => #{result.inspect}"
raise "Expected '20.0'" unless result == "20.0"

puts "✓ All global function tests passed!"
puts

# ============================================================================
# Test Calculator Class Instance Methods
# ============================================================================

puts "Testing Calculator Instance Methods"
puts "-" * 70

# Arity 0 - instance method with self only
calc = "Calculator"
result = calc.greet
puts "\"Calculator\".greet => #{result.inspect}"
raise "Expected 'Hello, Calculator!'" unless result == "Hello, Calculator!"

# Arity 1 - instance method with self + 1 argument
result = "5".add("10")
puts "\"5\".add(\"10\") => #{result.inspect}"
raise "Expected 15" unless result == 15

# Arity 2 - instance method with self + 2 arguments
result = "2".multiply_three("3", "4")
puts "\"2\".multiply_three(\"3\", \"4\") => #{result.inspect}"
raise "Expected 24" unless result == 24

# Test error handling
begin
  "test".always_fails
  raise "Should have raised an error!"
rescue RuntimeError => e
  puts "\"test\".always_fails => RuntimeError: #{e.message}"
  raise "Wrong error message" unless e.message == "This method always fails!"
end

puts "✓ All Calculator instance method tests passed!"
puts

# ============================================================================
# Test Calculator Class Methods
# ============================================================================

puts "Testing Calculator Class Methods (Singleton Methods)"
puts "-" * 70

# Class method with arity 0
result = Calculator.create_default
puts "Calculator.create_default => #{result.inspect}"
raise "Expected 'Calculator'" unless result == "Calculator"

# Class method with arity 1
result = Calculator.create_with_name("Advanced")
puts "Calculator.create_with_name('Advanced') => #{result.inspect}"
raise "Expected 'Calculator: Advanced'" unless result == "Calculator: Advanced"

puts "✓ All Calculator class method tests passed!"
puts

# ============================================================================
# Test StringUtils Module Functions
# ============================================================================

puts "Testing StringUtils Module Functions"
puts "-" * 70

# Module function with arity 0
result = StringUtils.get_version
puts "StringUtils.get_version => #{result.inspect}"
raise "Expected '1.0.0'" unless result == "1.0.0"

# Module function with arity 1
result = StringUtils.to_upper("hello")
puts "StringUtils.to_upper('hello') => #{result.inspect}"
raise "Expected 'HELLO'" unless result == "HELLO"

# Module function with arity 2
result = StringUtils.join_with("foo", "bar")
puts "StringUtils.join_with('foo', 'bar') => #{result.inspect}"
raise "Expected 'foo - bar'" unless result == "foo - bar"

puts "✓ All StringUtils module function tests passed!"
puts

# ============================================================================
# Test SolidusMath Module Class Methods
# ============================================================================

puts "Testing SolidusMath Module Class Methods"
puts "-" * 70

# Singleton method with arity 0
result = SolidusMath.pi
puts "SolidusMath.pi => #{result.inspect}"
raise "Expected '3.14159'" unless result == "3.14159"

# Singleton method with arity 1
result = SolidusMath.double("21")
puts "SolidusMath.double('21') => #{result.inspect}"
raise "Expected 42" unless result == 42

# Singleton method with arity 2
result = SolidusMath.power("2", "8")
puts "SolidusMath.power('2', '8') => #{result.inspect}"
raise "Expected 256" unless result == 256

puts "✓ All SolidusMath class method tests passed!"
puts

# ============================================================================
# Test Mixed Scenarios
# ============================================================================

puts "Testing Mixed Scenarios"
puts "-" * 70

# Combine global function with instance method
name = "World"
greeting = hello()
formatted = name.greet
puts "Combined: hello() + name.greet => #{greeting.inspect}, #{formatted.inspect}"

# Chain module functions
version = StringUtils.get_version
upper_version = StringUtils.to_upper(version)
puts "Chained: StringUtils.get_version |> to_upper => #{upper_version.inspect}"
raise "Expected '1.0.0'" unless upper_version == "1.0.0"

# Use global function result in module function
sum = add_numbers("5", "10")
doubled = SolidusMath.double(sum.to_s)
puts "Chained: add_numbers('5', '10') |> SolidusMath.double => #{doubled.inspect}"
raise "Expected 30" unless doubled == 30

puts "✓ All mixed scenario tests passed!"
puts

# ============================================================================
# Summary
# ============================================================================

puts "=" * 70
puts "ALL TESTS PASSED! ✓"
puts "=" * 70
puts
puts "Summary of tested features:"
puts "  • Global functions (4 tests, arities 0-3)"
puts "  • Instance methods (4 tests, arities 0-2)"
puts "  • Class methods (2 tests, arities 0-1)"
puts "  • Module functions (3 tests, arities 0-2)"
puts "  • Module class methods (3 tests, arities 0-2)"
puts "  • Error handling (1 test)"
puts "  • Mixed scenarios (3 tests)"
puts
puts "Total: 20 successful tests demonstrating the complete"
puts "Phase 3 method registration system!"
puts
