---
slice_id: S3
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-11` publishes with materially different item or event rules than the assumptions used by planned fixtures"
    - "`C-12` changes shared tool or stream invariants that affect Responses output shaping"
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
  - THR-13
contracts_produced:
  - C-13
contracts_consumed:
  - C-11
  - C-12
open_remediations: []
---
### S3 - Lock Responses Conformance And Negative Cases

- **User/system value**: the `POST /v1/responses` surface becomes regression-guarded so future changes cannot silently break modern OpenAI Responses compatibility (items, tool loops, and event streaming).
- **Scope (in/out)**:
  - In: add deterministic conformance + negative-case tests for:
    - sync Response object mapping (`output` items)
    - streaming event-subset coverage and `data.type` conventions
    - tool-loop continuation behavior (`function_call_output` items + `call_id`)
    - reject built-in tools and non-function tool call types
    - reject known-but-unsupported fields, ignore unknown fields (per `C-11`)
    - error envelope posture for negative cases
  - Out: expanding supported item types or schema outputs beyond `C-11`.
- **Acceptance criteria**:
  - tests fail when:
    - required streaming event types are missing or misordered
    - required payload fields or `data.type` conventions drift
    - tool-loop threading via `call_id` drifts
    - reject/ignore posture changes without contract updates
  - suite includes at least one positive and one negative case for each major contract rule group
- **Dependencies**:
  - `S00`, `S1`
  - `THR-12` is now published and `C-11` exists as a concrete contract artifact, so this slice can execute against landed Responses truth
  - `docs/foundation/openai-side-adapter-invariants-c12-contract.md` (`C-12`)
- **Verification**:
  - conformance tests exercise the public route via the harness (not only provider-level or transform-level tests)
  - tests assert the contracted event subset and error envelope deterministically and offline
- **Rollout/safety**: keep assertions contract-focused; avoid snapshotting incidental internal state or provider-shaped payload fragments.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`Likely mismatch hotspots`)

#### S3.T1 - Add Sync Responses Conformance Cases

- **Outcome**: sync Response object mapping is guarded for output items, tool calls, finish semantics, and usage behavior.
- **Inputs/outputs**: inputs are `C-11` clauses and representative request/response fixtures; outputs are `gateway/tests/*` cases plus fixtures under `gateway/tests/fixtures/openai_responses/`.
- **Thread/contract refs**: `THR-12`, `THR-13`, `C-11`, `C-13`
- **Implementation notes**: assert required output item types and ordering guarantees (where promised) without mirroring the full OpenAI spec.
- **Acceptance criteria**: tests cover: text-only output, tool-call output, mixed output, and error-envelope output for rejected inputs.
- **Test notes**: include negative cases for built-in tools and known-but-unsupported fields.
- **Risk/rollback notes**: Responses mapping drift will otherwise be discovered only by downstream SDK failures.

Checklist:
- Implement: add sync Responses conformance tests + fixtures
- Test: ensure failures point to specific `C-11` clause deltas
- Validate: confirm suite stays offline and deterministic
- Cleanup: unify assertion helpers with Chat Completions where possible

#### S3.T2 - Add Streaming Responses Conformance Cases

- **Outcome**: streaming event subset and conventions are guarded.
- **Inputs/outputs**: inputs are `C-11` event rules and streaming fixtures; outputs are streaming tests that assert `data.type`, required fields, and completion semantics.
- **Thread/contract refs**: `THR-12`, `THR-13`, `C-11`, `C-13`
- **Implementation notes**: parse SSE `data:` payloads into structured JSON and assert event sequence/fields rather than byte-for-byte snapshots of the entire stream.
- **Acceptance criteria**: tests cover the contracted event subset across text-only, tool-call-only, and at least one mixed streaming sequence.
- **Test notes**: include a failure-mode test where missing required events or wrong `data.type` breaks deterministically.
- **Risk/rollback notes**: if this suite is too loose, drift will be silent; if too strict, harmless additions will create noise. Follow `C-11` variance rules.

Checklist:
- Implement: add streaming Responses conformance tests + fixtures
- Test: confirm event ordering and termination semantics are asserted
- Validate: ensure tests do not depend on timing or network
- Cleanup: share SSE parsing helpers with Chat Completions streaming tests
