---
codename: routing_weasel
created: "2026-02-20T20:11:06Z"
status: ready_for_lockdown
depends_on:
  - provisioning_otter
execution_order: 40
adr: ADR-0033
adr_path: docs/project_management/adrs/draft/ADR-0033-routing-weasel.md
workstream_id: WS-routing_weasel
lockdown_prompt: docs/project_management/system/prompts/discovery/adr_lockdown.md
---

# ADR Intake Sheet

## 1. Codename + date + status

- Codename: `routing_weasel`
- Created: 2026-02-20T20:11:06Z
- Status: ready_for_lockdown
- ADR draft: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`

## 2. Working Title (tentative)

Add non-apt system package provisioning support (e.g. pacman) for world-deps system packages

## 3. Problem / Motivation (3–6 bullets)

- On Arch-based hosts (e.g. Manjaro), some built-in world-deps items (or operator expectations) can lead to “apt-like” behavior being attempted or suggested, which is confusing and often wrong.
- `provisioning_otter` locks the posture that system package installs must be explicit and happen at provisioning-time via `substrate world enable --provision-deps`, not during runtime `world deps current sync|install`.
- However, `--provision-deps` can only be meaningfully useful on worlds whose OS package manager is supported by Substrate for system provisioning.
- Today, “system package provisioning” is effectively apt-only on guest worlds (and Linux host-native is posture-blocked from mutating the workstation OS).
- We need a clear, scoped path to support additional system package managers in **the world OS** (not the host OS) when provisioning is allowed and safe (future Linux guest-rootfs, potential alternative guest images).

## 4. Proposed Outcome (1–3 bullets)

- Substrate can provision system packages via at least one additional package manager (initial target: `pacman`) when the **world OS** is Arch-family and provisioning is supported.
- On unsupported worlds, Substrate fails with clear manual guidance that uses correct terminology (“world OS package manager unsupported”) and points to remediation (choose a supported world image or follow manual steps).

## 5. Non-Goals (explicit)

- Mutating Linux host-native workstation OS packages as a side effect of world-deps (still forbidden by default posture).
- Replacing the world-deps inventory schema wholesale.
- Adding every package manager in one ADR (keep to one manager / one behavior delta).
- Solving “multiple worlds / world refresh” lifecycle management (separate track).

## 6. Constraints / Invariants (security, UX, compatibility, performance)

- **Security:** OS mutation remains explicit, operator-invoked, and limited to provisioning-time (`--provision-deps`).
- **Compatibility:** existing apt-based provisioning behavior remains unchanged.
- **Determinism:** provisioning logic must be based on the world OS, not the host OS PATH.
- **UX:** errors must be actionable and avoid suggesting the wrong manager.
- **Posture:** Linux host-native backend must not mutate the workstation OS; Linux system package provisioning is only supported when a guest-rootfs/guest backend is active (ADR-0009).

## 7. Interfaces / Contracts (CLI/config/API/files/events) — list concrete changes

- `substrate world enable --provision-deps`:
  - Detect/know the world OS package manager for the active backend and choose the correct provisioning implementation.
  - If the world OS is Arch-family, use `pacman` to install required system packages (derived from enabled world-deps set).
- World OS detection contract (locked): Substrate executes a small in-world probe under the provisioning flow (profile) to determine:
  - `/etc/os-release` `ID` / `ID_LIKE` (best-effort), and
  - manager presence via `command -v pacman` (and later others).
  This must not rely on host OS PATH.

## 8. Options (at least 2)

### Option 1 — Add `install.method=pacman` alongside `install.method=apt` in world-deps inventory (explicit per-item)

**Description (1 paragraph)**
Extend the inventory schema to allow `install.method=pacman` with `install.pacman: [...]`. Provisioning chooses the
method declared in each item and runs the corresponding manager inside the world OS.

**Pros**
- Explicit and readable; avoids magic mapping.
- Lets inventory authors target specific managers.

**Cons**
- Schema expansion; need to define versioning/validation and docs.
- Built-in items may need duplication (apt vs pacman variants) or platform filtering logic.

**Risk notes**
- Inventory bloat and drift if we add many managers this way.

### Option 2 — Introduce an abstract “system_packages” install method, with per-distro mapping (avoid in first ship)

**Description (1 paragraph)**
Define an abstract method like `install.method=system_packages` and let Substrate translate package names to the
appropriate manager for the world OS.

**Pros**
- Inventory stays manager-agnostic.

**Cons**
- Hard problem (package name mapping differs widely across distros).
- High risk of surprising failures and ongoing maintenance burden.

**Risk notes**
- This easily becomes a long-term “distro package mapping” project.

## 9. Recommendation (tentative) + “Choose Option X when…”

Locked proposal: **Option 1**, kept small:
- 1 new install method (`pacman`)
- limited to provisioning-time (`--provision-deps`)
- world OS must be detectably Arch-family
- no AUR helpers (`yay`/`paru`/`pamac`) in this ADR

Choose Option 2 only if we’re ready to own package-name translation across distros.

## 10. Slice Decomposition (required)

- ADR Candidate A (this one): add `pacman` system package provisioning in provisioning-time world enable flow.
  - Slice 1: Define how Substrate detects the world OS package manager (doctor field or probe).
  - Slice 2: Implement pacman provisioning path guarded by “world is Arch-family”.
  - Slice 3: Update docs + add tests/fixtures for the new method (at least validation + command construction tests).
- Candidate B (follow-up): add additional managers (dnf/yum/zypper) or expand inventory built-ins by platform.

## 11. Acceptance Criteria Draft (<= 8 items, observable outcomes)

- On an Arch-family world OS where provisioning is supported, `substrate world enable --provision-deps` installs required system packages using `pacman`.
- On non-Arch worlds, `pacman` provisioning path is not selected.
- On unsupported worlds, the command fails with an actionable error and does not suggest apt/pacman incorrectly.
- Runtime `substrate world deps current sync|install` continues to fail early for system packages (no OS mutation).

## 12. Dependencies

- depends_on_adrs: [`provisioning_otter`]
- depends_on_work_items: []
- blocks: []
- Related intakes (coordination only): [`detecting_badger`, `stashing_ferret`]

## 13. Lift Summary

### Lift Vector v1

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

### Computed outputs (from `make pm-lift-intake`)

```text
Lift Score (v1): 27
Estimated slices: 3
Confidence: low
Triggers:
- likely_split:lift_score>24
- missing_inputs:qa.new_test_cases
- missing_inputs:qa.new_test_files
- missing_inputs:risk.unknowns_high
- missing_inputs:touch.boundary_crossings
- missing_inputs:touch.create_files
Missing inputs:
- qa.new_test_cases
- qa.new_test_files
- risk.unknowns_high
- touch.boundary_crossings
- touch.create_files
```

## 14. Open Questions / Unknowns (with priority)

- (Locked) World OS detection source of truth: in-world probe during provisioning.
- P0: Where does “provisioning supported” live for Linux (blocked on ADR-0009 guest-rootfs) vs guest worlds?
- P1: Do we want built-in inventory items to include pacman variants now, or only enable pacman for user-defined inventory first?

## 15. Ready-to-lockdown checklist (yes/no with reasons)

- [x] One behavior delta locked (add `pacman` provisioning support).
- [x] World OS detection contract chosen (in-world probe).
- [x] Scope pinned to provisioning-time only (`--provision-deps`); no AUR helpers.
- [x] Clear posture statement for Linux host-native vs guest-rootfs (references ADR-0009).
