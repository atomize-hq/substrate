# Kickoff: LAITDP2-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/LAITDP2-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/llm-and-agent-identity-tuple-and-deployment-posture-laitdp2-code` on branch `llm-and-agent-identity-tuple-and-deployment-posture-laitdp2-code` and that `.taskmeta.json` exists.
2. Read: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/plan.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`, the spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" SLICE_ID="LAITDP2"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" TASK_ID="LAITDP2-code"`

## Requirements
- Implement exactly the behaviors in the spec.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`.
- Do not add new tests or new test files.
- Run a targeted baseline test set before changes and rerun the same test set after changes.

## End Checklist
1. Run the required commands and capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LAITDP2-code"`.
3. Hand off the baseline test commands and outcomes to the operator.
4. Do not delete the worktree.
