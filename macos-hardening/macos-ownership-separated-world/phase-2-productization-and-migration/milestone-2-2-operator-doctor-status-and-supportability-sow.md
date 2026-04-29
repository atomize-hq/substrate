# Milestone 2.2: Operator Doctor, Status, and Supportability

Status: Draft

Phase: 2 - Productization and Migration

Last updated: 2026-04-28

## Purpose / Outcome

Deliver the canonical macOS operator surfaces for the ownership-separated world:
health, lifecycle status, support evidence, repair guidance, and bounded
breakglass. After this milestone, normal macOS support work should be possible
through Substrate-owned commands and docs rather than direct guest entry.

## Why This Milestone Exists

Today’s macOS support posture is split across:

- `crates/shell/src/execution/platform/macos.rs` doctor logic
- `scripts/mac/lima-doctor.sh` guest-shell-based checks
- `scripts/mac/smoke.sh` as the main proof harness
- `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` troubleshooting
  sections that still point operators at guest `systemctl`, guest socket curls,
  and `limactl shell`

That is acceptable for a same-user prototype, but not for a product whose core
claim is host-side ownership separation. Operators need one authoritative status
story that explains whether the new owner, transport, world agent, broker path,
migration state, and runtime health are correct.

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

This milestone makes the CLI the contract owner and demotes helper scripts to
followers:

1. `substrate host doctor --json` reports host prerequisites, owner-model
   state, lifecycle state, and broker reachability.
2. `substrate world doctor --json` reports the host view plus in-world
   readiness through the supported broker path.
3. `substrate health --json` summarizes operator actionability and must explain
   whether the problem is install state, owner state, transport state, world
   state, or migration state.
4. A dedicated lifecycle/status surface is added for macOS ownership-separated
   deployments. If implemented as `substrate world status --json`, it becomes
   the machine-readable contract for support tooling. If the team chooses not to
   add a new subcommand, the equivalent schema must still exist in a canonical
   CLI JSON output.
5. `scripts/mac/lima-doctor.sh` becomes a support-engineering or breakglass
   helper, not the primary operator entrypoint.

## Dependencies / Sequencing

Depends on:

- the same-user hardening track milestones required by the parent phase:
  - phase 2 milestone 2.2 mount minimization and ingress contract
  - phase 3 milestone 3.1 Substrate-managed diagnostics and lifecycle
  - phase 3 milestone 3.2 breakglass reclassification and doc cutover
- Milestone 2.1 install and migration state definitions
- Phase 1 host ownership separation and broker architecture

Sequences before:

- Milestone 2.3, because the GA matrix needs stable status semantics and
  support evidence surfaces

## Concrete Repo Surfaces and File Pointers

Primary surfaces:

- `crates/shell/src/execution/platform/macos.rs`
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- `docs/WORLD.md`
- `docs/cross-platform/mac_world_setup.md`
- `docs/INSTALLATION.md`
- `docs/UNINSTALL.md`

Related implementation surfaces likely touched by downstream execution:

- `crates/world-mac-lima/src/lib.rs`
- `crates/world-mac-lima/src/forwarding.rs`
- `crates/world-mac-lima/src/transport.rs`
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
  state, and world readiness without using direct guest commands.
- CLI JSON output distinguishes at least these macOS states:
  clean ownership-separated install, migrated install, partial migration,
  unavailable world backend, and breakglass-required failure.
- Human-readable doctor/health output explains next actions concretely instead
  of only reporting that a check failed.
- `scripts/mac/lima-doctor.sh` no longer defines the normal support contract.
- The published docs clearly separate normal-path support commands from
  breakglass-only procedures.

## Validation / Evidence Plan

- Capture `substrate host doctor --json`, `substrate world doctor --json`, and
  `substrate health --json` from:
  - healthy clean install
  - healthy migrated install
  - broken broker path
  - broken world-agent path
  - incomplete migration state
- Expand `scripts/mac/smoke.sh` evidence to assert the new doctor/status schema
  and human-readable remediation posture.
- If a standalone status surface is introduced, capture its JSON output in the
  same scenarios above and document it in `docs/WORLD.md`.
- Compare the new CLI outputs against the legacy `scripts/mac/lima-doctor.sh`
  checks to ensure the script is no longer the only place where critical macOS
  health signals exist.

## Risks / Open Questions

- Where should ownership and migration state live in the CLI schema so it is
  stable enough for support automation but does not duplicate existing doctor
  payloads awkwardly?
- How much breakglass detail should be exposed in normal operator docs before it
  starts undermining the product boundary?
- If helper scripts remain for deep support, how do we keep them from drifting
  away from the CLI contract again?
- What is the minimum viable support bundle for macOS incidents without
  reintroducing direct access to owner-private control-plane material?
