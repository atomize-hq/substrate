# Plan: Linux Guest RootFS Backend and Guest-Only Linux Provisioning

This is the Phase 2 `PLAN` artifact for:
- `docs/project_management/_archived/next/linux_guest_rootfs_backend/spec.md`

## Goal
Turn the approved spec into an implementation approach that is reviewable before task breakdown or code starts. The plan must preserve Linux host safety, keep rollout opt-in, and realize the exact storage, readiness, doctor, and provisioning contract defined in `spec.md`.

## Major Components

### 1. Built-in image manifest and verification
- `crates/world/`
- Responsibilities:
  - define the built-in image id `ubuntu-24.04-amd64`
  - define the pinned OCI reference and digest metadata
  - verify digest, `/etc/os-release`, and architecture after unpack

### 2. Guest-rootfs storage and lifecycle
- `crates/world/`
- `scripts/linux/world-rootfs-warm.sh`
- Responsibilities:
  - create and repair `/var/lib/substrate/world-rootfs/`
  - materialize the immutable base image
  - scaffold generic storage layout without requiring workspace-specific input
  - preserve overlays during repair

### 3. Shared readiness evaluator
- `crates/world/`
- `crates/world-service/`
- `crates/shell/`
- Responsibilities:
  - compute one readiness result reused by doctor, provisioning, and runtime remediation
  - distinguish host-native, ready guest-rootfs, unready guest-rootfs, and unsupported-image states

### 4. Shell contract and config routing
- `crates/shell/`
- `crates/world-backend-factory/`
- Responsibilities:
  - resolve `world.linux.backend`
  - resolve `world.linux.image`
  - apply default-image behavior only for `guest_rootfs`
  - route `substrate world enable --provision-deps`

### 5. World execution path
- `crates/world/`
- `crates/world-service/`
- Responsibilities:
  - bootstrap guest-backed non-PTY and PTY execution
  - enforce fail-closed runtime behavior when guest-rootfs is configured but not ready

### 6. Full-isolation guest-root semantics
- `crates/world/`
- Responsibilities:
  - make the guest rootfs the effective system root for `world_fs.isolation=full`
  - ensure host `/usr`, `/etc`, and similar directories do not act as guest system state

### 7. Doctor schema and reporting
- `crates/world-service/`
- Responsibilities:
  - emit additive `world doctor --json` fields for backend, image id, world OS identity, and provisioning readiness
  - distinguish configured backend from ready or unready state

## Dependencies
- Image manifest and verification rules must exist before warm or readiness logic can be trusted.
- Storage layout and lifecycle rules must exist before shell routing can reliably distinguish ready from unready guest-rootfs.
- Shared readiness logic must exist before doctor, provisioning, and runtime remediation can stay consistent.
- Shell/backend selection must be fixed before world execution can safely branch between host-native and guest-rootfs.
- World execution must exist before full-isolation re-rooting can be validated meaningfully.
- Full-isolation guest-root behavior must be trustworthy before provisioning is enabled.

## Implementation Order

### Step 1: Lock the built-in image contract
- Define the built-in `ubuntu-24.04-amd64` image manifest shape.
- Define pinned OCI reference, digest, expected OS identity, and expected architecture.
- Define post-unpack verification rules.

Why first:
- warm, readiness, and doctor reporting all depend on a single authoritative image contract.

### Step 2: Implement `/var/lib/substrate/world-rootfs/` layout and warm behavior
- Create the exact directory layout from the spec.
- Enforce `root:substrate` ownership and required permissions.
- Implement idempotent warm behavior:
  - no-op when healthy
  - repair missing or damaged image/layout state
  - preserve healthy overlays
  - clean incomplete `tmp/` artifacts only
  - avoid pre-creating workspace-scoped overlays
- Implement explicit privilege posture through `sudo` or immediate failure.

Why second:
- selection and runtime cannot depend on guest-rootfs until the storage model is real.

### Step 3: Implement overlay keying and scope resolution
- Resolve workspace root canonical path.
- Derive `ws-<workspace-sha256>` for workspace-scoped overlays.
- Use `global` when no workspace root exists.
- Create scope-local overlays lazily on first execution or provisioning for that scope.
- Reuse the same overlay for execution and provisioning within one scope.

Why third:
- persistence, no-cross-workspace contamination, and provisioning semantics depend on exact scope keying.

### Step 4: Implement the shared readiness evaluator
- Evaluate:
  - configured backend
  - selected image id
  - built-in image support
  - base-image presence and verification
  - `/etc/os-release` verification
  - architecture match
  - permissions and directory layout
- Return structured readiness state reused verbatim by:
  - doctor
  - provisioning
  - runtime remediation
- Do not fail readiness solely because a scope-local overlay has not yet been created.

