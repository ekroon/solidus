#!/usr/bin/env ruby
# frozen_string_literal: true

# Phase 3 Attribute Macros - Test Script
#
# This script tests the #[solidus_macros::method] and #[solidus_macros::function]
# attribute macros that provide implicit pinning for method arguments.

require_relative 'target/debug/phase3_attr_macros'

puts "=" * 70
puts "Phase 3 Attribute Macros - Implicit Pinning Test"
puts "=" * 70
puts

# ============================================================================
# Test Global Functions with Implicit Pinning
# ============================================================================

puts "Testing Global Functions (Implicit Pinning)"
puts "-" * 70

# Arity 0
result = attr_get_greeting()
puts "attr_get_greeting() => #{result.inspect}"
raise "Expected 'Hello from attribute macros!'" unless result == "Hello from attribute macros!"

# Arity 1 - implicit pinning
result = attr_greet("World")
puts "attr_greet('World') => #{result.inspect}"
raise "Expected 'Hello, World!'" unless result == "Hello, World!"

# Arity 2 - implicit pinning
result = attr_join_strings("Hello", "World")
puts "attr_join_strings('Hello', 'World') => #{result.inspect}"
raise "Expected 'Hello World'" unless result == "Hello World"

puts "All global function tests passed!"
puts

# ============================================================================
# Test Global Functions with Explicit Pinning
# ============================================================================

puts "Testing Global Functions (Explicit Pinning)"
puts "-" * 70

# Explicit Pin<&StackPinned<T>> signature
result = attr_uppercase_explicit("hello")
puts "attr_uppercase_explicit('hello') => #{result.inspect}"
raise "Expected 'HELLO'" unless result == "HELLO"

puts "All explicit pinning tests passed!"
puts

# ============================================================================
# Test Global Functions with Mixed Pinning
# ============================================================================

puts "Testing Global Functions (Mixed Pinning)"
puts "-" * 70

# Mixed: explicit + implicit
result = attr_format_mixed("explicit", "implicit")
puts "attr_format_mixed('explicit', 'implicit') => #{result.inspect}"
raise "Expected '[explicit] -> [implicit]'" unless result == "[explicit] -> [implicit]"

puts "All mixed pinning tests passed!"
puts

# ============================================================================
# Test AttrString Instance Methods
# ============================================================================

puts "Testing AttrString Instance Methods"
puts "-" * 70

# AttrString inherits from String, so we can create instances directly
s = AttrString.new("test")
puts "Created AttrString: #{s.inspect}"

# Arity 0 - self only
result = s.attr_length
puts "s.attr_length => #{result.inspect}"
raise "Expected 4" unless result == 4

# Arity 1 - implicit pinning
result = s.attr_concat("_suffix")
puts "s.attr_concat('_suffix') => #{result.inspect}"
raise "Expected 'test_suffix'" unless result == "test_suffix"

# Arity 2 - implicit pinning
result = s.attr_surround("<<", ">>")
puts "s.attr_surround('<<', '>>') => #{result.inspect}"
raise "Expected '<<test>>'" unless result == "<<test>>"

# Explicit pinning method
result = s.attr_concat_explicit("_explicit")
puts "s.attr_concat_explicit('_explicit') => #{result.inspect}"
raise "Expected 'test_explicit'" unless result == "test_explicit"

# Mixed pinning method
result = s.attr_combine_mixed("mixed")
puts "s.attr_combine_mixed('mixed') => #{result.inspect}"
raise "Expected 'test+mixed'" unless result == "test+mixed"

puts "All AttrString instance method tests passed!"
puts

# ============================================================================
# Test AttrStringUtils Module Functions
# ============================================================================

puts "Testing AttrStringUtils Module Functions"
puts "-" * 70

# Module function with arity 1
result = AttrStringUtils.to_upper("hello")
puts "AttrStringUtils.to_upper('hello') => #{result.inspect}"
raise "Expected 'HELLO'" unless result == "HELLO"

# Module function - reverse
result = AttrStringUtils.reverse("hello")
puts "AttrStringUtils.reverse('hello') => #{result.inspect}"
raise "Expected 'olleh'" unless result == "olleh"

# Module function with arity 2
result = AttrStringUtils.repeat_join("ab", "-")
puts "AttrStringUtils.repeat_join('ab', '-') => #{result.inspect}"
raise "Expected 'ab-ab-ab'" unless result == "ab-ab-ab"

puts "All AttrStringUtils module function tests passed!"
puts

# ============================================================================
# Comparison: Implicit vs Explicit Pinning
# ============================================================================

puts "Demonstrating Implicit vs Explicit Pinning"
puts "-" * 70
puts
puts "In Rust, these two signatures are equivalent at runtime:"
puts
puts "  // Implicit (simple, recommended)"
puts "  #[solidus_macros::method]"
puts "  fn concat(rb_self: RString, other: RString) -> Result<RString, Error>"
puts
puts "  // Explicit (verbose, still supported)"
puts "  #[solidus_macros::method]"
puts "  fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<RString, Error>"
puts
puts "The macro automatically handles stack pinning for GC safety!"
puts

# ============================================================================
# Summary
# ============================================================================

puts "=" * 70
puts "ALL TESTS PASSED!"
puts "=" * 70
puts
puts "Summary of tested features:"
puts "  - Global functions with implicit pinning (arities 0-2)"
puts "  - Global functions with explicit pinning"
puts "  - Global functions with mixed pinning"
puts "  - Instance methods with implicit pinning (arities 0-2)"
puts "  - Instance methods with explicit pinning"
puts "  - Instance methods with mixed pinning"
puts "  - Module functions with implicit pinning (arities 1-2)"
puts
puts "The #[solidus_macros::method] and #[solidus_macros::function] attribute"
puts "macros successfully provide ergonomic implicit pinning while maintaining"
puts "full backward compatibility with explicit Pin<&StackPinned<T>> signatures."
puts
