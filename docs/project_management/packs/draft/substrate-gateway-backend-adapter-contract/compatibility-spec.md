# substrate-gateway-backend-adapter-contract - compatibility spec

This document is a narrow consumer artifact. It records the compatibility and supersession posture for
SEAM-3 without redefining any upstream contract surface.

## Contract boundary

Owned here:

- the additive rollout and compatibility story for existing operator workflows
- the historical-evidence-only treatment of ADR-0024
- the evidence-only treatment of ADR-0040 as consumed through ADR-0041 and
  `docs/contracts/substrate-gateway-runtime-parity.md`
- the explicit "no second control plane" invariant for this seam

Not owned here:

- backend-id grammar, config keys, policy rules, or inventory semantics
- runtime ownership clarification
- platform parity guarantees
- operator command semantics, status schema, or validation gates

## Compatibility posture

- Existing operator workflows remain compatible with the published adapter contract.
- This seam validates the already accepted backend-selection, protocol, schema, and runtime-parity
  boundaries; it does not widen them.
- Compatibility proof is additive: it shows that the gateway adapter contract can be consumed without
  changing the operator-facing contract shape or introducing a new control surface.
- Hidden implementation differences remain allowed only where upstream contracts already permit them.

## ADR-0024 supersession posture

- ADR-0024 is treated as historical evidence only.
- The architectural intent of ADR-0024 is superseded by ADR-0041, which preserves stable backend
  identity and pushes backend-adapter behavior into the gateway-hosted contract.
- This document does not restate ADR-0024's upstream routing, config, or backend-selection contract.
- The only compatibility claim made here is that the current adapter contract preserves the user-facing
  workflow that ADR-0024 originally tried to enable.

## ADR-0040 evidence-only posture

- ADR-0040 remains the prerequisite boundary evidence for runtime ownership.
- This seam consumes ADR-0040 through ADR-0041 and
  `docs/contracts/substrate-gateway-runtime-parity.md`, not by reopening ADR-0040 as a direct touch
  surface.
- Direct ADR-0040 edits remain out of scope unless landing evidence exposes a concrete owner-line
  mismatch that the current boundary no longer explains.
- This document does not claim any new runtime ownership, and it does not create a second control plane.

## Invariants

- The adapter contract stays additive relative to existing operator workflows.
- Backend identity remains stable and externally observable only through the published contract surfaces.
- Compatibility proof must not introduce new config, status, policy, or platform rules.
- Any future drift in the runtime boundary must be handled by the boundary owner, not by this document.

## Acceptance check

This document is complete only if it can be read as a consumer of ADR-0024, ADR-0040, ADR-0041, and
`docs/contracts/substrate-gateway-runtime-parity.md` without becoming a shadow contract for any of them.
