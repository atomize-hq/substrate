# Milestone 0.1: Target Mode and Support Contract SOW

Status: Draft  
Last updated: 2026-05-19

## Purpose / outcome

Define the authoritative target mode for hardened same-user Lima and freeze the support contract that all later macOS hardening work must satisfy. The milestone outcome is a decision-ready contract that says exactly what the supported mode is, what Linux properties it is expected to emulate, and which Linux guarantees are explicitly unavailable under the current same-user VM ownership model.

## Why this milestone exists

Today the macOS backend mixes partially landed parity with security and support
claims that are still too broad.

- `crates/world-mac-lima/src/lib.rs` already supports routed execution and
  shared-world session reuse, but `MacLimaBackend` still synthesizes a
  permissive backend-local `PolicySnapshotV3` and `apply_policy(...)` is still
  effectively just `fs_mode` storage.
- Shell-side direct request builders already resolve and carry
  broker-derived `policy_snapshot`, `world_network`, and world-fs enforcement
  inputs in `crates/shell/src/execution/policy_snapshot.rs`,
  `crates/shell/src/execution/routing/dispatch/world_ops.rs`, and
  `crates/shell/src/builtins/world_gateway.rs`; the overstatement is specifically
  in the backend-mediated macOS path, not in every shell path.
- Canonical operator surfaces already exist:
  `substrate host doctor`, `substrate world doctor`, and
  `substrate world gateway sync|status|restart`.
  The remaining gap is to make those the supported contract first and classify
  direct guest administration and host-side overrides accordingly.
- `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` still do not
  sharply separate guest-local Linux-like behavior from the unresolved same-user
  host ownership limitation.

Without a target-mode contract, later fixes will remain local patches instead of a coherent hardening program.

## In-scope

- Define the supported deployment posture for same-user Lima.
- Define the exact Linux parity claims macOS is allowed to make after hardening.
- Define unsupported claims, especially around host-side ownership and multi-user separation.
- Define the required operator story for normal mode, degraded mode, and unsupported mode.
- Produce a concrete gap ledger that later milestones can convert into code and docs work.

## Out-of-scope

- Implementing transport or policy changes.
- Choosing the exact Lima version floor; that is milestone 0.2.
- Solving true multi-user macOS isolation.
- Rewriting all operator docs in this milestone.

## Architectural approach

This milestone should lock the contract around these rules:

1. Supported mode
   - One macOS host user owns the Substrate process, the Lima VM lifecycle, and the host-side forwarding artifacts.
   - The world-service still runs inside Linux and keeps guest-local socket ACL semantics (`root:substrate 0660`) inside the guest.
   - Substrate-owned commands, not direct `limactl`, are the supported control plane.
   - The supported control plane already includes `substrate host doctor`,
     `substrate world doctor`, and
     `substrate world gateway sync|status|restart`; later phases harden and cut
     over around those surfaces rather than inventing new baseline commands.
2. Linux parity claims we keep
   - In-world execution remains the default.
   - Guest-local `world-service` behavior, socket-activation shape, and policy evaluation intent must converge toward Linux semantics.
   - PTY and non-PTY routing should become transport-agnostic from the shell's point of view.
   - Shared-world/orchestration support remains part of the supported runtime
     story; hardening must preserve the existing `shared_world` request/binding
     path instead of treating it as an unsupported corner case.
   - Gateway lifecycle is a first-class operator and support surface, not an
     optional add-on.
3. Linux claims we explicitly do not make
   - Host-side authorization is not equivalent to Linux multi-user socket ownership while the same host user can drive `limactl`.
   - Same-user Lima is not a privilege boundary against the owning host user.
   - Direct guest administration is not part of normal operation.
   - Host-side `SUBSTRATE_WORLD_SOCKET` override use is not the default supported
     Lima path; it is an advanced/test/breakglass escape hatch that bypasses the
     normal forwarded socket discovery contract.

## Dependencies / sequencing

