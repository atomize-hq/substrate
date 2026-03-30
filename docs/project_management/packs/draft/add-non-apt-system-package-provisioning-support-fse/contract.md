# add-non-apt-system-package-provisioning-support-fse — contract surface

This file is the pack-root contract for the manager-aware world-deps surface. It defines `C-01`, the authoritative operator contract that downstream seams consume through `THR-01`, and records `C-02`, the authoritative in-world probe/support-gate contract consumed through `THR-02`.

Reference inputs for this pack-root contract:
- `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
- `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- `docs/project_management/packs/implemented/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support-fse/decision_register.md`

## Authority handoff

- `C-01` is authoritative for the shared manager-aware operator contract for `substrate world enable --provision-deps` and for runtime `substrate world deps current sync|install` when system-package items are in scope.
- `C-02` is authoritative for deterministic in-world world-manager probe and support-gate outcomes, including supported `apt` / `pacman` selection, fail-closed unsupported and contradiction handling, and the ban on host-derived routing inputs.
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` remains authoritative for inventory layering, enabled-resolution semantics, bundle expansion, wrapper semantics, schema `version: 1` baseline, and non-system-package install behavior.
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` remains authoritative for the base meaning of exit codes.
- `docs/project_management/packs/implemented/world-deps-apt-provisioning/contract.md` remains authoritative for APT provisioning baseline semantics, provided it does not contradict `C-01`.
- `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md` and `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md` are orientation and rationale only; if they conflict with `C-01`, `C-01` wins.
- The shared-contract reconciliation targets remain owned by `SEAM-6`:
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - `docs/reference/world/deps/README.md`
  - any other shared world-deps doc that still implies runtime mutation or APT-only truth
- `REM-001` stays owned by `SEAM-6` and is not a blocker for this seam.

## `C-01` contract summary

`C-01` is the accepted manager-aware operator contract for this pack. It binds downstream seams to one truth for:

- `substrate world enable --provision-deps` as the only operator-facing system-package provisioning entrypoint
- runtime `substrate world deps current sync|install` as read-only with respect to system-package managers
- in-world manager selection only, with no host PATH or host package-manager routing
- fail-closed handling for unsupported backends, manager mismatches, and mixed-manager enabled sets
- v1 pacman support as provisioning-only and non-runnable
- no new config, env, protocol, trace, or log surface

The detailed command behavior, exit-code posture, and runtime remediation wording remain within `C-01` and the downstream execution seams. This file only fixes the authority handoff and decision basis that those seams consume.

## `C-02` contract summary

`C-02` is the accepted state contract for the in-world world-manager probe and support gate. It binds downstream seams to one truth for:

- in-world routing inputs only: `/etc/os-release` (`ID`, `ID_LIKE`) plus `command -v pacman`
- deterministic family mapping for Debian-family and Arch-family identifiers
- fail-closed handling for unreadable, unmapped, ambiguous, and contradictory probe results
- `apt`, `pacman`, or unsupported outcomes only
- exit `4` for unsupported/contradiction cases and exit `3` when the world backend is unavailable and the probe cannot run in-world
- dry-run performing the same probe and classification without mutation

The detailed mapping rules, contradiction reasons, and supported-platform posture remain defined by `C-02` and the pack-level review surfaces that consume it.

## CLI

### Commands in scope

- Provisioning:
  - `substrate world enable --provision-deps [--dry-run] [--verbose]`
- Runtime world-deps:
  - `substrate world deps current sync [--dry-run] [--verbose] [--all]`
  - `substrate world deps current install <ITEM...> [--dry-run] [--verbose]`

### Definitions

- **System-package item**: a world-deps package whose `install.method` is `apt` or `pacman`.
- **APT-backed item**: a system-package item whose `install.method=apt`.
- **Pacman-backed item**: a system-package item whose `install.method=pacman`.
- **In-scope set** for runtime application:
  - `deps current sync`: the effective enabled world-deps set for `cwd`, or every visible item when `--all` is set.
  - `deps current install <ITEM...>`: the explicit `<ITEM...>` arguments only, after bundle expansion. The effective enabled set is not added implicitly.
