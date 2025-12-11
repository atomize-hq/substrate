# Kickoff Prompt – LP1-test (Linux world provision parity fix)

## Scope
- Tests/fixtures/harnesses only; no production code. Cover provisioner parity per LP1-spec (SocketGroup=substrate, group/linger guidance, corrected socket metadata). Use stubs/dry-runs where privileged ops would be required; document skips.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, LP1-spec.md, this prompt.
3. Set `LP1-test` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start LP1-test`).
5. Create branch `ps-lp1-provision-test` from `feat/p0-platform-stability-macOS-parity`; run `git worktree add wt/ps-lp1-provision-test ps-lp1-provision-test`.
6. Do **not** edit docs/tasks/session_log.md from the worktree.

## Requirements
- Add tests/fixtures/harness coverage asserting `scripts/linux/world-provision.sh` writes `SocketGroup=substrate` and surfaces group/linger guidance; stub/dry-run approaches are acceptable with documented privileged skips.
- Capture expected socket/path/ownership fields in doctor/shim-status or helper artifacts where platform-agnostic.
- Required commands:  
  - `cargo fmt`  
  - Targeted `cargo test ...` / shellcheck / harness scripts you add/touch (record commands and skips in END entry).

## End Checklist
1. Run the required commands above and capture their outputs.
2. Inside `wt/ps-lp1-provision-test`, commit LP1-test changes to branch `ps-lp1-provision-test` (no docs/tasks/session_log.md edits).
3. From outside the worktree, ensure branch `ps-lp1-provision-test` contains the worktree commit (fast-forward if needed); do **not** merge into `feat/p0-platform-stability-macOS-parity`.
4. Checkout `feat/p0-platform-stability-macOS-parity`; update tasks.json to completed; add an END entry to session_log.md with commands/results/blockers; create downstream prompts if missing; commit docs (`docs: finish LP1-test`).
5. Remove worktree `wt/ps-lp1-provision-test`.
