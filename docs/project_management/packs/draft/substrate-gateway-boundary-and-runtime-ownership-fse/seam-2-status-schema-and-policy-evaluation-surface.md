---
seam_id: SEAM-2
seam_slug: status-schema-and-policy-evaluation-surface
type: integration
status: proposed
execution_horizon: next
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - `status --json` top-level shape changes
    - `client_wiring.*` family or absence semantics change
    - ADR-0042 additive metadata boundary changes
    - fail-closed placement or secret-delivery trust-boundary rules change
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
open_remediations: []
---

# SEAM-2 - Status schema and policy evaluation surface

- **Goal / value**:
  - Publish one authoritative inventory seam for machine-readable status/wiring and fail-closed policy evaluation so runtime and docs can consume a single truth surface.
  - Prevent status-schema drift, trust-boundary drift, and ADR-0042 metadata creep before runtime transport is planned deeply.
- **Scope**
  - In:
    - `substrate world gateway status --json` top-level object shape
    - `client_wiring.*` field family
    - non-secret output guarantees and absence semantics
    - gateway-integration decision flow over existing ADR-0027 keys
    - fail-closed no-host-fallback rule and host-to-world secret delivery boundary
    - the ban on trusting gateway-local config, admin, or persistence surfaces as policy inputs
  - Out:
    - command spelling and ownership-table wording
    - typed world-agent lifecycle/status endpoint shape
    - provisioning changes
    - manual validation/playbook lock-in and final docs alignment
- **Primary interfaces**
  - Inputs:
    - `C-01`
    - ADR-0027 config/policy contract and schema ownership
    - ADR-0042 additive metadata boundary
    - `pre-planning/spec_manifest.md`
    - `pre-planning/impact_map.md`
  - Outputs:
    - `C-02` for the status JSON envelope and `client_wiring.*`
    - `C-03` for policy evaluation, fail-closed placement, and trust-boundary rules
    - downstream-ready inputs for typed runtime planning and docs validation
- **Key invariants / rules**:
  - `status --json` remains the authoritative Substrate-owned machine-readable wiring surface
  - `client_wiring.*` is the locked field family for endpoint discovery in this pack
  - additive identity-tuple or placement-posture metadata outside `client_wiring.*` remains ADR-0042-owned
  - reused ADR-0027 keys stay externally owned; this seam only defines evaluation over them
  - when in-world execution is required, host-level gateway fallback is not authorized
  - gateway-local config/admin/persistence cannot become trusted policy inputs
- **Dependencies**
  - Direct blockers:
    - `SEAM-1`
    - `THR-01`
  - Transitive blockers:
    - stale link corrections in ADR-0040 and adjacent docs may still be needed to keep downstream schema/policy docs aligned
  - Direct consumers:
    - `SEAM-3`
    - `SEAM-4`
  - Derived consumers:
    - shared agent API crates
    - operator docs and tests
    - quality-gate evidence
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/policy-spec.md`
  - `crates/agent-api-types/src/lib.rs`
  - `crates/agent-api-client/src/lib.rs`
  - `docs/CONFIGURATION.md`
- **Verification**:
  - This seam consumes upstream contract `C-01`; verification may depend on accepted upstream evidence for the command family and ownership boundary.
  - This seam produces owned contracts `C-02` and `C-03`. At seam-brief depth, verification is those contracts becoming concrete enough for seam-local planning and implementation: field families, absence semantics, decision tables, fail-closed posture, and trust boundaries.
  - Later seam-local verification should prove:
    - the JSON envelope and `client_wiring.*` family are explicit and non-secret
    - absence semantics and omitted fields are deterministic
    - ADR-0042 additive metadata stays out of the owned field family
    - invalid integration state, dependency unavailability, and policy denial are distinguished cleanly
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
- **Risks / unknowns**:
  - Risk:
    - the machine-readable status shape can drift away from the operator contract or from later shared client types
  - De-risk plan:
    - keep the envelope, `client_wiring.*` family, and omission rules single-source before runtime transport planning begins
  - Risk:
    - ADR-0042 metadata can expand into the owned wiring family
  - De-risk plan:
    - make the boundary against additive metadata a first-class seam-local review check
  - Risk:
    - policy evaluation can accidentally depend on gateway-local admin or persistence surfaces
  - De-risk plan:
    - keep the non-trust rule explicit and verify no host fallback path appears when policy requires in-world execution
- **Rollout / safety**:
  - This seam should land before typed runtime work so runtime consumers inherit a stable status/policy contract instead of inventing one.
  - Safety depends on failing closed and preserving host-secret and gateway-local non-trust boundaries.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `next` because it is the immediate downstream consumer of the operator boundary and the last seam that should receive provisional deeper planning before future seams stay brief-only.
  - Which threads matter most
    - `THR-01`
    - `THR-02`
    - `THR-03`
  - What the first seam-local review should focus on
    - top-level JSON shape and conditional presence rules
    - `client_wiring.*` lock
    - ADR-0042 boundary
    - fail-closed decision taxonomy
    - host-to-world secret boundary
    - whether a reserved `S00` contract-definition slice is needed
  - Later next-seam planning posture
    - the remaining deeper planning is likely spike-grade around exact field tables and decision tables rather than a broader seam re-split
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-02`
    - `C-03`
  - Threads likely to advance:
    - `THR-02`
    - `THR-03`
  - Review-surface areas likely to shift after landing:
    - status-schema diagram
    - policy/placement flow
    - wiring discovery wording
  - Downstream seams most likely to require revalidation:
    - `SEAM-3`
    - `SEAM-4`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
