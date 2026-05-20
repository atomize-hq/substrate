# Kickoff: AHCSITC2-integ-linux (integration platform parity — linux)

## Scope
- Resolve Linux parity failures for the AHCSITC2 checkpoint-boundary slice.
- This pack currently sets `behavior_platforms_required=[]`, so this is a parity-only task. Do not invent feature-smoke requirements here.
- Spec: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC2/AHCSITC2-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run on a Linux host if possible.
2. Verify you are in `wt/agent-hub-core-successor-identity-tuple-compatible-ahcsitc2-integ-linux` on branch `agent-hub-core-successor-identity-tuple-compatible-ahcsitc2-integ-linux` and `.taskmeta.json` exists.
3. Read `plan.md`, `tasks.json`, `session_log.md`, `AHCSITC2-spec.md`, and this prompt.
4. Merge the `AHCSITC2-integ-core` branch into this worktree before making fixes.

## Requirements
- Keep fixes narrowly scoped to Linux parity issues revealed by `CP1-ci-checkpoint`.
- Run Linux-local parity gates: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and relevant tests.
- Record the compile-parity run id/URL that justified this task when one exists.

## End Checklist
1. Confirm the Linux parity issue is resolved or document the remaining blocker precisely.
2. From inside the worktree, run `make triad-task-finish TASK_ID="AHCSITC2-integ-linux"`.
3. Hand off the Linux-specific validation outputs to the operator.
