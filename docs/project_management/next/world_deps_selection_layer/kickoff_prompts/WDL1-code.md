# Kickoff: WDL1-code (Install classes)

## Scope
- Implement install-class routing and manifest schema changes per `S1-spec-install-classes.md`.
- Production code only; do not add or modify tests.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`, `docs/project_management/next/world_deps_selection_layer/decision_register.md`, and this prompt.
3. Set `WDL1-code` status to `in_progress` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add a START entry to `docs/project_management/next/world_deps_selection_layer/session_log.md`; commit docs (`docs: start WDL1-code`).
4. Create branch and worktree:
   - `git checkout -b ws-wdl1-install-classes-code`
   - `git worktree add wt/wdl1-install-classes-code ws-wdl1-install-classes-code`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Primary implementation targets:
  - `crates/common/src/manager_manifest/schema.rs` (schema bump + validation per S1)
  - `crates/common/src/world_deps_manifest.rs`
  - `crates/shell/src/builtins/world_deps/*` (routing and UX)
  - Substrate-owned manifests (update as required by S1):
    - `config/manager_hooks.yaml`
    - `scripts/substrate/world-deps.yaml`
- Hard requirement: runtime `substrate world deps sync` must never execute OS package managers.

## Required Commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/world-sync` (ff-only).
4. Update `docs/project_management/next/world_deps_selection_layer/tasks.json` + `docs/project_management/next/world_deps_selection_layer/session_log.md` (END entry), commit docs (`docs: finish WDL1-code`).
5. Remove worktree.
