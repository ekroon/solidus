# Agent Instructions for Solidus

## Project Overview

Solidus is a Rust library for writing Ruby extensions, designed as an alternative to Magnus.
The key innovation is **pinned-from-creation**: all Ruby VALUES must be pinned on the stack
or explicitly boxed for the heap from the moment they're created, enforced at compile time.

## Design Principles

1. **Safety by default**: All VALUE types are `!Copy`. Creation returns `NewValue<T>` that
   must be pinned on stack or boxed for heap. Compiler enforces what Magnus only documents.

2. **Explicit heap allocation**: When heap storage is needed, users explicitly use `BoxValue<T>`,
   which registers with Ruby's GC. This is the ONLY way to store VALUEs on the heap.

3. **Zero-cost abstractions**: All safety checks are compile-time only. No runtime overhead.

4. **Immediate values optimized**: `Fixnum`, `Symbol`, `true`, `false`, `nil` remain `Copy`
   as they don't need GC protection.

5. **Clear error messages**: Prefer compile-time errors over runtime panics. The type system
   prevents undefined behavior.

## Build/Test/Lint Commands

- **Build**: `cargo build`
- **Build all**: `cargo build --workspace`
- **Test Rust**: `cargo test --workspace` (28 tests, no Ruby required)
- **Test with Ruby (CI)**: `cargo test --workspace --features embed` (requires static Ruby)
- **Test with Ruby (local)**: `cargo test --workspace --features link-ruby` (39 tests, requires dynamic Ruby)
- **Run single test**: `cargo test -p solidus test_name`
- **Lint**: `cargo fmt --check && cargo clippy --workspace`
- **Format**: `cargo fmt --all`
- **Check docs**: `cargo doc --workspace --no-deps`

**Note on Ruby tests**: Unit tests that call Ruby C API functions use `rb-sys-test-helpers` 
with the `#[ruby_test]` macro to properly initialize Ruby. Both `embed` and `link-ruby` 
features work correctly. The Ruby tests are conditional on these features to avoid 
requiring Ruby for basic development.

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

### Git Pre-Commit Hook

**RECOMMENDED**: Install the pre-commit hook to automatically enforce formatting before commits.

To set up the pre-commit hook, run this command from the project root:

```bash
cp pre-commit.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
```

The hook will:
- Check code formatting with `cargo fmt --all -- --check` before allowing commits
- Reject commits if any files have formatting issues
- Provide clear instructions on how to fix formatting issues

If the hook blocks your commit due to formatting issues, simply run:

```bash
cargo fmt --all
git add -u  # Re-stage the formatted files
git commit  # Try again
```

### Pre-Commit Checklist

**CRITICAL**: CI will fail if code is not properly formatted. ALWAYS run `cargo fmt --all` 
before committing any code changes.

**IMPORTANT**: Always run these commands before committing code to avoid CI failures:

1. **Format code FIRST**: `cargo fmt --all` - Automatically fixes formatting issues
   - This MUST be run before committing - CI enforces zero formatting deviations
   - Fixes trailing whitespace, import ordering, line lengths, etc.
2. **Check formatting**: `cargo fmt --all -- --check` - Verify formatting is correct
3. **Run clippy**: `cargo clippy --workspace -- -D warnings` - Check for linting issues
4. **Run tests**: `cargo test --workspace` - Ensure basic tests pass
5. **Run Ruby tests** (if modifying Ruby integration): `cargo test --workspace --features link-ruby`

The CI pipeline enforces strict formatting and linting standards. Running these commands
locally before committing will catch issues early and prevent CI failures.

**Remember**: Step 1 (`cargo fmt --all`) is non-negotiable - formatting violations will 
cause the CI Format job to fail immediately. Using the pre-commit hook (see above) automates 
this check.

## Crate Structure

- **solidus** (`crates/solidus/`): Main library crate
- **solidus-macros** (`crates/solidus-macros/`): Proc-macro crate for `#[init]`, `#[wrap]`, etc.

## Key Types

| Type | Purpose |
|------|---------|
| `Value` | Raw Ruby VALUE wrapper (`!Copy`) |
| `NewValue<T>` | Guard requiring pinning or boxing of new values |
| `StackPinned<T>` | `!Unpin` wrapper for stack pinning |
| `BoxValue<T>` | Heap-allocated, GC-registered wrapper |
| `Ruby` | Handle to Ruby API (not `Copy`, passed by reference) |
| `Error` | Ruby exception wrapper |

## Method Signature Patterns

```rust
// Creating values - always returns NewValue
let guard = RString::new("hello");

// Option 1: Pin on stack (common case)
let pinned = guard.pin();
pin_on_stack!(s = pinned);
// s is Pin<&StackPinned<RString>>

// Option 2: Box for heap (for collections)
let boxed = guard.into_box();
let mut values = vec![boxed];  // Safe!

// Method with pinned argument
fn example(rb_self: RString, arg: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error>

// Method with immediate value (no pinning needed)  
fn example(rb_self: RString, count: i64) -> Result<NewValue<RString>, Error>

// Method with mixed arguments
fn example(rb_self: RString, count: i64, arg: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error>

// Function (no self)
fn example(arg: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error>

// Using &self for methods (all VALUE methods use &self, not self)
impl RString {
    pub fn len(&self) -> usize;
    pub fn to_string(&self) -> Result<String, Error>;
}
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
| 3 | [phase-3-tasks.md](docs/plan/phase-3-tasks.md) | Method registration in 10 stages |

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
