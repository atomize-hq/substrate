# Spec: Linux Guest RootFS Backend and Guest-Only Linux Provisioning

## Assumptions
1. This is the Phase 1 `SPECIFY` artifact for the Linux guest-rootfs feature and remains implementation-free.
2. Linux keeps `host_native` as the default backend in the first ship; `guest_rootfs` is explicit opt-in.
3. The first shipped guest image family is Ubuntu/Debian and the only built-in Linux guest image in v1 is `ubuntu-24.04-amd64`.
4. The warm and repair surface is script-first through `scripts/linux/world-rootfs-warm.sh`, not a new CLI verb in v1.
5. macOS and Windows do not change behavior in this feature; they remain compile/test parity and non-regression platforms only.

## Objective
Build a Linux `guest_rootfs` backend that lets Substrate run world execution against a guest userspace decoupled from the host distro, while preserving fail-closed isolation and enabling explicit guest-only APT provisioning through `substrate world enable --provision-deps` without mutating the Linux host OS.

Success means:
- Linux operators can opt into a guest userspace without using a full VM.
- `world_fs.isolation=full` uses the guest rootfs as the effective system root.
- `substrate world enable --provision-deps` succeeds only inside supported ready guest worlds and never mutates the host OS.

## Tech Stack
- Language: Rust 2021
- Core crates:
  - `crates/shell`
  - `crates/world`
  - `crates/world-service`
  - `crates/world-backend-factory`
- Supporting automation:
  - `scripts/linux/world-rootfs-warm.sh`
- Supporting docs:
  - `docs/WORLD.md`
  - `docs/reference/world/deps/README.md`

## Commands
- Build: `cargo build --workspace`
- Format check: `cargo fmt --all -- --check`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Full tests: `cargo test --workspace -- --nocapture`
- Shell-focused tests: `cargo test -p shell -- --nocapture`
- World backend tests: `cargo test -p world -- --nocapture`
- World-service tests: `cargo test -p world-service -- --nocapture`

## Project Structure
- `src/` -> top-level CLI entrypoints only
- `crates/shell/` -> config resolution, CLI behavior, provisioning UX, remediation
- `crates/world/` -> Linux backend behavior, rootfs lifecycle, isolation, overlays
- `crates/world-service/` -> execution endpoints and doctor/readiness reporting
- `crates/world-backend-factory/` -> backend selection plumbing if routing changes
- `scripts/linux/` -> privileged Linux warm and repair helpers
- `docs/` -> operator and architecture documentation
- `docs/project_management/_archived/next/linux_guest_rootfs_backend/` -> feature-local skill artifacts

## Behavior Contract

### Backend and image semantics
- `world.linux.backend` is required to resolve to exactly one of:
  - `host_native`
  - `guest_rootfs`
- Default value of `world.linux.backend` is `host_native`.
- `world.linux.image` is a stable image-identity string and applies only when `world.linux.backend=guest_rootfs`.
- If `world.linux.backend=host_native`:
  - Linux uses the existing host-native backend.
  - `world.linux.image` is ignored for activation and provisioning.
  - `substrate world enable --provision-deps` MUST fail with exit `4`.
- If `world.linux.backend=guest_rootfs` and `world.linux.image` is unset:
  - Substrate MUST select the built-in default image `ubuntu-24.04-amd64`.
- If `world.linux.backend=guest_rootfs` and `world.linux.image` is set to any value other than `ubuntu-24.04-amd64` in v1:
  - world execution MUST reject activation with exit `4`,
  - provisioning MUST reject activation with exit `4`,
  - doctor MUST report the backend as configured but not ready,
  - the error and doctor reason MUST identify the unsupported image id.
- Setting `world.linux.image` MUST NOT implicitly switch `world.linux.backend` from `host_native` to `guest_rootfs`.

