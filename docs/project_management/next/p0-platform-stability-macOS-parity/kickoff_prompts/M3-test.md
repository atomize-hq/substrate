# Kickoff Prompt – M3-test (Backend & doctor parity)

## Scope
- Tests/fixtures only; no production code. Cover mac fs_mode propagation, forwarding/readiness ordering, and doctor JSON/text outputs per M3-spec.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M3-spec.md, this prompt.
3. Set `M3-test` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M3-test`).
5. Create branch `mp-m3-backend-test` and worktree `wt/mp-m3-backend-test`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Add tests/fixtures that validate fs_mode propagation on mac (unit/fixture level), forwarding-before-probe behavior, and doctor/shim-status/health JSON/text outputs (platform-agnostic portions).
- Prefer platform-agnostic assertions; document any mac-only coverage or skips in the END log.
- Required commands:  
  - `cargo fmt`  
  - Targeted `cargo test ...` suites added/touched (record in session_log).

## End Checklist
1. Run required commands above.
2. Commit worktree changes.
3. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only && git merge --ff-only mp-m3-backend-test`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish M3-test`).
5. Remove worktree `wt/mp-m3-backend-test`.
