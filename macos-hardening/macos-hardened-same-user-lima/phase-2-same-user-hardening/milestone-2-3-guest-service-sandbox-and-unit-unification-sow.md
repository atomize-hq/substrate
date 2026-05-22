# Milestone 2.3: Guest Service Sandbox and Unit Unification

## Status

Draft

Last updated: 2026-05-19

## Purpose / outcome

Unify the macOS Lima guest `substrate-world-service` service and socket
definitions behind one authoritative source so the hardened sandbox,
socket-activation contract, and environment are consistent across VM creation,
repair, and diagnostics.

## Why this milestone exists

Today the repo defines guest units in at least two places:

- `scripts/mac/lima/substrate.yaml`
- `scripts/mac/lima-warm.sh`

Those copies are already divergent in capability sets and environment lines. Hardening work cannot be trusted if the initial VM profile and the repair path can recreate different sandboxes. This milestone exists to make the guest unit definition a single hardening authority and to align it with the reduced listener and mount contract established in milestones 2.1 and 2.2.

Current repo truth also means the unit contract has to account for more than
the socket alone:

- managed gateway runtime artifacts already live under
  `/run/substrate/substrate-gateway-runtime/`
- gateway lifecycle smoke coverage already exists
- integrated auth handoff already exists

So the unified service sandbox must preserve the supported gateway lifecycle
surface while tightening listener, mount, and writable-path posture.

## In-scope

- Define one authoritative source for the guest service and socket unit contents.
- Eliminate hardening-critical drift across provisioning and repair paths.
- Reconcile sandbox settings such as `ProtectHome`, `ReadWritePaths`, capabilities, runtime directories, and environment variables with the Phase 2 hardening goals.
- Reconcile those settings with the already-landed managed gateway runtime
  surface under `/run/substrate/substrate-gateway-runtime/`.
- Update doctor and documentation references so operators know which unit definition is authoritative.

## Out-of-scope

- Replacing systemd in the guest.
- Reworking Linux guest service installation outside the macOS/Lima path except where shared documentation needs consistency.
- Phase 3 CLI lifecycle ownership beyond exposing the right backend contract for it.

## Architectural approach

- Move to one guest unit source of truth that can be rendered or installed by both the Lima profile bootstrap path and the warm/repair path.
- Treat any setting that affects authority, access, or reachability as hardening-critical:
  - listener env vars
  - socket ownership and mode
  - service group and runtime directory settings
  - capability bounding and ambient capabilities
  - writable guest paths
- Use the same generated unit content for create, repair, doctor expectations,
  gateway lifecycle expectations, and doc examples.

The unified contract must explicitly encode:

- no default guest TCP listener injection for the hardened path
- writable/runtime paths required for `/run/substrate.sock` and
  `/run/substrate/substrate-gateway-runtime/`
- only the minimum environment needed for supported world-service and managed
  gateway operation

## Dependencies / sequencing

- Milestone 2.1 must be complete before this milestone is implemented, because
  unit unification must encode the landed reduced-listener contract rather than
  a provisional version of it.
- Milestone 2.2 should also be complete before this milestone is implemented;
  at minimum, its ingress contract must be fully frozen before any unit
  generation logic is finalized.
- Phase 3 depends on this milestone because Substrate-owned lifecycle commands should repair and diagnose one known service contract, not reconcile multiple variants.

## Concrete repo surfaces and file pointers

- `scripts/mac/lima/substrate.yaml`
  - bootstraps guest package install and unit files during VM creation
- `scripts/mac/lima-warm.sh`
  - currently rewrites the service and socket units during warm/repair
  - currently injects hardening-relevant environment and capability settings
- `crates/world-service/src/gateway_runtime.rs`
  - authoritative runtime-artifact expectations for `/run/substrate/substrate-gateway-runtime/`
- `scripts/mac/lima-doctor.sh`
  - should verify the unified unit contract rather than loosely checking partial state
- `scripts/mac/smoke.sh`
  - should keep proving gateway lifecycle against the unified service contract
- `docs/WORLD.md`
  - references guest unit behavior and operator verification
- `docs/cross-platform/mac_world_setup.md`
  - currently includes manual service enable/start flows that must match the unified definition

## Deliverables

- One source-of-truth design for macOS guest unit generation.
- A drift inventory documenting how the current YAML and warm-script units differ today.
- A hardened target unit contract aligned to the listener and ingress decisions from milestones 2.1 and 2.2.
- Updated validation and doc guidance for the unified unit.

## Acceptance criteria

- There is exactly one authoritative macOS guest unit definition path for `substrate-world-service.service` and `substrate-world-service.socket`.
- Hardening-critical settings are identical whether the guest is freshly created or repaired.
- Doctor, gateway lifecycle, and troubleshooting guidance point to the unified
  unit contract instead of to whichever path happened to run last.
- A reviewer can explain the guest service sandbox from one source file or generator rather than diffing two handwritten unit bodies.

## Validation / evidence plan

- Produce a side-by-side diff of the current YAML-embedded unit and warm-script unit as the baseline evidence.
- Define a rendered-unit verification step for macOS doctor or smoke coverage.
- Re-run `scripts/mac/lima-warm.sh`, `scripts/mac/lima-doctor.sh`, and
  `scripts/mac/smoke.sh` after unification to prove the created and repaired VM
  paths converge.
- Re-run `substrate world gateway sync|status|restart` and confirm the managed
  runtime artifacts and support path stay healthy after unit unification.
- Review doc examples to ensure any pasted unit snippets or systemctl guidance match the unified contract.

## Risks / open questions

- A single source of truth may need a small templating layer if profile bootstrap and repair-time installation have different constraints.
- Capability tightening may surface previously hidden dependencies in full-isolation or netfilter flows.
- If the unified unit depends on mount decisions not yet implemented, this milestone may need a staged landing with a frozen target contract and temporary compatibility notes.
