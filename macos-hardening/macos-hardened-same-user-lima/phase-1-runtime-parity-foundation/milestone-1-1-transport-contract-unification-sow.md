# Milestone 1.1: Transport Contract Unification SOW

Status: Draft

Owner: Substrate world backend / macOS transport layer

Last updated: 2026-04-28

## Purpose / Outcome

Define and implement one canonical host-to-guest transport contract for the
macOS Lima backend so that execution, readiness probing, and diagnostics all
describe the same path.

The concrete outcome is that VSock, SSH UDS, and any retained TCP fallback are
implementation details behind one declared contract rather than partially
independent behavior paths.

## Why This Milestone Exists

Transport behavior is currently fragmented:

- `17788` is used in forwarding and doctor code, while `7788` is still hardcoded
  elsewhere in the backend.
- the backend, forwarding layer, and doctor logic all carry separate endpoint
  assumptions.
- PTY and non-PTY parity depends on those assumptions staying accidentally in
  sync.

This creates false confidence: macOS can appear "working" while different code
paths are probing different endpoints.

## In Scope

- Unify transport constants, endpoint resolution, and forwarding metadata used
  by the macOS backend.
- Make routed execution and readiness probes consume the same selected
  transport.
- Normalize how transport selection is surfaced to shell doctor/reporting
  surfaces.
- Remove stale TCP constant drift and stale fallback assumptions.

## Out of Scope

- Adding listener authentication or attestation.
- Removing Lima from the same-user ownership model.
- Narrowing guest mounts or redesigning provisioning layout.
- Reworking backend policy semantics beyond the transport-related fields needed
  for endpoint delivery.

## Architectural Approach

The transport contract for this milestone is:

- canonical guest service endpoint: `/run/substrate.sock`
- canonical host transport abstraction: one selected forwarding mode exposed by
  `world-mac-lima`
- canonical health/readiness contract: route the same capabilities / doctor
  checks through the selected transport first

Implementation should centralize the selected transport description in
[crates/world-mac-lima/src/transport.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs)
or an adjacent shared type, then make
[crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs),
[crates/world-mac-lima/src/forwarding.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs),
and
[crates/shell/src/execution/platform/macos.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/macos.rs)
consume that one source of truth.

If TCP remains as a fallback, it must use one documented port constant and one
single reason for existence. If it is not truly supported, the contract should
say so and the dead fallback behavior should be removed.

## Dependencies / Sequencing

- This is the first runtime milestone in Phase 1.
- Milestone 1.2 depends on this milestone because policy parity must ride the
  same transport contract for PTY and non-PTY execution.
- Milestone 1.3 depends on this milestone because doctor and smoke evidence
  cannot be authoritative while transport probing remains split.

## Concrete Repo Surfaces and File Pointers

Primary implementation surfaces:

- [crates/world-mac-lima/src/transport.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs)
- [crates/world-mac-lima/src/forwarding.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs)
- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs)
- [crates/shell/src/execution/platform/macos.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/macos.rs)
- [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)

Current drift evidence:

- [crates/world-mac-lima/src/forwarding.rs:149](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs:149)
- [crates/world-mac-lima/src/transport.rs:53](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs:53)
- [crates/world-mac-lima/src/lib.rs:207](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:207)
- [crates/shell/src/execution/platform/macos.rs:360](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/macos.rs:360)

Secondary doc/script surfaces that must follow the contract:

- [scripts/mac/lima-doctor.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh)
- [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)

## Deliverables

- One shared transport model for the macOS Lima backend.
- Elimination of contradictory TCP / VSock / UDS constants and endpoint strings.
- A consistent agent-client endpoint construction path for routed execution.
- Doctor/reporting output that can state which transport was selected and
  whether that transport is healthy.
- Tests covering transport selection and endpoint derivation.

## Acceptance Criteria

- No remaining macOS backend code path disagrees on the loopback port used for a
  TCP or VSock-routed agent path.
- `MacLimaBackend` derives its agent endpoint from the same selected forwarding
  contract used by readiness probing.
- `substrate host doctor` and `substrate world doctor` do not probe a different
  endpoint contract than routed command execution.
- PTY and non-PTY macOS execution paths continue to work after transport
  centralization.
- Documentation describes one transport ordering and one fallback story.

## Validation / Evidence Plan

Required evidence for this milestone:

- targeted unit tests for `transport.rs` and forwarding endpoint construction
- `cargo test -p world-mac-lima`
- targeted shell tests for macOS doctor reporting if transport fields are
  surfaced there
- `scripts/mac/lima-doctor.sh`
- `substrate host doctor --json`
- `substrate world doctor --json`

Evidence should explicitly show which transport was selected and that the same
selection is used for command execution and doctor probing.

## Risks / Open Questions

- The current shell doctor implementation has its own probing logic; deciding
  whether to reuse backend transport code directly or mirror it via a shared
  utility needs care to avoid cyclic dependencies.
- TCP fallback may be partly vestigial. The team needs to decide whether to make
  it real, keep it only as an explicit breakglass path, or remove it.
- VSock availability varies with host setup; the contract must be stable even
  when VSock is absent.
