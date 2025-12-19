# Kickoff – M4-integ (World deps base manifest parity)

## Role
Integration agent: merge code+tests, reconcile to spec, and own final green. Do not edit docs/tasks/session logs from the worktree.

## Goal
Integrate M4-code + M4-test and ensure behavior matches M4-spec; gate with fmt/clippy/tests and finish with `make preflight`.

## Read first
- `docs/project_management/next/p0-platform-stability-macOS-parity/plan.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/tasks.json`
- `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/M4-spec.md`

## Start checklist (must follow)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Set `M4-integ` to `in_progress` in `tasks.json` (orchestration branch only).
3. Add START entry to `session_log.md`; commit docs (`docs: start M4-integ`).
4. Create branch `mp-m4-world-deps-manifest-integ`, then worktree: `git worktree add wt/mp-m4-world-deps-manifest-integ mp-m4-world-deps-manifest-integ`.
5. Do not edit docs/tasks/session_log from the worktree.

## Required commands (record output in END entry)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Relevant `cargo test ...`
- `make preflight` (required)

## End checklist
1. Merge `mp-m4-world-deps-manifest-code` and `mp-m4-world-deps-manifest-test` into the integration worktree; reconcile to spec.
2. Run required commands and capture outputs.
3. Commit integration changes and fast-forward merge into `feat/p0-platform-stability-macOS-parity`.
4. Update docs on orchestration branch: mark task completed; END entry; commit (`docs: finish M4-integ`).
5. Remove worktree `wt/mp-m4-world-deps-manifest-integ`.

