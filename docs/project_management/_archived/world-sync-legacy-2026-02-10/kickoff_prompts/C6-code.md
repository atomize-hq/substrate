# Kickoff: C6-code (.substrate-git integration)

## Scope
- Add internal git init/commit/checkpoint hooks per `C6-spec`. Production code only; no tests.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C6-spec.md, this prompt.
3. Set C6-code status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C6-code`).
4. Create branch `ws-c6-git-code`; worktree `wt/ws-c6-git-code`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Initialize/manage `.substrate-git`, ignore user repo safely, commit after sync, checkpoint command, clean-tree guard per spec.
- No tests added/modified.
- Not required to run unit/integration suites; do run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Sanity-check git flows manually if feasible.

## End Checklist
1. Run fmt/clippy; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C6-code status), add END entry to session_log.md (commands/results/blockers), ensure C6-test/C6-integ prompts exist.
5. Commit docs (`docs: finish C6-code`). Remove worktree if done.
