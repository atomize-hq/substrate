# Kickoff: F0-exec-preflight (execution preflight gate)

## Scope
- Run the feature-level execution preflight before any slice triad starts.
- This is a docs-only orchestration task.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Read `plan.md`, `tasks.json`, `session_log.md`, `execution_preflight_report.md`, and this prompt.
2. Confirm the pack still intends schema v4 automation and cross-platform execution.
3. Confirm the checkpoint plan still closes at `ITPS3`.

## Requirements
- Write a concrete `ACCEPT` or `REVISE` recommendation into `execution_preflight_report.md`.
- Verify the execution-gate surfaces and kickoff prompts are all present.
- Verify the checkpoint, platform-fix, and cleanup tasks still line up with the slice order.

## End Checklist
1. Mark `F0-exec-preflight` completed in `tasks.json`.
2. Record the recommendation and any required fixes in `session_log.md`.
3. Keep all work on the orchestration checkout.
