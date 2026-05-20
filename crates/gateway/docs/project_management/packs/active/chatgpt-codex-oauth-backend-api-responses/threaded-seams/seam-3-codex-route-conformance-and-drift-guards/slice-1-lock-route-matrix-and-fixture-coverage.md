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
  - THR-16
contracts_produced: []
contracts_consumed:
  - C-14
  - C-16
open_remediations: []
---
### S1 - Lock Route Matrix And Fixture Coverage

- **User/system value**: the route matrix and semantic SSE expectations become durable regressions instead of best-effort knowledge.
- **Scope (in/out)**:
  - In: deterministic sync and streaming coverage for accepted and rejected controls, fixture namespaces, and semantic stream assembly.
  - Out: auth ownership implementation changes and seam-exit publication.
- **Acceptance criteria**:
  - route-local positive and negative cases are concrete in deterministic regressions
  - sync and stream assertions consume the same semantic route truth
  - unsupported controls still fail explicitly rather than degrading silently
- **Dependencies**: `S00`, `crates/gateway/tests/openai_responses_conformance.rs`, `crates/gateway/src/server/openai_conformance_test_support.rs`
- **Verification**:
  - positive tests prove the supported route matrix and fixture-backed sync/stream parity
  - negative tests prove rejected controls and drift-sensitive cases fail deterministically
