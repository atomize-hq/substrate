---
slice_id: S1
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the route contract changes the minimal header contract or the exact auth-context fields the provider path consumes
    - Substrate delivery posture changes for auth bundles, secret-channel transport, or in-world consumption
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-14
  - THR-15
contracts_produced: []
contracts_consumed:
  - C-14
  - C-15
open_remediations: []
---
### S1 - Resolve Auth Context And Owner Line

- **User/system value**: the gateway has one explicit auth-resolution path instead of a loose mix of token parsing and ad hoc fallback logic.
- **Scope (in/out)**:
  - In: resolve Substrate-delivered auth context, preserve explicit `account_id` precedence, and keep gateway-local persistence out of the integrated trust boundary.
  - Out: provider header emission, conformance coverage, and seam-exit publication accounting.
- **Acceptance criteria**:
  - integrated-mode auth context can be resolved without host-local auth-file reads
  - explicit `account_id` remains the first source of truth
  - fallback behavior stays bounded and reviewable
- **Dependencies**: `S00`, `crates/gateway/src/auth/*`, `crates/gateway/src/server/oauth_handlers.rs`, `THR-15`
- **Verification**:
  - positive checks prove integrated and standalone mode are distinguishable
  - negative checks prove unresolved identity fails before the upstream call
- **Rollout/safety**: keep owner-line logic below the provider boundary and out of public ingress.
- **Review surface refs**: `../../review_surfaces.md` (`R3`) and `review.md`