### Blessed image source, provenance, and verification
- The only built-in guest image id in v1 is `ubuntu-24.04-amd64`.
- Its source of truth is a Substrate-owned built-in image manifest that maps image id to:
  - an immutable OCI image reference,
  - an expected OCI manifest digest,
  - expected guest OS identity,
  - expected architecture.
- `scripts/linux/world-rootfs-warm.sh` MUST fetch the image by pinned digest, not by floating tag.
- Warm-up MUST use OCI-style layer unpack semantics to materialize a root filesystem tree.
- Warm-up MUST verify, in order:
  - the fetched OCI manifest digest matches the built-in manifest,
  - the unpacked rootfs contains a valid `/etc/os-release`,
  - `/etc/os-release` identifies an Ubuntu or Debian-family guest matching the built-in manifest,
  - the unpacked rootfs architecture matches the host architecture expected by the built-in manifest.
- If any verification step fails:
  - the base image MUST be treated as unusable,
  - readiness MUST be false,
  - runtime execution and provisioning MUST fail with exit `4`,
  - doctor and remediation MUST surface the verification failure reason.

### Storage layout, ownership, and permissions
- All guest-rootfs assets MUST live under `/var/lib/substrate/world-rootfs/`.
- The exact v1 directory layout is:
  - `global/` MAY be created during warm as generic storage scaffolding.
  - `ws-<workspace-sha256>/` directories are created lazily on first execution or provisioning for that resolved scope.

```text
/var/lib/substrate/world-rootfs/
  images/
    ubuntu-24.04-amd64/
      manifest.json
      digest
      rootfs/
  overlays/
    ubuntu-24.04-amd64/
      global/
        upper/
        work/
        state.json
      ws-<workspace-sha256>/
        upper/
        work/
        state.json
  tmp/
```

- Ownership and permissions are:
  - `/var/lib/substrate/world-rootfs/` -> `root:substrate`, mode `0750`
  - `images/` and `overlays/` -> `root:substrate`, mode `0750`
  - unpacked base-image directories under `images/<image-id>/rootfs/` -> `root:substrate`, directories `0755`, files `0644`, with the tree treated as immutable after verification
  - overlay directories under `overlays/...` -> `root:substrate`, mode `0750`
  - `tmp/` -> `root:substrate`, mode `0750`
- Guest images or overlays MUST NEVER be stored inside a workspace or under `$SUBSTRATE_HOME`.

### Overlay keying, persistence, cleanup, repair, and reset
- The persistence boundary for writable guest state is one overlay per:
  - image id, and
  - execution scope.
- Execution scope is:
  - `ws-<workspace-sha256>` when a workspace root is resolved,
  - `global` when no workspace root is resolved.
- `<workspace-sha256>` is the SHA-256 of the canonical realpath of the workspace root.
- Scope-local overlays are created lazily by runtime execution or `substrate world enable --provision-deps` when that scope is used for the first time.
- `scripts/linux/world-rootfs-warm.sh` MUST NOT require a workspace input and MUST NOT be responsible for pre-creating `ws-<workspace-sha256>` overlays.
- Provisioning and execution within the same scope MUST reuse the same overlay.
- Different workspaces MUST NOT share overlays.
- The immutable base image is shared by all overlays for the same image id.
- Overlay cleanup rules:
  - `scripts/linux/world-rootfs-warm.sh` MAY delete incomplete download or unpack artifacts under `tmp/`,
  - `scripts/linux/world-rootfs-warm.sh` MUST NOT delete healthy overlays,
  - v1 does not perform automatic garbage collection of unused overlays.
- Repair vs reset rules:
  - repair means restoring missing directories, correcting ownership or permissions, re-fetching or re-unpacking a missing or digest-mismatched base image, and preserving existing overlays,
  - reset means explicit operator deletion of an overlay directory before re-running warm or execution,
  - warm MUST perform repair automatically when possible,
  - warm MUST NOT perform reset implicitly.

