# Kickoff – M5c-code (First-run UX wiring)

## Role
Code agent: production code/scripts only. No tests. Do not edit docs/tasks/session logs from the worktree.

## Goal
Implement M5c-spec: wire first-run/provision flows so macOS installs “feel like host” out of the box, with coherent guidance across installer/health/doctor.

## Read first
- `docs/project_management/next/p0-platform-stability-macOS-parity/plan.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/tasks.json`
- `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/M5c-spec.md`
- `docs/INSTALLATION.md`
- `docs/USAGE.md`

## Start checklist (must follow)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Set `M5c-code` to `in_progress` in `tasks.json` (orchestration branch only).
3. Add START entry to `session_log.md`; commit docs (`docs: start M5c-code`).
4. Create branch `mp-m5c-world-deps-first-run-code`, then worktree: `git worktree add wt/mp-m5c-world-deps-first-run-code mp-m5c-world-deps-first-run-code`.
5. Do not edit docs/tasks/session_log from the worktree.

## Constraints / guardrails
- Do not change tool inventory/layering (M5a) or host detection semantics (M5b) except as required to wire UX.
- Keep output actionable and avoid noisy debug logs in normal paths.

## Required commands (record output in END entry)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End checklist
1. Run required commands and capture outputs.
2. Commit changes inside the worktree (no docs edits).
3. Switch back to orchestration branch; mark task completed; add END entry; commit docs (`docs: finish M5c-code`).
4. Remove worktree `wt/mp-m5c-world-deps-first-run-code`.

