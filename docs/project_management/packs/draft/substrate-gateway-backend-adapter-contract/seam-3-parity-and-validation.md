---
seam_id: SEAM-3
seam_slug: parity-and-validation
type: conformance
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - ./governance/seam-1-closeout.md
    - ./governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - backend-id or published status subset semantics change upstream
    - adapter protocol/schema publication changes upstream
    - Linux/macOS/Windows guarantee wording changes
    - ADR-0024 supersession posture changes
    - ADR-0040 alignment stops being evidence-only or widens into direct touch surfaces
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
open_remediations: []
---

# SEAM-3 - Parity and validation

- **Goal / value**:
  - Prove the adapter contract is additive, cross-platform, and compatible with the already accepted ownership split once the upstream contract seams have published concrete truth.
- **Scope**
  - In:
    - `platform-parity-spec.md`
    - `compatibility-spec.md`
    - `manual_testing_playbook.md`
    - `pre-planning/ci_checkpoint_plan.md`
    - Linux/macOS/Windows guarantee matrix
    - ADR-0024 historical-evidence-only supersession proof
    - ADR-0040 alignment posture for runtime ownership evidence
    - deterministic validation gates for document review, ambiguity scan, and platform proof
  - Out:
    - backend-id selection contract definition
    - adapter lifecycle or schema definition
    - widening the operator contract or status schema beyond accepted upstream truth
    - introducing new runtime ownership that ADR-0040 does not already allow
- **Primary interfaces**
  - Inputs:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`
    - `docs/contracts/substrate-gateway-runtime-parity.md`
    - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
    - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - Outputs:
    - cross-platform validation evidence
    - compatibility and supersession proof
    - pack-closeout-ready drift guards for future gateway adapter work
- **Key invariants / rules**:
  - Linux, macOS, and Windows must preserve the same stable backend-id and allowlist semantics
  - hidden transport or bootstrap divergence may exist, but it must stay out of the contract unless explicitly published
  - ADR-0024 stays historical evidence only once this seam publishes compatibility proof
  - this seam must confirm that no second Substrate control plane has been introduced
  - manual validation assertions must stay downstream of the already published upstream contracts
- **Dependencies**
  - Direct blockers:
    - none at pre-exec; `SEAM-2` has published the adapter protocol and schema boundary and ADR-0040 remains settled basis evidence
  - Transitive blockers:
    - platform parity proof may expand if the upstream protocol/schema seam widens the runtime blast radius
  - Direct consumers:
    - pack closeout
  - Derived consumers:
    - future gateway adapter features
    - release validation and doc alignment
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/platform-parity-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/compatibility-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/ci_checkpoint_plan.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
- **Verification**:
  - This seam consumes upstream contracts `C-01` through `C-04`, so verification may depend on accepted upstream evidence for the landed selection, publication, protocol, and schema contracts.
  - At seam-brief depth, readiness is that the parity matrix, compatibility proof, and validation assertions are concrete enough for seam-local planning and implementation.
  - Downstream seam-local review should prove that cross-platform guarantees remain compatible with ADR-0040 and that the compatibility proof keeps ADR-0024 historical rather than active.
  - ADR-0040 is now explicitly confirmed as evidence-only basis for this seam:
    - ADR-0040 remains the owner of the Substrate versus `substrate-gateway` runtime boundary.
    - ADR-0041 and `docs/contracts/substrate-gateway-runtime-parity.md` carry the downstream consequences this seam must prove.
    - direct ADR-0040 edits stay out of scope unless landing evidence discovers a concrete runtime-ownership drift.
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
- **Risks / unknowns**:
  - Risk:
    - landing evidence could still expose a concrete runtime-ownership drift that ADR-0040 and the current runtime-parity contract no longer explain
  - De-risk plan:
    - reopen ADR-0040 alignment only if parity or compatibility proof surfaces a concrete mismatch during landing or closeout
  - Risk:
    - platform proof could accidentally restate or widen upstream contracts instead of validating them
  - De-risk plan:
    - keep this seam conformance-scoped and require upstream contract refs for every validation assertion
- **Rollout / safety**:
  - this seam lands last because it should validate accepted upstream truth rather than help invent it
  - safety depends on keeping compatibility proof additive and refusing any hidden second control plane
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `active` because `SEAM-2` is now closed and `THR-02` is published, so parity and compatibility planning can consume landed upstream truth
  - Which threads matter most
    - `THR-01`
    - `THR-02`
  - What the first seam-local review should focus on
    - Linux/macOS/Windows guarantee matrix
    - ADR-0024 supersession proof
    - ADR-0040 alignment decision
    - manual validation assertions and checkpoint boundary
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - none beyond the accepted proof surfaces and any durable parity/compatibility mirrors chosen during seam-local planning
  - Threads likely to advance:
    - `THR-01`
    - `THR-02` through revalidation rather than new contract publication
  - Review-surface areas likely to shift after landing:
    - cross-platform proof matrix
    - compatibility narrative
    - validation gate wording
  - Downstream seams most likely to require revalidation:
    - none inside this pack; follow-on gateway adapter features should consume the resulting closeout evidence
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
