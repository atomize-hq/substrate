# persist-detected-linux-distro-pkg-manager — contract surface

This file is the single authoritative operator-facing contract for persisted Linux install metadata introduced by `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`.

Decision inputs:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`

External authoritative inputs (this feature does not redefine these surfaces):
- `docs/reference/env/contract.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

Authority note:
- The canonical planning-pack directory for this feature is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`.
- ADR-0032 still references `docs/project_management/packs/draft/stashing-ferret/`. Those path references are stale and do not override this contract.

## In scope

- Entry points:
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`
- Persistence target:
  - `<resolved SUBSTRATE_HOME>/install_state.json`
- Platforms:
  - Linux: behavior defined by this contract.
  - macOS: no new `host_state.platform.*` write contract and no new file-presence guarantee from this feature.
  - Windows: no new `host_state.platform.*` write contract and no new file-presence guarantee from this feature.

## Locked write contract

### Successful Linux installs

- After any successful Linux install through either in-scope entry point, `<resolved SUBSTRATE_HOME>/install_state.json` MUST exist.
- The Linux write guarantee applies even when no `host_state.group` or `host_state.linger` event payload exists.
- The Linux write guarantee applies when `--no-world` is used. Skipping world provisioning MUST NOT suppress the persisted install-state surface selected by this feature.
- This feature extends the existing `install_state.json` file. It does not create a second metadata file.

### Dry-run rule

- `scripts/substrate/install-substrate.sh --dry-run` MUST NOT create, replace, or update `<resolved SUBSTRATE_HOME>/install_state.json`.
- During production-installer dry-run, the installer MUST log that install-state persistence was skipped because the invocation was non-mutating.
- `scripts/substrate/dev-install-substrate.sh` defines no dry-run surface in this feature.

## Locked stored-data contract

- This feature adds only the Linux persistence of `host_state.platform.*` under the existing `schema_version=1` install-state file.
- Persisted `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` MUST mirror the final selected values produced by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
- This feature MUST NOT redefine the package-manager detection algorithm, the manager-name vocabulary, or the `pkg_manager.source` enum semantics.
- When `/etc/os-release` inputs are partially unavailable, the installer MUST persist the available platform fields and omit unavailable `host_state.platform.os_release.*` keys. It MUST NOT persist placeholder strings for unavailable os-release values.
- Writes that add `host_state.platform.*` MUST preserve existing `host_state.group` and `host_state.linger` content.
- The persisted platform payload is limited to:
  - `host_state.platform.os_release.id`
  - `host_state.platform.os_release.id_like`
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source`

## Failure posture

- Metadata read, merge, parse, or write failures MUST NOT introduce a new non-zero exit on an otherwise successful install.
- On a metadata-persistence failure, the installer MUST emit a warning and continue the successful install path.
- Metadata persistence MUST write only under the resolved `SUBSTRATE_HOME`.
- This feature MUST NOT expand persistence beyond the install-state file and the locked platform fields listed above.

## Downstream read contract

- Later guidance consumers MUST prefer persisted `host_state.platform.*` metadata when `<resolved SUBSTRATE_HOME>/install_state.json` is readable and the requested fields are present.
- If the install-state file is missing, unreadable, on an incompatible schema, or lacks the requested platform fields, consumers MUST fall back to runtime detection instead of failing closed on the persisted-file read.
- Runtime fallback for package-manager guidance MUST reuse `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.

## Exit-code posture

- Exit-code taxonomy remains `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`.
- This feature introduces no new success-path exit code.
- A successful Linux install remains successful even when metadata persistence is skipped for dry-run or degrades to a warning after a metadata read, merge, parse, or write failure.
