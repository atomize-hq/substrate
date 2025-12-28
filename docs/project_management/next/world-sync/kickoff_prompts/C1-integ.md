# Kickoff: C1-integ (Sync config/CLI surface)

## Scope
- Merge C1-code and C1-test branches/worktrees per `C1-spec`.
- Ensure final state matches spec; resolve any code/test mismatches.

## Start Checklist
1. Confirm C1-code and C1-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C1-spec.md, this prompt.
4. Set C1-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C1-integ`).
5. Create branch `ws-c1-config-integ`; worktree `wt/ws-c1-config-integ` from that branch.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; align behavior with C1-spec (stub command only).
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests (at least those added in C1-test), then `make preflight`.
- You own resolving drift: adjust code/tests as needed to match spec.
- Run the feature-local smoke script for your platform and record results in `docs/project_management/next/world-sync/session_log.md`:
  - Linux: `bash docs/project_management/next/world-sync/smoke/linux-smoke.sh`
  - macOS: `bash docs/project_management/next/world-sync/smoke/macos-smoke.sh`
  - Windows: `pwsh -File docs/project_management/next/world-sync/smoke/windows-smoke.ps1`

## End Checklist
1. Ensure fmt/clippy/tests pass; ensure `make preflight` completes; capture outputs.
2. Run the feature-local smoke script for your platform; capture output in the END entry.
3. Commit integration worktree changes.
4. Merge back to feat/world-sync (ff-only).
5. Update tasks.json (C1-integ status), add END entry to session_log.md (commands/results/blockers).
6. Commit docs (`docs: finish C1-integ`). Remove worktree if done.
