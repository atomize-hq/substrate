# Kickoff: C0-code (E2E smoke)

Do not edit planning docs inside the worktree.

Goal:
- Create a new workspace member crate at `crates/triad_e2e_smoke_demo/`.
- Implement `pub fn answer() -> u32 { 42 }` in `src/lib.rs`.
- Add the crate to the workspace (root `Cargo.toml` members).

Constraints:
- Production code only; do not add tests in this task.
- Keep changes minimal and deterministic.

Required:
- Run `cargo fmt`
- Run `cargo clippy --workspace --all-targets -- -D warnings`

Finish:
- From inside this worktree run: `make triad-task-finish TASK_ID="C0-code"`
