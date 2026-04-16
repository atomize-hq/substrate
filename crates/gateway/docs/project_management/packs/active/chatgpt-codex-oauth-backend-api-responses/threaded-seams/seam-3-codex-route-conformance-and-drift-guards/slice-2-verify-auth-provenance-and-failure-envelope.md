---
slice_id: S2
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
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
  - THR-16
contracts_produced: []
contracts_consumed:
  - C-15
  - C-16
open_remediations: []
---
### S2 - Verify Auth Provenance And Failure Envelope

- **User/system value**: the auth owner line stays contract-backed instead of regressing into token-shape inference or leaked provider errors.
- **Scope (in/out)**:
  - In: route-boundary proof for integrated-vs-standalone auth behavior, bounded fallback, and shared auth error-envelope posture.
  - Out: auth-resolution implementation work and seam-exit publication.
- **Acceptance criteria**:
  - integrated and standalone auth-source proofs remain distinct and deterministic
  - unresolved identity still fails before upstream with the shared auth envelope
  - provider auth failures stay redacted and classified as `auth`
- **Dependencies**: `S00`, `crates/gateway/tests/openai_shared_parity.rs`, `crates/gateway/src/providers/openai.rs`, `C-15`
- **Verification**:
  - positive tests prove explicit account-id precedence and the integrated handoff posture
  - negative tests prove unresolved identity and provider auth failures remain shared auth-envelope behavior
