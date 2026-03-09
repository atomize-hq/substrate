# ADR-0032 — Persist Linux distro/pkg-manager detection in `install_state.json`

## Status
- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): TBD (ASSUMPTION: installer/host-provisioning maintainers)

## Scope
- Feature directory: `docs/project_management/packs/draft/stashing-ferret/` (to be created during planning)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)
- Intake: `docs/project_management/intake/adrs/stashing_ferret_adr_intake.md`
- Plan: `docs/project_management/packs/draft/stashing-ferret/plan.md`
- Tasks: `docs/project_management/packs/draft/stashing-ferret/tasks.json`
- Spec manifest: `docs/project_management/packs/draft/stashing-ferret/spec_manifest.md`
- Specs: (TBD; at minimum an installer/metadata spec)
- Decision Register (if required): `docs/project_management/packs/draft/stashing-ferret/decision_register.md`
- Impact Map (if required): `docs/project_management/packs/draft/stashing-ferret/impact_map.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 45d3c1ee693075301345bff0e7fffea5197a59a57b8941c04e7c9c458fd321ec

### Changes (operator-facing)
- Linux installer writes stable host-detection metadata to `$SUBSTRATE_HOME/install_state.json`
  - Existing: On Linux, installer may detect distro/pkg-manager but does not reliably persist that detection for later diagnostics; `install_state.json` may not exist after a successful install.
  - New: After any successful Linux install, `install_state.json` exists and includes `host_state.platform.*` metadata (`os_release.id`, `os_release.id_like`, `pkg_manager.selected`, `pkg_manager.source`).
  - Why: Enables later tools (doctor, retries, support) to provide correct guidance and reduces “wrong package manager” confusion without adding new provisioning behavior.
  - Links:
    - `scripts/substrate/install-substrate.sh`
    - `tests/installers/install_state_smoke.sh` (if present)
    - `docs/project_management/intake/adrs/stashing_ferret_adr_intake.md`

## Problem / Context
- Substrate’s Linux installer already performs best-effort distro/pkg-manager detection to install prerequisites, but that detection is not persisted in a stable, later-readable way.
- As a result, later diagnostics and guidance can be wrong or ambiguous (e.g., suggesting apt-style actions on a pacman-based host), and support has no authoritative “what did the installer detect?” record.
- `~/.substrate/install_state.json` already exists as the canonical installer metadata store (`schema_version=1`) used by uninstall flows, but it is currently written only when certain host-state “events” occur, meaning it can be absent on many successful installs.

## Goals
- Persist Linux distro identification (`/etc/os-release` `ID`, `ID_LIKE`) and the selected package manager into `$SUBSTRATE_HOME/install_state.json` in an additive, backwards-compatible way.
- Ensure `$SUBSTRATE_HOME/install_state.json` is created/updated at least once per successful Linux install, even when no group/linger events occurred.
- Define read semantics for future consumers: prefer persisted metadata for guidance strings but always fall back to runtime detection when missing/unreadable.

## Non-Goals
- Changing world-deps provisioning behavior or adding new package manager support (e.g., pacman/apk support for provisioning).
- Changing `install_state.json` `schema_version` (must remain `1` for uninstall compatibility).
- Using the persisted metadata to change doctor output or retry guidance in this ADR (follow-up work).
- Persisting sensitive host details (no hostnames, no environment dumps); only `/etc/os-release` fields and a selected manager string.
- Writing any platform metadata on macOS or Windows (Linux-only contract).

## Slice Decomposition
- C0 — Persist platform detection fields
  - Best-effort parse `/etc/os-release` and record `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like`, plus `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source`.
- C1 — Make `install_state.json` reliably present
  - Update installer write semantics so `install_state.json` is written on successful Linux installs even if no group/linger host-state events occurred; updates are idempotent.
- C2 — Add/extend installer smoke assertions
  - Update Linux installer smoke coverage to assert the new keys are present (and remain optional/ignored for older consumers).

## User Contract (Authoritative)

### CLI
- Commands:
  - No new CLI commands or flags are introduced by this ADR.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - No changes to existing installer exit code behavior are introduced by this ADR.

### Config
- Files and locations (precedence):
  1. `$SUBSTRATE_HOME/install_state.json` (ASSUMPTION: `$SUBSTRATE_HOME` defaults to `~/.substrate` on all platforms; on Linux this file is written/updated by the installer.)
- Schema:
  - `schema_version` remains `1`.
  - Additive fields are optional and must be safe to ignore by older uninstallers/consumers:
    - `host_state.platform.os_release.id`: string (from `/etc/os-release` `ID`)
    - `host_state.platform.os_release.id_like`: string (from `/etc/os-release` `ID_LIKE`, raw string)
    - `host_state.platform.pkg_manager.selected`: string (e.g., `apt-get`, `pacman`)
    - `host_state.platform.pkg_manager.source`: string enum (e.g., `flag|env|os_release|path_probe`)

### Platform guarantees
- Linux:
  - On successful install, `install_state.json` exists and includes the `host_state.platform.*` keys above when inputs are available.
  - If `/etc/os-release` is missing/unreadable, install still succeeds and `pkg_manager.*` is persisted with a source indicating the fallback mechanism (e.g., `path_probe`).
- macOS:
  - No change; this ADR does not require writing these fields.
- Windows:
  - No change; this ADR does not require writing these fields.

## Architecture Shape
- Components:
  - `scripts/substrate/install-substrate.sh`: detect distro/pkg-manager (best-effort) and persist metadata into `install_state.json` on Linux.
  - `tests/installers/install_state_smoke.sh` (or equivalent): assert installer emits/maintains the expected keys post-install.
- End-to-end flow:
  - Inputs: `/etc/os-release` (best-effort), installer-selected package manager, `$SUBSTRATE_HOME`.
  - Derived state: `distro_id`, `distro_like`, `pkg_manager.selected`, `pkg_manager.source`.
  - Actions: create/update `$SUBSTRATE_HOME/install_state.json` on successful Linux install; update platform metadata idempotently.
  - Outputs: `install_state.json` with stable `host_state.platform.*` fields available for later diagnostics/guidance.

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → (TBD during planning)
- Prerequisite integration task IDs:
  - Dependency (from intake): `detecting_badger` must land first if it defines/standardizes the installer’s detection logic used here.
- Coordination-only related intake: `provisioning_otter` (must not be blocked by this ADR; follow-up may consume the persisted metadata).

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "touch": {
    "create_files": 0,
    "edit_files": 1,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 0,
    "boundary_crossings": 0
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 1,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 0, "new_test_cases": 0 },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": false,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": true,
    "unknowns_high": 0
  },
  "notes": "Estimate: extend install_state.json (schema_version=1) with additive platform metadata and ensure file is written on successful installs."
}
```
<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture
- Fail-closed vs degrade:
  - Degrade: If platform metadata cannot be read or written, installer must not hard-fail solely due to metadata persistence (best-effort write; success path continues).
