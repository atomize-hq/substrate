# best-effort-distro-package-manager — contract surface

This file is the single authoritative operator-facing contract for Linux hosted package-manager detection, override, reporting, and wrapper exit-status behavior introduced by ADR-0031.

Decision inputs:
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md`
- `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

External authoritative inputs:
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Downstream persistence contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`

## Authority + scope

- Canonical planning-pack path for this feature: `docs/project_management/packs/implemented/best-effort-distro-package-manager/`
- In scope:
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/install.sh`
  - `docs/INSTALLATION.md`
  - `docs/reference/env/contract.md`
  - `tests/installers/pkg_manager_detection_smoke.sh`
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh`
- Out of scope:
  - `scripts/substrate/dev-install-substrate.sh`
  - runtime crates and world backends
  - persistence into `install_state.json`
  - macOS and Windows behavior changes
  - any new config file, persistent config key, trace field, or log field

## CLI

- Canonical direct installer entrypoint: `scripts/substrate/install-substrate.sh`
- Wrapper entrypoint: `scripts/substrate/install.sh`
- New Linux-only flag:
  - `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`
  - Allowed values are exactly: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`
  - Precedence: highest
  - Invalid value exits with code `2`
  - Valid value that is not found in `PATH` exits with code `3`
  - A valid `--pkg-manager` selection never falls back to any other manager
- Legacy Linux-only env override:
  - `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>`
  - Allowed values are exactly: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`
  - Precedence: lower than `--pkg-manager`, higher than os-release mapping and `PATH` probing
  - Invalid value exits with code `2`
  - Valid value that is not found in `PATH` exits with code `3`
  - A valid `PKG_MANAGER` selection never falls back to any other manager
- Exact selection precedence for Linux hosted installs:
  1. `--pkg-manager`
  2. `PKG_MANAGER`
  3. os-release mapping
  4. `PATH` probe in fixed order `apt-get -> dnf -> yum -> pacman -> zypper`
- Supported selected-manager vocabulary is fixed:
  - `apt-get`
  - `dnf`
  - `yum`
  - `pacman`
  - `zypper`
- `pkg_manager.source` vocabulary is fixed:
  - `flag`
  - `env`
  - `os_release`
  - `path_probe`

## Environment variables

### `PKG_MANAGER`

- Scope: Linux hosted installer only
- Meaning: explicit legacy override for the package manager used to install Linux host prerequisites
- Allowed values: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`
- Precedence: second, after `--pkg-manager`
- Absence semantics: unset or empty means “no explicit legacy override”
- Failure posture:
  - invalid value exits with code `2`
  - valid value missing from `PATH` exits with code `3`
  - this env var never falls back to os-release mapping or `PATH` probing after it selects a manager

### `SUBSTRATE_INSTALL_OS_RELEASE_PATH`

- Scope: Linux hosted installer only
- Meaning: alternate file path used for the os-release detection input
- Allowed value shape:
  - unset or empty: use `/etc/os-release`
  - non-empty absolute path to a readable regular file: use that file instead of `/etc/os-release`
  - non-empty value that is not an absolute path, is not a readable regular file, or cannot be opened: treat os-release input as unavailable
- Precedence:
  - this env var only affects the os-release input path
  - it does not override `--pkg-manager` or `PKG_MANAGER`
  - when it is set, the installer does not read `/etc/os-release`
- Absence semantics:
  - unset or empty means `/etc/os-release`
  - invalid or unreadable alternate path renders distro fields as `<unknown>` and continues to the remaining selection stages without falling back to `/etc/os-release`
- Platform rule:
  - ignored by macOS and Windows behavior because this ADR does not change those platforms

## os-release read + parsing contract

- Canonical production input path is `/etc/os-release` unless `SUBSTRATE_INSTALL_OS_RELEASE_PATH` selects an alternate file.
- Detection reads only the keys `ID` and `ID_LIKE`.
- Detection performs no network call.
- Parsing is line-oriented and never executes shell code.
- Exact parsing rules:
  - ignore blank lines
  - ignore lines whose first non-space character is `#`
  - read only lines whose key is exactly `ID` or `ID_LIKE`
  - split on the first `=`
  - trim leading and trailing ASCII whitespace around the raw value
  - strip one surrounding pair of matching single quotes or double quotes when present
  - do not evaluate escapes, variable expansions, command substitutions, or backticks
  - normalize captured values to ASCII lowercase for matching and for the emitted decision line
  - if a key appears more than once, the last well-formed assignment wins
- Missing-data sentinel is fixed: `<unknown>`
- Emitted distro fields:
  - `distro_id` is the normalized `ID` value or `<unknown>`
  - `distro_id_like` is the normalized `ID_LIKE` value or `<unknown>`

## os-release mapping contract

### Match inputs

- `ID` matching uses the normalized `distro_id` value.
- `ID_LIKE` matching uses whitespace-separated tokens from the normalized `distro_id_like` value.

### Family table

| Family | Match rule | Selected manager |
| --- | --- | --- |
| Debian/Ubuntu | `ID` is one of `debian`, `ubuntu`, `linuxmint`, `pop` or any `ID_LIKE` token is `debian` or `ubuntu` | `apt-get` if `apt-get` is in `PATH`; otherwise no selection at this stage |
| Fedora/RHEL | `ID` is one of `fedora`, `rhel`, `centos`, `rocky`, `almalinux`, `ol`, `amzn` or any `ID_LIKE` token is `fedora` or `rhel` | `dnf` if `dnf` is in `PATH`; otherwise `yum` if `yum` is in `PATH`; otherwise no selection at this stage |
| Arch | `ID` is one of `arch`, `manjaro`, `endeavouros`, `arcolinux`, `artix`, `garuda` or any `ID_LIKE` token is `arch` | `pacman` if `pacman` is in `PATH`; otherwise no selection at this stage |
| SUSE | `ID` contains `suse` or `opensuse` or any `ID_LIKE` token contains `suse` or `opensuse` | `zypper` if `zypper` is in `PATH`; otherwise no selection at this stage |

