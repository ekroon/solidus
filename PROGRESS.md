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
| 4 | [TypedData](docs/plan/phase-4-typed-data.md) | :hourglass: Pending | |
| 5 | [Polish](docs/plan/phase-5-polish.md) | :hourglass: Pending | |
| 6 | [Safety Validation](docs/plan/phase-6-safety-validation.md) | :hourglass: Pending | |

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

All core acceptance criteria met:
- `method!` and `function!` macros work for arities 0-4 (extensible to 15)
- `#[solidus_macros::method]` and `#[solidus_macros::function]` attribute macros with implicit pinning
- Method definition APIs available on Module trait and Ruby handle
- `#[solidus::init]` generates correct init functions
- Panic handling and error propagation work correctly
- Comprehensive test coverage (192+ tests pass with Ruby)
- Complete phase3_methods example demonstrating all features

## Design Change: Pinned-From-Creation (ADR-007)

**Date**: 2025-12-16

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

### Solution (ADR-007)

**All Ruby VALUEs must be pinned from the moment of creation in Rust.**

This requires:
1. VALUE types (`RString`, `RArray`, etc.) must be `!Copy`
2. Creation functions must return types that enforce immediate pinning
3. `BoxValue<T>` is the only way to store VALUEs on the heap
4. Return values must remain pinned until returned to Ruby

### Impact on Current Implementation

The following changes are needed:
- **Phase 2 (Types)**: Remove `Copy` from all VALUE wrapper types
- **Phase 3 (Methods)**: Reconsider/remove implicit pinning feature
- **New work**: Redesign creation APIs to enforce immediate pinning

This is a **breaking change** from the current implementation. The existing code
compiles but has the safety gap described above.

### Status

Design accepted. Implementation pending. See [decisions.md](docs/plan/decisions.md)
for the full ADR.

<!-- Add any relevant notes about progress, blockers, or decisions here -->

