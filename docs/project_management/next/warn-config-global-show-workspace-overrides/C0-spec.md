# C0 Spec — Warn on `config global show` when workspace config overrides

## Summary

This slice adds a single stderr note to `substrate config global show` when run from within an
enabled workspace whose workspace config patch is non-empty (or cannot be parsed).

This is user-facing messaging + tests only; it does **not** change config merge behavior.

## Definitions

- **Global config patch**: the sparse YAML patch at the global scope (`$SUBSTRATE_HOME/config.yaml`).
- **Workspace config patch**: the sparse YAML patch at workspace scope (`<workspace_root>/.substrate/workspace.yaml`).
- **Enabled workspace**: `workspace::find_workspace_root(cwd)` returns `Some(<root>)` (workspace marker exists and
  the workspace-disabled marker is absent).
- **Workspace override active (for this feature)**:
  - Parsing `<workspace_root>/.substrate/workspace.yaml` succeeds and the parsed patch is non-empty, OR
  - Parsing fails (invalid YAML) — treated as active for warning purposes, but MUST NOT fail the command.

## User-visible behavior

### Baseline (unchanged)

- `substrate config global show` prints the **global** config patch to stdout:
  - YAML by default
  - JSON when `--json` is present
- Exit code remains unchanged for successful show operations.

### New stderr note (added)

The command MUST emit the new note **iff**:
1. The current working directory is inside an enabled workspace, AND
2. The workspace override is **active** (definition above).

The note MUST be:
- exactly one line,
- emitted to **stderr** only, and
- prefixed with `substrate: note:`.

#### Note text (exact template)

`substrate: note: workspace config <WORKSPACE_CONFIG_PATH> overrides global config here; run 'substrate config show --explain' to view the effective config for this directory`

Where:
- `<WORKSPACE_CONFIG_PATH>` is the resolved path to `<workspace_root>/.substrate/workspace.yaml`
  (display form is platform-native).

#### Interaction with existing notes

`substrate config global show` already emits an informational note when the **global** patch is empty.

Rule:
- If the workspace-override note is emitted, the command MUST NOT also emit the existing “global config patch is empty”
  note in the same invocation.
- If the workspace-override note is not emitted, the existing note behavior remains unchanged.

### Cases that do not emit the new note

- Outside a workspace.
- Inside an enabled workspace when the workspace config patch parses successfully and is empty (`{}`).

## Output invariants (must hold)

- Stdout MUST contain only the serialized global patch. Warning/note text MUST NOT appear on stdout.
- In `--json` mode, stdout MUST remain valid JSON.
- Exit code MUST remain unchanged for successful show operations.

## Acceptance criteria

1. Outside a workspace, the workspace-override note is absent from stderr.
2. Inside an enabled workspace with an empty workspace config patch, the workspace-override note is absent from stderr.
3. Inside an enabled workspace with a non-empty workspace config patch, the workspace-override note is present on stderr
   and contains:
   - `workspace.yaml`
   - `overrides global config here`
   - `substrate config show --explain`
4. In `--json` mode with a non-empty workspace patch, stdout parses as JSON and contains only the global patch.
5. If the workspace config file is present but invalid YAML, `substrate config global show`:
   - still exits successfully,
   - prints the global patch to stdout, and
   - emits the workspace-override note on stderr.
