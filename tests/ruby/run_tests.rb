#!/usr/bin/env ruby
# frozen_string_literal: true

# Solidus Ruby Integration Test Runner
#
# This script builds and tests Solidus example extensions from Ruby's perspective.
# It verifies that extensions compile correctly and function as expected when
# loaded into Ruby.
#
# Usage: ruby tests/ruby/run_tests.rb [--verbose] [--example NAME]

require 'fileutils'
require 'optparse'

# Configuration
EXAMPLES_DIR = File.expand_path('../../examples', __dir__)
BUILD_MODE = ENV.fetch('BUILD_MODE', 'debug') # 'debug' or 'release'

# Examples with full Ruby integration tests (have Ruby-callable methods)
FULL_TEST_EXAMPLES = %w[
  hello_world
  phase3_methods
  phase3_attr_macros
  phase4_typed_data
].freeze

# Examples that only need build verification (no Ruby-callable methods yet)
# Note: Some phase2 examples have build issues due to API changes and are excluded
BUILD_ONLY_EXAMPLES = %w[
  phase2_string
  phase2_array
].freeze

# Test results tracking
class TestResults
  attr_reader :passed, :failed, :skipped

  def initialize
    @passed = []
    @failed = []
    @skipped = []
  end

  def pass(name, message = nil)
    @passed << { name: name, message: message }
  end

  def fail(name, message)
    @failed << { name: name, message: message }
  end

  def skip(name, message)
    @skipped << { name: name, message: message }
  end

  def success?
    @failed.empty?
  end

  def summary
    total = @passed.length + @failed.length + @skipped.length
    "#{@passed.length}/#{total} passed, #{@failed.length} failed, #{@skipped.length} skipped"
  end
end

# Determine library extension for the current platform
def library_extension
  case RUBY_PLATFORM
  when /darwin/
    'dylib'
  when /linux/
    'so'
  when /mingw|mswin/
    'dll'
  else
    raise "Unsupported platform: #{RUBY_PLATFORM}"
  end
end

# Determine the loadable extension for Ruby require
def loadable_extension
  case RUBY_PLATFORM
  when /darwin/
    'bundle'
  when /linux/
    'so'
  when /mingw|mswin/
    'dll'
  else
    raise "Unsupported platform: #{RUBY_PLATFORM}"
  end
end

# Build an example extension
def build_example(name, verbose: false)
  example_dir = File.join(EXAMPLES_DIR, name)

  unless File.exist?(File.join(example_dir, 'Cargo.toml'))
    return { success: false, message: "Cargo.toml not found in #{example_dir}" }
  end

  unless File.exist?(File.join(example_dir, 'src', 'lib.rs'))
    return { success: false, message: "src/lib.rs not found in #{example_dir}" }
  end

  build_args = ['cargo', 'build']
  build_args << '--release' if BUILD_MODE == 'release'
  build_args << '--manifest-path'
  build_args << File.join(example_dir, 'Cargo.toml')

  if verbose
    puts "  Building: #{build_args.join(' ')}"
    success = system(*build_args)
  else
    success = system(*build_args, out: File::NULL, err: File::NULL)
  end

  if success
    { success: true }
  else
    { success: false, message: 'cargo build failed' }
  end
end

# Create symlink for Ruby to load the extension
def setup_extension_symlink(name)
  example_dir = File.join(EXAMPLES_DIR, name)
  target_dir = File.join(example_dir, 'target', BUILD_MODE)

  # Library names follow cargo conventions (lib prefix, underscores)
  lib_name = "lib#{name}.#{library_extension}"
  lib_path = File.join(target_dir, lib_name)

  unless File.exist?(lib_path)
    return { success: false, message: "Library not found: #{lib_path}" }
  end

  # Create a symlink with the loadable extension name (e.g., .bundle on macOS)
  # Ruby's require looks for the library without 'lib' prefix
  loadable_name = "#{name}.#{loadable_extension}"
  loadable_path = File.join(target_dir, loadable_name)

  # Remove existing symlink if it exists
  FileUtils.rm_f(loadable_path)

  # Create symlink
  FileUtils.ln_s(lib_name, loadable_path)

  { success: true, path: loadable_path }
end

