---
seam_id: SEAM-1
seam_slug: adapter-selection-boundary
type: integration
status: exec-ready
execution_horizon: active
plan_version: v4
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - stable backend-id grammar changes
    - `llm.allowed_backends` evaluation order or deny-by-default semantics change
    - backend inventory filename-to-id matching changes
    - the adapter-visible `status --json` owner line changes
    - ADR-0041 path cleanup changes the cited authority set
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations:
  - REM-005
---

# SEAM-1 - Adapter selection boundary

- **Goal / value**:
  - Freeze the stable backend-id contract, trusted-input boundary, failure taxonomy, and published adapter-visible status boundary before downstream seams define internal adapter mechanics.
- **Scope**
  - In:
    - `contract.md`
    - `policy-spec.md`
    - the exact meaning of one backend id mapping to one adapter identity
    - allowlist gating before dispatch
    - invalid-selection versus dependency-unavailable versus policy-denied classification
    - the owner line for any additive adapter-visible `status --json` subset
    - the ban on trusting gateway-local config, admin, persistence, or session state for authorization
  - Out:
    - request/response payload shape
    - capability and extension-key subset
    - session-handle facets
    - event and trace handoff details beyond the selection boundary
    - Linux/macOS/Windows parity proof
- **Primary interfaces**
  - Inputs:
    - ADR-0041
    - `pre-planning/spec_manifest.md`
    - `pre-planning/impact_map.md`
    - ADR-0027 and the implemented config/policy pack
    - `docs/contracts/substrate-gateway-operator-contract.md`
    - `docs/contracts/substrate-gateway-status-schema.md`
    - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - Outputs:
    - `C-01`
    - `C-02`
- **Key invariants / rules**:
  - stable backend ids remain the only Substrate-facing backend identity
  - policy gating happens before adapter dispatch
  - one backend id does not split into planner, provider, router, wrapper, or auth-authority sub-identities
  - no secrets appear in backend identity fields or adapter selection surfaces
  - additive adapter-visible status metadata cannot widen the current external owner line without an explicit owner decision
  - this seam must not introduce a second Substrate control plane
- **Dependencies**
  - Direct blockers:
    - none inside this extracted pack; this seam is the prerequisite contract-definition seam
  - Transitive blockers:
    - stale ADR path references and archived ADR-0024 assumptions can still confuse downstream planning until this seam publishes the accepted owner line
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
  - Derived consumers:
    - gateway operator docs
    - backend inventory review
    - future adapter additions under the same `<kind>:<name>` contract
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/impact_map.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- **Verification**:
  - This seam produces owned contracts `C-01` and `C-02`.
  - At seam-brief depth, readiness means the stable backend-id semantics, ordered selection inputs, failure buckets, and `status --json` publication boundary are concrete enough for execution without inventing a second owner.
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md` is now the canonical `C-01` baseline.
  - `C-02` now resolves to a narrower v1 decision: no additive adapter-visible `status --json` field family is currently published beyond the existing `status` plus `client_wiring.*` schema, and any future additive family requires an explicit status-schema owner update before code changes.
  - With all three pre-exec gates passed, `basis.currentness: current`, and no blocking pre-exec remediations open, this seam is ready to execute while `REM-005` remains tracked as non-blocking cleanup.
  - Downstream seam-local review should verify that one selected backend id maps to one adapter identity, that the failure buckets stay fail-closed, and that any adapter-visible status subset stays inside a single explicit owner line.
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
- **Risks / unknowns**:
  - Risk:
    - a future attempt to publish adapter-visible `status --json` metadata could drift from the current v1 boundary if it widens the schema without an explicit status-schema owner update
  - De-risk plan:
    - keep the current v1 boundary explicit in the canonical status schema and require a schema-owner update before any additive field family ships
  - Risk:
    - stale `packs/active/...` references in ADR-0041 could leak incorrect authority paths into downstream docs
  - De-risk plan:
    - correct the path drift as part of the seam-local contract-definition bundle or record it as carried-forward evidence if an upstream ADR edit is intentionally deferred
- **Rollout / safety**:
  - this seam is safe to land first because it is contract-definition work that narrows trust boundaries without depending on runtime transport or platform-specific behavior
  - safety depends on keeping gateway-local control surfaces out of the authorization path
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `active` because every other seam depends on the stable backend-id contract, the failure taxonomy, and the publication boundary for adapter-visible status data
  - Which threads matter most
    - `THR-01`
  - What the first seam-local review should focus on
    - backend-id grammar
    - config/policy/input ordering
    - failure taxonomy
    - `status --json` owner line
    - non-trust boundary for gateway-local state
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-01`
    - `C-02`
  - Threads likely to advance:
    - `THR-01`
  - Review-surface areas likely to shift after landing:
    - backend selection flow
    - status publication boundary
    - operator-facing ownership wording
  - Downstream seams most likely to require revalidation:
    - `SEAM-2`
    - `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
