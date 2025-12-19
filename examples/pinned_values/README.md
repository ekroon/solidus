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

For most cases, pin values on the stack where GC can see them:

```rust
use solidus::prelude::*;

// Create a value - returns NewValue
let guard = RString::new("hello");

// Pin it on the stack
pin_on_stack!(s = guard);
// s is now Pin<&StackPinned<RString>>

// Access the value
let content = s.get().to_string()?;
```

### 2. Heap Boxing (`BoxValue<T>`)

When you need heap storage, use `BoxValue` which registers with Ruby's GC:

```rust
use solidus::prelude::*;

// Create and box a value
let guard = RString::new("hello");
let boxed = guard.into_box();

// Safe to store in collections!
let mut strings: Vec<BoxValue<RString>> = Vec::new();
strings.push(boxed);
```

## What This Example Demonstrates

### Functions with Pinned Arguments

```rust
fn process_pinned_string(
    input: Pin<&StackPinned<RString>>
) -> Result<NewValue<RString>, Error> {
    let content = input.get().to_string()?;
    Ok(RString::new(&content.to_uppercase()))
}
```

### Multiple Pinned Arguments

```rust
fn concatenate_pinned(
    first: Pin<&StackPinned<RString>>,
    second: Pin<&StackPinned<RString>>,
) -> Result<NewValue<RString>, Error> {
    let s1 = first.get().to_string()?;
    let s2 = second.get().to_string()?;
    Ok(RString::new(&format!("{}{}", s1, s2)))
}
```

### Instance Methods with Pinned Args

```rust
fn append_to_self(
    rb_self: RString,
    suffix: Pin<&StackPinned<RString>>,
) -> Result<NewValue<RString>, Error> {
    let self_str = rb_self.to_string()?;
    let suffix_str = suffix.get().to_string()?;
    Ok(RString::new(&format!("{}{}", self_str, suffix_str)))
}
```

### Collections with BoxValue

```rust
struct StringCollector {
    strings: Vec<BoxValue<RString>>,
}

impl StringCollector {
    fn add(&mut self, s: Pin<&StackPinned<RString>>) {
        let boxed = BoxValue::new(s.get().clone());
        self.strings.push(boxed);
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
| `pin_on_stack!` | Stack storage | Fast (no allocation) | Local processing, arguments |
| `BoxValue<T>` | Heap storage | Slower (GC registration) | Collections, caching |
| `NewValue<T>` | Creation guard | Zero-cost | Returned from constructors |

## Compile-Time Safety

Solidus enforces these rules at compile time:

1. **`!Copy` types** - Ruby values can't be accidentally copied to heap
2. **`NewValue` must-use** - Compiler warns if you don't pin or box
3. **`!Unpin` wrappers** - Pinned values can't escape their lifetime

If your code compiles, it's safe from GC-related undefined behavior.

## Related Documentation

- [Pinning Guide](../../docs/guide/pinning.md) - In-depth explanation
- [BoxValue API](../../crates/solidus/src/value/boxed.rs) - Implementation details
- [StackPinned API](../../crates/solidus/src/value/pinned.rs) - Pinning implementation
