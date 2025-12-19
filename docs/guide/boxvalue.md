# BoxValue: Storing Ruby Values on the Heap

This guide explains when and how to use `BoxValue<T>` for storing Ruby values on the heap.

## Why BoxValue Exists

In Solidus, Ruby values cannot be simply stored in Rust collections like `Vec` or `HashMap`. 
This is a deliberate safety constraint that prevents a class of subtle garbage collection bugs.

`BoxValue<T>` is the safe way to store Ruby values on the heap. It wraps a Ruby value 
and registers it with Ruby's garbage collector, ensuring the value won't be prematurely freed.

## The Problem: Ruby GC Only Scans the Stack

Ruby's garbage collector uses a conservative stack scanning approach. During GC, Ruby walks 
the C call stack looking for anything that looks like a VALUE (Ruby's internal representation 
of objects). Any VALUE found on the stack is considered "reachable" and won't be collected.

This works well for typical C extensions where VALUES live in local variables. However, 
it creates a problem for Rust:

```rust
// DANGER: This pattern is unsafe in Ruby extensions!
let strings: Vec<RString> = vec![
    RString::new("hello"),
    RString::new("world"),
];
// The Vec's heap allocation is NOT on the stack
// Ruby's GC cannot see these strings
// A GC cycle here could free the underlying Ruby objects!
```

When VALUES are stored on the heap (in a `Vec`, `HashMap`, or any heap-allocated structure), 
Ruby's stack scanner cannot find them. The GC may decide these objects are unreachable and 
free them, even though your Rust code still holds references.

This is undefined behavior - a use-after-free bug that can cause crashes, data corruption, 
or security vulnerabilities.

## How BoxValue Solves It

`BoxValue<T>` solves this by explicitly registering the VALUE's location with Ruby's GC:

```rust
use solidus::{BoxValue, RString, pin_on_stack};

// Create a value and convert it to BoxValue
pin_on_stack!(s = RString::new("hello"));
let boxed = BoxValue::new(s.get().clone());

// Now it's safe to store in a Vec
let mut strings: Vec<BoxValue<RString>> = Vec::new();
strings.push(boxed);
// Ruby's GC knows about these values and won't collect them
```

Under the hood, `BoxValue`:
1. Allocates the VALUE on the heap using `Box`
2. Calls `rb_gc_register_address()` to tell Ruby's GC about the heap location
3. When dropped, calls `rb_gc_unregister_address()` to clean up

This is the same mechanism Ruby's C API provides for storing VALUES in global or 
heap-allocated C variables.

## Creating BoxValue from PinGuard

When you create a new Ruby value (e.g., `RString::new()`), it returns a `PinGuard<T>`. 
You can convert this directly to a `BoxValue<T>` using `.into_box()`:

```rust
use solidus::{BoxValue, RString};

// Create and immediately box
let guard = RString::new("hello");
let boxed: BoxValue<RString> = guard.into_box();

// Now boxed can be stored anywhere
```

This is more efficient than first pinning on the stack and then boxing, as it avoids 
the intermediate step:

```rust
use solidus::{BoxValue, RString, pin_on_stack};

// Less efficient: pin then clone into box
pin_on_stack!(s = RString::new("hello"));
let boxed = BoxValue::new(s.get().clone());

// More efficient: direct conversion
let boxed = RString::new("world").into_box();
```

## Using BoxValue in Collections

`BoxValue` is designed for use in Rust collections:

### Vec

```rust
use solidus::{BoxValue, RString};

let mut strings: Vec<BoxValue<RString>> = Vec::new();

// Add values
strings.push(RString::new("first").into_box());
strings.push(RString::new("second").into_box());

// Access values
for s in &strings {
    println!("{}", s.to_string().unwrap());
}
```

### HashMap

```rust
use solidus::{BoxValue, RString, Value};
use std::collections::HashMap;

let mut cache: HashMap<String, BoxValue<Value>> = HashMap::new();

cache.insert("key1".to_string(), RString::new("value1").into_box().into());
cache.insert("key2".to_string(), RString::new("value2").into_box().into());
```

### In TypedData Structs

A common use case is storing Ruby values inside Rust structs that are wrapped as 
Ruby objects using TypedData:

```rust
use solidus::prelude::*;
use solidus::typed_data::{DataTypeFunctions, Marker};

#[solidus::wrap(class = "Container", mark)]
struct Container {
    items: Vec<BoxValue<Value>>,
}

impl Container {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn push(&mut self, value: BoxValue<Value>) {
        self.items.push(value);
    }
}

// When using `mark`, you must implement the mark callback
impl DataTypeFunctions for Container {
    fn mark(&self, marker: &Marker) {
        for item in &self.items {
            marker.mark_boxed(item);
        }
    }
}
```

