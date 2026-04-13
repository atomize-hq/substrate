---
slice_id: S1
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: future
status: landed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-10` changes request parsing, reject/ignore posture, or sync response mapping after this slice starts"
    - normalized message or tool shapes change such that `chat/completions` must branch away from the shared core
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
threads:
  - THR-10
  - THR-11
contracts_produced:
  - C-10
  - C-12
contracts_consumed: []
open_remediations: []
---
### S1 - Deliver Chat Completions Request And Sync Response Mapping

- **User/system value**: OpenAI clients can use the non-streaming `chat/completions` surface with the contracted request subset, tool-loop inputs, and output object shape instead of the current limited adapter.
- **Scope (in/out)**:
  - In: implement the request parser and sync response mapper for the ADR 0008 subset, including roles, parts, image URLs, function tools, `tool_choice`, model echo, error-envelope behavior, and tool-call response mapping.
  - Out: stream chunk emission, usage-chunk sequencing, `/v1/responses`, and seam-exit publication accounting.
- **Acceptance criteria**:
  - `openai_compat` no longer drops `tool`-role follow-ups or leaves function tools unmapped for the supported subset
  - sync `chat.completion` responses can represent text-only, tool-call-only, and mixed outputs with correct finish reasons and `arguments` JSON-string behavior
  - the handler preserves request `model` echo, `X-Provider` override behavior, and the contracted error envelope for rejected fields or routing/provider failures
  - no endpoint-specific execution path is introduced; the route still operates through the normalized core
- **Dependencies**: `S00`, `gateway/src/server/mod.rs`, `gateway/src/server/openai_compat.rs`, `gateway/src/core.rs`, `gateway/src/models/mod.rs`, `THR-10`, `THR-11`
- **Verification**:
  - positive sync fixtures exist for text-only, tool-call-only, mixed content, and usage present/absent cases
  - negative tests prove known-but-unsupported fields and non-function tools reject with the contracted error envelope
  - pass condition: a reviewer can trace one non-stream request from OpenAI input through the normalized core into a `chat.completion` object without raw provider-shape assumptions
- **Rollout/safety**: keep the existing limited route from silently widening in unsupported ways; every new supported field or behavior must follow `C-10`, and every unsupported known field must reject deterministically.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`Likely mismatch hotspots`, `Planned seam-exit gate focus`)

#### S1.T1 - Implement Request Parsing And Tool-Loop Inputs

- **Outcome**: supported OpenAI Chat Completions requests enter the normalized core with explicit tool and follow-up semantics.
- **Inputs/outputs**: inputs are `C-10`, current `OpenAIRequest` parsing, and normalized request/tool types; outputs are parser updates and handler validation behavior for the supported subset.
- **Thread/contract refs**: `THR-10`, `THR-11`, `C-10`, `C-12`
- **Implementation notes**: map `tool`-role messages and function tools into the normalized model without inventing endpoint-specific core fields; reject built-in tools and known-but-unsupported fields at the boundary.
- **Acceptance criteria**: supported roles, content parts, images, tools, and `tool_choice` have one explicit normalized mapping and one explicit reject/ignore disposition.
- **Test notes**: add request-path fixtures for text parts, image URLs, tool-result follow-up turns, non-function tool rejection, and unsupported field rejection.
- **Risk/rollback notes**: parser drift here will invalidate both downstream conformance and later `/v1/responses` reuse of `C-12`.

Checklist:
- Implement: extend request parsing and validation for the contracted subset
- Test: cover supported and rejected request shapes
- Validate: confirm the route still flows through `GatewayRequest`
- Cleanup: remove TODO or skip-based behavior for supported roles and tools

#### S1.T2 - Implement Sync Completion Mapping

- **Outcome**: normalized sync outputs map into the contracted `chat.completion` response object.
- **Inputs/outputs**: inputs are normalized gateway responses, `C-10`, and current response shaping; outputs are sync response-transform updates plus any supporting helper code.
- **Thread/contract refs**: `THR-10`, `THR-11`, `C-10`, `C-12`
- **Implementation notes**: handle tool-call objects, finish reasons, and usage without leaking reasoning or provider-specific payload details.
- **Acceptance criteria**: text-only, tool-call-only, and mixed outputs map correctly; `choices` stays single-element; `arguments` remain JSON strings; model echo uses the request model.
- **Test notes**: add golden tests for finish-reason mapping, tool-call object shape, usage handling, and model echo.
- **Risk/rollback notes**: if sync mapping invents endpoint-only semantics, the stream slice will likely fork instead of reusing the same invariants.

Checklist:
- Implement: update sync response transformation for the full contracted subset
- Test: add sync golden coverage for text, tool calls, mixed output, and usage behavior
- Validate: confirm chain-of-thought stays suppressed
- Cleanup: remove assumptions that only text blocks matter
