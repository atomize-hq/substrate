# Kickoff: LAITDP2-integ (integration final)

## Scope
- Merge the LAITDP2 core and platform-fix branches and finalize the slice.
- Spec: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/LAITDP2-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/llm-and-agent-identity-tuple-and-deployment-posture-laitdp2-integ` on branch `llm-and-agent-identity-tuple-and-deployment-posture-laitdp2-integ` and that `.taskmeta.json` exists.
2. Read: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/plan.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`, the spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" TASK_ID="LAITDP2-integ"`

## Requirements
- Merge `LAITDP2-integ-core`, `LAITDP2-integ-linux`, `LAITDP2-integ-macos`, and `LAITDP2-integ-windows` into this worktree.
- Do not merge planning-doc changes from the orchestration branch into this worktree.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
- Verify `CP2-ci-checkpoint` is complete and recorded in `session_log.md`.

## End Checklist
1. Ensure the merged state is committed and local integration gates are green.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LAITDP2-integ"`.
3. Hand off merge notes and any residual risks to the operator.
4. Do not delete the worktree.
