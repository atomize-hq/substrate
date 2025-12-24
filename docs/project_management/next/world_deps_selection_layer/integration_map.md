# World Deps Selection Layer — Integration Map

This document maps every component and surface area touched by ADR-0002 and the “world_deps_selection_layer” triad,
including how the work aligns with Y0 (YAML settings), I0–I5 (agent hub hardening), and C0–C9 (world-sync).

## Scope
- Covered: `substrate world deps …` (status/sync/install), selection config, install classes enforcement, provisioning-time routes for `system_packages`, platform parity behavior.
- Not covered: backwards-compat/migration layers (greenfield constraint).

## Primary user journeys (must be consistent across platforms)
1) Fresh install → enable world → configure selection → `world deps status` → `world deps sync`
2) Unconfigured selection → `world deps status|sync` (no-op + actionable guidance)
3) Tool selected but requires `system_packages` → explicit provisioning route (or explicit fail with guidance)
4) Full-cage requested/enforced → behavior is explicit and predictable (fail-closed where guarantees are requested)

## Components and touchpoints

### Host CLI (`substrate`)
- **Surface:** `substrate world deps status|sync|install`
- **Config inputs:** workspace + global YAML settings (post-Y0), overlays, selection file(s)
- **Outputs:** status JSON, actionable errors, execution spans / trace schema

### Shell world-deps implementation
- **Code:** `crates/shell/src/builtins/world_deps/*`
- **Responsibilities:**
  - parse selection + inventory + overlays
  - enforce install classes
  - dispatch to world backend for guest actions
  - produce consistent UX and exit codes

### World backend factory / routing
- **Code:** `crates/world-backend-factory/*`, `crates/shell/src/execution/routing/*`
- **Responsibilities:** choose backend (Linux host, macOS Lima, Windows WSL), normalize env, enforce backend constraints

### World agent (guest)
- **Surface:** REST/WS endpoints (`/v1/execute`, `/v1/stream`, `/v1/capabilities`)
- **Responsibilities:** execute requested guest commands, collect fs diffs, honor policy constraints

### Policy / broker
- **Code:** `crates/broker/*`
- **Responsibilities:** evaluate allow/deny/isolation; ensure world-deps operations do not silently escalate privileges

### Installer / provisioning
- **Scripts:** `scripts/substrate/*`, `scripts/mac/*`, `scripts/linux/*`, `scripts/windows/*`
- **Responsibilities:** provisioning-time system packages path (where supported), repair/upgrade commands, socket/service setup

## Install class routing (expected)
- `user_space`: always uses Substrate-managed writable prefixes inside the world; no OS package DB mutation.
- `system_packages`: routed to provisioning-time step(s) (or explicit “unsupported” error per platform).
- `copy_from_host`: explicit, selection-driven copying of known tools (where supported).
- `manual`: never installs; shows instructions.

## Platform notes (must be explicitly reconciled in the specs)

### macOS (Lima guest)
- Provisioning-time: guest OS mutation is allowed only via explicit provisioning commands/steps.
- Runtime: world-deps must not assume `apt`/`dpkg` is available/writable under the current cage/policy.

### Linux (host world-agent)
- Default posture: forbid host system package mutation via world-deps (unless explicitly opted-in).

### Windows (WSL)
- Provisioning-time: explicit and idempotent WSL bootstrap route for system packages (if required).

## Sequencing alignment (required output)
- List the exact prerequisite work items from Y0/I0–I5/C0–C9 that must land before implementing each slice here.
- Propose the exact edits to `docs/project_management/next/sequencing.json` needed for a conflict-free, aligned plan.

