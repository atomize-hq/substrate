---
seam_id: SEAM-3
seam_slug: openai-side-conformance-and-drift-guards
type: conformance
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-2-closeout.md
  required_threads:
    - THR-10
    - THR-11
    - THR-12
  stale_triggers:
    - any change to `SEAM-1` or `SEAM-2` public response shapes, error envelope rules, or streaming event/chunk semantics
    - any change to internal tool representation or usage accounting that affects adapter output mapping
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S99
  status: passed
open_remediations: []
---

# SEAM-3 - OpenAI-Side Conformance and Drift Guards

- **Goal / value**: prevent silent drift in OpenAI-side ingress by adding deterministic tests and negative-case coverage that enforce the contracted subset for both endpoints and shared behavior across them.
- **Scope**
  - In:
    - conformance tests for `/v1/chat/completions`:
      - non-streaming response mapping
      - streaming chunks + `[DONE]` termination
      - function tool call mapping and tool-loop continuation
      - rejection/ignore posture for unsupported fields
    - conformance tests for `/v1/responses`:
      - non-streaming response object mapping
      - streaming event-subset coverage and `data.type` conventions
      - function tool call mapping and tool-loop continuation
    - shared-behavior tests:
      - model echo
      - `X-Provider` forcing behavior
      - error envelope and status-code mapping
      - chain-of-thought suppression guarantees
  - Out:
    - broad fuzzing or large-scale load/perf work (can be added later if needed)
    - live upstream network dependency in CI (prefer fixtures and local provider fakes)
- **Primary interfaces**
  - Inputs:
    - test fixtures for representative requests and provider-normalized responses/streams
  - Outputs:
    - deterministic unit/integration tests that fail on contract drift
    - a minimal set of documentation anchors (if needed) that point to the contracted subset and tests
- **Key invariants / rules**:
  - conformance tests must be stable and deterministic (no real network dependencies required)
  - negative-case tests must assert the contracted error envelope and status mapping
  - shared behavior must remain consistent across both endpoints
- **Dependencies**
  - Direct blockers:
    - none; `THR-10`, `THR-11`, and `THR-12` are now published/revalidated enough that conformance is locking in landed truth instead of guessing
  - Transitive blockers:
    - internal test harness ability to inject provider-normalized responses/streams for adapters
  - Direct consumers:
    - future maintenance and future feature expansion work
  - Derived consumers:
    - downstream integration work that depends on stable OpenAI-side ingress behavior
- **Touch surface**:
  - `gateway/tests/*` (new tests + fixtures)
  - any adapter-level helper utilities needed to feed deterministic streams into SSE transforms
  - docs references (if required) that point to supported subset and test evidence
- **Verification**:
  - tests fail on: streaming termination regressions, tool-call mapping regressions, error envelope drift, model echo drift, and built-in tool leakage
  - tests provide at least one “happy path” and one “rejection path” per endpoint
- **Risks / unknowns**:
  - Risk: overly brittle golden tests may fail on benign changes; too-loose tests may miss drift.
  - De-risk plan: focus goldens on required fields and ordering, and keep optional fields tolerant where the contract allows it.
- **Rollout / safety**:
  - treat conformance as a publishable contract artifact for maintainers; keep tests small and fast
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: this seam is now landed basis outside the forward window after locking the published subset into offline conformance and parity coverage.
  - Which threads matter most: `THR-10`, `THR-11`, and `THR-12` are the revalidated inbound basis; `THR-13` is the durable publication target.
  - What the seam-local pre-exec review locked in: offline determinism, clause-to-test mapping for `C-13`, and strict drift assertions around tool loops, streaming termination, reject/ignore posture, and reasoning suppression.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-13`
  - Threads likely to advance: `THR-13` is now `published`
  - Review-surface areas likely to shift after landing: R3 touch surface will gain concrete test paths and fixture locations.
  - Downstream seams most likely to require revalidation: future OpenAI-side expansions once they exist and need compatibility lock-in.
