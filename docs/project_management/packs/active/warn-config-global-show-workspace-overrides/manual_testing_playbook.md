# Manual Testing Playbook — Scope clarity notes (`config global show`, implicit `config set`)

This playbook is authoritative for the manual validation described in:
- `docs/project_management/adrs/draft/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md`
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/slices/C0/C0-spec.md`

## Preconditions

- A working `substrate` binary on PATH (or set `SUBSTRATE_BIN`).
- Ability to create temporary directories/files.

Environment variables used in the cases below:
- `SUBSTRATE_HOME` (global patch lives at `$SUBSTRATE_HOME/config.yaml`)

Smoke scripts (mirror these cases):
- `smoke/linux-smoke.sh`
- `smoke/macos-smoke.sh`
- `smoke/windows-smoke.ps1`

## Cases

### Case 1 — Workspace patch is non-empty ⇒ note is emitted

Setup:
1. Create a temporary workspace directory `WS`.
2. Initialize workspace metadata:
   - `substrate workspace init WS`
3. Write a global patch:
   - `$SUBSTRATE_HOME/config.yaml`:
     ```yaml
     world:
       caged: false
     ```
4. Write a non-empty workspace patch:
   - `WS/.substrate/workspace.yaml`:
     ```yaml
     world:
       caged: true
     ```

Run (from inside `WS`):
- `substrate config global show`

Expected:
- stderr contains the single-line note:
  - contains `substrate: note: workspace config`
  - contains `workspace.yaml`
  - contains `overrides global config here`
  - contains `substrate config show --explain`
- stdout contains only the serialized global patch (no `substrate: note:` text).

Follow-up:
- `substrate config show --explain` shows `world.caged` sourced from the workspace patch.

### Case 2 — Workspace patch is empty ⇒ no note

Setup:
- In the same workspace `WS`, overwrite the workspace patch to an empty patch:
  - `WS/.substrate/workspace.yaml`:
    ```yaml
    {}
    ```

Run (from inside `WS`):
- `substrate config global show`

Expected:
- stderr does NOT contain the workspace-override note.
- stdout remains the global patch.

### Case 3 — Outside any workspace ⇒ no note

Run (from a directory that is not inside `WS`):
- `substrate config global show`

Expected:
- stderr does NOT contain the workspace-override note.
- stdout remains the global patch.

### Case 4 — Workspace patch is invalid YAML ⇒ command still succeeds + note is emitted

Setup:
- In workspace `WS`, write an invalid YAML file:
  - `WS/.substrate/workspace.yaml`:
    ```yaml
    world: [this is not valid
    ```

Run (from inside `WS`):
- `substrate config global show`

Expected:
- Command exits successfully (assuming the global patch is readable/valid).
- stderr contains the workspace-override note (the command MUST NOT fail because the workspace patch is invalid).
- stdout remains the global patch.

### Case 5 — JSON mode stdout remains valid JSON

Setup:
- Use the workspace from Case 1 (workspace patch non-empty).

Run (from inside `WS`):
- `substrate config global show --json`

Expected:
- stdout is valid JSON.
- stderr contains the workspace-override note.
- stdout does not contain `substrate: note:`.

### Case 6 — Implicit `config set` emits write-target note (stderr-only)

Setup:
- Use the workspace `WS` from Case 1.

Run (from inside `WS`):
- `substrate config set sync.auto_sync=true`

Expected:
- stderr contains a single-line note that:
  - contains `substrate: note: write target is workspace config`
  - contains `workspace.yaml`
  - contains `(implicit scope)`
- stdout contains only the serialized effective merged config (no `substrate: note:` text).

JSON mode:
- `substrate config set --json sync.auto_sync=true`

Expected (JSON mode):
- stdout is valid JSON and does not contain `substrate: note:`.
