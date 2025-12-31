# Kickoff: C3-integ (Auto-sync non-PTY)

## Scope
- Merge C3-code and C3-test; align to `C3-spec`.

## Start Checklist

Do not edit planning docs inside the worktree.

1. Confirm C3-code and C3-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C3-spec.md, this prompt.
4. Set C3-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C3-integ`).
5. Create branch `ws-c3-autosync-integ`; worktree `wt/ws-c3-autosync-integ`.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; resolve drift; ensure behavior matches spec.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests (at least those added in C3-test), then `make integ-checks`.
- Run the feature-local smoke script for your platform and record results in `docs/project_management/next/world-sync/session_log.md`:
  - Linux: `bash docs/project_management/next/world-sync/smoke/linux-smoke.sh`
  - macOS: `bash docs/project_management/next/world-sync/smoke/macos-smoke.sh`
  - Windows: `pwsh -File docs/project_management/next/world-sync/smoke/windows-smoke.ps1`

## End Checklist
1. Ensure fmt/clippy/tests pass; run `make integ-checks`; capture outputs.
2. Run the feature-local smoke script for your platform; capture output in the END entry.
3. Commit integration worktree changes.
4. Merge back to feat/world-sync (ff-only).
5. Update tasks.json (C3-integ status), add END entry to session_log.md (commands/results/blockers).
6. Commit docs (`docs: finish C3-integ`). Remove worktree if done.
