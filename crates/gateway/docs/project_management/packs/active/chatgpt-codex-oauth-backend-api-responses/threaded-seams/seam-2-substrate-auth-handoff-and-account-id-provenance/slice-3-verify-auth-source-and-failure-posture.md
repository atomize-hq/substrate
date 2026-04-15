---
slice_id: S3
seam_id: SEAM-2
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the route contract or auth-handoff contract changes the accepted auth-source behavior
    - the gateway's failure envelope or provider injection order changes
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-15
contracts_produced: []
contracts_consumed:
  - C-14
  - C-15
open_remediations: []
---
### S3 - Verify Auth Source And Failure Posture

- **User/system value**: deterministic evidence proves the integrated auth owner line instead of leaving it as an implementation assumption.
- **Scope (in/out)**:
  - In: add verification surfaces for integrated versus standalone source selection, explicit precedence, and pre-upstream failure behavior.
  - Out: route contract shaping, provider header wiring, and seam-exit publication accounting.
- **Acceptance criteria**:
  - deterministic checks prove integrated and standalone modes remain distinct
  - unresolved identity fails before upstream
  - docs and tests name the same owner line and fallback posture
- **Dependencies**: `S00`, `S1`, `S2`, `crates/gateway/tests/`, `crates/gateway/docs/OAUTH_SETUP.md`, `crates/gateway/docs/OAUTH_TESTING.md`, `THR-15`
- **Verification**:
  - positive tests prove source precedence and bounded fallback
  - negative tests prove unresolved identity fails deterministically
- **Rollout/safety**: keep the conformance surface route-specific and explicit.
- **Review surface refs**: `../../review_surfaces.md` (`R3`) and `review.md`
