# ADR-0033 — Provisioning-Time Pacman for World-Deps System Packages (Arch-family Worlds)

## Status
- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): ASSUMPTION: Shell maintainers; World backend maintainers

## Scope
- Feature directory: `docs/project_management/packs/active/world-deps-pacman-provisioning/` (ASSUMPTION: new pack)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md` (automation/worktree execution)

## Related Docs
- Internals (current behavior notes): `docs/internals/world/deps.md`
- World-deps contract / install classes: `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md`
- Provisioning-time system packages (APT baseline): `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
- Linux guest-rootfs roadmap context (system packages on Linux): `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
- Operator reference: `docs/reference/world/deps/README.md`
- Plan/specs/tasks/etc (to be created in the feature directory above):
  - Plan: `docs/project_management/packs/active/world-deps-pacman-provisioning/plan.md`
  - Tasks: `docs/project_management/packs/active/world-deps-pacman-provisioning/tasks.json`
  - Spec manifest: `docs/project_management/packs/active/world-deps-pacman-provisioning/spec_manifest.md`
  - Decision Register: `docs/project_management/packs/active/world-deps-pacman-provisioning/decision_register.md`
  - Impact Map: `docs/project_management/packs/active/world-deps-pacman-provisioning/impact_map.md`
  - Manual Playbook: `docs/project_management/packs/active/world-deps-pacman-provisioning/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=docs/project_management/adrs/draft/ADR-0033-routing-weasel.md` after drafting>

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
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md` (authoritative unless explicitly overridden here)
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
- Sequencing entry: `docs/project_management/packs/sequencing.json` → ASSUMPTION: add a new sprint entry for `world-deps-pacman-provisioning`
- Prerequisites:
  - `ADR-0030` (`provisioning_otter`) must land first (or concurrently) to define the provisioning-time system-packages workflow surface and runtime fail-early posture.
  - Linux guest-rootfs provisioning support depends on `ADR-0009` (Arch-family support may initially land via other guest images/backends).

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
- Manual playbook: `docs/project_management/packs/active/world-deps-pacman-provisioning/manual_testing_playbook.md`
  - Must cover: Arch-family guest provisioning success, mismatch remediation, and runtime fail-early behavior.

### Smoke scripts
- Linux: `docs/project_management/packs/active/world-deps-pacman-provisioning/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/active/world-deps-pacman-provisioning/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/active/world-deps-pacman-provisioning/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none
- Behavior change note: inventories that do not use `install.method=pacman` are unaffected; inventories that do may now be provisioned on Arch-family world OSes via `substrate world enable --provision-deps`.

## Decision Summary
- Options in the ADR (body-of-work shape):
  - Option A (recommended): explicit `install.method=pacman` schema extension (no package-name translation).
  - Option B (viable alternative): abstract `install.method=system_packages` with per-distro mapping.
  - This ADR documents the user-facing contract for Option A and the associated behavior delta.
- Decision Register vs ADR:
  - ADR should hold: the operator contract (`--provision-deps` behavior, exit codes, platform posture), the world OS probe contract, and the “no runtime OS mutation” invariant.
  - Decision Register should hold: fine-grained A/B decisions like “exact probe commands and parsing strategy”, “pacman invocation shape (refresh/idempotency)”, “behavior when enabled set contains mixed system-package methods”, and “whether built-in inventories ship pacman variants vs user-only initial support”.
- Decision Register entries:
  - `docs/project_management/packs/active/world-deps-pacman-provisioning/decision_register.md`:
    - DR-0001 (schema approach: explicit method vs abstract mapping)
    - DR-0002 (probe strategy: `/etc/os-release` vs manager presence vs both)
    - DR-0003 (pacman invocation and idempotency strategy)
    - DR-0004 (mismatch policy: fail vs partial provision)
