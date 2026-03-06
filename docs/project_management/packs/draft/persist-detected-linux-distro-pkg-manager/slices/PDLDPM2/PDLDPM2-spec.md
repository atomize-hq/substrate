# PDLDPM2-spec — Validate the Linux install-state persistence contract

## Behavior delta (single)
- Existing: Linux installer smoke coverage treats `install_state.json` mainly as cleanup-state metadata and does not fully own ADR-0032 assertions across both installer entrypoints.
- New: `tests/installers/install_state_smoke.sh` and `tests/installers/install_smoke.sh` become the canonical Linux validation surfaces for the persisted `host_state.platform.*` contract, its compatibility rules, and installer-entrypoint parity.
- Why: the Linux-only persistence contract is only credible if the fresh-write path, merge path, replacement path, and installer parity remain observable in automation.

## Scope
- Extend Linux smoke coverage only.
- Lock the validation ownership split between:
  - `tests/installers/install_state_smoke.sh`
  - `tests/installers/install_smoke.sh`
- Cover production-installer assertions, production `--no-world`, and dev-installer parity for the same install-state contract.
- Reuse the existing temp-root and stubbed privileged-command posture so validation remains host-safe.

## Inputs (authoritative)
- Operator contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Schema contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Compatibility contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md`
- Platform guarantees: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/platform-parity-spec.md`
- Implementation slice specs:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM3/PDLDPM3-spec.md`

## Behavior (authoritative)

### Validation ownership split
- `tests/installers/install_state_smoke.sh` MUST own Linux assertions for `scripts/substrate/install-substrate.sh` that verify fresh-file creation, compatible-file preservation, corrupt or wrong-schema replacement, and unreadable-`/etc/os-release` omission behavior.
- `tests/installers/install_smoke.sh` MUST own Linux assertions that verify the production installer success path, the production `--no-world` path, and dev-installer parity for the shared install-state contract.
- The two harnesses together are the full ADR-0032 Linux validation bundle; neither harness is allowed to redefine the JSON schema, package-manager vocabulary, or detection precedence owned by dependency docs.

### Linux execution rules
- All ADR-0032 behavior assertions run on Linux only.
- The validation surface MUST keep filesystem effects inside harness temp roots and MUST continue using harness stubs for privileged or systemd operations.
- Linux smoke coverage MUST verify both a fresh `install_state.json` write and a write against a compatible pre-existing file.
- Linux smoke coverage MUST verify that dry-run performs no persistence and that `--no-world` does not suppress persistence on successful non-dry-run installs.

### Non-Linux evidence boundary
- macOS and Windows remain parity-only platforms for this feature.
- This slice MUST NOT add non-Linux assertions that expect `host_state.platform.*` writes.
- Non-Linux no-delta evidence is recorded in `platform-parity-spec.md` and `plan.md`, not inside Linux behavior-smoke assertions.

## Acceptance criteria
- AC-PDLDPM2-01: `bash tests/installers/install_state_smoke.sh --scenario metadata` exits `0` on Linux and verifies that a successful non-dry-run `scripts/substrate/install-substrate.sh` run creates or updates `<resolved SUBSTRATE_HOME>/install_state.json` with `schema_version: 1` plus only the four allowed `host_state.platform.*` keys.
- AC-PDLDPM2-02: `bash tests/installers/install_smoke.sh --scenario prod-no-world` exits `0` on Linux and verifies that a successful non-dry-run production install invoked with `--no-world` still leaves `<resolved SUBSTRATE_HOME>/install_state.json` present under the same path and schema contract as a world-enabled run.
- AC-PDLDPM2-03: `bash tests/installers/install_state_smoke.sh --scenario metadata` verifies that, when a compatible existing `install_state.json` already contains unknown top-level keys plus `host_state.group` and `host_state.linger`, the rewritten file preserves those compatible fields while replacing `host_state.platform` and updating `updated_at`.
- AC-PDLDPM2-04: `bash tests/installers/install_state_smoke.sh --scenario metadata` verifies that a corrupt existing file and a wrong-schema existing file are each replaced by a fresh `schema_version: 1` document built from current-run data while the installer preserves the feature’s fail-open posture.
- AC-PDLDPM2-05: `bash tests/installers/install_state_smoke.sh --scenario metadata` verifies that when `/etc/os-release` is unreadable, `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` remain persisted when available, `host_state.platform.os_release` is omitted, and no `null` or sentinel substitute is written.
- AC-PDLDPM2-06: `bash tests/installers/install_smoke.sh --scenario dev` exits `0` on Linux and verifies that `scripts/substrate/dev-install-substrate.sh` writes `<resolved SUBSTRATE_HOME>/install_state.json` under the same path, schema, dry-run, and `--no-world` contract as the production installer.
- AC-PDLDPM2-07: The combined Linux validation bundle runs without mutating the real host by keeping filesystem effects inside harness temp roots and by using the harness stubs for privileged and systemd operations.

## Out of scope
- Changing package-manager detection, `pkg_manager.source` semantics, or `/etc/os-release` normalization owned by `best-effort-distro-package-manager`.
- Adding new macOS or Windows behavior-smoke assertions for `host_state.platform.*`.
- Updating operator docs or ADR references; those closeout edits remain outside this slice.
