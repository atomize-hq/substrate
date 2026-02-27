**PRE‑PLANNING ONLY — This document is a temporary alignment draft and MUST be deleted/retired during full planning.**

# persist-detected-linux-distro-pkg-manager — minimal spec draft

## Scope + authority

Authority inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
- Spec manifest: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md`
- Impact map: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/impact_map.md`

This draft is allowed to define:
- Cross-cutting invariants, precedence rules, and “must-align” constraints that apply to all slices in this pack.

This draft MUST NOT define:
- Slice-specific behavior, task breakdowns, or implementation details.
- Full schemas or wire/file formats (those belong in the spec docs selected by `spec_manifest.md`).

## Defaults + precedence

Contract surfaces introduced/changed by this work:
- Persist Linux host detection + pkg-manager selection metadata into `$SUBSTRATE_HOME/install_state.json` (additive; `schema_version=1`).

Precedence posture:
- This work introduces no new CLI commands/flags, no new config keys, and no new env vars.
- `$SUBSTRATE_HOME` resolution/precedence is owned by the existing env contract; this pack uses it unchanged.

Source-of-truth file/path:
- `$SUBSTRATE_HOME/install_state.json` is the single persistence location for this feature.

## Failure posture + invariants

Failure posture (authoritative):
- Metadata persistence is best-effort.
- The installer MUST NOT fail solely due to:
  - inability to read/parse `/etc/os-release`, or
  - inability to create/update `$SUBSTRATE_HOME/install_state.json`.

Security + redaction invariants (high-level):
- Persist only a minimal allowlist of non-sensitive host platform metadata (no hostnames, no env dumps, no machine identifiers).
- Logging MUST NOT emit the full contents of `/etc/os-release`.
- Writes MUST remain scoped to `$SUBSTRATE_HOME` (no writes outside that directory).

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This work introduces no new exit code meanings and no taxonomy override.

## Cross-cutting seams / constraints

Platform scope:
- Linux-only behavior delta.
- macOS and Windows MUST NOT gain new persisted `host_state.platform.*` fields from this work.

Schema + key-path invariants:
- `schema_version` MUST remain `1` (additive, backwards-compatible).
- Persisted key paths under `install_state.json` are fixed to:
  - `host_state.platform.os_release.id`
  - `host_state.platform.os_release.id_like`
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source`
- Updates MUST be additive and MUST NOT discard unrelated JSON keys when writing `host_state.platform.*`.

Write timing + idempotency:
- The installer MUST write/create/update `$SUBSTRATE_HOME/install_state.json` at least once per successful Linux install.
- Updates MUST be idempotent across repeated installs.
- Persistence MUST NOT be gated on world enablement (`--no-world` does not skip metadata persistence on the success path).

Detection boundary:
- This pack persists detection outputs.
- This pack MUST NOT define or duplicate:
  - `/etc/os-release` parsing/canonicalization rules, or
  - pkg-manager selection precedence/tie-breakers.
- Full planning MUST treat the upstream detection contract as authoritative (dependency gate); this work persists those upstream outputs without re-deriving them.

Dependency posture:
- This work MUST NOT introduce a new required runtime dependency solely to satisfy persistence (e.g., making `python3` a hard prerequisite).

## Follow-ups for full planning

1) Decide and lock the finite enum set + mapping rules for `host_state.platform.pkg_manager.source` (DR-0003).
2) Decide and lock canonicalization rules for persisted `/etc/os-release` values (especially `ID_LIKE`) in the schema spec.
3) Define the merge/overwrite model for updating an existing `install_state.json`, including corrupted-file posture.
4) Resolve ADR directory drift (`stashing-ferret` vs `persist-detected-linux-distro-pkg-manager`) after planning outputs exist.
5) Decide and document posture for non-default install prefixes vs uninstall cleanup (installer write location vs uninstall read location).
6) Identify the authoritative upstream contract doc path for detection outputs (`/etc/os-release` parsing + pkg-manager selection) and reference it as a hard dependency gate in this pack’s `plan.md`.
