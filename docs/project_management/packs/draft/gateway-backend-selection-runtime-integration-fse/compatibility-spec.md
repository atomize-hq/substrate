# gateway-backend-selection-runtime-integration-fse - compatibility spec

This document is a narrow consumer artifact. It records the rollout and compatibility posture for
this parity-validation seam without redefining any upstream contract surface.

## Contract boundary

Owned here:

- the additive rollout and compatibility story for existing operator workflows
- the evidence-backed publication framing for the first additional backend proof target, `api:openai`
- the explicit regression floor for `cli:codex`
- the explicit unsupported-backend no-fallback posture
- subordinate references to the canonical runtime-parity contract and upstream evidence

Not owned here:

- backend selection, policy rules, or inventory semantics
- runtime binding lookup, auth payload shape, or runtime artifact semantics
- platform parity guarantees, smoke procedure design, or manual validation ownership
- operator command semantics, status schema, or any support beyond `cli:codex` and `api:openai`
- seam closeout publication or exit-gate records

## Compatibility posture

- Existing operator workflows remain compatible with `docs/contracts/substrate-gateway-runtime-parity.md`.
- `cli:codex` remains the regression floor.
- `api:openai` is the first additional backend proof target reflected in rollout framing.
- Unsupported integrated backends are explicit negative cases and do not silently fall back to `cli:codex`.
- Hidden implementation differences remain allowed only where upstream contracts and evidence already permit them.

## Proof inputs

This document consumes, but does not redefine, the following upstream evidence:

- automated parity evidence from the runtime and shell test suites
- platform validation evidence from the Linux, macOS, and Windows proof surfaces
- the canonical runtime-parity contract in `docs/contracts/substrate-gateway-runtime-parity.md`

## Invariants

- One operator-facing lifecycle/status contract remains the public truth.
- No new support matrix, status field, or command family is introduced here.
- Compatibility publication remains additive and evidence-backed.
- Any future backend expansion requires a fresh evidence pass and a compatibility refresh.

## Acceptance check

This document is complete only if it can be read as a consumer of the canonical runtime-parity
contract and upstream evidence without becoming a shadow contract for either.
