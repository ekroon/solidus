# Solidus Implementation Progress

This file tracks the completion status of each implementation phase.
Update this file when a phase is completed to avoid requiring full analysis.

## Phase Status

| Phase | Name | Status | Completed Date |
|-------|------|--------|----------------|
| 0 | [Bootstrap](docs/plan/phase-0-bootstrap.md) | :white_check_mark: Complete | 2024-12 |
| 1 | [Foundation](docs/plan/phase-1-foundation.md) | :white_check_mark: Complete | 2025-12 |
| 2 | [Types](docs/plan/phase-2-types.md) | :white_check_mark: Complete | 2025-12-16 |
| 3 | [Methods](docs/plan/phase-3-methods.md) | :white_check_mark: Complete | 2025-12-16 |
| 4 | [TypedData](docs/plan/phase-4-typed-data.md) | :white_check_mark: Complete | 2025-12-18 |
| 5 | [Polish](docs/plan/phase-5-polish.md) | :white_check_mark: Complete | 2025-12-19 |
| 6 | [Safety Validation](docs/plan/phase-6-safety-validation.md) | :hourglass: Pending | |
| 7 | [Safety Enforcement](docs/plan/phase-7-safety-enforcement.md) | :white_check_mark: Complete | 2025-12-19 |

## Status Legend

- :white_check_mark: Complete - All tasks and acceptance criteria done
- :construction: In Progress - Currently being worked on
- :hourglass: Pending - Not yet started

## Notes

Phase 1 completed with the following components:
- `Value` - Base wrapper around Ruby's VALUE with type checking helpers
- `StackPinned<T>` - `!Unpin` wrapper enabling compile-time stack pinning guarantees
- `BoxValue<T>` - Heap-allocated wrapper with GC registration
- `ReprValue` trait - Common interface for Ruby value wrappers
- `Ruby` handle - Entry point for Ruby VM access
- `Error` type - Ruby exception handling with lazy class resolution
- `gc` module - GC registration/unregistration utilities
- `pin_on_stack!` macro - Convenient stack pinning

Phase 2 completed with all core Ruby types implemented:
- Stage 1: Conversion Traits (Complete) - `TryConvert` and `IntoValue` traits with comprehensive Rust type support
- Stage 2: Immediate Types (Complete) - `Qnil`, `Qtrue`, `Qfalse`, `Fixnum`, `Symbol`, `Flonum`
- Stage 3: Numeric Types (Complete) - `RBignum`, `Integer`, `RFloat`, `Float` with full conversions
- Stage 4: String Type (Complete) - `RString` with encoding support and `Encoding` type
- Stage 5: Array Type (Complete) - `RArray` with push/pop/entry/store/each and Vec conversions
- Stage 6: Hash Type (Complete) - `RHash` with insert/get/delete/each and HashMap conversions
- Stage 7: Class and Module Types (Complete) - `RClass`, `RModule` with `Module` trait for shared functionality
- Stage 8: Additional Types (Skipped) - Optional types deferred to future phases
- Stage 9: Final Integration (Complete) - All types exported, documented, and tested

All acceptance criteria met:
- All major Ruby types have Rust wrappers
- `TryConvert` and `IntoValue` work for common types (primitives, String, Vec, HashMap)
- Immediate values can be used without pinning
- Heap values require pinning in method signatures
- Comprehensive test coverage (153 tests pass with Ruby, 28 without)