### Warm-script behavior and privilege posture
- `scripts/linux/world-rootfs-warm.sh` is the only v1 warm and repair entrypoint.
- The script MUST be idempotent:
  - if the built-in image, directory layout, permissions, and generic storage scaffolding are already correct, it exits successfully without changing semantic state,
  - if directories, permissions, or the base-image materialization are missing or damaged, it repairs them in place,
  - it never provisions APT packages,
  - it never deletes healthy overlays,
  - it never pre-creates workspace-scoped overlays.
- The script MUST require explicit privilege for writes under `/var/lib/substrate/world-rootfs/`.
- If invoked as a non-root user:
  - it MUST either re-exec via `sudo` or fail immediately with actionable instructions that `sudo` is required,
  - it MUST NOT partially create root-owned state as an unprivileged user.

### Shared readiness logic
- Substrate MUST compute guest-rootfs readiness through one shared readiness evaluator reused by:
  - `substrate world doctor --json`,
  - `substrate world enable --provision-deps`,
  - runtime execution and runtime remediation.
- The shared readiness evaluator MUST consider:
  - configured backend,
  - configured image id,
  - built-in image support,
  - base-image presence and digest verification,
  - `/etc/os-release` verification,
  - architecture match,
  - required directory layout and permissions.
- The shared readiness evaluator MUST return enough structured state to determine:
  - whether provisioning is supported,
  - whether execution is ready,
  - the exact reason when readiness is false,
  - the correct remediation message when readiness is false.
- Missing scope-local overlay state for an otherwise valid scope MUST NOT make readiness false by itself.
- On first execution or provisioning for a scope:
  - Substrate MUST attempt to create the scope-local overlay lazily,
  - successful creation makes that overlay reusable for later execution and provisioning in the same scope,
  - failure to create the overlay MUST fail the triggering operation with an actionable error.

### `substrate world doctor --json` schema
- `substrate world doctor --json` MUST expose the following additive fields:

```json
{
  "world": {
    "backend": {
      "configured": "host_native | guest_rootfs",
      "kind": "linux_host_native | linux_guest_rootfs",
      "ready": true
    },
    "image": {
      "id": "ubuntu-24.04-amd64 | null"
    },
    "os": {
      "id": "ubuntu | debian | <host-os-id> | null",
      "version_id": "24.04 | <version> | null",
      "pretty_name": "Ubuntu 24.04 LTS | <name> | null",
      "arch": "x86_64 | aarch64 | <arch> | null"
    },
    "provisioning": {
      "supported": true,
      "ready": true,
      "reason": null
    }
  }
}
```

- Field semantics are:
  - `world.backend.configured` -> config-selected backend value before readiness checks
  - `world.backend.kind` -> actual backend class for this platform
  - `world.backend.ready` -> whether the configured backend is ready for runtime execution
  - `world.image.id` -> active built-in guest image id when `guest_rootfs` is configured; `null` for `host_native`
  - `world.os.*` -> guest OS identity for ready `guest_rootfs`, host OS identity for `host_native`, `null` values when `guest_rootfs` is configured but not yet verified
  - `world.provisioning.supported` -> whether provisioning is allowed for the configured backend/image
  - `world.provisioning.ready` -> whether provisioning can run immediately
  - `world.provisioning.reason` -> `null` when ready, otherwise the exact blocking reason reused by provisioning and runtime remediation

### Runtime execution behavior
- When `world.linux.backend=host_native`:
  - Linux world execution keeps the existing host-native behavior.
- When `world.linux.backend=guest_rootfs`:
  - Linux world execution MUST run against the selected guest userspace rather than the host distro userspace,
  - the backend MUST NOT silently fall back to `host_native`,
  - if readiness is false, runtime remediation MUST reuse the shared readiness reason and point to `scripts/linux/world-rootfs-warm.sh` when warm or repair is required.
- When `world.linux.backend=guest_rootfs` and `world_fs.isolation=full` is requested:
  - the guest rootfs MUST be the isolation root,
  - host `/usr`, `/etc`, and related system directories MUST NOT be rebound into the command view as the effective world system directories,
  - if the backend cannot satisfy that guarantee, execution MUST fail closed with exit `5`.

