---
slice_id: S2
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - auth precedence or fail-closed policy semantics change after `S00`
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-02
contracts_consumed:
  - C-02
open_remediations:
  - REM-001
---
### S2 - Land policy precedence and fail-closed boundaries at the shell boundary

- **User/system value**:
  - Prevents auth sourcing and policy denial behavior from remaining a Codex-only side effect by making the shell boundary obey one explicit canonical rule.
- **Scope (in/out)**:
  - In: env-versus-file precedence, governed input usage, no-host-fallback behavior, trusted-input boundaries, and policy-denial classification
  - Out: adapter auth payload schema, runtime delivery model, or managed artifact semantics
- **Acceptance criteria**:
  - one explicit precedence rule governs allowlisted env material versus allowlisted host credential files
  - in-world requirements remain fail-closed when the boundary is unavailable
  - gateway-local config, admin, and persistence stay outside trusted policy inputs
- **Dependencies**:
  - `S00`
  - `THR-01`
  - `C-02`
- **Verification**:
  - targeted shell tests and doc review for env-only, file-only, partial env, and deny-by-policy paths
- **Rollout/safety**:
  - do not widen auth-source authority or weaken no-host-fallback guarantees
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`

#### S2.T1 - Codify precedence instead of inheriting it accidentally

- **Outcome**:
  - the canonical policy doc and shell auth-resolution logic agree on one precedence rule for the integrated lifecycle.
- **Inputs/outputs**:
  - Inputs: `docs/contracts/substrate-gateway-policy-evaluation.md`, `crates/shell/src/builtins/world_gateway.rs`, supporting ADR-0046 policy/env-var docs
  - Outputs: aligned policy text and shell-side auth-source handling
- **Thread/contract refs**:
  - `THR-01`
  - `C-02`
- **Implementation notes**:
  - handle incomplete env material explicitly
  - keep the precedence decision attached to allowlisted sources only
- **Acceptance criteria**:
  - env-only, file-only, and partial-env cases cannot yield contradictory behavior between docs and code
- **Test notes**:
  - add or update cases covering env token present, env account id without token, and allowed file fallback
- **Risk/rollback notes**:
  - leaving the precedence rule half-documented will keep `REM-001` open even if code changes land

Checklist:
- Implement:
  - align the policy contract and shell auth-resolution logic to one precedence rule
- Test:
  - cover precedence and incomplete-env failure cases
- Validate:
  - confirm downstream seams consume one stable upstream auth rule

#### S2.T2 - Preserve fail-closed and trusted-input boundaries

- **Outcome**:
  - policy gating remains explicitly Substrate-owned and does not drift toward gateway-local state or host fallback shortcuts.
- **Inputs/outputs**:
  - Inputs: canonical `C-02`, shell gateway mode checks, supporting ADR-0046 docs
  - Outputs: aligned fail-closed behavior and trusted-input documentation
- **Thread/contract refs**:
  - `THR-01`
  - `C-02`
- **Implementation notes**:
  - keep invalid integration, dependency unavailable, and policy denial distinct
  - do not let gateway-local persistence participate in authorization truth
- **Acceptance criteria**:
  - no-host-fallback rules and trusted-input boundaries remain explicit in both docs and shell behavior
- **Test notes**:
  - exercise required in-world mode and unavailable boundary cases
- **Risk/rollback notes**:
  - any silent fallback or gateway-local trust expansion undermines the seam's main safety objective

Checklist:
- Implement:
  - align fail-closed and trusted-input rules across canonical docs and shell behavior
- Test:
  - verify policy-denied and boundary-unavailable paths stay distinct
- Validate:
  - confirm `SEAM-2` inherits a stable policy boundary rather than a best-effort fallback story