Phase 3 completed with comprehensive method registration:
- Stage 1: Method Infrastructure (Complete) - `MethodArg` and `ReturnValue` traits
- Stage 2: Basic Method Macro (Complete) - `method!` macro for arities 0-4 with explicit Pin signatures
- Stage 3: Ergonomic Method Macro (Complete) - `#[solidus_macros::method]` and `#[solidus_macros::function]` attribute macros with implicit pinning for simpler function signatures. Copy bound enforcement ensures type safety. Supports arities 0-2 with pattern for extension.
- Stage 4: Function Macro (Complete) - `function!` macro for arities 0-4 without self parameter
- Stage 5: Method Definition API (Complete) - `define_method`, `define_singleton_method`, `define_module_function` for Module trait, `define_global_function` for Ruby
- Stage 6: Init Macro (Complete) - `#[solidus::init]` attribute macro with automatic crate name detection, custom naming, panic handling, and comprehensive validation
- Stage 7: Variadic Arguments (Deferred) - Support for Ruby variadic methods deferred to future work
- Stage 8: Block Arguments (Deferred) - Support for Ruby blocks deferred to future work
- Stage 9: Keyword Arguments (Deferred) - Support for Ruby keyword arguments deferred to future work
- Stage 10: Integration and Polish (Complete) - Examples, documentation, and testing complete
- **Stage 11: ADR-007 Implementation (Complete)** - All VALUE types are `!Copy`, `NewValue<T>` enforces pinning from creation, methods use `&self` signatures

All core acceptance criteria met:
- `method!` and `function!` macros work for arities 0-4 (extensible to 15)
- `#[solidus_macros::method]` and `#[solidus_macros::function]` attribute macros with implicit pinning
- Method definition APIs available on Module trait and Ruby handle
- `#[solidus::init]` generates correct init functions
- Panic handling and error propagation work correctly
- **All VALUE types are `!Copy` (ADR-007)**: Prevents accidental heap escape
- **NewValue pattern enforces safe creation**: All new values must be pinned or boxed
- **Methods use `&self` instead of `self`**: Prevents moves of `!Copy` types
- Comprehensive test coverage (192+ tests pass with Ruby)
- Complete phase3_methods example demonstrating all features

Phase 4 completed with full TypedData support:
- `TypedData` trait for marking types that can be wrapped
- `DataType` and `DataTypeBuilder` for GC integration
- `wrap()`, `get()`, and `get_mut()` functions for wrapping/unwrapping
- `DataTypeFunctions` trait for advanced GC callbacks (mark, compact, size)
- `Marker` and `Compactor` types for GC callback helpers
- `#[solidus::wrap]` attribute macro for automatic TypedData implementation
- Comprehensive test coverage (6 tests for typed_data module, plus phase4_typed_data example)
- All public items have doc comments
- Full integration with the pinned-from-creation safety model

All acceptance criteria met:
- Rust types can be wrapped as Ruby objects with proper GC integration
- TypedData trait provides simple marker interface
- DataTypeBuilder allows configuration of GC behavior
- Advanced GC callbacks (mark, compact, size) supported via DataTypeFunctions
- `#[solidus::wrap]` macro generates boilerplate automatically
- Comprehensive test coverage and documentation
- Complete phase4_typed_data example demonstrating all features

## Design Change: Pinned-From-Creation (ADR-007) - ✅ COMPLETE

**Date**: 2025-12-16 (Completed: 2025-12-17)

A critical design flaw was identified in the implicit pinning approach used by the
`#[solidus_macros::method]` and `#[solidus_macros::function]` attribute macros.

### Problem

The implicit pinning feature relied on VALUE types being `Copy`. The macro would:
1. Pin the original VALUE on the wrapper's stack
2. Copy the VALUE to pass to the user function

However, since the user receives a `Copy` of the VALUE, they can store that copy
anywhere (Vec, Box, etc.), defeating the pinning protection. The pinned original
doesn't protect the escaped copy.

See: https://github.com/matsadler/magnus/issues/101 for background discussion.

### Solution (ADR-007) - IMPLEMENTED

**All Ruby VALUEs are now pinned from the moment of creation in Rust.**

Implementation changes:
1. ✅ All VALUE types (`RString`, `RArray`, etc.) are now `!Copy`
2. ✅ Creation functions return `NewValue<T>` that enforces immediate pinning or boxing
3. ✅ `BoxValue<T>` is the only way to store VALUEs on the heap (GC-registered)
4. ✅ Methods use `&self` instead of `self` to prevent moves
5. ✅ All tests pass with the new safety model

### API Changes

