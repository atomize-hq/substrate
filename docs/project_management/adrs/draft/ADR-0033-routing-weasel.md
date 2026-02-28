# ADR-0033 — Provisioning-Time Pacman for World-Deps System Packages (Arch-family Worlds)

## Status

- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): ASSUMPTION: Shell maintainers; World backend maintainers

## Scope

- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/` (ASSUMPTION: new pack)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

- Intake: `docs/project_management/intake/adrs/routing_weasel_adr_intake.md`
- Internals (current behavior notes): `docs/internals/world/deps.md`
- World-deps contract / install classes: `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md`
- Provisioning-time system packages (APT baseline): `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
- Linux guest-rootfs roadmap context (system packages on Linux): `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
- Operator reference: `docs/reference/world/deps/README.md`
- Plan: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md`
- Tasks: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json`
- Spec manifest: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`
- Specs:
  - ASSUMPTION: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/specs/world_deps_pacman_provisioning.md`
- Contract (if present): `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- Decision Register (if required): `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
- Impact Map (if required): `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/impact_map.md`
- Manual Playbook (if required): `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: ca936883fd215e450faa879edf03ad5657980781b082a20c0cdff0397ca2c17c

### Changes (operator-facing)

- Add Arch-family (pacman) provisioning-time support for world-deps system packages
  - Existing: Substrate’s provisioning-time “system package” support is effectively APT-only; on Arch-family world OSes this can yield confusing “apt-like” expectations and incorrect remediation.
  - New: When provisioning is supported, `substrate world enable --provision-deps` can provision world-deps system packages via `pacman` for Arch-family world OSes. On unsupported worlds/backends, Substrate fails with explicit “world OS package manager unsupported / provisioning unsupported” guidance (and does not imply host OS mutation).
  - Why: Extend the provisioning-time workflow to additional guest OS families without weakening the hardened runtime contract or mutating host-native Linux workstations.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md#L1`
    - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md#L1`
    - `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md#L1`

## Problem / Context

- Some Substrate worlds (now or soon: Linux guest-rootfs; alternative guest images) may be Arch-family, where APT is not the correct OS package manager.
- `ADR-0030` locks the posture that OS/system package mutation is explicit and provisioning-time only (`substrate world enable --provision-deps`), not a side-effect of runtime `substrate world deps current sync|install`.
- Without non-APT provisioning support, the `--provision-deps` workflow is incomplete for non-Debian/Ubuntu world OSes, and failure modes tend to produce confusing/incorrect “apt-like” operator guidance.

## Goals

- Support provisioning-time installation/ensuring of world-deps system packages via `pacman` on Arch-family world OSes, when provisioning is supported by the active backend.
- Keep OS mutation explicit, operator-invoked, and provisioning-time only (no runtime `pacman` from `world deps current sync|install`).
- Make unsupported-world/backends fail with correct, actionable remediation that does not suggest the wrong package manager.

## Non-Goals

- Mutating Linux host-native workstation OS packages as a side effect of world-deps (still forbidden by default posture).
- Adding every package manager (dnf/yum/zypper/apk/brew) in this ADR.
- Adding AUR helpers (`yay`/`paru`/`pamac`) or any non-official-repo flows.
- Solving package-name translation across distros (mapping project).
- Introducing version pinning for `pacman` system packages in v1.

## Slice Decomposition

### C0 — World OS package-manager probe (provisioning-time)

During `--provision-deps`, execute a small in-world probe (not host PATH-based) to derive the world OS family and available OS package manager, sufficient to gate `pacman` provisioning for Arch-family worlds.

### C1 — Pacman provisioning path for world-deps system packages

Add a provisioning-time installer implementation that provisions/ensures `install.method=pacman` requirements inside Arch-family world OSes, and fails closed with actionable remediation when provisioning is unsupported or the world OS is not compatible.

### C2 — Validation + operator docs updates

Add tests/fixtures covering schema validation and pacman command construction, and update operator-facing docs/errors to avoid “apt-like” remediation on non-APT worlds.

### Option A — Add `install.method=pacman` to inventory schema (recommended)

Extend the world-deps package schema so system packages can be expressed as `install.method=pacman` with an explicit `install.pacman` list. Provisioning derives the requirement set from enabled items and runs `pacman` inside the world OS.

