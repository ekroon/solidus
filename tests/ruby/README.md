# Ruby Integration Tests

This directory contains Ruby integration tests that verify Solidus extensions work correctly from Ruby's perspective.

## Prerequisites

- Ruby (any recent version, e.g., 3.0+)
- Rust toolchain with cargo
- rb-sys compatible Ruby headers (installed via ruby-dev or similar)

## Running Tests

From the project root:

```bash
ruby tests/ruby/run_tests.rb
```

Or from this directory:

```bash
ruby run_tests.rb
```

## What Gets Tested

The test runner builds and tests the following examples:

### Always Tested (have full implementations)
- `hello_world` - Minimal extension with a single function
- `phase3_methods` - Comprehensive method registration
- `phase3_attr_macros` - Attribute macro-based method definitions
- `phase4_typed_data` - Custom Ruby objects with Rust data

### Build-Only Tests (no Ruby-callable methods yet)
- `phase2_string` - String type handling
- `phase2_array` - Array type handling

## Test Modes

The test runner operates in two modes:

1. **Full Test** - Builds the extension, loads it into Ruby, and runs functional tests
2. **Build Only** - Just verifies the extension compiles (for examples without Ruby-callable methods)

## Adding New Tests

To add tests for a new example:

1. Ensure the example has a `src/lib.rs` with an init function
2. Add the example name to the appropriate list in `run_tests.rb`:
   - `FULL_TEST_EXAMPLES` for examples with Ruby-callable methods
   - `BUILD_ONLY_EXAMPLES` for examples that only demonstrate Rust-side features

## Troubleshooting

### Extension fails to load

The test runner creates a symlink from `.dylib`/`.so` to `.bundle` (macOS) or uses `.so` directly (Linux). If loading fails:

1. Check that the build succeeded
2. Verify Ruby can find the extension: `ruby -e "require 'path/to/extension'"`
3. Check for missing symbols with `nm -gU path/to/extension.bundle`

### Build fails

1. Ensure rb-sys is compatible with your Ruby version
2. Try building manually: `cd examples/hello_world && cargo build`
3. Check that Ruby development headers are installed

### Ruby allocator warnings

When loading typed data extensions (like `phase4_typed_data`), Ruby may emit warnings like:

```
warning: undefining the allocator of T_DATA class Point
```

These are expected and harmless - they occur because Solidus defines custom allocators for typed data classes.

## CI Integration

These tests can be run in CI by adding to `.github/workflows/ci.yml`:

```yaml
- name: Run Ruby integration tests
  run: ruby tests/ruby/run_tests.rb
```
