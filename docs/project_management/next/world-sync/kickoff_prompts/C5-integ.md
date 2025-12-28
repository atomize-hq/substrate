# Kickoff: C5-integ (Host→world pre-sync)

## Scope
- Merge C5-code and C5-test; align to `C5-spec`.

## Start Checklist
1. Confirm C5-code and C5-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C5-spec.md, this prompt.
4. Set C5-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C5-integ`).
5. Create branch `ws-c5-hostsync-integ`; worktree `wt/ws-c5-hostsync-integ`.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; resolve drift; ensure host→world behavior matches spec and remains safe.
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
5. Update tasks.json (C5-integ status), add END entry to session_log.md (commands/results/blockers).
6. Commit docs (`docs: finish C5-integ`). Remove worktree if done.
