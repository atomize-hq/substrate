# Kickoff: LAITDP0-test (test)

## Scope
- Tests only. No production code.
- Spec: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP0/LAITDP0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/llm-and-agent-identity-tuple-and-deployment-posture-laitdp0-test` on branch `llm-and-agent-identity-tuple-and-deployment-posture-laitdp0-test` and that `.taskmeta.json` exists.
2. Read: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/plan.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`, the spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" SLICE_ID="LAITDP0"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" TASK_ID="LAITDP0-test"`

## Requirements
- Add or modify tests that enforce the spec.
- Keep the task focused on the slice acceptance criteria.
- Run: `cargo fmt` and the targeted tests you add or touch.

## End Checklist
1. Run the required commands and capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LAITDP0-test"`.
3. Hand off the targeted test commands and outcomes to the operator.
4. Do not delete the worktree.
