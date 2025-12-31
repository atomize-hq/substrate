# Kickoff: C8-integ (World-side internal git bridge)

## Scope
- Merge C8-code and C8-test; ensure world-side git behavior matches `C8-spec`.

## Start Checklist

Do not edit planning docs inside the worktree.

1. Confirm C8-code and C8-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C8-spec.md, this prompt.
4. Set C8-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C8-integ`).
5. Create branch `ws-c8-worldgit-integ`; worktree `wt/ws-c8-worldgit-integ`.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; resolve drift; ensure host/world internal git alignment per spec.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Run the feature-local smoke script for your platform and record results in `docs/project_management/next/world-sync/session_log.md`:
  - Linux: `bash docs/project_management/next/world-sync/smoke/linux-smoke.sh`
  - macOS: `bash docs/project_management/next/world-sync/smoke/macos-smoke.sh`
  - Windows: `pwsh -File docs/project_management/next/world-sync/smoke/windows-smoke.ps1`

## End Checklist
1. Ensure fmt/clippy/tests pass; run `make integ-checks`; capture outputs.
2. Run the feature-local smoke script for your platform; capture output in the END entry.
3. Commit integration worktree changes.
4. Merge back to feat/world-sync (ff-only).
5. Update tasks.json (C8-integ status), add END entry to session_log.md (commands/results/blockers).
6. Commit docs (`docs: finish C8-integ`). Remove worktree if done.
