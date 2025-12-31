# C0-spec (E2E triad smoke)

## Scope
- Add a new workspace member crate `crates/triad_e2e_smoke_demo/`.
- Expose `pub fn answer() -> u32` returning `42`.
- Add a minimal test proving `answer() == 42`.

## Behavior
- The crate builds on Linux/macOS/Windows.
- `cargo test -p triad_e2e_smoke_demo` passes.

## Acceptance criteria
- `crates/triad_e2e_smoke_demo/Cargo.toml` exists and is a valid Rust crate.
- `crates/triad_e2e_smoke_demo/src/lib.rs` defines `answer()` returning `42`.
- `crates/triad_e2e_smoke_demo/tests/answer.rs` asserts `answer() == 42`.
- `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings` succeed.

## Out of scope
- Any behavior changes to existing Substrate functionality.
