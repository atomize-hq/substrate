# Phase 0: Architecture and Host Ownership Model

Status: Proposed

## Purpose / Outcome

Produce the architecture and contracts that make ownership separation concrete before implementation starts. Phase 0 ends when the repo has an agreed host ownership model, daemon responsibility boundary, broker/auth boundary, and breakglass policy for macOS Lima worlds.

## Why This Phase Exists

The current backend is shaped around same-user Lima control. If implementation starts without first freezing the ownership and transport contracts, the project is likely to preserve the existing direct host-to-guest reachability and simply move logic between scripts and crates. Phase 0 prevents that failure mode.

## In-Scope

- Define the host-side owner for Lima state, sockets, keys, and lifecycle.
- Define the contract between developer CLI code and the new daemon.
- Define the single broker endpoint and the authenticated daemon-to-agent boundary.
- Define which current direct-access operations become breakglass only.
- Identify installer, uninstaller, backend, and docs surfaces that must change later.

## Out-of-Scope

- Shipping the daemon, service, or broker implementation.
- Changing guest-side `world-agent` behavior beyond what is needed to specify the control contract.
- Finalizing distribution packaging or update delivery.

## Architectural Approach

Phase 0 treats the current macOS backend as evidence, not as the architecture to preserve. The phase documents a new control path:

- developer CLI talks to a Substrate-owned macOS daemon
- daemon owns Lima host state and lifecycle
- daemon exposes one broker endpoint to the CLI
- daemon talks to the guest agent over a private, non-routine transport
- direct Lima, SSH, guest service control, and guest socket probing are removed from the normal runtime path

## Dependencies / Sequencing

- This phase starts from the current behavior in `crates/world-mac-lima`, `scripts/mac/lima-warm.sh`, and existing macOS docs.
- Milestone 0.1 defines the ownership and daemon contract first.
- Milestone 0.2 defines the broker/auth contract second because it depends on 0.1’s control-plane boundary.
- Phase 1 prototype work cannot start until both milestone SOWs are accepted.

## Concrete Repo Surfaces and File Pointers

- `crates/world-mac-lima/src/lib.rs`
  Current VM ensure, agent readiness, forwarding setup, and endpoint selection that phase 0 must replace conceptually.
- `crates/world-mac-lima/src/forwarding.rs`
  Current direct host forwarding design that phase 0 must explicitly retire from normal operation.
- `scripts/mac/lima-warm.sh`
  Current same-user bootstrap and guest ownership assumptions that phase 1 will rework under the phase 0 contract.
- `scripts/substrate/install-substrate.sh`
  Current provisioning path that phase 0 must reframe around daemon installation and broker setup.
- `scripts/substrate/uninstall-substrate.sh`
  Current teardown path that phase 0 must reframe around daemon-owned state cleanup.
- `docs/WORLD.md`
  Current architecture description that will need a contract delta after implementation.
- `docs/cross-platform/mac_world_setup.md`
  Current operator workflow with direct guest access that phase 0 must mark as incompatible with the new target state.

## Deliverables

- Phase overview README.
- Milestone 0.1 SOW for host owner model and daemon contract.
- Milestone 0.2 SOW for brokered access and authenticated boundary contract.
- A dependency map from current same-user surfaces to future daemon-owned surfaces.

## Acceptance Criteria

- The phase documents a single authoritative answer for who owns `LIMA_HOME`, SSH material, forwarding state, instance metadata, and host sockets.
- The phase documents a single authoritative answer for how the CLI reaches the macOS world in routine operation.
- The phase documents which current commands are removed from normal workflows and under what breakglass conditions they are still allowed.
- The milestone SOWs identify all repo surfaces future implementation must touch first.

## Validation / Evidence Plan

- Review the phase docs against the current code and confirm each existing same-user path has an explicit replacement or deprecation.
- Confirm that no milestone leaves the developer user owning routine Lima control-plane artifacts.
- Confirm that doc reviewers can derive a concrete phase 1 implementation order without reopening core boundary questions.

## Risks / Open Questions

- The daemon install model may affect UX, signing requirements, and whether the initial prototype uses a root daemon or a dedicated service account launched by `launchd`.
- Some Lima internals may still assume user-session conventions that need proof-of-concept work before the contract is final.
- The project needs a crisp definition of what “breakglass” means in code, docs, and operator evidence.

## Milestones

- [Milestone 0.1: Host Owner Model and Daemon Contract](./milestone-0-1-host-owner-model-and-daemon-contract-sow.md)
- [Milestone 0.2: Brokered Access and Auth Boundary Contract](./milestone-0-2-brokered-access-and-auth-boundary-contract-sow.md)
