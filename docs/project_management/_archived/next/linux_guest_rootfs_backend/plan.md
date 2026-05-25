# Plan: Linux Guest RootFS Backend and Guest-Only Linux Provisioning

This is the Phase 2 `PLAN` artifact for:
- `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

## Goal
Turn the approved spec into an implementation approach that is reviewable before tasks or code start. The plan must keep Linux behavior explicit, fail-closed, and host-safe while introducing a real guest userspace path.

## Major Components

### 1. Shell contract and config routing
- `crates/shell/`
- Responsibilities:
  - resolve `world.linux.backend`
  - resolve `world.linux.image`
  - enforce provisioning gating and remediation
  - map failures to stable exit behavior

### 2. Guest-rootfs state and lifecycle
- `crates/world/`
- `scripts/linux/world-rootfs-warm.sh`
- Responsibilities:
  - represent warmed vs missing vs unsupported vs unhealthy guest state
  - manage immutable base image location
  - manage persistent writable overlay location
  - define explicit warm/repair lifecycle

### 3. World execution path
- `crates/world/`
- `crates/world-service/`
- Responsibilities:
  - bootstrap guest-backed command execution
  - support non-PTY and PTY flows
  - fail explicitly when `guest_rootfs` is selected but unavailable

### 4. Full-isolation rootfs semantics
- `crates/world/`
- Responsibilities:
  - make guest rootfs the effective system root for `world_fs.isolation=full`
  - prevent host `/usr`, `/etc`, and similar directories from acting as guest system state
  - fail closed on invariant violations

### 5. Provisioning path
- `crates/shell/`
- `crates/world/`
- `crates/world-service/`
- Responsibilities:
  - route `substrate world enable --provision-deps` to guest-only provisioning
  - install APT packages into persistent guest overlays only
  - reject `host_native` and unsupported/unready guest images

### 6. Observability and docs
- `crates/world-service/`
- `docs/WORLD.md`
- `docs/reference/world/deps/README.md`
- Responsibilities:
  - expose backend/image/readiness truth in doctor output
  - keep operator docs aligned with actual runtime behavior

## Dependencies
- Guest-rootfs lifecycle must be defined before backend selection can map to ready vs unready behavior.
- Backend and image selection must be wired before guest execution can be introduced safely.
- Guest execution must exist before full-isolation behavior can be re-rooted onto it.
- Full-isolation guest-root behavior must be trustworthy before provisioning is enabled.
- Observability and docs depend on the runtime contract being settled.

## Implementation Order

### Step 1: Lock the shared runtime model
- Define the guest-rootfs state model:
  - warmed
  - missing
  - unsupported image
  - unhealthy
- Define where base images and overlays live.
- Define what the warm script prepares and what it never does implicitly.

Why first:
- all later work depends on one consistent readiness and storage model.

### Step 2: Wire backend and image selection
- Add deterministic parsing and validation for:
  - `world.linux.backend`
  - `world.linux.image`
- Enforce:
  - `host_native` default
  - no implicit backend switch from image selection
  - explicit failure for unsupported images

Why second:
- execution and provisioning cannot safely branch until selection behavior is fixed.

### Step 3: Introduce guest-rootfs execution
- Add guest-backed non-PTY execution.
- Add guest-backed PTY execution.
- Ensure explicit refusal when `guest_rootfs` is selected but unavailable.

Why third:
- this proves the backend is real before changing isolation or provisioning semantics around it.

### Step 4: Re-root full isolation onto the guest rootfs
- Make `world_fs.isolation=full` use the guest rootfs as the effective system root.
- Remove dependence on host system directories as the guest system view.
- Fail closed if the invariant cannot be met.

Why fourth:
- isolation is the core safety claim of the backend and must be correct before provisioning becomes useful.

### Step 5: Enable guest-only provisioning
- Gate Linux provisioning on:
  - selected `guest_rootfs`
  - ready guest state
  - supported Ubuntu/Debian image
- Install APT packages into the guest overlay only.
- Keep `host_native` refusal explicit.

Why fifth:
- provisioning is only valid after execution and isolation semantics are trustworthy.

### Step 6: Finish doctor, docs, and manual validation flow
- Add doctor fields for backend, image, OS identity, readiness, and provisioning support.
- Update operator docs to match the actual warm/provisioning flow.
- Capture manual validation and smoke coverage.

Why last:
- observability and docs must describe the final runtime behavior, not an intermediate design.

## Risks and Mitigation

### Risk: guest state semantics diverge across crates
- Mitigation:
  - define one readiness model first
  - reuse it across shell, world, and world-service

### Risk: full isolation still leaks host system assumptions
- Mitigation:
  - validate guest `/etc/os-release`
  - inspect mount/layout behavior in integration tests
  - treat invariant failure as exit `5`

### Risk: provisioning mutates the wrong layer
- Mitigation:
  - keep the base image immutable
  - mutate overlays only
  - add explicit host-native refusal checks

### Risk: backend selection creates ambiguous invalid states
- Mitigation:
  - keep backend and image separate
  - validate both centrally
  - define explicit failure modes before tasks start

### Risk: Linux-specific work regresses other platforms
- Mitigation:
  - preserve platform seams
  - keep macOS and Windows in compile/test parity coverage

## Parallel vs Sequential Work

### Must stay sequential
- Guest state model before backend/image routing
- Backend/image routing before guest execution
- Guest execution before full-isolation re-rooting
- Full-isolation re-rooting before provisioning enablement

### Can run in parallel after prerequisites exist
- Shell contract tests and world-service readiness plumbing
- PTY and non-PTY guest execution work
- Doctor output work and operator-doc drafting
- Manual playbook drafting and smoke-script authoring

## Verification Checkpoints

### Checkpoint 1: selection and readiness
- The system can distinguish:
  - `host_native`
  - ready `guest_rootfs`
  - unready `guest_rootfs`
  - unsupported image

### Checkpoint 2: guest execution
- Commands run against guest userspace instead of the host distro userspace.

### Checkpoint 3: full isolation
- `world_fs.isolation=full` uses guest-root semantics and fails closed when that invariant breaks.

### Checkpoint 4: provisioning
- `substrate world enable --provision-deps` works only for supported ready guest worlds and leaves host package state unchanged.

### Checkpoint 5: operator truthfulness
- Doctor output and docs reflect the actual backend, image, OS identity, and provisioning readiness.

## What This Plan Does Not Do Yet
- It does not break work into task-sized units.
- It does not assign files per task.
- It does not start implementation.

Those belong to Phase 3 `TASKS`.
