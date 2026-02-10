# Kickoff: C1-code (Sync config/CLI surface)

## Scope
- Implement settings/CLI parsing and `substrate sync` stub per `C1-spec`.
- Production code only. No tests. No behavior changes to sync.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C1-spec.md, this prompt.
3. Set C1-code status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C1-code`).
4. Create branch `ws-c1-config-code`; worktree `wt/ws-c1-config-code` from that branch.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Add config/env/CLI surfaces and stub `substrate sync` output matching C1-spec defaults and enums.
- Protected paths listed in help/output.
- No tests added or modified.
- Not required to run unit/integration tests. Do run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Manually sanity-check stub output.

## End Checklist
1. Run fmt/clippy (as above); note results for log.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C1-code status), add END entry to session_log.md (commands/results/blockers), create/confirm C1-test and C1-integ kickoff prompts if missing.
5. Commit docs (`docs: finish C1-code`). Remove worktree if done.
