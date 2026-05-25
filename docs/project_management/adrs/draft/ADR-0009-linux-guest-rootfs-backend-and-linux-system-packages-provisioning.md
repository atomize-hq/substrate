# ADR-0009 — Linux Guest RootFS Backend + Distro-Decoupled Full Isolation and Provisioning

## Status
- Status: Draft
- Date (UTC): 2026-05-24
- Owner(s): Shell / World / Installer maintainers

## Scope
- Feature directory: `docs/project_management/_archived/next/linux_guest_rootfs_backend/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Prior ADR (world-deps install-class and provisioning posture): `docs/project_management/adrs/implemented/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
- Current provisioning surface: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
- Future non-APT guest-image routing context: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- Future backend capability alignment: `docs/project_management/adrs/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`
- Prior hardening track (full cage / Landlock): `docs/project_management/_archived/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`
- World architecture: `docs/WORLD.md`
- Operator reference for provisioning/runtime world-deps behavior: `docs/reference/world/deps/README.md`
- Skill Phase 2 plan: `docs/project_management/_archived/next/linux_guest_rootfs_backend/plan.md`
- Decision Register: `docs/project_management/_archived/next/linux_guest_rootfs_backend/decision_register.md`
- Skill Phase 1 spec: `docs/project_management/_archived/next/linux_guest_rootfs_backend/spec.md`
- World-image pinning roadmap (explicitly out of scope here): `docs/BACKLOG.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 056a47bf1016afd1fc53a0b8ec7bf419da811ee3b7bdcf0ce77abca92cc5b875

### Changes (operator-facing)
- Linux can run against an explicit guest distro without a full VM
  - Existing: Linux world execution is tied to the host distro/userspace, and `world_fs.isolation=full` still depends on host system content mounted read-only into the cage.
  - New: Linux can opt into a Substrate-managed `guest_rootfs` backend that runs world execution against a guest userspace unpacked from an OCI-style rootfs image on the host kernel. Backend selection and guest-image selection are separate concepts; the first shipped guest image family is Ubuntu/Debian.
  - Why: decouples host distro from world distro, enables a Manjaro host to run an Ubuntu world, and strengthens Linux isolation semantics without paying full-VM cost.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md#user-contract-authoritative`
    - `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md#architecture-shape`
    - `docs/project_management/_archived/next/linux_guest_rootfs_backend/decision_register.md#dr-0001--linux-guest-image-format-and-builder`
- Linux provisioning stays explicit and never mutates the host OS
  - Existing: `substrate world enable --provision-deps` is unsupported on Linux host-native because provisioning would mutate the workstation OS package set.
  - New: when Linux uses the `guest_rootfs` backend, the guest rootfs is warmed, and the active guest image is the blessed Ubuntu/Debian image, `substrate world enable --provision-deps` installs apt packages into a persistent guest overlay only. Linux `host_native` remains unsupported for provisioning.
  - Why: preserves the ADR-0002 / ADR-0030 threat model while making guest-backed Linux provisioning possible.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md#user-contract-authoritative`
    - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md#user-contract-authoritative`
    - `docs/reference/world/deps/README.md#provisioning-contract`

## Problem / Context
- Substrate's current Linux world backend is `host_native`: the agent and the effective Linux userspace are the workstation's own distro (`docs/WORLD.md`).
- Linux `world_fs.isolation=full` already hides host paths by building a minimal rootfs and `pivot_root`ing into it, but that cage still depends on read-only bind mounts of host `/usr`, `/etc`, and related system paths. That means the world's distro identity is still coupled to the host distro.
- ADR-0002 and ADR-0030 already locked the security posture that OS/system package mutation must be explicit, provisioning-time only, and must not occur on Linux host-native workstations.
- The current operator-facing provisioning surface is `substrate world enable --provision-deps`; this ADR must extend that contract for Linux guest worlds and must not resurrect the older `substrate world deps provision` shape.

Without a guest-rootfs backend:
- a Manjaro host cannot run an Ubuntu world without introducing a VM,
- Linux full isolation cannot be both distro-decoupled and package-provisionable,
- Linux remains permanently behind macOS Lima for guest-backed system-package flows.

