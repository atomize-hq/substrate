# Kickoff: WCU3-integ (integration)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green.
- Spec: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Gate file: `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
- Closeout report: `docs/project_management/next/workspace-config-policy-unification/WCU3-closeout_report.md`

Do not edit planning docs inside the worktree.

## Requirements
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Run smoke scripts for behavior platforms and record evidence:
  - `docs/project_management/next/workspace-config-policy-unification/smoke/linux-smoke.sh`
  - `docs/project_management/next/workspace-config-policy-unification/smoke/macos-smoke.sh`
  - `docs/project_management/next/workspace-config-policy-unification/smoke/windows-smoke.ps1`

## End Checklist
1. Complete `WCU3-closeout_report.md` with Phase B evidence and smoke results.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU3-integ"`
