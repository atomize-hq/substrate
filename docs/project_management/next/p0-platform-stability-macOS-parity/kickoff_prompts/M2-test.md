# Kickoff Prompt – M2-test (Installer parity – macOS)

## Scope
- Tests/fixtures/harnesses only; no production code. Cover mac installer/uninstaller parity, including build-in-VM fallback, cleanup-state metadata, and logging guidance per M2-spec.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M2-spec.md, this prompt.
3. Set `M2-test` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M2-test`).
5. Create branch `mp-m2-installer-test` and worktree `wt/mp-m2-installer-test`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Add tests/harnesses for mac install/uninstall flows: missing-agent fallback, cleanup-state metadata, and log/diagnostic expectations (platform-agnostic portions).
- Capture outputs/fixtures so behavior can be validated without a mac VM where possible; note any mac-only requirements in the END log.
- Required commands:  
  - `cargo fmt`  
  - Targeted `cargo test ...` suites you add/touch (document in session_log).

## End Checklist
1. Run required commands above.
2. Commit worktree changes.
3. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only && git merge --ff-only mp-m2-installer-test`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish M2-test`).
5. Remove worktree `wt/mp-m2-installer-test`.
