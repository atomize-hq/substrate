# Kickoff: C1-integ-core (integration core)

## Scope
- Merge code + tests, reconcile to spec, and make the slice green on the primary dev platform before CI dispatch.
- Spec: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/C1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/policy-patch-only-broker-effective-resolution-c1-integ-core` on branch `policy-patch-only-broker-effective-resolution-c1-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan, tasks, session log, spec, this prompt.

## Requirements
- Spec wins over implementation details.
- Run local integration gates:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Dispatch validation:
  - CI compile parity (GitHub-hosted): `make ci-compile-parity CI_WORKFLOW_REF="feat/policy-patch-only-broker-effective-resolution" CI_REMOTE=origin CI_CLEANUP=1`
  - Behavioral smoke (self-hosted): `make feature-smoke FEATURE_DIR="docs/project_management/_archived/policy-patch-only-broker-effective-resolution" PLATFORM=behavior SMOKE_SLICE_ID="C1" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/policy-patch-only-broker-effective-resolution" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C1-integ-core"`.
2. Hand off compile parity and smoke run ids/URLs to the operator.

