# Pinning in Solidus

This guide explains why Ruby values need pinning, how Solidus enforces it at compile time, and how to use the different storage options.

## Why Ruby Values Need Pinning

Ruby's garbage collector (GC) uses a technique called **conservative stack scanning** to find live objects. When the GC runs, it scans the C stack looking for values that look like valid Ruby VALUE pointers. Any VALUE found on the stack is considered "live" and won't be collected.

This works well for C code, where local variables naturally live on the stack. But it creates a problem for Rust: **if a VALUE is moved to the heap (into a `Vec`, `Box`, or `HashMap`), the GC cannot see it**.

```
Stack (GC scans this)          Heap (GC cannot scan this)
┌──────────────────┐           ┌──────────────────────────┐
│  local_var: VALUE ──────────►│  Ruby String "hello"     │
│  (GC sees this)  │           │                          │
└──────────────────┘           └──────────────────────────┘

Vec on heap (GC cannot see)    Heap
┌──────────────────┐           ┌──────────────────────────┐
│  vec[0]: VALUE ──────────────►│  Ruby String "world"     │
│  (invisible!)    │           │  (may be collected!)     │
└──────────────────┘           └──────────────────────────┘
```

When the GC collects a VALUE that's still referenced from the heap, you get **use-after-free** bugs that can corrupt memory, crash Ruby, or cause subtle data corruption.

## The Problem with Other Libraries

Other Ruby extension libraries like Magnus rely on documentation warnings to prevent this issue. The type system doesn't prevent you from writing dangerous code:

```rust
// Magnus allows this - but it's UNSAFE!
fn dangerous_code() -> Vec<RString> {
    let s = RString::new("hello");  // VALUE on stack
    let mut values = Vec::new();
    values.push(s);  // Moved to heap - GC can't see it!
    values  // Use-after-free waiting to happen
}
```

