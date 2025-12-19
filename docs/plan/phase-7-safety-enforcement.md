# Phase 7: Compile-Time Safety Enforcement

## Objective

Redesign the VALUE creation and return API to provide **true compile-time enforcement**
of safety invariants. The current `PinGuard<T>` design has fundamental gaps that allow
unsafe patterns to compile.

## Dependencies

- Phases 1-6 complete
- Understanding of Pin/Unpin semantics and their limitations

## Problem Statement

### The Original Design Intent

The original `PinGuard<T>` design was intended to prevent storing Ruby VALUEs in heap
collections without GC registration. The theory was:

1. Make `PinGuard<T>` implement `!Unpin` (via `PhantomPinned`)
2. Force users to either `pin_on_stack!` or `into_box()` the guard
3. The compiler would reject attempts to store the guard in collections

### Why It Doesn't Work

**The fundamental flaw: `!Unpin` does NOT prevent `Vec` storage.**

```rust
// This compiles! Vec<T> does NOT require T: Unpin
let guard = RString::new("hello");
let vec: Vec<PinGuard<RString>> = vec![guard];  // COMPILES - SAFETY HOLE!
```

The `Unpin` trait only matters when working with `Pin<P>`. It does not affect whether
a type can be moved or stored in collections. The `Vec<T>` type has no `Unpin` bound.

**Being `!Copy` also doesn't help:**

```rust
// !Copy only prevents copies, not moves
let guard = RString::new("hello");
let moved = guard;  // This is a MOVE, not a copy - totally allowed
let vec = vec![moved];  // SAFETY HOLE!
```

### The Consequence

The current design provides a **false sense of security**. Users believe the compiler
is protecting them when it's not. This is worse than having no safety at all because:

1. Users write code believing it's safe
2. The code compiles without warnings
3. UB can occur at runtime, causing hard-to-debug crashes
4. The documentation promises safety guarantees we don't deliver

### What Actually Needs Enforcement

1. **At creation**: VALUEs must not be storable in collections without explicit action
2. **At return**: Values returned to Ruby must not be storable in collections either
3. **The escape hatch**: `unsafe` code should be the only way to bypass these rules

## Solution Design

### 1. NewValue<T> with Unsafe Constructors

Rename `PinGuard<T>` to `NewValue<T>` to better communicate its purpose - it represents
a newly-created VALUE that needs handling.

**Key change: All VALUE-creating constructors are `unsafe`.**

```rust
/// A newly-created Ruby value that must be pinned or boxed.
#[must_use = "NewValue must be handled via pin_on_stack! or into_box()"]
pub struct NewValue<T: ReprValue> {
    value: T,
}

impl RString {
    /// Create a new Ruby string.
    ///
    /// # Safety
    ///
    /// The returned `NewValue` must be immediately:
    /// - Pinned on the stack via `pin_on_stack!`, OR
    /// - Boxed for heap storage via `into_box()`
    ///
    /// Storing the `NewValue` directly in a collection is undefined behavior.
    /// Use `new_boxed()` for safe heap storage.
    pub unsafe fn new(s: &str) -> NewValue<RString> {
        // Create the Ruby string...
        NewValue::new(rstring)
    }

    /// Create a new Ruby string, boxed for heap storage.
    ///
    /// This is the safe way to create strings for storage in collections.
    /// The returned `BoxValue` is registered with Ruby's GC.
    pub fn new_boxed(s: &str) -> BoxValue<RString> {
        // SAFETY: We immediately box the value
        unsafe { Self::new(s) }.into_box()
    }
}
```

**Why `unsafe` constructors work:**

1. Users cannot accidentally create a `NewValue` without acknowledging unsafety
2. The `pin_on_stack!` macro encapsulates the `unsafe` block
3. Safe `_boxed` variants provide a fully-safe path for heap storage
4. Deliberate `unsafe` code is the explicit opt-out (matching Rust philosophy)

### 2. WitnessedReturn<'w, T> for Method Returns

For methods that return values to Ruby, we need a different approach. The returned
value will be immediately passed to Ruby (safe), but we must prevent the user from
storing it in a local collection first.

**Solution: Use lifetime constraints to prevent collection storage.**

