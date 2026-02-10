# WS0-spec — Workspace sync CLI + gating + dry-run baseline

## Scope
- Introduce the workspace-scoped CLI surfaces (DR-0001):
  - `substrate workspace sync`
  - `substrate workspace checkpoint` (stub in WS0)
  - `substrate workspace rollback` (stub in WS0)
- Enforce workspace gating for these commands (exit `2` outside a workspace).
- Implement `workspace sync --dry-run` as a config-only preview (no world backend calls; no mutations).

## Behavior (authoritative)

### Workspace gating
For each command:
- `substrate workspace sync …`
- `substrate workspace checkpoint …`
- `substrate workspace rollback …`

If the current directory is not inside a workspace (no `.substrate/workspace.yaml` in any ancestor):
- Print a single actionable error message to stderr:
  - Must contain: `not in a workspace` and ``substrate workspace init``
- Exit `2`.
- Perform no mutations.

### `substrate workspace sync --dry-run`
When invoked inside a workspace:
- Resolve effective config and CLI overrides for:
  - `sync.auto_sync`
  - `sync.direction`
  - `sync.conflict_policy`
  - `sync.exclude` (including injected protected excludes)
- Print a deterministic, human-readable preview including:
  - Effective `direction`, `conflict_policy`, and the full effective exclude list.
  - A line stating that pending diff discovery is not implemented until WS1.
- Exit `0`.
- Perform no mutations (no world backend access).

CLI override behavior (preview only):
- `--direction` overrides the printed effective direction for this invocation.
- `--conflict-policy` overrides the printed effective conflict policy for this invocation.
- `--exclude` appends to the printed effective exclude list for this invocation.

### Stubs in WS0 (explicit unsupported)
Inside a workspace:
- `substrate workspace sync` (without `--dry-run`) MUST:
  - Exit `4`
  - Print an explicit “not implemented until WS2” message
  - Perform no mutations
- `substrate workspace checkpoint …` MUST:
  - Exit `4`
  - Print an explicit “not implemented until WS6” message
  - Perform no mutations
- `substrate workspace rollback …` MUST:
  - Exit `4`
  - Print an explicit “not implemented until WS7” message
  - Perform no mutations

### Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- 0: dry-run preview succeeded
- 2: workspace gating failure or invalid flag value
- 4: stubbed (explicit unsupported) commands in WS0

## Acceptance criteria
- Outside a workspace, `workspace sync|checkpoint|rollback` exit `2` with actionable guidance.
- Inside a workspace:
  - `workspace sync --dry-run` exits `0` and prints effective config values and exclude list.
  - `workspace sync` (no `--dry-run`) exits `4` with the required message.
  - `workspace checkpoint` exits `4` with the required message.
  - `workspace rollback` exits `4` with the required message.

## Out of scope
- Pending diff discovery (WS1).
- Sync apply semantics (WS2/WS5).
- Auto-sync (WS3).
- Internal checkpoint/rollback (WS6/WS7).
