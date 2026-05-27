# Install State Schema

This document is the durable contract reference for the stable `install_state.json` schema used by
installer metadata.

Related references:
- `docs/INSTALLATION.md`
- `docs/adr/implemented/ADR-0031-best-effort-linux-distro-package-manager-discovery-during-install.md`
- `docs/adr/implemented/ADR-0032-persist-linux-distro-package-manager-detection-in-install-state.md`

## Scope

This contract is authoritative for the additive installer metadata schema used at:

- `<effective_prefix>/install_state.json`
- `$SUBSTRATE_HOME/install_state.json` when the default user-scoped prefix is active

The stable top-level schema version remains:

- `schema_version = 1`

## Compatibility Policy

- additive-only
- existing fields must not be renamed or removed
- consumers must ignore unknown fields
- writers must preserve existing unknown fields unless another stable contract explicitly owns them
- this schema must not bump `schema_version`

## Top-Level Structure

- `schema_version`
  - type: integer
  - required: yes
  - allowed value: `1`
- `host_state`
  - type: object
  - required: yes when any host metadata is persisted

## Preserved Legacy Fields

This contract does not redefine the JSON shape of:

- `host_state.group`
- `host_state.linger`

If either field exists before a rewrite, writers must preserve the exact JSON value.

## Linux Platform Metadata

For Linux installer flows that persist package-manager detection metadata:

- `host_state.platform`
  - required: yes
  - must contain both `os_release` and `pkg_manager`
- `host_state.platform.os_release.id`
  - type: string
  - required: yes
- `host_state.platform.os_release.id_like`
  - type: string
  - required: yes
- `host_state.platform.pkg_manager.selected`
  - type: string
  - required: yes
- `host_state.platform.pkg_manager.source`
  - type: string
  - required: yes

Semantics:

- `os_release.id` and `os_release.id_like` must copy the normalized distro detection output
  verbatim
- `<unknown>` is persisted literally when detection produced `<unknown>`
- `pkg_manager.selected` and `pkg_manager.source` must copy the upstream detection result
  verbatim
- the persistence writer must not re-derive or locally normalize these values

## Merge and Write Rules

- writers must create `install_state.json` on successful Linux installs even when no separate
  group/linger event occurred
- writers must read the existing file before rewriting it
- writers must preserve existing `host_state.group` and `host_state.linger` values exactly
- writers must preserve existing unknown fields
- writers must replace the file atomically so each successful write leaves a complete JSON document
- writers must not remove `host_state.group` or `host_state.linger` solely because the current run
  did not emit new values for those fields

## Invalid States

This contract treats the following as invalid schema states:

- `schema_version != 1`
- `host_state.platform` present without `host_state.platform.os_release`
- `host_state.platform` present without `host_state.platform.pkg_manager`
- `host_state.platform.os_release.id` omitted when `host_state.platform.os_release` is present
- `host_state.platform.os_release.id_like` omitted when `host_state.platform.os_release` is
  present