- **Normalized APT requirement set**: the normalized union of `install.apt[]` entries for the in-scope APT-backed items, using the de-duplication, ordering, and version-conflict rules inherited from `docs/project_management/packs/implemented/world-deps-apt-provisioning/contract.md`.
- **Normalized pacman requirement set**: the normalized union of `install.pacman[]` entries for the in-scope pacman-backed items. De-duplicate by exact package name and sort in ascending byte order.
- **Detected world manager**: the provisioning-time manager selected by the in-world probe described in DR-0002. The only supported values are `apt` and `pacman`.

### Operator-visible rendering

- APT requirement entries MUST render as `name` when `version` is unset or `name=version` when `version` is set.
- Pacman requirement entries MUST render as `name`.
- Dry-run and verbose output MUST render each normalized requirement entry on its own line in stable order.

## Config + env

- This feature introduces no new config keys.
- This feature introduces no new environment variables.
- `SUBSTRATE_WORLD_REQUEST_PROFILE` is not an operator control surface for this feature.
- Provisioning may require stricter internal guard rails, but that is an implementation detail and not an operator-facing profile knob.
- Runtime `substrate world deps current sync|install` MUST NOT use a provisioning request profile.
- This feature does not add a new structured log field, trace field, protocol field, or agent API request field.

## Platform / backend guarantees

| Platform/backend | `substrate world enable --provision-deps` | Runtime `substrate world deps current sync|install` for system-package items |
| --- | --- | --- |
| Linux host-native world backend | Unsupported. Exit `4`. Substrate MUST NOT mutate the host OS. | Never invokes `apt`, `dpkg`, or `pacman`. Fails early with exit `4` when a required system package is missing. Remediation MUST state that Substrate will not mutate the host OS. |
| macOS Lima guest world backend | Supported when the in-world probe selects `apt` or `pacman` and the enabled set uses exactly that one manager. | Never invokes `apt`, `dpkg`, or `pacman`. Fails early with exit `4` when a required system package is missing. Remediation MUST include `substrate world enable --provision-deps`. |
| Windows WSL world backend | Unsupported. Exit `4`. `scripts/substrate/world-enable.sh` already defines `substrate world enable` as unsupported on Windows, and this pack does not widen that surface. | Never invokes `apt`, `dpkg`, or `pacman`. Fails early with exit `4` when a required system package is missing. Remediation MUST state that `substrate world enable --provision-deps` is unsupported on Windows and that Substrate will not mutate the Windows host OS. |

## Safety invariants

- Runtime `substrate world deps current sync|install` MUST NOT invoke `apt`, `dpkg`, or `pacman`.
- The only Substrate surface that performs world OS package mutation for world-deps is `substrate world enable --provision-deps` on supported guest backends.
- Provisioning manager selection MUST be derived in-world. Host PATH, host installer detection, and host package-manager state are not routing inputs.
- Linux host-native provisioning MUST NOT mutate the host OS.
- Provisioning is fail-closed. Unsupported probe results, unsupported platforms, manager mismatch, and mixed-manager enabled sets exit before any OS package-manager command is executed.
- `install.method=pacman` is a provisioning-only system-package method in v1.
- `install.method=pacman` packages are non-runnable in v1. They MUST NOT rely on runnable-wrapper generation, and they MUST NOT define new pacman-specific present semantics in this pack.
- This pack does not require new built-in pacman inventory entries. V1 pacman support extends the contract for authored inventory items without expanding the built-in package catalog.

## Provisioning contract — `substrate world enable --provision-deps`

### Operational sequence

When `--provision-deps` is present, `substrate world enable` is a two-phase workflow:
1. World-backend enable.
2. System-package provisioning for the effective enabled world-deps set.

Phase 2 runs only after phase 1 succeeds.

### Inputs

- Provisioning derives requirements from the effective enabled world-deps set for `cwd`.
- V1 accepts no explicit item list for provisioning.
- Provisioning derives two normalized requirement sets:
  - the normalized APT requirement set
  - the normalized pacman requirement set

### Manager selection

