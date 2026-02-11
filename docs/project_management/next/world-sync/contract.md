# world-sync — contract surface

This file is the single place to consolidate the user-facing contract for this feature (CLI/config/exit codes/paths).

## CLI
### Workspace sync

Command:
- `substrate workspace sync`

Flags:
- `--dry-run` (default: false)
  - When true: print what would be applied and exit without mutating the workspace.
- `--direction <from_world|from_host|both>` (default: from effective config `sync.direction`; see Config below)
- `--conflict-policy <prefer_host|prefer_world|abort>` (default: from effective config `sync.conflict_policy`)
- `--exclude <PATTERN>` (repeatable; default: none)
  - Semantics: appended to the effective exclude list for this invocation only.
- `--verbose` (default: false)
  - When true: print per-path decisions (apply/skip/conflict/protected/filtered) in addition to the summary.

Exit codes:
- `0`: success (including no-op: no pending diffs)
- `1`: unexpected internal error (e.g., filesystem operation failure mid-apply)
- `2`: not in a workspace, invalid flag value, or invalid exclude pattern
- `3`: world backend required but unavailable (direction requires world, or no world session exists)
- `4`: backend/platform does not support the required sync capability (explicit unsupported)
- `5`: safety-rail refusal (protected paths, size guard, or conflict-policy abort)

### Workspace checkpoint

Command:
- `substrate workspace checkpoint`

Flags:
- `--message <TEXT>` (optional; default is deterministic; see `internal-git-spec.md`)
- `--verbose` (default: false)

Exit codes:
- `0`: success (including no-op: nothing changed since last checkpoint)
- `2`: not in a workspace, invalid flag value
- `3`: required dependency unavailable (e.g., `git` not found)
- `5`: safety-rail refusal (e.g., user repo dirty when a guard applies)

### Workspace rollback

Command:
- `substrate workspace rollback <target>`

Targets:
- `last` (restore to the most recent checkpoint)
- `<CHECKPOINT_ID>` (opaque checkpoint identifier as printed by `substrate workspace checkpoint --verbose`)

Flags:
- `--force` (default: false)
  - When false: rollback refuses in the presence of safety-rail conditions (see `internal-git-spec.md`).
- `--verbose` (default: false)

Exit codes:
- `0`: success
- `2`: not in a workspace, invalid target, invalid flag value
- `3`: required dependency unavailable (e.g., `git` not found)
- `5`: safety-rail refusal (e.g., dirty workspace without `--force`)

## Config
### Files (authoritative paths)
- Workspace config patch: `<workspace_root>/.substrate/workspace.yaml`
- Global config patch: `$SUBSTRATE_HOME/config.yaml` (or default home if unset)

### Keys (authoritative)
All keys below live under the effective config model (`crates/shell/src/execution/config_model.rs`) and are set via:
- `substrate config workspace set <key>=<value>` (workspace-scoped)
- `substrate config global set <key>=<value>` (global-scoped)

World-sync keys:
- `sync.auto_sync: bool` (default: `false`)
- `sync.direction: from_world | from_host | both` (default: `from_world`)
- `sync.conflict_policy: prefer_host | prefer_world | abort` (default: `prefer_host`)
- `sync.exclude: list[string]` (default: `[]` plus injected protected excludes)

Protected excludes (always injected, always first, cannot be removed):
- `.git/**`
- `.substrate/**`

Precedence for effective config (high → low):
- CLI flags (only where defined by this feature; see CLI above)
- `SUBSTRATE_OVERRIDE_*` override inputs (operator-provided; see ADR-0008)
- Workspace patch (`.substrate/workspace.yaml`)
- Global patch (`$SUBSTRATE_HOME/config.yaml`)
- Built-in defaults

## Exit codes
- Taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- Overrides: none

## Platform guarantees
- Authoritative: `docs/project_management/next/world-sync/platform-parity-spec.md`
- Summary:
  - Linux:
    - `workspace sync`: supported for non-PTY and PTY pending diff discovery; apply is supported at checkpoint boundaries per specs.
    - `checkpoint` / `rollback`: supported.
  - macOS:
    - `workspace sync`: supported when the backend advertises required capabilities; otherwise exits `4` (explicit unsupported).
    - `checkpoint` / `rollback`: supported.
  - Windows:
    - `workspace sync`: explicit unsupported in this feature pack (exit `4`) until Windows/WSL backend exposes pending diff discovery/apply.
    - `checkpoint` / `rollback`: supported.
