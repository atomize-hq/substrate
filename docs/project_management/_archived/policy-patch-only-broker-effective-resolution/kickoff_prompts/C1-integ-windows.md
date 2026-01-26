# Kickoff: C1-integ-windows (integration platform-fix: windows)

## Scope
- Address Windows-only issues discovered by smoke or CI parity for the C1 slice.
- Spec: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/C1-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/policy-patch-only-broker-effective-resolution-c1-integ-windows` on branch `policy-patch-only-broker-effective-resolution-c1-integ-windows` and that `.taskmeta.json` exists at the worktree root.
2. Read: spec, tasks, this prompt.

## End Checklist
1. Dispatch Windows smoke via CI: `make feature-smoke FEATURE_DIR="docs/project_management/_archived/policy-patch-only-broker-effective-resolution" PLATFORM=windows SMOKE_SLICE_ID="C1" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/policy-patch-only-broker-effective-resolution" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C1-integ-windows"`.

