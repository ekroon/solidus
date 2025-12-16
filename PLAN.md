# Solidus Implementation Plan

## Vision

Solidus is a Rust library for writing Ruby extensions that solves Magnus's fundamental 
safety limitation: **users no longer need to think about keeping Ruby values on the stack**.

### The Problem

In Magnus, storing Ruby values on the heap is undefined behavior:

```rust
// This is UB in Magnus - values moved to heap, invisible to Ruby GC
let values: Vec<Value> = vec![ruby.str_new("hello")];
```

Users must manually ensure values stay on the stack, which is:
- Error-prone
- Not enforced by the type system
- Not visible at the API level

### The Solution

Solidus uses Rust's `Pin` type to enforce stack locality at compile time:

```rust
// Method arguments are automatically stack-pinned
fn concat(rb_self: RString, other: Pin<&StackPinned<RString>>) -> Result<RString, Error> {
    // `other` cannot be moved to heap - enforced by type system
}

// Explicit opt-in for heap allocation
fn store(arr: Pin<&StackPinned<RArray>>) -> Result<(), Error> {
    let boxed: BoxValue<RString> = BoxValue::new(arr.get().entry(0)?);  // GC-registered
    // boxed can safely be stored in Vec, HashMap, etc.
}
```

## Phases Overview

| Phase | Name | Description |
|-------|------|-------------|
| 0 | [Bootstrap](docs/plan/phase-0-bootstrap.md) | Project scaffolding, CI, licensing |
| 1 | [Foundation](docs/plan/phase-1-foundation.md) | Core types: Value, StackPinned, BoxValue, Error |
| 2 | [Types](docs/plan/phase-2-types.md) | Ruby types: RString, RArray, RHash, Integer, etc. |
| 3 | [Methods](docs/plan/phase-3-methods.md) | method!, function! macros with pinning |
| 4 | [TypedData](docs/plan/phase-4-typed-data.md) | #[wrap], TypedData for Rust types in Ruby |
| 5 | [Polish](docs/plan/phase-5-polish.md) | Documentation, examples, testing |
| 6 | [Safety Validation](docs/plan/phase-6-safety-validation.md) | Tests confirming we prevent Magnus's UB |

## Architecture Decisions

Key decisions are documented in [docs/plan/decisions.md](docs/plan/decisions.md).

## Success Criteria

1. **Safety**: Impossible to accidentally move Ruby values to heap without explicit `BoxValue`
2. **Ergonomics**: Common patterns are concise and intuitive
3. **Performance**: No overhead compared to Magnus for equivalent operations
4. **Documentation**: All public APIs documented with examples
5. **Testing**: Comprehensive test coverage including Ruby integration tests
6. **Validation**: Demonstrable prevention of undefined behavior that Magnus allows

## Non-Goals (Initial Release)

- API compatibility with Magnus
- `embed` feature (embedding Ruby in Rust)
- Support for Ruby < 3.4
- Support for Rust < latest stable

## Dependencies

- `rb-sys` (>= 0.9.113): Low-level Ruby bindings
- `rb-sys-env`: Build-time Ruby configuration

## File Structure

```
solidus/
├── AGENTS.md
├── PLAN.md
├── Cargo.toml                    # Workspace root
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── .github/
│   └── workflows/
│       └── ci.yml
├── crates/
│   ├── solidus/                  # Main library
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ruby.rs
│   │       ├── error.rs
│   │       ├── gc.rs
│   │       ├── value/
│   │       ├── types/
│   │       ├── convert/
│   │       ├── method/
│   │       └── typed_data/
│   └── solidus-macros/           # Proc macros
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── examples/
│   └── README.md
├── tests/
│   ├── integration/
│   └── ruby/
└── docs/
    ├── guide/
    │   └── README.md
    └── plan/
        ├── phase-0-bootstrap.md
        ├── phase-1-foundation.md
        ├── phase-2-types.md
        ├── phase-3-methods.md
        ├── phase-4-typed-data.md
        ├── phase-5-polish.md
        ├── phase-6-safety-validation.md
        └── decisions.md
```
