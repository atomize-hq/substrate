# Kickoff: WO0-code (Stable mount topology + strategy probe + fallback metadata)

## Scope
- Production code only; no tests.
- ADR: `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
- Spec: `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/woe-wo0-code` on branch `woe-wo0-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-overlayfs-enumeration/plan.md`, `docs/project_management/next/world-overlayfs-enumeration/tasks.json`, `docs/project_management/next/world-overlayfs-enumeration/session_log.md`, ADR, spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/world-overlayfs-enumeration" SLICE_ID="WO0"`

## Requirements
- Implement the project-isolation mount choreography change required by ADR-0004 (`mount --move`, not `mount --bind`).
- Implement kernel overlayfs → fuse-overlayfs retry behavior driven by the enumeration probe contract in ADR-0004.
- Emit the required trace fields and doctor JSON keys from ADR-0004 (additive only; do not break existing consumers).
- Implement the required warning-line contract for world-optional fallback to host.

## Commands (required)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands and capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WO0-code"`.
3. On the orchestration branch, update `tasks.json` and add the END entry to `session_log.md`; commit docs (`docs: finish WO0-code`).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