### Option B — Abstract `install.method=system_packages` with per-distro mapping

Introduce an abstract system-packages method and have Substrate translate package names per world OS/distro and then execute the matching manager.

### Recommendation (selection guidance)

- Choose Option A when you want an explicit, low-risk extension that avoids cross-distro package-name translation and keeps inventory authors in control of the manager-specific package names.
- Choose Option B when you are willing to own ongoing package-name mapping and validation across distros/managers (and can accept higher maintenance risk and more surprising failures).

## User Contract (Authoritative)

### CLI

- Provisioning (extended):
  - `substrate world enable --provision-deps [--dry-run] [--verbose]`
    - Derives required system packages from the effective enabled world-deps set.
    - Determines the world OS package manager via an in-world probe (see “Architecture Shape”).
    - When supported:
      - If the effective enabled set contains `install.method=pacman` items and the world OS is Arch-family, provisions via `pacman`.
      - If the effective enabled set contains `install.method=apt` items and the world OS is Debian/Ubuntu-family, provisions via APT (per `ADR-0030`).
    - `--dry-run` prints the derived requirement set(s) and intended actions; performs no mutation.
    - If the enabled set includes system-package items whose `install.method` does not match the detected world OS package manager, the command MUST fail with actionable remediation (e.g., choose a supported world image or override inventory appropriately).
