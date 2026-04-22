---
seam_id: SEAM-1
seam_slug: backend-selection-and-policy-surface
type: integration
status: proposed
execution_horizon: active
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - backend inventory roots or filename rules publish outside this seam
    - auth precedence becomes explicit in a contract or code path outside this seam
    - selection taxonomy drifts between ADR-0046, code, and external contract docs
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations:
  - REM-001
  - REM-002
---

# SEAM-1 - Backend selection and policy surface

- **Goal / value**:
  - Freeze the integrated lifecycle truth for backend selection and policy evaluation before downstream runtime work proceeds.
  - Turn the current Codex-only implementation evidence into an explicit contract boundary rather than leaving selection, precedence, and inventory semantics implied by one code path.
- **Scope**
  - In:
    - selected backend resolution from existing config and policy
    - deny-by-default backend allowlisting
    - backend-id grammar and trusted-input boundary
    - backend inventory lookup roots and filename/id rules
    - auth-source precedence between env material and host credential files
    - policy distinction between invalid integration, policy denial, and dependency unavailable at the selection boundary
  - Out:
    - integrated adapter binding metadata and capability gates
    - adapter-specific auth payload schemas
    - runtime config rendering and artifact naming
    - parity validation and rollout proof
    - tuple metadata, tuple-policy keys, or `status --json` widening
- **Primary interfaces**
  - Inputs:
    - ADR-0046 goals and non-goals
    - current shell-side request construction and policy gating in `crates/shell/src/builtins/world_gateway.rs`
    - external authorities in `docs/contracts/substrate-gateway-backend-adapter-selection.md` and `docs/contracts/substrate-gateway-policy-evaluation.md`
  - Outputs:
    - one landed ADR-0046 delta in `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md`
    - one landed ADR-0046 delta in `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md`
    - one landed ADR-0046 delta in `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md`
    - downstream-ready selection/policy truth carried by `THR-01`
- **Key invariants / rules**:
  - backend ids remain stable `<kind>:<name>` selectors only
  - gateway-local config, admin mutation, and persistence are not trusted authorization inputs
  - selection must stay on existing ADR-0027 config/policy roots
  - this seam must not widen `status --json` or pull ADR-0042/0043 surfaces into scope
  - selection and policy truth must be explicit enough that downstream runtime planning does not infer taxonomy from hardcoded Codex branches
- **Dependencies**
  - Direct blockers:
    - none inside the pack
  - Transitive blockers:
    - external authorities in ADR-0040/0041 and `docs/contracts/*` must remain evidence-only, not silently overwritten by local planning prose
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
  - Derived consumers:
    - shell gateway requests
    - policy review
    - validation and rollout artifacts
- **Touch surface**:
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
- **Verification**:
  - This seam produces owned contracts `C-01` and `C-02` through the feature-local ADR-0046 docs `contract.md`, `policy-spec.md`, and `env-vars-spec.md`. Verification at seam-brief depth is those feature-local deltas becoming concrete enough for seam-local planning and implementation: exact selection order, inventory discoverability, allowlist order, auth precedence, and failure-taxonomy boundaries.
  - The consumed external authorities under `docs/contracts/*` remain evidence and compatibility checks; verification here does not require editing them.
  - Later seam-local verification should prove:
    - selection remains on existing config/policy roots
    - deny-by-default allowlisting happens before adapter dispatch
    - backend inventory roots and filename rules are explicit
    - auth precedence and policy gates are explicit rather than Codex-only side effects
- **Canonical contract refs**:
  - Owned feature-local outputs:
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md`
  - Consumed external authorities:
    - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
    - `docs/contracts/substrate-gateway-policy-evaluation.md`
- **Risks / unknowns**:
  - Risk:
    - current code only proves a Codex-specific auth precedence path, not a general integrated contract
  - De-risk plan:
    - publish explicit precedence and selection rules before downstream runtime protocol/schema work starts
  - Risk:
    - missing backend inventory roots or filename rules will force runtime work to invent filesystem semantics locally
  - De-risk plan:
    - treat discoverability and filename invariants as first-class active-seam contract work
  - Risk:
    - adjacent docs may tempt downstream work to widen tuple or status surfaces here
  - De-risk plan:
    - keep ADR-0042/0043 and `status --json` widening explicit out-of-scope checks in seam-local review
- **Rollout / safety**:
  - This seam is the safest active starting point because it fixes trusted-input and failure-taxonomy boundaries before runtime expansion.
  - Safety depends on failing closed and preventing gateway-local state from becoming authorization truth.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `active` because its unresolved authority questions block every downstream seam.
  - Which threads matter most
    - `THR-01`
  - What the first seam-local review should focus on
    - backend selection order
    - allowlist timing
    - inventory roots and filename rules
    - auth precedence
    - out-of-scope checks for tuple/status widening
    - whether `S00` is needed to freeze the contract before implementation slices begin
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-01` via `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md`
    - `C-02` via `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md` and `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md`
  - Threads likely to advance:
    - `THR-01`
  - Review-surface areas likely to shift after landing:
    - selected-backend flow
    - auth-source diagram
    - failure-taxonomy wording
  - Downstream seams most likely to require revalidation:
    - `SEAM-2`
    - `SEAM-3`
  - Seam exit should record the landed feature-local outputs and their compatibility against consumed external authorities; it does not require editing the external authorities themselves.