- Protected paths/invariants:
  - Only write under `$SUBSTRATE_HOME`; do not write outside that directory.
  - Do not log or persist sensitive host information; only `/etc/os-release` `ID`/`ID_LIKE` and the selected package manager string.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - If shared parsing helpers are introduced, add targeted unit tests close to the parsing/writing code (TBD by implementation shape).
- Integration tests:
  - Extend existing installer smoke coverage (e.g., `tests/installers/install_state_smoke.sh`) to assert:
    - `install_state.json` exists after a successful Linux install.
    - New keys are present when `/etc/os-release` is available.
    - Missing `/etc/os-release` does not cause install failure and still records `pkg_manager.*` with a fallback `source`.

### Manual validation
- Manual playbook: not required for this vertical slice if smoke coverage is sufficient; any manual checks must verify file creation and key presence only (no provisioning changes).

### Smoke scripts
- Linux: `tests/installers/install_state_smoke.sh` (or feature-local smoke scripts if planning chooses to relocate these checks)
- macOS: none (no behavior delta)
- Windows: none (no behavior delta)

## Rollout / Backwards Compatibility
- Policy: additive, backwards-compatible change to `install_state.json` with `schema_version=1` unchanged.
- Compat work: none beyond ensuring older uninstall flows ignore unknown keys.

## Decision Summary
- Decision Register entries (if applicable):
  - `docs/project_management/packs/draft/stashing-ferret/decision_register.md`:
    - DR-0001 (metadata persistence location)
    - DR-0002 (field naming + nesting)
    - DR-0003 (pkg_manager.source enum set)
- Options (required; at least two):
  - A) Extend `$SUBSTRATE_HOME/install_state.json` (`schema_version=1`) with additive `host_state.platform.*` metadata (recommended).
  - B) Write a separate `$SUBSTRATE_HOME/host_platform.json` file for host detection metadata.
- Selection:
  - Chosen: A
  - Rationale: Keeps one canonical, documented installer metadata store while remaining backwards compatible (`schema_version=1` unchanged) and avoids state fragmentation across multiple files.
  - Choose A when: we can keep `install_state.json` strictly additive and are willing to write it on successful installs even when no “event” records occurred.
  - Choose B when: we decide `install_state.json` must remain “event-only” for uninstall coupling and can accept an additional state file + documentation surface.
