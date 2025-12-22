# Kickoff: C6-test (.substrate-git integration)

## Scope
- Add tests for internal git init/commit/checkpoint and clean-tree guard per `C6-spec`.
- Tests only; production code changes limited to minimal test helpers if required.

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C6-spec.md, this prompt.
3. Set C6-test status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C6-test`).
4. Create branch `ws-c6-git-test`; worktree `wt/ws-c6-git-test`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Tests cover git init/ignore, commit creation after sync, checkpoint command, and clean-tree guard per spec.
- Run `cargo fmt` and targeted tests you add/touch; not responsible for full suite.

## End Checklist
1. Run fmt + targeted tests; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C6-test status), add END entry to session_log.md (commands/results/blockers), ensure C6-integ prompt exists.
5. Commit docs (`docs: finish C6-test`). Remove worktree if done.
