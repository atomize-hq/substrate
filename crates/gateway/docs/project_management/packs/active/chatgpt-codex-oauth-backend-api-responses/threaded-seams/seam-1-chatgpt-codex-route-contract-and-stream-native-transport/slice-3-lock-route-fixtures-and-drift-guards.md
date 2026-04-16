---
slice_id: S3
seam_id: SEAM-1
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the route contract changes accepted/rejected controls, semantic event expectations, or sync-failure posture after this slice starts
    - live upstream behavior drifts beyond the fixture-backed route evidence frozen by this slice
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-14
contracts_produced:
  - C-14
contracts_consumed: []
open_remediations: []
---
### S3 - Lock Route Fixtures And Drift Guards

- **User/system value**: the owned route contract is backed by deterministic evidence, so downstream seams consume named route truth instead of reverse-engineering behavior from provider code or one-off probes.
- **Scope (in/out)**:
  - In: add provider-focused regression coverage, fixture-backed request/stream evidence, and route-specific verification anchors for endpoint parity, request shaping, semantic assembly, sync-drain failure, continuation rules, and reasoning visibility.
  - Out: downstream auth-handoff verification ownership and pack-wide conformance ownership that belongs to `SEAM-3`.
- **Acceptance criteria**:
  - deterministic tests exist for accepted versus rejected request controls, sync/stream endpoint parity, minimal headers, continuation ordering, semantic event assembly, and sync-drift failures
  - route-specific fixtures capture the semantic event families and malformed/truncated failure cases needed by this seam
  - the canonical route contract note and implementation tests cite each other clearly enough that `THR-14` can later publish without ADR-only evidence
- **Dependencies**: `S00`, `S1`, `S2`, `crates/gateway/tests/openai_responses_conformance.rs`, `crates/gateway/tests/openai_shared_parity.rs`, `THR-14`
- **Verification**:
  - deterministic regressions prove the route matrix, event-family authority, continuation rules, and failure posture without live upstream dependence in the core path
  - pass condition: the seam can later publish `THR-14` using named contract and fixture evidence rather than one-off probe notes
- **Rollout/safety**: keep the regression surface route-specific and explicit; do not rely on permissive generic Responses tests to imply Codex-route correctness.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`Planned seam-exit gate focus`)

#### S3.T1 - Add Deterministic Request And Header Regression Coverage

- **Outcome**: request shaping and minimal-header behavior are locked to the route contract.
- **Inputs/outputs**: inputs are `C-14`, serializer behavior from `S1`, and current gateway test harnesses; outputs are request-shape and header-focused regressions plus any supporting fixtures.
- **Thread/contract refs**: `THR-14`, `C-14`
- **Implementation notes**: prove endpoint parity, minimal headers, field allowlist/reject posture, typed-message and image translation, flat tool shapes, and deterministic rejection of unsupported controls.
- **Acceptance criteria**: tests fail if the route silently widens or degrades compatibility.
- **Test notes**: cover accepted image inputs, rejected `stream_options`, rejected nested tool shapes, rejected `tool_choice = "required"`, and rejected generic Responses fields.
- **Risk/rollback notes**: missing request coverage will let route drift hide behind generic provider behavior.

Checklist:
- Implement: add request and header regressions plus supporting fixtures
- Test: cover accepted and rejected controls plus endpoint parity
- Validate: confirm unsupported controls fail before the upstream call
- Cleanup: keep fixture intent explicit and route-specific

#### S3.T2 - Add Semantic Event, Continuation, And Drift-Failure Coverage

- **Outcome**: semantic assembly, continuation legality, and sync failure posture are backed by deterministic route evidence.
- **Inputs/outputs**: inputs are `C-14`, event assembly from `S2`, and current stream/test harnesses; outputs are semantic-stream and sync-failure regressions plus malformed-stream fixtures.
- **Thread/contract refs**: `THR-14`, `C-14`
- **Implementation notes**: prove event-family authority, text/tool/mixed assembly, synthesized continuation order, reasoning suppression, and `502 transport_drift` on malformed sync drains.
- **Acceptance criteria**: the route contract can be revalidated by offline tests and fixtures rather than by ADR prose alone.
- **Test notes**: add text-only, tool-only, mixed, reasoning-bearing, orphaned-continuation, truncated-stream, and missing-completion cases.
- **Risk/rollback notes**: if malformed sync drains are not fixture-backed, the route will regress into partial-success behavior.

Checklist:
- Implement: add semantic-event and sync-failure regressions plus malformed-stream fixtures
- Test: cover text, tool, mixed, reasoning-bearing, orphaned, truncated, and missing-completion cases
- Validate: confirm deterministic evidence matches the owned contract note
- Cleanup: keep live upstream probes out of the core regression path
