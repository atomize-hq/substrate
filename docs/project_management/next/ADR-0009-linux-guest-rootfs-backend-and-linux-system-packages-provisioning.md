# ADR-0009 — Linux Guest RootFS Backend + Linux System Packages Provisioning

## Status
- Status: Draft
- Date (UTC): 2026-01-11
- Owner(s): Shell / World / Installer maintainers

## Scope
- Feature directory: `docs/project_management/next/linux_guest_rootfs_backend/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md` (automation/worktree execution)

## Related Docs
- Prior ADR (world-deps provisioning posture): `docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
- Prior hardening track (full cage / Landlock): `docs/project_management/_archived/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`
- Plan (stub): `docs/project_management/next/linux_guest_rootfs_backend/plan.md`
- Decision Register (stub; required): `docs/project_management/next/linux_guest_rootfs_backend/decision_register.md`
- Integration Map (stub): `docs/project_management/next/linux_guest_rootfs_backend/integration_map.md`
- Manual Playbook (stub): `docs/project_management/next/linux_guest_rootfs_backend/manual_testing_playbook.md`
- World architecture: `docs/WORLD.md`
- World-sync (future pinning surface; out of scope here): `docs/project_management/next/world-sync/plan.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=docs/project_management/next/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md` after drafting>

### Changes (operator-facing)
- Linux can provision `system_packages` without mutating the host OS
  - Existing: on Linux, `world-agent` runs on the host and `world deps provision` is intentionally unsupported because installing OS packages would mutate the workstation.
  - New: Linux gains a guest-like backend based on a Substrate-managed Linux root filesystem (“guest rootfs”), enabling `world deps provision` to install apt packages into the guest rootfs (not the host) while keeping the same explicit/selection-driven UX as macOS (Lima) and Windows (WSL).
  - Why: restores parity for `system_packages` while preserving the agent-hub threat model (“no privileged host mutation as a side effect of tool selection”).
  - Links:
    - `docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
    - `docs/WORLD.md`

## Problem / Context
- Substrate’s Linux world backend is “host-native”: the agent runs directly on the workstation (`docs/WORLD.md`), so any OS package install performed during provisioning would change the host OS package set.
- The world-deps selection layer formalizes `system_packages` as “must never be installed during runtime sync/install; must be fulfilled by an explicit provisioning command on supported guests” (`docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`).
- Substrate’s Linux “full isolation” mode (mount namespace + `pivot_root`) is designed to make host paths unnameable and constrain writes, but today it still relies on read-only bind mounts of host `/usr`, `/etc`, etc. That deliberately prevents any OS package mutation inside the cage, so apt/dpkg cannot be used as a “guest provisioning” mechanism without changing the design.

We need a Linux “guest-like” environment that:
- runs on the host kernel (no VM required),
- isolates the filesystem root via `pivot_root` (same hardening posture),
- provides persistent OS package state for apt installs,
- does not mutate the host OS package set.

## Goals
- Provide a Linux “guest rootfs” backend that Substrate manages and can execute inside when `world_fs.isolation=full` is requested.
- Enable `substrate world deps provision` on Linux when and only when the guest-rootfs backend is active, using the existing `system_packages` routing contract (apt-only in the initial ship).
- Make failure modes explicit and scriptable (exit codes), including clear remediation when the guest rootfs is not present/ready.
- Preserve the security posture of ADR-0002 (no implicit OS mutation; explicit provisioning only).

## Non-Goals
- Distro selection/pinning and cross-platform “world image” management UI (explicitly deferred; see `docs/BACKLOG.md`).
- Supporting non-apt package managers (`dnf`, `pacman`, `apk`) for provisioning in this ship.
- Any opt-in flow that mutates host OS packages on Linux (still forbidden by default posture).
- Replacing Lima (macOS) or WSL (Windows) backends.
- Reworking world-sync semantics; image pinning may later integrate with workspace init but is out of scope here.

## User Contract (Authoritative)

### CLI

#### Linux guest-rootfs warm/provisioning
- A guest rootfs MUST be created by an explicit operator action (no implicit creation during `world deps` runtime flows).
- Initial ship command surface is script-first:
  - `scripts/linux/world-rootfs-warm.sh` (new): provisions or repairs the guest rootfs used by the Linux guest-rootfs backend.

If `world deps provision` requires guest rootfs and it is missing/unready, Substrate MUST print a remediation block that points to `scripts/linux/world-rootfs-warm.sh` (similar to Lima/WSL warm guidance in S2).

#### `substrate world deps provision` (Linux behavior)
- When Linux is using the host-native backend (no guest rootfs active):
  - Behavior remains unchanged from ADR-0002: exit `4` and print manual package guidance (no host mutation).
- When Linux is using the guest-rootfs backend and the guest rootfs is ready:
  - `substrate world deps provision` is supported and installs apt packages inside the guest rootfs only (no host OS mutation).
  - Package manager support remains apt-only in this ship (Ubuntu/Debian-family guest rootfs).