### Provisioning behavior
- The Linux provisioning command is exactly:
  - `substrate world enable --provision-deps`
- When `world.linux.backend=host_native`:
  - behavior remains explicit failure with exit `4`,
  - output MUST state that provisioning is unsupported because it would mutate the host OS,
  - doctor MUST report `world.backend.configured=host_native` and `world.provisioning.supported=false`.
- When `world.linux.backend=guest_rootfs` and readiness is true:
  - provisioning is supported,
  - package-manager support is apt-only in v1,
  - package installation MUST occur inside the persistent guest overlay for the resolved scope only,
  - host OS package state MUST remain unchanged.
- When `world.linux.backend=guest_rootfs` and readiness is false:
  - provisioning MUST fail with exit `4`,
  - the failure MUST reuse the shared readiness reason,
  - remediation MUST point to `scripts/linux/world-rootfs-warm.sh` when warm or repair is the blocking step.

## Code Style
```rust
fn ensure_guest_rootfs_ready(state: &GuestRootfsState) -> anyhow::Result<()> {
    if !state.ready {
        anyhow::bail!("guest_rootfs is selected but the warmed rootfs is unavailable");
    }
    Ok(())
}
```

Conventions:
- Return `Result<T, anyhow::Error>` on fallible paths.
- Add `anyhow::Context` at filesystem, mount, unpack, and process-execution boundaries.
- Keep Linux-specific behavior behind platform seams or `#[cfg(target_os = "linux")]`.
- Keep operator-facing failures singular and actionable.

## Testing Strategy
- Unit tests:
  - config parsing for backend and image selection
  - default-image behavior and unsupported-image failures
  - readiness-state mapping
  - guest-rootfs path safety and storage invariants
  - doctor-field rendering for `host_native`, ready `guest_rootfs`, and unready `guest_rootfs`
- Integration tests:
  - prove `guest_rootfs` runs against guest userspace rather than host distro userspace
  - prove `world_fs.isolation=full` uses guest root semantics
  - prove provisioning persists in guest overlays across later runs in the same scope
  - prove different workspaces do not share overlays
  - prove `host_native` provisioning refuses without invoking host package-manager behavior
- Manual validation:
  - warm and repair flow
  - backend selection and doctor output
  - guest execution on a non-Debian host
  - provisioning persistence and no-host-mutation evidence

## Boundaries
- Always:
  - keep `host_native` as the Linux default in v1
  - fail closed when readiness, image support, or isolation guarantees are missing
  - preserve `substrate world enable --provision-deps` as the Linux provisioning surface
  - reuse one readiness evaluator for doctor, provisioning, and runtime remediation
- Ask first:
  - adding a new public CLI verb for rootfs or image management
  - expanding beyond `ubuntu-24.04-amd64` in this feature
  - changing storage ownership away from `/var/lib/substrate/world-rootfs/`
- Never:
  - mutate the Linux host OS package set through Substrate
  - silently fall back from `guest_rootfs` to `host_native`
  - store guest base images or overlays in the workspace or under `$SUBSTRATE_HOME`

## Success Criteria
- Linux can opt into `world.linux.backend=guest_rootfs` without changing existing default behavior for `host_native`.
- `guest_rootfs` execution proves guest OS identity distinct from the host distro on a non-Debian Linux host.
- `world_fs.isolation=full` uses the guest rootfs as the effective system root and fails closed when that guarantee breaks.
- `substrate world enable --provision-deps` succeeds only for ready `ubuntu-24.04-amd64` guest worlds and installs packages into scope-local guest overlays only.
- Linux host-native provisioning still rejects and leaves host package state untouched.
- `substrate world doctor --json` exposes backend, image, OS identity, and readiness truth that matches runtime and provisioning behavior exactly.

## Open Questions
- None blocking Phase 3 task breakdown.
