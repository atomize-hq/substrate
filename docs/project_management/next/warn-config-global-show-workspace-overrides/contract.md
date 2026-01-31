# Contract — `config global show` workspace override note

This contract is authoritative for the user-visible behavior of:
- `substrate config global show`

It does not change:
- config precedence / merge semantics,
- patch serialization format, or
- supported config keys.

## CLI contract

### Command: `substrate config global show`

Purpose:
- Print the **global config patch** (`$SUBSTRATE_HOME/config.yaml`) for inspection.

Stdout:
- MUST contain only the serialized global config patch.
- Default format: YAML.
- With `--json`: stdout MUST be valid JSON.

Stderr:
- MAY contain informational notes.
- MUST NOT contain warnings that change the command exit code (see below).

Exit codes:
- `0`: command completed successfully.
- Non-zero: only for existing error cases (e.g., unreadable global patch, invalid global patch).

### Workspace override note (new)

When invoked from within an enabled workspace where the workspace override is **active**
(see `C0-spec.md` for the exact definition), the command MUST emit exactly one stderr line:

`substrate: note: workspace config <WORKSPACE_CONFIG_PATH> overrides global config here; run 'substrate config show --explain' to view the effective config for this directory`

Additional requirements:
- The note MUST be emitted on stderr only (stdout remains patch-only).
- The note MUST be a single line and MUST use the `substrate: note:` prefix.

### Robustness requirement (no new failure modes)

`substrate config global show` MUST NOT begin failing because of workspace config issues.

Specifically:
- If `<workspace_root>/.substrate/workspace.yaml` is unreadable or invalid YAML, the command:
  - MUST still print the global patch to stdout and exit `0` (assuming the global patch itself is readable/valid),
  - MUST treat the workspace override as active for the purpose of emitting the note.

### Existing note interaction

If the workspace override note is emitted, the command MUST NOT also emit the existing
“global config patch is empty (no overrides)” note in the same invocation.

## Config surface (paths and precedence)

Paths (by precedence; higher overrides lower for effective config):
1. Workspace patch: `<workspace_root>/.substrate/workspace.yaml`
2. Global patch: `$SUBSTRATE_HOME/config.yaml`

Reminder:
- `config global show` prints only the global patch, regardless of workspace.
- `substrate config show --explain` is the authoritative effective view for the current directory.
