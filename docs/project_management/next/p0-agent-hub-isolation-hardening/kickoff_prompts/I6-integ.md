# Task I6-integ (`substrate world verify`) – INTEGRATION

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I6-spec.md`, and this prompt.
3. Set `I6-integ` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I6-integ`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i6-world-verify-integ
   git worktree add wt/ahih-i6-world-verify-integ ahih-i6-world-verify-integ
   cd wt/ahih-i6-world-verify-integ
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Duties
- Merge `ahih-i6-world-verify-code` and `ahih-i6-world-verify-test`.
- Reconcile any drift so behavior matches `I6-spec.md`.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets -- --nocapture
make preflight
```

## Smoke Scripts (required)
- Linux: `bash docs/project_management/next/p0-agent-hub-isolation-hardening/smoke/linux-smoke.sh`
- macOS: `bash docs/project_management/next/p0-agent-hub-isolation-hardening/smoke/macos-smoke.sh`
- Windows: `pwsh -File docs/project_management/next/p0-agent-hub-isolation-hardening/smoke/windows-smoke.ps1`

## End Checklist
1. Commit integration changes.
2. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
3. Run the feature-local smoke script for your platform; capture output for the END entry.
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I6-integ`).
5. Remove worktree.
