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

1) **Y0 (YAML settings migration)** must land first (Y0-spec): new config/settings surfaces are YAML-only and are the precedent for new runtime YAML surfaces.
2) **Agent hub hardening** must land before we rely on full cage behavior:
   - I2/I3 must include a writable mount for `/var/lib/substrate/world-deps` (DR-0008).
3) **World sync init/gating (C0)** must land before we make workspace selection first-class:
   - `.substrate/` exists and is treated as a protected path by sync filters.

Final sequencing changes are recorded in:
- `docs/project_management/next/world_deps_selection_layer/integration_map.md`
- `docs/project_management/next/sequencing.json`

---

## UX principles (non-negotiable)
- Unconfigured selection → no-op, exit 0, with one crisp instruction line.
- Errors are explicit and actionable, including platform-specific “why” and “what to do next”.
- Install classes are visible in `status` output (human + JSON).
- “Worlds must feel the same”: if a workflow cannot work on a platform, it must fail with an explicit, actionable error (not silent fallback).
