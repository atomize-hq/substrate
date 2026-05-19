# Milestone 1.1: Transport Contract Unification SOW

Status: Draft

Owner: Substrate world backend / macOS transport layer

Last updated: 2026-05-19

## Purpose / Outcome

Define and implement one canonical host-to-guest transport contract for the
macOS Lima backend so that execution, readiness probing, gateway lifecycle, and
diagnostics all describe the same path.

The concrete outcome is that the guest endpoint contract is unambiguously
`/run/substrate.sock`, with VSock and SSH UDS as ways to reach that socket from
the host. The remaining transport work is to remove stale `7788` references and
to reclassify or eliminate retained host TCP `17788` compatibility probes.

## Why This Milestone Exists

Transport behavior is partly unified already, but the repo still describes it
incoherently.

- `crates/world-mac-lima/src/forwarding.rs` already forwards into the guest UDS
  endpoint `/run/substrate.sock` and intentionally skips SSH TCP fallback.
- `crates/world-mac-lima/src/transport.rs` still advertises `127.0.0.1:7788`
  for TCP and `MacLimaBackend::test_agent_connection(...)` still probes that
  stale port.
- `crates/shell/src/execution/platform/macos.rs` and
  `crates/shell/src/builtins/world_gateway.rs` prefer the host UDS when
  available, then fall back to host TCP `17788` or in-VM probing.
- Host-side `SUBSTRATE_WORLD_SOCKET` override use bypasses the normal Lima path
  and should be treated as advanced/test/breakglass rather than as the default
  transport contract.

This creates false confidence: macOS can appear "working" while different code
paths are probing different endpoints.

## In Scope

- Unify transport constants, endpoint resolution, and forwarding metadata used
  by the macOS backend.
- Make routed execution and readiness probes consume the same selected
  transport.
- Keep the supported story centered on UDS-backed routing rather than on guest
  TCP reachability.
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
- canonical host default path: the managed host UDS under
  `~/.substrate/sock/agent.sock` when present, otherwise one selected
  forwarding mode exposed by `world-mac-lima`
- canonical forwarding modes:
  - VSock proxy exposing host TCP `127.0.0.1:17788` into guest UDS
  - SSH UDS forwarding into the same guest UDS
- unsupported default path: SSH TCP fallback
- canonical health/readiness contract: probe the selected routed transport
  first, then classify any retained host TCP `17788` probing as
  compatibility-only until removed

Implementation should centralize the selected transport description in
`crates/world-mac-lima/src/transport.rs` or an adjacent shared type, then make
`crates/world-mac-lima/src/lib.rs`,
`crates/world-mac-lima/src/forwarding.rs`,
`crates/shell/src/execution/platform/macos.rs`, and
`crates/shell/src/builtins/world_gateway.rs` consume that one source of truth.

If host TCP remains anywhere after centralization, it must use one documented
port constant, one explicit reason for existence, and must not be described as
the default supported Lima contract.

## Dependencies / Sequencing

- This is the first runtime milestone in Phase 1.
- Milestone 1.2 depends on this milestone because policy parity must ride the
  same transport contract for PTY and non-PTY execution.
- Milestone 1.3 depends on this milestone because doctor and smoke evidence
  cannot be authoritative while transport probing remains split.

## Concrete Repo Surfaces and File Pointers

Primary implementation surfaces:

- `crates/world-mac-lima/src/transport.rs`
- `crates/world-mac-lima/src/forwarding.rs`
- `crates/world-mac-lima/src/lib.rs`
- `crates/shell/src/execution/platform/macos.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/shell/src/execution/routing/dispatch/exec.rs`

Current drift evidence:

- `crates/world-mac-lima/src/forwarding.rs:83`
- `crates/world-mac-lima/src/forwarding.rs:189`
- `crates/world-mac-lima/src/transport.rs:20`
- `crates/world-mac-lima/src/lib.rs:206`
- `crates/shell/src/execution/platform/macos.rs:369`
- `crates/shell/src/builtins/world_gateway.rs:178`

Secondary doc/script surfaces that must follow the contract:

- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- `docs/WORLD.md`

## Deliverables

- One shared transport model for the macOS Lima backend.
- Elimination of contradictory TCP / VSock / UDS constants and endpoint strings.
- A consistent agent-client endpoint construction path for routed execution.
- Doctor/reporting output that can state which transport was selected and
  whether that transport is healthy.
- Tests covering transport selection and endpoint derivation.

## Acceptance Criteria

- No remaining macOS backend code path disagrees on the host compatibility port
  used for VSock-proxy routing, and no stale `7788` references remain.
- `MacLimaBackend` derives its agent endpoint from the same selected forwarding
  contract used by readiness probing.
- `substrate host doctor`, `substrate world doctor`, and
  `substrate world gateway status` do not define a different supported endpoint
  contract than routed command execution.
- PTY and non-PTY macOS execution paths continue to work after transport
  centralization.
- Documentation describes one transport ordering, one default path, and one
  explicitly-classified fallback story.
- Host-side `SUBSTRATE_WORLD_SOCKET` override use is documented as
  advanced/test/breakglass, not as the default Lima path.

## Validation / Evidence Plan

Required evidence for this milestone:

- targeted unit tests for `transport.rs` and forwarding endpoint construction
- `cargo test -p world-mac-lima`
- targeted shell tests for macOS doctor reporting if transport fields are
  surfaced there
- targeted shell tests for `substrate world gateway` macOS endpoint resolution
- `scripts/mac/lima-doctor.sh`
- `substrate host doctor --json`
- `substrate world doctor --json`
- `substrate world gateway status --json`

Evidence should explicitly show which transport was selected and that the same
selection is used for command execution, gateway lifecycle/status, and doctor
probing.

## Risks / Open Questions

- The current shell doctor implementation has its own probing logic; deciding
  whether to reuse backend transport code directly or mirror it via a shared
  utility needs care to avoid cyclic dependencies.
- Host TCP `17788` use exists today as a VSock-proxy and compatibility probe
  surface; the team needs to decide whether to keep that probe temporarily,
  route everything through shared endpoint resolution, or remove it.
- VSock availability varies with host setup; the contract must be stable even
  when VSock is absent.
