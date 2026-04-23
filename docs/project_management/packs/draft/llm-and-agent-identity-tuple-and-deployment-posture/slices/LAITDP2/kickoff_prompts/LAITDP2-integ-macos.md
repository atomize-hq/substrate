# Kickoff: LAITDP2-integ-macos (integration platform-fix — macos)

## Scope
- Resolve macOS follow-up work after `CP2-ci-checkpoint`.
- This task may modify production code or tests as needed to restore macOS parity for `LAITDP2`.
- Spec: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/LAITDP2-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a macOS machine.
2. Verify you are in `wt/llm-and-agent-identity-tuple-and-deployment-posture-laitdp2-integ-macos` on branch `llm-and-agent-identity-tuple-and-deployment-posture-laitdp2-integ-macos` and that `.taskmeta.json` exists.
3. Read: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/plan.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`, the spec, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" TASK_ID="LAITDP2-integ-macos"`

## Requirements
- Merge `LAITDP2-integ-core` into this worktree before making macOS fixes.
- Keep fixes narrow and limited to macOS parity issues surfaced by CP2.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and relevant tests.

## End Checklist
1. Ensure macOS parity is green and capture the run id or command evidence from CP2 follow-up work.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LAITDP2-integ-macos"`.
3. Hand off macOS notes and evidence to the operator.
4. Do not delete the worktree.