See [Magnus issue #101](https://github.com/matsadler/magnus/issues/101) for a detailed discussion of this problem.

The fundamental issue is that VALUE wrapper types in Magnus implement `Copy`, allowing them to be freely duplicated and stored anywhere. Even with careful documentation, it's easy to accidentally introduce bugs that only manifest under specific GC timing conditions.

## How Solidus Solves This

Solidus enforces safety at **compile time** through three mechanisms:

### 1. Constructors are `unsafe`

Value constructors are marked `unsafe` to prevent accidental storage in collections:

```rust
// This requires unsafe - makes you think about what you're doing
let s = unsafe { RString::new("hello") };

// Can't accidentally put it in a Vec without explicit unsafe
let vec = vec![s];  // ERROR: RString is !Copy
```

The `unsafe` requirement ensures you consciously acknowledge the safety requirements
when creating Ruby values.

### 2. Safe Paths via `pin_on_stack!` and `_boxed` Variants

Solidus provides safe paths that handle the unsafe code internally:

```rust
// PREFERRED: pin_on_stack! handles unsafe internally
pin_on_stack!(s = RString::new("hello"));
// s is Pin<&StackPinned<RString>>, safe to use

// For heap storage, use the safe _boxed variants
let boxed = RString::new_boxed("hello");  // Returns BoxValue<RString>
let mut vec = vec![boxed];  // Safe!
```

### 3. All VALUE Types are `!Copy`

Solidus VALUE wrapper types (`RString`, `RArray`, `RHash`, etc.) do not implement `Copy`. This prevents accidental duplication to heap storage:

```rust
let s = unsafe { RString::new("hello") };
let vec = vec![s];  // ERROR: RString is !Copy
```

### 4. Methods Use `&self`

All methods on VALUE types take `&self`, not `self`. This prevents moves of `!Copy` types during method calls:

```rust
impl RString {
    pub fn len(&self) -> usize;  // &self, not self
    pub fn to_string(&self) -> Result<String, Error>;
}
```

## Stack Pinning with `pin_on_stack!`

For most use cases, you want to pin values on the stack. The `pin_on_stack!` macro
is the **preferred way** to create Ruby values because it handles the unsafe code
internally:

```rust
use solidus::prelude::*;

// PREFERRED: One-shot creation and pinning (no unsafe needed!)
pin_on_stack!(s = RString::new("hello"));
// s is Pin<&StackPinned<RString>>

// Use the value through the pinned reference
let len = s.get().len();
let content = s.get().to_string()?;
```

The `pin_on_stack!` macro:

1. Creates the value using the unsafe constructor internally
2. Wraps the value in `StackPinned<T>` (which is `!Unpin`)
3. Creates a `Pin<&StackPinned<T>>` reference

Once pinned, the value cannot be moved to the heap.

### Why This is Safe

The `pin_on_stack!` macro is safe because:

1. The value is created and immediately pinned on the stack
2. The `Pin<&StackPinned<T>>` reference cannot be moved to the heap
3. The value remains visible to Ruby's GC throughout its lifetime

### Accessing Pinned Values

Use `.get()` to access the inner value:

```rust
pin_on_stack!(s = RString::new("hello"));

// Get a reference to the inner RString
let inner: &RString = s.get();

// Call methods on it
let content = inner.to_string()?;
```

For mutable access, use the `mut` variant:

```rust
pin_on_stack!(mut s = RString::new("hello"));
let inner: &mut RString = s.get_mut();
```

## Heap Allocation with Safe `_boxed` Variants

Sometimes you genuinely need to store Ruby values on the heap - in a `Vec`, `HashMap`, or across async boundaries. Solidus provides **safe `_boxed` constructor variants** for this:

```rust
use solidus::prelude::*;

// Safe heap storage - no unsafe needed!
let boxed = RString::new_boxed("hello");  // Returns BoxValue<RString>

// Now it's safe to store in collections
let mut strings: Vec<BoxValue<RString>> = Vec::new();
strings.push(boxed);
```

### How BoxValue Works

When you use a `_boxed` variant (like `RString::new_boxed()`):

1. The value is allocated on the heap
2. `rb_gc_register_address()` is called to tell Ruby's GC about the heap location
3. Ruby's GC will now scan this heap address during collection

When the `BoxValue` is dropped:

1. `rb_gc_unregister_address()` is called to remove the registration
2. The heap allocation is freed

### Performance Considerations

`BoxValue` has overhead compared to stack pinning:

- Heap allocation
- GC registration/unregistration
- Indirect access through a pointer

Prefer `pin_on_stack!` when possible. Use `_boxed` variants only when you need heap storage.

## When to Use Each Approach

### Use `pin_on_stack!` When:

- Processing values within a single function
- Passing values to other functions
- Returning values to Ruby
- Most common case

```rust
fn process_string(input: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let content = input.get().to_string()?;
    let processed = content.to_uppercase();
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&processed) })
}
```

### Use `_boxed` Variants When:

- Storing values in collections (`Vec`, `HashMap`, etc.)
- Keeping values alive across async boundaries
- Building data structures with Ruby values
- Caching Ruby values in Rust structs

```rust
struct Cache {
    strings: Vec<BoxValue<RString>>,
}

impl Cache {
    fn add(&mut self, content: &str) {
        // Safe! No unsafe needed
        self.strings.push(RString::new_boxed(content));
    }
}
```

### Use Raw `unsafe` Constructors When:

- Returning values immediately to Ruby from methods
- You need fine-grained control over value creation
- Performance-critical code where you've verified safety

```rust
fn greet() -> Result<NewValue<RString>, Error> {
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new("Hello!") })
}
```

## Method Signatures

When defining Ruby methods in Rust, argument types determine how values are passed:

### Stack-Pinned Arguments

For Ruby objects that need GC protection:

```rust
fn concat(
    rb_self: RString,
    other: Pin<&StackPinned<RString>>
) -> Result<NewValue<RString>, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&format!("{}{}", self_str, other_str)) })
}
```

### Immediate Values

Some Ruby values are encoded directly in the VALUE pointer and don't need GC protection:

- `Fixnum` - Small integers
- `Symbol` - Interned strings
- `Qnil`, `Qtrue`, `Qfalse` - Singleton values

These can be passed directly without pinning:

```rust
fn add(a: Fixnum, b: Fixnum) -> i64 {
    a.to_i64() + b.to_i64()
}
```

### Return Values

Return `NewValue<T>` for new Ruby objects:

```rust
// Return a new Ruby string
fn create_greeting(name: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let n = name.get().to_string()?;
    // SAFETY: Value is immediately returned to Ruby
    Ok(unsafe { RString::new(&format!("Hello, {}!", n)) })
}

// Return an immediate value (no NewValue wrapper needed)
fn compute_sum(a: Fixnum, b: Fixnum) -> i64 {
    a.to_i64() + b.to_i64()
}
```

## Summary

| Mechanism | Purpose | When to Use |
|-----------|---------|-------------|
| `unsafe` constructors | Force acknowledgment of safety requirements | Method return values, advanced use |
| `pin_on_stack!` | Safe stack storage (handles unsafe internally) | **Most cases** - local processing |
| `_boxed` variants | Safe heap storage | Collections, caching, TypedData |
| `!Copy` types | Prevent accidental heap moves | Automatic - all VALUE types |
| `NewValue<T>` | Mark values returned to Ruby | Method return types |

Solidus shifts the burden from "remember the rules" to "the compiler enforces the rules". The safe paths (`pin_on_stack!` and `_boxed` variants) make correct code easy to write, while `unsafe` constructors are available when you need more control.
