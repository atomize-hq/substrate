---
date: 2026-04-28T00:00:00-04:00
type: external-research
topic: "macOS Lima parity and lockdown"
focus: general
sources:
  - repo
  - lima-docs
  - apple-docs
status: complete
---

# Research: macOS Lima parity and lockdown

## Summary

Substrate’s current macOS Lima backend is not “reachable only through Substrate.” The effective boundary is the owning macOS user plus any local process that can reach the forwarded endpoint. The strongest viable path to Linux-like ownership is not a wrapper around `limactl`; it is host-side ownership separation plus transport and workflow hardening.

## Repo-backed findings

### 1. Host ownership is same-user, not Substrate-owned

- `world-mac-lima` resolves the VM name from `SUBSTRATE_LIMA_VM_NAME` or `LIMA_VM_NAME`, and resolves the forwarded agent socket under the current user’s home as `~/.substrate/sock/agent.sock`: [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:42)
- SSH forwarding resolves `LIMA_HOME` or defaults to `$HOME/.lima`, then reads `<instance>/ssh.config` from there: [crates/world-mac-lima/src/forwarding.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs:376)
- The installer, uninstaller, and dev-uninstaller all operate Lima as the current user. None introduces a dedicated macOS host `substrate` account: [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:1854), [scripts/substrate/uninstall-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/uninstall-substrate.sh:453), [scripts/substrate/dev-uninstall-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-uninstall-substrate.sh:628)

### 2. Direct guest entry is a supported normal path

- Setup docs tell operators to build in the guest, install in the guest, restart guest `systemd`, probe the guest socket, and inspect guest logs with direct guest commands: [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:82), [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:105), [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:119), [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:146)
- Core docs continue that pattern for capability inspection, logs, and cleanup: [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:139), [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:148), [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:370)
- Doctor and smoke scripts validate by shelling into the guest directly: [scripts/mac/lima-doctor.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh:62), [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:155)

### 3. Current transport surfaces are broader than “Substrate only”

- Host-visible surfaces:
  - UDS: `~/.substrate/sock/agent.sock` via SSH local forwarding: [crates/world-mac-lima/src/forwarding.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs:186)
  - Loopback TCP: `127.0.0.1:17788` via `vsock-proxy`: [crates/world-mac-lima/src/forwarding.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs:149)
- Guest-visible surfaces:
  - UDS: `/run/substrate.sock`: [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs:47)
  - Optional TCP: `127.0.0.1:$SUBSTRATE_AGENT_TCP_PORT`: [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs:151)
- The agent API itself is unauthenticated at the listener layer; access control is effectively “who can reach the socket/port”: [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs:252)

### 4. The guest identity and mount model are far from locked down

- The warm flow creates a guest `substrate` group and adds the login user to it: [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:263)
- The agent service runs as root with `Group=substrate`, not as a dedicated non-root service user: [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:665)
- Both base Lima profiles mount the entire host `$HOME` read-only and the active project read-write at `/src`: [scripts/mac/lima/substrate.yaml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/substrate.yaml:16), [scripts/mac/lima/substrate-dev.yaml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/substrate-dev.yaml:19)
- The warm rewrite also enables `SUBSTRATE_AGENT_TCP_PORT=61337` and broad ambient capabilities: [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:671), [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:687)

### 5. There are parity gaps even before lockdown

- `MacLimaBackend` still injects a synthetic permissive policy snapshot and leaves `apply_policy` as a no-op: [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:274), [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:463)
- PTY routing still effectively prefers the Unix-socket path and does not present one clean canonical transport contract across VSock, SSH UDS, and fallback TCP: [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs:699)
- Transport constants are inconsistent (`17788` in forwarding code vs `7788` in some transport code): [crates/world-mac-lima/src/forwarding.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs:149), [crates/world-mac-lima/src/transport.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs:53), [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:205)

## External findings

### Lima constraints

