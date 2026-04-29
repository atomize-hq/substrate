# Phase 0: Security Contract and Scope

Status: Draft  
Last updated: 2026-04-28

## Purpose / outcome

Freeze the security and support contract for hardened same-user Lima before any implementation phases change code or operator workflows. The outcome of phase 0 is one unambiguous planning baseline for what macOS should guarantee, what it cannot guarantee, which workflows are supported, and which existing guest-administration paths are reclassified as breakglass.

## Why this phase exists

The current macOS backend is already close enough to Linux behavior that implementation changes are tempting, but the repo still lacks a locked contract for the following disagreements:

- The code claims Linux-like policy semantics while `MacLimaBackend` still synthesizes a permissive policy snapshot in `crates/world-mac-lima/src/lib.rs`.
- Transport selection is not fully coherent across `crates/world-mac-lima/src/forwarding.rs` and `crates/world-mac-lima/src/transport.rs`.
- The Lima profile and setup docs still normalize broad host mounts and manual guest administration.
- Doctor and smoke workflows validate "it can be made to work" more than "it is operating in the supported hardened mode."

If phase 0 is skipped, later work risks hardening one layer while another layer keeps the old, more permissive support story.

## In-scope

- Define the target security posture for same-user Lima.
- Define the support boundary versus Linux parity claims.
- Define breakglass operator rules.
- Define the versioning and environment assumptions that later implementation can rely on.
- Produce milestone SOWs that later code phases can execute against.

## Out-of-scope

- Editing runtime code, provisioning scripts, or top-level docs outside this phase directory.
- Closing the current transport, policy, or mount gaps in this phase.
- Replacing Lima, redesigning the entire macOS backend, or solving multi-user host isolation.

## Architectural approach

Phase 0 uses two planning milestones:

1. `milestone-0-1-target-mode-and-support-contract-sow.md`
   - names the supported mode
   - names the non-goals
   - freezes the Linux-versus-macOS claims we will and will not make
2. `milestone-0-2-lima-version-and-breakglass-contract-sow.md`
   - defines required Lima/runtime assumptions
   - defines which direct guest workflows remain available only for breakglass
   - defines which Substrate-owned workflows must become the normal operator path

This phase is complete when later implementation can treat the phase docs as authoritative inputs instead of rediscovering policy, transport, and ops intent from scattered code comments and setup guides.

## Dependencies / sequencing

- Depends on the current macOS state captured in:
  - `crates/world-mac-lima/src/lib.rs`
  - `crates/world-mac-lima/src/forwarding.rs`
  - `crates/world-mac-lima/src/transport.rs`
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/lima/substrate.yaml`
  - `docs/WORLD.md`
  - `docs/cross-platform/mac_world_setup.md`
  - `scripts/mac/lima-doctor.sh`
  - `scripts/mac/smoke.sh`
- Milestone 0.1 must land conceptually before milestone 0.2, because the version and breakglass contract depend on the chosen target mode.
- No later hardening phase should change default transport, guest mounts, or operator guidance until both milestone SOWs are accepted.

## Concrete repo surfaces and file pointers

- Policy snapshot injection and backend policy no-op: `crates/world-mac-lima/src/lib.rs`
- Forwarding endpoint and transport fallback behavior: `crates/world-mac-lima/src/forwarding.rs`, `crates/world-mac-lima/src/transport.rs`
- Lima guest image, mount, and unit definitions: `scripts/mac/lima/substrate.yaml`
- Provisioning and lifecycle commands: `scripts/mac/lima-warm.sh`
- Troubleshooting and smoke entry points: `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`
- Operator-facing macOS setup and world architecture docs: `docs/cross-platform/mac_world_setup.md`, `docs/WORLD.md`

## Deliverables

- Phase overview README.
- Milestone 0.1 SOW.
- Milestone 0.2 SOW.
- A phase-local issue ledger, embedded in those SOWs, that later implementation phases can use as their scope baseline.

## Acceptance criteria

- The phase documents define one supported same-user Lima posture and reject vague "Linux parity except where different" language.
- The phase documents identify the exact repo surfaces that encode the current mismatched behavior.
- The phase documents distinguish supported operator flows from breakglass flows.
- The milestone ordering is clear enough that a future implementation owner can start with milestone 0.1 decisions and then execute 0.2 without reopening scope.

## Validation / evidence plan

- Review the phase docs against the cited repo surfaces and confirm each claimed gap is observable today.
- Require sign-off from backend, security, and docs owners before phase completion.
- Use later implementation-phase kickoffs to reference the phase-0 decisions by file path and section, not by verbal summary.

## Risks / open questions

- Some breakglass workflows are currently embedded in normal setup docs, so reclassification will create short-term friction for operators.
- The right Lima version floor may force changes in transport assumptions, especially around `vsock-proxy` availability and SSH config stability.
- The project may need an ADR later if phase-0 decisions materially narrow the macOS support matrix.
