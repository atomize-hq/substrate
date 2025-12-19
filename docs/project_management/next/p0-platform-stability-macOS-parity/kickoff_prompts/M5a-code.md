# Kickoff – M5a-code (World deps inventory & layering)

## Role
Code agent: production code/scripts only. No tests. Do not edit docs/tasks/session logs from the worktree.

## Goal
Implement M5a-spec: align the world deps tool inventory with shim doctor/health and make `world-deps.yaml` (installed + user overlay) behave as an override layer.

## Read first
- `docs/project_management/next/p0-platform-stability-macOS-parity/plan.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/tasks.json`
- `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/M5a-spec.md`
- `docs/USAGE.md`
- `docs/CONFIGURATION.md`

## Start checklist (must follow)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Set `M5a-code` to `in_progress` in `tasks.json` (orchestration branch only).
3. Add START entry to `session_log.md`; commit docs (`docs: start M5a-code`).
4. Create branch `mp-m5a-world-deps-inventory-code`, then worktree: `git worktree add wt/mp-m5a-world-deps-inventory-code mp-m5a-world-deps-inventory-code`.
5. Do not edit docs/tasks/session_log from the worktree.

## Constraints / guardrails
- Do not change host detection semantics (that is M5b).
- Do not change installer/provision UX (that is M5c).
- Keep layering explicit: base inventory + installed overlay + user overlay.

## Required commands (record output in END entry)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End checklist
1. Run required commands and capture outputs.
2. Commit changes inside the worktree (no docs edits).
3. Switch back to orchestration branch; mark task completed; add END entry; commit docs (`docs: finish M5a-code`).
4. Remove worktree `wt/mp-m5a-world-deps-inventory-code`.

