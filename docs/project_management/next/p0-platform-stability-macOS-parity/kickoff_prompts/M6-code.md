# Kickoff – M6-code (World deps failure safety)

## Role
Code agent: production code/scripts only. No tests. Do not edit docs/tasks/session logs from the worktree.

## Goal
Implement M6-spec so macOS world deps cannot silently fall back to host execution for install/sync, and status clearly reports guest unavailability.

## Read first
- `docs/project_management/next/p0-platform-stability-macOS-parity/plan.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/tasks.json`
- `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/M6-spec.md`

## Start checklist (must follow)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Set `M6-code` to `in_progress` in `tasks.json` (orchestration branch only).
3. Add START entry to `session_log.md`; commit docs (`docs: start M6-code`).
4. Create branch `mp-m6-world-deps-safety-code`, then worktree: `git worktree add wt/mp-m6-world-deps-safety-code mp-m6-world-deps-safety-code`.
5. Do not edit docs/tasks/session_log from the worktree.

## Constraints / guardrails
- Keep behavior macOS-scoped where appropriate (avoid changing Linux/WSL behavior unless required by the spec).
- Ensure output remains actionable (doctor command, forwarding hints).

## Required commands (record output in END entry)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End checklist
1. Run required commands and capture outputs.
2. Commit changes inside the worktree (no docs edits).
3. Switch back to orchestration branch; mark task completed; add END entry; commit docs (`docs: finish M6-code`).
4. Remove worktree `wt/mp-m6-world-deps-safety-code`.

