# Milestone 2.1: Installer, Upgrade, and Migration Path

Status: Draft

Phase: 2 - Productization and Migration

Last updated: 2026-05-19

## Purpose / Outcome

Deliver a supported macOS lifecycle path for the ownership-separated backend:
clean install, in-place upgrade, migration from same-user Lima, rollback-safe
rerun behavior, and complete uninstall. This milestone owns how Substrate gets
from today’s repo reality to the new control-plane model on actual user
machines.

## Why This Milestone Exists

The current macOS lifecycle is same-user by construction:

- `scripts/substrate/install-substrate.sh` provisions Lima as the invoking user
- `scripts/substrate/world-enable.sh` reuses the same-user installer
  provisioning helpers
- `scripts/substrate/dev-install-substrate.sh` stages guest binaries and writes
  same-user world metadata
- `scripts/substrate/uninstall-substrate.sh` deletes the user-owned Lima VM
- `scripts/substrate/dev-uninstall-substrate.sh` assumes user-owned helper
  state
- `docs/INSTALLATION.md` and `docs/UNINSTALL.md` describe the same-user flow as
  the supported path

That is incompatible with the product claim that only Substrate controls the
VM. Ownership separation is not shippable until install and migration rewrite
the host-side lifecycle around the new owner model.

## In Scope

- macOS install flow for the ownership-separated backend
- upgrade flow from one ownership-separated release to another
- migration flow from same-user Lima installs into the new owner model
- idempotent rerun and rollback-safe behavior for partial installs
- uninstall behavior for both migrated and clean ownership-separated installs
- installer/uninstaller metadata needed to distinguish legacy, migrated, and
  native ownership-separated states
- operator documentation updates for install, upgrade, migration, and uninstall
- lifecycle integration for the already-landed gateway/runtime surfaces

## Out of Scope

- doctor/status UX beyond the minimum needed to confirm install success
- broad support escalation tooling
- redefining the transport/authentication contract itself
- preserving direct `limactl shell` as a supported normal-path operator action
- Linux and Windows lifecycle redesign

## Architectural Approach

The installer becomes the authoritative state transition engine for macOS world
ownership:

1. Detect whether the machine is a clean install, an ownership-separated
   upgrade, or a same-user Lima migration candidate.
2. Materialize or verify the dedicated host owner and private control-plane
   state required by phase 1.
3. Migrate only the artifacts that are safe and necessary to preserve. Do not
   carry forward same-user-owned control-plane material as trusted state.
4. Write explicit install metadata so upgrade and uninstall paths know whether
   the host is legacy, migrated, or natively ownership-separated.
5. Expose one documented rollback posture for interrupted or failed migration.
6. Ensure the normal lifecycle path still lands with working world and gateway
   operator surfaces.

The uninstall path must understand the new metadata and clean up the new owner
model without guessing based on same-user Lima conventions.

## Dependencies / Sequencing

Depends on:

- the same-user hardening track milestones required by the parent phase:
  - phase 2 milestone 2.2 mount minimization and ingress contract
  - phase 3 milestone 3.1 Substrate-managed diagnostics and lifecycle
  - phase 3 milestone 3.2 breakglass reclassification and doc cutover
- phase 1 host ownership separation
- a stable macOS control-plane registration mechanism produced as a phase 1
  prototype closeout artifact, with this milestone owning its install, upgrade,
  and migration integration into the shipped macOS lifecycle
- a stable normal-path transport endpoint that no longer treats user-owned
  forwarded sockets or loopback ports as the security boundary

Sequences before:

- milestone 2.2, because doctor/status/supportability must report the actual
  lifecycle state this milestone defines
- milestone 2.3, because the GA matrix depends on install, migration, and
  uninstall behaviors being fixed first

## Concrete Repo Surfaces and File Pointers

Primary surfaces:

- `scripts/substrate/world-enable.sh`
- `scripts/substrate/dev-install-substrate.sh`
- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/install.sh`
- `scripts/substrate/dev-uninstall-substrate.sh`
- `scripts/substrate/uninstall-substrate.sh`
- `scripts/substrate/uninstall.sh`
- `scripts/mac/lima-warm.sh`
- `../../../docs/INSTALLATION.md`
- `../../../docs/UNINSTALL.md`
- `../../../docs/WORLD.md`
- `../../../docs/cross-platform/mac_world_setup.md`

State and runtime surfaces likely affected:

- `scripts/mac/lima/substrate.yaml`
- `scripts/mac/lima/substrate-dev.yaml`
- `crates/shell/src/execution/platform/macos.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/world-service/src/gateway_runtime.rs`

Legacy-state evidence to account for:

- user-owned `~/.lima`
- user-owned `~/.substrate`
- same-user forwarded endpoints described in current docs and scripts
- same-user guest binary staging and verification for `substrate-gateway`

## Deliverables

- a macOS install contract for ownership-separated deployment
- a macOS upgrade contract for already-migrated hosts
- a same-user Lima migration contract with explicit preserve/recreate/discard
  rules for existing host and guest artifacts
- a shipped lifecycle integration plan for the phase 1 control-plane
  registration mechanism, including clean install bootstrap, upgrade handling,
  and uninstall cleanup expectations
- installer metadata schema updates needed to encode ownership model and
  migration status
- uninstall behavior that removes new control-plane assets without destroying
  unrelated user state
- operator docs covering clean install, migration, rollback, and uninstall

## Acceptance Criteria

- Clean install on supported macOS creates an ownership-separated deployment
  without requiring the operator to run direct guest commands.
- Rerunning install on an already-migrated machine is idempotent and does not
  duplicate or corrupt control-plane state.
- Migration from a same-user Lima install is explicit, detectable, and
  recoverable if interrupted.
- Uninstall handles both clean ownership-separated installs and migrated hosts.
- The documented normal path does not instruct operators to use `limactl shell`,
  guest `systemctl`, or guest `curl`.
- The install flow records enough metadata for downstream doctor/status surfaces
  to distinguish clean, upgraded, migrated, and partially failed states.
- The post-install normal path leaves `substrate host doctor`,
  `substrate world doctor`, `substrate health`, and
  `substrate world gateway status --json` working through the new owner model.

## Validation / Evidence Plan

- Capture clean-install transcripts from `scripts/substrate/install-substrate.sh`
  on a fresh supported macOS machine.
- Capture migration transcripts from a machine preloaded with the current
  same-user Lima install path.
- Capture rerun evidence showing idempotent install and upgrade behavior.
- Capture uninstall transcripts for both migrated and clean
  ownership-separated hosts.
- Capture post-install and post-migration readiness with:
  - `substrate host doctor --json`
  - `substrate world doctor --json`
  - `substrate health --json`
  - `substrate world gateway status --json`
- Capture at least one gateway lifecycle round trip after install:
  - `substrate world gateway sync`
  - `substrate world gateway restart`
- Update `docs/INSTALLATION.md`, `docs/UNINSTALL.md`, `docs/WORLD.md`, and
  `docs/cross-platform/mac_world_setup.md` so the documented contract matches
  the transcripts above.

## Risks / Open Questions

- Which same-user Lima artifacts are safe to reuse, and which must be rebuilt
  under the new owner to avoid trusting old access paths?
- How much operator prompting is acceptable during migration if privilege,
  daemon registration, or account creation is required?
- Should migration be in-place, or should the product create a fresh
  ownership-separated deployment and then import only selected state?
- How are failed mid-migration hosts recovered when both legacy and new state
  exist at once?
- Does macOS dev-install need an ownership-separated variant, or should the dev
  flow stay intentionally outside the supported product path?
