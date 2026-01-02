# Kickoff: PCP0-code (code) — Workspace Config Precedence Over Env

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/pcp0-precedence-code` on branch `pcp-pcp0-precedence-code` and that `.taskmeta.json` exists at the worktree root.
2. Read (end-to-end): `plan.md`, `tasks.json`, `session_log.md`, `PCP0-spec.md`, `decision_register.md`, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" SLICE_ID="PCP0" LAUNCH_CODEX=1`

## Requirements
- Implement the precedence change: when a workspace exists, workspace config overrides `SUBSTRATE_*` env vars.
- Preserve strict parsing, legacy `.substrate/settings.yaml` hard-error behavior, and protected sync excludes behavior.

## Required Commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

Baseline testing (required):
- Before changes: run a targeted baseline test set relevant to your change.
- After changes: re-run the same tests and ensure results are unchanged (or improved).
- Recommended baseline (adjust as needed for the touched area):
  - `cargo test -p substrate-shell --test config_show -- --nocapture`

Tests boundary:
- Do not add new tests or new test files.
- Only update existing tests if required to restore baseline expectations after the spec’s behavior change (still no new test cases).

## End Checklist
1. Run required commands; capture baseline test command(s) and outcomes (before + after).
2. Commit changes to the task branch.
3. From inside the worktree, run: `make triad-task-finish TASK_ID="PCP0-code"`.
4. Hand off the baseline test command(s) + outcomes to the operator (do not edit planning docs inside the worktree).
