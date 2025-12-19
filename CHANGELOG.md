# Changelog

All notable changes to Solidus will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed - BREAKING

#### Phase 7: Safety Enforcement via Unsafe Constructors (2025-12-19)

This change strengthens the compile-time safety guarantees by making VALUE constructors
`unsafe`. While `NewValue<T>` enforces that values must be pinned or boxed, it didn't
prevent users from storing `NewValue<T>` itself in heap collections before pinning.

**What Changed:**

1. **VALUE constructors are now `unsafe`**
   - `RString::new()` → `unsafe { RString::new() }`
   - `RArray::new()` → `unsafe { RArray::new() }`
   - All heap-allocated VALUE creation functions require `unsafe`
   - This forces users to explicitly acknowledge the safety contract

2. **`pin_on_stack!` macro handles unsafe internally**
   - `pin_on_stack!(s = RString::new("hello"))` works without explicit `unsafe`
   - The macro provides the safe, ergonomic path for stack pinning

3. **Safe `_boxed` variants added**
   - `RString::new_boxed("hello")` → `BoxValue<RString>` (no `unsafe` needed)
   - `RArray::new_boxed()` → `BoxValue<RArray>` (no `unsafe` needed)
   - All constructors have safe boxed variants for heap storage

4. **`ReturnWitness` and `WitnessedReturn` types added**
   - Optional extra safety layer using lifetime witnesses
   - `WitnessedReturn<'w, T>` cannot outlive the witness scope
   - Prevents storing return values in heap collections

**Migration Guide:**

Before (Phase 7):
```rust
let guard = RString::new("hello");  // Compiled, but guard could be stored in Vec
pin_on_stack!(s = guard.pin());
```

After (Phase 7):
```rust
// Option 1: Use pin_on_stack! (recommended, no unsafe needed)
pin_on_stack!(s = RString::new("hello"));

// Option 2: Use _boxed variants for heap storage (no unsafe needed)
let s = RString::new_boxed("hello");

// Option 3: Explicit unsafe (when you need NewValue directly)
let guard = unsafe { RString::new("hello") };
pin_on_stack!(s = guard.pin());
```

**Why This Change:**

The previous `NewValue<T>` pattern prevented storing the *inner* VALUE on the heap,
but `NewValue<T>` itself could be stored in a `Vec<NewValue<T>>` before being pinned.
By making constructors `unsafe`, we force users to either:
1. Use `pin_on_stack!` (safe, ergonomic)
2. Use `_boxed` variants (safe, for heap storage)
3. Explicitly opt into unsafe (advanced use cases)

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
