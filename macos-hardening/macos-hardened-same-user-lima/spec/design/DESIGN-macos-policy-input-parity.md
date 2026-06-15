# Design: macOS Policy Input Parity

Status: draft design input  
Last updated: 2026-06-11

## Why this doc exists

The phase docs already identify a key runtime-parity gap: shell-side routed
paths resolve effective policy/world inputs, while backend-mediated Lima
execution still synthesizes or drops those semantics inside
`MacLimaBackend`.

This design exists to freeze the architectural direction before a `SPEC-*`
lands backend changes:

1. shell-resolved policy truth remains authoritative,
2. backend-mediated Lima execution must consume that truth rather than invent a
   separate source.

## Relationship to the phase docs

This design composes with:

1. [`../../phase-1-runtime-parity-foundation/README.md`](../../phase-1-runtime-parity-foundation/README.md)
2. [`../../phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md`](../../phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md)
3. [`DESIGN-macos-lima-transport-contract.md`](./DESIGN-macos-lima-transport-contract.md)

## Problem statement

How should the repo converge backend-mediated Lima execution with the
already-landed shell routing behavior so that:

1. effective policy, network, and filesystem mode truth is computed once,
2. the backend does not synthesize a permissive local fallback,
3. future parity claims are grounded in the same authority model across paths?

## Observed repo-floor direction

The current phase docs already establish the intended floor:

1. shell-side routed request builders already resolve and forward:
   - `policy_snapshot`
   - `world_network`
   - `world_fs_mode`
2. backend-mediated Lima execution still diverges because:
   - `MacLimaBackend::convert_exec_request` synthesizes a permissive policy
     snapshot
   - backend policy application is not yet equivalent to routed shell truth

## Frozen direction

This design freezes the following:

1. **Single authority model**
   - effective policy/world inputs should be resolved once and propagated
     forward
2. **No backend-local permissive synthesis**
   - `MacLimaBackend` should not remain a second policy source of truth for
     backend-mediated Lima execution
3. **Prefer contract widening over hidden divergence**
   - if shared backend seams such as `WorldSpec` or adjacent contracts need to
     widen to carry the right inputs, that is preferable to preserving
     backend-local synthesis
4. **Parity scope**
   - this work is about policy/world input parity for the Lima-backed path
   - it is not a claim that the macOS host-side ownership boundary equals Linux

## Non-goals

This design does not:

1. define every field-level transport change yet,
2. settle every shared backend ripple into Windows or other backends,
3. redesign the broker or policy schema,
4. reopen the same-user ownership limitation.

## Architectural rules

Future specs should preserve these rules:

1. model the policy gap as a backend-mediated propagation problem, not as a
   repo-wide absence of shell policy propagation,
2. keep shell-resolved truth authoritative,
3. make backend behavior explanation-ready so doctor/tests can show what
   effective policy/world inputs were applied,
4. avoid duplicate policy interpretation layers with different defaults.

## File and surface implications

This design should inform future work across:

1. `crates/world-mac-lima/src/lib.rs`
2. `crates/shell/src/execution/routing/dispatch/world_ops.rs`
3. `crates/shell/src/repl/async_repl.rs`
4. `crates/shell/src/builtins/world_gateway.rs`
5. shared world/backend contract surfaces that may need widening

## Questions future specs should answer

1. What exact contract seam should carry the missing policy/world inputs into
   backend-mediated Lima execution?
2. Which tests best prove that backend-mediated execution now consumes the same
   effective inputs as direct routed shell paths?
3. What structured evidence should doctor or traces surface to prove the parity
   claim?
