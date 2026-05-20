# Milestone 2.2: Operator Doctor, Status, and Supportability

Status: Draft

Phase: 2 - Productization and Migration

Last updated: 2026-05-19

## Purpose / Outcome

Deliver the canonical macOS operator surfaces for the ownership-separated
world: health, lifecycle status, support evidence, repair guidance, and bounded
breakglass. After this milestone, normal macOS support work should be possible
through Substrate-owned commands and docs rather than direct guest entry.

## Why This Milestone Exists

Today’s macOS support posture is split across:

- `crates/shell/src/execution/platform/macos.rs` doctor logic
- `crates/shell/src/builtins/world_gateway.rs` gateway lifecycle/status logic
- `scripts/mac/lima-doctor.sh` guest-shell-based checks
- `scripts/mac/smoke.sh` as the main proof harness
- `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` troubleshooting
  sections that still point operators at guest `systemctl`, guest socket curls,
  and `limactl shell`

That is acceptable for a same-user prototype, but not for a product whose core
claim is host-side ownership separation. Operators need one authoritative
status story that explains whether the new owner, transport, world agent,
gateway path, migration state, and runtime health are correct.

## In Scope

- macOS host/world doctor surfaces for ownership-separated deployments
- machine-readable lifecycle/status surfaces needed by support and automation
- operator-visible remediation guidance for the expected failure classes
- evidence-collection flows for support cases
- explicit normal-path vs breakglass guidance in docs
- convergence or retirement plan for `scripts/mac/lima-doctor.sh`

## Out of Scope

- initial installer and migration mechanics
- GA release gating and the full validation matrix
- ad hoc debug tooling that bypasses the ownership boundary without controls
- redefining Linux or Windows doctor semantics except where parity language is
  needed for shared docs

## Architectural Approach

This milestone makes the CLI and durable contract docs the owner lines, and
demotes helper scripts to followers:

1. `substrate host doctor --json` reports host prerequisites, owner-model
   state, lifecycle state, and broker reachability.
2. `substrate world doctor --json` reports the host view plus in-world
   readiness through the supported broker path.
3. `substrate health --json` summarizes operator actionability and must explain
   whether the problem is install state, owner state, transport state, world
   state, gateway state, or migration state.
4. `substrate world gateway status --json` remains the authoritative
   machine-readable gateway wiring surface. This milestone must consume that
   contract, not replace it.
5. If macOS-specific lifecycle state needs additive JSON beyond the existing
   gateway status envelope, it should live in the appropriate doctor/health or
   separate CLI surface without redefining the gateway contract.
6. `scripts/mac/lima-doctor.sh` becomes a support-engineering or breakglass
   helper, not the primary operator entrypoint.

## Dependencies / Sequencing

Depends on:

- the same-user hardening track milestones required by the parent phase:
  - phase 2 milestone 2.2 mount minimization and ingress contract
  - phase 3 milestone 3.1 Substrate-managed diagnostics and lifecycle
  - phase 3 milestone 3.2 breakglass reclassification and doc cutover
- milestone 2.1 install and migration state definitions
- phase 1 host ownership separation and broker architecture

Sequences before:

- milestone 2.3, because the GA matrix needs stable status semantics and
  support evidence surfaces

## Concrete Repo Surfaces and File Pointers

Primary surfaces:

- `crates/shell/src/execution/platform/macos.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- `../../../docs/WORLD.md`
- `../../../docs/cross-platform/mac_world_setup.md`
- `../../../docs/INSTALLATION.md`
- `../../../docs/UNINSTALL.md`
- `../../../docs/contracts/substrate-gateway-operator-contract.md`
- `../../../docs/contracts/substrate-gateway-status-schema.md`
- `../../../docs/contracts/substrate-gateway-policy-evaluation.md`

Related implementation surfaces likely touched by downstream execution:

- `crates/world-mac-lima/src/lib.rs`
- `crates/world-mac-lima/src/forwarding.rs`
- `crates/world-mac-lima/src/transport.rs`
- `crates/world-agent/src/gateway_runtime.rs`
- `scripts/mac/lima-warm.sh`

## Deliverables

- a macOS ownership-separated doctor/status contract
- CLI JSON fields that identify:
  - ownership model
  - install/migration state
  - broker reachability
  - world-agent reachability through the supported path
  - breakglass requirement when normal-path repair is exhausted
- operator-facing health output with actionable remediation text
- a support evidence collection recipe rooted in CLI outputs and bounded logs
- rewritten troubleshooting guidance in `docs/WORLD.md` and
  `docs/cross-platform/mac_world_setup.md`
- a defined role for `scripts/mac/lima-doctor.sh`: parity follower, support
  tool, or breakglass helper

## Acceptance Criteria

- A supported operator can determine install state, owner-model state, broker
  state, gateway state, and world readiness without using direct guest
  commands.
- CLI JSON output distinguishes at least these macOS states:
  clean ownership-separated install, migrated install, partial migration,
  unavailable world backend, unavailable gateway component, and
  breakglass-required failure.
- Human-readable doctor/health output explains next actions concretely instead
  of only reporting that a check failed.
- `scripts/mac/lima-doctor.sh` no longer defines the normal support contract.
- The published docs clearly separate normal-path support commands from
  breakglass-only procedures.
- Managed gateway runtime artifact paths under
  `/run/substrate/substrate-gateway-runtime/` are surfaced as part of the
  supported operator story when lifecycle failures reference them.

## Validation / Evidence Plan

- Capture `substrate host doctor --json`, `substrate world doctor --json`, and
  `substrate health --json` from:
  - healthy clean install
  - healthy migrated install
  - broken broker path
  - broken world-agent path
  - incomplete migration state
- Capture `substrate world gateway status --json` in the same scenarios above,
  plus after `substrate world gateway sync` and `restart`.
- Expand `scripts/mac/smoke.sh` evidence to assert the doctor/status schema and
  human-readable remediation posture.
- Compare the new CLI outputs against the legacy `scripts/mac/lima-doctor.sh`
  checks to ensure the script is no longer the only place where critical macOS
  health signals exist.
- Treat `docs/WORLD.md` as descriptive context only; validate gateway lifecycle
  meaning against the durable contract docs under `docs/contracts/`.

## Risks / Open Questions

- Where should ownership and migration state live in the CLI schema so it is
  stable enough for support automation but does not duplicate existing doctor
  payloads awkwardly?
- How much breakglass detail should be exposed in normal operator docs before
  it starts undermining the product boundary?
- If helper scripts remain for deep support, how do we keep them from drifting
  away from the CLI contract again?
- What is the minimum viable support bundle for macOS incidents without
  reintroducing direct access to owner-private control-plane material?
