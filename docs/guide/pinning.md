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

Solidus enforces safety at **compile time** through the `Context` type and pinned references:

### The Context Type

The `Context` type provides stack-allocated storage slots for Ruby VALUES during method execution. When you create a Ruby value through the Context, it:

1. Allocates the VALUE in one of Context's stack-local slots
2. Returns a `Pin<&'ctx StackPinned<T>>` reference with lifetime tied to the Context
3. Ensures the value cannot be moved to the heap or outlive the method call

This approach makes safe Ruby value creation ergonomic and enforces safety at compile time.

### Stack Storage with Context

For most use cases in Ruby methods, use the Context to create values:

```rust
use solidus::prelude::*;

fn greet<'ctx>(
    ctx: &'ctx Context,
    name: Pin<&StackPinned<RString>>
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let name_str = name.get().to_string()?;
    ctx.new_string(&format!("Hello, {}!", name_str))
        .map_err(Into::into)
}
```

The Context:
- Provides 8 VALUE slots by default (customizable via const generics)
- Returns `Pin<&'ctx StackPinned<T>>` which is already pinned and ready to use
- Prevents the value from being moved to the heap
- Ties the value's lifetime to the method call's stack frame

### Accessing Values

Values created through Context are returned as `Pin<&'ctx StackPinned<T>>`. Use `.get()` to access the inner value:

```rust
fn process<'ctx>(
    ctx: &'ctx Context,
    input: Pin<&StackPinned<RString>>
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    // Access the inner RString
    let content = input.get().to_string()?;
    let processed = content.to_uppercase();
    
    // Create and return a new pinned value
    ctx.new_string(&processed).map_err(Into::into)
}
```

### All VALUE Types are `!Copy`

Solidus VALUE wrapper types (`RString`, `RArray`, `RHash`, etc.) do not implement `Copy`. This prevents accidental duplication to heap storage:

```rust
let s = unsafe { RString::new("hello") };
let vec = vec![s];  // ERROR: RString is !Copy
```

### Methods Use `&self`

All methods on VALUE types take `&self`, not `self`. This prevents moves of `!Copy` types during method calls:

```rust
impl RString {
    pub fn len(&self) -> usize;  // &self, not self
    pub fn to_string(&self) -> Result<String, Error>;
}
```

## Context Capacity

By default, Context provides 8 VALUE slots. If you need more, customize the capacity:

```rust
// Default: 8 slots
fn example<'ctx>(ctx: &'ctx Context) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    ctx.new_string("hello").map_err(Into::into)
}

// Custom: 16 slots
fn example_large<'ctx>(ctx: &'ctx Context<16>) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    // Can create up to 16 values
    ctx.new_string("hello").map_err(Into::into)
}
```

If slots are exhausted, `ctx.new_xxx()` methods return `Err(AllocationError)`. In practice, 8 slots is sufficient for most methods.

## Heap Allocation with BoxValue

Sometimes you genuinely need to store Ruby values on the heap - in a `Vec`, `HashMap`, or across async boundaries. Solidus provides **safe `_boxed` constructor variants** for this:

```rust
use solidus::prelude::*;

// Safe heap storage
let boxed = RString::new_boxed("hello");  // Returns BoxValue<RString>

// Now it's safe to store in collections
let mut strings: Vec<BoxValue<RString>> = Vec::new();
strings.push(boxed);
```

You can also create boxed values through Context:

```rust
fn build_cache<'ctx>(ctx: &'ctx Context) -> Vec<BoxValue<RString>> {
    let mut cache = Vec::new();
    
    // Using Context's boxed methods
    cache.push(ctx.new_string_boxed("item1"));
    cache.push(ctx.new_string_boxed("item2"));
    
    // Or using the type's boxed constructor directly
    cache.push(RString::new_boxed("item3"));
    
    cache
}
```

### How BoxValue Works

When you use a `_boxed` variant:

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

Prefer Context for method-local values. Use `_boxed` variants only when you need heap storage.

## When to Use Each Approach

### Use Context When:

- Creating values within Ruby methods (most common case)
- Processing values within a single function
- Passing values to other functions
- Returning values to Ruby

```rust
fn process_string<'ctx>(
    ctx: &'ctx Context,
    input: Pin<&StackPinned<RString>>
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let content = input.get().to_string()?;
    let processed = content.to_uppercase();
    ctx.new_string(&processed).map_err(Into::into)
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
        self.strings.push(RString::new_boxed(content));
    }
}
```

### Use `pin_on_stack!` When:

- Working outside of Ruby method context (e.g., in tests or initialization)
- You need fine-grained control over value creation

```rust
// In tests or initialization code
pin_on_stack!(s = RString::new("hello"));
// s is Pin<&StackPinned<RString>>
let len = s.get().len();
```

**Note**: Within Ruby methods, prefer using Context over `pin_on_stack!` because Context is more ergonomic and integrates better with method signatures.

## Method Signatures

When defining Ruby methods in Rust, the Context is always the first parameter:

### Instance Methods

Instance methods receive Context first, then `self`, then arguments:

```rust
fn concat<'ctx>(
    ctx: &'ctx Context,
    rb_self: RString,
    other: Pin<&StackPinned<RString>>
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let self_str = rb_self.to_string()?;
    let other_str = other.get().to_string()?;
    ctx.new_string(&format!("{}{}", self_str, other_str))
        .map_err(Into::into)
}
```

### Functions (No Self)

Functions and class methods receive Context first, then arguments:

```rust
fn to_upper<'ctx>(
    ctx: &'ctx Context,
    s: Pin<&StackPinned<RString>>
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let input = s.get().to_string()?;
    ctx.new_string(&input.to_uppercase()).map_err(Into::into)
}
```

### Immediate Values

Some Ruby values are encoded directly in the VALUE pointer and don't need GC protection:

- `Fixnum` - Small integers
- `Symbol` - Interned strings
- `Qnil`, `Qtrue`, `Qfalse` - Singleton values

These can be passed directly without pinning:

```rust
fn add(_ctx: &Context, a: Fixnum, b: Fixnum) -> i64 {
    a.to_i64() + b.to_i64()
}
```

### Return Values

Return `Pin<&'ctx StackPinned<T>>` for values created through Context:

```rust
fn create_greeting<'ctx>(
    ctx: &'ctx Context,
    name: Pin<&StackPinned<RString>>
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let n = name.get().to_string()?;
    ctx.new_string(&format!("Hello, {}!", n)).map_err(Into::into)
}
```

For immediate values, return the raw type:

```rust
fn compute_sum(_ctx: &Context, a: Fixnum, b: Fixnum) -> i64 {
    a.to_i64() + b.to_i64()
}
```

## Summary

| Mechanism | Purpose | When to Use |
|-----------|---------|-------------|
| `Context` | Stack-allocated VALUE slots | **Most cases** - creating values in Ruby methods |
| `Pin<&'ctx StackPinned<T>>` | Safe reference to stack value | Return type from Context methods |
| `_boxed` variants | Safe heap storage | Collections, caching, TypedData |
| `!Copy` types | Prevent accidental heap moves | Automatic - all VALUE types |
| `pin_on_stack!` | Manual stack pinning | Tests, initialization, outside method context |

Solidus shifts the burden from "remember the rules" to "the compiler enforces the rules". The Context type makes safe Ruby value creation ergonomic and natural, while the type system prevents unsafe patterns at compile time.
