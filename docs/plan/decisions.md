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

## ADR-007: VALUES Must Be Pinned From Creation

**Status**: Accepted

**Context**: Ruby's GC scans the C stack to find VALUE references. For a VALUE to be
protected, it must be on the stack from the moment it's created until Ruby takes
ownership of it (e.g., when returning to Ruby).

The initial design used `Pin<&StackPinned<T>>` for method arguments, which pins values
passed from Ruby. However, this is insufficient because:

1. **Implicit pinning with Copy types is unsafe**: If VALUE wrapper types (RString, RArray,
   etc.) implement `Copy`, users can copy them out of pinned locations and store the
   copies on the heap. The pinned original doesn't protect the copy.

2. **Values created in Rust need protection too**: When user code calls `RString::new()`,
   that VALUE must be pinned immediately. If it's returned as a plain `RString` (Copy),
   users can store it in a `Vec` before returning - making it invisible to Ruby's GC.

3. **Return values need protection**: Any VALUE created inside a Rust function must remain
   on the stack until it's returned to Ruby. A GC pause between creation and return could
   collect an unprotected VALUE.

See: https://github.com/matsadler/magnus/issues/101 for background discussion.

**Decision**: All Ruby VALUE wrapper types must be pinned from the moment of creation in
Rust. This requires:

1. **VALUE types should be `!Copy`** (or creation returns pinned types): Prevent accidental
   copying to heap storage.

2. **Creation functions return pinned values**: `RString::new()` and similar should return
   a type that is immediately pinned or requires pinning context.

3. **Explicit `BoxValue<T>` for heap storage**: When users need to store VALUEs on the
   heap (in Vec, HashMap, etc.), they must explicitly use `BoxValue<T>` which registers
   with Ruby's GC.

4. **Return values are safe if pinned**: Returning a pinned value is safe because the
   extern "C" wrapper's stack frame keeps it alive until Ruby receives it.

**Consequences**:
- VALUE wrapper types cannot implement `Copy`
- More verbose API for VALUE creation and handling
- Compile-time enforcement of GC safety
- `BoxValue<T>` becomes the standard way to store VALUEs on the heap
- Need to redesign `RString::new()`, `RArray::new()`, etc. to enforce pinning
- Implicit pinning feature (from Stage 3) needs to be reconsidered or removed

**Implementation Notes**:

The API will likely look like:
```rust
// Option A: Creation requires pinning macro
pin_on_stack!(s = RString::new("hello")?);
// s is Pin<&StackPinned<RString>>, cannot be stored in Vec

// Option B: Creation returns a guard type that must be pinned
let guard = RString::new("hello")?;
pin_on_stack!(s = guard);  // Consumes guard, produces pinned ref

// Heap storage requires explicit BoxValue
let boxed = BoxValue::new(s);  // Explicit, GC-registered
let mut vec: Vec<BoxValue<RString>> = vec![boxed];  // Safe
```

**Related**:
- ADR-001: Use Pin for Stack Pinning
- ADR-002: Immediate Values Bypass Pinning (still valid - immediates don't need GC protection)
