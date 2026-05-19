# Milestone 0.1: Host Owner Model and Daemon Contract SOW

Status: Proposed

Last updated: 2026-05-19

## Purpose / Outcome

Specify the authoritative host ownership model for the macOS world and the
contract between the Substrate CLI/backend code and a new host-side daemon. The
outcome is a written implementation contract that removes ambiguity about who
owns VM lifecycle, artifacts, credentials, runtime authority, and the host side
of the current gateway lifecycle path.

## Why This Milestone Exists

Current macOS behavior is anchored in the logged-in developer account.
`MacLimaBackend` starts the VM directly, `scripts/mac/lima-warm.sh` provisions
the instance directly, gateway/client forwarding artifacts are created under the
developer’s home directory, and same-user lifecycle helpers such as
`scripts/substrate/world-enable.sh` and `scripts/substrate/dev-install-substrate.sh`
assume the CLI can directly provision and verify the guest. Without first
changing the ownership contract, later transport hardening would still rest on a
developer-owned substrate.

## In-Scope

- Decide the host-side owner model for Lima instance state, including
  `LIMA_HOME`, instance metadata, SSH config material, and host-reachable
  sockets.
- Define the daemon responsibilities for VM create/start/stop, provisioning,
  health, and transport establishment.
- Define the CLI-to-daemon contract at the level needed for implementation
  planning.
- Define how installer, dev-install, world-enable, and uninstaller flows change
  when the daemon, not the user, owns the control plane.
- Define routine versus breakglass operations for lifecycle and maintenance.
- Define how the current same-user doctor/gateway surfaces remain available
  after ownership changes.

## Out-of-Scope

- Implementing the daemon or its IPC transport.
- Implementing auth/attestation details for daemon-to-agent requests.
- Finalizing product UX text for daemon install prompts or privilege
  escalation.

## Architectural Approach

This milestone should lock the following model:

- A Substrate-owned macOS daemon or service account becomes the only routine
  owner of Lima state.
- `LIMA_HOME` moves out of the developer-owned default location and into
  daemon-owned storage.
- SSH keys, SSH config, forwarded socket files, and instance lifecycle metadata
  move under daemon ownership as well.
- The developer-facing CLI no longer calls `limactl` for routine world
  lifecycle. It issues requests to the daemon instead.
- Existing same-user scripts become one of:
  - daemon-internal implementation details
  - installer/bootstrap tools
  - breakglass-only maintenance tools
- Current world and gateway lifecycle operator commands remain visible to
  operators, but become daemon-mediated rather than same-user Lima-mediated.

The milestone must also state whether the prototype prefers:

- a root-owned `launchd` daemon
- or a dedicated macOS service account whose service is launched and managed by
  `launchd`

If `SMAppService` is considered for a later productized install path, that
should be captured as a packaging option, not allowed to blur the runtime
ownership boundary.

## Dependencies / Sequencing

- Inputs:
  - `crates/world-mac-lima/src/lib.rs`
  - `crates/world-mac-lima/src/forwarding.rs`
  - `scripts/mac/lima-warm.sh`
  - `scripts/substrate/world-enable.sh`
  - `scripts/substrate/dev-install-substrate.sh`
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/uninstall-substrate.sh`
  - `../../../docs/WORLD.md`
  - `../../../docs/contracts/substrate-gateway-operator-contract.md`
  - `../../../docs/contracts/substrate-gateway-policy-evaluation.md`
- Output of this milestone is required before milestone 0.2, because the
  broker/auth boundary depends on the daemon ownership boundary.

## Concrete Repo Surfaces and File Pointers

- `crates/world-mac-lima/src/lib.rs`
  Replace direct `limactl`-driven `ensure_vm_running`, readiness checks, and
  direct forwarding establishment with a future daemon client boundary.
- `crates/world-mac-lima/src/transport.rs`
  Reclassify current transport enumeration as daemon-internal or obsolete for
  routine CLI access.
- `crates/world-mac-lima/src/forwarding.rs`
  Stop treating `~/.substrate/sock/agent.sock` or host-local TCP as
  developer-owned routine endpoints.
- `crates/shell/src/builtins/world_gateway.rs`
  Rebase current gateway lifecycle/status traffic on a daemon-owned host owner
  and endpoint rather than the same-user Lima path.
- `scripts/mac/lima-warm.sh`
  Future split between daemon bootstrap/provisioning logic and breakglass
  repair logic.
- `scripts/mac/lima/substrate.yaml`
  Future ownership-sensitive assumptions around mounts, sockets, and guest
  bootstrap.
- `scripts/substrate/install-substrate.sh`
  Replace "provision macOS world directly" with "install/register/start daemon
  and hand daemon the release root or packaged assets."
- `scripts/substrate/dev-install-substrate.sh`
  Decide whether dev-install stays same-user only, gets an ownership-separated
  mode, or is explicitly excluded from the supported product contract.
- `scripts/substrate/world-enable.sh`
  Reclassify this helper as daemon-triggering or deprecated on macOS.
- `scripts/substrate/uninstall-substrate.sh`
  Replace same-user VM deletion and socket cleanup with daemon-owned teardown
  steps.

## Deliverables

- A written daemon ownership decision with rationale.
- A host artifact ownership table covering:
  - `LIMA_HOME`
  - Lima instance directory
  - SSH keys and known-hosts material
  - host broker socket or listener
  - any host-side metadata needed to surface gateway lifecycle and runtime log
    locations safely
  - any staging/build/cache directories needed for provisioning
- A CLI-to-daemon responsibility table covering:
  - lifecycle
  - health
  - provisioning
  - logging
  - diagnostics
  - gateway lifecycle mediation
  - breakglass entry
- A migration note for existing same-user installs.
- A contract delta list describing what becomes invalid in current macOS docs
  and scripts.

## Acceptance Criteria

- The milestone names one routine owner for all host-side Lima control-plane
  artifacts, and that owner is not the developer user.
- The milestone states that the developer CLI does not directly run `limactl`,
  direct SSH, or host socket forwarding in normal operation.
- The milestone defines a narrow daemon API surface sufficient for phase 1
  prototype implementation.
- The milestone identifies how installation, upgrade, dev-install,
  world-enable, and uninstall flows will transfer or recreate
  ownership-separated state.
- The milestone explicitly marks direct `limactl shell`, guest `systemctl`, and
  guest `curl` as breakglass-only operations.
- The milestone treats `docs/WORLD.md` as descriptive evidence and
  `docs/contracts/` as the durable owner for gateway lifecycle/status and
  policy semantics.

## Validation / Evidence Plan

- Build an artifact inventory from current repo surfaces and map each artifact
  to its future owner.
- Walk the current `MacLimaBackend` startup flow and show where daemon calls
  replace direct host actions.
- Walk the current installer, dev-install, world-enable, and uninstaller logic
  and show how ownership moves from user scripts to daemon-managed operations.
- Review the milestone with the constraint "can the developer user still
  directly control the VM without going through Substrate?" The answer must be
  "not in routine operation."

## Risks / Open Questions

- Root-owned and service-account-owned designs have different operational
  burdens on macOS; the milestone must choose one for the prototype, not defer
  the decision.
- Some Substrate assets currently assume `HOME`-relative paths and may need a
  new daemon state-root abstraction.
- A migration path from `~/.lima/substrate` to daemon-owned state may require
  instance re-creation rather than in-place adoption.
