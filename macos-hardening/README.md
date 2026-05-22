# macOS Hardening Program

Status: Draft

Owner: Substrate world backend / platform security

Last updated: 2026-05-19

## Purpose

This directory is the planning root for the macOS hardening program described in
[the research note](./research/2026-04-28-macos-lima-parity-lockdown.md).
It separates two product tracks that share some runtime foundations but have
different security claims:

- `macos-hardened-same-user-lima/`: improve behavioral parity with Linux and
  materially reduce bypass in the current same-user Lima model.
- `macos-ownership-separated-world/`: redesign the macOS control plane so the
  VM, forwarding, and gateway lifecycle path are no longer effectively owned by
  the invoking user.

## Why This Split Exists

The current macOS backend can be improved substantially without changing the
host ownership model, but that work does not satisfy the stronger requirement
that only Substrate controls the VM and its lifecycle path. The research note
and current repo state show that these are distinct outcomes:

- Functional parity and hardening gaps live in runtime/backend surfaces such as
  `crates/world-mac-lima/src/lib.rs`,
  `crates/world-mac-lima/src/forwarding.rs`,
  `crates/world-mac-lima/src/transport.rs`, and
  `crates/shell/src/execution/routing/dispatch/exec.rs`.
- Same-user operator surfaces already exist for world and gateway lifecycle:
  `scripts/substrate/world-enable.sh`,
  `scripts/substrate/dev-install-substrate.sh`,
  `scripts/substrate/install-substrate.sh`,
  `scripts/substrate/uninstall-substrate.sh`,
  `substrate host doctor`,
  `substrate world doctor`,
  `substrate health`, and
  `substrate world gateway sync|status|restart`.
- Ownership-boundary gaps still live in host lifecycle, Lima ownership,
  transport reachability, and support workflow surfaces such as
  `scripts/mac/lima-warm.sh`,
  `docs/WORLD.md`,
  `docs/contracts/substrate-gateway-operator-contract.md`,
  `docs/contracts/substrate-gateway-policy-evaluation.md`, and
  `docs/contracts/substrate-gateway-status-schema.md`.
- Managed gateway runtime artifacts under
  `/run/substrate/substrate-gateway-runtime/` already matter in the supported
  operator story. Ownership separation must eventually cover how those
  artifacts, lifecycle logs, and brokered status flows are exposed without
  preserving same-user Lima control.

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
  It resolves transport drift, policy drift, mount overexposure, gateway
  runtime parity, unit drift, and direct-guest operational shortcuts.
- The ownership-separated track depends on the same-user track for transport and
  operational clarity, but then replaces the host ownership model with a
  daemon-owned or service-account-owned control plane.
- Work in the ownership-separated track should not reintroduce any direct guest
  admin path as part of the normal operator contract.
- `docs/WORLD.md` remains architecture evidence and descriptive context. It is
  not the contract owner for gateway lifecycle/status or policy semantics.
  Those durable contracts live under `docs/contracts/` and must stay the source
  of truth when these plans discuss operator/status behavior.

## Required Evidence Sources

All SOWs in this tree should stay anchored to the current repo and research
inputs:

- [Research note](./research/2026-04-28-macos-lima-parity-lockdown.md)
- [`../docs/WORLD.md`](../docs/WORLD.md)
- [`../docs/contracts/substrate-gateway-operator-contract.md`](../docs/contracts/substrate-gateway-operator-contract.md)
- [`../docs/contracts/substrate-gateway-policy-evaluation.md`](../docs/contracts/substrate-gateway-policy-evaluation.md)
- [`../docs/contracts/substrate-gateway-status-schema.md`](../docs/contracts/substrate-gateway-status-schema.md)
- [`../docs/contracts/substrate-gateway-runtime-parity.md`](../docs/contracts/substrate-gateway-runtime-parity.md)
- `crates/world-mac-lima/src/lib.rs`
- `crates/world-mac-lima/src/forwarding.rs`
- `crates/world-mac-lima/src/transport.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/world-service/src/gateway_runtime.rs`
- `scripts/mac/lima-warm.sh`
- `scripts/mac/lima/substrate.yaml`
- `scripts/mac/smoke.sh`
- `scripts/mac/lima-doctor.sh`
- `scripts/substrate/world-enable.sh`
- `scripts/substrate/dev-install-substrate.sh`
- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/uninstall-substrate.sh`

## Exit Condition

This planning tree is complete when:

- every phase has a scoped overview
- every milestone has a concrete SOW
- dependencies between same-user hardening and ownership separation are explicit
- acceptance criteria and validation evidence are defined per milestone
- the plans distinguish clearly between:
  - current same-user world/gateway lifecycle surfaces that already exist
  - unresolved ownership, mount, and direct-guest gaps that still block the
    stronger macOS claim
- cross-document consistency has been reviewed after the draft wave lands
