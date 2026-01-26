# Kickoff: C0-integ-linux (integration platform-fix: linux)

## Scope
- Address Linux-only issues discovered by smoke or CI parity for the C0 slice.
- Spec: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/C0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/policy-patch-only-broker-effective-resolution-c0-integ-linux` on branch `policy-patch-only-broker-effective-resolution-c0-integ-linux` and that `.taskmeta.json` exists at the worktree root.
2. Read: spec, tasks, this prompt.

## End Checklist
1. Dispatch Linux smoke via CI: `scripts/ci/dispatch_feature_smoke.sh --feature-dir "docs/project_management/_archived/policy-patch-only-broker-effective-resolution" --runner-kind self-hosted --platform linux --workflow-ref "feat/policy-patch-only-broker-effective-resolution" --cleanup`
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ-linux"`.

