# ADR-0032 — Persist Linux distro/pkg-manager detection in `install_state.json`

## Status
- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): TBD (ASSUMPTION: installer/host-provisioning maintainers)

## Scope
- Feature directory: `docs/project_management/packs/active/stashing-ferret/` (to be created during planning)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Intake: `docs/project_management/intake/adrs/stashing_ferret_adr_intake.md`
- Plan: `docs/project_management/packs/active/stashing-ferret/plan.md`
- Tasks: `docs/project_management/packs/active/stashing-ferret/tasks.json`
- Spec manifest: `docs/project_management/packs/active/stashing-ferret/spec_manifest.md`
- Specs: (TBD; at minimum an installer/metadata spec)
- Decision Register: `docs/project_management/packs/active/stashing-ferret/decision_register.md`
- Impact Map: `docs/project_management/packs/active/stashing-ferret/impact_map.md` (if required by planning)

## Executive Summary (Operator)

ADR_BODY_SHA256: fd7159c30e9a134154397264008dfa2d8f36ef9f4f246f7563e17aeb4685616c

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
- This ADR defines the end-to-end contract for persisting Linux platform detection metadata and making `install_state.json` reliably present after install.
- Fine-grained A/B decisions (pack-local; must live in the Decision Register and not be duplicated here):
  - Where to persist metadata (extend `install_state.json` vs separate file vs config).
  - Exact field naming and nesting (confirm/lock `host_state.platform.*`).
  - Source attribution enumeration values for `pkg_manager.source`.
- Decision Register entries:
  - `docs/project_management/packs/active/stashing-ferret/decision_register.md`:
    - DR-0001 (metadata persistence location)
    - DR-0002 (field naming + nesting)
    - DR-0003 (pkg_manager.source enum set)

## Options Considered (Summary)

### Option A — Extend `install_state.json` (`schema_version=1`) with platform metadata (recommended)
- Add optional keys under `host_state.platform.*` and ensure the file is written on successful Linux installs even without group/linger events.

### Option B — Write a separate `$SUBSTRATE_HOME/host_platform.json`
- Keep `install_state.json` strictly for uninstall-coupled host-state events; store detection output in a dedicated file.

### Option C — Store detection in `$SUBSTRATE_HOME/config.yaml` (avoid)
- Persist detected host facts as config values (conflates detected facts with user intent).

### Recommendation guardrails
- Choose Option A when you want a single canonical, documented metadata store and can keep the change strictly additive (`schema_version=1` unchanged).
- Choose Option B when you must keep `install_state.json` “event-only” and are willing to accept an additional state file and documentation surface.
