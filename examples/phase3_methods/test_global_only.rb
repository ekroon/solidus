#!/usr/bin/env ruby
require_relative 'target/debug/phase3_methods'

puts "Testing Global Functions Only"
puts "=" * 70

# Test arity 0
result = hello()
puts "hello() => #{result.inspect}"
raise "Expected 'Hello from Solidus!'" unless result == "Hello from Solidus!"

# Test arity 1
result = repeat_string("Hi!")
puts "repeat_string('Hi!') => #{result.inspect}"
raise "Expected 'Hi!Hi!Hi!'" unless result == "Hi!Hi!Hi!"

# Test arity 2
result = add_numbers("10", "20")
puts "add_numbers('10', '20') => #{result.inspect}"
raise "Expected 30" unless result == 30

# Test arity 3
result = average_three("10", "20", "30")
puts "average_three('10', '20', '30') => #{result.inspect}"
raise "Expected '20.0'" unless result == "20.0"

puts "=" * 70
puts "ALL GLOBAL FUNCTION TESTS PASSED!"