Why fourth:
- this is the contract seam that keeps operator reporting and runtime behavior aligned.

### Step 5: Wire backend and image semantics through shell and backend factory
- Parse `world.linux.backend`.
- Parse `world.linux.image`.
- Apply default-image behavior only when backend is `guest_rootfs`.
- Ignore image activation for `host_native`.
- Reject unsupported images deterministically.

Why fifth:
- only after readiness exists can the shell route backend behavior without inventing duplicate logic.

### Step 6: Introduce guest-rootfs execution
- Add guest-backed non-PTY execution.
- Add guest-backed PTY execution.
- Refuse execution with shared readiness reasons when guest-rootfs is configured but unavailable.

Why sixth:
- guest execution must exist before isolation or provisioning can be proven against the real backend.

### Step 7: Re-root full isolation onto the guest rootfs
- Make `world_fs.isolation=full` use the guest rootfs as the effective system root.
- Remove dependence on host system directories as the guest system view.
- Fail closed with exit `5` when invariants break.

Why seventh:
- isolation is the core safety property that makes guest-rootfs materially different from host-native execution.

### Step 8: Enable `substrate world enable --provision-deps`
- Gate provisioning on:
  - `guest_rootfs` selected
  - built-in image supported
  - shared readiness true
- Install packages into the scope-local overlay only.
- Keep host-native refusal explicit and unchanged.

Why eighth:
- provisioning is valid only after storage, execution, and isolation guarantees are real.

### Step 9: Finish doctor output, docs, and validation flow
- Emit the additive doctor fields from the spec.
- Update docs to match warm, readiness, and provisioning behavior exactly.
- Add manual validation and smoke coverage for repair, readiness, execution, isolation, and provisioning persistence.

Why last:
- docs and validation must describe the final runtime contract, not an earlier design draft.

## Risks and Mitigation

### Risk: image verification and runtime behavior drift apart
- Mitigation:
  - define the built-in image manifest first
  - make warm and readiness use the same verification rules

### Risk: directory ownership or permissions become inconsistent across repair paths
- Mitigation:
  - keep one canonical layout under `/var/lib/substrate/world-rootfs/`
  - make warm the only repair entrypoint
  - test repair idempotence explicitly

### Risk: overlays contaminate each other across workspaces
- Mitigation:
  - key overlays by image id plus scope
  - hash canonical workspace roots
  - test cross-workspace separation explicitly

### Risk: doctor, runtime, and provisioning disagree about readiness
- Mitigation:
  - implement one shared readiness evaluator
  - forbid duplicate probing logic in shell or doctor code paths

### Risk: full isolation still leaks host system assumptions
- Mitigation:
  - validate guest `/etc/os-release`
  - inspect mount and root layout in integration tests
  - treat invariant failure as exit `5`

### Risk: Linux-specific work regresses other platforms
- Mitigation:
  - preserve platform seams
  - keep macOS and Windows in compile/test parity coverage

## Parallel vs Sequential Work

### Must stay sequential
- Built-in image contract before storage implementation
- Storage implementation before overlay scope keying
- Overlay scope keying before shared readiness
- Shared readiness before shell routing and doctor schema
- Shell routing before guest execution
- Guest execution before full-isolation re-rooting
- Full-isolation re-rooting before provisioning enablement

### Can run in parallel after prerequisites exist
- Shell config tests and doctor-schema rendering after shared readiness exists
- PTY and non-PTY execution work after backend bootstrap semantics are stable
- Warm-script repair tests and storage-permission tests after storage layout is implemented
- Manual playbook drafting and smoke-script authoring after readiness and provisioning semantics are fixed

## Verification Checkpoints

### Checkpoint 1: image and storage contract
- The built-in image manifest, verification rules, directory layout, ownership, and permissions behave exactly as specified.

### Checkpoint 2: scope-local persistence
- Execution and provisioning reuse one overlay within a scope, and different workspaces do not share overlays.

### Checkpoint 3: readiness consistency
- Doctor, runtime remediation, and provisioning report the same readiness status and reason for the same configuration.

### Checkpoint 4: guest execution
- Commands run against guest userspace instead of the host distro userspace.

### Checkpoint 5: full isolation
- `world_fs.isolation=full` uses guest-root semantics and fails closed when that invariant breaks.

### Checkpoint 6: provisioning
- `substrate world enable --provision-deps` works only for supported ready guest worlds and leaves host package state unchanged.

### Checkpoint 7: operator truthfulness
- Doctor output and docs reflect configured backend, active backend kind, image id, OS identity, and provisioning readiness exactly.

## Phase 3 Readiness
- No blocking design questions remain in the spec.
- Phase 3 can break the work into task-sized units directly from this plan.