### Stage result

- When the family table selects a manager, `pkg_manager.source` is `os_release`.
- When the family table does not select a manager, control falls through to the `PATH` probe stage.

## `PATH` probe contract

- The fixed ordered probe list is:
  1. `apt-get`
  2. `dnf`
  3. `yum`
  4. `pacman`
  5. `zypper`
- The installer probes all five managers with `command -v`.
- When no supported manager is found, the `PATH` probe stage makes no selection.
- When exactly one supported manager is found, the stage selects that manager and sets `pkg_manager.source=path_probe`.
- When more than one supported manager is found, the stage:
  - selects the earliest detected manager in the fixed ordered probe list
  - sets `pkg_manager.source=path_probe`
  - emits exactly one warning line to stderr before the decision line

### Multi-manager warning line

- Warning template:
  - `Multiple supported package managers found in PATH: <manager_list>; selecting <selected> by fixed probe order (apt-get -> dnf -> yum -> pacman -> zypper). Override with --pkg-manager <apt-get|dnf|yum|pacman|zypper> or PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>.`
- `<manager_list>` is a comma-and-space delimited list of the detected managers in the fixed ordered probe list.
- `<selected>` is the earliest manager from the same ordered list that was present in `PATH`.

## Installer output contract

- Linux hosted installs print the stable decision line to stderr exactly once when a concrete manager has been selected.
- Decision-line placement:
  - after manager selection is complete
  - before any package-manager install command is invoked
  - before any feature-specific exit `3` remediation branch is emitted
- Decision-line template:
  - `Detected distro: <id> (like: <id_like>), using package manager: <pkg_manager> (source: <flag|env|os_release|path_probe>)`
- `<id>` is the normalized `ID` value or `<unknown>`
- `<id_like>` is the normalized `ID_LIKE` value or `<unknown>`
- The decision line is not emitted for:
  - invalid `--pkg-manager`
  - invalid `PKG_MANAGER`
  - exit `4` branches where no supported manager was selected
- When the multi-manager `PATH` warning is required, the warning line appears before the decision line.

## Wrapper interaction rules

- `scripts/substrate/install.sh` passes all installer arguments through to `scripts/substrate/install-substrate.sh`.
- For this feature’s explicit contract branches, `scripts/substrate/install.sh` preserves the direct installer exit status:
  - `0`
  - `2`
  - `3`
  - `4`
- `scripts/substrate/install.sh` does not collapse those feature-specific non-zero exits to `1`.

## Exit codes

- Taxonomy source: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Feature-specific meanings:

| Exit code | Meaning |
| --- | --- |
| `0` | Linux install path completed successfully |
| `2` | Invalid `--pkg-manager` value or invalid `PKG_MANAGER` value |
| `3` | A manager selected by `--pkg-manager` or `PKG_MANAGER` was not found in `PATH` |
| `4` | No supported manager was selected after os-release mapping and `PATH` probing |

- Shared taxonomy slots retained without modification:
  - `1` keeps the shared generic installer-failure meaning
  - `5` keeps the shared higher-order installer taxonomy meaning when another feature uses it

## Remediation content contract

### Exit `2` remediation

- The error message names the invalid value.
- The error message names the input source:
  - `--pkg-manager`
  - `PKG_MANAGER`
- The error message lists the exact allowed values:
  - `apt-get`, `dnf`, `yum`, `pacman`, `zypper`
- The error message tells the operator to rerun with one of the allowed values or remove the invalid override.

### Exit `3` remediation

- The error message names the selected manager.
- The error message names the selecting source:
  - `--pkg-manager`
  - `PKG_MANAGER`
- The error message states that the selected manager was not found in `PATH`.
- The error message tells the operator to install that manager or rerun with another allowed manager from the fixed vocabulary.

### Exit `4` remediation

- The error message states that no supported package manager was detected.
- The error message lists the exact missing prerequisite commands for the current installer branch.
- The error message tells the operator to install the missing prerequisites manually and rerun.
- The error message tells the operator that a rerun may also use:
  - `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`
  - `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>`

## Platform guarantees

- Linux:
  - this full contract applies
  - detection is local-only and reads only the selected os-release file plus `PATH`
  - the wrapper preserves feature-specific exit classes `0`, `2`, `3`, and `4`
- macOS:
  - no behavior change under ADR-0031
- Windows:
  - no behavior change under ADR-0031

## Protected paths + invariants

- This ADR introduces no new config file.
- This ADR introduces no persistent config key.
- This ADR introduces no structured log field and no structured trace field.
- This ADR does not change the prerequisite command set; it changes only how the installer selects the package manager used to install that existing command set.
- This ADR does not change per-manager package-name mapping tables.
- This ADR does not change `scripts/substrate/dev-install-substrate.sh`.
- This ADR does not change world-deps behavior or guest-world provisioning semantics.
- `docs/INSTALLATION.md`, `docs/reference/env/contract.md`, `tests/installers/pkg_manager_detection_smoke.sh`, and `docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh` must reuse the precedence chain, manager vocabulary, `pkg_manager.source` vocabulary, `<unknown>` sentinel, warning line, decision line, exit-code meanings, and remediation posture defined here without drift.
