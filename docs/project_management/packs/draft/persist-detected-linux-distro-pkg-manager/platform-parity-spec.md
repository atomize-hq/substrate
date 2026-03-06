# persist-detected-linux-distro-pkg-manager — platform parity spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for platform guarantees and permitted divergences for ADR-0032 install-state persistence.

## Required platforms

- Behavior platforms (smoke required): `linux`
- CI parity platforms (parity required): `linux`, `macos`, `windows`
- WSL required: `false`

## Guarantees (explicit)

What must hold on Linux:

- Successful non-dry-run runs through `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` create or update `<resolved SUBSTRATE_HOME>/install_state.json`.
- The Linux write contract is identical for world-enabled and `--no-world` runs.
- Dry-run never creates, rewrites, truncates, or merges `<resolved SUBSTRATE_HOME>/install_state.json`.
- Persisted `host_state.platform.*` data is limited to the four schema-owned keys and preserves existing `host_state.group` and `host_state.linger` per `install-state-schema-spec.md` and `compatibility-spec.md`.
- Missing or unreadable `/etc/os-release` input omits only unavailable `host_state.platform.os_release.*` keys; available `host_state.platform.pkg_manager.*` data remains persistable.

What must remain unchanged on macOS and Windows:

- This feature introduces no new `host_state.platform.*` write contract.
- This feature introduces no second metadata file and no new installer flag, config, or environment-variable surface.
- Existing installer behavior outside the Linux metadata contract remains unchanged by ADR-0032.

What may diverge (explicit list + rationale):

- Linux carries new behavior-smoke assertions because Linux is the only behavior-delta platform for this feature.
- macOS and Windows use parity-only validation evidence because the touched installer scripts are shared code paths but the new persisted metadata contract is Linux-only.

## Known platform hazards (explicit)

- `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` are shared installer entrypoints, so Linux metadata changes can regress non-Linux flows even though non-Linux has no new write contract.
- `tests/installers/install_smoke.sh` skips on non-Linux hosts, so macOS and Windows evidence for this feature relies on parity gates plus explicit no-delta closeout notes rather than behavior smoke.
- `tests/installers/install_state_smoke.sh` exercises temp-scoped metadata files and replacement paths; Linux validation must keep those assertions host-safe and temp-root bounded.

## Validation evidence (explicit)

- Linux smoke scripts required:
  - `bash tests/installers/install_state_smoke.sh --scenario metadata`
  - `bash tests/installers/install_smoke.sh --scenario prod`
  - `bash tests/installers/install_smoke.sh --scenario prod-no-world`
  - `bash tests/installers/install_smoke.sh --scenario dev`
- CI parity evidence required:
  - parity execution for the touched installer validation surfaces on `linux`, `macos`, and `windows`
  - closeout note that macOS and Windows gained no `host_state.platform.*` write assertions and no new metadata file
- Operator-doc closeout evidence required:
  - `docs/INSTALLATION.md` reflects one shared `install_state.json` surface for both installers while stating that the new `host_state.platform.*` contract is Linux-only
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` points at this feature directory and matches the Linux-only parity contract

## Acceptance criteria (testable)

- Linux:
  - The Linux smoke bundle proves that successful non-dry-run installs through both installer entrypoints create or update `<resolved SUBSTRATE_HOME>/install_state.json`.
  - The Linux smoke bundle proves that `--no-world` does not suppress install-state persistence and that dry-run performs no persistence.
  - The Linux smoke bundle proves that the additive `host_state.platform.*` contract preserves compatible `host_state.group` and `host_state.linger` content and keeps `schema_version` at `1`.
- macOS and Windows:
  - Closeout evidence states that ADR-0032 introduces no new `host_state.platform.*` write contract on either platform.
  - Parity validation confirms that the touched installer entrypoints keep their existing non-Linux behavior while Linux-only assertions remain excluded.
