# C0-spec — Scope clarity notes for `config global show` and implicit `config set`

## Behavior delta (single)
- Existing: `substrate config global show` prints the global config patch to stdout and does not emit a note about workspace overrides; implicit-scope `substrate config set <KEY>=<VALUE> ...` performs the write without a write-target note.
- New: When inside an enabled workspace with a non-empty (or invalid) workspace config patch, `substrate config global show` emits exactly one stderr note indicating the workspace config overrides the global config here, and implicit-scope `substrate config set <KEY>=<VALUE> ...` emits exactly one stderr note indicating the write target.
- Why: Make workspace override state and implicit write scope observable without changing stdout invariants or config merge behavior.

## Scope
- Add a single stderr note to `substrate config global show` when the workspace override is active for this directory.
- Add a single stderr note to implicit-scope `substrate config set <KEY>=<VALUE> ...` stating the write target (scope + path).
- Preserve stdout-only patch output and JSON validity rules.

## Behavior (authoritative)
### Definitions
- **Global config patch**: the sparse YAML patch at the global scope (`$SUBSTRATE_HOME/config.yaml`).
- **Workspace config patch**: the sparse YAML patch at workspace scope (`<workspace_root>/.substrate/workspace.yaml`).
- **Enabled workspace**: `workspace::find_workspace_root(cwd)` returns `Some(<root>)` (workspace marker exists and the workspace-disabled marker is absent).
- **Workspace override active (for this feature)**:
  - Parsing `<workspace_root>/.substrate/workspace.yaml` succeeds and the parsed patch is non-empty, OR
  - Parsing fails (invalid YAML); this is treated as active for warning purposes and MUST NOT fail the command.

### Global show note
#### Baseline (unchanged)
- `substrate config global show` prints the **global** config patch to stdout:
  - YAML by default
  - JSON when `--json` is present
- Exit code remains unchanged for successful show operations.

#### Workspace-override stderr note (added)
`substrate config global show` MUST emit the new note iff:
1. The current working directory is inside an enabled workspace, AND
2. The workspace override is active (definition above).

The note MUST be:
- exactly one line,
- emitted to stderr only, and
- prefixed with `substrate: note:`.

Note text (exact template):

`substrate: note: workspace config <WORKSPACE_CONFIG_PATH> overrides global config here; run 'substrate config show --explain' to view the effective config for this directory`

Where:
- `<WORKSPACE_CONFIG_PATH>` is the resolved path to `<workspace_root>/.substrate/workspace.yaml` (platform-native display).

#### Interaction with existing notes
`substrate config global show` already emits an informational note when the global patch is empty.

Rule:
- If the workspace-override note is emitted, the command MUST NOT also emit the existing “global config patch is empty” note in the same invocation.
- If the workspace-override note is not emitted, the existing note behavior remains unchanged.

#### Cases that do not emit the new note
- Outside a workspace.
- Inside an enabled workspace when the workspace config patch parses successfully and is empty (`{}`).

### Implicit `config set` write-target note
When invoked as `substrate config set <KEY>=<VALUE> ...` (without an explicit `global` / `workspace` subcommand), the command MUST emit exactly one stderr note stating the write target:

`substrate: note: write target is workspace config <WORKSPACE_CONFIG_PATH> (implicit scope); run 'substrate config workspace show' to view the workspace patch`

Where:
- `<WORKSPACE_CONFIG_PATH>` is the resolved path to `<workspace_root>/.substrate/workspace.yaml` (platform-native display).

### Output invariants
- Stdout MUST contain only the serialized global patch. Warning/note text MUST NOT appear on stdout.
- In `--json` mode for `substrate config global show`, stdout MUST remain valid JSON.
- Exit code MUST remain unchanged for successful show operations.

## Acceptance criteria
- AC-C0-01: Outside a workspace, the workspace-override note is absent from stderr.
- AC-C0-02: Inside an enabled workspace with an empty workspace config patch, the workspace-override note is absent from stderr.
- AC-C0-03: Inside an enabled workspace with a non-empty workspace config patch, the workspace-override note is present on stderr and contains:
  - `workspace.yaml`
  - `overrides global config here`
  - `substrate config show --explain`
- AC-C0-04: In `--json` mode with a non-empty workspace patch, stdout parses as JSON and contains only the global patch.
- AC-C0-05: If the workspace config file is present but invalid YAML, `substrate config global show` exits successfully, prints the global patch to stdout, and emits the workspace-override note on stderr.
- AC-C0-06: `substrate config set <KEY>=<VALUE> ...` emits exactly one stderr note stating the write target and contains:
  - `workspace.yaml`
  - `write target is workspace config`
  - `(implicit scope)`
- AC-C0-07: In `--json` mode for `substrate config set ...`, stdout parses as JSON and contains only the effective merged config.

## Out of scope
- No change to config merge behavior.
