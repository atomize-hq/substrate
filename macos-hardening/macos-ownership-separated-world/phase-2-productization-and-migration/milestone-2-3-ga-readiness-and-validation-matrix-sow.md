# Milestone 2.3: GA Readiness and Validation Matrix

Status: Draft

Phase: 2 - Productization and Migration

Last updated: 2026-05-19

## Purpose / Outcome

Define and execute the release gate for macOS ownership separation. This
milestone turns the earlier milestone outputs into a GA decision backed by a
published validation matrix, concrete evidence artifacts, and explicit release
blockers.

## Why This Milestone Exists

Without a formal validation matrix, the macOS ownership-separated path could
ship with one successful demo path but miss the real release-critical cases:
same-user migration, reinstall, reboot/relogin persistence, partial failure
recovery, uninstall cleanup, gateway lifecycle parity, and operator
supportability under broken states.

The current repo already has test and evidence entrypoints, but they do not yet
express a GA gate for this new product claim:

- `scripts/mac/smoke.sh` already proves selected runtime and gateway lifecycle
  flows
- `scripts/mac/lima-doctor.sh` proves selected host/guest checks in the current
  same-user model
- `docs/INSTALLATION.md`, `docs/UNINSTALL.md`, `docs/WORLD.md`, and
  `docs/cross-platform/mac_world_setup.md` describe operator behavior but do
  not define a release matrix

This milestone closes that gap.

## In Scope

- the macOS ownership-separated GA validation matrix
- mandatory scenario coverage for install, upgrade, migration, reboot, repair,
  gateway lifecycle, uninstall, and support workflows
- artifact expectations for each matrix row
- release-blocking criteria
- final doc consistency review across install, world, gateway, and uninstall
  guidance

## Out of Scope

- adding new product functionality unrelated to release readiness
- replacing milestone 2.1 or 2.2 implementation work with new design changes
- unlimited exploratory QA outside the defined matrix

## Architectural Approach

The GA gate is evidence-first and scenario-based:

1. Define the supported macOS versions, hardware assumptions, and install
   origins in a matrix that maps directly to repo verification surfaces.
2. For each scenario, name the expected artifacts: install transcript, doctor
   JSON, health JSON, gateway status JSON, smoke artifacts, and uninstall
   evidence where applicable.
3. Treat migration from same-user Lima as a first-class GA scenario, not a beta
   follow-up.
4. Treat supportability as part of the gate: a healthy runtime is not enough if
   the operator contract cannot explain or repair failure modes.
5. Block GA on doc drift: the published docs must match the validated
   lifecycle/status behavior exactly.

## Dependencies / Sequencing

Depends on:

- the same-user hardening track milestones required by the parent phase:
  - phase 2 milestone 2.2 mount minimization and ingress contract
  - phase 3 milestone 3.1 Substrate-managed diagnostics and lifecycle
  - phase 3 milestone 3.2 breakglass reclassification and doc cutover
- milestone 2.1 for install/upgrade/migration/uninstall behavior
- milestone 2.2 for doctor/status/supportability behavior

Final milestone in this phase:

- this is the phase closeout gate for productization and migration

## Concrete Repo Surfaces and File Pointers

Primary evidence and contract surfaces:

- `scripts/mac/smoke.sh`
- `scripts/mac/lima-doctor.sh`
- `scripts/substrate/world-enable.sh`
- `scripts/substrate/dev-install-substrate.sh`
- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/uninstall-substrate.sh`
- `crates/shell/src/execution/platform/macos.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `../../../docs/INSTALLATION.md`
- `../../../docs/UNINSTALL.md`
- `../../../docs/WORLD.md`
- `../../../docs/contracts/substrate-gateway-operator-contract.md`
- `../../../docs/contracts/substrate-gateway-status-schema.md`
- `../../../docs/cross-platform/mac_world_setup.md`

Likely supporting surfaces:

- `scripts/mac/lima-warm.sh`
- `scripts/mac/lima/substrate.yaml`
- `scripts/mac/lima/substrate-dev.yaml`
- `crates/world-service/src/gateway_runtime.rs`

## Deliverables

- a macOS ownership-separated validation matrix
- a required-artifacts checklist per matrix scenario
- a release-blocker list for install, migration, gateway/status,
  supportability, and uninstall regressions
- a final cross-doc review checklist for install/world/gateway/uninstall/mac
  setup docs
- a phase-closeout evidence bundle definition

## Acceptance Criteria

- The matrix includes, at minimum:
  - clean install on supported macOS
  - upgrade from prior ownership-separated release
  - migration from current same-user Lima install
  - rerun/idempotency after successful install
  - interrupted install or interrupted migration recovery
  - reboot/relogin persistence
  - healthy doctor/status/support outputs
  - healthy gateway sync/status/restart outputs
  - representative broken-state doctor/status/support outputs
  - uninstall after clean install
  - uninstall after migration
- Each matrix row names the exact required evidence artifacts.
- Release blockers are explicit enough that "security boundary claim not
  supportable" is a shippability failure, not a soft concern.
- The validated docs no longer present direct guest commands as the normal path
  for install or support.

## Validation / Evidence Plan

- Execute the matrix on supported macOS hardware/software combinations chosen by
  the release owner.
- For each scenario, capture:
  - install or uninstall transcript when relevant
  - `substrate host doctor --json`
  - `substrate world doctor --json`
  - `substrate health --json`
  - `substrate world gateway status --json`
  - `substrate world gateway sync`
  - `substrate world gateway restart`
  - `scripts/mac/smoke.sh` artifacts for the relevant runtime scenario
- Require one evidence set from a host that started on the legacy same-user
  Lima flow and was migrated successfully.
- Require one evidence set showing operator guidance for a deliberately broken
  state without relying on direct guest commands.
- Require one evidence set that proves lifecycle failures pointing at
  `/run/substrate/substrate-gateway-runtime/...` remain diagnosable through the
  supported operator boundary.
- Run a final doc review against `docs/INSTALLATION.md`, `docs/UNINSTALL.md`,
  `docs/WORLD.md`, and `docs/cross-platform/mac_world_setup.md` to ensure every
  validated scenario is documented consistently.

## Risks / Open Questions

- Which macOS versions and Apple Silicon host classes are mandatory for GA,
  versus best-effort or post-GA coverage?
- How much manual evidence collection is acceptable before the matrix becomes
  too costly to rerun for each release candidate?
- If migration succeeds functionally but leaves behind legacy same-user state,
  is that a GA blocker or a documented cleanup defect?
- What is the exact threshold for "breakglass only" documentation so support
  can still recover severely broken machines without normalizing direct guest
  entry?
