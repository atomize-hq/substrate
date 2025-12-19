# Kickoff – M5c-integ (First-run UX wiring)

## Role
Integration agent: merge code+tests, reconcile to spec, and own final green. Do not edit docs/tasks/session logs from the worktree.

## Goal
Integrate M5c-code + M5c-test and ensure behavior matches M5c-spec; gate with fmt/clippy/tests and finish with `make preflight`.

## Read first
- `docs/project_management/next/p0-platform-stability-macOS-parity/plan.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/tasks.json`
- `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/M5c-spec.md`

## Start checklist (must follow)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Set `M5c-integ` to `in_progress` in `tasks.json` (orchestration branch only).
3. Add START entry to `session_log.md`; commit docs (`docs: start M5c-integ`).
4. Create branch `mp-m5c-world-deps-first-run-integ`, then worktree: `git worktree add wt/mp-m5c-world-deps-first-run-integ mp-m5c-world-deps-first-run-integ`.
5. Do not edit docs/tasks/session_log from the worktree.

## Required commands (record output in END entry)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Relevant `cargo test ...`
- `make preflight` (required)

## End checklist
1. Merge `mp-m5c-world-deps-first-run-code` and `mp-m5c-world-deps-first-run-test` into the integration worktree; reconcile to spec.
2. Run required commands and capture outputs.
3. Commit integration changes and fast-forward merge into `feat/p0-platform-stability-macOS-parity`.
4. Update docs on orchestration branch: mark task completed; END entry; commit (`docs: finish M5c-integ`).
5. Remove worktree `wt/mp-m5c-world-deps-first-run-integ`.

