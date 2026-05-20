# macOS Hardened Same-User Lima

Status: Draft

Owner: Substrate world backend / macOS hardening

Last updated: 2026-05-19

## Purpose / outcome

Define the hardening program for the current macOS Lima-backed world backend without pretending it can match Linux's ownership boundary. The target mode keeps the already-landed Substrate operator surfaces as the baseline, names the irreducible same-user limitations, and sequences the remaining repo work needed to make the supported path narrower and clearer.

## Why this feature exists

The current macOS path is much closer to Linux parity than these docs currently admit:

- `substrate host doctor` and `substrate world doctor` already exist as canonical health surfaces.
- `substrate world gateway sync`, `substrate world gateway status`, and `substrate world gateway restart` already exist as canonical gateway lifecycle/status surfaces.
- Shared-world/orchestration support already exists on the Lima-backed path, including explicit shared-owner proof handling and macOS orchestration smoke coverage.

The remaining macOS problem is no longer "there is no Substrate-owned surface." It is that the hardened contract is still incomplete in several concrete places:

- `crates/world-mac-lima/src/lib.rs` still synthesizes a permissive `PolicySnapshotV3` in `MacLimaBackend::convert_exec_request` and leaves `apply_policy` as a no-op on the backend-mediated Lima path.
- The shell-side routed request builders already resolve and forward `policy_snapshot`, `world_network`, and `world_fs_mode` on direct routed paths, so the parity gap must be scoped to `MacLimaBackend` and other backend-mediated flows rather than described as a repo-wide absence of policy propagation.
- `scripts/mac/lima/substrate.yaml` still mounts broad host state, including a read-only `$HOME` mount that is wider than the hardened runtime contract should require.
- `scripts/mac/lima-warm.sh` still writes `SUBSTRATE_AGENT_TCP_PORT=61337`, leaving an extra guest listener enabled by default.
- Guest unit/socket definitions still drift between `scripts/mac/lima/substrate.yaml` and `scripts/mac/lima-warm.sh`.
- `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` still normalize direct guest setup and troubleshooting more than the hardened same-user path should.
- `SUBSTRATE_WORLD_SOCKET` is still available as an advanced/test/breakglass bypass, but it is not the standard Lima-backed operator path and already rejects explicit shared-owner reuse on macOS.

Phase 0 exists to lock that support contract before implementation work continues, because the repo still mixes three different stories:

- a real Substrate-owned CLI surface that already exists
- same-user Lima implementation shortcuts that are still tolerated
- direct guest administration that is still too prominent in docs and troubleshooting

## In-scope

- Define the supported target mode for hardened same-user Lima.
- Document which Linux properties are expected to match and which cannot match under the current same-user VM model.
- Define the boundary between supported operation, degraded-but-supported operation, and breakglass operation.
- Establish milestone sequencing for backend, provisioning, and docs changes that remain open.

## Out-of-scope

- Implementing backend, transport, provisioning, or docs changes outside this feature directory.
- Designing a true multi-user macOS boundary equivalent to Linux `root:substrate 0660`.
- Replacing Lima with a different macOS virtualization backend in this feature.
- Solving Windows or Linux transport issues except where they are used as parity references.

## Architectural approach

The feature proceeds contract-first.

1. Phase 0 locks the target mode, support posture, version-floor assumptions, and breakglass rules.
2. Phase 1 converges backend-mediated runtime behavior with the already-landed shell/operator surfaces:
   - transport behavior in `crates/world-mac-lima`
   - backend policy application semantics for Lima-backed execution
   - doctor/smoke evidence for the supported path
3. Phase 2 narrows the same-user Lima attack surface:
   - remove default extra TCP exposure
   - minimize broad host mounts
   - unify guest unit/socket definitions
4. Phase 3 finishes the operator-surface cutover:
   - teach CLI-owned doctor/gateway/lifecycle flows first
   - reclassify direct guest setup and troubleshooting as breakglass

