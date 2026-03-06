# PDLDPM0-spec — Persist additive Linux platform metadata

## Behavior delta (single)
- Existing: installer metadata persistence is centered on `host_state.group` / `host_state.linger`, and the final Linux distro/package-manager detection outputs are not stored in `install_state.json`.
- New: successful non-dry-run Linux installs persist an additive `host_state.platform` subtree in `install_state.json`, reusing the existing detection outputs for `os_release.*` and `pkg_manager.*` without redefining the detection algorithm or creating a second metadata file.
- Why: later consumers and operators need one stable persisted record of the installer's final Linux platform detection results, but the stored payload must stay minimal and compatibility-safe.

## Scope
- Define the implementation seam that serializes Linux `host_state.platform` data under `<resolved SUBSTRATE_HOME>/install_state.json`.
- Reuse the dependency-owned detection outputs for:
  - `host_state.platform.os_release.id`
  - `host_state.platform.os_release.id_like`
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source`
- Apply omission rules so unavailable values are absent from JSON instead of serialized as sentinels or `null`.
- Replace the full `host_state.platform` subtree for the current run rather than merging stale child keys forward.

## Behavior (authoritative)
### Linux platform payload
- The writer MUST persist only the four feature-owned keys listed in `install-state-schema-spec.md`.
- `pkg_manager.selected` and `pkg_manager.source` MUST be copied from the final dependency-owned detection result without renaming values, inventing aliases, or restating fallback reasoning in the stored strings.
- `os_release.id` and `os_release.id_like` MUST reuse the dependency-owned normalized `/etc/os-release` outputs and MUST remain omitted when those inputs were unavailable or normalized to the dependency-owned unknown sentinel.

### Omission and parent-object rules
- Unavailable feature-owned values MUST be omitted key-by-key.
- `host_state.platform.os_release` MUST be omitted when both `os_release` child fields are absent.
- `host_state.platform.pkg_manager` MUST be omitted when both `pkg_manager` child fields are absent.
- On Linux, `host_state.platform` MUST be omitted only when all four feature-owned values are unavailable for the current write.
- The writer MUST NOT serialize unavailable values as `null`, empty strings, `<unknown>`, `unknown`, or any other sentinel.

### Merge boundary and platform guard
- On a compatible existing file, the writer MUST preserve content outside the feature-owned `host_state.platform` subtree exactly as required by `compatibility-spec.md`.
- The writer MUST replace the full `host_state.platform` subtree atomically for the current run so stale child keys from earlier installs do not survive.
- macOS and Windows installs MUST NOT persist `host_state.platform`.
- This slice MUST NOT introduce a new metadata file, a second detection pass, or feature-local package-manager vocabulary.

## Acceptance criteria
- AC-PDLDPM0-01: On a successful non-dry-run Linux install where the dependency-owned detector returns `os_release.id=ubuntu`, `os_release.id_like=debian`, `pkg_manager.selected=apt-get`, and `pkg_manager.source=os_release`, `install_state.json` contains exactly those four values under `host_state.platform` and does not add any other `host_state.platform.*` keys.
- AC-PDLDPM0-02: On a successful non-dry-run Linux install where `/etc/os-release` inputs are unavailable but the detector still returns `pkg_manager.selected` and `pkg_manager.source`, `install_state.json` contains `host_state.platform.pkg_manager.*`, omits `host_state.platform.os_release`, and contains no `null`, empty-string, or sentinel substitutes for the missing `os_release.*` fields.
- AC-PDLDPM0-03: When a compatible existing `install_state.json` already contains `host_state.platform.os_release.id_like` from an earlier run and the current run produces only `pkg_manager.*`, the rewritten file preserves unrelated keys plus `host_state.group` / `host_state.linger`, and the stale `host_state.platform.os_release.*` keys are absent after the write.
- AC-PDLDPM0-04: On macOS and Windows installs, no new `host_state.platform` subtree is written, and the feature does not create any alternate platform-metadata file outside `<resolved SUBSTRATE_HOME>/install_state.json`.

## Out of scope
- Changing the Linux package-manager detection algorithm, manager-name vocabulary, `pkg_manager.source` enum meanings, or `/etc/os-release` normalization rules owned by `best-effort-distro-package-manager`.
- Defining downstream reader behavior beyond the persisted payload and omission rules.
- Expanding smoke coverage or task wiring for this slice.
