# world-deps-apt-provisioning — contract surface

This file is the single place to consolidate the operator-facing contract introduced by ADR-0030 (CLI behavior, exit codes, platform/backends guarantees, and remediation invariants).

Decision inputs:
- `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md` (DR-0001/2/3)
- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md` (surface ownership)
- `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md` (User Contract)

External authoritative inputs (this feature does not redefine these surfaces):
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- World-deps inventory/enabled semantics and `install.method=apt` schema: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- Environment variable registry (incl. `SUBSTRATE_WORLD_REQUEST_PROFILE`): `docs/CONFIGURATION.md`
- Agent API request/response shapes (`/v1/execute`): `docs/WORLD.md`

## CLI

### Commands in scope

- Provisioning (new):
  - `substrate world enable --provision-deps [--dry-run] [--verbose]`
- Runtime world-deps (APT behavior changed):
  - `substrate world deps current sync [--dry-run] [--verbose] [--all]`
  - `substrate world deps current install <ITEM...> [--dry-run] [--verbose]`

### Definitions

- **APT-backed item**: a world-deps package whose install schema resolves `install.method=apt`.
- **In-scope set** for runtime application:
  - `deps current sync`: the effective enabled world-deps set for `cwd` (or all visible items when `--all` is set).
  - `deps current install <ITEM...>`: the explicit `<ITEM...>` arguments only (after bundle expansion); the effective enabled set is not added implicitly.
- **APT requirement set**: the normalized union of `install.apt[]` entries for the in-scope APT-backed items, with deterministic de-dup/ordering and version-pin conflict behavior per DR-0001.
- **APT requirement rendering** (when printed): each entry MUST be rendered as:
  - `name` when `version` is unset, or
  - `name=version` when `version` is set,
  in stable order, one per line.

## Config + env

- This feature introduces **no new config keys** and **no new environment variables**.
- `SUBSTRATE_WORLD_REQUEST_PROFILE` is not part of the operator workflow for this feature.
- Provisioning (`world enable --provision-deps`) MUST ignore `SUBSTRATE_WORLD_REQUEST_PROFILE` and MUST use the provisioning isolation model selected in DR-0003.

## Platform / backend guarantees

| Platform/backend | `substrate world enable --provision-deps` | Runtime `substrate world deps current sync|install` for APT-backed items |
| --- | --- | --- |
| Linux host-native world backend | Unsupported (exit `4`); Substrate MUST NOT mutate the host OS | Never runs APT/dpkg; fails early with exit `4` when APT requirements are unmet; remediation includes `substrate world enable --provision-deps` plus explicit “no host OS mutation” guidance |
| macOS Lima guest world backend | Supported | Never runs APT/dpkg; fails early with exit `4` when APT requirements are unmet; remediation includes `substrate world enable --provision-deps` |
| Windows | Unsupported (exit `4`) | Never runs APT/dpkg; fails early with exit `4` when APT requirements are unmet; remediation includes `substrate world enable --provision-deps` plus explicit “unsupported on Windows” guidance |

## Safety invariants (non-negotiable)

- Runtime `substrate world deps current sync|install` MUST NOT invoke APT/dpkg.
- The only supported Substrate surface that performs APT/dpkg OS mutation for world-deps is `substrate world enable --provision-deps` on supported guest backends.
- Linux host-native MUST NOT perform host OS mutation via APT/dpkg under any circumstances.
- Remediation for APT-backed items MUST include the exact command:

  ```text
  substrate world enable --provision-deps
  ```

## Provisioning contract — `substrate world enable --provision-deps`

### Operational scope + ordering (follow-up #3)

When `--provision-deps` is present, the `world enable` workflow MUST be a two-phase sequence:
1) World-backend enable (baseline `substrate world enable` provisioning steps).
2) APT provisioning for the effective enabled world-deps set (this feature).

The APT provisioning phase MUST run only after world-backend enable succeeds.

### Inputs

- The APT provisioning phase derives requirements from the effective enabled world-deps set for `cwd`.
- v1 accepts no explicit item list for APT provisioning.

### Success + no-op

- If the derived APT requirement set is empty, the APT provisioning phase MUST be a no-op and the command MUST exit `0`.
- If the derived APT requirement set is non-empty and all required APT packages are already present in the world (per DR-0002), the APT provisioning phase MUST be a no-op and the command MUST exit `0`.

### Unsupported backends

- On Linux host-native, the command MUST exit `4` and MUST include an explicit statement that Substrate will not mutate the host OS.
- On Windows, the command MUST exit `4`.

### `--dry-run`

When `--dry-run` is present:
- The command MUST print the derived APT requirement set deterministically (stable ordering per DR-0001), using the APT requirement rendering rules above.
- The command MUST perform no mutation (no world-backend provisioning and no APT/dpkg execution).
- If provisioning is unsupported on this platform/backend, the command MUST still exit `4`.

### `--verbose`

When `--verbose` is present, stdout MUST include:
- the derived APT requirement set (same content and ordering as `--dry-run`), and
- the selected provisioning request profile value (DR-0003).

### Exit codes

- Taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Overrides: none

Mapping for `substrate world enable --provision-deps`:
- `0`: success (including no-op)
- `2`: user/config error (including DR-0001 version-pin conflict)
- `3`: required dependency unavailable (world backend unavailable / cannot connect to world-agent when execution is required)
- `4`: not supported / missing prerequisites (includes: unsupported backend/platform for provisioning)
- `5`: safety / protected-path violation (reserved for fail-closed safety violations)
- `1`: unexpected internal error

## Runtime contract — `substrate world deps current sync|install` (APT-backed items)

### Invariant (ADR-0030)

`substrate world deps current sync|install` MUST NOT invoke APT/dpkg.

### APT in-scope rule (follow-up #4)

- `deps current sync`: APT-backed items are in scope when they are present in the in-scope set (effective enabled set, or all items under `--all`).
- `deps current install <ITEM...>`: APT-backed items are in scope only when they are present in the explicit `<ITEM...>` arguments (after bundle expansion).

### Fail-early rule (high level)

For the in-scope set:
1) Derive the normalized APT requirement set (DR-0001).
2) If the APT requirement set is non-empty, perform a read-only presence probe (DR-0002).
3) If any required APT package is missing:
   - Exit `4`.
   - Write remediation to stderr.
   - Remediation MUST include `substrate world enable --provision-deps`.
   - On Linux host-native, remediation MUST include explicit “no host OS mutation” guidance.
4) If all required APT packages are present, the command proceeds with non-APT installs (scripts/wrappers/manual behavior) per the upstream world-deps contract, without invoking APT/dpkg.

### `--dry-run` and `--verbose` under fail-early (follow-up #5)

- `--dry-run` MUST perform no mutation.
- `--dry-run` MUST still enforce the fail-early rule and MUST exit `4` when required APT packages are missing.
- When `--dry-run` is present and the derived APT requirement set is non-empty, stdout MUST include the derived APT requirement set using the APT requirement rendering rules above.
- When `--verbose` is present and the command exits `4` due to missing APT packages, stderr MUST include the derived APT requirement set (stable ordering per DR-0001).

### Exit codes

Runtime `deps current sync|install` exit codes remain taxonomy-aligned and MUST match the upstream world-deps contract, with the following stable meanings for this feature:
- `0`: success (including no-op)
- `2`: user/config error
- `3`: required dependency unavailable (world backend unavailable when probing/executing)
- `4`: not supported / missing prerequisites (includes: missing APT provisioning for in-scope items)
- `5`: safety / protected-path violation
- `1`: unexpected internal error
