# Phase 5: Polish

## Objective

Complete documentation, add comprehensive examples, and ensure thorough testing.

## Dependencies

- Phases 1-4 complete

## Tasks

### 5.1 Documentation

#### API Documentation

- [ ] All public types have doc comments
- [ ] All public functions have doc comments with examples
- [ ] Module-level documentation explains concepts
- [ ] `#![doc]` attribute on lib.rs with overview

#### Guide Documentation

Create `docs/guide/`:

- [ ] `getting-started.md` - First extension walkthrough
- [ ] `pinning.md` - Explanation of pinning and why it matters
- [ ] `types.md` - Working with Ruby types
- [ ] `methods.md` - Defining methods
- [ ] `typed-data.md` - Wrapping Rust types
- [ ] `error-handling.md` - Error patterns
- [ ] `boxvalue.md` - When and how to use BoxValue

### 5.2 Examples

Create complete, working examples in `examples/`:

#### hello_world

Minimal extension demonstrating:
- Basic project structure
- `#[solidus::init]`
- Single method definition

- [ ] `examples/hello_world/Cargo.toml`
- [ ] `examples/hello_world/src/lib.rs`
- [ ] `examples/hello_world/test.rb`
- [ ] `examples/hello_world/README.md`

#### pinned_values

Demonstrates pinning concepts:
- Methods with pinned arguments
- Converting to BoxValue
- Storing values in collections

- [ ] Create example
- [ ] Add README explaining concepts

#### typed_data

Demonstrates wrapping Rust types:
- Simple struct wrapping
- Mutable types with RefCell
- Types with GC marking

- [ ] Create example
- [ ] Add README

#### collections

Working with Ruby arrays and hashes:
- Iterating collections
- Building collections
- Converting between Rust and Ruby collections

- [ ] Create example
- [ ] Add README

### 5.3 Testing

#### Unit Tests

- [ ] All types have unit tests
- [ ] All traits have tests for implementations
- [ ] Edge cases covered

#### Integration Tests

Using `rb-sys-test-helpers`:

```rust
// tests/integration/string_test.rs
use rb_sys_test_helpers::ruby_test;

#[ruby_test]
fn test_string_creation() {
    let ruby = unsafe { Ruby::get() };
    let s = RString::new("hello").unwrap();
    assert_eq!(s.to_string().unwrap(), "hello");
}
```

- [ ] Set up `rb-sys-test-helpers` dependency
- [ ] Tests for each type
- [ ] Tests for method registration
- [ ] Tests for TypedData

#### Ruby Integration Tests

Scripts in `tests/ruby/`:

```ruby
# tests/ruby/test_string_methods.rb
require 'solidus_test_extension'

class TestStringMethods < Minitest::Test
  def test_concat
    assert_equal "helloworld", "hello".concat_pinned("world")
  end
end
```

- [ ] Create test extension crate
- [ ] Write Ruby test scripts
- [ ] Add to CI

### 5.4 CI Enhancements

- [ ] Test on multiple Ruby versions (3.4.x)
- [ ] Test on Linux, macOS, Windows
- [ ] Run examples as tests
- [ ] Run Ruby integration tests
- [ ] Code coverage reporting (optional)

### 5.5 README

Complete README.md with:

- [ ] Project description and motivation
- [ ] Quick start example
- [ ] Comparison with Magnus
- [ ] Installation instructions
- [ ] Links to documentation
- [ ] Contributing section
- [ ] License information

### 5.6 Final Review

- [ ] Run `cargo doc --open` and review all docs
- [ ] Review all examples compile and run
- [ ] Review all tests pass
- [ ] Check for any `TODO` or `FIXME` comments
- [ ] Review error messages are helpful
- [ ] Performance sanity check

## Acceptance Criteria

- [ ] All public APIs documented
- [ ] Guide covers all major concepts
- [ ] Examples are complete and working
- [ ] Test coverage is comprehensive
- [ ] CI passes on all platforms
- [ ] README is complete and accurate