Exit codes:
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `0`: success (including “no system packages required”)
- `2`: configuration/usage error (invalid selection YAML, unknown tool id, schema mismatch)
- `3`: world backend unavailable when required (e.g., world-agent/socket unavailable)
- `4`: unsupported/missing prerequisites (e.g., Linux host-native backend selected; guest rootfs missing; guest rootfs does not support apt)
- `5`: safety-rail refusal (hardening/cage conflict prevents operation; protected-path violation)

### Config

This ADR introduces a new Linux backend selection knob (exact schema location is implemented by the planning pack; contract-level behavior is fixed here):
- Default: Linux uses the host-native backend.
- When the Linux guest-rootfs backend is enabled for the session/workspace:
  - `world_fs.isolation=full` pivots into the guest rootfs (not host `/usr`/`/etc` bind mounts).
  - `world deps provision` is permitted to install `system_packages` into the guest rootfs.

### Platform guarantees
- Linux:
  - Host-native backend: provisioning-time OS package installation remains unsupported (manual guidance only).
  - Guest-rootfs backend: `system_packages` provisioning is supported and MUST NOT mutate the host OS package set.
- macOS:
  - No behavior changes required by this ADR (Lima remains the guest; apt-only provisioning remains supported there).
- Windows:
  - No behavior changes required by this ADR (WSL remains the guest; apt-only provisioning remains supported there).

## Architecture Shape

### Components
- `crates/world`:
  - Add guest-rootfs mount/overlay plumbing used by `world_fs.isolation=full` (Linux).
  - Ensure the guest rootfs provides persistent system state for apt installs (no host `/usr` mutation).
- `crates/world-agent`:
  - Expose capability/doctor readiness for guest-rootfs availability.
  - Ensure both non-PTY (`/v1/execute`) and PTY (`/v1/stream`) enter the guest-rootfs-backed full cage when configured.
- `crates/shell`:
  - Plumb the backend selection from config/env into the world-agent request environment (or explicit request fields, as implemented).
  - Ensure error messaging points to the warm script when guest rootfs prerequisites are missing.
- `scripts/linux/world-rootfs-warm.sh` (new):
  - Build/repair the guest rootfs under a Substrate-owned location, with explicit privilege use (sudo) and idempotence.

### End-to-end flow
- Inputs:
  - world-deps selection + install class metadata (ADR-0002 / WDL0–WDL2)
  - backend selection (host-native vs guest-rootfs) for Linux
  - world fs isolation settings (`world_fs.isolation=full`, allowlists)
- Derived state:
  - whether guest-rootfs is active
  - whether guest-rootfs is ready and apt-capable
  - computed apt package list for the selected tools
- Actions:
  - warm: create/repair guest rootfs (operator action)
  - execute: pivot_root into guest rootfs for full isolation (Linux guest-rootfs backend)
  - provision: run `apt-get update` + `apt-get install` inside the guest rootfs
- Outputs:
  - explicit success/failure with stable exit codes
  - no host OS package mutation on Linux in guest-rootfs mode

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → `linux_guest_rootfs_backend` (to be added)
- Prerequisites:
  - WDL2 (`docs/project_management/next/world_deps_selection_layer/`) must land first so `world deps provision` exists and has a stable contract for `system_packages`.
  - Linux full-cage mechanics (I2/I3) must already be available (or this feature must fail closed when requested but unavailable).

## Security / Safety Posture
- Fail-closed rules:
  - If guest-rootfs-backed full isolation is required but the guest rootfs is unavailable/unhealthy, Substrate MUST fail closed (do not silently run a weaker mode).
- Protected paths/invariants:
  - Guest rootfs storage MUST live under a Substrate-owned, non-workspace path (e.g., under `/var/lib/substrate/...`) and must never be placed inside the workspace.
  - `world deps provision` MUST NOT execute host package-manager operations on Linux host-native backend.
  - All mounts and overlay operations MUST remain confined to Substrate-managed roots and the project mount points established by full isolation.
- Observability requirements:
  - `substrate world doctor --json` MUST surface whether the guest-rootfs backend is active and ready (exact field names defined by the planning pack).
  - Trace spans SHOULD annotate whether full isolation used host-native binds vs guest-rootfs (no schema-breaking changes; add fields only if required).

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Validate guest rootfs path computation, safety checks (no workspace paths), and readiness detection.
  - Validate that Linux `world deps provision` selects “unsupported/manual guidance” when host-native backend is active.
- Integration tests:
  - A Linux integration test that enables guest-rootfs mode, provisions a trivial apt package, and verifies it is available in subsequent executions (persistence).
  - A regression test proving no host OS package-manager calls occur when host-native backend is active.

### Manual validation
- Manual playbook: `docs/project_management/next/linux_guest_rootfs_backend/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/linux_guest_rootfs_backend/smoke/linux-smoke.sh` (to be created)
- macOS: N/A (no behavior change in this ADR)
- Windows: N/A (no behavior change in this ADR)

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none (feature is gated behind explicit enablement; existing Linux behavior remains unchanged by default)

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/linux_guest_rootfs_backend/decision_register.md`:
    - DR-0001, DR-0002, DR-0003, DR-0004, DR-0005 (to be written before execution)

