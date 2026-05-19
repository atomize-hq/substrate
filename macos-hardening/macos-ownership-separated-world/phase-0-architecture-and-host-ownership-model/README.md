# Phase 0: Architecture and Host Ownership Model

Status: Proposed

Last updated: 2026-05-19

## Purpose / Outcome

Produce the architecture and contracts that make ownership separation concrete
before implementation starts. Phase 0 ends when the repo has an agreed host
ownership model, daemon responsibility boundary, broker/auth boundary, gateway
lifecycle/status owner line, and breakglass policy for macOS Lima worlds.

## Why This Phase Exists

The current backend is shaped around same-user Lima control. HEAD now also has
same-user world/gateway lifecycle surfaces that operators can actually use:
`substrate host doctor`, `substrate world doctor`, `substrate health`, and
`substrate world gateway sync|status|restart`. If implementation starts without
first freezing the ownership and transport contracts, the project is likely to
preserve those same-user assumptions and simply move logic between scripts and
crates. Phase 0 prevents that failure mode.

## In-Scope

- Define the host-side owner for Lima state, sockets, keys, and lifecycle.
- Define the contract between developer CLI code and the new daemon.
- Define the single broker endpoint and the authenticated daemon-to-agent
  boundary.
- Define how the existing gateway lifecycle/status surface survives under the
  new ownership model.
- Define which current direct-access operations become breakglass only.
- Identify installer, uninstaller, backend, contract-doc, and macOS docs
  surfaces that must change later.

## Out-of-Scope

- Shipping the daemon, service, or broker implementation.
- Changing guest-side `world-agent` behavior beyond what is needed to specify
  the control contract.
- Finalizing distribution packaging or update delivery.

## Architectural Approach

Phase 0 treats the current macOS backend as evidence, not as the architecture
to preserve. The phase documents a new control path:

- developer CLI talks to a Substrate-owned macOS daemon
- daemon owns Lima host state and lifecycle
- daemon exposes one broker endpoint to the CLI
- daemon talks to the guest agent over a private, non-routine transport
- gateway lifecycle/status commands continue to exist, but run through the same
  daemon-owned path
- direct Lima, SSH, guest service control, and guest socket probing are removed
  from the normal runtime path

## Dependencies / Sequencing

- This phase starts from the current behavior in `crates/world-mac-lima`,
  `scripts/mac/lima-warm.sh`, `crates/shell/src/builtins/world_gateway.rs`, and
  the existing macOS docs/contracts.
- Milestone 0.1 defines the ownership and daemon contract first.
- Milestone 0.2 defines the broker/auth contract second because it depends on
  0.1’s control-plane boundary and on the existing gateway contract owner docs.
- Phase 1 prototype work cannot start until both milestone SOWs are accepted.

## Concrete Repo Surfaces and File Pointers

- `crates/world-mac-lima/src/lib.rs`
  Current VM ensure, agent readiness, forwarding setup, and endpoint selection
  that phase 0 must replace conceptually.
- `crates/world-mac-lima/src/forwarding.rs`
  Current direct host forwarding design that phase 0 must explicitly retire
  from normal operation.
- `crates/shell/src/builtins/world_gateway.rs`
  Current same-user gateway lifecycle/status client path that phase 0 must
  preserve semantically while replacing operationally.
- `crates/world-agent/src/gateway_runtime.rs`
  Current managed gateway runtime and auth-bundle launch model that future
  ownership separation must route through a new host owner.
- `scripts/mac/lima-warm.sh`
  Current same-user bootstrap and guest ownership assumptions that phase 1 will
  rework under the phase 0 contract.
- `scripts/substrate/world-enable.sh`
  Current world provisioning helper that phase 0 must reframe around daemon
  ownership or deprecate on macOS.
- `scripts/substrate/dev-install-substrate.sh`
  Current dev lifecycle helper that already stages `substrate-gateway` and
  writes world metadata in a same-user model.
- `scripts/substrate/install-substrate.sh`
  Current provisioning path that phase 0 must reframe around daemon
  installation and broker setup.
- `scripts/substrate/uninstall-substrate.sh`
  Current teardown path that phase 0 must reframe around daemon-owned state
  cleanup.
- `../../../docs/WORLD.md`
  Current descriptive architecture evidence that will need a contract delta
  after implementation.
- `../../../docs/contracts/substrate-gateway-operator-contract.md`
  Durable owner of the gateway lifecycle command family.
- `../../../docs/contracts/substrate-gateway-policy-evaluation.md`
  Durable owner of the auth and policy boundary.
- `../../../docs/contracts/substrate-gateway-status-schema.md`
  Durable owner of `status --json`.

## Deliverables

- Phase overview README.
- Milestone 0.1 SOW for host owner model and daemon contract.
- Milestone 0.2 SOW for brokered access and authenticated boundary contract.
- A dependency map from current same-user surfaces to future daemon-owned
  surfaces.
- An owner-line map that distinguishes descriptive architecture docs from
  durable operator/status contract docs.

## Acceptance Criteria

- The phase documents a single authoritative answer for who owns `LIMA_HOME`,
  SSH material, forwarding state, instance metadata, and host sockets.
- The phase documents a single authoritative answer for how the CLI reaches the
  macOS world in routine operation.
- The phase documents how current doctor/health/gateway lifecycle commands keep
  working without preserving same-user host ownership.
- The phase documents which current commands are removed from normal workflows
  and under what breakglass conditions they are still allowed.
- The milestone SOWs identify all repo surfaces future implementation must
  touch first.

## Validation / Evidence Plan

- Review the phase docs against the current code and confirm each existing
  same-user path has an explicit replacement or deprecation.
- Confirm that no milestone leaves the developer user owning routine Lima
  control-plane artifacts.
- Confirm that `docs/WORLD.md` is treated as descriptive evidence and that
  gateway operator/status semantics are sourced from `docs/contracts/`.
- Confirm that doc reviewers can derive a concrete phase 1 implementation order
  without reopening core boundary questions.

## Risks / Open Questions

- The daemon install model may affect UX, signing requirements, and whether the
  initial prototype uses a root daemon or a dedicated service account launched
  by `launchd`.
- Some Lima internals may still assume user-session conventions that need
  proof-of-concept work before the contract is final.
- The project needs a crisp definition of what "breakglass" means in code,
  docs, and operator evidence.

## Milestones

- [Milestone 0.1: Host Owner Model and Daemon Contract](./milestone-0-1-host-owner-model-and-daemon-contract-sow.md)
- [Milestone 0.2: Brokered Access and Auth Boundary Contract](./milestone-0-2-brokered-access-and-auth-boundary-contract-sow.md)
