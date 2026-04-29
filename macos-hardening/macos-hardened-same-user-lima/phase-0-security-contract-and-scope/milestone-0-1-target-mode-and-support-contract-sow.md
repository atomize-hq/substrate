# Milestone 0.1: Target Mode and Support Contract SOW

Status: Draft  
Last updated: 2026-04-28

## Purpose / outcome

Define the authoritative target mode for hardened same-user Lima and freeze the support contract that all later macOS hardening work must satisfy. The milestone outcome is a decision-ready contract that says exactly what the supported mode is, what Linux properties it is expected to emulate, and which Linux guarantees are explicitly unavailable under the current same-user VM ownership model.

## Why this milestone exists

Today the macOS backend mixes functional parity goals with security claims it cannot fully uphold.

- `crates/world-mac-lima/src/lib.rs` presents Linux-like execution flow but still injects a permissive synthetic policy snapshot and does not apply backend policy after session setup.
- `docs/WORLD.md` describes socket ACLs and platform-hidden divergence, but does not sharply distinguish guest-local ACL parity from host-side ownership-boundary parity.
- `docs/cross-platform/mac_world_setup.md` still teaches direct `limactl shell` and direct guest `systemctl` as standard setup and validation steps.

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
   - The world-agent still runs inside Linux and keeps guest-local socket ACL semantics (`root:substrate 0660`) inside the guest.
   - Substrate-owned commands, not direct `limactl`, are the supported control plane.
2. Linux parity claims we keep
   - In-world execution remains the default.
   - Guest-local `world-agent` behavior, socket-activation shape, and policy evaluation intent must converge toward Linux semantics.
   - PTY and non-PTY routing should become transport-agnostic from the shell's point of view.
3. Linux claims we explicitly do not make
   - Host-side authorization is not equivalent to Linux multi-user socket ownership while the same host user can drive `limactl`.
   - Same-user Lima is not a privilege boundary against the owning host user.
   - Direct guest administration is not part of normal operation.

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

- Synthetic policy snapshot and backend no-op:
  - `crates/world-mac-lima/src/lib.rs`
  - `convert_exec_request`
  - `apply_policy`
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
  - remove permissive synthetic backend policy behavior
  - stop documenting direct guest administration as normal
  - narrow host exposure and forwarding defaults to the supported mode

## Acceptance criteria

- The milestone defines the supported mode as same-user hardened Lima, not Linux-equivalent host authorization.
- The milestone states clearly that Linux ownership-boundary parity is impossible under the current same-user Lima model.
- The milestone names which Linux semantics remain implementation goals:
  - world-agent-first execution
  - guest-local socket activation and ACL discipline
  - Linux-like backend policy semantics
  - transport-agnostic PTY and non-PTY behavior
- The milestone names which semantics are explicitly non-goals:
  - host-side multi-user socket authorization parity
  - security isolation from the owning host user
  - routine reliance on direct `limactl shell` administration
- The milestone includes enough file pointers that an implementation owner can trace every target gap back to code or docs.

## Validation / evidence plan

- Read `crates/world-mac-lima/src/lib.rs` and confirm the current synthetic policy snapshot and `apply_policy` no-op are captured accurately in this SOW.
- Read `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` and confirm the SOW correctly identifies where operator language currently over-normalizes direct guest management.
- Use milestone review to force a binary decision on this statement:
  - "macOS same-user Lima can match Linux behavior in the guest, but not Linux host authorization semantics."
- Treat any attempt to keep ambiguous parity language as a phase-0 review defect.

## Risks / open questions

- Narrowing the support claim may be perceived as a regression even though it is only making the existing boundary honest.
- Later implementation may reveal edge cases where PTY routing or replay behavior cannot fully converge without larger backend refactors.
- The project may need a follow-on ADR if support posture changes affect release notes, installers, or product positioning.