```rust
/// A marker type created on the method wrapper's stack frame.
/// The lifetime 'w is tied to this marker's stack location.
pub struct ReturnWitness {
    _marker: PhantomData<*const ()>,  // !Send + !Sync
}

impl ReturnWitness {
    /// Create a new return witness.
    ///
    /// # Safety
    ///
    /// Must only be called by the method! macro on its stack frame.
    /// The witness must not outlive the stack frame.
    #[doc(hidden)]
    pub unsafe fn new() -> Self {
        ReturnWitness {
            _marker: PhantomData,
        }
    }
}

/// A value that has been witnessed for return to Ruby.
///
/// The lifetime `'w` is borrowed from a `ReturnWitness`, preventing
/// this value from being stored in any collection that would outlive
/// the current function.
pub struct WitnessedReturn<'w, T: ReprValue> {
    value: T,
    _witness: PhantomData<&'w ReturnWitness>,
}

impl<'w, T: ReprValue> WitnessedReturn<'w, T> {
    /// Create a witnessed return value.
    pub fn new(_witness: &'w ReturnWitness, value: T) -> Self {
        WitnessedReturn {
            value,
            _witness: PhantomData,
        }
    }
}
```

**Why this works:**

The lifetime `'w` is borrowed from the `ReturnWitness` which lives on the method
wrapper's stack. This means:

```rust
fn my_method<'w>(rb_self: RString, w: &'w ReturnWitness) -> Result<WitnessedReturn<'w, RString>, Error> {
    let result = WitnessedReturn::new(w, /* ... */);

    // This would NOT compile:
    // let vec: Vec<WitnessedReturn<'w, RString>> = vec![result];
    // Error: cannot infer an appropriate lifetime for autoref

    // Vec<T> requires T: 'static (implicitly, for the vec to be storable anywhere)
    // WitnessedReturn<'w, T> has lifetime 'w which is NOT 'static

    Ok(result)
}
```

### 3. Combined Safety Model

The two mechanisms work together:

| Creation Context | Type | Safety Mechanism |
|------------------|------|------------------|
| Standalone code | `NewValue<T>` | `unsafe` constructor + `pin_on_stack!` macro |
| Method return | `WitnessedReturn<'w, T>` | Lifetime prevents collection storage |
| Heap storage | `BoxValue<T>` | Safe `_boxed` constructors |

**The only ways to bypass safety:**

1. Write `unsafe` code explicitly (Rust philosophy - explicit opt-out)
2. Use `std::mem::transmute` or similar (requires `unsafe`)

## API Examples

### 1. Creating and Pinning a Value with `pin_on_stack!`

```rust
use solidus::prelude::*;

fn process_string() -> Result<(), Error> {
    // The pin_on_stack! macro handles the unsafe internally
    pin_on_stack!(s = RString::new("hello"));

    // s is now Pin<&StackPinned<RString>> - safely pinned on stack
    println!("String length: {}", s.len()?);

    Ok(())
}
```

**What the macro expands to:**

```rust
fn process_string() -> Result<(), Error> {
    // SAFETY: Value is immediately pinned on the stack
    let new_value = unsafe { RString::new("hello") };
    let mut stack_pinned = StackPinned::new(new_value.into_inner());
    let s = unsafe { Pin::new_unchecked(&stack_pinned) };

    println!("String length: {}", s.len()?);

    Ok(())
}
```

### 2. Creating a Boxed Value with `_boxed` Variant

```rust
use solidus::prelude::*;

fn store_strings() -> Vec<BoxValue<RString>> {
    let mut strings = Vec::new();

    // Safe: new_boxed() returns BoxValue<RString> directly
    strings.push(RString::new_boxed("hello"));
    strings.push(RString::new_boxed("world"));

    strings
}
```

### 3. Method Signature with WitnessedReturn

```rust
use solidus::prelude::*;
use solidus::method::WitnessedReturn;

// User writes this function
fn concat<'w>(
    rb_self: RString,
    other: &RString,
    w: &'w ReturnWitness,
) -> Result<WitnessedReturn<'w, RString>, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.to_string()?;

    // Create the result - witnessed for safe return
    let result = format!("{}{}", self_str, other_str);

    // SAFETY: We're returning immediately, not storing
    Ok(WitnessedReturn::new(w, unsafe { RString::new(&result) }.into_inner()))
}

// Alternative: Use a helper that makes this cleaner
fn concat_v2<'w>(
    rb_self: RString,
    other: &RString,
    w: &'w ReturnWitness,
) -> Result<WitnessedReturn<'w, RString>, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.to_string()?;
    let result = format!("{}{}", self_str, other_str);

    // Helper method on ReturnWitness to create strings safely
    w.string(&result)
}
```

