# Milestone 0.2: Brokered Access and Auth Boundary Contract SOW

Status: Proposed

Last updated: 2026-05-19

## Purpose / Outcome

Specify the runtime access boundary for the ownership-separated macOS world. The
outcome is a contract for one developer-reachable Substrate broker endpoint, a
private daemon-to-agent channel, and an authentication or attestation model
that prevents the developer user from bypassing the broker in normal operation.

## Why This Milestone Exists

Moving Lima state under a daemon is necessary but insufficient. The current
backend still assumes direct host-reachable transports: VSock proxy, SSH UDS
forwarding into `~/.substrate/sock/agent.sock`, and direct guest socket
probing. Those paths are now used for more than command execution: on macOS,
the current `substrate world gateway sync|status|restart` entrypoints also ride
the same forwarded world path. If those paths remain routine, the developer
user can still talk to the guest agent outside the broker boundary. This
milestone exists to remove that gap at the contract level before code is
written.

## In-Scope

- Define the single routine broker endpoint exposed to the CLI and any other
  host-side clients.
- Define how the daemon reaches the guest `world-service` without exposing the
  guest agent directly to the developer user.
- Define authentication and attestation expectations between daemon/broker and
  `world-service`.
- Define how the existing gateway lifecycle/status contract is preserved while
  its transport is rehomed under the daemon.
- Define which current diagnostics and setup flows must be rewritten because
  they use direct guest reachability.
- Define logging and evidence expectations for broker-mediated access.

## Out-of-Scope

- Implementing the broker transport or token machinery.
- Rewriting the `world-service` protocol in detail.
- Finalizing long-term cryptographic material handling if the prototype uses a
  simpler trust bootstrap.

## Architectural Approach

This milestone should lock the following runtime boundary:

- The CLI talks only to a Substrate-owned broker endpoint on the host.
- The host broker endpoint is owned by the daemon and permissioned so the
  intended local user can reach it, but cannot use it to escape the broker
  policy boundary.
- The daemon reaches the guest agent over a private transport that is not left
  exposed as a reusable developer-owned socket or direct shell path.
- The guest agent authenticates the daemon or broker, and the daemon verifies
  that it is talking to the expected guest instance.
- Requests are traced as broker-mediated operations, not as arbitrary local
  guest socket calls.
- The existing gateway lifecycle command family stays intact:
  - `substrate world gateway sync`
  - `substrate world gateway status`
  - `substrate world gateway restart`
  - `substrate world gateway status --json`
- The existing auth and status contracts stay intact:
  - integrated auth remains the landed auth-bundle FD handoff described in
    `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `status --json` remains governed by
    `docs/contracts/substrate-gateway-status-schema.md`

The milestone should also define how the existing transport candidates change:

- VSock may remain viable only if the daemon owns the host endpoint and the CLI
  never reaches the guest socket directly.
- SSH UDS forwarding in the current form should be considered incompatible with
  the target state because it places a host socket under the developer user’s
  home directory.
- Direct SSH shell access is breakglass only.

## Dependencies / Sequencing

- Depends on milestone 0.1’s daemon ownership contract.
- Produces the boundary needed by phase 1 milestone 1.2.
- Must be reviewed against the current docs and diagnostics before phase 1
  prototype work starts.

## Concrete Repo Surfaces and File Pointers

- `crates/world-mac-lima/src/forwarding.rs`
  Current direct VSock and SSH UDS logic to be conceptually replaced by a
  daemon-private forwarding model.
- `crates/world-mac-lima/src/lib.rs`
  Current endpoint creation and direct agent client setup to be replaced by
  broker client behavior.
- `crates/world-mac-lima/src/transport.rs`
  Current host-visible transport selection assumptions that need a new
  classification: private daemon transport versus public broker endpoint.
- `crates/shell/src/builtins/world_gateway.rs`
  Current gateway lifecycle/status client that must keep its stable CLI surface
  while changing its trust and transport boundary.
- `crates/world-service/src/gateway_runtime.rs`
  Current auth-bundle and managed-runtime path that must remain Substrate-owned
  while moving behind the new macOS host owner.
- `scripts/mac/lima-warm.sh`
  Current guest socket activation and service bootstrap that may need broker
  identity bootstrap or daemon-to-guest trust material installation.
- `../../../docs/WORLD.md`
  Current description of host-to-guest transport using VSock and SSH
  forwarding. It is evidence only and must be updated after implementation.
- `../../../docs/contracts/substrate-gateway-operator-contract.md`
  Durable owner of the gateway lifecycle command family.
- `../../../docs/contracts/substrate-gateway-policy-evaluation.md`
  Durable owner of the integrated auth and policy boundary.
- `../../../docs/contracts/substrate-gateway-status-schema.md`
  Durable owner of `substrate world gateway status --json`.
- `../../../docs/contracts/substrate-gateway-runtime-parity.md`
  Durable owner of the typed lifecycle/status runtime parity boundary.
- `../../../docs/cross-platform/mac_world_setup.md`
  Current direct `curl --unix-socket /run/substrate.sock`, `limactl shell`, and
  socket troubleshooting flows that conflict with the new boundary.

## Deliverables

- A broker endpoint contract covering:
  - IPC type
  - ownership and permissions
  - expected request classes
  - failure semantics
- A daemon-to-agent transport contract covering:
  - transport type
  - visibility and ownership
  - guest bootstrap requirements
  - reconnect behavior
- An auth/attestation contract covering:
  - daemon identity presented to the guest
  - guest identity verification by the daemon
  - key or token rotation expectations across VM recreation
- A diagnostics contract stating which checks remain routine and which become
  breakglass.

## Acceptance Criteria

- The milestone defines exactly one routine host entrypoint for macOS world
  execution.
- The milestone explicitly forbids developer-user routine access to a
  host-forwarded guest socket such as `~/.substrate/sock/agent.sock`.
- The milestone preserves the existing `substrate world gateway status --json`
  authority line as the machine-readable gateway wiring surface.
- The milestone defines how broker-to-agent requests are authenticated or
  attested in the prototype.
- The milestone defines how `substrate host doctor`, `substrate world doctor`,
  `substrate health`, and `substrate world gateway status --json` can continue
  to function without requiring direct guest socket reachability by the
  developer user.
- The milestone lists the current docs and scripts whose instructions become
  invalid under the new model.

## Validation / Evidence Plan

- Trace the current direct transport paths from `forwarding.rs` and demonstrate
  how each is either removed, privatized, or reclassified as breakglass.
- Produce a request-flow diagram for:
  - `CLI -> daemon broker -> guest agent`
  - `CLI -> daemon broker -> guest gateway lifecycle path`
- Review the proposed boundary against the constraint "can a user with the
  normal CLI account talk to the guest agent without the broker?" The answer
  must be "no."
- Confirm that planned doctor, health, and gateway commands still have a
  routine evidence path through the broker.

## Risks / Open Questions

- The simplest prototype auth mechanism may not be the final production
  mechanism; the milestone must still define a trust anchor good enough to
  validate the boundary.
- Some debugging workflows will become slower once direct guest socket access
  is removed from normal use.
- If VSock support varies by environment, the private transport design needs a
  fallback that does not recreate a developer-owned socket escape hatch.
