# Phase 1: Runtime Parity Foundation

Status: Draft

Owner: Substrate world backend / macOS runtime parity

Last updated: 2026-04-28

## Purpose / Outcome

Establish one credible macOS runtime contract for the current same-user Lima
model before deeper hardening work begins. The outcome of this phase is not
"macOS is secure enough"; it is "macOS and Linux describe the same runtime
behavior at the transport, policy-application, and readiness-evidence layers."

Phase 1 is complete when the macOS backend no longer depends on transport drift,
synthetic policy injection, or direct-guest operational shortcuts to appear
healthy.

## Why This Phase Exists

The current macOS backend still diverges from Linux in three foundational ways:

- transport behavior is split across
  [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:207),
  [crates/world-mac-lima/src/forwarding.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs:149),
  and
  [crates/world-mac-lima/src/transport.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs:53),
  including the current `17788` vs `7788` inconsistency.
- policy semantics are not actually applied by the backend because
  [MacLimaBackend::convert_exec_request](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:275)
  synthesizes a permissive snapshot and
  [MacLimaBackend::apply_policy](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:463)
  is a no-op.
- operational validation still normalizes direct `limactl shell` entry as part
  of normal setup, diagnosis, and smoke flows in
  [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:788),
  [scripts/mac/lima-doctor.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh:62),
  [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:155),
  [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:148),
  and
  [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:85).

Until these three layers are aligned, later same-user hardening work would be
stacked on top of a runtime that still behaves differently from Linux and still
proves itself using bypass-heavy workflows.

## In Scope

- Define one canonical macOS host-to-guest transport contract and remove
  internal constant drift.
- Make PTY, non-PTY, readiness probing, and doctor reporting consume the same
  transport contract.
- Replace the synthetic permissive policy snapshot path with Linux-like policy
  application semantics, even if the backend contract must be extended to carry
  the normalized snapshot.
- Make doctor/smoke/setup evidence prove the routed Substrate path first, with
  direct guest entry moved out of the normal operator path.

## Out of Scope

- Changing the host ownership boundary of Lima or `LIMA_HOME`.
- Claiming Linux-equivalent ownership isolation on macOS.
- Removing all same-user hardening gaps such as broad host mounts, extra guest
  listeners, or direct control-plane ownership.
- Designing the ownership-separated macOS architecture.
- Rewriting provisioning to eliminate every internal `limactl` dependency.

## Architectural Approach

Phase 1 treats `/run/substrate.sock` inside the guest as the only canonical
service endpoint. VSock, SSH UDS, and any retained TCP surface are host-side
transport adapters, not separate behavioral modes.

The backend must then stop inventing policy. The evaluated broker snapshot used
by Linux-facing world execution should become the source of truth for macOS as
well. If
[world_api::WorldSpec](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:14)
cannot currently carry the policy and routing state required for parity, this
phase extends that contract rather than preserving backend-local synthesis.

Finally, readiness flows must prove the same path that users depend on:

- `substrate host doctor`
- `substrate world doctor`
- `scripts/mac/smoke.sh`

Direct guest commands remain available only as breakglass diagnostics and must
no longer define the happy-path contract.

## Dependencies / Sequencing

This phase is ordered and should be executed in the following sequence:

1. [Milestone 1.1: Transport Contract Unification](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md)
2. [Milestone 1.2: Policy Application Parity](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md)
3. [Milestone 1.3: Doctor / Smoke Readiness Parity](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md)

Milestone 1.3 can prepare doc rewrites in parallel, but final readiness
criteria should not be signed off until Milestones 1.1 and 1.2 land because the
doctor/smoke evidence needs to prove the new transport and policy behavior.

## Concrete Repo Surfaces and File Pointers

Primary runtime/backend surfaces:

- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs)
- [crates/world-mac-lima/src/forwarding.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs)
- [crates/world-mac-lima/src/transport.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs)
- [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs)
- [crates/shell/src/execution/policy_snapshot.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_snapshot.rs)
- [crates/shell/src/execution/routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs)
- [crates/shell/src/execution/platform/macos.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/macos.rs)
- [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)

Primary scripts and operator-doc surfaces:

- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
- [scripts/mac/lima-doctor.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh)
- [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md)

Research input anchoring this phase:

- [thoughts/shared/research/2026-04-28-macos-lima-parity-lockdown.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/thoughts/shared/research/2026-04-28-macos-lima-parity-lockdown.md)

## Deliverables

- A phase-local milestone set with explicit sequencing and acceptance gates.
- A canonical macOS transport contract design covering backend, shell, doctor,
  and smoke consumers.
- A backend policy-application plan that removes synthetic permissive snapshots.
- A readiness-evidence plan that makes CLI doctors and smoke flows authoritative
  and relegates direct guest entry to breakglass.

## Acceptance Criteria

- There is one documented and implemented macOS transport contract for routed
  world traffic, and no remaining `17788` vs `7788` disagreement.
- Mac PTY and non-PTY requests consume the same transport-selection rules and
  endpoint semantics.
- `MacLimaBackend` no longer invents a permissive policy snapshot from
  `WorldFsMode` alone.
- macOS doctor/smoke readiness proves routed Substrate behavior before any
  direct guest fallback checks.
- The phase docs make explicit that same-user Lima still does not provide the
  Linux ownership boundary.

## Validation / Evidence Plan

Implementation work under this phase should attach evidence from:

- targeted Rust tests for `world-mac-lima`, `world-api`, and shell routing /
  doctor surfaces
- `cargo test -p world-mac-lima`
- relevant shell tests covering world routing and doctor output
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- `substrate host doctor --json`
- `substrate world doctor --json`

Evidence must show the selected transport, routed agent reachability, and
policy-parity behavior through the CLI path, not only through direct in-guest
`curl`.

## Risks / Open Questions

- Extending `WorldSpec` or adjacent backend contracts may ripple into the
  Windows backend and shared factory code.
- The repo currently mixes backend abstraction and direct agent API request
  paths; parity work must avoid creating two competing policy sources of truth.
- Some direct guest checks may remain necessary during bootstrap failure, but
  those must be classified as breakglass diagnostics rather than normal setup.
- Phase 1 improves parity, not ownership isolation; future hardening phases must
  still address broad mounts, extra listeners, and same-user Lima control-plane
  ownership.
