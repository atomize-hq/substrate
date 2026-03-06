# persist-detected-linux-distro-pkg-manager — contract surface

This file is the single authoritative operator-facing contract for persisting detected Linux distro and package-manager metadata in `install_state.json`.

Decision inputs:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` (DR-0001/2/3)
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`

External authoritative inputs (this feature does not redefine these surfaces):
- `SUBSTRATE_HOME` meaning and default path resolution: `docs/reference/env/contract.md`
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Linux package-manager detection algorithm, selected-manager vocabulary, and `pkg_manager.source` enum semantics: `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- Linux `/etc/os-release` parsing and normalization rules used by persisted `os_release.*` values: `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

## In scope

- Scripts:
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`
- File surface:
  - `<resolved SUBSTRATE_HOME>/install_state.json`
- Platforms:
  - Linux: behavior defined by this contract
  - macOS: no behavior change introduced by this feature
  - Windows: no behavior change introduced by this feature

## Contract summary

- This feature introduces no new CLI commands, flags, config files, environment variables, log fields, or exit codes.
- This feature extends the existing `install_state.json` file and does not create a second metadata file.
- Linux install-state persistence uses one shared contract across `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh`.
- This pack is authoritative for the feature directory `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`. The ADR path drift to `draft/stashing-ferret/` is an external follow-up and does not change the contract defined here.

## File path and write boundary

- The persisted metadata path is exactly `<resolved SUBSTRATE_HOME>/install_state.json`.
- Metadata writes introduced by this feature MUST occur only under the resolved `SUBSTRATE_HOME`.
- The installer MUST NOT write a second platform-metadata file such as `host_platform.json`.

## Linux write contract

### Successful install guarantee

For Linux only:
- A successful non-dry-run install MUST leave `<resolved SUBSTRATE_HOME>/install_state.json` present when either installer entrypoint completes with exit code `0`.
- The file-presence guarantee applies even when no `host_state.group` or `host_state.linger` event payload was produced during the install.
- The file-presence guarantee applies to normal installs and `--no-world` installs equally.

### `--no-world` rule

For Linux only:
- Passing `--no-world` MUST NOT suppress install-state persistence.
- A successful non-dry-run Linux install invoked with `--no-world` MUST create or update `<resolved SUBSTRATE_HOME>/install_state.json` using the same metadata contract as a world-enabled run.

### Dry-run rule

For all platforms:
- A dry-run invocation MUST NOT create, rewrite, truncate, or merge `<resolved SUBSTRATE_HOME>/install_state.json`.
- Dry-run output may report what would be persisted, but the on-disk install-state file MUST remain unchanged.
- Because dry-run performs no persistence, the Linux successful-install write guarantee applies only to non-dry-run installs.

### Partial-input rule

For Linux only:
- The installer MUST persist `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` whenever the upstream detection contract produces them.
- If `/etc/os-release` data is partially unavailable, unavailable `host_state.platform.os_release.*` keys MUST be omitted from JSON rather than written as sentinel strings or `null`.
- Missing or unreadable `/etc/os-release` data MUST NOT prevent install-state persistence when `pkg_manager.*` is available.
- This contract does not redefine how `pkg_manager.*` or `os_release.*` values are derived; it only defines when they are persisted.

## Read contract for downstream consumers

Later guidance or diagnostic consumers that read installer metadata MUST use this precedence:
1. Read `<resolved SUBSTRATE_HOME>/install_state.json`.
2. If the file exists, parses successfully, and matches the current compatibility contract, prefer the persisted `host_state.platform.*` values.
3. If the file is missing, unreadable, corrupt, or unusable for compatibility reasons, fall back to runtime detection without failing the higher-level command solely because persisted metadata was unavailable.

The compatibility rules for corrupt JSON, wrong-schema files, and additive merges are authoritative in `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md`.

## Failure posture

- Metadata persistence is fail-open.
- If install-state read, merge, or write steps fail during an otherwise successful install, the installer MUST continue the successful install path and MUST NOT introduce a new non-zero exit code solely because persistence failed.
- The installer MAY emit a warning to stderr that metadata persistence failed.
- A persistence failure MUST NOT broaden host inspection, retry with different detection semantics, or write outside the resolved `SUBSTRATE_HOME`.

## Exit-code posture

- Taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- New exit codes introduced by this feature: none
- Metadata persistence MUST NOT add a new success-path non-zero exit on either installer entrypoint.

## Platform guarantees

### Linux

- Successful non-dry-run installs through either installer entrypoint MUST create or update `<resolved SUBSTRATE_HOME>/install_state.json`.
- Linux persistence MUST use the additive `host_state.platform.*` schema defined by `install-state-schema-spec.md`.
- Linux persistence MUST preserve existing `host_state.group` and `host_state.linger` content per `compatibility-spec.md`.

### macOS

- This feature introduces no new `host_state.platform.*` write contract on macOS.
- Existing macOS installer behavior outside `host_state.platform.*` remains unchanged by this feature.

### Windows

- This feature introduces no new `host_state.platform.*` write contract on Windows.
- Existing Windows installer behavior outside `host_state.platform.*` remains unchanged by this feature.

## Authority boundaries and invariants

- Detection authority boundary:
  - This pack persists the final detection outputs.
  - `best-effort-distro-package-manager` remains authoritative for package-manager selection, `pkg_manager.source` spellings, and `/etc/os-release` parsing semantics.
- Compatibility invariants:
  - `schema_version` remains `1`.
  - Existing `host_state.group` and `host_state.linger` content MUST be preserved during Linux metadata updates.
- Data-minimization invariant:
  - Newly persisted platform metadata is limited to `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`.
