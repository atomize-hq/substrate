# PDLDPM0-spec — Persist Linux platform metadata into install state

## Behavior delta (single)
- Existing: `install_state.json` preserves installer-owned metadata such as `host_state.group` and `host_state.linger`, but successful Linux installs do not persist distro identity or selected package-manager metadata under a stable `host_state.platform.*` boundary.
- New: when the installer executes the Linux metadata-persistence step, it writes only `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source` into the existing `schema_version = 1` document, preserving unrelated install-state content.
- Why: future consumers need one canonical persisted source for Linux distro and package-manager metadata without duplicating detector vocabulary or breaking existing install-state compatibility.

## Scope
- Define the payload written by the Linux metadata-persistence step for `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh`.
- Consume four current-run inputs only:
  - normalized os-release `ID`
  - normalized os-release `ID_LIKE`
  - selected package manager
  - selected package-manager source
- Constrain persistence to the four leaf paths owned by `install-state-schema-spec.md` and preserve pre-existing install-state data outside those paths.
- Reuse the upstream detector contract as the only authority for selected-manager spelling, `pkg_manager.source` values, and os-release parsing semantics.

## Inputs (authoritative)
- Installer-facing persistence contract, canonical path rule, Linux-only boundary, and write-matrix ownership:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Exact JSON field paths, absence semantics, and merge/preservation rules:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Upstream detector authority for selected-manager vocabulary, `pkg_manager.source` vocabulary, and normalized os-release inputs:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

## Behavior (authoritative)

### Persistence inputs
- This slice consumes the current-run Linux detection outputs that already exist before persistence runs:
  - normalized distro `ID`
  - normalized distro `ID_LIKE`
  - selected package manager
  - selected package-manager source
- The selected package manager and package-manager source MUST be the exact emitted values from `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
- This slice MUST NOT introduce a second detector, a second manager-selection pass, or a local vocabulary table for manager/source strings.

### Persisted field boundary
- The Linux metadata-persistence step MUST write platform metadata only at these leaf paths:
  - `host_state.platform.os_release.id`
  - `host_state.platform.os_release.id_like`
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source`
- `host_state.platform.os_release.id` stores the current-run normalized distro `ID` string when available.
- `host_state.platform.os_release.id_like` stores the current-run normalized distro `ID_LIKE` string when available.
- `host_state.platform.pkg_manager.selected` stores the exact selected-manager string emitted by the upstream detector.
- `host_state.platform.pkg_manager.source` stores the exact source string emitted by the upstream detector.
- Missing current-run inputs are represented by absence of the corresponding owned leaf path. This slice MUST NOT persist placeholder strings, empty arrays, or `null` for any of the four owned leaf fields.

### Merge and preservation rules
- The post-write document remains `schema_version = 1`.
- The platform-merge step MUST preserve every existing key outside the four owned leaf paths.
- In particular, the platform-merge step MUST preserve pre-existing `host_state.group` and `host_state.linger` content unchanged, except for any changes already owned by the pre-existing installer metadata flow outside this slice.
- The platform-merge step MUST create only the container objects needed for owned leaf fields that are present after merge and MUST remove owned empty containers when no owned leaf field remains under that container.
- Unknown sibling keys under `host_state`, `host_state.platform`, `host_state.platform.os_release`, and `host_state.platform.pkg_manager` MUST be preserved.

### Missing os-release boundary
- If the current run cannot provide `ID` or `ID_LIKE`, the post-write document MUST leave `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` absent.
- Missing os-release data MUST NOT block persistence of `host_state.platform.pkg_manager.selected` or `host_state.platform.pkg_manager.source` when those package-manager inputs remain available for the same run.

### Contract-link boundary
- Canonical path selection, Linux write-trigger branches, `--dry-run` no-write behavior, warning-only failure posture, and temp-file replacement are owned by `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` and later traced by `PDLDPM1`.
- This slice defines only the persisted payload and merge boundary once the persistence step runs.

## Acceptance criteria
- AC-PDLDPM0-01: When a successful Linux persistence step receives current-run values `ID=ubuntu`, `ID_LIKE=debian`, `selected=apt-get`, and `source=os_release`, the resulting `install_state.json` records those values only at `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`.
- AC-PDLDPM0-02: `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` are copied verbatim from the upstream detector contract output, with no local remapping, aliasing, or second-pass derivation of manager/source strings.
- AC-PDLDPM0-03: If `/etc/os-release` data for the current run is missing or unreadable but package-manager selection still succeeds, the post-write document omits `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` while still persisting `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source`.
- AC-PDLDPM0-04: When the pre-write document already contains `host_state.group` and `host_state.linger`, the platform-merge step leaves those values intact while adding or updating only the owned `host_state.platform.*` paths.
- AC-PDLDPM0-05: When one or more owned current-run inputs are unavailable, the post-write document represents that gap by omitting only the corresponding owned leaf paths and does not store placeholder strings, empty arrays, or `null` for any owned field.
- AC-PDLDPM0-06: The post-write document remains `schema_version = 1`, preserves unknown sibling keys outside the four owned leaf paths, and introduces no second metadata file for Linux distro or package-manager state.

## Out of scope
- Deciding which installer branches perform persistence, including hosted install, hosted `--no-world`, dev install, dev `--no-world`, and `--dry-run`.
- Defining temp-file replacement, idempotent rewrite mechanics, or warning-only degradation when read/write steps fail.
- Defining smoke-harness assertions, operator-doc wording, or downstream consumer validation evidence.
