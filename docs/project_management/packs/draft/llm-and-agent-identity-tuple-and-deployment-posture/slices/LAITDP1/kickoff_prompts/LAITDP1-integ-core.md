# Kickoff: LAITDP1-integ-core (integration core)

## Scope
- Merge the LAITDP1 code and test branches and make the core integration branch green before CP1.
- Spec: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP1/LAITDP1-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/llm-and-agent-identity-tuple-and-deployment-posture-laitdp1-integ-core` on branch `llm-and-agent-identity-tuple-and-deployment-posture-laitdp1-integ-core` and that `.taskmeta.json` exists.
2. Read: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/plan.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`, the spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" TASK_ID="LAITDP1-integ-core"`

## Requirements
- Reconcile code and tests to the spec. The spec wins.
- Merge the code and test branches into this worktree.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
- Finish this task before running `CP1-ci-checkpoint`.

## End Checklist
1. Ensure the merged state is committed and local integration gates are green.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LAITDP1-integ-core"`.
3. Hand off the local integration commands and outcomes to the operator.
4. Do not delete the worktree.
