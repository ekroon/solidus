# Phase 3 Methods Example

This example demonstrates the complete method registration system in Solidus Phase 3,
showing all the different ways to register Rust functions as Ruby methods.

## What's Working

This example successfully demonstrates:

- **Instance methods** on String class (extending built-in types)
- **Class methods** (singleton methods) on custom classes
- **Module functions** that can be called on modules
- **Global functions** available everywhere
- Various arities (0-3 arguments)
- Error handling and propagation
- Manual Init_ function registration

## Building

```bash
cd examples/phase3_methods
cargo build
```

This creates `target/debug/libphase3_methods.dylib` (or `.so` on Linux, `.dll` on Windows).

## Testing

```bash
# Create symlink for Ruby to load
cd target/debug
ln -sf libphase3_methods.dylib phase3_methods.bundle
cd ../..

# Run a simple test
ruby -e "require './target/debug/phase3_methods'; puts hello()"
```

Expected output:
```
Hello from Solidus!
```

## Implementation Details

### Instance Methods

Instance methods are defined with `method!` macro and include `rb_self`:

```rust
fn greet(rb_self: RString) -> Result<RString, Error> {
    let name = rb_self.to_string()?;
    Ok(RString::new(&format!("Hello, {}!", name)))
}

// Register
calc_rclass.define_method("greet", solidus::method!(greet, 0), 0)?;
```

### Class Methods (Singleton Methods)

Class methods use `function!` macro (no `rb_self`) and `define_singleton_method`:

```rust
fn create_default() -> Result<RString, Error> {
    Ok(RString::new("Calculator"))
}

// Register
calc_rclass.define_singleton_method(
    "create_default",
    solidus::function!(create_default, 0),
    0
)?;
```

### Module Functions

Module functions can be called as `Module.function`:

```rust
fn get_version() -> Result<RString, Error> {
    Ok(RString::new("1.0.0"))
}

// Register
string_utils_rmodule.define_module_function(
    "get_version",
    solidus::function!(get_version, 0),
    0
)?;
```

### Global Functions

Global functions are available everywhere:

```rust
fn hello() -> Result<RString, Error> {
    Ok(RString::new("Hello from Solidus!"))
}

// Register
ruby.define_global_function("hello", solidus::function!(hello, 0), 0)?;
```

### Manual Init Function

Due to Rust 2024 edition compatibility issues with the `#[solidus::init]` macro,
this example uses a manual `Init_` function:

```rust
#[no_mangle]
pub unsafe extern "C" fn Init_phase3_methods() {
    Ruby::mark_ruby_thread();
    let ruby = Ruby::get();
    if let Err(e) = init_solidus(ruby) {
        e.raise();
    }
}
```

## Known Limitations

1. **String Type Conversion**: There's currently an issue with Ruby string type detection
   in some contexts. Global functions expecting RString arguments may fail with
   "expected String" errors. This is being investigated.

2. **Immediate Values**: The current `function!` macro implementation wraps all arguments
   with `Pin<&StackPinned<T>>`, which doesn't work for immediate values like `i64`.
   This example works around it by passing all numeric arguments as strings.

3. **Macro Safety**: The `#[solidus::init]` macro generates code that requires `unsafe`
   markers in Rust 2024 edition. This example uses a manual Init function instead.

## Code Organization

```
phase3_methods/
├── Cargo.toml          # Extension gem configuration  
├── build.rs            # Ruby extension build script
├── src/
│   └── lib.rs          # Main extension code with all methods
├── test.rb             # Test script (partial - see known limitations)
└── README.md           # This file
```

## What This Example Demonstrates

Despite the known limitations, this example successfully shows:

1. **Complete method registration flow** - from Rust function to Ruby method
2. **Different method types** - instance, class, module, and global
3. **Error handling** - propagating Rust errors to Ruby exceptions
4. **Stack pinning** - using `Pin<&StackPinned<T>>` for heap-allocated arguments
5. **Return value conversion** - converting Rust types back to Ruby values
6. **Manual initialization** - registering the extension with Ruby

## Future Work

- Fix string type detection for function arguments
- Add support for immediate value arguments without pinning
- Resolve Rust 2024 edition compatibility for the `#[solidus::init]` macro
- Complete comprehensive test coverage

## Related Documentation

- [Phase 3 Tasks](../../docs/plan/phase-3-tasks.md) - Implementation details
- [Method Examples](../../crates/solidus/src/method/examples.md) - More examples
- [Main README](../../README.md) - Project overview

## License

MIT
