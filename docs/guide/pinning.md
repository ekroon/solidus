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

### 1. All VALUE Types are `!Copy`

Solidus VALUE wrapper types (`RString`, `RArray`, `RHash`, etc.) do not implement `Copy`. This prevents accidental duplication to heap storage:

```rust
let s = RString::new("hello");
let vec = vec![s];  // ERROR: RString is !Copy
```

### 2. Creation Returns `NewValue<T>`

When you create a Ruby value, you get a `NewValue<T>` that **must** be either pinned on the stack or explicitly boxed. The `#[must_use]` attribute warns if you forget:

```rust
let guard = RString::new("hello");
// WARNING if you don't pin or box this!
```

`NewValue` is itself `!Unpin`, so it can't be stored in collections either.

### 3. Methods Use `&self`

All methods on VALUE types take `&self`, not `self`. This prevents moves of `!Copy` types during method calls:

```rust
impl RString {
    pub fn len(&self) -> usize;  // &self, not self
    pub fn to_string(&self) -> Result<String, Error>;
}
```

## Stack Pinning with `pin_on_stack!`

For most use cases, you want to pin values on the stack. This is fast (no allocation) and keeps the VALUE visible to the GC:

```rust
use solidus::prelude::*;

// Create a value - returns NewValue<RString>
let guard = RString::new("hello");

// Pin it on the stack
pin_on_stack!(s = guard);
// s is now Pin<&StackPinned<RString>>

// Use the value through the pinned reference
let len = s.get().len();
let content = s.get().to_string()?;
```

The `pin_on_stack!` macro:

1. Consumes the `NewValue`
2. Wraps the value in `StackPinned<T>` (which is `!Unpin`)
3. Creates a `Pin<&StackPinned<T>>` reference

Once pinned, the value cannot be moved to the heap.

### One-Shot Pinning

You can combine creation and pinning in one statement:

```rust
pin_on_stack!(s = RString::new("hello"));
// s is immediately Pin<&StackPinned<RString>>
```

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

## Heap Allocation with `BoxValue<T>`

Sometimes you genuinely need to store Ruby values on the heap - in a `Vec`, `HashMap`, or across async boundaries. Solidus provides `BoxValue<T>` for this:

```rust
use solidus::prelude::*;

// Create a value
let guard = RString::new("hello");

// Box it for heap storage
let boxed = guard.into_box();

// Now it's safe to store in collections
let mut strings: Vec<BoxValue<RString>> = Vec::new();
strings.push(boxed);
```

### How BoxValue Works

When you call `.into_box()`:

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

Prefer `pin_on_stack!` when possible. Use `BoxValue` only when you need heap storage.

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
    Ok(RString::new(&processed))
}
```

### Use `BoxValue<T>` When:

- Storing values in collections (`Vec`, `HashMap`, etc.)
- Keeping values alive across async boundaries
- Building data structures with Ruby values
- Caching Ruby values in Rust structs

```rust
struct Cache {
    strings: Vec<BoxValue<RString>>,
}

impl Cache {
    fn add(&mut self, s: Pin<&StackPinned<RString>>) {
        self.strings.push(BoxValue::new(s.get().clone()));
    }
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
    Ok(RString::new(&format!("{}{}", self_str, other_str)))
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

Return `NewValue<T>` for new Ruby objects or immediate types for simple values:

```rust
// Return a new Ruby string
fn create_greeting(name: Pin<&StackPinned<RString>>) -> Result<NewValue<RString>, Error> {
    let n = name.get().to_string()?;
    Ok(RString::new(&format!("Hello, {}!", n)))
}

// Return an immediate value
fn compute_sum(a: Fixnum, b: Fixnum) -> i64 {
    a.to_i64() + b.to_i64()
}
```

## Summary

| Mechanism | Purpose | When to Use |
|-----------|---------|-------------|
| `!Copy` types | Prevent accidental heap moves | Automatic - all VALUE types |
| `NewValue<T>` | Force explicit storage choice | Returned from constructors |
| `pin_on_stack!` | Fast stack storage | Most cases |
| `BoxValue<T>` | Safe heap storage | Collections, caching |

Solidus shifts the burden from "remember the rules" to "the compiler enforces the rules". If your code compiles, it's safe from GC-related undefined behavior.
