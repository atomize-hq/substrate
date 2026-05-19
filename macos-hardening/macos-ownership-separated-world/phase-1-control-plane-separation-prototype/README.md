# Phase 1: Control Plane Separation Prototype

Status: Proposed

Last updated: 2026-05-19

## Purpose / Outcome

Build a working prototype of the ownership-separated macOS world control plane.
Phase 1 ends when Substrate can boot and use a Lima-backed macOS world through a
daemon-owned control plane, a single broker endpoint, and a breakglass channel
that is no longer part of routine execution.

## Why This Phase Exists

Phase 0 settles the architecture. Phase 1 proves the architecture can actually
replace current same-user Lima behavior in this repo. The prototype should be
strong enough to invalidate the current assumptions embedded in
`MacLimaBackend`, `forwarding.rs`, `lima-warm.sh`, the gateway lifecycle client
path, and the macOS operator docs.

## In-Scope

- Establish daemon-owned Lima state and instance lifecycle.
- Replace direct host-visible forwarding with a private forwarding model and
  one routine broker endpoint.
- Carry current execution, doctor, and gateway lifecycle traffic over that new
  path.
- Add a breakglass maintenance channel that is explicit, gated, and auditable.
- Update the implementation and validation path enough to demonstrate end-to-end
  ownership separation.

## Out-of-Scope

- Final consumer packaging, signing, or notarization.
- Full migration automation for all existing user-owned installs.
- Final doc rewrite across the entire repo beyond the prototype evidence path.

## Architectural Approach

Phase 1 follows the contracts from phase 0 and lands in three milestones:

1. daemon-owned `LIMA_HOME` and instance lifecycle
2. private forwarding plus a single broker endpoint
3. breakglass maintenance channel

The prototype should prefer minimal irreversible change in the Linux guest while
making the host-side boundary unmistakable. It should preserve the current
operator command family and move it under the new owner:

- `substrate host doctor`
- `substrate world doctor`
- `substrate health`
- `substrate world gateway sync|status|restart`

## Dependencies / Sequencing

- This phase assumes the same-user hardening track has already clarified the
  canonical transport contract and breakglass vocabulary, so the prototype does
  not encode the current same-user drift as part of the new daemon-owned path.
- More concretely, phase 1 should not start until the same-user track has
  completed:
  - phase 1 milestone 1.1 transport contract unification
  - phase 1 milestone 1.2 policy application parity
  - phase 1 milestone 1.3 doctor/smoke readiness parity
  - phase 3 milestone 3.2 breakglass reclassification and doc cutover
- Requires phase 0 milestone 0.1 and 0.2 acceptance.
- Milestone 1.1 establishes ownership-separated host state first.
- Milestone 1.2 depends on 1.1 and moves runtime traffic to the broker.
- Milestone 1.3 depends on 1.1 and 1.2 and reinstates only the maintenance
  access required for diagnosis and recovery.

## Concrete Repo Surfaces and File Pointers

- `crates/world-mac-lima/src/lib.rs`
  Primary host-side runtime path to redirect from direct VM and agent
  management to daemon IPC.
- `crates/world-mac-lima/src/forwarding.rs`
  Primary forwarding code to replace or quarantine behind daemon-private logic.
- `crates/world-mac-lima/src/transport.rs`
  Transport classification and endpoint selection surfaces to split into
  broker-public versus daemon-private paths.
- `crates/shell/src/builtins/world_gateway.rs`
  Current macOS gateway lifecycle/status client that phase 1 must rehome behind
  the new broker boundary.
- `crates/world-agent/src/gateway_runtime.rs`
  Current guest-owned managed runtime and auth-bundle seam that phase 1 must
  reach through the new host owner.
- `scripts/mac/lima-warm.sh`
  Provisioning entrypoint that must either become daemon-internal or be
  replaced by daemon bootstrap hooks.
- `scripts/mac/lima/substrate.yaml`
  Guest definition and unit layout that may need trust-bootstrap or
  provisioning changes.
- `scripts/mac/smoke.sh`
  Prototype proof harness that already exercises gateway lifecycle and must be
  updated to validate the new path.
- `scripts/substrate/install-substrate.sh`
  Install path that must provision the daemon-owned control plane, not same-user
  Lima.
- `scripts/substrate/uninstall-substrate.sh`
  Teardown path that must remove daemon-owned state safely.
- `../../../docs/WORLD.md`
  Architecture evidence that must eventually describe the new runtime path and
  breakglass classification.
- `../../../docs/contracts/substrate-gateway-operator-contract.md`
  Durable gateway lifecycle command contract that phase 1 must preserve.

## Deliverables

- Phase overview README.
- Milestone 1.1, 1.2, and 1.3 SOWs.
- A prototype sequencing plan that ties implementation work back to the
  concrete repo surfaces above.
- A phase 1 closeout artifact defining the stable macOS control-plane
  registration mechanism that phase 2 installer and migration work integrates
  into the shipped lifecycle.

## Acceptance Criteria

- The phase defines a prototype that is meaningfully different from same-user
  Lima and not just a wrapper around it.
- Each milestone has concrete deliverables, acceptance gates, and evidence
  requirements tied to current implementation surfaces.
- The breakglass path is explicitly narrower than current routine workflows.
- Prototype validation shows that gateway lifecycle traffic uses the same new
  broker boundary rather than a leftover same-user forwarding seam.

## Validation / Evidence Plan

- Prototype validation must show developer-visible runtime access only through
  the broker endpoint.
- Prototype validation must show daemon-owned state paths on the host.
- Prototype validation must show breakglass commands are disabled from normal
  runtime flow and require explicit operator action.
- Validation evidence should include:
  - `substrate host doctor --json`
  - `substrate world doctor --json`
  - `substrate health --json`
  - `substrate world gateway status --json`

## Risks / Open Questions

- The prototype may expose integration gaps in installer UX and daemon
  packaging that need a follow-on phase.
- The daemon-private transport may require extra guest bootstrap state not
  currently provisioned.
- Existing diagnostic scripts may need temporary compatibility shims during the
  prototype window.

## Milestones

- [Milestone 1.1: Daemon-Owned Lima Home and Instance Lifecycle](./milestone-1-1-daemon-owned-lima-home-and-instance-lifecycle-sow.md)
- [Milestone 1.2: Private Forwarding and Single Broker Endpoint](./milestone-1-2-private-forwarding-and-single-broker-endpoint-sow.md)
- [Milestone 1.3: Breakglass Maintenance Channel](./milestone-1-3-breakglass-maintenance-channel-sow.md)
