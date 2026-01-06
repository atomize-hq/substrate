# Kickoff: PCP0-test (test) — Workspace Config Precedence Over Env

## Scope
- Tests only (plus minimal test-only helpers/fixtures/mocks if needed); no production code.
- Spec: `docs/project_management/_archived/policy_and_config_precedence/PCP0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/pcp0-precedence-test` on branch `pcp-pcp0-precedence-test` and that `.taskmeta.json` exists at the worktree root.
2. Read (end-to-end): `plan.md`, `tasks.json`, `session_log.md`, `PCP0-spec.md`, `decision_register.md`, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/_archived/policy_and_config_precedence" SLICE_ID="PCP0" LAUNCH_CODEX=1`

## Requirements
- Update `crates/shell/tests/config_show.rs` precedence assertions to match `PCP0-spec.md`.
- Preserve protected excludes assertions.
- Tests may be red on this branch until the code branch lands; they must compile and fail deterministically for spec-driven reasons.

## Required Commands
- `cargo fmt`
- Run the targeted tests you add/modify (even if expected to fail due to missing code), e.g.:
  - `cargo test -p substrate-shell --test config_show -- --nocapture`

## End Checklist
1. Run required commands; capture targeted test command(s) + outcomes.
2. Commit changes to the task branch.
3. From inside the worktree, run: `make triad-task-finish TASK_ID="PCP0-test"`.
4. Hand off the targeted test command(s) + outcomes to the operator (do not edit planning docs inside the worktree).