## Goals
- Add an optional Linux `guest_rootfs` backend that decouples the world distro from the host distro while still running on the host kernel.
- Keep `host_native` as the default Linux backend in the first ship.
- Make backend selection and guest-image selection separate config surfaces.
- Run Linux world execution against the selected guest userspace when `guest_rootfs` is active, and ensure `world_fs.isolation=full` uses the guest rootfs rather than host system-directory bind mounts.
- Support `substrate world enable --provision-deps` on Linux when and only when the `guest_rootfs` backend is selected, warmed, and using the supported Ubuntu/Debian guest image.
- Make rootfs/image warm-up explicit, doctor-visible, and fail-closed when prerequisites are missing.

## Non-Goals
- Making `guest_rootfs` the default Linux backend in the first ship.
- Introducing a full VM-backed Linux backend.
- Supporting arbitrary guest distros, user-imported guest images, or a general world-image catalog in this ship.
- Supporting non-APT Linux guest-image provisioning in this ADR (`pacman`, `dnf`, `yum`, `apk`, `zypper` remain follow-on work).
- Replacing or reworking the macOS Lima backend or the current Windows posture.
- Solving workspace-level world-image pinning and upgrade policy; that remains follow-on work under the backlog world-image/pinning track.

## User Contract (Authoritative)

### CLI

#### Linux guest-rootfs warm flow
- The Linux guest rootfs/image store MUST be created or repaired by an explicit operator action.
- The initial warm surface is script-first:
  - `scripts/linux/world-rootfs-warm.sh`
- The warm script MUST:
  - fetch or otherwise obtain the blessed Ubuntu/Debian guest rootfs as an OCI-style rootfs image source,
  - verify image digest and provenance against the built-in manifest,
  - unpack the immutable base image into a Substrate-owned system path,
  - create and repair the system-path storage layout required by the backend,
  - repair ownership and permissions under the backend storage root,
  - perform generic backend warm and repair work that is not scope-local overlay creation,
  - be idempotent and safe to re-run as the repair path.
- World execution and `substrate world enable --provision-deps` MUST NOT implicitly:
  - fetch images,
  - verify or re-verify image provenance as repair work,
  - recreate or repair the base-image store,
  - fix ownership or permissions,
  - perform generic warm or repair.
- After the backend is otherwise ready, world execution and `substrate world enable --provision-deps` MAY lazily create the empty scope-local overlay directory for the resolved scope under `/var/lib/substrate/world-rootfs/overlays/...`.
- Failure to create that scope-local overlay MUST fail the triggering operation with actionable remediation.
- When Linux is configured for `guest_rootfs` and the guest rootfs is missing or unready, operator-facing remediation MUST point to `scripts/linux/world-rootfs-warm.sh`.

#### `substrate world enable --provision-deps` (Linux behavior)
- When `world.linux.backend=host_native`:
  - behavior remains explicit failure with exit `4`,
  - output MUST state that provisioning is unsupported because it would mutate the host OS,
  - Substrate MUST NOT execute host package-manager commands.
- When `world.linux.backend=guest_rootfs` and the guest rootfs is warmed and ready:
  - provisioning is supported,
  - package-manager support is apt-only in this ship,
  - package installation MUST occur inside the persistent guest overlay only,
  - host OS package state MUST remain unchanged.
- When `world.linux.backend=guest_rootfs` but the guest rootfs is missing, unready, or not provision-capable:
  - exit `4`,
  - print actionable remediation that points to `scripts/linux/world-rootfs-warm.sh` or the specific unsupported-image reason.
- When `world.linux.backend=guest_rootfs` and the active guest image is not the blessed Ubuntu/Debian image:
  - exit `4`,
  - do not attempt provisioning,
  - print that the active guest image does not support apt provisioning in this ship.

#### Linux world execution behavior
- When `world.linux.backend=host_native`:
  - Linux world execution keeps the existing host-native behavior.
- When `world.linux.backend=guest_rootfs`:
  - Linux world execution MUST run against the selected guest userspace rather than the host distro userspace,
  - the backend MUST NOT silently fall back to `host_native`,
  - `substrate world doctor --json` MUST report the active backend, active guest image, and guest OS identity.
- When `world.linux.backend=guest_rootfs` and `world_fs.isolation=full` is requested:
  - the guest rootfs MUST be the isolation root,
  - host `/usr`, `/etc`, and related system directories MUST NOT be rebound into the command view as the effective world system directories,
  - if the backend cannot satisfy that guarantee, execution MUST fail closed.