### 4. What the method! Macro Generates

```rust
// User writes:
method!(String, "concat", concat, 1);

// Macro generates:
extern "C" fn __solidus_concat(
    argc: std::ffi::c_int,
    argv: *const rb_sys::VALUE,
    rb_self: rb_sys::VALUE,
) -> rb_sys::VALUE {
    // Create witness on this stack frame
    // SAFETY: Witness lives for the duration of this extern "C" function
    let witness = unsafe { ReturnWitness::new() };

    // Parse arguments...
    let args = unsafe { Args::from_raw(argc, argv, 1) };
    let other = args.get::<RString>(0);

    // Call user function with witness
    let result = concat(
        unsafe { RString::from_raw(rb_self) },
        &other,
        &witness,
    );

    // Handle result
    match result {
        Ok(witnessed) => witnessed.into_raw(),
        Err(e) => {
            // Raise Ruby exception
            unsafe { rb_sys::rb_raise(e.class(), e.message()) };
        }
    }
}
```

### 5. Attempting Unsafe Patterns (Compiler Errors)

```rust
// ATTEMPT 1: Store NewValue in Vec
fn unsafe_attempt_1() {
    // ERROR: call to unsafe function `RString::new` requires unsafe block
    let s = RString::new("hello");
    let vec = vec![s];
}

// ATTEMPT 2: Store in Vec using unsafe block
fn unsafe_attempt_2() {
    // This compiles, but user explicitly wrote "unsafe" - they own the consequences
    let s = unsafe { RString::new("hello") };
    let vec = vec![s];  // User chose to bypass safety
}

// ATTEMPT 3: Store WitnessedReturn in Vec
fn unsafe_attempt_3<'w>(w: &'w ReturnWitness) {
    let s = w.string("hello").unwrap();
    // ERROR: `s` does not live long enough
    // Vec requires 'static, but s has lifetime 'w
    let vec: Vec<_> = vec![s];
}

// ATTEMPT 4: Return WitnessedReturn from function (without macro)
fn unsafe_attempt_4<'w>(w: &'w ReturnWitness) -> WitnessedReturn<'w, RString> {
    // This is fine - we're returning it, not storing it
    w.string("hello").unwrap()
}
```

## Tasks

### 7.1 Rename and Restructure

- [ ] Rename `PinGuard<T>` to `NewValue<T>`
- [ ] Update all documentation references
- [ ] Update `pin_on_stack!` macro to work with new name

### 7.2 Make Constructors Unsafe

- [ ] Make `RString::new()` unsafe
- [ ] Make `RArray::new()` and `RArray::with_capacity()` unsafe
- [ ] Make `RHash::new()` unsafe
- [ ] Make `Value::funcall()` and similar methods return unsafe-constructed values
- [ ] Add safe `_boxed` variants for all constructors

### 7.3 Implement WitnessedReturn System

- [ ] Create `ReturnWitness` struct
- [ ] Create `WitnessedReturn<'w, T>` struct
- [ ] Add helper methods on `ReturnWitness` for common value creation
- [ ] Implement `IntoValue` for `WitnessedReturn`

### 7.4 Update method! Macro

- [ ] Generate `ReturnWitness` on wrapper stack frame
- [ ] Pass witness to user function
- [ ] Update function signature requirements
- [ ] Handle backward compatibility (if needed)

### 7.5 Update Examples and Documentation

- [ ] Update all examples to new API
- [ ] Update guide documentation
- [ ] Add migration guide section
- [ ] Update AGENTS.md with new patterns

### 7.6 Add Compile-Fail Tests

- [ ] Test: Cannot call `RString::new()` without `unsafe`
- [ ] Test: Cannot store `NewValue<T>` in Vec (when obtained unsafely)
- [ ] Test: Cannot store `WitnessedReturn<'w, T>` in Vec
- [ ] Test: `pin_on_stack!` compiles without explicit `unsafe`
- [ ] Test: `_boxed` variants compile without `unsafe`

