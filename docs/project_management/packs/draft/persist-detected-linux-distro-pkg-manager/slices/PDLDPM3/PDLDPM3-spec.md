# PDLDPM3-spec — Keep dev-installer parity on the shared install-state file

## Behavior delta (single)
- Existing: `scripts/substrate/dev-install-substrate.sh` writes the same `install_state.json` path used by the production installer, but its Linux metadata behavior can drift because ADR-0032 does not yet pin parity for the shared file contract.
- New: Successful non-dry-run Linux runs of `scripts/substrate/dev-install-substrate.sh` write the same `schema_version=1` install-state contract as the production installer, including the additive `host_state.platform` subtree, the same merge/reset rules, and the same `--no-world` persistence rule.
- Why: One on-disk path must keep one meaning across both installer entry points.

## Scope
- Define the Linux dev-installer parity rule for `<resolved SUBSTRATE_HOME>/install_state.json`.
- Require the dev installer to use the same schema, omission, and compatibility rules as the production installer for persisted platform metadata.
- Define the exact validation harness that owns dev-installer parity assertions.

## Inputs (authoritative)
- Feature contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Schema rules: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Compatibility rules: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md`
- Decision register: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
- Dev-installer validation harness: `tests/installers/install_smoke.sh`

## Behavior (authoritative)
### Shared file contract
- `scripts/substrate/dev-install-substrate.sh` MUST use `<resolved SUBSTRATE_HOME>/install_state.json` as the only persisted file for the ADR-0032 Linux metadata surface.
- The dev installer MUST keep `schema_version=1`.
- The dev installer MUST use the same `host_state.platform` key set and omission rules defined by `install-state-schema-spec.md`.

### Linux parity requirements
- Successful non-dry-run Linux dev installs MUST create or update `<resolved SUBSTRATE_HOME>/install_state.json` even when no group/linger event payload was recorded.
- Successful non-dry-run Linux dev installs with `--no-world` MUST still create or update `<resolved SUBSTRATE_HOME>/install_state.json`.
- When the prior file is readable JSON with `schema_version=1`, the dev installer MUST preserve readable `host_state.group` and `host_state.linger`, preserve `created_at`, refresh `updated_at`, and replace `host_state.platform` with the current run's payload.
- When the prior file is corrupt JSON or readable JSON with `schema_version != 1`, the dev installer MUST emit a warning, discard the prior contents for merge purposes, and write a fresh `schema_version=1` document.
- If metadata read, merge, parse, or write work fails during an otherwise successful Linux dev install, the dev installer MUST emit a warning and continue the successful install path without introducing a new non-zero exit.

### Validation ownership
- `tests/installers/install_smoke.sh` is the required validation target path for dev-installer parity assertions in this feature.
- The `dev` scenario in `tests/installers/install_smoke.sh` MUST assert that the dev installer preserves the same install-state meaning selected by `contract.md`, `install-state-schema-spec.md`, and `compatibility-spec.md`.

## Acceptance criteria
- AC-PDLDPM3-01: A successful non-dry-run Linux invocation of `scripts/substrate/dev-install-substrate.sh` creates or updates `<resolved SUBSTRATE_HOME>/install_state.json` using `schema_version=1` and the same `host_state.platform` structure selected for the production installer.
- AC-PDLDPM3-02: A successful non-dry-run Linux invocation of `scripts/substrate/dev-install-substrate.sh --no-world` still creates or updates `<resolved SUBSTRATE_HOME>/install_state.json` with the current platform payload.
- AC-PDLDPM3-03: The dev installer uses the same omission rules as the production installer: unavailable normalized `os_release.id` and `os_release.id_like` fields are omitted from JSON rather than stored as placeholder strings or sentinel values.
- AC-PDLDPM3-04: When the existing install-state file is readable JSON with `schema_version=1`, the dev installer preserves readable `host_state.group` and `host_state.linger`, preserves `created_at`, refreshes `updated_at`, and replaces only `host_state.platform`.
- AC-PDLDPM3-05: When the existing install-state file is corrupt JSON or readable JSON with `schema_version != 1`, the dev installer emits a warning, rewrites a fresh `schema_version=1` document, and does not convert the successful dev install into a new failure exit.
- AC-PDLDPM3-06: `tests/installers/install_smoke.sh` owns the dev-installer parity assertions for this slice, and its `dev` scenario verifies that the shared `install_state.json` path keeps one Linux metadata meaning across the production and dev installers.

## Out of scope
- Adding a dry-run mode to `scripts/substrate/dev-install-substrate.sh`.
- Changing macOS or Windows behavior; this slice defines Linux parity only.
- Reassigning validation ownership away from `tests/installers/install_smoke.sh`.
