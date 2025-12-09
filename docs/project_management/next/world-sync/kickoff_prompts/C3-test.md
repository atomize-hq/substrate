# Kickoff: C3-test (Auto-sync non-PTY)

## Scope
- Add tests for non-PTY auto-sync hooks and safety rails per `C3-spec`.
- Tests only; production code changes limited to minimal test helpers if required.

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C3-spec.md, this prompt.
3. Set C3-test status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C3-test`).
4. Create branch `ws-c3-autosync-test`; worktree `wt/ws-c3-autosync-test`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Tests cover auto-sync trigger conditions, filters/protected skips, size guard, dry-run, and overlay-unavailable skips per spec.
- Run `cargo fmt` and targeted tests you add/touch; no responsibility for full suite.

## End Checklist
1. Run fmt + targeted tests; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C3-test status), add END entry to session_log.md (commands/results/blockers), ensure C3-integ prompt exists.
5. Commit docs (`docs: finish C3-test`). Remove worktree if done.
