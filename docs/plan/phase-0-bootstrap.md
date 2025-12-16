# Phase 0: Bootstrap

## Objective

Set up the project scaffolding, CI pipeline, and basic documentation structure.

## Tasks

### 0.1 Initialize Project Structure

- [ ] Create workspace `Cargo.toml`
- [ ] Create `crates/solidus/Cargo.toml` with rb-sys dependency
- [ ] Create `crates/solidus-macros/Cargo.toml` as proc-macro crate
- [ ] Create placeholder `lib.rs` files that compile
- [ ] Create `build.rs` for solidus crate (rb-sys-env integration)

### 0.2 Licensing

- [ ] Create `LICENSE-MIT`
- [ ] Add license headers to Cargo.toml files

### 0.3 Documentation Structure

- [ ] Create `README.md` with project overview and badges
- [ ] Create `AGENTS.md` (this file)
- [ ] Create `docs/guide/README.md` placeholder
- [ ] Create `docs/plan/` directory with phase files

### 0.4 CI Pipeline

- [ ] Create `.github/workflows/ci.yml`:
  - Build on Linux, macOS, Windows
  - Test against Ruby 3.4
  - Run `cargo fmt --check`
  - Run `cargo clippy`
  - Run `cargo test`
  - Run `cargo doc`

### 0.5 Development Setup

- [ ] Create `.gitignore`
- [ ] Create `rust-toolchain.toml` (optional, pin to stable)
- [ ] Verify `cargo build` works
- [ ] Verify `cargo test` works (even if no tests yet)

## Acceptance Criteria

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` succeeds
- [ ] CI pipeline passes
- [ ] Documentation structure in place

## Files to Create

```
solidus/
├── .github/workflows/ci.yml
├── .gitignore
├── Cargo.toml
├── LICENSE-MIT
├── README.md
├── AGENTS.md
├── PLAN.md
├── crates/
│   ├── solidus/
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/lib.rs
│   └── solidus-macros/
│       ├── Cargo.toml
│       └── src/lib.rs
└── docs/
    ├── guide/README.md
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