#### Exit codes
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `0`: success
- `2`: configuration or usage error (invalid backend value, malformed image selection, incompatible config combination)
- `3`: world backend unavailable when execution is required (socket/service/connectivity failure)
- `4`: unsupported operation or missing prerequisite (host-native provisioning request, missing guest rootfs warm-up, unsupported guest image, guest provisioning unavailable)
- `5`: fail-closed safety refusal (hardening invariant or protected-path invariant could not be preserved)

### Config

Config files and precedence:
- Global config: `$SUBSTRATE_HOME/config.yaml`
- Workspace config: `<workspace_root>/.substrate/workspace.yaml`
- Precedence: workspace config overrides global config for the keys defined below.

Logical config surfaces introduced by this ADR:
- `world.linux.backend`
  - allowed values: `host_native | guest_rootfs`
  - default: `host_native`
- `world.linux.image`
  - type: stable image-identity string
  - applies only when `world.linux.backend=guest_rootfs`

Behavioral constraints:
- Backend selection and image selection are separate concepts. Setting an image MUST NOT implicitly switch the backend from `host_native` to `guest_rootfs`.
- If `world.linux.backend=guest_rootfs` and `world.linux.image` is unset, Substrate MUST select the built-in default Ubuntu/Debian guest image.
- In the first ship, exactly one built-in guest image family is supported for Linux `guest_rootfs`: Ubuntu/Debian, apt-capable.
- If `world.linux.image` is set to any other image identity in the first ship:
  - world execution MAY still reject activation with exit `4`,
  - provisioning MUST reject activation with exit `4`,
  - the error MUST identify the unsupported image as the reason.

### Platform guarantees
- Linux:
  - `host_native` remains the default Linux backend.
  - `guest_rootfs` is explicit opt-in.
  - The first shipped guest image family is Ubuntu/Debian and apt-capable.
  - Linux provisioning support exists only through `substrate world enable --provision-deps` on `guest_rootfs`.
- macOS:
  - no behavior change in this ADR; Lima remains the guest backend and the current provisioning contract remains authoritative there.
- Windows:
  - no behavior change in this ADR;
  - the current unsupported posture for `substrate world enable` / provisioning remains authoritative.

## Architecture Shape

### Components
- `crates/world`
  - add a Linux `guest_rootfs` backend path that runs commands against a guest userspace unpacked from an OCI-style rootfs image,
  - maintain an immutable base-image store plus persistent writable overlays,
  - ensure `world_fs.isolation=full` uses the guest rootfs as the effective root rather than host system-directory bind mounts.
- `crates/world-service`
  - surface backend/image readiness and guest OS identity in doctor responses,
  - ensure both non-PTY (`/v1/execute`) and PTY (`/v1/stream`) executions honor the selected Linux backend and image.
- `crates/shell`
  - resolve `world.linux.backend` and `world.linux.image` from config precedence,
  - pass backend/image selection into world execution and provisioning flows,
  - route Linux `substrate world enable --provision-deps` through the guest-rootfs readiness gate and remediation path.
- `scripts/linux/world-rootfs-warm.sh`
  - obtain and unpack the blessed guest rootfs image,
  - verify provenance and digest before making the image available,
  - prepare the immutable base-image store and backend storage layout under a Substrate-owned system path,
  - perform explicit privileged setup and repair.

### End-to-end flow
- Inputs:
  - `world.linux.backend`
  - `world.linux.image`
  - `world_fs` isolation/mode policy
  - effective enabled world-deps set for the current workspace
- Derived state:
  - active Linux backend kind
  - active guest image identity
  - guest rootfs warm/readiness state
  - guest OS identity (`/etc/os-release`, arch, kernel where available)
  - normalized apt package requirement set for provisioning
- Actions:
  - warm: `scripts/linux/world-rootfs-warm.sh` prepares the image/overlay store
  - lazy overlay create: runtime execution or `substrate world enable --provision-deps` may create the empty scope-local overlay for the resolved scope once the backend is otherwise ready
  - execute: world-service runs commands against the selected backend and guest userspace
  - full isolation: `guest_rootfs` mode uses the guest image as the world root and applies the existing full-isolation semantics inside that guest-backed world
  - provision: `substrate world enable --provision-deps` installs apt packages into the guest overlay only
