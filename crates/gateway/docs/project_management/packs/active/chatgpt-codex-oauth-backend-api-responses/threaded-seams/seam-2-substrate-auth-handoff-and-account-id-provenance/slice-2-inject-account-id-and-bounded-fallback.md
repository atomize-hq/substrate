---
slice_id: S2
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the route contract changes the minimal header contract, header casing expectations, or provider request-builder shape
    - auth-context resolution changes in a way that alters account-id precedence
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
### S2 - Inject Account Id And Bounded Fallback

- **User/system value**: the provider sends the right `ChatGPT-Account-ID` header without taking over auth ownership.
- **Scope (in/out)**:
  - In: inject `ChatGPT-Account-ID` from resolved auth context, keep JWT parsing as bounded fallback, and fail before upstream when identity cannot be resolved.
  - Out: route contract shaping, seam-exit publication, and broad OAuth UX changes.
- **Acceptance criteria**:
  - provider request builder injects `ChatGPT-Account-ID` from resolved auth context first
  - JWT-derived fallback remains bounded and explicit
  - unresolved identity returns the normal failure envelope before the upstream call
- **Dependencies**: `S00`, `S1`, `crates/gateway/src/providers/openai.rs`, `THR-15`
- **Verification**:
  - positive tests prove header injection from resolved auth context
  - negative tests prove fallback does not redefine ownership
- **Rollout/safety**: keep the provider path as a consumer of resolved auth context, not a host-credential owner.
- **Review surface refs**: `../../review_surfaces.md` (`R3`) and `review.md`
