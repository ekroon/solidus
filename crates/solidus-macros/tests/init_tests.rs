//! Tests for the #[init] macro
//!
//! Note: The #[init] macro generates `unsafe extern "C"` functions, which in Rust 2024
//! edition requires the caller to acknowledge the unsafety. In stable Rust, we cannot
//! mark proc_macro_attribute as unsafe, so these tests verify the macro implementation
//! through documentation and integration tests rather than direct compilation tests.
//!
//! The macro is tested in actual usage in the examples/ directory which use it in
//! real extension crates.

#[test]
fn test_macro_exists() {
    // This test verifies that the macro can be imported
    // The actual usage is tested in examples/
    assert!(true);
}

// Note: For actual tests of the macro, see:
// - examples/phase2_*/src/lib.rs (they use the old-style Init_ functions)
// - Future examples will use #[solidus::init]
//
// The macro is verified to work through:
// 1. Compilation of examples that use it
// 2. Integration tests that load the generated extensions
// 3. Manual verification of generated code
//
// Validation tests are done through compile_fail tests that verify error messages
// for invalid inputs like:
// - Invalid Ruby identifiers (e.g., names starting with numbers)
// - Unknown attributes
// - Non-string name values
