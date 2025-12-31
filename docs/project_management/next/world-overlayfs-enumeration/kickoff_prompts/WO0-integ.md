# Kickoff: WO0-integ (Integration: land WO0 with smoke + playbook validation)

## Scope
- Merge `WO0-code` + `WO0-test`, reconcile drift to spec, and make the slice green.
- ADR: `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
- Spec: `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`
- Closeout gate report: `docs/project_management/next/world-overlayfs-enumeration/WO0-closeout_report.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/woe-wo0-integ` on branch `woe-wo0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-overlayfs-enumeration/plan.md`, `docs/project_management/next/world-overlayfs-enumeration/tasks.json`, `docs/project_management/next/world-overlayfs-enumeration/session_log.md`, ADR, spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world-overlayfs-enumeration" TASK_ID="WO0-integ"`
4. Merge `woe-wo0-code` and `woe-wo0-test` into this worktree and resolve drift to spec (spec wins).

## Commands (required)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Relevant `cargo test ...` suites (at minimum the ones added or modified by `WO0-test`)
- `docs/project_management/next/world-overlayfs-enumeration/smoke/linux-smoke.sh`
- `make integ-checks`

## Requirements
- Run the manual playbook: `docs/project_management/next/world-overlayfs-enumeration/manual_testing_playbook.md`.
- Fill the closeout report: `docs/project_management/next/world-overlayfs-enumeration/WO0-closeout_report.md`.

## End Checklist
1. Run required commands and capture outputs (including smoke results).
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WO0-integ"`.
3. On the orchestration branch, update `tasks.json` and add the END entry to `session_log.md`; commit docs (`docs: finish WO0-integ`).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
