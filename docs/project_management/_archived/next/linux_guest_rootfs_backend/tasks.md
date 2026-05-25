# Phase 3 Tasks: Linux Guest RootFS Backend and Guest-Only Linux Provisioning

Phase 1 `spec.md` and Phase 2 `plan.md` were loaded and accepted as the authoritative inputs for Phase 3. `ADR-0009` and the decision register were also reviewed as supporting context. No contradiction was found that requires reopening Phase 1 or Phase 2.

- [ ] Task: Introduce the built-in guest image contract for `ubuntu-24.04-amd64`.
  - Acceptance: A single authoritative image-manifest surface defines the built-in image id, pinned OCI reference, expected manifest digest, expected guest OS identity, and expected architecture; unsupported image ids are classified deterministically for later exit-4 handling.
  - Verify: `cargo test -p world -- --nocapture`
  - Files: `crates/world/src/lib.rs`, new `crates/world/src/guest_rootfs/mod.rs`, new `crates/world/src/guest_rootfs/image_manifest.rs`, new `crates/world/tests/guest_rootfs_manifest.rs`

- [ ] Task: Add canonical guest-rootfs storage-path and permission helpers for `/var/lib/substrate/world-rootfs/`.
  - Acceptance: The exact v1 layout is modeled in one place; ownership and mode expectations are explicit; helpers reject workspace or `$SUBSTRATE_HOME` placement; scope resolution primitives can distinguish `global` from workspace-scoped storage.
  - Verify: `cargo test -p world -- --nocapture`
  - Files: new `crates/world/src/guest_rootfs/storage.rs`, new `crates/world/src/guest_rootfs/scope.rs`, `crates/world/src/guest_rootfs/mod.rs`, new `crates/world/tests/guest_rootfs_storage.rs`

- [ ] Task: Implement `scripts/linux/world-rootfs-warm.sh` as the only global warm/repair entrypoint.
  - Acceptance: The script fetches by pinned digest, unpacks OCI layers into the immutable base-image tree, verifies digest plus `/etc/os-release` plus architecture, repairs directory layout and permissions, cleans incomplete `tmp/` artifacts only, preserves healthy overlays, and never pre-creates workspace overlays.
  - Verify: `sudo scripts/linux/world-rootfs-warm.sh`; rerun the same command to confirm idempotence
  - Files: `scripts/linux/world-rootfs-warm.sh`, new `crates/world/src/guest_rootfs/verification.rs`, `crates/world/src/guest_rootfs/image_manifest.rs`, new `crates/world/tests/guest_rootfs_warm_contract.rs`

- [ ] Task: Add lazy scope-local overlay creation and reuse rules.
  - Acceptance: The runtime/provisioning path can derive `global` or `ws-<workspace-sha256>`, create an empty scope-local overlay lazily on first use, reuse the same overlay for later execution and provisioning in that scope, and avoid broadening lazy creation into fetch/repair/permission work.
  - Verify: `cargo test -p world -- --nocapture`
  - Files: new `crates/world/src/guest_rootfs/overlay.rs`, `crates/world/src/guest_rootfs/scope.rs`, `crates/world/src/guest_rootfs/mod.rs`, new `crates/world/tests/guest_rootfs_overlay_scope.rs`

- [ ] Task: Implement the shared guest-rootfs readiness evaluator.
  - Acceptance: One structured readiness result covers configured backend, selected image id, built-in image support, base-image presence, digest verification, `/etc/os-release`, architecture match, and directory/permission health; missing scope-local overlay state alone does not make readiness false.
  - Verify: `cargo test -p world -- --nocapture`
  - Files: new `crates/world/src/guest_rootfs/readiness.rs`, `crates/world/src/guest_rootfs/mod.rs`, `crates/world/src/lib.rs`, new `crates/world/tests/guest_rootfs_readiness.rs`

- [ ] Task: Wire `world.linux.backend` and `world.linux.image` semantics through config resolution and Linux backend selection.
  - Acceptance: `host_native` remains the default; `guest_rootfs` is explicit opt-in; default-image behavior applies only when `guest_rootfs` is selected; setting `world.linux.image` never flips the backend implicitly; unsupported images resolve to deterministic exit-4-ready state.
  - Verify: `cargo test -p shell -- --nocapture`; `cargo test -p world-backend-factory -- --nocapture`
  - Files: `crates/shell/src/execution/config_model.rs`, `crates/world-backend-factory/src/lib.rs`, `crates/world/src/lib.rs`, new `crates/shell/tests/world_linux_backend_config.rs`

- [ ] Task: Add the guest-rootfs non-PTY execution path.
  - Acceptance: Linux non-PTY execution uses the guest userspace when `guest_rootfs` is selected, never falls back silently to `host_native`, consults shared readiness before execution, and surfaces shared remediation when the backend is unready.
  - Verify: `cargo test -p world-service -- --nocapture`
  - Files: `crates/world/src/lib.rs`, `crates/world/src/session.rs`, `crates/world/src/exec.rs`, `crates/world-service/src/service.rs`, new `crates/world-service/tests/guest_rootfs_nonpty.rs`

