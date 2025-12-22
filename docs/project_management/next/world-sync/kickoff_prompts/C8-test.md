# Kickoff: C8-test (World-side internal git bridge)

## Scope
- Add tests for world-side `.substrate-git` bootstrap/bridge per `C8-spec`.
- Tests only; production code changes limited to minimal test helpers if required.

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C8-spec.md, this prompt.
3. Set C8-test status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C8-test`).
4. Create branch `ws-c8-worldgit-test`; worktree `wt/ws-c8-worldgit-test`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Tests cover world repo creation/clone/mirror behavior, alignment with host commits/mapping, and error handling when unavailable; ensure user `.git` untouched.
- Run `cargo fmt` and targeted tests you add/touch; not responsible for full suite.

## End Checklist
1. Run fmt + targeted tests; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C8-test status), add END entry to session_log.md (commands/results/blockers), ensure C8-integ prompt exists.
5. Commit docs (`docs: finish C8-test`). Remove worktree if done.
