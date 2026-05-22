---
date: 2026-05-19T00:00:00-04:00
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

Substrate’s current macOS Lima backend is still not "reachable only through
Substrate." The effective boundary remains the owning macOS user plus any local
process that can reach the forwarded endpoint. At the same time, HEAD now has
real same-user operator surfaces for gateway lifecycle, integrated auth handoff,
doctoring, and runtime metadata. Those landed surfaces are useful evidence and
good prerequisites, but they do not yet create true host-side ownership
separation.

## Repo-backed findings

### 1. Host ownership is same-user, not Substrate-owned

- `world-mac-lima` still resolves the VM name from `SUBSTRATE_LIMA_VM_NAME` or
  `LIMA_VM_NAME`, and resolves the forwarded agent socket under the current
  user’s home as `~/.substrate/sock/agent.sock`:
  `crates/world-mac-lima/src/lib.rs`
- SSH forwarding still resolves `LIMA_HOME` or defaults to `$HOME/.lima`, then
  reads `<instance>/ssh.config` from there:
  `crates/world-mac-lima/src/forwarding.rs`
- The installer, uninstaller, and dev-uninstaller still operate Lima as the
  current user. None introduces a dedicated macOS host `substrate` account:
  `scripts/substrate/install-substrate.sh`,
  `scripts/substrate/uninstall-substrate.sh`, and
  `scripts/substrate/dev-uninstall-substrate.sh`
- `scripts/substrate/world-enable.sh` is also a same-user lifecycle helper
  today. It reuses installer provisioning helpers rather than calling through a
  daemon-owned macOS control plane.

### 2. Direct guest entry is still a supported normal path

- Setup docs still tell operators to build in the guest, install in the guest,
  restart guest `systemd`, probe the guest socket, and inspect guest logs with
  direct guest commands: `docs/cross-platform/mac_world_setup.md`
- `docs/WORLD.md` still records direct guest inspection and forwarding behavior
  as descriptive architecture evidence. That is useful context, but it should
  not be confused with the durable operator/status contract owner.
- `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` still rely on direct
  guest interactions as part of the current same-user verification model.

### 3. Current transport surfaces are broader than "Substrate only"

- Host-visible surfaces still include:
  - UDS: `~/.substrate/sock/agent.sock` via SSH local forwarding
  - loopback TCP via `vsock-proxy`
- Guest-visible surfaces still include:
  - UDS: `/run/substrate.sock`
  - optional TCP via `SUBSTRATE_AGENT_TCP_PORT`
- The agent API itself is still protected primarily by transport reachability:
  whoever can reach the socket or port can talk to the listener:
  `crates/world-service/src/lib.rs`
- This same forwarded transport path now also carries the current gateway
  lifecycle/status traffic on macOS. `substrate world gateway sync`,
  `substrate world gateway status --json`, and `substrate world gateway restart`
  already exist, but on macOS they still ride the same Lima-backed forwarded
  path that ownership separation must eventually privatize:
  `crates/shell/src/builtins/world_gateway.rs`

### 4. Gateway lifecycle and runtime metadata are now real repo surfaces

- `substrate-gateway` is now part of the installed and verified macOS story.
  The macOS installer and dev-install paths verify or stage
  `/usr/local/bin/substrate-gateway` in the Lima guest:
  `scripts/substrate/install-substrate.sh` and
  `scripts/substrate/dev-install-substrate.sh`
- `scripts/mac/smoke.sh` already exercises gateway lifecycle proof on macOS by
  running:
  - `substrate world gateway sync`
  - `substrate world gateway status --json`
  - `substrate world gateway restart`
- Gateway lifecycle/status meaning is no longer hypothetical. The durable
  operator contract already lives in:
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
- Managed gateway runtime artifacts under
  `/run/substrate/substrate-gateway-runtime/` already matter for diagnostics and
  supportability:
  `crates/world-service/src/gateway_runtime.rs`,
  `docs/WORLD.md`, and `docs/INSTALLATION.md`

### 5. Integrated auth handoff is now landed, but not sufficient for ownership separation

- The current integrated handoff no longer lives only in "future auth" design
  language. The durable policy contract now states that the landed carrier is a
  Substrate-owned auth bundle delivered over an inherited FD via
  `SUBSTRATE_LLM_AUTH_BUNDLE_FD`:
  `docs/contracts/substrate-gateway-policy-evaluation.md`
- `world-service` now owns gateway runtime launch and hands the auth bundle to the
  in-world gateway through the managed runtime path:
  `crates/world-service/src/gateway_runtime.rs`
- Gateway status now exposes non-secret lifecycle metadata through the existing
  machine-readable surface, especially `status` and `client_wiring.*`:
  `docs/contracts/substrate-gateway-status-schema.md`
- This is materially better than a purely speculative future design, but it
  still does not solve the core macOS ownership problem. The same developer user
  still owns the Lima control plane, forwarded transport setup, and direct guest
  maintenance tools on the host side.

### 6. The guest identity and mount model are still far from locked down

- The warm flow still creates a guest `substrate` group and adds the login user
  to it: `scripts/mac/lima-warm.sh`
- The agent service still runs as root with `Group=substrate`, not as a
  dedicated non-root service user: `scripts/mac/lima-warm.sh`
- Both base Lima profiles still mount the entire host `$HOME` read-only and the
  active project read-write at `/src`:
  `scripts/mac/lima/substrate.yaml` and
  `scripts/mac/lima/substrate-dev.yaml`
- The warm rewrite still enables `SUBSTRATE_AGENT_TCP_PORT=61337` and broad
  ambient capabilities: `scripts/mac/lima-warm.sh`