The `mark` callback tells Ruby's GC that the values inside the container are reachable. 
This provides defense-in-depth: even though `BoxValue` registers with the GC, the mark 
callback ensures proper behavior during the mark phase.

## Accessing the Value Inside BoxValue

`BoxValue<T>` implements `Deref` and `DerefMut`, allowing transparent access to the 
inner value's methods:

```rust
use solidus::{BoxValue, RString};

let boxed = RString::new("hello").into_box();

// Call methods directly through Deref
let len = boxed.len();  // Calls RString::len()
let s = boxed.to_string().unwrap();

// Get a reference to the inner value
let inner: &RString = &*boxed;
```

### Getting a Clone

Use `.get()` to get a clone of the inner value:

```rust
let boxed = RString::new("hello").into_box();
let cloned: RString = boxed.get();  // Returns a clone
```

### Consuming the BoxValue

Use `.into_inner()` to consume the `BoxValue` and get the inner value:

```rust
let boxed = RString::new("hello").into_box();
let inner: RString = boxed.into_inner();
// boxed is consumed; inner is no longer GC-protected
// You must ensure inner stays on the stack or re-register it
```

**Warning**: After calling `.into_inner()`, the returned value is no longer protected 
from GC. You should immediately pin it on the stack or wrap it in another `BoxValue`.

## When to Use Stack Pinning vs BoxValue

| Scenario | Use |
|----------|-----|
| Temporary value in a function | `pin_on_stack!` |
| Passing value to another function | `Pin<&StackPinned<T>>` |
| Storing in a collection | `BoxValue<T>` |
| Field in a TypedData struct | `BoxValue<T>` |
| Returning from a function | `PinGuard<T>` (caller decides) |
| Global/static storage | `BoxValue<T>` or `gc::register_mark_object` |

### Stack Pinning (Common Case)

Most Ruby values should be stack-pinned. This is the default and most efficient approach:

```rust
use solidus::{RString, pin_on_stack};

fn process_string() {
    pin_on_stack!(s = RString::new("hello"));
    // Use s within this function
    // When the function returns, s is automatically cleaned up
}
```

### BoxValue (When Heap Storage is Needed)

Use `BoxValue` when you need the value to outlive the current stack frame or when 
storing in collections:

```rust
use solidus::{BoxValue, RString};

struct MyProcessor {
    cached_values: Vec<BoxValue<RString>>,
}

impl MyProcessor {
    fn cache(&mut self, value: BoxValue<RString>) {
        self.cached_values.push(value);
    }
}
```

## Performance Considerations

`BoxValue` has overhead compared to stack pinning:

1. **Heap allocation**: Each `BoxValue` allocates memory on the heap
2. **GC registration**: `rb_gc_register_address()` is called on creation
3. **GC unregistration**: `rb_gc_unregister_address()` is called on drop

For most applications, this overhead is negligible. However, in hot paths with many 
short-lived values, prefer stack pinning:

```rust
use solidus::{RString, pin_on_stack};

// Good: Stack pinning in a loop
for i in 0..1000 {
    pin_on_stack!(s = RString::new(&format!("item_{}", i)));
    process(&s);
    // s is cleaned up at each iteration, no heap allocation
}

// Avoid: Boxing in a hot loop when not needed
for i in 0..1000 {
    let boxed = RString::new(&format!("item_{}", i)).into_box();
    process_boxed(&boxed);
    // Extra heap allocation and GC registration per iteration
}
```

### When BoxValue Overhead is Worth It

- Storing values that need to persist across function calls
- Building data structures that Ruby will query later
- Caching computed values
- Any situation where stack pinning isn't possible

## Summary

- **Problem**: Ruby's GC only scans the stack; heap-stored VALUES can be collected
- **Solution**: `BoxValue<T>` registers VALUES with Ruby's GC
- **Creation**: Use `guard.into_box()` or `BoxValue::new(value)`
- **Access**: Use `Deref`/`DerefMut` or `.get()`/`.into_inner()`
- **Collections**: `Vec<BoxValue<T>>`, `HashMap<K, BoxValue<T>>`, etc.
- **Performance**: Prefer stack pinning; use `BoxValue` only when needed

## See Also

- [Pinning](pinning.md) - Why Ruby values need pinning
- [TypedData](typed-data.md) - Wrapping Rust structs with Ruby values
- [`BoxValue` API docs](https://docs.rs/solidus/latest/solidus/value/struct.BoxValue.html)
