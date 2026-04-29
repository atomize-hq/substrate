# Milestone 1.2: Private Forwarding and Single Broker Endpoint SOW

Status: Proposed

## Purpose / Outcome

Prototype a runtime path where the developer-facing CLI reaches the macOS world only through one Substrate-owned broker endpoint, while daemon-to-guest transport stays private and authenticated. The outcome is removal of current developer-owned host-visible guest forwarding from routine execution.

## Why This Milestone Exists

Today `crates/world-mac-lima/src/forwarding.rs` can create a host socket in `~/.substrate/sock/agent.sock` or use a direct host port for VSock-backed access. That makes the guest agent reachable outside a Substrate-owned broker. Milestone 1.2 exists to close that hole in the prototype.

## In-Scope

- Replace routine direct host-visible guest forwarding with a daemon-private channel.
- Introduce one developer-reachable broker endpoint owned by Substrate.
- Update the macOS backend runtime path so `AgentClient` traffic is broker-mediated rather than pointed at a guest-forwarded socket directly.
- Prototype broker-to-agent authentication or attestation as defined in phase 0.
- Update doctor and health planning so routine evidence flows through the broker.

## Out-of-Scope

- Full redesign of the guest `world-agent` API.
- Final production crypto lifecycle.
- Final breakglass tooling.

## Architectural Approach

The prototype should separate public and private transports:

- Public: one host-local broker endpoint with explicit ownership and ACLs.
- Private: daemon-to-guest transport using VSock, SSH, or another mechanism that is not exposed as a reusable developer-owned endpoint.

The host CLI should stop thinking in terms of direct transport selection. Instead, it should think in terms of contacting the daemon broker and letting the daemon choose or maintain the private transport.

## Dependencies / Sequencing

- Requires phase 0 milestone 0.2 plus the same-user hardening track
  prerequisites named by the parent phase:
  - phase 1 milestone 1.1 transport contract unification
  - phase 1 milestone 1.2 policy application parity
  - phase 1 milestone 1.3 doctor/smoke readiness parity
  - phase 3 milestone 3.2 breakglass reclassification and doc cutover
- Requires milestone 1.1 because the daemon must own the VM state and transport artifacts.
- Implements the boundary specified by phase 0 milestone 0.2.
- Must land before milestone 1.3 so breakglass can be designed around the new routine path.

## Concrete Repo Surfaces and File Pointers

- `crates/world-mac-lima/src/forwarding.rs`
  Replace current `auto_select`, `create_vsock_forwarding`, and `create_ssh_uds_forwarding` assumptions for routine CLI use.
- `crates/world-mac-lima/src/lib.rs`
  Replace direct agent endpoint derivation and local socket/TCP probing with broker endpoint usage.
- `crates/world-mac-lima/src/transport.rs`
  Rework current transport visibility assumptions so VSock or SSH are daemon-private implementation details, not CLI-facing choices.
- `scripts/mac/lima-warm.sh`
  Add any guest bootstrap required for daemon-to-agent trust or private transport support.
- `docs/WORLD.md`
  Prepare to replace the current “host ⇄ guest transport” section that describes direct VSock and SSH forwarding.
- `docs/cross-platform/mac_world_setup.md`
  Prepare to remove direct host socket and guest `curl` instructions from routine workflows.

## Deliverables

- Prototype broker endpoint design and ownership model.
- Prototype private transport design with fallback rules that do not recreate a developer-owned bypass.
- A backend refactor plan showing how `MacLimaBackend` becomes a broker client.
- A diagnostics plan for `substrate host doctor` and `substrate world doctor` in the new path.

## Acceptance Criteria

- Routine macOS execution uses one Substrate-owned broker endpoint, not a host-forwarded guest socket in the developer’s home directory.
- The milestone explicitly retires `~/.substrate/sock/agent.sock` from routine operation.
- The milestone defines how broker-to-agent requests are authenticated or attested in the prototype.
- The milestone preserves a routine validation path for world readiness and health without requiring direct guest socket access by the developer user.

## Validation / Evidence Plan

- Show current `forwarding.rs` paths and identify which are deleted, privatized, or reserved for breakglass.
- Produce a concrete runtime sequence for:
  - CLI command submission
  - broker request handling
  - daemon-to-agent execution
  - result return
- Confirm there is no routine host artifact equivalent to the current developer-owned forwarded socket.
- Confirm diagnostic commands can produce evidence through broker-mediated calls.

## Risks / Open Questions

- Some transport fallback designs are easier to implement but accidentally recreate host-visible escape hatches; those must be rejected even if convenient.
- Guest attestation may need additional state or identity material provisioned during VM creation.
- Existing tests and smoke scripts likely assume direct guest reachability and will need prototype-specific evidence paths.
