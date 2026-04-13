---
slice_id: S2
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-10` changes chunk ordering, tool-call delta rules, usage-chunk rules, or reject/ignore posture after this slice starts"
    - "`C-12` changes shared behavior or normalized stream model semantics in a way that requires fixture updates"
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
  - THR-11
  - THR-13
contracts_produced:
  - C-13
contracts_consumed:
  - C-10
  - C-12
open_remediations: []
---
### S2 - Lock Chat Completions Conformance And Negative Cases

- **User/system value**: the `POST /v1/chat/completions` surface becomes regression-guarded so future changes cannot silently break SDK compatibility, streaming termination, or tool-loop behavior.
- **Scope (in/out)**:
  - In: add deterministic conformance + negative-case tests for:
    - sync response mapping (text-only, tool-call-only, mixed)
    - streaming chunk emission (text deltas, tool-call deltas, optional usage chunk, `[DONE]`)
    - tool-loop continuation behavior (tool messages via `tool` role + `tool_call_id`)
    - reject known-but-unsupported fields, reject non-function tools, ignore unknown fields
    - error envelope posture for negative cases
  - Out: expanding supported field set beyond `C-10` or widening tool support.
- **Acceptance criteria**:
  - tests fail when:
    - `[DONE]` is missing or misordered
    - chunk payloads drift away from `C-10` required fields and ordering guarantees
    - tool-call assembly or `arguments` encoding drifts
    - reject/ignore posture changes without contract updates
  - suite includes at least one positive and one negative case for each major contract rule group
- **Dependencies**:
  - `S00`, `S1`
  - `docs/foundation/openai-side-chat-completions-c10-contract.md` (`C-10`)
  - `docs/foundation/openai-side-adapter-invariants-c12-contract.md` (`C-12`)
  - existing unit-test anchors in `gateway/src/server/openai_compat.rs` (use as evidence, but do not treat as the full drift guard)
- **Verification**:
  - conformance tests exercise the public route via the harness (not only transform-level unit tests)
  - at least one test asserts both: public output shape and thin-adapter invariants (no provider-specific public stream logic)
- **Rollout/safety**: keep assertions contract-focused; where optional fields exist, assert presence only when required by `C-10`.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`Likely mismatch hotspots`)

#### S2.T1 - Add Sync Chat Completions Conformance Cases

- **Outcome**: sync mapping is guarded for text, tool calls, finish reasons, and usage behavior.
- **Inputs/outputs**: inputs are `C-10` contract clauses and representative request/response fixtures; outputs are new `gateway/tests/*` cases plus fixtures under `gateway/tests/fixtures/openai_chat_completions/`.
- **Thread/contract refs**: `THR-11`, `THR-13`, `C-10`, `C-13`
- **Implementation notes**: assert required fields and stable ordering (where promised) without snapshotting internal-only fields.
- **Acceptance criteria**: tests cover: text-only, tool-call-only, mixed content, and error-envelope output for rejected inputs.
- **Test notes**: include negative cases for built-in tools and known-but-unsupported fields (`n`, `logprobs`, `audio`, `modalities`).
- **Risk/rollback notes**: sync output drift often hides inside “just refactors”; this slice makes it explicit.

Checklist:
- Implement: add sync conformance tests + fixtures
- Test: run suite and ensure failures are crisp and actionable
- Validate: ensure tests assert the public contract, not internal debug details
- Cleanup: factor common assertions into the shared harness utilities

#### S2.T2 - Add Streaming Chat Completions Conformance Cases

- **Outcome**: streaming chunk emission and termination semantics are guarded.
- **Inputs/outputs**: inputs are `C-10` streaming rules and stream fixtures; outputs are streaming tests that assert chunk payloads and `[DONE]` behavior deterministically.
- **Thread/contract refs**: `THR-11`, `THR-13`, `C-10`, `C-13`
- **Implementation notes**: test line-by-line SSE payloads; prefer contract-focused parsing of `data:` payloads over brittle byte-for-byte snapshots of entire streams.
- **Acceptance criteria**: tests cover:
  - text delta sequence
  - tool-call delta sequence
  - optional usage chunk behavior (when requested by the contract)
  - guaranteed termination via `[DONE]`
- **Test notes**: include at least one failure-mode test where a missing termination or wrong ordering triggers an assertion.
- **Risk/rollback notes**: streaming is the highest compat risk; these tests are the drift guard.

Checklist:
- Implement: add streaming conformance tests + fixtures
- Test: confirm the suite is deterministic (no timing-based sleeps, no network)
- Validate: ensure streaming assertions match `C-10` ordering guarantees
- Cleanup: centralize SSE parsing helpers in the harness
