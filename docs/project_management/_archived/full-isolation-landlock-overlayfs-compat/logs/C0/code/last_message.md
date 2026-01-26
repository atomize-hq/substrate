- Implemented runtime derivation of overlayfs `upperdir`/`workdir` from `/proc/self/mountinfo` and extended the full-isolation Landlock write allowlist with those dirs; if Landlock is supported but derivation fails, the exec now fails closed with exit code `4` and an actionable hint (`crates/world-agent/src/internal_exec.rs:42`, `crates/world-agent/src/internal_exec.rs:153`).
- Added a reusable mountinfo parser/helper in `crates/world/src/mountinfo.rs:40` and exported it from `crates/world/src/lib.rs:17`.

- Baseline tests (pre-change): `cargo test -p world-agent --lib -- --nocapture` (pass, exit `0`); `cargo test -p world --lib -- --nocapture` (pass, exit `0`).
- Validation (post-change): `cargo fmt` (ok); `cargo clippy --workspace --all-targets -- -D warnings` (ok); re-ran the same baseline tests (pass, exit `0`).
- Finished task: `make triad-task-finish TASK_ID="C0-code"` (exit `0`), HEAD `c9383284d249bdd0cb7b2ac864b68a6ede2ef0b1`.