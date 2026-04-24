# Kickoff: AHCSITC2-integ-core (integration core)

## Scope
- Merge the AHCSITC2 code and test branches, resolve drift to spec, and make the slice green on the primary development platform before `CP1-ci-checkpoint`.
- Spec: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC2/AHCSITC2-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/agent-hub-core-successor-identity-tuple-compatible-ahcsitc2-integ-core` on branch `agent-hub-core-successor-identity-tuple-compatible-ahcsitc2-integ-core` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `AHCSITC2-spec.md`, and this prompt.
3. If the worktree metadata is missing or mismatched, stop and ask the operator to rerun `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible" TASK_ID="AHCSITC2-integ-core"`.

## Requirements
- Merge the `AHCSITC2-code` and `AHCSITC2-test` task branches into this worktree.
- Reconcile code and tests to the spec. The spec wins if drift appears.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
- Do not dispatch checkpoint CI from this task. Finish this branch first, then run `CP1-ci-checkpoint` from the orchestration checkout.

## End Checklist
1. Run the required commands and capture outputs.
2. From inside the worktree, run `make triad-task-finish TASK_ID="AHCSITC2-integ-core"`.
3. Hand off the checkpoint preconditions and key validation outputs to the operator.
