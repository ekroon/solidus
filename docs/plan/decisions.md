# Architecture Decisions

This document records key architectural decisions made during Solidus development.

## ADR-001: Use Pin for Stack Pinning

**Status**: Accepted

**Context**: Ruby's garbage collector scans the C stack to find live VALUE references. Values moved to the heap are invisible to the GC and may be collected while still in use.

**Decision**: Use Rust's `Pin<&StackPinned<T>>` type for method arguments to enforce stack locality at compile time.

**Consequences**:
- Users cannot accidentally move values to the heap
- Explicit `BoxValue<T>` is required for heap allocation
- Method signatures are more verbose but safer
- Zero runtime overhead

## ADR-002: Immediate Values Bypass Pinning

**Status**: Accepted

**Context**: Immediate values (Fixnum, Symbol, true, false, nil) are encoded directly in the VALUE pointer and don't need GC protection.

**Decision**: Immediate values can be passed directly without `Pin<&StackPinned<T>>` wrapping.

**Consequences**:
- Cleaner API for common cases
- Users must understand which types are immediate
- Type system enforces correct usage

## ADR-003: Single Crate with Proc-Macro Companion

**Status**: Accepted

**Context**: Proc macros must be in a separate crate from the code that uses them.

**Decision**: Split into `solidus` (main library) and `solidus-macros` (proc macros).

**Consequences**:
- Standard Rust proc-macro pattern
- Macros re-exported from main crate for convenience
- Two crates to maintain

## ADR-004: MIT License Only

**Status**: Accepted

**Context**: Need to choose a license for the project.

**Decision**: Use MIT license only (not dual MIT/Apache-2.0).

**Consequences**:
- Simple, permissive license
- Compatible with most use cases
- Single license file to maintain

## ADR-005: Minimum Ruby 3.4

**Status**: Accepted

**Context**: Need to decide which Ruby versions to support.

**Decision**: Support only Ruby 3.4 and later.

**Consequences**:
- Can use modern Ruby C API features
- Simpler maintenance without version-specific code
- Users on older Ruby must use alternative libraries

## ADR-006: Rust Edition 2024

**Status**: Accepted

**Context**: Need to choose a Rust edition for the project.

**Decision**: Use Rust Edition 2024 (MSRV 1.85+).

**Consequences**:
- Access to latest Rust features
- May limit adoption by users with older toolchains
- Cleaner code with modern idioms