- Runtime world-deps (system packages remain provisioning-time only):
  - `substrate world deps current sync`
  - `substrate world deps current install`
    - MUST NOT invoke OS package managers (APT or pacman).
    - If the effective enabled set contains `install.method=apt` or `install.method=pacman` items:
      - Exit non-zero with a friendly, actionable error explaining that system-package items are provisioning-time.
      - Remediation MUST include the exact command: `substrate world enable --provision-deps`.
      - On backends where provisioning is unsupported, remediation MUST provide manual guidance and explicitly state that the host OS will not be mutated by Substrate.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` (authoritative unless explicitly overridden here)
  - `0`: success
  - `2`: invalid inventory/config schema (including unsupported `install.method` / invalid `install.pacman` shape)
  - `3`: world backend unavailable / cannot connect to world-agent
  - `4`: unmet prerequisites or unsupported operation (includes: provisioning unsupported on this backend; enabled set contains system-package items requiring provisioning; detected world OS manager does not match required install methods)
  - `5`: safety / policy violation (reserved; runtime flow should avoid triggering this by failing early)

### Config

- Inventory schema (v1 extension):
  - `install.method` is extended to: `apt | pacman | script | manual`.
  - `install.pacman` is required iff `install.method=pacman` and is an ordered list of package names (version pinning is out of scope in v1).
- Effective enabled world-deps set (existing):
  - Provisioning derives required system packages from the same effective enabled view used by `substrate world deps current ...`.
  - This ADR introduces no new config keys and does not change config precedence rules.

### Platform guarantees

- Linux (host-native backend):
  - `substrate world enable --provision-deps` is unsupported by default (no host OS mutation).
  - Runtime `world deps current sync|install` fails early with explicit manual guidance for system-package items.
  - Future: if/when a Linux guest-rootfs backend exists, provisioning MAY be supported there (see related ADR-0009).
- macOS (Lima guest backend):
  - Provisioning-time OS mutation remains supported only when the active guest world permits it; pacman support applies only if/when the world OS is Arch-family.
- Windows (WSL backend):
  - ASSUMPTION: if/when `substrate world enable` is supported on Windows+WSL, `--provision-deps` provisions inside the WSL guest and never mutates the Windows host OS.

## Architecture Shape

- Components:
  - `crates/shell/src/builtins/world_deps/inventory.rs`: extend `InstallMethodV1` and the inventory schema to represent `pacman` system packages.
  - `crates/shell/src/builtins/world_enable/…`: extend provisioning-time flow to:
    - probe the world OS family/manager, and
    - execute the correct system-package provisioning path (APT per `ADR-0030`; pacman per this ADR) when supported.
  - `crates/world-agent/src/service.rs` (and/or execution plumbing): ensure provisioning execution is possible without weakening hardened runtime execution (distinct request profile or explicit guard rails).
  - Docs: update operator reference and error text (`docs/reference/world/deps/…`) to reflect manager-aware provisioning guidance.
- End-to-end flow:
  - Inputs:
    - world-deps inventory (built-ins + global + workspace chain)
    - effective enabled selection (global + workspace patches)
    - backend capability: provisioning supported vs unsupported
  - Derived state:
    - system package requirement sets:
      - pacman requirement set for enabled items with `install.method=pacman`
      - APT requirement set for enabled items with `install.method=apt`
    - world OS package-manager probe result (in-world):
      - `/etc/os-release` (`ID` / `ID_LIKE`) best-effort, and
      - manager presence check (e.g., `command -v pacman`)
  - Actions:
    - provisioning path: install/ensure required packages via the detected OS manager (or `--dry-run` print only)
    - runtime path: abort early with remediation (no OS manager execution)
  - Outputs:
    - clear operator stdout/stderr guidance (manager-aware)
    - stable exit codes per taxonomy

## Sequencing / Dependencies

- Sequencing entry: `docs/project_management/packs/sequencing.json` → ASSUMPTION: add a new sprint entry for `add-non-apt-system-package-provisioning-support`
- Prerequisites:
  - `ADR-0030` (`provisioning_otter`) must land first (or concurrently) to define the provisioning-time system-packages workflow surface and runtime fail-early posture.
  - Linux guest-rootfs provisioning support depends on `ADR-0009` (Arch-family support may initially land via other guest images/backends).

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->

```json
{
  "touch": {
    "create_files": null,
    "edit_files": 3,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 2,
    "boundary_crossings": null
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 1,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": null, "new_test_cases": null },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": true,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": null
  },
  "notes": "Discovery estimate; add pacman provisioning support (inventory method + provisioning-time implementation)."
}
```

<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture

- Fail-closed rules:
  - Runtime `world deps current sync|install` must never attempt OS package manager execution (APT or pacman).
  - Linux host-native must not mutate the host OS via provisioning.
  - Provisioning must not select a manager based on host PATH; selection is derived from in-world probe only.
- Protected paths/invariants:
  - OS mutation is operator-invoked and explicitly surfaced via `--provision-deps`.
  - Hardened runtime execution remains constrained to Substrate-managed writable surfaces (e.g., `/var/lib/substrate/world-deps`, `/tmp`).

## Validation Plan (Authoritative)

### Tests

- Unit tests:
  - Inventory schema: `install.method=pacman` requires `install.pacman` and rejects unsupported shapes/fields (shell crate).
  - Provisioning derivation: enabled set → pacman requirement list (shell crate).
  - Runtime short-circuit: system-package methods (`apt|pacman`) cause fail-early with exit `4` and correct remediation text.
- Integration tests:
  - Guest-world: verify provisioning path selects `pacman` on an Arch-family world OS probe result and issues the intended in-world commands (mocked/recorded per existing harness patterns).
  - Unsupported backend: verify provisioning is rejected with exit `4` and clear “provisioning unsupported / no host mutation” messaging.

### Manual validation

- Manual playbook: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
  - Must cover: Arch-family guest provisioning success, mismatch remediation, and runtime fail-early behavior.

### Smoke scripts

- Linux: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility

- Policy: greenfield breaking is allowed
- Compat work: none
- Behavior change note: inventories that do not use `install.method=pacman` are unaffected; inventories that do may now be provisioned on Arch-family world OSes via `substrate world enable --provision-deps`.

## Decision Summary

- Decision Register entries (if applicable):
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`:
    - DR-0001 (schema approach: explicit method vs abstract mapping)
    - DR-0002 (probe strategy: `/etc/os-release` vs manager presence vs both)
    - DR-0003 (pacman invocation and idempotency strategy)
    - DR-0004 (mismatch policy: fail vs partial provision)
- Options (required; at least two):
  - A) Add explicit `install.method=pacman` alongside `install.method=apt` in the world-deps inventory schema (recommended).
  - B) Introduce an abstract `install.method=system_packages` with per-distro mapping/translation (avoid in first ship).
- Selection:
  - Chosen: A
  - Rationale: Keeps the behavior delta small and explicit (no cross-distro package-name translation), while enabling provisioning-time system packages for Arch-family world OSes when supported.
  - Choose A when: we want explicit, readable inventory entries and are willing to add one manager at a time without solving translation.
  - Choose B when: we are ready to own long-term cross-distro package-name mapping and accept higher maintenance and failure risk.
