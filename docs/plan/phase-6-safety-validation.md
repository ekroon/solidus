# Phase 6: Safety Validation

## Objective

Create tests that demonstrate Solidus prevents the undefined behavior that Magnus allows.
These tests serve as both validation and documentation of the safety guarantees.

## Dependencies

- Phases 1-5 complete (feature complete)

## Background

### The Problem with Magnus

In Magnus, this code compiles and runs but is **undefined behavior**:

```rust
fn bad_example(ruby: &Ruby) {
    // UB: Ruby value moved to heap, invisible to GC
    let values: Vec<Value> = vec![ruby.str_new("hello")];
    
    // If GC runs here, the string might be collected
    ruby.gc_start();
    
    // Accessing values[0] is now UB - it might point to freed memory
    println!("{}", values[0]);
}
```

Magnus documents this limitation but cannot prevent it at compile time.

### How Solidus Prevents This

Solidus uses `Pin<&StackPinned<T>>` to prevent heap allocation:

```rust
fn solidus_example(arg: Pin<&StackPinned<RString>>) {
    // This won't compile - StackPinned<T> is !Unpin
    // let values: Vec<Pin<&StackPinned<RString>>> = vec![arg];
    
    // To store on heap, you must explicitly use BoxValue
    let boxed = BoxValue::new(*arg.get());  // Explicit, GC-registered
}
```

## Tasks

### 6.1 Compile-Fail Tests

Create tests that verify certain patterns don't compile.

Using `trybuild` or `compiletest-rs`:

```rust
// tests/compile_fail/heap_storage.rs
// This should fail to compile

use solidus::{RString, StackPinned};
use std::pin::Pin;

fn store_pinned(arg: Pin<&StackPinned<RString>>) {
    let vec: Vec<_> = vec![arg];  // Should fail: can't store Pin<&StackPinned<T>>
}
```

- [ ] Set up compile-fail test infrastructure
- [ ] Test: Cannot store `Pin<&StackPinned<T>>` in Vec
- [ ] Test: Cannot store `Pin<&StackPinned<T>>` in HashMap
- [ ] Test: Cannot store `Pin<&StackPinned<T>>` in Box
- [ ] Test: Cannot return `Pin<&StackPinned<T>>` from function
- [ ] Test: Cannot move `StackPinned<T>` after pinning

### 6.2 Runtime Safety Tests

Tests that verify GC interaction is safe:

```rust
#[ruby_test]
fn test_gc_safety_with_boxvalue() {
    let ruby = unsafe { Ruby::get() };
    
    // Create many strings to trigger GC
    let stored: Vec<BoxValue<RString>> = (0..1000)
        .map(|i| BoxValue::new(RString::new(&format!("string_{}", i)).unwrap()))
        .collect();
    
    // Force GC
    ruby.gc_start();
    
    // All values should still be valid
    for (i, s) in stored.iter().enumerate() {
        assert_eq!(s.to_string().unwrap(), format!("string_{}", i));
    }
}
```

- [ ] Test: BoxValue survives GC
- [ ] Test: Multiple BoxValues survive GC
- [ ] Test: BoxValue in nested data structures survives GC
- [ ] Test: Dropping BoxValue doesn't cause issues

### 6.3 Stress Tests

Tests that stress the GC to find edge cases:

```rust
#[ruby_test]
fn test_gc_stress() {
    let ruby = unsafe { Ruby::get() };
    
    for _ in 0..100 {
        let stored: Vec<BoxValue<RString>> = (0..100)
            .map(|i| BoxValue::new(RString::new(&format!("s{}", i)).unwrap()))
            .collect();
        
        // Rapid GC cycles
        for _ in 0..10 {
            ruby.gc_start();
        }
        
        // Verify all values
        for (i, s) in stored.iter().enumerate() {
            assert_eq!(s.to_string().unwrap(), format!("s{}", i));
        }
    }
}
```

- [ ] GC stress test with many allocations
- [ ] Concurrent-like stress test (if applicable)
- [ ] Long-running stability test

### 6.4 Comparison Tests

Side-by-side comparison with Magnus (documentation purposes):

```rust
// tests/comparison/magnus_vs_solidus.rs

/// This test documents the difference between Magnus and Solidus.
/// 
/// In Magnus, you could write:
/// ```ignore
/// fn magnus_example(s: RString) {
///     let vec = vec![s];  // Compiles but is UB
/// }
/// ```
/// 
/// In Solidus, the equivalent is a compile error:
/// ```compile_fail
/// fn solidus_example(s: Pin<&StackPinned<RString>>) {
///     let vec = vec![s];  // Does not compile
/// }
/// ```
/// 
/// To store values, Solidus requires explicit BoxValue:
/// ```
/// fn solidus_safe(s: Pin<&StackPinned<RString>>) {
///     let boxed = BoxValue::new(*s.get());
///     let vec = vec![boxed];  // Safe: GC-registered
/// }
/// ```
#[test]
fn document_safety_difference() {
    // This test exists for documentation
}
```

- [ ] Document Magnus UB patterns
- [ ] Show corresponding Solidus safe patterns
- [ ] Add to documentation

### 6.5 Edge Case Tests

Test boundary conditions:

- [ ] Empty BoxValue (if applicable)
- [ ] Very large values
- [ ] Deeply nested structures with BoxValue
- [ ] BoxValue containing other BoxValues
- [ ] Circular references (if possible)

### 6.6 Memory Leak Tests

Verify no memory leaks:

```rust
#[ruby_test]
fn test_no_memory_leak() {
    let ruby = unsafe { Ruby::get() };
    
    let initial_count = ruby.gc_count();
    
    for _ in 0..1000 {
        let _boxed = BoxValue::new(RString::new("test").unwrap());
        // BoxValue dropped here, should unregister from GC
    }
    
    ruby.gc_start();
    
    // Memory should be reclaimed
    // (This is a simplified test - real implementation might use
    // Ruby's ObjectSpace or similar for verification)
}
```

- [ ] Test BoxValue cleanup on drop
- [ ] Test nested BoxValue cleanup
- [ ] Test cleanup during panic

## Test Infrastructure

### 6.7 trybuild Setup

```toml
# Cargo.toml
[dev-dependencies]
trybuild = "1.0"
```

```rust
// tests/compile_tests.rs
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
```

- [ ] Add trybuild dependency
- [ ] Create test runner
- [ ] Organize compile-fail tests

### 6.8 Documentation Integration

- [ ] Add safety validation section to guide
- [ ] Reference tests in API documentation
- [ ] Create "Why Solidus?" document with examples

## Acceptance Criteria

- [ ] All compile-fail tests correctly reject unsafe patterns
- [ ] All runtime safety tests pass
- [ ] Stress tests don't reveal any issues
- [ ] No memory leaks detected
- [ ] Clear documentation of safety guarantees
- [ ] Comparison with Magnus is well-documented
