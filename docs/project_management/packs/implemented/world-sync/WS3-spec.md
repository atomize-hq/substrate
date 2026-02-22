# WS3-spec — Auto-sync trigger (non-PTY)

## Scope
- Implement `sync.auto_sync` behavior for non-PTY world executions.

## Behavior (authoritative)

### Trigger rule
When `sync.auto_sync=true` and a non-PTY command executes in the world backend:
- After the command completes with exit `0`, Substrate MUST:
  - run `substrate workspace sync --direction <effective>` automatically.
- If the command exits non-zero, auto-sync MUST NOT run.

Direction gating:
- If the effective direction is `from_world` or `both`, auto-sync performs a from_world apply.
- If the effective direction is `from_host`, auto-sync performs no action (no-op) and exits with the command’s original exit code.

### Failure propagation
If auto-sync runs and fails:
- The overall `substrate` process exit code is the auto-sync failure’s exit code.
- The output MUST include a clear prefix line: `auto-sync failed:` plus the underlying error reason.

### Exit codes
- Inherited from `workspace sync` when auto-sync runs.

## Acceptance criteria
- With `sync.auto_sync=true`, a successful non-PTY world command that produces pending diffs triggers an automatic sync apply.
- Auto-sync does not run on non-zero command exit.
- Auto-sync failure propagates as the overall process exit code with the required message prefix.

## Out of scope
- PTY auto-sync (WS5).
