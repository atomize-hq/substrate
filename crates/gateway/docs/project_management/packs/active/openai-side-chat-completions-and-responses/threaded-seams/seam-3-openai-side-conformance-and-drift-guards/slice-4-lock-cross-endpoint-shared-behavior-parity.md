---
slice_id: S4
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-12` invariants change (model echo, error envelope, reasoning suppression) without updating the parity suite"
    - endpoint handlers diverge (one endpoint enforces invariants while the other drifts) and parity tests do not catch it
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
  - THR-13
contracts_produced:
  - C-13
contracts_consumed:
  - C-12
open_remediations: []
---
### S4 - Lock Cross-Endpoint Shared Behavior Parity

- **User/system value**: shared behavior stays consistent across Chat Completions and Responses so clients and operators do not encounter “same gateway, different rules” drift.
- **Scope (in/out)**:
  - In: add parity tests that exercise both endpoints and assert `C-12` invariants:
    - model echo behavior is consistent
    - `X-Provider` forcing behavior is consistent (including failure modes)
    - error envelope and status-code mapping is consistent for negative cases
    - chain-of-thought / reasoning suppression holds in both sync and stream modes
  - Out: expanding endpoint feature surface beyond the contracted subset.
- **Acceptance criteria**:
  - at least one parity test covers each invariant area listed above
  - tests explicitly detect divergence: same conceptual input yields inconsistent behavior across endpoints (within the contracted subset)
  - tests run offline and deterministically using the shared harness from `S1`
- **Dependencies**:
  - `S00`, `S1`
  - `docs/foundation/openai-side-adapter-invariants-c12-contract.md` (`C-12`)
  - both endpoints must be available and stable enough for revalidation to pass (especially `/v1/responses`)
- **Verification**:
  - suite includes at least one sync and one stream parity case (where streaming exists)
  - failures point to specific invariant clauses and identify which endpoint drifted
- **Rollout/safety**: keep parity assertions focused on the invariants, not on endpoint-specific response shapes that are already covered by endpoint conformance slices.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`Likely mismatch hotspots`)

#### S4.T1 - Add Model Echo And Provider Forcing Parity Tests

- **Outcome**: both endpoints preserve model echo and `X-Provider` forcing semantics consistently.
- **Inputs/outputs**: inputs are `C-12` invariants and representative requests; outputs are parity tests that run each endpoint with the same injected provider setup.
- **Thread/contract refs**: `THR-10`, `THR-13`, `C-12`, `C-13`
- **Implementation notes**: assert model echo at the public boundary (the model in responses/events), and assert provider forcing through deterministic routing/provider stubs.
- **Acceptance criteria**: the parity suite identifies divergence as an invariant breach (not as an endpoint behavior difference).
- **Test notes**: include at least one negative case where forcing fails and the error envelope is asserted.
- **Risk/rollback notes**: provider forcing drift often appears as a routing refactor; parity tests make it explicit.

Checklist:
- Implement: add parity tests for model echo and `X-Provider` forcing
- Test: ensure failures identify the endpoint that drifted
- Validate: keep assertions contract-focused (no incidental snapshots)
- Cleanup: reuse harness helpers instead of duplicating setup logic

#### S4.T2 - Add Error Envelope And Reasoning Suppression Parity Tests

- **Outcome**: both endpoints share the same error envelope posture and never leak reasoning/chain-of-thought into public text fields.
- **Inputs/outputs**: inputs are `C-12` invariants and negative-case request fixtures; outputs are parity tests for both endpoints in sync and stream modes.
- **Thread/contract refs**: `THR-10`, `THR-13`, `C-12`, `C-13`
- **Implementation notes**: ensure the suite uses deliberately-invalid inputs to trigger errors and asserts the redacted/classified envelope; for reasoning suppression, use stubbed outputs that contain reasoning-like markers and assert they never surface.
- **Acceptance criteria**: the suite detects any leak as a hard failure and points to the invariant clause.
- **Test notes**: include at least one streaming case (where available) to ensure suppression holds in streamed deltas/events too.
- **Risk/rollback notes**: suppression regressions are high-risk; parity tests are the required guard.

Checklist:
- Implement: add parity tests for error envelope and reasoning suppression
- Test: run offline and verify deterministic output
- Validate: ensure tests detect leaks in both endpoints
- Cleanup: keep leak-detection heuristics explicit and contract-justified