- The in-world probe from DR-0002 selects exactly one detected world manager or returns unsupported.
- Provisioning executes only the package manager that matches the detected world manager.
- Provisioning MUST NOT fall back from `apt` to `pacman` or from `pacman` to `apt`.

### Mixed-manager rule

- If both the normalized APT requirement set and the normalized pacman requirement set are non-empty, provisioning MUST exit `4` before any OS package-manager command runs.
- The error MUST name the detected world manager and MUST state that the enabled set mixes incompatible system-package methods.
- The error MUST instruct the operator to use a world image whose package manager matches the enabled inventory or to change the enabled inventory so only one system-package manager is in scope.
- No partial provisioning is allowed. When the mixed-manager rule triggers, Substrate executes neither `apt` nor `pacman`.

### Supported execution paths

- If the detected world manager is `apt`, provisioning uses only the normalized APT requirement set.
- If the detected world manager is `pacman`, provisioning uses only the normalized pacman requirement set.
- If the normalized requirement set for the detected manager is empty, phase 2 is a no-op and the command exits `0`.

### Pacman-specific execution

- Pacman provisioning uses the exact command shape selected in DR-0003:

  ```text
  pacman -Sy --noconfirm --needed <packages...>
  ```

- Package arguments MUST follow the normalized pacman requirement order.
- Pacman provisioning MUST NOT invoke AUR helpers or non-official package-manager frontends.
- Pacman provisioning MUST NOT retry, clear lock files, or continue after a non-zero pacman exit.

### `--dry-run`

When `--dry-run` is present:
- The command MUST perform the in-world manager probe.
- The command MUST derive and print the normalized APT and pacman requirement sets.
- The command MUST enforce the mixed-manager rule.
- The command MUST print the detected world manager when the probe succeeds.
- The command MUST perform no mutation.
- Unsupported platform/backend and mixed-manager cases still exit `4`.

### `--verbose`

When `--verbose` is present, stdout MUST include:
- the detected world manager
- the normalized APT and pacman requirement sets

## Runtime contract — `substrate world deps current sync|install`

### Invariant

`substrate world deps current sync|install` MUST NOT invoke `apt`, `dpkg`, or `pacman`.

### System-package scope rule

- `deps current sync`: system-package items are in scope when they are present in the effective enabled set, or when `--all` includes them.
- `deps current install <ITEM...>`: system-package items are in scope only when they are present in the explicit `<ITEM...>` arguments after bundle expansion.

### Fail-early rule

For the runtime in-scope set:
1. Derive the normalized APT requirement set and normalized pacman requirement set.
2. Perform read-only presence probes for the derived requirement sets.
3. If every required system package is already present in the world, proceed with non-system-package behavior from the upstream world-deps contract.
4. If any required system package is missing, exit `4` before any non-system-package mutation runs.

### Remediation invariants

When runtime exits `4` because a required system package is missing, stderr MUST:
- include the exact command

  ```text
  substrate world enable --provision-deps
  ```

- name the missing system-package requirement entries in stable order
- state that runtime system-package mutation is not supported
- on Linux host-native, state that Substrate will not mutate the host OS
- on Windows, state that `substrate world enable --provision-deps` is unsupported on Windows and that Substrate will not mutate the Windows host OS

### `--dry-run` and `--verbose`

- `--dry-run` MUST perform no mutation.
- `--dry-run` MUST still enforce the fail-early rule.
- When a normalized system-package requirement set is non-empty, `--dry-run` MUST print the derived requirement entries in stable order.
- When `--verbose` is present and runtime exits `4`, stderr MUST include the derived missing requirement entries in stable order.

## Exit codes

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- 0: success, including a contract-defined no-op
- 2: invalid inventory or schema input, including invalid `install.method`, invalid `install.pacman` shape, or enabling an unknown item
- 3: world backend unavailable or cannot connect to world-agent when execution is required
- 4: not supported / missing prerequisites, including unsupported platforms/backends, unsupported world manager, mixed-manager enabled sets, manager mismatches, or missing required system packages at runtime
- 5: safety / protected-path violation
- 1: unexpected internal error
