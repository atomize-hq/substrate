# PDLDPM1-spec — Guarantee production-installer install-state writes

## Behavior delta (single)
- Existing: `scripts/substrate/install-substrate.sh` writes `install_state.json` only when host-state event payload exists and skips metadata persistence entirely when `--no-world` is used.
- New: Successful non-dry-run Linux runs of `scripts/substrate/install-substrate.sh` always create or update `<resolved SUBSTRATE_HOME>/install_state.json`, even when no group/linger events occurred and even when `--no-world` was selected, while dry-run remains non-mutating and metadata failures remain warning-only.
- Why: ADR-0032 requires a reliable Linux install-state artifact after successful installs instead of an event-only write path.

## Scope
- Define the production-installer write trigger for successful Linux installs.
- Define creation, merge, and reset behavior for the existing `schema_version=1` install-state file.
- Define the selected `--no-world` and `--dry-run` rules for the production installer.
- Define the fail-open warning posture when metadata read, merge, parse, or write steps fail.

## Inputs (authoritative)
- Feature contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Compatibility rules: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md`
- Schema rules: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Decision register: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
- Linux validation harness: `tests/installers/install_state_smoke.sh`

## Behavior (authoritative)
### Successful Linux write trigger
- `scripts/substrate/install-substrate.sh` MUST create or update `<resolved SUBSTRATE_HOME>/install_state.json` after every successful non-dry-run Linux install.
- The write trigger MUST run even when no group-creation, group-membership, or linger event payload was recorded during the install.
- The write trigger MUST run when the install used `--no-world`.

### Document creation and merge
- If `<resolved SUBSTRATE_HOME>/install_state.json` is missing, the installer MUST create a fresh `schema_version=1` document with:
  - `created_at`
  - `updated_at`
  - baseline `host_state.group`
  - baseline `host_state.linger`
  - the current `host_state.platform` payload
- If the file is readable JSON with `schema_version=1`, the installer MUST:
  - preserve `created_at`
  - refresh `updated_at`
  - preserve readable content outside `host_state.platform`
  - preserve readable `host_state.group`
  - preserve readable `host_state.linger`
  - replace `host_state.platform` with the current run's payload
- If the file is corrupt JSON or readable JSON with `schema_version != 1`, the installer MUST emit a warning, discard the prior contents for merge purposes, and write a fresh `schema_version=1` document.

### Dry-run and failure posture
- `scripts/substrate/install-substrate.sh --dry-run` MUST NOT create, replace, or update `<resolved SUBSTRATE_HOME>/install_state.json`.
- During dry-run, the installer MUST log that install-state persistence was skipped because the invocation was non-mutating.
- If metadata read, merge, parse, or write work fails during an otherwise successful Linux install, the installer MUST emit a warning and continue the successful install path without introducing a new non-zero exit.
- Metadata persistence for this slice MUST write only under `<resolved SUBSTRATE_HOME>`.

## Acceptance criteria
- AC-PDLDPM1-01: A successful non-dry-run Linux invocation of `scripts/substrate/install-substrate.sh` creates `<resolved SUBSTRATE_HOME>/install_state.json` even when no group or linger events were recorded during the install.
- AC-PDLDPM1-02: A successful non-dry-run Linux invocation of `scripts/substrate/install-substrate.sh --no-world` still creates or updates `<resolved SUBSTRATE_HOME>/install_state.json` with the current `host_state.platform` payload.
- AC-PDLDPM1-03: `scripts/substrate/install-substrate.sh --dry-run` leaves `<resolved SUBSTRATE_HOME>/install_state.json` absent or unchanged and logs that install-state persistence was skipped because the run was non-mutating.
- AC-PDLDPM1-04: When the existing install-state file is readable JSON with `schema_version=1`, the installer preserves `created_at`, preserves readable `host_state.group` and `host_state.linger`, refreshes `updated_at`, and replaces only `host_state.platform`.
- AC-PDLDPM1-05: When the existing install-state file is corrupt JSON or readable JSON with `schema_version != 1`, the installer emits a warning, writes a fresh `schema_version=1` document, and does not convert the successful install into a new failure exit.
- AC-PDLDPM1-06: If metadata persistence cannot complete after the install work otherwise succeeds, the installer emits a warning and exits with the same success status it would have returned without the metadata failure.
- AC-PDLDPM1-07: All writes introduced by this slice target only `<resolved SUBSTRATE_HOME>/install_state.json`; the production installer does not create a second metadata file for the platform payload.

## Out of scope
- Changing the stored `host_state.platform` key set or its value semantics.
- Extending the same contract to `scripts/substrate/dev-install-substrate.sh`; that parity work is owned by `PDLDPM3`.
- Defining downstream guidance-reader behavior beyond the write guarantees required for the production installer.
