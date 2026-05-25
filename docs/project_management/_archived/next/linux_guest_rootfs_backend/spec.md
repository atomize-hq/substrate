# Spec: Linux Guest RootFS Backend and Guest-Only Linux Provisioning

## Assumptions
1. This is the Phase 1 `SPECIFY` artifact for the Linux guest-rootfs feature and should remain implementation-free.
2. Linux keeps `host_native` as the default backend in the first ship; `guest_rootfs` is explicit opt-in.
3. The first shipped guest image family is Ubuntu/Debian and it is the only Linux guest image allowed to support APT provisioning in v1.
4. The warm and repair surface is script-first through `scripts/linux/world-rootfs-warm.sh`, not a new CLI verb in v1.
5. macOS and Windows do not change behavior in this feature; they remain parity and non-regression platforms only.

## Objective
Build a Linux `guest_rootfs` backend that lets Substrate run world execution against a guest userspace decoupled from the host distro, while preserving fail-closed isolation and enabling explicit guest-only APT provisioning without mutating the Linux host OS.

Success means:
- Linux operators can opt into a guest userspace without using a full VM.
- `world_fs.isolation=full` uses the guest rootfs as the effective system root.
- `substrate world deps provision` succeeds only inside supported ready guest worlds and never mutates the host OS.

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
- `docs/project_management/_archived/next/linux_guest_rootfs_backend/` -> feature-local planning artifacts

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
  - invalid-combination and unsupported-image failures
  - readiness-state mapping
  - guest-rootfs path safety and storage invariants
- Integration tests:
  - prove `guest_rootfs` runs against guest userspace rather than host distro userspace
  - prove `world_fs.isolation=full` uses guest root semantics
  - prove provisioning persists in guest overlays across later runs
  - prove host-native provisioning refuses without invoking host package-manager behavior
- Manual validation:
  - warm and repair flow
  - backend selection and doctor output
  - guest execution on a non-Debian host
  - provisioning persistence and no-host-mutation evidence

## Boundaries
- Always:
  - keep `host_native` as the Linux default in v1
  - fail closed when readiness, image support, or isolation guarantees are missing
  - preserve `substrate world deps provision` as the Linux provisioning surface
- Ask first:
  - adding a new public CLI verb for rootfs or image management
  - expanding beyond Ubuntu/Debian guest provisioning in this feature
  - changing storage ownership away from Substrate-managed system paths
- Never:
  - mutate the Linux host OS package set through Substrate
  - silently fall back from `guest_rootfs` to `host_native`
  - store guest base images or overlays in the workspace or under `$SUBSTRATE_HOME`

## Success Criteria
- Linux can opt into `world.linux.backend=guest_rootfs` without changing existing default behavior for `host_native`.
- `guest_rootfs` execution proves guest OS identity distinct from the host distro on a non-Debian Linux host.
- `world_fs.isolation=full` uses the guest rootfs as the effective system root and fails closed when that guarantee breaks.
- `substrate world deps provision` succeeds only for ready Ubuntu/Debian `guest_rootfs` worlds and installs packages into guest overlays only.
- Linux host-native provisioning still rejects and leaves host package state untouched.
- `substrate world doctor --json` exposes enough backend, image, and readiness truth for operators to diagnose configuration and warm-state issues.

## Open Questions
- What is the persistence boundary for guest overlays in v1: per image, per workspace, or per effective world policy profile?
- What exact provenance and verification checks are required for the blessed Ubuntu/Debian OCI-style rootfs before warm-up is considered successful?
