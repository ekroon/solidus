# Changelog

All notable changes to Solidus will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed - BREAKING

#### Phase 8: Context-Based Value Creation (2025-12-20)

This change introduces a **Context-based API** that eliminates `unsafe` constructors and
`NewValue<T>` guards entirely. The `Context` type provides stack-allocated storage for
Ruby values, ensuring safety through Rust's lifetime system.

**What Changed:**

1. **Context-based value creation**
   - `ctx.new_string("hello")` returns `Result<Pin<&'ctx StackPinned<RString>>, Error>`
   - `ctx.new_array()` returns `Result<Pin<&'ctx StackPinned<RArray>>, Error>`
   - All value creation methods tied to `Context` lifetime
   - No `unsafe`, no `NewValue<T>` guards, no manual pinning needed

2. **Method signatures now require Context**
   - First parameter is always `ctx: &'ctx Context`
   - Instance methods: `fn method(ctx: &'ctx Context, rb_self: RString, ...)`
   - Module/global functions: `fn function(ctx: &'ctx Context, ...)`
   - Return types: `Result<Pin<&'ctx StackPinned<T>>, Error>`

3. **Boxed variants for heap storage**
   - `ctx.new_string_boxed("hello")` returns `BoxValue<RString>`
   - Use when storing values in collections: `Vec<BoxValue<RString>>`
   - GC-registered automatically, no manual management needed

4. **BoxValue API changes**
   - `BoxValue::inner()` replaces deprecated `BoxValue::get()`
   - Returns reference to inner type: `&T`
   - Consistent with Rust naming conventions

**Migration Guide:**

Before (Phase 7):
```rust
// Unsafe constructors with manual pinning
pin_on_stack!(s = unsafe { RString::new("hello") });

// Or boxed variants
let s = RString::new_boxed("hello");
```

After (Phase 8):
```rust
// Context-based creation (recommended)
fn greet<'ctx>(ctx: &'ctx Context) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    ctx.new_string("hello")  // No unsafe, returns pinned reference
}

// For heap storage
let boxed = ctx.new_string_boxed("hello");
let mut values = vec![boxed];  // Safe!
```

**Why This Change:**

The Context-based API provides several key advantages:
1. **Zero `unsafe` in user code** - Context manages safety internally
2. **Automatic lifetime management** - Values cannot outlive the Context
3. **Ergonomic API** - No guards, no manual pinning, just works
4. **Better error handling** - Returns `Result` for Ruby exceptions
5. **Prevents common mistakes** - Compiler enforces correct usage

#### Phase 7: Safety Enforcement via Unsafe Constructors (2025-12-19)

**Note**: Phase 7 has been superseded by Phase 8's Context-based API. The unsafe
constructors and `NewValue<T>` guards described below are no longer the recommended
approach. See Phase 8 above for the current API.

<details>
<summary>Historical: Phase 7 Changes (superseded by Phase 8)</summary>

This change strengthened compile-time safety by making VALUE constructors `unsafe`.

**What Changed:**

1. **VALUE constructors became `unsafe`**
   - Required explicit `unsafe { RString::new() }`
   - Forced acknowledgment of safety contract

2. **`pin_on_stack!` macro handled unsafe internally**
   - Provided safe path for stack pinning

3. **Safe `_boxed` variants were added**
   - `RString::new_boxed()` for heap storage
   - Now superseded by `ctx.new_string_boxed()`

4. **`ReturnWitness` types were added**
   - Extra safety layer using lifetime witnesses
   - Now unnecessary with Context-based API

</details>

#### Phase 6: Pinned-From-Creation (ADR-007) (2025-12-17)

**Note**: Phase 6 has been superseded by Phase 8's Context-based API. The `NewValue<T>`
guards and manual pinning described below are no longer used. See Phase 8 above.

<details>
<summary>Historical: Phase 6 Changes (superseded by Phase 8)</summary>

This was a major change that made VALUE types `!Copy` and introduced `NewValue<T>` guards.

**What Changed:**

1. **All VALUE types became `!Copy`**
   - Prevented accidental duplication of VALUEs to heap storage
   - Immediate types (`Fixnum`, `Symbol`, etc.) remained `Copy`

2. **Creation functions returned `NewValue<T>`**
   - Required explicit pinning or boxing
   - `NewValue<T>` was `#[must_use]`

3. **Methods used `&self` instead of `self`**
   - Prevented moves of VALUE types

4. **`NewValue<T>` API for value creation**
   - `.pin()` for stack storage
   - `.into_box()` for heap storage with GC registration

**Why This Change:**

Ruby's GC scans the C stack to find VALUE references. Making VALUE types `!Copy`
enforced at compile time that all VALUES are either stack-pinned or explicitly
boxed with GC registration.

See [ADR-007](docs/plan/decisions.md#adr-007-values-must-be-pinned-from-creation) 
and [Magnus issue #101](https://github.com/matsadler/magnus/issues/101) for details.

</details>

### Added

- **Phase 8**: `Context` type for stack-allocated value creation
- **Phase 8**: Context methods: `new_string()`, `new_array()`, `new_hash()`, etc.
- **Phase 8**: Context boxed methods: `new_string_boxed()`, `new_array_boxed()`, etc.
- **Phase 8**: `BoxValue::inner()` method to access inner value
- **Phase 6**: `NewValue<T>` type for enforcing pinning from creation (superseded by Context)
- **Phase 6**: `NewValue::pin()` and `into_box()` methods (superseded by Context)

### Changed

- **Phase 8**: All methods now require `ctx: &'ctx Context` as first parameter
- **Phase 8**: Return types changed from `NewValue<T>` to `Pin<&'ctx StackPinned<T>>`
- **Phase 6**: All heap-allocated VALUE types are now `!Copy`
- **Phase 6**: Methods use `&self` instead of `self`

### Deprecated

- `BoxValue::get()` - use `BoxValue::inner()` instead

### Removed

- **Phase 8**: Direct VALUE constructors (`RString::new()`, etc.) - use Context methods
- **Phase 8**: `NewValue<T>` guard type - replaced by Context lifetime management
- **Phase 7**: `pin_on_stack!` macro - no longer needed with Context API
- **Phase 6**: `Copy` implementation from heap-allocated VALUE types

## [0.1.0] - TBD

Initial release (not yet published).
