# Kickoff: LAITDP2-integ-windows (integration platform-fix — windows)

## Scope
- Resolve Windows CI parity follow-up work after `CP2-ci-checkpoint`.
- This task may modify production code or tests as needed to restore Windows parity for `LAITDP2`.
- Spec: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/LAITDP2-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a Windows machine.
2. Verify you are in `wt/llm-and-agent-identity-tuple-and-deployment-posture-laitdp2-integ-windows` on branch `llm-and-agent-identity-tuple-and-deployment-posture-laitdp2-integ-windows` and that `.taskmeta.json` exists.
3. Read: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/plan.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`, the spec, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" TASK_ID="LAITDP2-integ-windows"`

## Requirements
- Merge `LAITDP2-integ-core` into this worktree before making Windows fixes.
- Keep fixes narrow and limited to Windows parity issues surfaced by CP2.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and relevant tests.
- This task is parity-only. Do not dispatch feature smoke from this task.

## End Checklist
1. Ensure Windows CI parity is green and capture the run id or command evidence from CP2 follow-up work.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LAITDP2-integ-windows"`.
3. Hand off Windows notes and evidence to the operator and ask for a checkpoint rerun if parity needs confirmation.
4. Do not delete the worktree.
