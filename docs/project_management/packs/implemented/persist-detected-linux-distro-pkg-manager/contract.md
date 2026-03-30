# persist-detected-linux-distro-pkg-manager — contract surface

This file is the single authoritative operator-facing contract for persisting Linux distro and package-manager detection into `install_state.json` for this feature.

Decision inputs:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`

External authoritative inputs:
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Detection contract: `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

## Authority + scope

- Canonical planning-pack path for this feature: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- Stale ADR references to `docs/project_management/packs/draft/stashing-ferret/` do not override this contract or this feature directory.
- In scope:
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`
  - Linux installer metadata persistence into `install_state.json`
- Out of scope:
  - package-manager detection semantics
  - world-deps provisioning behavior
  - uninstaller path-alignment work
  - macOS and Windows platform-metadata writes

## CLI

- This ADR introduces no new commands.
- This ADR introduces no new flags.
- Existing installer flags that participate in this contract remain:
  - `--prefix`
  - `--no-world`
  - hosted installer `--dry-run`
- This ADR introduces no new environment variable.
- `SUBSTRATE_INSTALL_OS_RELEASE_PATH` remains owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
- This ADR introduces no new log field and no new trace field.

## Config + path semantics

- The canonical metadata file for this feature is `install_state.json`.
- The producer write path is `<effective_prefix>/install_state.json`.
- `effective_prefix` resolves from the installer's effective `--prefix` value.
- The default prefix is `~/.substrate`.
- The operator-facing name for the same canonical path is the default-prefix alias `$SUBSTRATE_HOME/install_state.json`.
- This feature introduces no second metadata file and no feature-local config file.
- `schema_version` remains `1`.
- The persisted payload contract is additive-only under `schema_version = 1`.
- When present, `host_state.platform` contains both `os_release` and `pkg_manager`.
- `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` copy the detector's normalized outputs verbatim, including the literal `<unknown>` sentinel when emitted.
- `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` copy the upstream detection contract verbatim; `best-effort-distro-package-manager/contract.md` owns that vocabulary.
- Exact field paths, types, examples, and merge rules belong to `install-state-schema-spec.md`.

## Producer contract

- Hosted install and dev install share one metadata-producer contract.
- Successful Linux producer flows MUST create or update the canonical `install_state.json` file even when no `host_state.group` or `host_state.linger` event was recorded.
- Linux producer flows covered by this rule are:
  - hosted install
  - hosted install with `--no-world`
  - dev install
  - dev install with `--no-world`
- Hosted installer `--dry-run` is a no-write branch.
- macOS and Windows producer flows MUST NOT add `host_state.platform.*` metadata under this ADR.
- `pkg_manager.selected` and `pkg_manager.source` MUST be copied verbatim from the detection contract and MUST NOT be re-derived locally.

## Failure posture

- Missing `/etc/os-release` MUST NOT fail an otherwise successful Linux install.
- Metadata read failure MUST degrade to warning-only behavior.
- Metadata write failure MUST degrade to warning-only behavior.
- Metadata persistence failure MUST NOT change the exit status of an otherwise successful install.
- Dry-run MUST NOT create `install_state.json`, a metadata temp file, or a metadata-only parent directory.
- When `/etc/os-release` is missing or unreadable, installers MUST continue to persist package-manager metadata when the detection contract still produced it.
- This ADR MUST NOT invent fallback `os_release` values outside the schema rules owned by `install-state-schema-spec.md`.

## Update + atomicity rules

- The canonical file MUST be updated by writing the next document to `<effective_prefix>/install_state.json.tmp`.
- The temp file MUST live in the same directory as the canonical file.
- The replace step MUST occur only after the temp file contains a complete JSON document.
- The replace step MUST use a single file-replacement operation from the temp path to the canonical path.
- In-place truncation of `install_state.json` is not allowed.
- A failed temp-file write MUST leave the prior canonical file unchanged.
- A failed temp-file write MUST remove the temp file when removal succeeds.
- Updates MUST preserve existing `host_state.group` and `host_state.linger` data.
- Updates MUST preserve unknown keys unless another authoritative spec owns those keys.

## Future-consumer read contract

- Consumers that generate operator guidance from host platform metadata MUST prefer persisted `host_state.platform.*` values when the file is present and readable.
- Consumers MUST fall back to runtime detection when the persisted file is missing or unreadable.
- Consumers MUST treat persisted metadata as advisory guidance input, not as a reason to fail commands.
- Consumers MUST NOT require backfill or migration beyond normal successful Linux install writes.

## Exit codes

- Taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This ADR introduces no new exit code.
- Successful installer runs remain success even when metadata persistence degraded to warning-only behavior.
- Failure semantics for invalid package-manager override values and package-manager selection remain owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.

## Platform guarantees

- Linux:
  - successful producer flows create or update `install_state.json`
  - the persisted platform metadata is limited to the schema owned by `install-state-schema-spec.md`
  - metadata persistence remains best-effort
- macOS:
  - no new `host_state.platform.*` writes
- Windows:
  - no new `host_state.platform.*` writes

## Protected paths + invariants

- This ADR writes only under the effective Substrate home derived from the effective install prefix.
- This ADR does not authorize writes outside the effective Substrate home.
- `install_state.json` remains the only persisted metadata file touched by this ADR.
- This ADR does not rename or remove existing installer metadata fields.
- This ADR does not persist secrets, hostnames, full environment snapshots, policy data, or new telemetry fields.
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` remains the single source of truth for Linux distro detection, supported package-manager spellings, and `pkg_manager.source` vocabulary.
