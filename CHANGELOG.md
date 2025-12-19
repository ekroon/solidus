# Changelog

All notable changes to Solidus will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed - BREAKING

#### ADR-007: Pinned-From-Creation (2025-12-17)

This is a **major breaking change** that fundamentally changes how Ruby values are
created and used in Solidus. These changes enforce safety at compile time and prevent
the undefined behavior possible in libraries like Magnus.

**What Changed:**

1. **All VALUE types are now `!Copy`**
   - `RString`, `RArray`, `RHash`, `RBignum`, `RFloat`, `Integer`, `Float`, `RClass`, `RModule` no longer implement `Copy`
   - Immediate types (`Fixnum`, `Symbol`, `Qnil`, `Qtrue`, `Qfalse`, `Flonum`) remain `Copy`
   - This prevents accidental duplication of VALUEs to heap storage

2. **Creation functions now return `NewValue<T>`**
   - `RString::new()` returns `NewValue<RString>` instead of `RString`
   - `RArray::new()` returns `NewValue<RArray>` instead of `RArray`
   - All creation functions follow this pattern
   - `NewValue<T>` is `#[must_use]` - compiler warns if not consumed

3. **Methods now use `&self` instead of `self`**
   - `len(&self)` instead of `len(self)`
   - `to_string(&self)` instead of `to_string(self)`
   - All methods updated to prevent moves

4. **New `NewValue<T>` API for value creation**
   - `.pin()` → converts to `StackPinned<T>` for stack storage
   - `.into_box()` → converts to `BoxValue<T>` for heap storage (GC-registered)
   - Must explicitly choose stack or heap storage

**Migration Guide:**

Before (unsafe):
```rust
let s = RString::new("hello");
let arr = RArray::new();
arr.push(s);  // UNSAFE - could be stored in Vec without GC protection
```

After (safe):
```rust
// Stack pinning (common case)
let s_guard = RString::new("hello");
let s = s_guard.pin();
pin_on_stack!(s_ref = s);

// Heap boxing (for collections)
let arr_guard = RArray::new();
let arr = arr_guard.into_box();  // Explicit GC registration
let mut values = vec![arr];      // Safe!
```

**Why This Change:**

Ruby's GC scans the C stack to find VALUE references. If VALUES are moved to the
heap without GC registration (e.g., stored in a `Vec`), the GC cannot see them and
may collect the underlying Ruby objects, causing use-after-free bugs.

Previous Solidus versions (and current Magnus) relied on VALUE types being `Copy`,
but this created a safety gap: users could copy VALUES out of pinned locations and
store the copies on the heap, defeating the pinning protection.

By making VALUE types `!Copy` and using `NewValue<T>`, we enforce at compile time
that all VALUES are either:
1. Stack-pinned (GC can see them)
2. Explicitly boxed with GC registration (GC is notified)

See [ADR-007](docs/plan/decisions.md#adr-007-values-must-be-pinned-from-creation) 
and [Magnus issue #101](https://github.com/matsadler/magnus/issues/101) for details.

**Benefits:**

- ✅ Compile-time prevention of VALUE heap escape
- ✅ Clear, explicit API with `#[must_use]` warnings
- ✅ Zero runtime overhead - all checks are compile-time
- ✅ Prevents undefined behavior that Magnus allows

### Added

- `NewValue<T>` type for enforcing pinning from creation
- `NewValue::pin()` method to convert to `StackPinned<T>`
- `NewValue::into_box()` method to convert to `BoxValue<T>`
- `NewValue::as_ref()` and `as_mut()` for inspection without consuming

### Removed

- `Copy` implementation from all heap-allocated VALUE types
- Direct construction of VALUE types (now must go through `NewValue`)

## [0.1.0] - TBD

Initial release (not yet published).
