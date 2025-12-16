# Agent Instructions for Solidus

## Project Overview

Solidus is a Rust library for writing Ruby extensions, designed as an alternative to Magnus.
The key innovation is **automatic stack pinning** of Ruby values, eliminating the need for
users to manually ensure values stay on the stack.

## Design Principles

1. **Safety by default**: Ruby values are stack-pinned via `Pin<&StackPinned<T>>` in method
   signatures. Users cannot accidentally move values to the heap.

2. **Explicit heap allocation**: When heap storage is needed, users explicitly use `BoxValue<T>`,
   which registers with Ruby's GC.

3. **Zero-cost abstractions**: Pinning has no runtime overhead - it's purely a compile-time
   guarantee.

4. **Immediate values bypass pinning**: `Fixnum`, `Symbol`, `true`, `false`, `nil` don't need
   GC protection and can be passed directly without pinning.

5. **Clear error messages**: Prefer compile-time errors over runtime panics.

## Build/Test/Lint Commands

- **Build**: `cargo build`
- **Build all**: `cargo build --workspace`
- **Test Rust**: `cargo test --workspace`
- **Test with Ruby**: `cargo test --workspace --features embed`
- **Run single test**: `cargo test -p solidus test_name`
- **Lint**: `cargo fmt --check && cargo clippy --workspace`
- **Format**: `cargo fmt --all`
- **Check docs**: `cargo doc --workspace --no-deps`

## Code Style

- **Rust Edition**: 2024
- **MSRV**: Latest stable (currently 1.83+)
- **Formatting**: `rustfmt` defaults
- **Linting**: `clippy` with default settings, treat warnings as errors in CI
- **Imports**: Group std → external crates → local modules, separated by blank lines
- **Naming**: 
  - `snake_case` for functions/variables
  - `CamelCase` for types
  - `SCREAMING_SNAKE_CASE` for constants
  - Prefix Ruby wrapper types with `R` (e.g., `RString`, `RArray`)
- **Error handling**: Return `Result<T, Error>`, never panic in library code
- **Safety**: Document all `unsafe` blocks with `// SAFETY:` comments
- **Documentation**: All public items must have doc comments

## Crate Structure

- **solidus** (`crates/solidus/`): Main library crate
- **solidus-macros** (`crates/solidus-macros/`): Proc-macro crate for `#[init]`, `#[wrap]`, etc.

## Key Types

| Type | Purpose |
|------|---------|
| `Value` | Raw Ruby VALUE wrapper |
| `StackPinned<T>` | `!Unpin` wrapper for stack pinning |
| `BoxValue<T>` | Heap-allocated, GC-registered wrapper |
| `Ruby` | Handle to Ruby API (not `Copy`, passed by reference) |
| `Error` | Ruby exception wrapper |

## Method Signature Patterns

```rust
// Method with pinned argument (most common)
fn example(rb_self: RString, arg: Pin<&StackPinned<RString>>) -> Result<RString, Error>

// Method with immediate value (no pinning needed)  
fn example(rb_self: RString, count: i64) -> Result<RString, Error>

// Method with mixed arguments
fn example(rb_self: RString, count: i64, arg: Pin<&StackPinned<RString>>) -> Result<RString, Error>

// Function (no self)
fn example(arg: Pin<&StackPinned<RString>>) -> Result<RString, Error>
```

## Testing Strategy

1. **Unit tests**: In `#[cfg(test)]` modules, test Rust logic without Ruby
2. **Integration tests**: In `tests/`, use `rb-sys-test-helpers` with `#[ruby_test]`
3. **Ruby scripts**: In `tests/ruby/`, exercise extensions from Ruby side
4. **Examples**: In `examples/`, complete working extension gems
5. **Safety validation**: Tests that confirm we prevent the undefined behavior Magnus allows

## Working with This Codebase

### Adding a New Ruby Type

1. Create file in `crates/solidus/src/types/`
2. Implement `ReprValue`, `TryConvert`, `IntoValue`
3. Add `from_value()` constructor
4. Add type-specific methods
5. Re-export from `crates/solidus/src/types/mod.rs`
6. Add tests

### Modifying the Method Macro

1. Changes to `method!` are in `crates/solidus/src/method/`
2. The macro generates `extern "C"` wrapper functions
3. Test with various arities (0-15) and argument combinations
4. Ensure pinning logic is correct for each argument position

### Adding Examples

1. Create directory in `examples/`
2. Include `Cargo.toml`, `src/lib.rs`, and test Ruby script
3. Update workspace `Cargo.toml` to exclude from workspace (examples build separately)
4. Document in `examples/README.md`

## Progress Tracking

**Check `PROGRESS.md` for current phase completion status.** This file tracks which
implementation phases are complete, in progress, or pending. Always consult it before
starting work to understand the current state without needing to analyze all phase files.

- Update `PROGRESS.md` when completing a phase
- Individual phase details are in `docs/plan/phase-*.md`
- **Task breakdowns** are in `docs/plan/phase-*-tasks.md` files
- See `PLAN.md` for the overall implementation plan

## Task Files

Complex phases have separate task files with step-by-step implementation guides:

| Phase | Task File | Description |
|-------|-----------|-------------|
| 2 | [phase-2-tasks.md](docs/plan/phase-2-tasks.md) | Ruby types implementation in 9 stages |

Task files contain:
- Ordered tasks with dependencies
- Code snippets and API designs
- Ruby C API function references
- Acceptance criteria for each stage

When working on a phase, **always check for a corresponding task file** and follow
the implementation order specified there.

## References

- [Magnus source](https://github.com/matsadler/magnus) - Original inspiration
- [rb-sys](https://github.com/oxidize-rb/rb-sys) - Low-level Ruby bindings
- [Ruby C API docs](https://ruby-doc.org/core/doc/extension_rdoc.html)
- [Pin documentation](https://doc.rust-lang.org/std/pin/)
