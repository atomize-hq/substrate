# Kickoff: C9-code (Init UX & migration)

## Scope
- Enhance init UX and migration per `C9-spec`. Production code only; no tests.

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C9-spec.md, this prompt.
3. Set C9-code status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C9-code`).
4. Create branch `ws-c9-initux-code`; worktree `wt/ws-c9-initux-code`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Add interactive/dry-run init options, repair flow, actionable gating errors per spec; protect user data.
- Migration handling for existing workspaces (repair missing pieces, warn on stale/partial state).
- No tests added/modified.
- Not required to run unit/integration suites; do run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Sanity-check UX manually if feasible.

## End Checklist
1. Run fmt/clippy; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C9-code status), add END entry to session_log.md (commands/results/blockers), ensure C9-test/C9-integ prompts exist.
5. Commit docs (`docs: finish C9-code`). Remove worktree if done.