- Uses evidence from:
  - `crates/world-mac-lima/src/lib.rs`
  - `docs/WORLD.md`
  - `docs/cross-platform/mac_world_setup.md`
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/lima/substrate.yaml`
- Must complete before milestone 0.2, because version-floor and breakglass rules depend on the supported-mode definition.
- Must be accepted before any implementation milestone changes default mounts, forwarding, or policy wiring.

## Concrete repo surfaces and file pointers

- Backend-local policy synthesis and weak policy application:
  - `crates/world-mac-lima/src/lib.rs`
  - `crates/world-mac-lima/src/lib.rs:416`
  - `crates/world-mac-lima/src/lib.rs:604`
- Shell-side already-landed policy and gateway request builders:
  - `crates/shell/src/execution/policy_snapshot.rs`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
- Existing operator/gateway contract surfaces:
  - `crates/shell/src/execution/platform/macos.rs`
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
- Guest lifecycle and mount posture:
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/lima/substrate.yaml`
- Current operator claims and normal-mode drift:
  - `docs/WORLD.md`
  - `docs/cross-platform/mac_world_setup.md`

## Deliverables

- A written target-mode contract section in this SOW that later phases can quote directly.
- A support matrix with three states:
  - supported same-user hardened mode
  - degraded but supported diagnostic mode using routed Substrate surfaces or
    compatibility wrappers around them, not raw direct guest procedures
  - breakglass / unsupported direct guest mode
- A gap list that future implementation must close, at minimum:
  - remove backend-local policy synthesis from `MacLimaBackend`
  - strengthen `apply_policy(...)` semantics in the backend-mediated path
  - stop documenting direct guest administration as normal
  - classify host-side `SUBSTRATE_WORLD_SOCKET` override usage as advanced/test/breakglass
  - narrow host exposure and forwarding defaults to the supported mode

## Acceptance criteria

- The milestone defines the supported mode as same-user hardened Lima, not Linux-equivalent host authorization.
- The milestone states clearly that Linux ownership-boundary parity is impossible under the current same-user Lima model.
- The milestone names which Linux semantics remain implementation goals:
  - world-service-first execution
  - guest-local socket activation and ACL discipline
  - Linux-like backend policy semantics for the backend-mediated execution path
  - transport-agnostic PTY and non-PTY behavior
  - supported shared-world/orchestration behavior
  - gateway lifecycle/status as a normal support surface
- The milestone names which semantics are explicitly non-goals:
  - host-side multi-user socket authorization parity
  - security isolation from the owning host user
  - routine reliance on direct `limactl shell` administration
  - treating `SUBSTRATE_WORLD_SOCKET` override as the normal macOS Lima path
- The milestone includes enough file pointers that an implementation owner can trace every target gap back to code or docs.

## Validation / evidence plan

- Read `crates/world-mac-lima/src/lib.rs` and confirm the current synthetic
  policy snapshot, `shared_world` passthrough, and weak `apply_policy(...)`
  semantics are captured accurately in this SOW.
- Read `crates/shell/src/execution/policy_snapshot.rs`,
  `crates/shell/src/execution/routing/dispatch/world_ops.rs`, and
  `crates/shell/src/builtins/world_gateway.rs` and confirm the SOW correctly
  narrows the policy-drift claim to the backend-mediated macOS path.
- Read `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` and confirm
  the SOW correctly identifies where operator language currently
  over-normalizes direct guest management and host-side overrides.
- Use milestone review to force a binary decision on this statement:
  - "macOS same-user Lima can match Linux behavior in the guest, but not Linux host authorization semantics."
- Treat any attempt to keep ambiguous parity language as a phase-0 review defect.

## Risks / open questions

- Narrowing the support claim may be perceived as a regression even though it is only making the existing boundary honest.
- Later implementation may reveal edge cases where PTY routing or replay behavior cannot fully converge without larger backend refactors.
- The project may need a follow-on ADR if support posture changes affect release notes, installers, or product positioning.
