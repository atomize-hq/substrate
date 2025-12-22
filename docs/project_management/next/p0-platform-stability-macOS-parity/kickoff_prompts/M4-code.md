# Kickoff – M4-code (World deps base manifest parity)

## Role
Code agent: production code/scripts only. No tests. Do not edit docs/tasks/session logs from the worktree.

## Goal
Implement M4-spec so `substrate world deps` uses the installed manifest by default (and retains explicit overrides).

## Read first
- `docs/project_management/next/p0-platform-stability-macOS-parity/plan.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/tasks.json`
- `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/M4-spec.md`
- `docs/CONFIGURATION.md`

## Start checklist (must follow)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Set `M4-code` to `in_progress` in `tasks.json` (orchestration branch only).
3. Add START entry to `session_log.md`; commit docs (`docs: start M4-code`).
4. Create branch `mp-m4-world-deps-manifest-code`, then worktree: `git worktree add wt/mp-m4-world-deps-manifest-code mp-m4-world-deps-manifest-code`.
5. Do not edit docs/tasks/session_log from the worktree.

## Constraints / guardrails
- Do not change tool lists or recipes; this triad is about manifest resolution + overrides only.
- Keep behavior consistent across `world deps status/install/sync`.
- Preserve `SUBSTRATE_WORLD_DEPS_MANIFEST` override semantics.

## Required commands (record output in END entry)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End checklist
1. Run required commands and capture outputs.
2. Commit changes inside the worktree (no docs edits).
3. Switch back to orchestration branch; mark task completed; add END entry; commit docs (`docs: finish M4-code`).
4. Remove worktree `wt/mp-m4-world-deps-manifest-code`.

