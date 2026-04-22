---
slice_id: S1
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the named `api:openai` proof target changes after review refresh
    - runtime parity tests change supported-backend or unsupported-backend semantics before landing
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
  - THR-03
contracts_produced: []
contracts_consumed:
  - C-03
  - C-04
  - C-05
open_remediations: []
---
### S1 - Lock the parity regression floor and backend matrix

- **User/system value**:
  - Makes parity proof executable from one explicit matrix instead of ad hoc assertions scattered across runtime and shell tests.
- **Scope (in/out)**:
  - In: `cli:codex` regression floor, `api:openai` proof coverage, unsupported-backend/no-fallback coverage, and matrix-level evidence targets
  - Out: platform-specific smoke/manual publication and downstream rollout prose
- **Acceptance criteria**:
  - `cli:codex` remains the regression floor across runtime and shell coverage
  - `api:openai` is exercised end to end as the named additional-backend proof target
  - unsupported integrated backends fail explicitly with no silent fallback
- **Dependencies**:
  - `THR-02`
  - `C-03`
  - `C-04`
  - `C-05`
- **Verification**:
  - targeted updates or confirmations in `crates/world-agent/tests/gateway_runtime_parity.rs` and `crates/shell/tests/world_gateway.rs`
- **Rollout/safety**:
  - do not let parity coverage imply support for any backend beyond `cli:codex` and the named `api:openai` proof target
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`

#### S1.T1 - Make the parity matrix explicit in automated coverage

- **Outcome**:
  - runtime and shell tests together prove one readable backend matrix rather than isolated one-off assertions.
- **Inputs/outputs**:
  - Inputs: `THR-02`, canonical runtime parity contract, current runtime and shell tests
  - Outputs: refreshed test evidence targets and any bounded parity-test updates
- **Thread/contract refs**:
  - `THR-02`
  - `THR-03`
  - `C-05`
- **Implementation notes**:
  - keep `cli:codex`, `api:openai`, and unsupported-backend cases visible in the same proof story
  - avoid inventing new support tiers or rollout taxonomies inside test names
- **Acceptance criteria**:
  - closeout can cite one stable matrix for regression floor, proof target, and unsupported-backend behavior
  - parity coverage does not depend on hidden knowledge of current runtime internals
- **Test notes**:
  - verify coverage still names `api:openai` and explicit unsupported-backend failure cases
- **Risk/rollback notes**:
  - a vague matrix will force platform evidence and rollout notes to infer behavior again

Checklist:
- Implement:
  - make the parity matrix explicit in automated evidence
- Test:
  - confirm `cli:codex`, `api:openai`, and unsupported-backend paths are all covered
- Validate:
  - confirm no silent fallback remains in the proof story