- Outputs:
  - explicit success/failure with stable exit codes
  - no host OS package mutation on Linux
  - doctor-visible backend/image/readiness truth

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `linux_guest_rootfs_backend` (to be added during planning)
- Dependencies:
  - The current `substrate world enable --provision-deps` operator contract from ADR-0030 and `docs/reference/world/deps/README.md` is the provisioning surface this ADR extends for Linux guest worlds.
  - This ADR MUST NOT reintroduce or depend on the older `substrate world deps provision` command surface.
  - Linux full-isolation mechanics from the prior hardening track remain prerequisites; if those mechanics are unavailable, `guest_rootfs` requests must fail closed.
  - Future non-APT guest-image support aligns with ADR-0033 and is explicitly out of scope for this first ship.
  - Future world-image pinning and cross-platform image identity work remains follow-on planning under the backlog world-image track.

## Security / Safety Posture
- Fail-closed rules:
  - When `world.linux.backend=guest_rootfs`, Substrate MUST NOT silently fall back to `host_native`.
  - When `guest_rootfs` is selected but the rootfs/image store is missing or unhealthy, guest-rootfs execution and provisioning MUST fail closed.
  - When `world_fs.isolation=full` is requested under `guest_rootfs`, failure to preserve the guest-rootfs-as-root guarantee MUST be treated as exit `5`.
- Protected paths and invariants:
  - Base images and overlays MUST live under a Substrate-owned system path (for example, under `/var/lib/substrate/...`) and MUST NEVER be placed in the workspace or under `$SUBSTRATE_HOME`.
  - The base guest image MUST be treated as immutable after warm/unpack.
  - Package provisioning MUST mutate only the persistent guest overlay, never the base image and never the host OS package set.
  - Runtime execution and provisioning MAY create the empty scope-local overlay directory for a resolved scope, but they MUST NOT broaden that behavior into generic backend repair.
  - All mount, overlay, and unpack operations MUST remain confined to Substrate-managed paths plus the explicit project mounts used for world execution.
- Observability requirements:
  - `substrate world doctor --json` MUST surface, at minimum:
    - `world.backend.kind`
    - `world.image.id` when `guest_rootfs` is active
    - `world.os` for the active world
    - `world.provisioning.supported`
    - `world.provisioning.ready`
    - `world.provisioning.reason` when ready is false
  - Trace spans SHOULD record the active backend kind, active guest image identity when present, and effective isolation mode using additive fields only.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - validate backend/image config parsing and precedence,
  - validate guest-image selection rules and unsupported-image failures,
  - validate guest-rootfs path safety (no workspace or `$SUBSTRATE_HOME` placement),
  - validate readiness-state mapping for doctor/provisioning.
- Integration tests:
  - on Linux, prove that `guest_rootfs` execution runs against the Ubuntu/Debian guest userspace rather than the host distro userspace,
  - prove that `world_fs.isolation=full` under `guest_rootfs` does not depend on host `/usr` / `/etc` content as the effective guest system directories,
  - provision a trivial apt package via `substrate world enable --provision-deps` and verify it remains available in subsequent guest-rootfs executions,
  - prove that Linux `host_native` still rejects provisioning and never invokes host package-manager commands.

### Manual validation
- Manual validation MUST cover at least:
  - warm and repair flow,
  - backend selection and doctor output,
  - guest-rootfs execution on a non-Debian Linux host running an Ubuntu/Debian guest world,
  - provisioning success and persistence,
  - host-native rejection and no-host-mutation evidence,
  - first-use lazy creation of a scope-local overlay.

### Smoke scripts
- Linux: `docs/project_management/_archived/next/linux_guest_rootfs_backend/smoke/linux-smoke.sh` (to be created)
- macOS: N/A (no behavior delta in this ADR)
- Windows: N/A (no behavior delta in this ADR)

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work:
  - none required for the first ship because `host_native` remains the default Linux backend,
  - `guest_rootfs` is explicit opt-in,
  - unsupported or missing guest-image prerequisites fail with explicit exit `4` instead of changing existing Linux behavior.

## Decision Summary
- Decision Register entries:
  - `docs/project_management/_archived/next/linux_guest_rootfs_backend/decision_register.md`
    - DR-0001 — Linux guest image format and builder
    - DR-0002 — Guest rootfs persistence model
    - DR-0003 — Storage location and ownership
    - DR-0004 — Warm command surface
    - DR-0005 — Readiness and doctor surface
    - DR-0006 — Backend selection vs image selection
    - DR-0007 — Linux backend rollout posture