The key design constraint is explicit: same-user Lima can match much of Linux's execution behavior, guest-local socket contract, gateway lifecycle surface, and shared-world/orchestration flow shape, but it still cannot provide Linux's host-side ownership boundary while the same host user can drive `limactl` for that VM.

## Dependencies / sequencing

- Phase 0 is the gate for all later hardening work.
- Later phases should treat `substrate host doctor`, `substrate world doctor`, and `substrate world gateway sync|status|restart` as baseline surfaces to harden around, not as future inventions.
- Later implementation phases should not broaden the operator contract beyond what the current CLI already owns; they should narrow Lima-specific bypasses and direct guest dependencies instead.

## Concrete repo surfaces and file pointers

- Backend contract: `crates/world-mac-lima/src/lib.rs`
- Forwarding and endpoint selection: `crates/world-mac-lima/src/forwarding.rs`, `crates/world-mac-lima/src/transport.rs`
- Shell-side routed request builders and policy/world input propagation:
  `crates/shell/src/execution/routing/dispatch/world_ops.rs`,
  `crates/shell/src/repl/async_repl.rs`,
  `crates/shell/src/builtins/world_gateway.rs`
- Lima profile and guest units: `scripts/mac/lima/substrate.yaml`
- Provisioning and readiness workflow: `scripts/mac/lima-warm.sh`, `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`, `scripts/mac/orchestration-smoke.sh`
- Operator-facing architecture and setup guidance: `docs/WORLD.md`, `docs/cross-platform/mac_world_setup.md`, `docs/USAGE.md`, `docs/contracts/substrate-gateway-operator-contract.md`
- Phase overviews: `macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/`

## Deliverables

- This feature overview.
- Four phase overview documents that sequence the work from contract-setting through operator-surface replacement.
- Milestone SOWs that focus later work on the remaining gaps:
  - same-user Lima ownership boundary
  - backend-mediated policy parity
  - broad `$HOME` mount reduction
  - default `SUBSTRATE_AGENT_TCP_PORT=61337` removal
  - YAML versus warm-script unit convergence
  - direct guest setup/troubleshooting demotion to breakglass

## Acceptance criteria

- The feature docs state plainly that Linux ownership-boundary parity is not achievable in the current same-user Lima model.
- The docs treat `substrate host doctor`, `substrate world doctor`, and `substrate world gateway sync|status|restart` as already-landed baseline surfaces.
- The docs scope policy-synthesis concerns to `MacLimaBackend` and backend-mediated Lima paths, while acknowledging that shell-side routed request builders already carry resolved policy/world inputs.
- The docs describe shared-world/orchestration support as current functionality, not speculative future work.
- The docs identify `SUBSTRATE_WORLD_SOCKET` as an advanced/test/breakglass bypass rather than the normal Lima-backed path.

## Validation / evidence plan

- Cross-check every claimed gap against the cited repo surfaces before phase promotion.
- Use the current CLI and smoke surfaces as the baseline evidence set:
  - `substrate host doctor --json`
  - `substrate world doctor --json`
  - `substrate world gateway status --json`
  - `scripts/mac/smoke.sh`
  - `scripts/mac/orchestration-smoke.sh`
- Treat this feature as ready for implementation only when reviewers can answer three questions from the docs alone:
  - What is the supported same-user mode?
  - What is explicitly not promised relative to Linux?
  - Which current workflows are normal operation versus breakglass?

## Risks / open questions

- The repo already has real CLI-owned surfaces, so the main risk is overstating the amount of missing architecture and then widening scope unnecessarily.
- A minimum supported Lima version is not yet frozen here; that decision affects which forwarding and guest-management paths can be required.
- If the project later needs true host-side multi-user isolation on macOS, this feature remains an intermediate hardening step rather than the final security model.

## Phase Index

- [Phase 0: Security Contract and Scope](./phase-0-security-contract-and-scope/README.md)
- [Phase 1: Runtime Parity Foundation](./phase-1-runtime-parity-foundation/README.md)
- [Phase 2: Same-User Hardening](./phase-2-same-user-hardening/README.md)
- [Phase 3: Substrate-Owned Operations](./phase-3-substrate-owned-operations/README.md)