# Run tests for an example that has full Ruby integration
def run_full_tests(name, verbose: false)
  tests_passed = 0
  tests_failed = 0
  failures = []

  example_dir = File.join(EXAMPLES_DIR, name)
  target_dir = File.join(example_dir, 'target', BUILD_MODE)

  # Add target directory to load path
  $LOAD_PATH.unshift(target_dir) unless $LOAD_PATH.include?(target_dir)

  begin
    require name
  rescue LoadError => e
    return { success: false, message: "Failed to load extension: #{e.message}" }
  end

  case name
  when 'hello_world'
    tests_passed, tests_failed, failures = run_hello_world_tests(verbose: verbose)
  when 'phase3_methods'
    tests_passed, tests_failed, failures = run_phase3_methods_tests(verbose: verbose)
  when 'phase3_attr_macros'
    tests_passed, tests_failed, failures = run_phase3_attr_macros_tests(verbose: verbose)
  when 'phase4_typed_data'
    tests_passed, tests_failed, failures = run_phase4_typed_data_tests(verbose: verbose)
  else
    return { success: false, message: "No tests defined for #{name}" }
  end

  if tests_failed.zero?
    { success: true, message: "#{tests_passed} tests passed" }
  else
    { success: false, message: "#{tests_failed}/#{tests_passed + tests_failed} tests failed: #{failures.join(', ')}" }
  end
end

# Test: hello_world
def run_hello_world_tests(verbose: false)
  passed = 0
  failed = 0
  failures = []

  # Test hello() function
  begin
    result = hello
    if result == 'Hello from Solidus!'
      passed += 1
      puts '    hello() => OK' if verbose
    else
      failed += 1
      failures << "hello() returned #{result.inspect}"
    end
  rescue StandardError => e
    failed += 1
    failures << "hello() raised #{e.class}: #{e.message}"
  end

  [passed, failed, failures]
end

# Test: phase3_methods
def run_phase3_methods_tests(verbose: false)
  passed = 0
  failed = 0
  failures = []

  tests = [
    # Global functions
    ['hello()', -> { hello }, 'Hello from Solidus!'],
    ["repeat_string('Hi!')", -> { repeat_string('Hi!') }, 'Hi!Hi!Hi!'],
    ["add_numbers('10', '20')", -> { add_numbers('10', '20') }, 30],
    ["average_three('10', '20', '30')", -> { average_three('10', '20', '30') }, '20.0'],

    # Calculator class methods
    ['Calculator.create_default', -> { Calculator.create_default }, 'Calculator'],
    ["Calculator.create_with_name('Test')", -> { Calculator.create_with_name('Test') }, 'Calculator: Test'],

    # StringUtils module functions
    ['StringUtils.get_version', -> { StringUtils.get_version }, '1.0.0'],
    ["StringUtils.to_upper('hello')", -> { StringUtils.to_upper('hello') }, 'HELLO'],
    ["StringUtils.join_with('a', 'b')", -> { StringUtils.join_with('a', 'b') }, 'a - b'],

    # SolidusMath module class methods
    ['SolidusMath.pi', -> { SolidusMath.pi }, '3.14159'],
    ["SolidusMath.double('21')", -> { SolidusMath.double('21') }, 42],
    ["SolidusMath.power('2', '8')", -> { SolidusMath.power('2', '8') }, 256]
  ]

  tests.each do |name, test_proc, expected|
    begin
      result = test_proc.call
      if result == expected
        passed += 1
        puts "    #{name} => OK" if verbose
      else
        failed += 1
        failures << "#{name}: expected #{expected.inspect}, got #{result.inspect}"
      end
    rescue StandardError => e
      failed += 1
      failures << "#{name} raised #{e.class}: #{e.message}"
    end
  end

  [passed, failed, failures]
end

# Test: phase3_attr_macros
def run_phase3_attr_macros_tests(verbose: false)
  passed = 0
  failed = 0
  failures = []

  tests = [
    # Global functions
    ['attr_get_greeting()', -> { attr_get_greeting }, 'Hello from attribute macros!'],
    ["attr_greet('World')", -> { attr_greet('World') }, 'Hello, World!'],
    ["attr_join_strings('Hello', 'World')", -> { attr_join_strings('Hello', 'World') }, 'Hello World'],
    ["attr_uppercase_explicit('hello')", -> { attr_uppercase_explicit('hello') }, 'HELLO'],

    # AttrString instance methods
    ["AttrString.new('test').attr_length", -> { AttrString.new('test').attr_length }, 4],
    ["AttrString.new('test').attr_concat('_suffix')", -> { AttrString.new('test').attr_concat('_suffix') }, 'test_suffix'],

    # AttrStringUtils module functions
    ["AttrStringUtils.to_upper('hello')", -> { AttrStringUtils.to_upper('hello') }, 'HELLO'],
    ["AttrStringUtils.reverse('hello')", -> { AttrStringUtils.reverse('hello') }, 'olleh']
  ]

  tests.each do |name, test_proc, expected|
    begin
      result = test_proc.call
      if result == expected
        passed += 1
        puts "    #{name} => OK" if verbose
      else
        failed += 1
        failures << "#{name}: expected #{expected.inspect}, got #{result.inspect}"
      end
    rescue StandardError => e
      failed += 1
      failures << "#{name} raised #{e.class}: #{e.message}"
    end
  end

  [passed, failed, failures]
