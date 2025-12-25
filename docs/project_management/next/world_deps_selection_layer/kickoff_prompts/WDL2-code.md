# Kickoff: WDL2-code (System packages provisioning)

## Scope
- Implement the `substrate world deps provision` action and provisioning gating per `S2-spec-system-packages-provisioning.md`.
- Production code only; do not add or modify tests.

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`, and this prompt.
3. Set `WDL2-code` status to `in_progress` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add a START entry to `docs/project_management/next/world_deps_selection_layer/session_log.md`; commit docs (`docs: start WDL2-code`).
4. Create branch and worktree:
   - `git checkout -b ws-wdl2-provision-code`
   - `git worktree add wt/wdl2-provision-code ws-wdl2-provision-code`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Primary implementation targets:
  - `crates/shell/src/builtins/world_deps/*`
- Hard requirement: Linux host must not mutate OS packages; `provision` must fail with exit 4 and actionable manual guidance.

## Required Commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/world-sync` (ff-only).
4. Update `docs/project_management/next/world_deps_selection_layer/tasks.json` + `docs/project_management/next/world_deps_selection_layer/session_log.md` (END entry), commit docs (`docs: finish WDL2-code`).
5. Remove worktree.
