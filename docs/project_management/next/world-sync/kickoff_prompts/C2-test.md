# Kickoff: C2-test (Manual world→host sync, non-PTY)

## Scope
- Add tests for non-PTY world→host sync engine per `C2-spec`.
- Tests only; production code changes limited to minimal test helpers if required.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C2-spec.md, this prompt.
3. Set C2-test status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C2-test`).
4. Create branch `ws-c2-sync-test`; worktree `wt/ws-c2-sync-test`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Tests cover conflict policies, filters/protected-path skips, size guard, and clear errors for unsupported directions per C2-spec.
- Target non-PTY path only.
- Run `cargo fmt` and the tests you add/touch (targeted `cargo test`); not responsible for full suite.

## End Checklist
1. Run fmt + targeted tests; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C2-test status), add END entry to session_log.md (commands/results/blockers), ensure C2-integ prompt exists.
5. Commit docs (`docs: finish C2-test`). Remove worktree if done.
