# Kickoff: LAITDP0-integ (integration final)

## Scope
- Merge the LAITDP0 code and test branches and make the slice green.
- Spec: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP0/LAITDP0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/llm-and-agent-identity-tuple-and-deployment-posture-laitdp0-integ` on branch `llm-and-agent-identity-tuple-and-deployment-posture-laitdp0-integ` and that `.taskmeta.json` exists.
2. Read: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/plan.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`, the spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" TASK_ID="LAITDP0-integ"`

## Requirements
- Reconcile code and tests to the spec. The spec wins.
- Merge the code and test branches into this worktree.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
- Cross-platform checkpoint tasks do not run from this task.

## End Checklist
1. Ensure the merged state is committed and local integration gates are green.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LAITDP0-integ"`.
3. Hand off the local integration commands and outcomes to the operator.
4. Do not delete the worktree.
