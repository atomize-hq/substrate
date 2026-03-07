# PDLDPM0-spec — Persist additive Linux platform metadata

## Behavior delta (single)
- Existing: `install_state.json` preserves install-time `host_state.group` and `host_state.linger` data, but it does not persist the Linux distro/package-manager metadata selected during installer execution.
- New: On successful non-dry-run Linux installs, the installer metadata writers add `host_state.platform` to the existing `schema_version=1` document and persist only `os_release.id`, `os_release.id_like`, `pkg_manager.selected`, and `pkg_manager.source`, while leaving detection semantics external to `best-effort-distro-package-manager`.
- Why: ADR-0032 needs one additive storage seam for later guidance consumers without creating a second metadata file or redefining package-manager detection.

## Scope
- Define the exact `host_state.platform` JSON shape written into `<resolved SUBSTRATE_HOME>/install_state.json`.
- Capture the final installer-selected package-manager values without redefining the detection algorithm or value vocabulary.
- Persist normalized `/etc/os-release` values only when the external detection contract makes them available.
- Keep the stored payload limited to the additive platform subtree selected by the pack contract.

## Inputs (authoritative)
- Feature contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Schema contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Compatibility rules: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md`
- External detection authority: `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- External os-release normalization authority: `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

## Behavior (authoritative)
### Stored subtree
- This slice writes exactly one additive subtree under `host_state`:
  - `platform`
- `host_state.platform` MUST contain:
  - `pkg_manager.selected`
  - `pkg_manager.source`
- `host_state.platform.os_release` MUST be present only when at least one persisted `os_release.*` field is available.

### Value sources
- `host_state.platform.pkg_manager.selected` MUST exactly equal the final selected manager from the external detection contract.
- `host_state.platform.pkg_manager.source` MUST exactly equal the final selection source from the external detection contract.
- `host_state.platform.os_release.id` MUST use the normalized `ID` value from the external detection contract when available.
- `host_state.platform.os_release.id_like` MUST use the normalized `ID_LIKE` value from the external detection contract when available.

### Omission and serialization rules
- If normalized `ID` is unavailable, `host_state.platform.os_release.id` MUST be omitted.
- If normalized `ID_LIKE` is unavailable, `host_state.platform.os_release.id_like` MUST be omitted.
- If both normalized `ID` and `ID_LIKE` are unavailable, `host_state.platform.os_release` MUST be omitted.
- The writer MUST NOT persist placeholder or sentinel values for unavailable os-release inputs, including `<unknown>` and `null`.
- The writer MUST NOT persist any platform field outside `host_state.platform`.

### Platform guard
- This slice changes Linux install-state content only.
- macOS and Windows MUST NOT gain a new `host_state.platform.*` write contract from this slice.

## Acceptance criteria
- AC-PDLDPM0-01: A successful non-dry-run Linux install that resolves normalized `ID=ubuntu`, `ID_LIKE=debian`, `selected=apt-get`, and `source=os_release` writes `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source` under `<resolved SUBSTRATE_HOME>/install_state.json`.
- AC-PDLDPM0-02: When the normalized Linux install inputs contain `ID=arch` and no normalized `ID_LIKE`, the written JSON contains `host_state.platform.os_release.id="arch"` and omits `host_state.platform.os_release.id_like`.
- AC-PDLDPM0-03: When normalized `/etc/os-release` inputs are unavailable but the installer still selects a package manager successfully, the written JSON omits `host_state.platform.os_release` and still writes `host_state.platform.pkg_manager.selected` plus `host_state.platform.pkg_manager.source`.
- AC-PDLDPM0-04: The persisted `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` values exactly match the external detection contract output for the same invocation; this slice does not rename, remap, or re-enumerate either value.
- AC-PDLDPM0-05: The written JSON never stores sentinel values for unavailable os-release inputs; `<unknown>`, `null`, empty `os_release` objects, and placeholder strings are absent from the persisted platform subtree.
- AC-PDLDPM0-06: The slice introduces no persisted platform keys outside the exact set `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`.

## Out of scope
- Changing the package-manager detection precedence, supported manager names, or `pkg_manager.source` semantics.
- Defining the successful-install write trigger, dry-run behavior, or fail-open warning posture for the production installer write path.
- Defining the dev-installer validation split; that parity work is owned by `PDLDPM3`.
