#!/usr/bin/env ruby
require_relative 'target/debug/phase3_methods'

puts "Testing hello() - arity 0:"
result = hello()
puts "  Result: #{result.inspect}"

puts "\nTesting repeat_string('Hi!') - arity 1:"
begin
  result = repeat_string("Hi!")
  puts "  Result: #{result.inspect}"
rescue => e
  puts "  ERROR: #{e.class}: #{e.message}"
  puts e.backtrace.first(5)
end
