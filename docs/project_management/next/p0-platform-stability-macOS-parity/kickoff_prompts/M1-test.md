# Kickoff Prompt – M1-test (Lima migration & socket parity)

## Scope
- Tests/fixtures/harnesses only; no production code changes. Cover migration detection/remediation, socket activation state, and doctor outputs per M1-spec.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M1-spec.md, this prompt.
3. Set `M1-test` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M1-test`).
5. Create branch `mp-m1-migration-test` and worktree `wt/mp-m1-migration-test`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Add tests/fixtures that validate mac Lima migration logic and socket activation state (platform-agnostic portions), including doctor/log outputs where possible.
- Capture harness outputs for migration guidance without needing a live mac VM in CI (e.g., fixture-based).
- Keep tests mac-targeted where necessary and avoid breaking Linux/WSL pipelines.
- Required commands:  
  - `cargo fmt`  
  - Targeted `cargo test ...` suites you add/touch (document in session_log).

## End Checklist
1. Run required commands above.
2. Commit worktree changes.
3. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only && git merge --ff-only mp-m1-migration-test`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish M1-test`).
5. Remove worktree `wt/mp-m1-migration-test`.
