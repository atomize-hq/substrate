# PDLDPM0-spec — Persist Linux platform metadata into `install_state.json`

## Behavior delta (single)
- Existing: Linux installer metadata writes preserve `schema_version`, `host_state.group`, and `host_state.linger`, but they do not define one authoritative persisted `host_state.platform.*` payload for distro identity and package-manager selection metadata.
- New: Whenever a Linux installer metadata write occurs, the written document includes one additive `host_state.platform` block containing normalized distro identifiers plus the selected package manager and selection source copied from the upstream detection contract.
- Why: One persisted platform-metadata payload lets future guidance consumers read stable Linux host metadata from `install_state.json` without creating a second vocabulary authority or damaging existing install-state data.

## Scope
- Define the exact persisted payload under `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*`.
- Define the producer inputs this slice consumes:
  - normalized distro identifiers produced by the upstream os-release detection path,
  - selected package-manager and source strings produced by `best-effort-distro-package-manager`.
- Define the merge rule that preserves existing `host_state.group`, `host_state.linger`, and unknown sibling keys during upgrade writes.
- Apply one payload contract to both `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` whenever either script performs a Linux install-state write.
- Keep write-trigger selection, atomic temp-file replacement, dry-run no-write behavior, and warning-only failure posture owned by `contract.md` and `PDLDPM1`.

## Inputs (authoritative)
- Operator-facing persistence contract and authority boundary: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Exact field paths, schema invariants, and merge rules: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Accepted JSON nesting and verbatim-copy ownership decisions:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` (`DR-0002`, `DR-0003`)
- Upstream source of truth for normalized distro detection, selected manager vocabulary, and `pkg_manager.source` vocabulary:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

## Behavior (authoritative)

### Slice boundary and authority
- This slice owns only the persisted Linux platform-metadata payload inside `install_state.json`.
- This slice does not redefine the canonical file path, the installer write matrix, the temp-file replacement flow, or installer exit-code behavior.
- Those surfaces remain owned by `contract.md`, `install-state-schema-spec.md`, and `slices/PDLDPM1/PDLDPM1-spec.md`.

### Consumed producer inputs
- The persistence layer MUST consume four already-resolved detection outputs:
  - `distro_id` → `host_state.platform.os_release.id`
  - `distro_id_like` → `host_state.platform.os_release.id_like`
  - selected package manager → `host_state.platform.pkg_manager.selected`
  - selected package-manager source → `host_state.platform.pkg_manager.source`
- For the os-release fields, the persisted string MUST match the normalized detector output exactly, including the literal `<unknown>` sentinel when the upstream detector emitted `<unknown>` because the selected os-release file was missing or unreadable.
- For the package-manager fields, the persisted string MUST match the upstream detection output exactly. The persistence layer MUST NOT rerun package-manager selection, remap distro families, or translate the `source` value before writing.
- The persistence layer MUST NOT reread `/etc/os-release` or `SUBSTRATE_INSTALL_OS_RELEASE_PATH` to recompute persisted values after the upstream detection contract has already produced its outputs.

### JSON shape and merge rules
- On each Linux install-state write within this slice's scope, the resulting document MUST contain:
  - `host_state.platform.os_release.id`
  - `host_state.platform.os_release.id_like`
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source`
- The writer MUST create `host_state` and `host_state.platform` objects when they are absent.
- The writer MUST preserve existing JSON values at `host_state.group` and `host_state.linger` exactly as read.
- The writer MUST preserve unknown top-level keys and unknown `host_state` sibling keys unless another authoritative spec for those keys says otherwise.
- No flat aliases, duplicate keys, or alternate nesting for the four platform fields are allowed.

### Missing os-release degradation within persisted payload
- If upstream detection produced `distro_id="<unknown>"` and `distro_id_like="<unknown>"` because the selected os-release file was missing or unreadable, the writer MUST still persist those literal `<unknown>` values and MUST still persist `pkg_manager.selected` and `pkg_manager.source` when those values resolved successfully.
- This slice defines no macOS or Windows `host_state.platform.*` write behavior.

## Acceptance criteria
- AC-PDLDPM0-01: Given a Linux metadata write with resolved inputs `distro_id=ubuntu`, `distro_id_like=debian`, `selected=apt-get`, and `source=os_release`, the resulting JSON contains exactly `host_state.platform.os_release.id="ubuntu"`, `host_state.platform.os_release.id_like="debian"`, `host_state.platform.pkg_manager.selected="apt-get"`, and `host_state.platform.pkg_manager.source="os_release"`, with no flat aliases or alternate nesting.
- AC-PDLDPM0-02: Verbatim copy-through: if the upstream detection contract emitted `pkg_manager.selected="yum"` and `pkg_manager.source="env"` for a Linux write, the persisted fields use exactly `yum` and `env` even when `/etc/os-release` content or `PATH` contents would also support another manager.
- AC-PDLDPM0-03: Missing os-release degradation: when upstream detection emitted `distro_id="<unknown>"`, `distro_id_like="<unknown>"`, `pkg_manager.selected="apt-get"`, and `pkg_manager.source="path_probe"`, the resulting JSON persists all four values and does not omit the package-manager block.
- AC-PDLDPM0-04: Upgrade-write preservation: starting from an `install_state.json` that already contains `host_state.group` and `host_state.linger`, a Linux platform-metadata write adds or updates only the `host_state.platform.*` block and preserves the preexisting `group` and `linger` JSON values exactly as values.
- AC-PDLDPM0-05: Additive compatibility: starting from an `install_state.json` that contains unknown top-level keys or unknown `host_state` siblings, a Linux platform-metadata write preserves those unknown keys while adding or updating the accepted `host_state.platform.*` fields.
- AC-PDLDPM0-06: Producer parity: whenever `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` each perform a Linux install-state write, both producers persist the same four field paths and the same verbatim-copy rules defined by this slice.

## Out of scope
- Exact hosted/dev write-trigger branches, including hosted `--dry-run` and `--no-world`, remain owned by `slices/PDLDPM1/PDLDPM1-spec.md`.
- Temp-file path, replace-step, and failed-write cleanup semantics remain owned by `contract.md` and `slices/PDLDPM1/PDLDPM1-spec.md`.
- Smoke-harness assertions and operator-doc reconciliation remain owned by `slices/PDLDPM2/PDLDPM2-spec.md`.
- Package-manager selection precedence, override validation, os-release parsing normalization, and supported vocabulary ownership remain owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