## Migration from Old API

### Breaking Changes

1. **All VALUE constructors are now `unsafe`**

   ```rust
   // Before (Phase 6)
   let guard = RString::new("hello");
   pin_on_stack!(s = guard);

   // After (Phase 7)
   pin_on_stack!(s = RString::new("hello"));  // macro handles unsafe

   // Or explicitly:
   let new_value = unsafe { RString::new("hello") };
   pin_on_stack!(s = new_value);
   ```

2. **Method signatures require `ReturnWitness`**

   ```rust
   // Before (Phase 6)
   fn my_method(rb_self: RString) -> Result<PinGuard<RString>, Error> {
       Ok(RString::new("result"))
   }

   // After (Phase 7)
   fn my_method<'w>(
       rb_self: RString,
       w: &'w ReturnWitness
   ) -> Result<WitnessedReturn<'w, RString>, Error> {
       Ok(w.string("result")?)
   }
   ```

3. **Safe heap storage uses `_boxed` variants**

   ```rust
   // Before (Phase 6) - seemed safe but wasn't!
   let guard = RString::new("hello");
   let boxed = guard.into_box();

   // After (Phase 7) - truly safe
   let boxed = RString::new_boxed("hello");
   ```

### Migration Steps

1. **Find all VALUE creation sites**
   - Search for `RString::new`, `RArray::new`, `RHash::new`, etc.
   - For standalone code: wrap in `pin_on_stack!` or use `_boxed` variants
   - For method returns: add witness parameter and use `WitnessedReturn`

2. **Update method signatures**
   - Add `<'w>` lifetime parameter
   - Add `w: &'w ReturnWitness` parameter
   - Change return type to `WitnessedReturn<'w, T>`

3. **Update method! macro calls**
   - May need to update arity counts if witness is counted
   - Review generated code for compatibility

4. **Run compiler**
   - The compiler will catch any remaining issues
   - Fix errors by following the patterns above

### Backward Compatibility Considerations

If gradual migration is needed:

```rust
// Temporary: Allow old-style returns with deprecation warning
#[deprecated(note = "Use WitnessedReturn for safety")]
impl<T: ReprValue> ReturnValue for NewValue<T> {
    fn into_return_value(self) -> Result<Value, Error> {
        // SAFETY: Returning to Ruby immediately
        Ok(unsafe { self.into_inner() }.as_value())
    }
}
```

## Design Rationale

### Why Not Use a Different Approach?

**Alternative 1: Runtime checks**
- Could panic if value is moved to heap
- Rejected: Runtime panics are worse than compile errors

**Alternative 2: Custom `Vec` type**
- Provide `RubyVec<T>` that only accepts `BoxValue<T>`
- Rejected: Users can still use `std::vec::Vec`

**Alternative 3: Lint/clippy rule**
- Add custom lint to detect the pattern
- Rejected: Lints can be ignored, not a true guarantee

**Alternative 4: Unsafe constructors (chosen)**
- Makes the unsafe explicit at the point of creation
- Matches Rust's philosophy of safe-by-default
- `pin_on_stack!` macro provides ergonomic safe path
- `_boxed` variants provide safe heap storage

### Consistency with Rust Philosophy

This design follows Rust's core principle: **safe by default, explicit opt-out**.

1. The safe path (`pin_on_stack!`, `_boxed` variants) requires no `unsafe`
2. Bypassing safety requires explicit `unsafe` blocks
3. The `unsafe` blocks are at the right granularity - at creation time
4. Users who write `unsafe` are acknowledging they understand the risks

## Acceptance Criteria

- [ ] `RString::new()` and other constructors require `unsafe`
- [ ] `pin_on_stack!` macro provides safe creation+pinning
- [ ] `_boxed` variants provide safe heap storage
- [ ] Method returns use `WitnessedReturn` with lifetime constraints
- [ ] Compile-fail tests verify unsafe patterns don't compile
- [ ] All examples updated to new API
- [ ] Migration guide is complete and accurate
- [ ] No runtime overhead compared to Phase 6 design
