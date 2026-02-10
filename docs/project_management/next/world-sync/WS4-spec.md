# WS4-spec — Pending diff discovery (PTY) + dry-run reporting

## Scope
- Extend pending diff discovery to include PTY-originated changes for the current world session.
- Extend `workspace sync --dry-run` to report PTY pending diffs distinctly from non-PTY.

## Behavior (authoritative)

### PTY pending diff discovery
When a PTY session has executed commands that mutate the workspace in the world overlay:
- `workspace sync --dry-run --direction from_world` MUST report:
  - pending non-PTY diff summary (if any), and
  - pending PTY diff summary (if any),
  - plus a combined total.

### Unsupported behavior
If the backend cannot provide PTY pending diffs:
- `workspace sync --dry-run` MUST:
  - still report non-PTY pending diffs (if supported),
  - and print a clear “PTY pending diffs unsupported” line.
  - Exit `0` (because this is a reporting command), unless the backend is required and unavailable (exit `3`).

### Exit codes
- 0: dry-run summary printed
- 3: world backend required but unavailable
- 4: unsupported direction values (from_host/both) until WS5

## Acceptance criteria
- Dry-run output includes PTY pending diff counts when supported.
- Unsupported PTY diff discovery is explicit in output and does not silently omit PTY semantics.

## Out of scope
- Applying PTY diffs to host (WS5).
