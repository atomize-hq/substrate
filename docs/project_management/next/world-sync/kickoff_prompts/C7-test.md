# Kickoff: C7-test (Rollback CLI)

## Scope
- Add tests for rollback CLI per `C7-spec`.
- Tests only; production code changes limited to minimal test helpers if required.

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C7-spec.md, this prompt.
3. Set C7-test status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C7-test`).
4. Create branch `ws-c7-rollback-test`; worktree `wt/ws-c7-rollback-test`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Tests cover rollback last/checkpoint/session, protected-path guards, clean-tree guard, missing tag handling, and world overlay refresh expectations per spec.
- Run `cargo fmt` and targeted tests you add/touch; not responsible for full suite.

## End Checklist
1. Run fmt + targeted tests; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C7-test status), add END entry to session_log.md (commands/results/blockers), ensure C7-integ prompt exists.
5. Commit docs (`docs: finish C7-test`). Remove worktree if done.
