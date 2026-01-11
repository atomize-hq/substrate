# World Deps Selection Layer (ADR-0002) — Plan

## Context (why this exists)
`substrate world deps` must become:
- selection-driven (no surprise installs) and no-op when unconfigured (ADR-0001 D1),
- explicit about what is installable at runtime vs what requires provisioning (`system_packages`) (ADR-0002),
- compatible with upcoming full-cage hardening (pivot_root + Landlock additive hardening) (I2–I4),
- consistent across Linux, macOS (Lima), and Windows (WSL) where technically possible (“worlds must feel the same”).

Hard constraints:
- YAML-only runtime config (Y0); no new TOML.
- World-deps selection is **required**; missing selection → **no-op**.
- World-deps must be selection-driven and selection-driven world-deps behavior must be a **no-op** when unconfigured (including `install`/`sync`).
- World-deps “system packages” installs must be **provisioning-time** and **explicit**; never implicit during `sync/install`.
- Greenfield: no backwards-compat layers; breaking changes are recorded explicitly.

Authoritative decision log:
- `docs/project_management/next/world_deps_selection_layer/decision_register.md`

---

## Execution model (triad + automation + cross-platform)

This Planning Pack is authored for strict triad execution:
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`
- Orchestration branch (DR-0015): `feat/world_deps_selection_layer`
- Cross-platform integration model (DR-0016): platform-fix when needed (core + per-platform + final aggregator per slice)
- Required platform sets (P3-008):
  - Behavior platforms (smoke required): `linux`, `macos`, `windows`
  - CI parity platforms (compile parity required): `linux`, `macos`, `windows`
  - WSL coverage: not required by this Planning Pack (`meta.wsl_required=false` by omission)
- Execution gates:
  - Feature start gate: `F0-exec-preflight` → `execution_preflight_report.md`
  - Per-slice end gate: `WDLx-closeout_report.md` after each `WDLx-integ` (final) task

---

## Shipping increments (explicit scope boundaries)

### Increment 1 (P0 / first ship)
- Add selection config + gating + UX (`S0`).
- Add install classes schema + enforcement (`S1`).
- Add explicit provisioning path for `system_packages` on macOS Lima + Windows WSL, and explicit “unsupported on Linux host” with manual guidance (`S2`).
- Update Substrate-owned manager inventory/overlays to declare install class metadata and to remove runtime OS-package installs from `sync/install` paths (breaking but required by ADR-0002).
- Add automatable smoke coverage via feature-local smoke scripts under `docs/project_management/next/world_deps_selection_layer/smoke/`.

Out of scope for Increment 1 (deferred, explicitly)
- `copy_from_host` implementation (schema may reserve the class; behavior remains “unsupported”).
- Supporting non-apt package managers inside guests (`apk`, `dnf`, `pacman`) beyond explicit errors/guidance.
- Any migration from existing selection/overlay files (greenfield).
- Any new privileged “mutate Linux host packages” flows (explicitly forbidden by DR-0010).

---

## Slice breakdown (spec triads)

### WDL0 — Selection config + UX (Spec: `S0-spec-selection-config-and-ux.md`)
- Define YAML schema + file locations + precedence.
- Define CLI semantics (including `--all`) and sample outputs.
- Define JSON output fields for “configured vs unconfigured”.
- Define exit codes and warn/fail behavior for all failure modes.

### WDL1 — Install classes (Spec: `S1-spec-install-classes.md`)
- Extend layered manager manifest schema to declare install class and (where applicable) system package requirements.
- Enforce class routing in `world deps status|sync|install`.
- Ensure `system_packages` are never installed by runtime `sync/install` paths.
- Provide consistent error/warning messages and status signals.

### WDL2 — Provisioning-time system packages (Spec: `S2-spec-system-packages-provisioning.md`)
- Implement `substrate world deps provision`:
  - selection-driven: installs packages required by selected tools (or all inventory with `--all`),
  - apt-only inside guest for macOS Lima + Windows WSL (Ubuntu guests),
  - Linux host: explicit unsupported error + manual guidance.
- Add an explicit “repair/upgrade” story: re-running `provision` is safe and idempotent.
- Extend platform smoke scripts to validate `provision` + `sync` journeys.

---

## Cross-sprint prerequisites (must be satisfied; no open dependencies)

1) **Y0 (YAML settings migration)** must land first (YAML-only runtime config baseline):
   - Reference: `docs/project_management/_archived/yaml-settings-migration/Y0-spec.md`
2) **Hardening (full cage)** must already include a writable mount for `/var/lib/substrate/world-deps` (DR-0008) before WDL2 is considered “full-cage compatible”:
   - References:
     - `docs/project_management/_archived/p0-agent-hub-isolation-hardening/I2-spec.md`
     - `docs/project_management/_archived/p0-agent-hub-isolation-hardening/I3-spec.md`

Explicit non-dependency:
- This feature does not require the `world-sync` workspace init flow (`substrate init`) to exist before WDL0 ships.
  - `substrate world deps init --workspace` and `substrate world deps select --workspace` create `.substrate/` when needed and only write the selection file (S0).

Final sequencing changes are recorded in:
- `docs/project_management/next/world_deps_selection_layer/integration_map.md`
- `docs/project_management/next/sequencing.json`

---

## UX principles (non-negotiable)
- Unconfigured selection → no-op, exit 0, with one crisp instruction line.
- Errors are explicit and actionable, including platform-specific “why” and “what to do next”.
- Install classes are visible in `status` output (human + JSON).
- “Worlds must feel the same”: if a workflow cannot work on a platform, it must fail with an explicit, actionable error (not silent fallback).

## Validation artifacts (authoritative)
- Manual playbook: `docs/project_management/next/world_deps_selection_layer/manual_testing_playbook.md`
- Feature smoke entrypoints (DR-0017):
  - Linux: `docs/project_management/next/world_deps_selection_layer/smoke/linux-smoke.sh`
  - macOS: `docs/project_management/next/world_deps_selection_layer/smoke/macos-smoke.sh`
  - Windows: `docs/project_management/next/world_deps_selection_layer/smoke/windows-smoke.ps1`
