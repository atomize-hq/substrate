# Task I3-integ (Full cage PTY parity) – INTEGRATION

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I3-spec.md`, and this prompt.
3. Set `I3-integ` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I3-integ`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i3-full-cage-pty-integ
   git worktree add wt/ahih-i3-full-cage-pty-integ ahih-i3-full-cage-pty-integ
   cd wt/ahih-i3-full-cage-pty-integ
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Duties
- Merge `ahih-i3-full-cage-pty-code` and `ahih-i3-full-cage-pty-test`.
- Reconcile any drift so behavior matches `I3-spec.md`.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p world-agent --tests -- --nocapture
make integ-checks
```

## Smoke Scripts (required)
- Linux: `bash docs/project_management/next/p0-agent-hub-isolation-hardening/smoke/linux-smoke.sh`
- macOS: `bash docs/project_management/next/p0-agent-hub-isolation-hardening/smoke/macos-smoke.sh`
- Windows: `pwsh -File docs/project_management/next/p0-agent-hub-isolation-hardening/smoke/windows-smoke.ps1`

## End Checklist
1. Commit integration changes.
2. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
3. Run the feature-local smoke script for your platform; capture output for the END entry.
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I3-integ`).
5. Remove worktree.


Do not edit planning docs inside the worktree.