**Before (unsafe)**:
```rust
let s = RString::new("hello"); // Copy-able, could be stored in Vec
let arr = RArray::new();       // Copy-able, GC-unsafe
```

**After (safe)**:
```rust
// Stack pinning (common case)
let s_guard = RString::new("hello");
let s = s_guard.pin();
pin_on_stack!(s_ref = s);

// Heap boxing (for collections)
let arr_guard = RArray::new();
let arr_boxed = arr_guard.into_box(); // GC-registered
let mut values = vec![arr_boxed];     // Safe!
```

### Benefits

- **Compile-time safety**: Cannot accidentally move VALUEs to heap without GC registration
- **Clear API**: `NewValue` with `#[must_use]` makes requirements explicit
- **Zero runtime cost**: All safety checks are compile-time only
- **Prevents Magnus-style UB**: See https://github.com/matsadler/magnus/issues/101

### Status

✅ **COMPLETE** - All implementation tasks finished. See Stage 10 in phase-2-tasks.md.

Phase 5 completed with comprehensive documentation and examples:
- **Guide Documentation** (Complete):
  - `docs/guide/getting-started.md` - First extension walkthrough
  - `docs/guide/pinning.md` - Explanation of pinning and why it matters
  - `docs/guide/types.md` - Working with Ruby types
  - `docs/guide/methods.md` - Defining Ruby methods
  - `docs/guide/typed-data.md` - Wrapping Rust types as Ruby objects
  - `docs/guide/error-handling.md` - Error handling patterns
  - `docs/guide/boxvalue.md` - When and how to use BoxValue
- **Examples** (Complete):
  - `examples/hello_world/` - Minimal extension demonstrating basic setup
  - `examples/pinned_values/` - Stack pinning and heap boxing patterns
  - `examples/collections/` - Working with RArray and RHash
- **Ruby Integration Tests** (Complete):
  - `tests/ruby/run_tests.rb` - Test runner for all examples
  - `tests/ruby/README.md` - Documentation for running tests
- **CI Enhancements** (Complete):
  - Added `test-examples` job to CI for ubuntu-latest and macos-latest
  - Tests hello_world, pinned_values, phase3_methods, phase3_attr_macros, phase4_typed_data
- **README Enhancement** (Complete):
  - Added Quick Start example
  - Added API Overview section
  - Added Comparison with Magnus section
  - Added Documentation links
  - Added Contributing section

All acceptance criteria met:
- All public APIs documented (lib.rs, all modules have doc comments)
- Guide covers all major concepts (7 guide documents)
- Examples are complete and working (3 new examples + existing phase examples)
- Test coverage is comprehensive (95 tests pass)
- CI passes on all platforms (Linux, macOS, Windows)
- README is complete and accurate

Phase 7 completed with compile-time safety enforcement:
- **Unsafe Constructors** (Complete):
  - All VALUE constructors (`RString::new()`, `RArray::new()`, etc.) are now `unsafe`
  - Forces users to acknowledge safety contract when creating values directly
- **Safe `_boxed` Variants** (Complete):
  - `RString::new_boxed()`, `RArray::new_boxed()`, etc. return `BoxValue<T>`
  - Safe path for heap storage without `unsafe`
- **Updated `pin_on_stack!` Macro** (Complete):
  - Handles `unsafe` internally for ergonomic stack pinning
  - `pin_on_stack!(s = RString::new("hello"))` works without explicit `unsafe`
- **`ReturnWitness` and `WitnessedReturn` Types** (Complete):
  - Optional extra safety layer using lifetime witnesses
  - Prevents storing return values in heap collections
- **Documentation Updated** (Complete):
  - All guides updated for new API
  - All examples updated and working

All acceptance criteria met:
- `NewValue<T>` cannot be stored in heap collections without unsafe
- Safe `_boxed` variants provide heap storage path
- `pin_on_stack!` provides ergonomic stack pinning
- All tests pass (103 doc tests, full test suite)
- All examples work with new API

<!-- Add any relevant notes about progress, blockers, or decisions here -->

