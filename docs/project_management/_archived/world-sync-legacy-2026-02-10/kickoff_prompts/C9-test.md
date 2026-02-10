# Kickoff: C9-test (Init UX & migration)

## Scope
- Add tests for init UX and migration per `C9-spec`.
- Tests only; production code changes limited to minimal test helpers if required.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C9-spec.md, this prompt.
3. Set C9-test status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C9-test`).
4. Create branch `ws-c9-initux-test`; worktree `wt/ws-c9-initux-test`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Tests cover init interactive/dry-run behaviors, repair/migration flows, gating error messaging per spec.
- Run `cargo fmt` and targeted tests you add/touch; not responsible for full suite.

## End Checklist
1. Run fmt + targeted tests; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C9-test status), add END entry to session_log.md (commands/results/blockers), ensure C9-integ prompt exists.
5. Commit docs (`docs: finish C9-test`). Remove worktree if done.
