# WS1-spec — Pending diff discovery (non-PTY) + dry-run reporting

## Scope

- Implement pending diff discovery for the current world session for **non-PTY** executions.
- Extend `workspace sync --dry-run` to print pending diff summary for non-PTY changes.

## Behavior (authoritative)

### Direction support (still limited)

- `direction=from_world` is supported.
- `direction=from_host` and `direction=both` MUST:
  - perform no mutations,
  - exit `4`,
  - print: “not implemented until WS5”.

### World backend requirement

Inside a workspace, `workspace sync --dry-run` requires the world backend when direction includes `from_world`:

- If the user forced host-only execution via `--no-world`:
  - Exit `2` with an actionable message: “workspace sync requires world; remove --no-world”.
- If the world backend cannot be reached/ensured:
  - Exit `3` with an actionable message pointing to `substrate world enable` and `substrate world doctor`.

### Pending diff discovery

When the world backend is available:

- `workspace sync --dry-run --direction from_world` MUST:
  - retrieve the current session’s pending non-PTY diff record (DR-0002; `filesystem-semantics-spec.md`),
  - apply exclude filtering rules from `filesystem-semantics-spec.md`,
  - print a deterministic summary:
    - total pending paths (writes/mods/deletes),
    - counts per bucket,
    - whether any paths were excluded by patterns.
  - When `--verbose` is set, output MUST include:
    - `session_started_at`, and
    - `diff_id`.
- If the backend does not support pending diff discovery:
  - Exit `4` with an explicit unsupported message.

No mutations:

- WS1 is discovery only. `workspace sync` without `--dry-run` remains unsupported until WS2 (exit `4`).

### Exit codes

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- 0: dry-run summary printed successfully
- 2: invalid flags or forced host-only execution (`--no-world`)
- 3: world backend required but unavailable
- 4: unsupported direction (from_host/both) or backend lacks pending diff discovery

## Acceptance criteria

- `workspace sync --dry-run --direction from_world` prints a pending diff summary on supported backends.
- `workspace sync --direction from_world` (no `--dry-run`) still exits `4` with “not implemented until WS2”.
- `workspace sync --dry-run --direction from_host|both` exits `4` with “not implemented until WS5”.
- With `--no-world`, `workspace sync --dry-run` exits `2` with the required actionable message.

## Out of scope

- Applying diffs to host (WS2).
- PTY diff discovery (WS4).