- [ ] Task: Add the guest-rootfs PTY/stream execution path.
  - Acceptance: PTY execution honors the same backend/image selection and readiness rules as non-PTY execution and runs against the guest userspace rather than the host distro userspace.
  - Verify: `cargo test -p world-service -- --nocapture`
  - Files: `crates/world/src/stream.rs`, `crates/world/src/session.rs`, `crates/world-service/src/pty.rs`, new `crates/world-service/tests/guest_rootfs_pty.rs`

- [ ] Task: Enforce full-isolation guest-root semantics for `world_fs.isolation=full`.
  - Acceptance: Full isolation uses the guest rootfs as the effective system root, host `/usr`, `/etc`, and related system directories are not the effective guest system view, and invariant failure exits fail-closed with code `5`.
  - Verify: `cargo test -p world-service full_isolation_nonpty -- --nocapture`; `cargo test -p world-service full_isolation_pty -- --nocapture`
  - Files: `crates/world/src/isolation.rs`, `crates/world/src/session.rs`, `crates/world-service/src/service.rs`, `crates/world-service/tests/full_isolation_nonpty.rs`, `crates/world-service/tests/full_isolation_pty.rs`

- [ ] Task: Gate `substrate world enable --provision-deps` on guest-rootfs readiness and scope-local overlay semantics.
  - Acceptance: `host_native` continues to fail with exit `4` and no host package mutation; `guest_rootfs` provisioning reuses shared readiness, lazily creates the scope-local overlay when needed, installs only into that overlay, and surfaces unsupported-image or warm-required failures without attempting host mutation.
  - Verify: `cargo test -p shell -- --nocapture`
  - Files: `crates/shell/src/builtins/world_enable/runner.rs`, `crates/shell/src/builtins/world_enable/runner/provision_deps.rs`, `crates/shell/src/builtins/world_deps/surfaces.rs`, new `crates/shell/tests/world_enable_provision_deps_guest_rootfs.rs`

- [ ] Task: Extend `substrate world doctor --json` with the additive guest-rootfs fields.
  - Acceptance: Doctor reports `world.backend.configured`, `world.backend.kind`, `world.backend.ready`, `world.image.id`, `world.os.*`, and `world.provisioning.{supported,ready,reason}` with the exact host-native vs guest-rootfs semantics from the spec and shared readiness as the single source of truth.
  - Verify: `cargo test -p transport-api-types -- --nocapture`; `cargo test -p shell -- --nocapture`; `substrate world doctor --json | jq '.world.backend, .world.image, .world.os, .world.provisioning'`
  - Files: `crates/transport-api-types/src/lib.rs`, `crates/world-service/src/handlers.rs`, `crates/shell/src/execution/platform/linux.rs`, new `crates/shell/tests/doctor_scopes_guest_rootfs.rs`

- [ ] Task: Publish operator-facing config and runtime docs for guest-rootfs.
  - Acceptance: Docs describe the script-first warm flow, `world.linux.backend` / `world.linux.image` behavior, guest-rootfs-only provisioning, doctor semantics, and fail-closed remediation; operator-facing provisioning references remain `substrate world enable --provision-deps`.
  - Verify: `cargo test -p shell world_deps_apt_fail_early_wdap1 -- --nocapture`; `rg -n "substrate world enable --provision-deps" docs/WORLD.md docs/reference/world/deps/README.md docs/reference/config/world.md docs/CONFIGURATION.md`
  - Files: `docs/WORLD.md`, `docs/reference/world/deps/README.md`, `docs/reference/config/world.md`, `docs/CONFIGURATION.md`

- [ ] Task: Add Linux regression, smoke, and manual verification coverage for guest-rootfs.
  - Acceptance: Coverage proves image verification, overlay reuse/separation, readiness consistency, guest execution, full isolation, provisioning persistence, and host-native rejection; manual validation lives in normal doc/smoke surfaces rather than planning-pack artifacts.
  - Verify: `cargo test -p world -- --nocapture`; `cargo test -p world-service -- --nocapture`; `cargo test -p shell -- --nocapture`; `bash docs/project_management/_archived/next/linux_guest_rootfs_backend/smoke/linux-smoke.sh`
  - Files: new `crates/world/tests/guest_rootfs_integration.rs`, new `crates/world-service/tests/guest_rootfs_execution.rs`, new `crates/shell/tests/world_enable_provision_deps_guest_rootfs.rs`, new `docs/project_management/_archived/next/linux_guest_rootfs_backend/smoke/linux-smoke.sh`, new `docs/manual_verification/linux_guest_rootfs.md`

Assumptions carried into this breakdown:
- I assumed the implementation will introduce a dedicated `crates/world/src/guest_rootfs/` module; if the code lands in a different existing Linux-world seam, the ordering still holds.
- The handoff’s missing `manual_testing_playbook.md` and `integration_map.md` references are non-blocking because those stubs were intentionally deleted; manual verification is therefore scoped to ordinary docs/smoke surfaces, not planning-pack artifacts.
