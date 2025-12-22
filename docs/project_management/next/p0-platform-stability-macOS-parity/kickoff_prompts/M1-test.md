# Kickoff Prompt – M1-test (Lima socket parity)

## Scope
- Tests/fixtures/harnesses only; no production code changes. Cover replacement/provisioning detection, socket activation state, and doctor outputs per M1-spec (no backwards compatibility or user-data carry-over expected).

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M1-spec.md, this prompt.
3. Set `M1-test` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M1-test`).
5. Create branch `mp-m1-sockets-test` from `feat/p0-platform-stability-macOS-parity`; run `git worktree add wt/mp-m1-sockets-test mp-m1-sockets-test`.
6. Do **not** edit docs/tasks/session_log.md from the worktree.

## Requirements
- Add tests/fixtures that validate mac Lima replacement/provisioning logic and socket activation state (platform-agnostic portions), including doctor/log outputs where possible.
- Capture harness outputs for replacement guidance without needing a live mac VM in CI (e.g., fixture-based).
- Keep tests mac-targeted where necessary and avoid breaking Linux/WSL pipelines.
- Required commands:  
  - `cargo fmt`  
  - Targeted `cargo test ...` suites you add/touch (document in session_log).

## End Checklist
1. Run the required commands above and capture their outputs.
2. Inside `wt/mp-m1-sockets-test`, commit M1-test changes to branch `mp-m1-sockets-test` (no docs/tasks/session_log.md edits).
3. From outside the worktree, ensure branch `mp-m1-sockets-test` contains the worktree commit (fast-forward if needed); do **not** merge into `feat/p0-platform-stability-macOS-parity`.
4. Checkout `feat/p0-platform-stability-macOS-parity`; update tasks.json to completed; add an END entry to session_log.md with commands/results/blockers; create downstream prompts if missing; commit docs (`docs: finish M1-test`).
5. Remove worktree `wt/mp-m1-sockets-test`.
