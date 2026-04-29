# macOS Hardened Same-User Lima

Status: Draft

Owner: Substrate world backend / macOS hardening

Last updated: 2026-04-28

## Purpose / outcome

Define the hardening program for the current macOS Lima-backed world backend without pretending it can match Linux's ownership boundary. The outcome of this feature is a documented target mode that preserves Linux-like execution semantics where possible, explicitly names the irreducible same-user limitations, and stages the repo changes required to stop treating direct guest administration and permissive transports as normal operation.

## Why this feature exists

The current macOS path is functionally close to Linux, but its security and operator contract are still parity-by-assertion rather than parity-by-enforcement.

- `crates/world-mac-lima/src/lib.rs` injects a synthetic permissive `PolicySnapshotV3` and leaves `apply_policy` as a no-op.
- `crates/world-mac-lima/src/forwarding.rs` and `crates/world-mac-lima/src/transport.rs` disagree on TCP endpoints (`17788` vs `7788`) and still carry fallback logic that is not the hardened default.
- `scripts/mac/lima/substrate.yaml` mounts all of `$HOME` read-only and leaves guest service administration patterns closer to ad hoc VM management than to Substrate-owned lifecycle control.
- `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` still normalize `limactl shell`, direct `systemctl`, and direct guest probes as routine operator paths.

Phase 0 exists to freeze the security contract before implementation work starts, because the codebase currently mixes three different stories:

- Linux-like operator semantics.
- Same-user Lima convenience shortcuts.
- Breakglass guest administration that is documented as normal.

## In-scope

- Define the supported target mode for hardened same-user Lima.
- Document which Linux properties are expected to match and which cannot match under the current same-user VM model.
- Define the breakglass boundary for direct guest administration and manual transport debugging.
- Establish milestone sequencing for later code, docs, and validation work.

## Out-of-scope

- Implementing backend, transport, provisioning, or docs changes outside this feature directory.
- Designing a true multi-user macOS boundary equivalent to Linux `root:substrate 0660`.
- Replacing Lima with a different macOS virtualization backend in this feature.
- Solving Windows or Linux transport issues except where they are used as parity references.

## Architectural approach

The feature proceeds contract-first.

1. Phase 0 locks the target mode, support contract, version floor inputs, and breakglass rules.
2. Later phases will converge implementation to that contract:
   - transport and policy semantics in `crates/world-mac-lima`
   - provisioning and unit ownership in `scripts/mac/lima*`
   - operator surface reduction in `docs/WORLD.md`, `docs/cross-platform/mac_world_setup.md`, and CLI-owned workflows
3. Validation shifts from "manual `limactl` can make it work" to "Substrate-owned entry points prove the supported mode, and anything else is breakglass."

The key design constraint is explicit: same-user Lima can match Linux execution behavior and guest-local socket ACLs, but it cannot provide the same host-side ownership boundary while the host user can always drive `limactl` for that VM.

## Dependencies / sequencing

- Phase 0 is the gate for all later hardening work.
- Milestone 0.1 defines target mode and support posture.
- Milestone 0.2 defines Lima version assumptions and breakglass classification.
- Later implementation phases should not change transport defaults, guest mounts, policy application, or operator docs until phase-0 decisions are accepted.

## Concrete repo surfaces and file pointers

- Backend contract: `crates/world-mac-lima/src/lib.rs`
- Forwarding and endpoint selection: `crates/world-mac-lima/src/forwarding.rs`, `crates/world-mac-lima/src/transport.rs`
- Lima profile and guest units: `scripts/mac/lima/substrate.yaml`
- Provisioning and readiness workflow: `scripts/mac/lima-warm.sh`, `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`
- Operator-facing architecture and setup guidance: `docs/WORLD.md`, `docs/cross-platform/mac_world_setup.md`
- Phase-0 planning docs: `macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/`

## Deliverables

- This feature overview.
- Four phase overview documents that sequence the work from contract-setting
  through operator-surface replacement.
- Ten milestone SOWs covering:
  - target mode and support contract
  - Lima version and breakglass contract
  - transport contract unification
  - policy application parity
  - doctor and smoke parity
  - agent surface hardening
  - mount minimization and ingress definition
  - guest service sandboxing and unit unification
  - Substrate-managed diagnostics and lifecycle
  - breakglass reclassification and docs cutover

## Acceptance criteria

- The feature docs state plainly that Linux ownership-boundary parity is not achievable in the current same-user Lima model.
- The docs distinguish supported operation, degraded operation, and breakglass operation.
- The docs identify the exact repo surfaces that later implementation phases must change.
- The docs sequence future work so transport, policy, provisioning, and operator docs converge on one contract rather than separate local fixes.

## Validation / evidence plan

- Cross-check every claimed gap against the cited repo surfaces before phase promotion.
- Use the phase-0 README and milestone SOWs as the review set for security, backend, and docs owners.
- Treat this feature as ready for implementation only when reviewers can answer three questions from the docs alone:
  - What is the supported same-user mode?
  - What is explicitly not promised relative to Linux?
  - Which current workflows become breakglass?

## Risks / open questions

- The repo currently documents and tests workflows that bypass the intended socket-activation boundary; unwinding those habits will require coordinated code and docs changes.
- A minimum supported Lima version is not yet frozen here; that decision is delegated to milestone 0.2 because it changes what transport and guest-management paths can be required.
- If the project later needs true host-side multi-user isolation on macOS, this feature will only be an intermediate hardening step, not the final security model.

## Phase Index

- [Phase 0: Security Contract and Scope](./phase-0-security-contract-and-scope/README.md)
- [Phase 1: Runtime Parity Foundation](./phase-1-runtime-parity-foundation/README.md)
- [Phase 2: Same-User Hardening](./phase-2-same-user-hardening/README.md)
- [Phase 3: Substrate-Owned Operations](./phase-3-substrate-owned-operations/README.md)
