# Milestone 1.2: Policy Application Parity SOW

Status: Draft

Owner: Substrate world backend / policy application path

Last updated: 2026-04-28

## Purpose / Outcome

Make the macOS Lima backend execute with the same evaluated policy semantics
that Linux world execution already depends on, instead of synthesizing a
permissive backend-local snapshot.

The concrete outcome is that macOS world execution uses policy data derived from
the broker/shell policy-resolution pipeline, and backend policy updates become
observable and testable rather than effectively ignored.

## Why This Milestone Exists

Today the macOS backend only carries `WorldFsMode` state and invents the rest of
the policy request:

- [convert_exec_request](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:275)
  fabricates `PolicySnapshotV3` with host-visible, current-directory-only,
  permissive settings.
- [apply_policy](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:463)
  does not apply anything beyond storing `fs_mode`.
- [world_api::WorldSpec](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:14)
  currently lacks the richer normalized policy payload that the Linux-facing
  direct request path already uses.

This means macOS parity can be overstated even when the backend is not honoring
the same policy contract the rest of Substrate resolves.

## In Scope

- Stop synthesizing permissive `PolicySnapshotV3` values inside
  `world-mac-lima`.
- Thread the evaluated policy snapshot and related world-routing inputs through
  the macOS backend path.
- Implement meaningful `apply_policy` semantics for the macOS backend.
- Align macOS world execution with Linux expectations for `net_allowed`,
  `world_fs.fail_closed`, and world-fs enforcement-plan inputs as far as the
  same-user Lima model can support.

## Out of Scope

- Introducing new policy language or changing broker policy semantics.
- Solving ownership-boundary problems or direct transport authentication.
- Replacing the Linux guest enforcement engine.
- Mount-model hardening beyond policy data propagation needed for parity.

## Architectural Approach

The shell / broker policy-resolution flow should remain the source of truth:

- policy resolution lives in
  [crates/shell/src/execution/policy_snapshot.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_snapshot.rs)
- direct world operations already construct agent requests with resolved
  `policy_snapshot`, `world_network`, and world-fs enforcement-plan inputs in
  [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

The backend path used by `WorldBackend`-driven execution should be brought up to
that same contract. If
[crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs)
cannot carry the necessary evaluated policy state today, this milestone extends
the world backend abstraction rather than preserving macOS-only synthesis.

The target behavior is:

- the broker-resolved snapshot is canonical
- the backend stores or receives the current snapshot for the world session
- execution requests sent through `MacLimaBackend` reuse that snapshot
- `apply_policy` becomes the point where drift between world session state and
  current policy is reconciled or rejected

## Dependencies / Sequencing

- Depends on Milestone 1.1 transport unification because policy parity must be
  tested over the canonical routed transport.
- Blocks Milestone 1.3 because readiness evidence should prove policy parity,
  not just transport reachability.

## Concrete Repo Surfaces and File Pointers

Primary backend and contract surfaces:

- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs)
- [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs)
- [crates/world-backend-factory/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-backend-factory/src/lib.rs)

Primary shell policy-resolution surfaces:

- [crates/shell/src/execution/policy_snapshot.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_snapshot.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/src/execution/routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs)

Current divergence evidence:

- [crates/world-mac-lima/src/lib.rs:275](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:275)
- [crates/world-mac-lima/src/lib.rs:463](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:463)
- [crates/shell/src/execution/policy_snapshot.rs:155](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_snapshot.rs:155)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs:1046](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1046)

## Deliverables

- A world-backend contract that can carry evaluated policy state needed for
  macOS parity.
- Removal of synthetic permissive snapshot generation from `MacLimaBackend`.
- Implemented `apply_policy` behavior for macOS world sessions.
- Regression tests that prove macOS backend requests carry the expected policy
  values.
- Updated parity docs describing the new backend policy source of truth.

## Acceptance Criteria

- No macOS backend request path fabricates a permissive `PolicySnapshotV3` from
  `WorldFsMode` alone.
- A policy change that affects world routing or world-fs semantics is visible to
  the macOS backend and can trigger session update, reapply, or fail-closed
  behavior by design.
- macOS execution requests carry the same relevant policy fields that Linux
  world-agent requests already depend on.
- `apply_policy` is no longer a semantic no-op.
- Tests cover at least `net_allowed`, `world_fs.write.enabled`, and
  fail-closed/routing-relevant behavior in the macOS backend path.

## Validation / Evidence Plan

Required evidence for this milestone:

- targeted unit tests in `world-mac-lima` for request conversion / policy state
- targeted tests in `world-api` or shell routing if the backend contract is
  extended
- relevant shell tests that already assert policy snapshot contents for world
  requests
- `cargo test -p world-mac-lima`
- targeted end-to-end proof via `substrate world doctor --json` and one or more
  routed world operations that would previously have succeeded under the
  synthetic permissive path

Evidence should show not just that the agent is reachable, but that the
effective policy carried to macOS matches the broker-resolved policy snapshot.

## Risks / Open Questions

- Extending the backend abstraction may expose similar drift in the Windows WSL
  backend, which currently also synthesizes policy in a backend-local path.
- The repo has both backend-mediated execution and direct agent-request code
  paths; this milestone must decide whether to unify those paths structurally or
  keep them separate while sharing a common policy snapshot source.
- Session reuse semantics may need new invalidation behavior when the policy
  snapshot changes.
