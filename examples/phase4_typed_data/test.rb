require_relative 'target/debug/phase4_typed_data'

puts "=" * 60
puts "Testing Point (Task 4.7.1)"
puts "=" * 60

# Test Point creation
p1 = Point.new(0.0, 0.0)
p2 = Point.new(3.0, 4.0)

puts "Point 1: (#{p1.x}, #{p1.y})"
puts "Point 2: (#{p2.x}, #{p2.y})"
puts "Distance: #{p1.distance(p2)}"

# Should print 5.0 (3-4-5 triangle)
raise "Expected 5.0" unless p1.distance(p2) == 5.0

puts "Point tests passed!"
puts

puts "=" * 60
puts "Testing Counter (Task 4.7.2)"
puts "=" * 60

# Test Counter with RefCell mutation
counter = Counter.new(10)
puts "Initial value: #{counter.get}"
raise "Expected 10" unless counter.get == 10

puts "Incrementing..."
result = counter.increment
puts "After increment: #{result}"
raise "Expected 11" unless result == 11

puts "Current value: #{counter.get}"
raise "Expected 11" unless counter.get == 11

counter.increment
counter.increment
puts "After 2 more increments: #{counter.get}"
raise "Expected 13" unless counter.get == 13

puts "Counter tests passed!"
puts

puts "=" * 60
puts "Testing Container (Task 4.7.3)"
puts "=" * 60

# Test Container with GC marking
container = Container.new
puts "Initial length: #{container.len}"
raise "Expected 0" unless container.len == 0

container.push("hello")
container.push(42)
container.push([1, 2, 3])

puts "After adding 3 items: #{container.len}"
raise "Expected 3" unless container.len == 3

puts "Item 0: #{container.get(0).inspect}"
puts "Item 1: #{container.get(1).inspect}"
puts "Item 2: #{container.get(2).inspect}"

raise "Expected 'hello'" unless container.get(0) == "hello"
raise "Expected 42" unless container.get(1) == 42
raise "Expected [1, 2, 3]" unless container.get(2) == [1, 2, 3]

# Test that items survive GC
GC.start
puts "After GC: #{container.len} items"
raise "Expected 3" unless container.len == 3
raise "Expected 'hello' after GC" unless container.get(0) == "hello"

puts "Container tests passed!"
puts

puts "=" * 60
puts "All tests passed!"
puts "=" * 60
