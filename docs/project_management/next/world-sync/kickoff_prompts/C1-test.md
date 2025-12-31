# Kickoff: C1-test (Sync config/CLI surface tests)

## Scope
- Add tests/fixtures for settings/CLI precedence and defaults per `C1-spec`.
- Tests only; production code changes limited to tiny test-only helpers if absolutely required.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C1-spec.md, this prompt.
3. Set C1-test status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C1-test`).
4. Create branch `ws-c1-config-test`; worktree `wt/ws-c1-config-test` from that branch.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Tests cover defaults, CLI/env/config precedence, and stub output (dry-run) per C1-spec.
- No production code changes beyond minimal test scaffolding.
- Run `cargo fmt` and relevant tests you add/touch (targeted `cargo test` invocations). You are responsible only for tests, not full suite.

## End Checklist
1. Run fmt + targeted tests; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C1-test status), add END entry to session_log.md (commands/results/blockers), ensure C1-integ prompt exists.
5. Commit docs (`docs: finish C1-test`). Remove worktree if done.
