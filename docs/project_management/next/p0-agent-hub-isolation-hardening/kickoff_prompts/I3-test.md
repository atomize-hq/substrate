# Task I3-test (Full cage PTY parity) – TEST

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I3-spec.md`, and this prompt.
3. Set `I3-test` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I3-test`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i3-full-cage-pty-test
   git worktree add wt/ahih-i3-full-cage-pty-test ahih-i3-full-cage-pty-test
   cd wt/ahih-i3-full-cage-pty-test
   ```

## Spec (shared with I3-code)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I3-spec.md`

## Scope & Guardrails
- Tests only (plus minimal test-only helpers if absolutely needed).
- Add PTY-path coverage; skip cleanly when privileges/features are unavailable.

## Suggested Commands
```
cargo fmt
cargo test -p world-agent --tests -- --nocapture
```

## End Checklist
1. Confirm fmt + targeted tests are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I3-test`).
5. Remove worktree.

