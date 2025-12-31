# Kickoff: C4-test (PTY overlay diff + sync)

## Scope
- Add PTY-focused sync tests per `C4-spec`.
- Tests only; production code changes limited to minimal test helpers if required.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C4-spec.md, this prompt.
3. Set C4-test status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C4-test`).
4. Create branch `ws-c4-pty-test`; worktree `wt/ws-c4-pty-test`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Tests cover PTY diff availability, manual/auto sync application, protected-path skips, size guard, and overlay-unavailable skips per spec.
- Run `cargo fmt` and targeted PTY-related tests you add/touch; skip gracefully when overlay unavailable (with clear skip messaging).

## End Checklist
1. Run fmt + targeted tests; capture outputs (including skips).
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C4-test status), add END entry to session_log.md (commands/results/blockers), ensure C4-integ prompt exists.
5. Commit docs (`docs: finish C4-test`). Remove worktree if done.
