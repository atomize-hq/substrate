# persist-detected-linux-distro-pkg-manager — compatibility spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

This spec is authoritative for backward- and forward-compatibility rules when ADR-0032 extends:
- `<resolved SUBSTRATE_HOME>/install_state.json`

This spec governs:
- `schema_version` invariance
- additive merge rules for compatible files
- replace/reset rules for corrupt or incompatible files
- unknown-key tolerance for existing uninstall consumers

Out of scope:
- the exact field list and omission rules for `host_state.platform.*`: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- operator-facing write/read guarantees: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- package-manager detection semantics: `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

## Compatibility policy

- `schema_version` MUST remain `1`.
- ADR-0032 is additive-only.
- Existing compatible data MUST be preserved outside the feature-owned `host_state.platform` subtree.
- A separate ADR is required before any schema bump, field removal, field rename, or incompatible type change is allowed.

## File classification

The writer MUST classify the existing file, if present, into exactly one of these states before writing:

### State A: missing

- Condition:
  - `<resolved SUBSTRATE_HOME>/install_state.json` does not exist.
- Required action:
  - Create a new file with `schema_version: 1`.
  - Populate standard timestamps and the currently known host-state content.

### State B: compatible existing file

- Condition:
  - The file exists.
  - JSON parsing succeeds.
  - Top-level `schema_version` equals `1`.
- Required action:
  - Reuse the parsed document as the base object.
  - Preserve all existing keys except where the current install run is allowed to rewrite:
    - top-level `updated_at`
    - installer-owned `host_state.group` changes from the same run
    - installer-owned `host_state.linger` changes from the same run
    - full replacement of `host_state.platform`

### State C: corrupt file

- Condition:
  - The file exists.
  - JSON parsing fails for any reason.
- Required action:
  - Treat the file as unusable.
  - Emit a warning.
  - Discard the unreadable payload for merge purposes.
  - Replace the file with a fresh `schema_version: 1` document containing only data available during the current run.

### State D: wrong schema version

- Condition:
  - The file exists.
  - JSON parsing succeeds.
  - Top-level `schema_version` is absent or not equal to `1`.
- Required action:
  - Treat the file as incompatible.
  - Emit a warning.
  - Do not perform field-by-field migration from the incompatible payload.
  - Replace the file with a fresh `schema_version: 1` document containing only data available during the current run.

## Merge rules for compatible files

When the existing file is State B:
- The writer MUST preserve:
  - top-level `created_at`
  - any unknown top-level keys
  - `host_state.group` content, except fields the same install run legitimately updates
  - `host_state.linger` content, except entries the same install run legitimately updates
- The writer MUST update:
  - `updated_at`
  - `host_state.platform`
- The writer MUST NOT:
  - delete unknown keys from compatible files
  - rename existing keys
  - coerce preserved values into a new shape
  - drop `host_state.group` or `host_state.linger` merely because the current run did not emit group or linger events

## Replace rules for incompatible files

When the existing file is State C or State D:
- The writer MUST replace the full file content with a new compatible document.
- The writer MUST set `schema_version` to `1`.
- The writer MUST generate fresh `created_at` and `updated_at` values for the replacement document.
- The writer MUST write only data that is authoritative for the current run.
- The writer MUST NOT attempt partial salvage from unreadable or incompatible payloads.

## Unknown-key tolerance

- Consumers that only need `host_state.group` and `host_state.linger` MUST ignore `host_state.platform` without error.
- Older uninstall flows MUST continue reading only their known keys and MUST NOT fail solely because `host_state.platform` is present.
- Future consumers MUST ignore unknown keys added under compatible `schema_version=1` documents unless a later ADR states otherwise.

## Additive evolution rule

- Additional `host_state.platform.*` keys are allowed only when all of these hold:
  - `schema_version` remains `1`
  - existing keys keep their types and meanings
  - existing consumers can ignore the new keys safely
  - the change is documented in this feature pack or a superseding ADR
- Renaming or deleting the four ADR-0032 platform keys is forbidden under this compatibility policy.

## End condition

- This compatibility policy remains authoritative until a later ADR explicitly changes the `install_state.json` schema policy.