end

# Test: phase4_typed_data
def run_phase4_typed_data_tests(verbose: false)
  passed = 0
  failed = 0
  failures = []

  tests = [
    # Point tests
    ['Point.new(0, 0).x', -> { Point.new(0.0, 0.0).x }, 0.0],
    ['Point.new(3, 4).y', -> { Point.new(3.0, 4.0).y }, 4.0],
    ['Point distance (3-4-5 triangle)', -> { Point.new(0.0, 0.0).distance(Point.new(3.0, 4.0)) }, 5.0],

    # Counter tests
    ['Counter.new(10).get', -> { Counter.new(10).get }, 10],
    ['Counter increment', -> { c = Counter.new(10); c.increment; c.get }, 11],

    # Container tests
    ['Container.new.len', -> { Container.new.len }, 0],
    ['Container push and len', -> { c = Container.new; c.push('a'); c.push('b'); c.len }, 2],
    ['Container get', -> { c = Container.new; c.push('hello'); c.get(0) }, 'hello']
  ]

  tests.each do |name, test_proc, expected|
    begin
      result = test_proc.call
      if result == expected
        passed += 1
        puts "    #{name} => OK" if verbose
      else
        failed += 1
        failures << "#{name}: expected #{expected.inspect}, got #{result.inspect}"
      end
    rescue StandardError => e
      failed += 1
      failures << "#{name} raised #{e.class}: #{e.message}"
    end
  end

  [passed, failed, failures]
end

# Main test runner
def run_tests(options)
  results = TestResults.new
  if options[:example]
    examples_to_test = [options[:example]]
  else
    examples_to_test = FULL_TEST_EXAMPLES + BUILD_ONLY_EXAMPLES
  end

  puts '=' * 70
  puts 'Solidus Ruby Integration Tests'
  puts '=' * 70
  puts

  examples_to_test.each do |name|
    example_dir = File.join(EXAMPLES_DIR, name)

    unless File.directory?(example_dir)
      results.skip(name, 'directory not found')
      puts "[ SKIP ] #{name} - directory not found"
      next
    end

    unless File.exist?(File.join(example_dir, 'src', 'lib.rs'))
      results.skip(name, 'src/lib.rs not found')
      puts "[ SKIP ] #{name} - src/lib.rs not found"
      next
    end

    print "Building #{name}..."
    build_result = build_example(name, verbose: options[:verbose])

    unless build_result[:success]
      results.fail(name, "build failed: #{build_result[:message]}")
      puts " FAILED"
      puts "         #{build_result[:message]}" if options[:verbose]
      next
    end
    puts ' OK'

    # Setup symlink for loading
    symlink_result = setup_extension_symlink(name)
    unless symlink_result[:success]
      results.fail(name, "symlink failed: #{symlink_result[:message]}")
      puts "  [ FAIL ] symlink: #{symlink_result[:message]}"
      next
    end

    # Determine test mode
    if FULL_TEST_EXAMPLES.include?(name) || options[:example]
      print "Testing #{name}..."
      test_result = run_full_tests(name, verbose: options[:verbose])

      if test_result[:success]
        results.pass(name, test_result[:message])
        puts " PASSED (#{test_result[:message]})"
      else
        results.fail(name, test_result[:message])
        puts " FAILED"
        puts "         #{test_result[:message]}"
      end
    else
      results.pass(name, 'build only')
      puts "  [BUILD] #{name} - build verified"
    end

    puts if options[:verbose]
  end

  # Summary
  puts
  puts '=' * 70
  puts "Results: #{results.summary}"
  puts '=' * 70

  if results.failed.any?
    puts
    puts 'Failed tests:'
    results.failed.each do |f|
      puts "  - #{f[:name]}: #{f[:message]}"
    end
  end

  results.success?
end

# Parse command line options
options = {
  verbose: false,
  example: nil
}

OptionParser.new do |opts|
  opts.banner = 'Usage: run_tests.rb [options]'

  opts.on('-v', '--verbose', 'Show detailed output') do
    options[:verbose] = true
  end

  opts.on('-e', '--example NAME', 'Test only the specified example') do |name|
    options[:example] = name
  end

  opts.on('-h', '--help', 'Show this help') do
    puts opts
    exit
  end
end.parse!

# Run tests and exit with appropriate code
success = run_tests(options)
exit(success ? 0 : 1)
