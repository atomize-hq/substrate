---
slice_id: S1
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-11` changes request parsing, item mapping, or sync response object rules after this slice starts"
    - "`C-12` invariant deltas require reworking how `/v1/responses` routes through the normalized core"
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-10
  - THR-12
contracts_produced:
  - C-11
contracts_consumed:
  - C-12
open_remediations: []
---
### S1 - Deliver Responses Request And Sync Response Object Mapping

- **User/system value**: OpenAI clients can use the non-streaming `/v1/responses` surface with the contracted item-based request subset, function-tool loop inputs, and Response object output shape.
- **Scope (in/out)**:
  - In: implement request parsing and sync Response object mapping for the ADR 0008 subset, including `input` shorthand expansion, item arrays, function tools only, `tool_choice` and `parallel_tool_calls`, model echo, error-envelope behavior, and tool-call output item mapping.
  - Out: streaming event emission, full pack conformance ownership, and provider-specific public stream parsing.
- **Acceptance criteria**:
  - the public adapter rejects built-in tools and known-but-unsupported fields deterministically per `C-11`
  - sync Response objects represent text-only, tool-call-only, and mixed outputs with stable item ordering and finish semantics
  - the handler preserves request `model` echo, `X-Provider` override behavior, and the contracted error envelope
  - the route remains a thin adapter over the normalized core per `C-12`
  - request parsing accepts only the frozen `input` shapes (`string`, `message[]`, `function_call_output[]`) and preserves `call_id` as the normalized tool identifier
  - sync `output` items remain limited to `message` and `function_call`, with tool arguments preserved as JSON strings
- **Dependencies**: `S00`, `gateway/src/server/mod.rs`, new Responses adapter module(s), `gateway/src/core.rs`, `gateway/src/models/*`, `THR-10`, `THR-12`
- **Verification**:
  - positive sync fixtures exist for text-only, tool-call-only, mixed content, and usage present/absent cases
  - negative tests prove built-in tools and known-but-unsupported fields reject with the contracted error envelope
  - pass condition: a reviewer can trace one non-stream request from Responses input through `GatewayRequest` into a Response object without raw provider-shape assumptions
- **Rollout/safety**: keep the surface narrow; every supported field or item type must follow `C-11`, and every known-but-unsupported field must reject deterministically.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`Likely mismatch hotspots`)

#### S1.T1 - Implement Request Parsing And Tool-Loop Inputs

- **Outcome**: supported Responses requests enter the normalized core with explicit tool and follow-up semantics.
- **Inputs/outputs**: inputs are `C-11`, current core request/tool types, and existing route wiring patterns; outputs are parser updates and handler validation behavior for the supported subset.
- **Thread/contract refs**: `THR-10`, `THR-12`, `C-11`, `C-12`
- **Implementation notes**: map `message` and `function_call_output` items into the normalized model without inventing endpoint-specific core fields; validate `call_id` deterministically; reject built-in tools and known-but-unsupported fields at the boundary.
- **Acceptance criteria**: item normalization and tool-loop continuation rules have one explicit mapping and one explicit reject/ignore disposition.
- **Test notes**: add request-path fixtures for item parsing, `call_id` threading, built-in tool rejection, and unsupported field rejection.
- **Risk/rollback notes**: parser drift here will invalidate downstream conformance and streaming event planning.

Checklist:
- Implement: extend request parsing and validation for the contracted subset
- Test: cover supported and rejected request shapes
- Validate: confirm the route still flows through `GatewayRequest`
- Cleanup: remove TODO or skip-based behavior for supported item types

#### S1.T2 - Implement Sync Response Object Mapping

- **Outcome**: normalized sync outputs map into the contracted Responses object (`object: response`, `output` items).
- **Inputs/outputs**: inputs are normalized gateway responses, `C-11`, and any existing sync mapping helpers; outputs are sync response-transform updates plus supporting helper code.
- **Thread/contract refs**: `THR-10`, `THR-12`, `C-11`, `C-12`
- **Implementation notes**: handle tool-call output items, finish semantics, and usage without leaking reasoning or provider-specific payload details.
- **Acceptance criteria**: text-only, tool-call-only, and mixed outputs map correctly; tool arguments remain JSON strings; model echo uses the request model.
- **Test notes**: add golden tests for output item ordering, tool-call mapping, usage handling, and model echo.
- **Risk/rollback notes**: if sync mapping invents endpoint-only semantics, the streaming slice will fork instead of reusing shared invariants.

Checklist:
- Implement: update sync Response object transformation for the contracted subset
- Test: add sync golden coverage for text, tool calls, mixed output, and usage behavior
- Validate: confirm chain-of-thought stays suppressed
- Cleanup: keep sync mapping aligned with the eventual event-stream mapping