### 7. There are parity gaps even before lockdown

- `MacLimaBackend` still injects a synthetic permissive policy snapshot and
  leaves `apply_policy` as a no-op:
  `crates/world-mac-lima/src/lib.rs`
- PTY routing still effectively prefers the Unix-socket path and does not
  present one clean canonical transport contract across VSock, SSH UDS, and
  fallback TCP:
  `crates/shell/src/execution/routing/dispatch/exec.rs`
- Transport constants are still inconsistent in the current code:
  `crates/world-mac-lima/src/forwarding.rs`,
  `crates/world-mac-lima/src/transport.rs`, and
  `crates/world-mac-lima/src/lib.rs`

## External findings

### Lima constraints

- `limactl shell` is SSH-backed, and direct SSH via per-instance config is an
  explicitly supported path: [Lima SSH docs](https://lima-vm.io/docs/usage/ssh/)
- `LIMA_HOME` defaults to `~/.lima`, and Lima control-plane material, SSH
  config, identity, and instance state live there:
  [Lima internals](https://lima-vm.io/docs/dev/internals/),
  [Lima env vars](https://lima-vm.io/docs/config/environment-variables/)
- Lima documents that default user-mode guest IP is not host-reachable by
  design, but host access still remains available through SSH, forwarding, and
  control-plane artifacts:
  [Lima default user-mode network](https://lima-vm.io/docs/config/network/user/)
- `--mount-none` and `--plain` can remove default file-sharing surfaces, but
  `plain` is version-sensitive and should not be treated as a stable security
  guarantee by name alone:
  [Lima create reference](https://lima-vm.io/docs/reference/limactl_create/),
  [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/),
  [Lima port forwarding](https://lima-vm.io/docs/config/port/)
- Current Lima docs remain partially inconsistent on
  `ssh.loadDotSSHPubKeys`; any hardening plan should pin the exact Lima version
  in scope.

### Apple/macOS constraints

- `launchd` supports daemon ownership separation and on-demand socket
  activation:
  [Apple launchd jobs](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html),
  [Apple launch_activate_socket](https://developer.apple.com/documentation/xpc/launch_activate_socket)
- `SMAppService` is the modern Apple-supported registration path for
  LaunchDaemons on macOS 13+:
  [Apple SMAppService](https://developer.apple.com/documentation/servicemanagement/smappservice)
- Apple’s Virtualization.framework shared-directory model still does not create
  a separate host principal; shared-directory access remains bound to host-side
  user permissions:
  [Apple VZVirtioFileSystemDevice](https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice)

## Conclusion

If the target is merely "reduce accidental bypass," Substrate can continue to
improve the current same-user Lima model by removing direct docs/scripts,
tightening mounts, removing extra listeners, and normalizing gateway/runtime
parity.

If the target is "only Substrate can access the VM" against the same logged-in
macOS user, wrapper-only hardening is still insufficient. The repo findings and
external Lima/macOS constraints continue to point to the same answer: the
control-plane owner must not be the developer’s macOS account.

## Recommended target architecture

### Required for true ownership separation

1. Run Lima under a dedicated macOS service account or daemon-owned context not
   controlled by the developer user.
2. Move `LIMA_HOME` into that owner’s private directory.
3. Remove direct user access to Lima SSH config, identities, hostagent state,
   and instance disks.
4. Replace direct host-local guest reachability with a single Substrate-owned
   broker endpoint.
5. Extend ownership separation to the already-landed gateway lifecycle path so
   `substrate world gateway sync|status|restart` no longer depend on same-user
   forwarding.
6. Preserve the landed auth-bundle and status-metadata surfaces, but make them
   flow through a daemon-owned control plane rather than a developer-owned Lima
   path.
7. Treat all direct `limactl shell`, direct SSH, guest `systemctl`, and guest
   `curl` flows as breakglass only.

### Required even in a same-user "hardened but not absolute" mode

1. Remove `SUBSTRATE_AGENT_TCP_PORT=61337` by default.
2. Stop mounting all of `$HOME`; use `--mount-none` or an explicit narrow
   allowlist. Make `/src` the only normal writable ingress.
3. Normalize unit definitions into one source of truth and stop bypassing
   socket-activation ACLs.
4. Add a dedicated guest service user or at least trim capabilities materially.
5. Unify the macOS transport contract and eliminate the current transport drift
   and permissive synthetic policy snapshot.
6. Replace operator-facing direct guest workflows with Substrate-owned commands
   for doctor, logs, repair, cleanup, and gateway lifecycle validation.

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
- Keep the current gateway lifecycle/status contract stable while moving it away
  from same-user forwarding assumptions.

### Phase 2: Reach functional parity

- Apply real backend policy snapshots on macOS.
- Bring PTY/non-PTY transport behavior into one canonical contract.
- Ensure doctor, health, and gateway lifecycle evidence all run through the
  same supported path.

### Phase 3: Add true ownership separation

- Introduce launchd-managed Substrate VM owner.
- Private `LIMA_HOME`.
- Non-user-readable keys/sockets/state.
- Breakglass-only maintenance path.

## Risks and open questions

- The exact Lima version in supported installs still matters for `plain`,
  forwarding, SSH defaults, and any guestagent behavior.
- A dedicated host owner will still require broad repo changes across runtime,
  scripts, tests, docs, and operator flows.
- The repo already has same-user doctor/gateway surfaces. The risk is not
  absence of operator UX; it is that those surfaces still sit on top of the
  wrong host ownership model.
- If the product depends on current broad host file visibility inside the
  guest, mount minimization may still surface feature gaps that need explicit
  ingress and sync design.

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
