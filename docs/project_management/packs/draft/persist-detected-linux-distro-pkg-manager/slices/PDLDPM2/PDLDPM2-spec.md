# PDLDPM2-spec — Validate the persisted Linux install-state contract

## Behavior delta (single)
- Existing: installer smoke coverage verifies the baseline install/uninstall flows, but it does not yet assert the ADR-0032 platform payload, the Linux `--no-world` persistence rule, dev-installer parity, or omission semantics when `/etc/os-release` is unreadable.
- New: extend the Linux smoke harnesses so `tests/installers/install_state_smoke.sh` and `tests/installers/install_smoke.sh` prove the persisted `host_state.platform.*` contract, `schema_version=1` preservation, host-state preservation, unreadable `/etc/os-release` omission behavior, production `--no-world` persistence, and dev-installer parity.
- Why: ADR-0032 needs executable evidence that the persisted install-state contract is correct, backward compatible, and Linux-only.

## Scope
- Extend `tests/installers/install_state_smoke.sh` to own production-installer assertions for:
  - persisted `host_state.platform.*` keys,
  - `schema_version=1` preservation,
  - preservation of existing `host_state.group` and `host_state.linger` content,
  - reset behavior for corrupt or wrong-schema files,
  - omission of unavailable `host_state.platform.os_release.*` keys.
- Extend `tests/installers/install_smoke.sh` to own:
  - the production-installer `--no-world` persistence assertion,
  - the dev-installer parity assertion for the shared `install_state.json` contract.
- Keep macOS and Windows validation limited to explicit no-delta documentation plus compile parity.

## Inputs (authoritative)
- Feature contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Schema contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Compatibility rules: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md`
- Platform boundary: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/platform-parity-spec.md`
- Upstream implementation slices: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM3/PDLDPM3-spec.md`

## Behavior (authoritative)

### `tests/installers/install_state_smoke.sh` ownership
- `tests/installers/install_state_smoke.sh` is the required validation target path for production-installer metadata assertions in this feature.
- Its `metadata` scenario MUST assert that a successful non-dry-run Linux production install writes `<resolved SUBSTRATE_HOME>/install_state.json` with:
  - `schema_version=1`,
  - the additive `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` fields,
  - `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` when the normalized values are available.
- Its `metadata` scenario MUST also assert that readable `schema_version=1` merges preserve `created_at`, preserve existing readable `host_state.group` and `host_state.linger` content, refresh `updated_at`, and replace only `host_state.platform`.
- Its `metadata` scenario MUST include a subcase where `/etc/os-release` is unreadable or unavailable and MUST assert that the installer still persists `host_state.platform.pkg_manager.*` while omitting unavailable `host_state.platform.os_release.*` keys and omitting placeholder strings.
- Its `cleanup` scenario MUST continue to prove that uninstall cleanup uses preserved `host_state.group` and `host_state.linger` state and ignores the additive `host_state.platform` subtree.
- Its `missing` scenario MUST continue to prove that missing or corrupt install-state metadata falls back to guidance instead of causing a hard failure.

### `tests/installers/install_smoke.sh` ownership
- `tests/installers/install_smoke.sh` is the required validation target path for Linux installer-entrypoint parity in this feature.
- Its `prod-no-world` scenario MUST assert that a successful Linux production install with `--no-world` still creates or updates `<resolved SUBSTRATE_HOME>/install_state.json` and writes the same `schema_version=1` platform payload contract.
- Its `dev` scenario MUST assert that `scripts/substrate/dev-install-substrate.sh` writes the same `schema_version=1` install-state meaning as the production installer, including the same `host_state.platform.*` subtree, omission rules, merge rules, and warning-only failure posture.

### Non-Linux evidence boundary
- macOS and Windows MUST NOT gain new host-state platform smoke tests for this feature.
- Evidence for macOS and Windows is limited to:
  - explicit no-delta statements in `plan.md` and `platform-parity-spec.md`,
  - compile parity via `make ci-compile-parity CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager"`.

## Acceptance criteria
- AC-PDLDPM2-01: `bash tests/installers/install_state_smoke.sh --scenario metadata` asserts that a successful non-dry-run Linux production install creates `<resolved SUBSTRATE_HOME>/install_state.json` with `schema_version=1`, preserved baseline `host_state.group` and `host_state.linger` scaffolding, and the additive `host_state.platform.pkg_manager.selected` plus `host_state.platform.pkg_manager.source` fields.
- AC-PDLDPM2-02: The same `metadata` scenario asserts that when normalized `/etc/os-release` values are available, the written JSON includes `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` with the normalized values from the external detection contract.
- AC-PDLDPM2-03: The same `metadata` scenario includes an unreadable-or-missing `/etc/os-release` subcase and asserts that the install still succeeds, persists `host_state.platform.pkg_manager.*`, omits unavailable `host_state.platform.os_release.*` keys, and does not persist placeholder strings such as `<unknown>` or `null`.
- AC-PDLDPM2-04: The same `metadata` scenario asserts that readable `schema_version=1` merges preserve `created_at`, preserve existing readable `host_state.group` and `host_state.linger`, refresh `updated_at`, and replace only `host_state.platform`; wrong-schema or corrupt files are rewritten as fresh `schema_version=1` documents with a warning-only posture.
- AC-PDLDPM2-05: `bash tests/installers/install_state_smoke.sh --scenario cleanup` and `bash tests/installers/install_state_smoke.sh --scenario missing` continue to prove backward compatibility: uninstall cleanup ignores the additive `host_state.platform` subtree and falls back to guidance when metadata is missing or unreadable.
- AC-PDLDPM2-06: `bash tests/installers/install_smoke.sh --scenario prod-no-world` asserts that a successful Linux production install with `--no-world` still creates or updates `<resolved SUBSTRATE_HOME>/install_state.json` with the persisted platform payload selected by `contract.md` and `install-state-schema-spec.md`.
- AC-PDLDPM2-07: `bash tests/installers/install_smoke.sh --scenario dev` asserts that `scripts/substrate/dev-install-substrate.sh` preserves the same `schema_version=1` install-state meaning as the production installer, including the same `host_state.platform.*` subtree and merge behavior.
- AC-PDLDPM2-08: `make ci-compile-parity CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager"` remains the required cross-platform gate, and the pack records macOS plus Windows as explicit no-delta platforms instead of adding new host-state platform smoke expectations there.

## Out of scope
- Changing the installer persistence contract, schema, or failure posture defined by `contract.md`, `install-state-schema-spec.md`, and `compatibility-spec.md`.
- Adding macOS or Windows host-state platform writes.
- Moving validation ownership away from `tests/installers/install_state_smoke.sh` and `tests/installers/install_smoke.sh`.
