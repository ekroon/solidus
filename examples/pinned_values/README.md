# Pinned Values Example

This example demonstrates the core pinning concepts in Solidus, showing why Ruby values need special handling and how Solidus enforces safety at compile time.

## The Problem

Ruby's garbage collector uses **conservative stack scanning** to find live VALUE references. It only scans the C stack - if a VALUE is moved to the heap (into a `Vec`, `Box`, or `HashMap`), the GC cannot see it and may collect the underlying Ruby object.

```
Stack (GC scans this)          Heap (GC cannot scan this!)
+------------------+           +------------------------+
| local_var: VALUE |---------->| Ruby String "hello"    |
| (GC sees this)   |           | (protected)            |
+------------------+           +------------------------+

Vec on heap (invisible!)       Heap
+------------------+           +------------------------+
| vec[0]: VALUE    |---------->| Ruby String "world"    |
| (GC can't see!)  |           | (MAY BE COLLECTED!)    |
+------------------+           +------------------------+
```

This leads to use-after-free bugs that are notoriously difficult to debug.

## The Solution

Solidus provides two mechanisms to keep values safe:

### 1. Stack Pinning (`Pin<&StackPinned<T>>`)

For most cases, values are automatically pinned by the Context:

```rust
use solidus::prelude::*;

// Context provides stack-allocated storage for Ruby values
fn example<'ctx>(ctx: &'ctx Context) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    // Create and return a pinned value
    let s = ctx.new_string("hello")?;
    // s is already Pin<&'ctx StackPinned<RString>>
    
    // Access the value
    let content = s.get().to_string()?;
    Ok(s)
}
```

### 2. Heap Boxing (`BoxValue<T>`)

When you need heap storage, use `BoxValue` which registers with Ruby's GC:

```rust
use solidus::prelude::*;

fn example<'ctx>(ctx: &'ctx Context) -> Result<Vec<BoxValue<RString>>, Error> {
    // Create and box a value using Context
    let boxed = ctx.new_string_boxed("hello")?;
    
    // Safe to store in collections!
    let mut strings: Vec<BoxValue<RString>> = Vec::new();
    strings.push(boxed);
    Ok(strings)
}
```

## What This Example Demonstrates

### Functions with Pinned Arguments

```rust
fn process_pinned_string<'ctx>(
    ctx: &'ctx Context,
    input: Pin<&StackPinned<RString>>
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let content = input.get().to_string()?;
    ctx.new_string(&content.to_uppercase()).map_err(Into::into)
}
```

### Multiple Pinned Arguments

```rust
fn concatenate_pinned<'ctx>(
    ctx: &'ctx Context,
    first: Pin<&StackPinned<RString>>,
    second: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let s1 = first.get().to_string()?;
    let s2 = second.get().to_string()?;
    ctx.new_string(&format!("{}{}", s1, s2)).map_err(Into::into)
}
```

### Instance Methods with Pinned Args

```rust
fn append_to_self<'ctx>(
    ctx: &'ctx Context,
    rb_self: RString,
    suffix: Pin<&StackPinned<RString>>,
) -> Result<Pin<&'ctx StackPinned<RString>>, Error> {
    let self_str = rb_self.to_string()?;
    let suffix_str = suffix.get().to_string()?;
    ctx.new_string(&format!("{}{}", self_str, suffix_str))
        .map_err(Into::into)
}
```

### Collections with BoxValue

```rust
struct StringCollector {
    strings: Vec<BoxValue<RString>>,
}

impl StringCollector {
    fn add<'ctx>(&mut self, ctx: &'ctx Context, text: &str) -> Result<(), Error> {
        let boxed = ctx.new_string_boxed(text)?;
        self.strings.push(boxed);
        Ok(())
    }
}
```

## Building

```bash
cd examples/pinned_values
cargo build
```

## Testing

```bash
# Create symlink for Ruby to load
cd target/debug
ln -sf libpinned_values.dylib pinned_values.bundle  # macOS
# OR: ln -sf libpinned_values.so pinned_values.so   # Linux
cd ../..

# Run the tests
ruby test.rb
```

## Key Concepts

| Mechanism | Purpose | Performance | When to Use |
|-----------|---------|-------------|-------------|
| `Context` methods | Stack storage | Fast (no allocation) | Creating return values, local values |
| `BoxValue<T>` | Heap storage | Slower (GC registration) | Collections, caching |
| `Pin<&StackPinned<T>>` | Pinned reference | Zero-cost | Method arguments, parameters |

## Compile-Time Safety

Solidus enforces these rules at compile time:

1. **`!Copy` types** - Ruby values can't be accidentally copied to heap
2. **Context lifetime** - Return values are tied to the Context lifetime
3. **`!Unpin` wrappers** - Pinned values can't escape their lifetime

If your code compiles, it's safe from GC-related undefined behavior.

## Related Documentation

- [Pinning Guide](../../docs/guide/pinning.md) - In-depth explanation
- [BoxValue API](../../crates/solidus/src/value/boxed.rs) - Implementation details
- [StackPinned API](../../crates/solidus/src/value/pinned.rs) - Pinning implementation
