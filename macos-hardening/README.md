# macOS Hardening Program

Status: Draft

Owner: Substrate world backend / platform security

Last updated: 2026-04-28

## Purpose

This directory is the planning root for the macOS hardening program described in
[thoughts/shared/research/2026-04-28-macos-lima-parity-lockdown.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/thoughts/shared/research/2026-04-28-macos-lima-parity-lockdown.md).
It separates two product tracks that share some runtime foundations but have
different security claims:

- `macos-hardened-same-user-lima/`: improve behavioral parity with Linux and
  materially reduce bypass in the current same-user Lima model.
- `macos-ownership-separated-world/`: redesign the macOS control plane so the
  VM and its transport are no longer effectively owned by the invoking user.

## Why This Split Exists

The current macOS backend can be improved substantially without changing the
host ownership model, but that work does not satisfy the stronger requirement
that only Substrate can access the VM. The research note and current repo state
show that these are distinct outcomes:

- Functional parity and hardening gaps live in runtime/backend surfaces such as
  [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:1),
  [crates/world-mac-lima/src/forwarding.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs:1),
  [crates/world-mac-lima/src/transport.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs:1),
  and [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs:699).
- Ownership-boundary gaps live in host lifecycle, installer, Lima ownership,
  transport reachability, and operator workflow surfaces such as
  [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:665),
  [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:136),
  [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:82),
  and [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:1854).

## Program Structure

- `macos-hardened-same-user-lima/`
  - Feature overview plus phase overviews and milestone SOWs for the same-user
    hardening path.
- `macos-ownership-separated-world/`
  - Feature overview plus phase overviews and milestone SOWs for the stronger
    ownership-separated target architecture.

Each feature directory contains:

- a feature overview document (`README.md`)
- one directory per phase
- a phase overview document in each phase directory (`README.md`)
- one milestone SOW per milestone in that phase

## Sequencing Rules

The two tracks are related but not interchangeable.

- The same-user track is the prerequisite for a credible hardened macOS story.
  It resolves transport drift, policy drift, mount overexposure, unit drift,
  and direct-guest operational shortcuts.
- The ownership-separated track depends on the same-user track for transport and
  operational clarity, but then replaces the host ownership model with a daemon-
  owned or service-account-owned control plane.
- Work in the ownership-separated track should not reintroduce any direct guest
  admin path as part of the normal operator contract.

## Required Evidence Sources

All SOWs in this tree should stay anchored to the current repo and research
inputs:

- [thoughts/shared/research/2026-04-28-macos-lima-parity-lockdown.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/thoughts/shared/research/2026-04-28-macos-lima-parity-lockdown.md)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md)
- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs)
- [crates/world-mac-lima/src/forwarding.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs)
- [crates/world-mac-lima/src/transport.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs)
- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
- [scripts/mac/lima/substrate.yaml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/substrate.yaml)
- [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh)
- [scripts/mac/lima-doctor.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh)

## Exit Condition

This planning tree is complete when:

- every phase has a scoped overview
- every milestone has a concrete SOW
- dependencies between same-user hardening and ownership separation are explicit
- acceptance criteria and validation evidence are defined per milestone
- cross-document consistency has been reviewed after the draft wave lands
