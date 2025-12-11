# Kickoff Prompt – LP1-test (Linux world provision parity fix)

## Scope
- Tests/fixtures/harnesses only; no production code. Cover provisioner parity per LP1-spec.

## Start Checklist
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, LP1-spec.md, this prompt.
3. Set `LP1-test` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start LP1-test`).
5. Create branch `ps-lp1-provision-test` and worktree `wt/ps-lp1-provision-test`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Add tests/fixtures/harness coverage asserting `scripts/linux/world-provision.sh` writes `SocketGroup=substrate` and surfaces group/linger guidance; stub/dry-run approaches are acceptable with documented privileged skips.
- Capture expected socket/path/ownership fields in doctor/shim-status or helper artifacts where platform-agnostic.
- Required commands:  
  - `cargo fmt`  
  - Targeted `cargo test ...` / shellcheck / harness scripts you add/touch (record commands and skips in END entry).

## End Checklist
1. Run required commands above.
2. Commit worktree changes.
3. `git checkout feat/p0-platform-stability && git pull --ff-only && git merge --ff-only ps-lp1-provision-test`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish LP1-test`).
5. Remove worktree `wt/ps-lp1-provision-test`.
