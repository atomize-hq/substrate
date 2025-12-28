# PCM3 — Env Scripts + World Enable Home + Legacy Removals (ADR-0003)

## Scope (authoritative)
Implement ADR-0003 global env script behavior, world enable home semantics, and removals.

### `$SUBSTRATE_HOME/env.sh` (stable exports)
Rules:
- Written only by:
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`
  - `substrate config global init|set`
  - `substrate world enable`
- Runtime `substrate` execution does not rewrite this file.

Format:
- Bash script with shebang `#!/usr/bin/env bash`.
- Safe to source repeatedly (idempotent exports only).

Exports (exact set for this ADR):
- `SUBSTRATE_HOME`
- `SUBSTRATE_WORLD=enabled|disabled` (derived from `world.enabled`)
- `SUBSTRATE_CAGED=1|0` (derived from `world.caged`)
- `SUBSTRATE_ANCHOR_MODE=workspace|follow-cwd|custom` (derived from `world.anchor_mode`)
- `SUBSTRATE_ANCHOR_PATH` (derived from `world.anchor_path`)
- `SUBSTRATE_POLICY_MODE=disabled|observe|enforce` (derived from `policy.mode`)

### `$SUBSTRATE_HOME/manager_env.sh` (runtime manager wiring)
Rules:
- Runtime `substrate` execution regenerates this file on startup when shims are enabled.
- The file sources `$SUBSTRATE_HOME/env.sh` if it exists and continues without failure if it does not exist.
- `substrate world deps *` ensures `$SUBSTRATE_HOME/manager_env.sh` exists before invoking guest-side manager tooling.

Format:
- Bash script with shebang `#!/usr/bin/env bash`.

Required behavior:
1. Source `$SUBSTRATE_HOME/env.sh` if present.
2. Source `$SUBSTRATE_HOME/manager_init.sh` if present.
3. Source user original `BASH_ENV` if captured as `SUBSTRATE_ORIGINAL_BASH_ENV` and the file exists.
4. Source the legacy bashenv file at `~/.substrate/bashenv` if it exists.

### `substrate world enable` home semantics
Command:
- `substrate world enable --home <PATH> [other existing flags not removed by this ADR]`

Rules:
- `--home` sets `$SUBSTRATE_HOME` for the operation and all state writes live under that home.
- `--prefix` does not exist and is rejected.
- `SUBSTRATE_PREFIX` has no effect.

### Removed names and legacy artifacts (hard removals)
Removal set (must not exist in loader/CLI logic):
- Workspace legacy:
  - `.substrate/settings.yaml`
  - `.substrate-profile`
  - `.substrate-profile.d/*`
- Policy legacy:
  - `.substrate-policy.yaml`
  - `world_fs.cage`
  - `SUBSTRATE_WORLD_FS_CAGE`
- Anchor/root legacy:
  - `world.root_mode`, `world.root_path`
  - `--world-root-mode`, `--world-root-path`
  - `SUBSTRATE_WORLD_ROOT_MODE`, `SUBSTRATE_WORLD_ROOT_PATH`
- World enable legacy:
  - `--prefix`
  - `SUBSTRATE_PREFIX`
  - `SUBSTRATE_MANAGER_ENV`

Canonical naming:
- Anchor: `world.anchor_mode`, `world.anchor_path`, `--anchor-mode`, `--anchor-path`, `SUBSTRATE_ANCHOR_MODE`, `SUBSTRATE_ANCHOR_PATH`
- Filesystem isolation: `world_fs.isolation` and `SUBSTRATE_WORLD_FS_ISOLATION`
- Roaming guard: `world.caged`, `--caged/--uncaged`, and `SUBSTRATE_CAGED`

## Non-scope (explicit)
- Workspace and config CLI (`PCM0`), except for `$SUBSTRATE_HOME/env.sh` ownership that is explicitly in-scope here.
- Policy CLI (`PCM1`).
- Policy evaluation and routing semantics (`PCM2`).

## Acceptance (implementation outcomes)
- `$SUBSTRATE_HOME/env.sh` and `$SUBSTRATE_HOME/manager_env.sh` behavior and ownership match ADR-0003 exactly.
- `substrate world enable --home` semantics match ADR-0003 exactly and `--prefix` is rejected.
- All removed names and legacy artifacts are absent from loader/CLI logic and produce explicit, actionable errors where required by ADR-0003.

