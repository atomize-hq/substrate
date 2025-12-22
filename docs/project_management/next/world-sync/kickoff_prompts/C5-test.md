# Kickoff: C5-test (Host→world pre-sync)

## Scope
- Add tests for host→world pre-sync and directionality per `C5-spec`.
- Tests only; production code changes limited to minimal test helpers if required.

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C5-spec.md, this prompt.
3. Set C5-test status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C5-test`).
4. Create branch `ws-c5-hostsync-test`; worktree `wt/ws-c5-hostsync-test`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Tests cover host→world pre-sync application, conflict policies, filters/protected-path skips, size guard, and direction semantics (`from_host`, `both`) per spec.
- Run `cargo fmt` and targeted tests you add/touch; not responsible for full suite.

## End Checklist
1. Run fmt + targeted tests; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C5-test status), add END entry to session_log.md (commands/results/blockers), ensure C5-integ prompt exists.
5. Commit docs (`docs: finish C5-test`). Remove worktree if done.