- `limactl shell` is SSH-backed, and direct SSH via per-instance config is an explicitly supported path: [Lima SSH docs](https://lima-vm.io/docs/usage/ssh/)
- `LIMA_HOME` defaults to `~/.lima`, and Lima control-plane material, SSH config, identity, and instance state live there: [Lima internals](https://lima-vm.io/docs/dev/internals/), [Lima env vars](https://lima-vm.io/docs/config/environment-variables/)
- Lima documents that default user-mode guest IP is not host-reachable by design, but host access remains available through SSH, forwarding, and control-plane artifacts: [Lima default user-mode network](https://lima-vm.io/docs/config/network/user/)
- `--mount-none` and `--plain` can remove default file-sharing surfaces, but `plain` is version-sensitive and should not be treated as a stable security guarantee by name alone: [Lima create reference](https://lima-vm.io/docs/reference/limactl_create/), [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/), [Lima port forwarding](https://lima-vm.io/docs/config/port/)
- Current Lima docs remain partially inconsistent on `ssh.loadDotSSHPubKeys`; current release docs and templates indicate newer defaults than some internals text. Any hardening plan should pin the exact Lima version in scope.

### Apple/macOS constraints

- `launchd` supports daemon ownership separation and on-demand socket activation using daemon plist ownership fields and socket registration: [Apple launchd jobs](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html), [Apple launch_activate_socket](https://developer.apple.com/documentation/xpc/launch_activate_socket)
- `SMAppService` is the modern Apple-supported registration path for LaunchDaemons on macOS 13+: [Apple SMAppService](https://developer.apple.com/documentation/servicemanagement/smappservice)
- Apple’s Virtualization.framework shared-directory model does not create a separate host principal; shared-directory access is still bound to host-side user permissions. This reinforces that same-user Lima ownership is not a meaningful same-user security boundary: [Apple VZVirtioFileSystemDevice](https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice)

## Conclusion

If the target is merely “reduce accidental bypass,” Substrate can improve the current same-user Lima model by removing direct docs/scripts, tightening mounts, removing extra listeners, and authenticating the agent path.

If the target is “only Substrate can access the VM” against the same logged-in macOS user, wrapper-only hardening is insufficient. The repo findings and external Lima/macOS constraints both point to the same answer: the control-plane owner must not be the developer’s macOS account.

## Recommended target architecture

### Required for true ownership separation

1. Run Lima under a dedicated macOS service account or daemon-owned context not controlled by the developer user.
2. Move `LIMA_HOME` into that owner’s private directory.
3. Remove direct user access to Lima SSH config, identities, hostagent state, and instance disks.
4. Replace direct host-local transport reachability with a single Substrate-owned broker endpoint.
5. Add request authentication/attestation on the broker-to-agent path; do not rely on socket reachability alone.
6. Treat all direct `limactl shell`, direct SSH, guest `systemctl`, and guest `curl` flows as breakglass only.

### Required even in a same-user “hardened but not absolute” mode

1. Remove `SUBSTRATE_AGENT_TCP_PORT=61337` by default.
2. Stop mounting all of `$HOME`; use `--mount-none` or an explicit narrow allowlist. Make `/src` the only normal writable ingress.
3. Normalize unit definitions into one source of truth and stop bypassing socket-activation ACLs.
4. Add a dedicated guest service user or at least trim capabilities materially.
5. Unify the macOS transport contract and eliminate the current `17788`/`7788` inconsistency and the permissive synthetic policy snapshot.
6. Replace operator-facing direct guest workflows with Substrate-owned commands for doctor, logs, repair, cleanup, and smoke validation.

## Practical sequencing

### Phase 0: Clarify the security bar

- Decide explicitly whether the requirement is:
  - `A.` reduce accidental/operator bypass
  - `B.` resist same-user bypass on the host

If the answer is `B`, account separation is mandatory.

### Phase 1: Close obvious exposure gaps

- Remove extra TCP listener.
- Narrow mounts.
- Fix unit/socket-activation drift.
- Unify transport constants and readiness checks.
- Stop normal-path direct guest docs and scripts.

### Phase 2: Reach functional parity

- Apply real backend policy snapshots on macOS.
- Bring PTY/non-PTY transport behavior into one canonical contract.
- Add authenticated brokered access instead of reachability-only listener access.

### Phase 3: Add true ownership separation

- Introduce launchd-managed Substrate VM owner.
- Private `LIMA_HOME`.
- Non-user-readable keys/sockets/state.
- Breakglass-only maintenance path.

## Risks and open questions

- The exact Lima version in supported installs matters for `plain`, forwarding, SSH defaults, and any guestagent behavior.
- A dedicated host owner will require broad repo changes across runtime, scripts, tests, docs, and operator flows.
- If the product depends on current broad host file visibility inside the guest, mount minimization may surface feature gaps that need explicit ingress/sync design.

## Sources

- Repo files cited inline above.
- Lima:
  - [SSH](https://lima-vm.io/docs/usage/ssh/)
  - [Internal data structure](https://lima-vm.io/docs/dev/internals/)
  - [Environment variables](https://lima-vm.io/docs/config/environment-variables/)
  - [Filesystem mounts](https://lima-vm.io/docs/config/mount/)
  - [Default user-mode network](https://lima-vm.io/docs/config/network/user/)
  - [VMNet networks](https://lima-vm.io/docs/config/network/vmnet/)
  - [user-v2 network](https://lima-vm.io/docs/config/network/user-v2/)
  - [Port forwarding](https://lima-vm.io/docs/config/port/)
  - [create](https://lima-vm.io/docs/reference/limactl_create/)
  - [breaking changes](https://lima-vm.io/docs/releases/breaking/)
- Apple:
  - [SMAppService](https://developer.apple.com/documentation/servicemanagement/smappservice)
  - [launch_activate_socket](https://developer.apple.com/documentation/xpc/launch_activate_socket)
  - [Creating Launch Daemons and Agents](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html)
  - [VZVirtioFileSystemDevice](https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice)
