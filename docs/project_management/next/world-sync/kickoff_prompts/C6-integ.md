# Kickoff: C6-integ (.substrate-git integration)

## Scope
- Merge C6-code and C6-test; ensure internal git behavior matches `C6-spec`.

## Start Checklist
1. Confirm C6-code and C6-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C6-spec.md, this prompt.
4. Set C6-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C6-integ`).
5. Create branch `ws-c6-git-integ`; worktree `wt/ws-c6-git-integ`.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; resolve drift; ensure git flows meet spec and remain safe.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight`.
- Run the feature-local smoke script for your platform and record results in `docs/project_management/next/world-sync/session_log.md`:
  - Linux: `bash docs/project_management/next/world-sync/smoke/linux-smoke.sh`
  - macOS: `bash docs/project_management/next/world-sync/smoke/macos-smoke.sh`
  - Windows: `pwsh -File docs/project_management/next/world-sync/smoke/windows-smoke.ps1`

## End Checklist
1. Ensure fmt/clippy/tests pass; run `make preflight`; capture outputs.
2. Run the feature-local smoke script for your platform; capture output in the END entry.
3. Commit integration worktree changes.
4. Merge back to feat/world-sync (ff-only).
5. Update tasks.json (C6-integ status), add END entry to session_log.md (commands/results/blockers).
6. Commit docs (`docs: finish C6-integ`). Remove worktree if done.
