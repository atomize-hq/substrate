# Kickoff: C0-test (Init & Gating)

## Scope
- Add tests for `substrate init` and world gating per `C0-spec`.
- Tests only; production code changes limited to minimal test helpers if required.

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C0-spec.md, this prompt.
3. Set C0-test status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C0-test`).
4. Create branch `ws-c0-init-test`; worktree `wt/ws-c0-init-test`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Tests cover init creation/idempotence, gating behavior (world denied until init), host-only allowed without init, and ignore rules for `.substrate-git`.
- Run `cargo fmt` and targeted tests you add/touch; not responsible for full suite.

## End Checklist
1. Run fmt + targeted tests; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C0-test status), add END entry to session_log.md (commands/results/blockers), ensure C0-integ prompt exists.
5. Commit docs (`docs: finish C0-test`). Remove worktree if done.
